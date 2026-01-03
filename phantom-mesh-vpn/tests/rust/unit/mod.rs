//! Unit tests for PhantomMesh-VPN
//!
//! Test organization:
//! - crypto_tests: ΣVault and cryptographic primitives
//! - tunnel_tests: Tunnel engine functionality
//! - threat_tests: Threat detection engine
//! - metrics_tests: Prometheus metrics

pub mod crypto_tests;
pub mod metrics_tests;
pub mod threat_tests;
pub mod tunnel_tests;
