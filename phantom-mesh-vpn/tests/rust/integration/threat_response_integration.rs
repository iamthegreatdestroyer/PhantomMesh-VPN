//! Threat detection and response integration tests
//!
//! STAGE 7 AUDIT (security-repair plan, 2026-07-07): these are placeholder
//! stubs, NOT wired into Cargo.toml / the build (no [[test]] entry, no
//! `tests/rust.rs` root file, nothing does `mod rust;` anywhere) — they have
//! never actually executed as part of `cargo test`.
//!
//! Reviewed against real coverage: `vpn_core::tunnel_engine::tests::
//! test_threat_engine_wiring` only confirms a ThreatEngine can be attached
//! to a TunnelEngine (`engine.threat_engine.is_some()`); it does not run a
//! real packet through active threat detection during a live tunnel, and
//! there is no automatic-response (quarantine/block) test anywhere in the
//! real suite. Verdict: KEPT, not deleted — genuine, still-open gaps,
//! dependent on the same threat_engine.rs internals flagged in
//! tests/rust/unit/threat_tests.rs.
//!
//! Do not delete these without either (a) implementing real coverage, or
//! (b) an explicit decision that end-to-end threat response is out of scope
//! for this security-repair plan.

#[tokio::test]
async fn test_threat_detection_in_tunnel() {
    // GAP: no real test runs an active tunnel and confirms a malicious
    // packet is actually flagged by ThreatEngine during live traffic.
    assert!(true);
}

#[tokio::test]
async fn test_automatic_threat_response() {
    // GAP: no real test confirms an automatic response action (quarantine/
    // block) is actually triggered and takes effect end-to-end.
    assert!(true);
}
