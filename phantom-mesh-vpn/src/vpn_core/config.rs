//! Configuration module for PhantomMesh VPN
//!
//! Loads configuration from TOML file at /etc/phantommesh/config.toml
//! or a path specified via PHANTOMMESH_CONFIG env var.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub interface: InterfaceConfig,
    #[serde(default)]
    pub peers: Vec<PeerEntry>,
    #[serde(default)]
    pub dns: DnsConfig,
    #[serde(default)]
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    pub private_key: String,
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
    #[serde(default = "default_tun_name")]
    pub tun_name: String,
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_mtu")]
    pub mtu: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEntry {
    pub public_key: String,
    pub endpoint: Option<String>,
    #[serde(default)]
    pub allowed_ips: Vec<String>,
    pub preshared_key: Option<String>,
    #[serde(default)]
    pub persistent_keepalive: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    #[serde(default = "default_dns")]
    pub servers: Vec<String>,
    #[serde(default = "default_true")]
    pub leak_protection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_true")]
    pub kill_switch: bool,
    #[serde(default = "default_true")]
    pub threat_detection: bool,
    #[serde(default)]
    pub quantum_resistant: bool,
}

fn default_listen_port() -> u16 { 51820 }
fn default_tun_name() -> String { "pm0".to_string() }
fn default_address() -> String { "10.77.0.1/24".to_string() }
fn default_mtu() -> u16 { 1420 }
fn default_dns() -> Vec<String> { vec!["9.9.9.9".into(), "1.1.1.1".into()] }
fn default_true() -> bool { true }

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            servers: default_dns(),
            leak_protection: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            kill_switch: true,
            threat_detection: true,
            quantum_resistant: false,
        }
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = path
            .map(PathBuf::from)
            .or_else(|| std::env::var("PHANTOMMESH_CONFIG").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("/etc/phantommesh/config.toml"));

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config {}: {}", config_path.display(), e))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        Ok(config)
    }

    pub fn generate_example() -> String {
        r#"# PhantomMesh VPN Configuration
# Place at /etc/phantommesh/config.toml

[interface]
# Generate with: phantommesh genkey
private_key = "YOUR_PRIVATE_KEY_HERE"
listen_port = 51820
tun_name = "pm0"
address = "10.77.0.1/24"
mtu = 1420

[[peers]]
public_key = "PEER_PUBLIC_KEY_HERE"
endpoint = "192.168.1.170:51820"
allowed_ips = ["10.77.0.0/24"]
persistent_keepalive = 25

[dns]
servers = ["9.9.9.9", "1.1.1.1"]
leak_protection = true

[security]
kill_switch = true
threat_detection = true
quantum_resistant = false
"#.to_string()
    }

    pub fn listen_addr(&self) -> SocketAddr {
        format!("0.0.0.0:{}", self.interface.listen_port)
            .parse()
            .expect("Invalid listen address")
    }

    pub fn decode_private_key(&self) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        let bytes = base64_decode(&self.interface.private_key)?;
        if bytes.len() != 32 {
            return Err("Private key must be 32 bytes".into());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    }
}

fn base64_decode(s: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use ring::test::from_hex;
    // Try hex first, then base64
    if s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(from_hex(s).map_err(|_| "Invalid hex")?);
    }
    Err("Key must be 64-char hex string".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_config_parses() {
        let example = Config::generate_example();
        // Replace placeholder key with valid hex
        let example = example.replace(
            "YOUR_PRIVATE_KEY_HERE",
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
        ).replace(
            "PEER_PUBLIC_KEY_HERE",
            "b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3",
        );
        let config: Config = toml::from_str(&example).unwrap();
        assert_eq!(config.interface.listen_port, 51820);
        assert_eq!(config.interface.tun_name, "pm0");
        assert_eq!(config.peers.len(), 1);
        assert!(config.dns.leak_protection);
        assert!(config.security.kill_switch);
    }

    #[test]
    fn test_default_dns() {
        let config: Config = toml::from_str(r#"
[interface]
private_key = "aabbccdd"
"#).unwrap();
        assert_eq!(config.dns.servers, vec!["9.9.9.9", "1.1.1.1"]);
    }
}
