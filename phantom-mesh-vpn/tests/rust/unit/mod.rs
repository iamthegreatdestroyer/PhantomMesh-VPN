//! Unit tests for PhantomMesh-VPN
//!
//! NOTE: this whole tests/rust/** tree is NOT wired into Cargo.toml / the
//! build (no [[test]] entry, nothing does `mod rust;` anywhere) — these
//! files never actually execute as part of `cargo test`. See individual
//! file headers for the Stage 7 audit (2026-07-07) that reviewed each
//! stub against real coverage in `src/`.
//!
//! Test organization (all remaining below are genuine, still-open coverage
//! gaps flagged by that audit — crypto_tests and tunnel_tests were deleted
//! as redundant with real coverage that already exists in `src/`):
//! - threat_tests: Threat detection engine (gap: threat_engine.rs internals)
//! - metrics_tests: Prometheus metrics (gap: metrics.rs internals)

pub mod metrics_tests;
pub mod threat_tests;
