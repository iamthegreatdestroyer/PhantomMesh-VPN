//! Application state management for PhantomMesh VPN client

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Primary application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub connection: ConnectionState,
    pub settings: VpnSettings,
    pub session: Option<SessionInfo>,
    pub stats: ConnectionStats,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            settings: VpnSettings::default(),
            session: None,
            stats: ConnectionStats::default(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// VPN connection states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum ConnectionState {
    Disconnected,
    Connecting {
        server_id: String,
    },
    Connected {
        server: ServerInfo,
        connected_at: DateTime<Utc>,
    },
    Disconnecting,
    Reconnecting {
        attempt: u32,
    },
    Error {
        message: String,
    },
}

/// VPN server information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub country: String,
    pub country_code: String,
    pub city: String,
    pub ip_address: String,
    pub load: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<u32>,
    pub features: Vec<String>,
    pub protocol: VpnProtocol,
}

/// User settings for VPN behavior - aligned with frontend VpnSettings interface
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnSettings {
    pub kill_switch: bool,
    pub auto_connect: bool,
    pub protocol: VpnProtocol,
    pub encryption_level: EncryptionLevel,
    pub split_tunneling: bool,
    pub excluded_apps: Vec<String>,
    pub dns_servers: Vec<String>,
    pub start_minimized: bool,
    pub show_notifications: bool,
}

impl Default for VpnSettings {
    fn default() -> Self {
        Self {
            kill_switch: true,
            auto_connect: false,
            protocol: VpnProtocol::WireGuard,
            encryption_level: EncryptionLevel::Aes256, // Default: balanced speed/security
            split_tunneling: false,
            excluded_apps: Vec::new(),
            dns_servers: vec!["1.1.1.1".to_string(), "1.0.0.1".to_string()],
            start_minimized: false,
            show_notifications: true,
        }
    }
}

/// VPN protocol options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum VpnProtocol {
    #[default]
    WireGuard,
    OpenVPN,
    Stealth,
}

/// Encryption strength levels
///
/// - `Aes256`: AES-256-GCM (Default, balanced speed/security)
/// - `ChaCha20`: ChaCha20-Poly1305 (Optimized for ARM/mobile)
/// - `Military`: AES-256-GCM-SIV (Nonce-misuse resistant, high security)
/// - `Paranoid`: Cascade AES→ChaCha20 (Double encryption, maximum protection)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EncryptionLevel {
    #[default]
    Aes256,
    ChaCha20,
    Military,
    Paranoid,
}

/// Active VPN session information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub session_id: Uuid,
    pub server: ServerInfo,
    pub connected_at: DateTime<Utc>,
    pub assigned_ip: String,
    pub protocol: VpnProtocol,
    pub cipher: String,
}

/// Connect response wrapper - matches frontend { server: ServerInfo } expectation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectResponse {
    pub server: ServerInfo,
}

/// Connection statistics - aligned with frontend ConnectionStats interface
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStats {
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub duration: u64,
    pub current_speed: SpeedStats,
}

/// Speed statistics for current connection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedStats {
    pub download: u64,
    pub upload: u64,
}

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub username: String,
    pub email: String,
    pub tier: AccountTier,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_connections: u32,
    pub current_connections: u32,
}

/// Account tier levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AccountTier {
    Free,
    Plus,
    Visionary,
    Enterprise,
}

/// Server list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerList {
    pub servers: Vec<ServerInfo>,
    pub last_updated: DateTime<Utc>,
}

/// Quick connect preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickConnectProfile {
    pub id: String,
    pub name: String,
    pub criteria: QuickConnectCriteria,
}

/// Criteria for quick connect server selection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuickConnectCriteria {
    Fastest,
    Random,
    Country { code: String },
    SecureCore,
    P2P,
    Streaming,
    LastUsed,
}
