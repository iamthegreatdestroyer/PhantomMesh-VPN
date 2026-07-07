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
use tracing::{error, info, warn};

use phantom_mesh::mesh::healer::MeshHealer;
use phantom_mesh::security_layer::crypto_manager::CryptoManager;
use phantom_mesh::security_layer::handshake::NodeIdentity;
use phantom_mesh::security_layer::threat_engine::ThreatEngine;
use phantom_mesh::vpn_core::api_gateway::ApiGateway;
use phantom_mesh::vpn_core::config::Config;
use phantom_mesh::vpn_core::tunnel_engine::{TunnelEngine, PeerConfig};
use phantom_mesh::metrics::{init_metrics, update_system_metrics};

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
    // Real X25519 keypair (CryptoManager::generate_keypair() was deleted in
    // the Stage 1 crypto fix — it produced two independent random values
    // with no cryptographic relationship between them, i.e. was not usable
    // as a real keypair at all).
    let identity = NodeIdentity::generate().expect("Failed to generate keypair");
    println!("{}", hex::encode(identity.x25519_private));
    eprintln!("Public key: {}", hex::encode(identity.x25519_public));
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
    // Derive public key via real X25519 (private_bytes is exactly 32 bytes, checked above).
    let mut private_key = [0u8; 32];
    private_key.copy_from_slice(&private_bytes);
    let public_key = x25519_dalek::x25519(private_key, x25519_dalek::X25519_BASEPOINT_BYTES);
    println!("{}", hex::encode(public_key));
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
    let api_url = "http://127.0.0.1:8080/health".to_string();
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

        // IPv6 parity: mirror every IPv4 rule above via ip6tables. This is
        // best-effort/non-fatal — hosts with no IPv6 support (ip6tables
        // missing, or the kernel module not loaded) must not fail `up`
        // entirely over this, since IPv4-only kill-switch protection is
        // still strictly better than none. Without this, IPv6 traffic
        // would bypass the kill switch completely on any dual-stack host.
        setup_kill_switch_v6(tun_name, listen_port);
    }
    Ok(())
}

/// IPv6 mirror of `setup_kill_switch`'s iptables rules, via ip6tables.
/// Every failure (missing binary, no ip6 support, etc.) is logged and
/// swallowed rather than propagated — see `run_cmd_best_effort`.
#[cfg(target_os = "linux")]
fn setup_kill_switch_v6(tun_name: &str, listen_port: u16) {
    if !run_cmd_best_effort("ip6tables", &["-A", "OUTPUT", "-o", "lo", "-j", "ACCEPT"]) {
        info!("ip6tables unavailable or failed; continuing with IPv4-only kill switch");
        return;
    }
    run_cmd_best_effort("ip6tables", &["-A", "OUTPUT", "-m", "state", "--state", "ESTABLISHED,RELATED", "-j", "ACCEPT"]);
    run_cmd_best_effort("ip6tables", &["-A", "OUTPUT", "-o", tun_name, "-j", "ACCEPT"]);
    run_cmd_best_effort("ip6tables", &["-A", "OUTPUT", "-p", "udp", "--dport", &listen_port.to_string(), "-j", "ACCEPT"]);
    run_cmd_best_effort("ip6tables", &["-A", "OUTPUT", "-p", "udp", "--dport", "546:547", "-j", "ACCEPT"]); // DHCPv6 (v6 equivalent of the v4 67:68 rule)
    run_cmd_best_effort("ip6tables", &["-A", "OUTPUT", "-p", "udp", "--dport", "53", "!", "-o", tun_name, "-j", "DROP"]);
    run_cmd_best_effort("ip6tables", &["-A", "OUTPUT", "-p", "tcp", "--dport", "53", "!", "-o", tun_name, "-j", "DROP"]);

    info!("IPv6 kill switch active (mirrors IPv4 rules)");
}

