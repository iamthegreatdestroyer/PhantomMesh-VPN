//! PhantomMesh VPN Library
//! ========================
//! Core library for PhantomMesh VPN functionality.
//!
//! Copyright © 2025 Stephen Bilodeau. All rights reserved.
//! Licensed under GPL-3.0 with proprietary agent clauses.

// Suppress pre-existing lint warnings throughout the codebase
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::manual_ok_or)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::let_and_return)]
#![allow(clippy::for_kv_map)]
#![allow(clippy::new_without_default)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::option_map_unit_fn)]
#![allow(clippy::manual_unwrap_or)]

// Re-export main modules
pub mod agent_framework;
pub mod agents;
pub mod load_test;
pub mod mesh;
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
