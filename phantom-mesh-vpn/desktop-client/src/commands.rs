//! Tauri commands exposed to the frontend

use chrono::Utc;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::{ErrorResponse, VpnError};
use crate::state::*;
use crate::vpn::VpnClient;

type AppStateHandle = State<'_, Arc<RwLock<AppState>>>;

// ============================================================================
// Connection Commands
// ============================================================================

/// Connect to a specific VPN server
#[tauri::command]
pub async fn connect(
    state: AppStateHandle,
    server_id: String,
) -> Result<ConnectResponse, ErrorResponse> {
    info!("🔌 Connecting to server: {}", server_id);

    let mut app_state = state.write().await;

    // Check if already connected
    if let ConnectionState::Connected { .. } = &app_state.connection {
        warn!("Already connected, disconnecting first...");
        app_state.connection = ConnectionState::Disconnecting;
    }

    // Update state to connecting
    app_state.connection = ConnectionState::Connecting {
        server_id: server_id.clone(),
    };

    // Simulate connection (replace with actual VPN logic)
    let server = get_mock_server(&server_id);

    // Create session
    let session = SessionInfo {
        session_id: Uuid::new_v4(),
        server: server.clone(),
        connected_at: Utc::now(),
        assigned_ip: "10.8.0.42".to_string(),
        protocol: app_state.settings.protocol.clone(),
        cipher: "ChaCha20-Poly1305".to_string(),
    };

    app_state.connection = ConnectionState::Connected {
        server: server.clone(),
        connected_at: session.connected_at,
    };
    app_state.session = Some(session.clone());
    app_state.stats = ConnectionStats::default();

    info!("✅ Connected successfully. Session: {}", session.session_id);

    Ok(ConnectResponse { server })
}

/// Disconnect from VPN
#[tauri::command]
pub async fn disconnect(state: AppStateHandle) -> Result<(), ErrorResponse> {
    info!("🔌 Disconnecting from VPN...");

    let mut app_state = state.write().await;

    app_state.connection = ConnectionState::Disconnecting;

    // Simulate disconnection delay
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    app_state.connection = ConnectionState::Disconnected;
    app_state.session = None;

    info!("✅ Disconnected successfully");

    Ok(())
}

/// Quick connect to the best available server
#[tauri::command]
pub async fn quick_connect(
    state: AppStateHandle,
    criteria: Option<String>,
) -> Result<ConnectResponse, ErrorResponse> {
    info!("⚡ Quick connect with criteria: {:?}", criteria);

    // Find best server based on criteria
    let server_id = match criteria.as_deref() {
        Some("fastest") | None => "us-east-1".to_string(),
        Some("p2p") => "nl-amsterdam-p2p".to_string(),
        Some("streaming") => "us-west-streaming".to_string(),
        Some(country) => format!("{}-auto", country.to_lowercase()),
    };

    connect(state, server_id).await
}

/// Get current connection status
#[tauri::command]
pub async fn get_connection_status(state: AppStateHandle) -> ConnectionState {
    let app_state = state.read().await;
    app_state.connection.clone()
}

// ============================================================================
// Server Commands
// ============================================================================

/// Get list of available VPN servers
#[tauri::command]
pub async fn get_servers() -> Result<Vec<ServerInfo>, ErrorResponse> {
    info!("📋 Fetching server list...");

    // Return mock server list (replace with actual API call)
    let servers = vec![
        create_server(
            "us-east-1",
            "US East #1",
            "United States",
            "US",
            "New York",
            23,
            12,
        ),
        create_server(
            "us-west-1",
            "US West #1",
            "United States",
            "US",
            "Los Angeles",
            45,
            28,
        ),
        create_server(
            "uk-london-1",
            "UK #1",
            "United Kingdom",
            "GB",
            "London",
            31,
            45,
        ),
        create_server(
            "de-frankfurt-1",
            "Germany #1",
            "Germany",
            "DE",
            "Frankfurt",
            18,
            52,
        ),
        create_server(
            "nl-amsterdam-1",
            "Netherlands #1",
            "Netherlands",
            "NL",
            "Amsterdam",
            15,
            61,
        ),
        create_server("jp-tokyo-1", "Japan #1", "Japan", "JP", "Tokyo", 67, 142),
        create_server(
            "sg-singapore-1",
            "Singapore #1",
            "Singapore",
            "SG",
            "Singapore",
            54,
            178,
        ),
        create_server(
            "au-sydney-1",
            "Australia #1",
            "Australia",
            "AU",
            "Sydney",
            38,
            195,
        ),
        create_server(
            "ca-toronto-1",
            "Canada #1",
            "Canada",
            "CA",
            "Toronto",
            29,
            35,
        ),
        create_server(
            "ch-zurich-1",
            "Switzerland #1",
            "Switzerland",
            "CH",
            "Zurich",
            12,
            68,
        ),
        create_server(
            "nl-amsterdam-p2p",
            "Netherlands P2P",
            "Netherlands",
            "NL",
            "Amsterdam",
            42,
            61,
        ),
        create_server(
            "us-west-streaming",
            "US Streaming",
            "United States",
            "US",
            "Los Angeles",
            35,
            28,
        ),
    ];

    Ok(servers)
}

/// Get load for a specific server
#[tauri::command]
pub async fn get_server_load(server_id: String) -> Result<u8, ErrorResponse> {
    // Simulate server load query
    let load = (server_id.len() as u8 * 7) % 100;
    Ok(load)
}

