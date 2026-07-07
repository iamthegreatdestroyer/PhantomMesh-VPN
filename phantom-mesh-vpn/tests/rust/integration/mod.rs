//! Integration tests for PhantomMesh-VPN
//!
//! NOTE: this whole tests/rust/** tree is NOT wired into Cargo.toml / the
//! build (no [[test]] entry, nothing does `mod rust;` anywhere) — these
//! files never actually execute as part of `cargo test`. See individual
//! file headers for the Stage 7 audit (2026-07-07) that reviewed each
//! stub against real coverage in `src/`.
//!
//! Test organization (all remaining below are genuine, still-open coverage
//! gaps flagged by that audit — tunnel_integration was deleted as redundant
//! with real coverage that already exists in `src/vpn_core/tunnel_engine.rs`):
//! - peer_mesh_integration: 3+-peer mesh topology, dynamic route optimization
//! - threat_response_integration: live-tunnel threat detection + auto-response

pub mod peer_mesh_integration;
pub mod threat_response_integration;
