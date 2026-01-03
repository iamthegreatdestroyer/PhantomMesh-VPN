//! PhantomMesh VPN Library
//! ========================
//! Core library for PhantomMesh VPN functionality.
//!
//! Copyright © 2025 Stephen Bilodeau. All rights reserved.
//! Licensed under GPL-3.0 with proprietary agent clauses.

// Re-export main modules
pub mod agent_framework;
pub mod load_test;
pub mod metrics;
pub mod security_layer;
pub mod vpn_core;

// Re-export key types for external use
pub use security_layer::crypto_manager::CryptoManager;
pub use vpn_core::tunnel_engine::{TunnelEngine, TunnelEvent};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the PhantomMesh library
pub fn init() {
    tracing::info!("PhantomMesh library v{} initialized", VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_init() {
        init();
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