/// Ping a server to get latency
#[tauri::command]
pub async fn ping_server(server_id: String) -> Result<u32, ErrorResponse> {
    info!("🏓 Pinging server: {}", server_id);

    // Simulate ping (replace with actual ICMP/TCP ping)
    let base_latency = match server_id.split('-').next() {
        Some("us") => 25,
        Some("uk") | Some("de") | Some("nl") | Some("ch") => 55,
        Some("jp") | Some("sg") => 150,
        Some("au") => 180,
        Some("ca") => 40,
        _ => 100,
    };

    // Add some variance
    let variance = (server_id.len() as u32 % 20) as i32 - 10;
    let latency = (base_latency as i32 + variance).max(5) as u32;

    Ok(latency)
}

// ============================================================================
// Settings Commands
// ============================================================================

/// Get current VPN settings
#[tauri::command]
pub async fn get_settings(state: AppStateHandle) -> VpnSettings {
    let app_state = state.read().await;
    app_state.settings.clone()
}

/// Update VPN settings
#[tauri::command]
pub async fn update_settings(
    state: AppStateHandle,
    settings: VpnSettings,
) -> Result<(), ErrorResponse> {
    info!("⚙️ Updating settings...");

    let mut app_state = state.write().await;
    app_state.settings = settings;

    info!("✅ Settings updated");
    Ok(())
}

/// Toggle kill switch
#[tauri::command]
pub async fn toggle_kill_switch(
    state: AppStateHandle,
    enabled: bool,
) -> Result<bool, ErrorResponse> {
    let mut app_state = state.write().await;
    app_state.settings.kill_switch = enabled;

    info!(
        "🛡️ Kill switch: {}",
        if enabled { "ENABLED" } else { "DISABLED" }
    );

    Ok(enabled)
}

/// Toggle auto-connect
#[tauri::command]
pub async fn toggle_auto_connect(
    state: AppStateHandle,
    enabled: bool,
) -> Result<bool, ErrorResponse> {
    let mut app_state = state.write().await;
    app_state.settings.auto_connect = enabled;

    info!(
        "🔄 Auto-connect: {}",
        if enabled { "ENABLED" } else { "DISABLED" }
    );

    Ok(enabled)
}

// ============================================================================
// Statistics Commands
// ============================================================================

/// Get connection statistics
#[tauri::command]
pub async fn get_connection_stats(state: AppStateHandle) -> ConnectionStats {
    let app_state = state.read().await;
    app_state.stats.clone()
}

/// Get bandwidth usage
#[tauri::command]
pub async fn get_bandwidth_usage(state: AppStateHandle) -> Result<BandwidthUsage, ErrorResponse> {
    let app_state = state.read().await;

    Ok(BandwidthUsage {
        upload_speed_bps: app_state.stats.current_speed.upload,
        download_speed_bps: app_state.stats.current_speed.download,
        total_uploaded: app_state.stats.bytes_out,
        total_downloaded: app_state.stats.bytes_in,
    })
}

#[derive(serde::Serialize)]
pub struct BandwidthUsage {
    pub upload_speed_bps: u64,
    pub download_speed_bps: u64,
    pub total_uploaded: u64,
    pub total_downloaded: u64,
}

// ============================================================================
// Account Commands
// ============================================================================

/// Get account information
#[tauri::command]
pub async fn get_account_info() -> Result<AccountInfo, ErrorResponse> {
    // Mock account info (replace with actual API call)
    Ok(AccountInfo {
        username: "phantom_user".to_string(),
        email: "user@phantommesh.io".to_string(),
        tier: AccountTier::Plus,
        expires_at: Some(Utc::now() + chrono::Duration::days(365)),
        max_connections: 10,
        current_connections: 1,
    })
}

// ============================================================================
// System Commands
// ============================================================================

/// Check for application updates
#[tauri::command]
pub async fn check_updates() -> Result<UpdateInfo, ErrorResponse> {
    info!("🔍 Checking for updates...");

    Ok(UpdateInfo {
        update_available: false,
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        latest_version: env!("CARGO_PKG_VERSION").to_string(),
        release_notes: None,
        download_url: None,
    })
}

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
}

/// Get application version
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Open logs folder in file explorer
#[tauri::command]
pub async fn open_logs_folder() -> Result<(), ErrorResponse> {
    info!("📂 Opening logs folder...");

    #[cfg(target_os = "windows")]
    {
        let logs_path = dirs::data_local_dir()
            .unwrap_or_default()
            .join("PhantomMesh VPN")
            .join("logs");

        let _ = std::process::Command::new("explorer")
            .arg(logs_path)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let logs_path = dirs::data_local_dir()
            .unwrap_or_default()
            .join("PhantomMesh VPN")
            .join("logs");

        let _ = std::process::Command::new("open").arg(logs_path).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let logs_path = dirs::data_local_dir()
            .unwrap_or_default()
            .join("phantommesh-vpn")
            .join("logs");

        let _ = std::process::Command::new("xdg-open")
            .arg(logs_path)
            .spawn();
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_mock_server(server_id: &str) -> ServerInfo {
    create_server(
        server_id,
        &format!("Server {}", server_id),
        "United States",
        "US",
        "New York",
        25,
        15,
    )
}

fn create_server(
    id: &str,
    name: &str,
    country: &str,
    country_code: &str,
    city: &str,
    load: u8,
    latency: u32,
) -> ServerInfo {
    let mut features = vec![];

    if id.contains("p2p") {
        features.push("P2P".to_string());
    }
    if id.contains("streaming") {
        features.push("Streaming".to_string());
    }
    if id.contains("secure") {
        features.push("SecureCore".to_string());
    }

    ServerInfo {
        id: id.to_string(),
        name: name.to_string(),
        country: country.to_string(),
        country_code: country_code.to_string(),
        city: city.to_string(),
        ip_address: format!("10.{}.{}.1", (id.len() % 255), (city.len() % 255)),
        load,
        latency: Some(latency),
        features,
        protocol: VpnProtocol::WireGuard,
    }
}