fn teardown_kill_switch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "linux")]
    {
        info!("Removing kill switch rules");
        let _ = run_cmd("iptables", &["-F", "OUTPUT"]);
        let _ = run_cmd("iptables", &["-P", "OUTPUT", "ACCEPT"]);

        // IPv6 teardown mirrors IPv4, same best-effort semantics: a host
        // with no IPv6 support should not error out of the shutdown path.
        run_cmd_best_effort("ip6tables", &["-F", "OUTPUT"]);
        run_cmd_best_effort("ip6tables", &["-P", "OUTPUT", "ACCEPT"]);
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

/// Like `run_cmd`, but never propagates an error: logs and returns `false`
/// on any failure (binary missing, non-zero exit, etc.) instead. Used for
/// the IPv6 kill-switch mirror, which must be best-effort — a host with no
/// IPv6 support (no `ip6tables` binary, module not loaded, ...) must not
/// fail `up` entirely just because IPv6-specific tooling isn't present.
fn run_cmd_best_effort(cmd: &str, args: &[&str]) -> bool {
    match Command::new(cmd).args(args).output() {
        Ok(output) if output.status.success() => true,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            info!("{} {:?} failed (non-fatal): {}", cmd, args, stderr.trim());
            false
        }
        Err(e) => {
            info!("{} not available (non-fatal): {}", cmd, e);
            false
        }
    }
}

/// Shared cleanup sequence for a clean tunnel shutdown, run identically
/// regardless of what triggered it (Ctrl-C/SIGINT, SIGTERM, or a fatal
/// task failure reported via `TunnelEvent::FatalError`). Factored into one
/// function so the different trigger paths cannot structurally drift
/// apart over time — previously only Ctrl-C ran this sequence at all.
async fn shutdown_and_exit(engine: Arc<TunnelEngine>, reason: &str) -> ! {
    info!("Shutting down PhantomMesh VPN... ({})", reason);
    engine.stop().await;
    let _ = teardown_kill_switch();
    let _ = restore_dns();
    info!("PhantomMesh VPN is DOWN");
    std::process::exit(0);
}

