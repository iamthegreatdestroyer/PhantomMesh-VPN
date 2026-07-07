//! Prometheus metrics tests
//!
//! STAGE 7 AUDIT (security-repair plan, 2026-07-07): these are placeholder
//! stubs, NOT wired into Cargo.toml / the build (no [[test]] entry, no
//! `tests/rust.rs` root file, nothing does `mod rust;` anywhere) — they have
//! never actually executed as part of `cargo test`.
//!
//! Reviewed against real coverage in src/metrics.rs (191 lines, 14 distinct
//! Prometheus metrics registered via lazy_static, plus `init_metrics()` and
//! `MetricsServer::encode_metrics()`). Verdict: KEPT, not deleted — this is a
//! REAL, STILL-OPEN coverage gap. src/metrics.rs has zero `#[test]`
//! functions of its own, and none of the 80 passing lib tests call
//! `init_metrics()` or assert on registry contents.
//!
//! Do not delete these without either (a) implementing real coverage, or
//! (b) an explicit decision that metrics-registration correctness is out of
//! scope for this security-repair plan.

#[test]
fn test_metrics_initialization() {
    // GAP: no real test calls init_metrics() and confirms the registry
    // populates without error / is idempotent on repeated calls.
    assert!(true);
}

#[test]
fn test_vpn_metrics_exist() {
    // GAP: no real test confirms the expected metric names (e.g.
    // vpn_packets_total, vpn_active_tunnels, threat_events_total) are
    // actually present in METRICS_REGISTRY after init_metrics().
    assert!(true);
}
