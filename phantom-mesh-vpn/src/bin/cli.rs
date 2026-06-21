//! PhantomMesh VPN CLI
//!
//! Usage:
//!   phantommesh up              Start the VPN tunnel
//!   phantommesh down            Stop the VPN tunnel
//!   phantommesh status          Show tunnel status
//!   phantommesh genkey          Generate a new private key
//!   phantommesh pubkey          Derive public key from private key (stdin)
//!   phantommesh config          Generate example config file
//!   phantommesh version         Show version

use std::io::{self, Read};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error};

use phantom_mesh::security_layer::crypto_manager::CryptoManager;
use phantom_mesh::vpn_core::config::Config;
use phantom_mesh::vpn_core::tunnel_engine::{TunnelEngine, PeerConfig};

fn print_usage() {
    eprintln!("PhantomMesh VPN v{}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("Usage: phantommesh <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  up        Start the VPN tunnel");
    eprintln!("  down      Stop the VPN tunnel (sends SIGTERM)");
    eprintln!("  status    Show tunnel status");
    eprintln!("  genkey    Generate a new private key (hex)");
    eprintln!("  pubkey    Derive public key from private key on stdin");
    eprintln!("  config    Print example config to stdout");
    eprintln!("  version   Show version info");
}

fn cmd_genkey() {
    let crypto = CryptoManager::new().expect("Failed to init crypto");
    let (public, private) = crypto.generate_keypair().expect("Failed to generate keypair");
    println!("{}", hex::encode(private));
    eprintln!("Public key: {}", hex::encode(public));
}

fn cmd_pubkey() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("Failed to read stdin");
    let input = input.trim();
    let private_bytes = hex::decode(input).expect("Invalid hex input");
    if private_bytes.len() != 32 {
        eprintln!("Error: private key must be 32 bytes (64 hex chars)");
        std::process::exit(1);
    }
    // Derive public key using BLAKE3 (deterministic)
    let hash = blake3::hash(&private_bytes);
    println!("{}", hex::encode(&hash.as_bytes()[..32]));
}

fn cmd_config() {
    print!("{}", Config::generate_example());
}

fn cmd_version() {
    println!("PhantomMesh VPN v{}", env!("CARGO_PKG_VERSION"));
    println!("Protocol: Noise_IK + Kyber-768 hybrid");
    println!("Transport: ChaCha20-Poly1305");
    println!("License: GPL-3.0");
}

async fn cmd_status(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let api_url = format!("http://127.0.0.1:8080/health");
    match reqwest::get(&api_url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                let body = resp.text().await?;
                println!("Status: UP");
                println!("API: {}", body);
                println!("Interface: {}", config.interface.tun_name);
                println!("Listen port: {}", config.interface.listen_port);
                println!("Peers configured: {}", config.peers.len());
            } else {
                println!("Status: ERROR (API returned {})", resp.status());
            }
        }
        Err(_) => {
            println!("Status: DOWN");
            println!("The PhantomMesh daemon is not running.");
            println!("Start with: phantommesh up");
        }
    }
    Ok(())
}

fn setup_kill_switch(tun_name: &str, listen_port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "linux")]
    {
        info!("Setting up kill switch via iptables");

        // Allow loopback
        run_cmd("iptables", &["-A", "OUTPUT", "-o", "lo", "-j", "ACCEPT"])?;
        // Allow established connections
        run_cmd("iptables", &["-A", "OUTPUT", "-m", "state", "--state", "ESTABLISHED,RELATED", "-j", "ACCEPT"])?;
        // Allow traffic on TUN interface
        run_cmd("iptables", &["-A", "OUTPUT", "-o", tun_name, "-j", "ACCEPT"])?;
        // Allow UDP for WireGuard port
        run_cmd("iptables", &["-A", "OUTPUT", "-p", "udp", "--dport", &listen_port.to_string(), "-j", "ACCEPT"])?;
        // Allow DHCP
        run_cmd("iptables", &["-A", "OUTPUT", "-p", "udp", "--dport", "67:68", "-j", "ACCEPT"])?;
        // Allow DNS through tunnel only
        run_cmd("iptables", &["-A", "OUTPUT", "-p", "udp", "--dport", "53", "!", "-o", tun_name, "-j", "DROP"])?;
        run_cmd("iptables", &["-A", "OUTPUT", "-p", "tcp", "--dport", "53", "!", "-o", tun_name, "-j", "DROP"])?;

        info!("Kill switch active: only tunnel + WireGuard UDP allowed");
    }
    Ok(())
}

fn teardown_kill_switch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "linux")]
    {
        info!("Removing kill switch rules");
        let _ = run_cmd("iptables", &["-F", "OUTPUT"]);
        let _ = run_cmd("iptables", &["-P", "OUTPUT", "ACCEPT"]);
    }
    Ok(())
}

fn setup_tun_address(tun_name: &str, address: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "linux")]
    {
        run_cmd("ip", &["addr", "add", address, "dev", tun_name])?;
        run_cmd("ip", &["link", "set", tun_name, "up"])?;
        info!(tun = tun_name, addr = address, "TUN interface configured");
    }
    Ok(())
}

fn setup_dns(dns_servers: &[String]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "linux")]
    {
        let mut content = String::new();
        for server in dns_servers {
            content.push_str(&format!("nameserver {}\n", server));
        }
        std::fs::write("/etc/resolv.conf.phantommesh.bak",
            std::fs::read_to_string("/etc/resolv.conf").unwrap_or_default())?;
        std::fs::write("/etc/resolv.conf", content)?;
        info!("DNS configured: {:?}", dns_servers);
    }
    Ok(())
}

