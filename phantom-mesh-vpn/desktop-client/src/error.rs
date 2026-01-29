//! Error types for PhantomMesh VPN client

use serde::Serialize;
use thiserror::Error;

/// Application-level errors
#[derive(Debug, Error)]
pub enum VpnError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Server unreachable: {0}")]
    ServerUnreachable(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Kill switch active - blocking network traffic")]
    KillSwitchActive,

    #[error("No servers available")]
    NoServersAvailable,

    #[error("Session expired")]
    SessionExpired,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Rate limited: retry after {0} seconds")]
    RateLimited(u32),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Serializable error for frontend
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub retry_after_secs: Option<u32>,
}

impl From<VpnError> for ErrorResponse {
    fn from(error: VpnError) -> Self {
        match &error {
            VpnError::ConnectionFailed(msg) => ErrorResponse {
                code: "CONNECTION_FAILED".to_string(),
                message: msg.clone(),
                recoverable: true,
                retry_after_secs: Some(5),
            },
            VpnError::AuthenticationFailed(msg) => ErrorResponse {
                code: "AUTH_FAILED".to_string(),
                message: msg.clone(),
                recoverable: false,
                retry_after_secs: None,
            },
            VpnError::ServerUnreachable(msg) => ErrorResponse {
                code: "SERVER_UNREACHABLE".to_string(),
                message: msg.clone(),
                recoverable: true,
                retry_after_secs: Some(10),
            },
            VpnError::KillSwitchActive => ErrorResponse {
                code: "KILL_SWITCH_ACTIVE".to_string(),
                message: error.to_string(),
                recoverable: false,
                retry_after_secs: None,
            },
            VpnError::RateLimited(secs) => ErrorResponse {
                code: "RATE_LIMITED".to_string(),
                message: error.to_string(),
                recoverable: true,
                retry_after_secs: Some(*secs),
            },
            _ => ErrorResponse {
                code: "INTERNAL_ERROR".to_string(),
                message: error.to_string(),
                recoverable: false,
                retry_after_secs: None,
            },
        }
    }
}

/// Result type alias for VPN operations
pub type VpnResult<T> = Result<T, VpnError>;

impl From<reqwest::Error> for VpnError {
    fn from(error: reqwest::Error) -> Self {
        VpnError::NetworkError(error.to_string())
    }
}

impl From<std::io::Error> for VpnError {
    fn from(error: std::io::Error) -> Self {
        VpnError::Internal(error.to_string())
    }
}

impl From<serde_json::Error> for VpnError {
    fn from(error: serde_json::Error) -> Self {
        VpnError::ConfigurationError(error.to_string())
    }
}
