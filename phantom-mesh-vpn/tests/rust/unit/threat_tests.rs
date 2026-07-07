//! Threat engine tests
//!
//! STAGE 7 AUDIT (security-repair plan, 2026-07-07): these are placeholder
//! stubs, NOT wired into Cargo.toml / the build (no [[test]] entry, no
//! `tests/rust.rs` root file, nothing does `mod rust;` anywhere) — they have
//! never actually executed as part of `cargo test`.
//!
//! Reviewed against real coverage in src/security_layer/threat_engine.rs
//! (632 lines) and its usage in src/vpn_core/tunnel_engine.rs. Verdict: KEPT,
//! not deleted — this is a REAL, STILL-OPEN coverage gap, not redundant with
//! anything that exists today. The only real test that touches ThreatEngine
//! is `vpn_core::tunnel_engine::tests::test_threat_engine_wiring`, which only
//! asserts `engine.threat_engine.is_some()` — it does not construct a
//! ThreatSignature, run signature matching, or exercise anomaly detection.
//! `src/security_layer/threat_engine.rs` itself has zero `#[test]` functions.
//!
//! Do not delete these without either (a) implementing real coverage, or
//! (b) an explicit decision that threat-signature/anomaly-detection logic is
//! out of scope for this security-repair plan.

#[tokio::test]
async fn test_threat_engine_initialization() {
    // GAP: no real test constructs ThreatEngine::new() and inspects its
    // initial state (registered signatures, default severity thresholds,
    // response-action channel wiring) beyond "it exists".
    assert!(true);
}

#[test]
fn test_signature_matching() {
    // GAP: no real test builds a ThreatSignature and confirms the engine
    // actually flags a matching packet/payload as a threat.
    assert!(true);
}

#[test]
fn test_anomaly_detection() {
    // GAP: no real test exercises statrs-based statistical anomaly
    // detection against PacketStats.
    assert!(true);
}