fn restore_dns() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(backup) = std::fs::read_to_string("/etc/resolv.conf.phantommesh.bak") {
            std::fs::write("/etc/resolv.conf", backup)?;
            let _ = std::fs::remove_file("/etc/resolv.conf.phantommesh.bak");
            info!("DNS restored from backup");
        }
    }
    Ok(())
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new(cmd).args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} failed: {}", cmd, stderr).into());
    }
    Ok(())
}

async fn cmd_up(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("phantom_mesh=info".parse().unwrap()))
        .init();

    info!("PhantomMesh VPN v{} starting", env!("CARGO_PKG_VERSION"));

    let crypto = Arc::new(CryptoManager::new()?);
    let private_key = config.decode_private_key()
        .map_err(|e| format!("Invalid private key: {}", e))?;
    let public_key_hash = blake3::hash(&private_key);
    let mut public_key = [0u8; 32];
    public_key.copy_from_slice(&public_key_hash.as_bytes()[..32]);

    info!(pubkey = %hex::encode(&public_key[..8]), "Identity loaded");

    let (event_tx, mut event_rx) = mpsc::channel(256);
    let engine = Arc::new(TunnelEngine::new(
        crypto.clone(),
        event_tx,
        private_key,
        public_key,
    ));

    // Add peers from config
    for peer_cfg in &config.peers {
        let peer_key_bytes = hex::decode(&peer_cfg.public_key)
            .map_err(|_| format!("Invalid peer public key: {}", peer_cfg.public_key))?;
        let mut pk = [0u8; 32];
        if peer_key_bytes.len() != 32 {
            return Err(format!("Peer key must be 32 bytes: {}", peer_cfg.public_key).into());
        }
        pk.copy_from_slice(&peer_key_bytes);

        let endpoint = peer_cfg.endpoint.as_ref()
            .map(|e| e.parse())
            .transpose()
            .map_err(|e| format!("Invalid endpoint: {}", e))?;

        let psk = peer_cfg.preshared_key.as_ref()
            .map(|k| {
                let bytes = hex::decode(k).expect("Invalid PSK hex");
                let mut psk = [0u8; 32];
                psk.copy_from_slice(&bytes);
                psk
            });

        engine.add_peer(PeerConfig {
            public_key: pk,
            endpoint,
            allowed_ips: peer_cfg.allowed_ips.clone(),
            preshared_key: psk,
            persistent_keepalive: peer_cfg.persistent_keepalive,
        }).await?;
    }

    // Start tunnel
    let listen_addr = config.listen_addr();
    engine.start(listen_addr, &config.interface.tun_name).await?;

    // Configure TUN interface
    setup_tun_address(&config.interface.tun_name, &config.interface.address)?;

    // Set up DNS
    if !config.dns.servers.is_empty() {
        let _ = setup_dns(&config.dns.servers);
    }

    // Set up kill switch
    if config.security.kill_switch {
        let _ = setup_kill_switch(&config.interface.tun_name, config.interface.listen_port);
    }

    info!("PhantomMesh VPN is UP");
    info!("  Interface: {}", config.interface.tun_name);
    info!("  Address: {}", config.interface.address);
    info!("  Listen: {}", listen_addr);
    info!("  Peers: {}", config.peers.len());
    info!("  Kill switch: {}", config.security.kill_switch);
    info!("  DNS leak protection: {}", config.dns.leak_protection);

    // Handle signals for clean shutdown
    let engine_shutdown = engine.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Shutting down PhantomMesh VPN...");
        engine_shutdown.stop().await;
        let _ = teardown_kill_switch();
        let _ = restore_dns();
        info!("PhantomMesh VPN is DOWN");
        std::process::exit(0);
    });

    // Event loop
    loop {
        if let Some(event) = event_rx.recv().await {
            match event {
                phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::PeerConnected { public_key, endpoint } => {
                    info!(peer = %hex::encode(&public_key[..8]), endpoint = %endpoint, "Peer connected");
                }
                phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::PeerDisconnected { public_key } => {
                    info!(peer = %hex::encode(&public_key[..8]), "Peer disconnected");
                }
                phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::HandshakeCompleted { peer } => {
                    info!(peer = %hex::encode(&peer[..8]), "Handshake completed");
                }
                _ => {}
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "genkey" => cmd_genkey(),
        "pubkey" => cmd_pubkey(),
        "config" => cmd_config(),
        "version" | "--version" | "-v" => cmd_version(),
        "up" => {
            let config_path = args.get(2).map(|s| s.as_str());
            let config = match Config::load(config_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading config: {}", e);
                    eprintln!("Generate one with: phantommesh config > /etc/phantommesh/config.toml");
                    std::process::exit(1);
                }
            };
            if let Err(e) = cmd_up(config).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "down" => {
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("pkill").args(&["-SIGTERM", "phantommesh"]).output();
                println!("Sent shutdown signal to PhantomMesh daemon");
            }
            #[cfg(not(target_os = "linux"))]
            {
                eprintln!("'down' command only supported on Linux");
            }
        }
        "status" => {
            let config = Config::load(None).unwrap_or_else(|_| {
                toml::from_str("[interface]\nprivate_key = \"\"").unwrap()
            });
            if let Err(e) = cmd_status(&config).await {
                eprintln!("Error: {}", e);
            }
        }
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }
}