async fn cmd_up(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("phantom_mesh=info".parse().unwrap()))
        .init();

    info!("PhantomMesh VPN v{} starting", env!("CARGO_PKG_VERSION"));

    // Prometheus metrics + status HTTP server (Stage 6). Previously only
    // src/main.rs's `phantom-node` binary started this — but `phantom-node`
    // never actually starts a tunnel (see main.rs's own TODOs), so in
    // production (where this `phantommesh` binary, not `phantom-node`, is
    // what systemd actually runs) `/metrics` and `/health` were never
    // reachable at all, and `phantommesh status` (which polls
    // http://127.0.0.1:8080/health below) always reported DOWN regardless
    // of real tunnel state. Wiring it in here — the actual `up` path — is
    // what makes both of those genuinely work.
    init_metrics();
    info!("Prometheus metrics initialized");

    let crypto = Arc::new(CryptoManager::new()?);
    let private_key = config.decode_private_key()
        .map_err(|e| format!("Invalid private key: {}", e))?;
    let public_key = x25519_dalek::x25519(private_key, x25519_dalek::X25519_BASEPOINT_BYTES);

    info!(pubkey = %hex::encode(&public_key[..8]), "Identity loaded");

    let (event_tx, mut event_rx) = mpsc::channel(256);
    // Mesh healer: 75s timeout (3 missed 25s keepalives), matching the
    // production wiring in src/main.rs's phantom-node binary. Previously
    // this CLI (`phantommesh up` — the binary that actually runs a real
    // tunnel end-to-end) never wired a healer at all, meaning the mesh-heal
    // reconnect path added in this stage would never run for anyone
    // actually using `phantommesh up`, regardless of the underlying fix.
    let mesh_healer = Arc::new(MeshHealer::new(75));

    // Real in-tunnel threat detection (Stage 6). `main.rs`'s phantom-node
    // binary already wired this via `.with_threat_engine()` — every inbound
    // decrypted packet gets passed through `analyze_packet`, and a match
    // emits `TunnelEvent::ThreatSignature` (handled below in the event
    // loop). This CLI (the binary that actually runs a real tunnel) never
    // did this, meaning `phantommesh up` ran with zero in-tunnel threat
    // detection regardless of the detection engine itself working fine.
    // Separate instance from `api_threat_engine` above: `with_threat_engine`
    // needs a bare `Arc<ThreatEngine>` (called concurrently from the tunnel
    // packet path), while `ApiGateway::new` needs `Arc<Mutex<ThreatEngine>>`
    // for its own independently-locked /threat/* routes — same two-instance
    // split main.rs already uses, not new duplication introduced here.
    let tunnel_threat_engine = Arc::new(ThreatEngine::new()?);
    tunnel_threat_engine.initialize().await?;

    let engine = Arc::new(TunnelEngine::new(
        crypto.clone(),
        event_tx,
        private_key,
        public_key,
    )
        .with_mesh_healer(mesh_healer)
        .with_threat_engine(Arc::clone(&tunnel_threat_engine)));

    // Start the metrics/status HTTP server (same ApiGateway + "/health" +
    // "/metrics" routes main.rs's phantom-node binary already served — see
    // vpn_core::api_gateway::ApiGateway::router). ApiGateway needs its own
    // ThreatEngine handle for the /threat/* routes; matching main.rs's
    // existing pattern, this is a separate ThreatEngine instance from the
    // one (if any) wired into the tunnel engine itself, since ApiGateway's
    // constructor takes ownership of the Arc<Mutex<...>> independently.
    let api_threat_engine = Arc::new(tokio::sync::Mutex::new(ThreatEngine::new()?));
    api_threat_engine.lock().await.initialize().await?;
    let api_gateway = ApiGateway::new(api_threat_engine);
    let _api_handle = tokio::spawn(async move {
        if let Err(e) = api_gateway.serve("0.0.0.0:8080").await {
            error!("API gateway error: {}", e);
        }
    });
    info!("API gateway available on http://0.0.0.0:8080 (/health, /metrics)");

    // Periodic system-metrics refresh (memory/CPU gauges), same interval
    // and pattern as main.rs's background task.
    let _metrics_handle = tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            update_system_metrics();
        }
    });

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

    // Handle signals for clean shutdown. SIGINT (Ctrl-C) and SIGTERM (the
    // signal `systemd stop` / plain `kill <pid>` send by default) both run
    // the IDENTICAL shutdown_and_exit() sequence above — previously only
    // Ctrl-C was handled at all, so a SIGTERM'd process left its TUN
    // interface, iptables rules, and DNS config behind.
    let engine_sigint = engine.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_and_exit(engine_sigint, "SIGINT").await;
    });

    #[cfg(target_os = "linux")]
    {
        let engine_sigterm = engine.clone();
        tokio::spawn(async move {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(mut sigterm) => {
                    sigterm.recv().await;
                    shutdown_and_exit(engine_sigterm, "SIGTERM").await;
                }
                Err(e) => {
                    error!("Failed to install SIGTERM handler: {}", e);
                }
            }
        });
    }

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
                phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::PacketRouted { dimension, bytes } => {
                    tracing::debug!(dimension, bytes, "Packet routed");
                }
                phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::ThreatSignature { signature, source } => {
                    // Mirrors main.rs's handling exactly: re-run the full
                    // engine (not just the inline detection that already
                    // ran once inside the tunnel packet path) so a real
                    // alert actually gets generated, not just logged.
                    warn!(source = ?source, "Threat signature detected in tunnel");

                    let threat_result = tunnel_threat_engine.analyze_packet(&signature, Some(&source)).await;
                    if let Some(threat) = threat_result {
                        warn!(
                            threat_id = ?threat.signature_id,
                            severity = ?threat.severity,
                            confidence = threat.confidence,
                            "Threat confirmed by engine"
                        );
                        tunnel_threat_engine.generate_alert(&threat).await?;
                    }
                }
                phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::FatalError { task, reason } => {
                    // A supervised task (UDP recv, TUN read, or keepalive)
                    // hit a fatal error; TunnelEngine has already flipped
                    // `running` to false and aborted the sibling tasks
                    // internally. Run the same full cleanup path here too
                    // (kill switch, DNS) instead of leaving the process
                    // half-alive with `status` still reporting UP.
                    error!(task = task, reason = %reason, "Fatal task failure, shutting down tunnel");
                    shutdown_and_exit(engine.clone(), "fatal task failure").await;
                }
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
                let _ = Command::new("pkill").args(["-SIGTERM", "phantommesh"]).output();
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
