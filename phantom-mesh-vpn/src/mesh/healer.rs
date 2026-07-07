//! Mesh Healing and Auto-Reconnect
//!
//! Monitors peers via heartbeat, detects disconnection, reconnects with
//! exponential back-off, and reroutes traffic through alternate peers
//! during the reconnect window.
//!
//! CRITICAL: tunnel decryption keys are never passed to this module and must
//! not appear in any log output here or in callers.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn};

use lazy_static::lazy_static;
use prometheus::{Counter, Gauge, Opts, Registry};

// ── Prometheus metrics ──────────────────────────────────────────────────────

lazy_static! {
    pub static ref MESH_RECONNECT_ATTEMPTS: Counter = Counter::with_opts(
        Opts::new(
            "mesh_reconnect_attempts_total",
            "Total mesh reconnect attempts",
        )
    )
    .unwrap();

    pub static ref MESH_RECONNECT_SUCCESS_RATE: Gauge = Gauge::with_opts(
        Opts::new(
            "mesh_reconnect_success_rate",
            "Rolling success rate of mesh reconnects (0.0–1.0)",
        )
    )
    .unwrap();
}

pub fn register_metrics(registry: &Registry) {
    registry
        .register(Box::new(MESH_RECONNECT_ATTEMPTS.clone()))
        .ok();
    registry
        .register(Box::new(MESH_RECONNECT_SUCCESS_RATE.clone()))
        .ok();
}

// ── Core types ───────────────────────────────────────────────────────────────

/// Stable identifier for a mesh peer (public key fingerprint, not key material)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeerId(pub String);

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Reason a heal cycle was triggered
#[derive(Clone, Debug)]
pub enum DisconnectReason {
    HeartbeatTimeout,
    ExplicitDisconnect,
    NetworkError(String),
}

impl std::fmt::Display for DisconnectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HeartbeatTimeout => write!(f, "heartbeat_timeout"),
            Self::ExplicitDisconnect => write!(f, "explicit_disconnect"),
            Self::NetworkError(e) => write!(f, "network_error:{e}"),
        }
    }
}

/// Reconnect outcome
#[derive(Debug, PartialEq, Eq)]
pub enum ReconnectOutcome {
    Success,
    Exhausted,
}

// ── Back-off sequence ────────────────────────────────────────────────────────

/// Produces 1 → 2 → 4 → 8 → 16 → 32 → 60 clamped to `max_secs`.
pub struct ExponentialBackoff {
    next_secs: u64,
    max_secs: u64,
}

impl ExponentialBackoff {
    pub fn new(max_secs: u64) -> Self {
        Self { next_secs: 1, max_secs }
    }

    pub fn next(&mut self) -> Duration {
        let delay = self.next_secs.min(self.max_secs);
        self.next_secs = (self.next_secs * 2).min(self.max_secs);
        Duration::from_secs(delay)
    }

    pub fn reset(&mut self) {
        self.next_secs = 1;
    }
}

// ── Peer state ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PeerStatus {
    Connected,
    Reconnecting,
    Dead,
}

#[derive(Clone, Debug)]
struct PeerState {
    status: PeerStatus,
    last_heartbeat: Instant,
}

// ── MeshHealer ───────────────────────────────────────────────────────────────

/// Monitors peer heartbeats and drives reconnect / reroute logic.
pub struct MeshHealer {
    heartbeat_timeout: Duration,
    peers: Arc<Mutex<HashMap<PeerId, PeerState>>>,
    attempts: Arc<Mutex<u64>>,
    successes: Arc<Mutex<u64>>,
}

impl MeshHealer {
    pub fn new(heartbeat_timeout_secs: u64) -> Self {
        Self {
            heartbeat_timeout: Duration::from_secs(heartbeat_timeout_secs),
            peers: Arc::new(Mutex::new(HashMap::new())),
            attempts: Arc::new(Mutex::new(0)),
            successes: Arc::new(Mutex::new(0)),
        }
    }

    pub fn with_default_timeout() -> Self {
        Self::new(30)
    }

    // ── Peer management ──────────────────────────────────────────────────────

    pub async fn register_peer(&self, peer_id: PeerId) {
        let mut peers = self.peers.lock().await;
        peers.insert(
            peer_id,
            PeerState {
                status: PeerStatus::Connected,
                last_heartbeat: Instant::now(),
            },
        );
    }

    /// Record a heartbeat from a peer, clearing any reconnecting state.
    pub async fn record_heartbeat(&self, peer_id: &PeerId) {
        let mut peers = self.peers.lock().await;
        if let Some(state) = peers.get_mut(peer_id) {
            state.last_heartbeat = Instant::now();
            state.status = PeerStatus::Connected;
        }
    }

    pub async fn peer_status(&self, peer_id: &PeerId) -> Option<PeerStatus> {
        self.peers.lock().await.get(peer_id).map(|s| s.status.clone())
    }

    /// Mark a peer `Dead` immediately, bypassing `heal_peer`'s retry/back-off
    /// loop entirely.
    ///
    /// Used by the tunnel engine for a session that has independently earned
    /// "dead" status by its own security-expiry criteria (missed 3 keepalives
    /// AND exceeded its rekey deadline without successfully rehandshaking) —
    /// such a session should not get another `heal_peer` retry cycle, since
    /// the underlying problem isn't transient connectivity but a session
    /// whose keys are past their allowed lifetime. Running it through
    /// `heal_peer` would incorrectly reset it into `Reconnecting` and retry
    /// for up to a minute of back-off before landing on `Dead` anyway; this
    /// skips straight there. A no-op if the peer isn't registered.
    pub async fn mark_dead(&self, peer_id: &PeerId) {
        let mut peers = self.peers.lock().await;
        if let Some(s) = peers.get_mut(peer_id) {
            s.status = PeerStatus::Dead;
        }
    }

    // ── Heartbeat check ──────────────────────────────────────────────────────

    /// Returns peer IDs whose last heartbeat exceeded the configured timeout.
    pub async fn detect_timed_out_peers(&self) -> Vec<PeerId> {
        let peers = self.peers.lock().await;
        peers
            .iter()
            .filter(|(_, s)| {
                s.status == PeerStatus::Connected
                    && s.last_heartbeat.elapsed() > self.heartbeat_timeout
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    // ── Reconnect loop ───────────────────────────────────────────────────────

    /// Drive the heal cycle for one peer.
    ///
    /// `try_connect` is an async closure that attempts to re-establish the
    /// tunnel; it must not receive or log key material.
    ///
    /// Back-off: 1 s → 2 s → 4 s → 8 s → 16 s → 32 s → 60 s (max), up to
    /// `max_attempts` tries.
    pub async fn heal_peer<F, Fut>(
        &self,
        peer_id: &PeerId,
        reason: DisconnectReason,
        max_attempts: u32,
        mut try_connect: F,
    ) -> ReconnectOutcome
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        {
            let mut peers = self.peers.lock().await;
            if let Some(s) = peers.get_mut(peer_id) {
                s.status = PeerStatus::Reconnecting;
            }
        }

        let mut backoff = ExponentialBackoff::new(60);

        for attempt in 1..=max_attempts {
            MESH_RECONNECT_ATTEMPTS.inc();
            *self.attempts.lock().await += 1;

            info!(
                peer_id = %peer_id,
                reason = %reason,
                attempt,
                "mesh heal attempt"
            );

            if try_connect().await {
                info!(peer_id = %peer_id, attempt, "mesh heal succeeded");

                *self.successes.lock().await += 1;
                {
                    let mut peers = self.peers.lock().await;
                    if let Some(s) = peers.get_mut(peer_id) {
                        s.status = PeerStatus::Connected;
                        s.last_heartbeat = Instant::now();
                    }
                }
                self.update_success_rate().await;
                return ReconnectOutcome::Success;
            }

            if attempt < max_attempts {
                let delay = backoff.next();
                tokio::time::sleep(delay).await;
            }
        }

        warn!(peer_id = %peer_id, "mesh heal exhausted all attempts, marking dead");
        {
            let mut peers = self.peers.lock().await;
            if let Some(s) = peers.get_mut(peer_id) {
                s.status = PeerStatus::Dead;
            }
        }
        self.update_success_rate().await;
        ReconnectOutcome::Exhausted
    }

    // ── Traffic rerouting ────────────────────────────────────────────────────

    /// Return an alternate peer to route through while `disconnected_peer`
    /// is being healed. Returns `None` if no connected alternate exists.
    pub async fn select_reroute_target(&self, disconnected_peer: &PeerId) -> Option<PeerId> {
        let peers = self.peers.lock().await;
        peers
            .iter()
            .find(|(id, s)| *id != disconnected_peer && s.status == PeerStatus::Connected)
            .map(|(id, _)| id.clone())
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    async fn update_success_rate(&self) {
        let attempts = *self.attempts.lock().await;
        let successes = *self.successes.lock().await;
        if attempts > 0 {
            MESH_RECONNECT_SUCCESS_RATE.set(successes as f64 / attempts as f64);
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(id: &str) -> PeerId {
        PeerId(id.to_string())
    }

    #[tokio::test]
    async fn test_heartbeat_timeout() {
        // Use a 0-second timeout so any registered peer is immediately "timed out"
        let healer = MeshHealer::new(0);
        healer.register_peer(peer("p1")).await;

        // Give Instant::elapsed a tiny moment to exceed 0 s
        tokio::time::sleep(Duration::from_millis(5)).await;

        let timed_out = healer.detect_timed_out_peers().await;
        assert!(timed_out.contains(&peer("p1")));
    }

    #[tokio::test]
    async fn test_heartbeat_resets_timeout() {
        // Use a 60s timeout so a fresh heartbeat never looks stale
        let healer = MeshHealer::new(60);
        healer.register_peer(peer("p1")).await;
        healer.record_heartbeat(&peer("p1")).await;

        let timed_out = healer.detect_timed_out_peers().await;
        assert!(!timed_out.contains(&peer("p1")));
    }

    #[test]
    fn test_reconnect_backoff_sequence() {
        let mut bo = ExponentialBackoff::new(60);
        let expected_secs: &[u64] = &[1, 2, 4, 8, 16, 32, 60, 60];
        for &expected in expected_secs {
            assert_eq!(bo.next(), Duration::from_secs(expected));
        }
    }

    #[test]
    fn test_reconnect_backoff_clamp() {
        let mut bo = ExponentialBackoff::new(30);
        // 1 → 2 → 4 → 8 → 16 → 30 → 30 …
        let seq: Vec<u64> = (0..8).map(|_| bo.next().as_secs()).collect();
        assert_eq!(seq, vec![1, 2, 4, 8, 16, 30, 30, 30]);
    }

    #[tokio::test]
    async fn test_reconnect_succeeds_on_first_attempt() {
        let healer = MeshHealer::new(30);
        healer.register_peer(peer("p1")).await;

        let outcome = healer
            .heal_peer(&peer("p1"), DisconnectReason::HeartbeatTimeout, 5, || async { true })
            .await;

        assert_eq!(outcome, ReconnectOutcome::Success);
        assert_eq!(healer.peer_status(&peer("p1")).await, Some(PeerStatus::Connected));
    }

    #[tokio::test]
    async fn test_mark_dead_bypasses_retry_loop() {
        let healer = MeshHealer::new(30);
        healer.register_peer(peer("p1")).await;
        assert_eq!(healer.peer_status(&peer("p1")).await, Some(PeerStatus::Connected));

        healer.mark_dead(&peer("p1")).await;

        assert_eq!(healer.peer_status(&peer("p1")).await, Some(PeerStatus::Dead));
    }

    #[tokio::test]
    async fn test_mark_dead_is_noop_for_unregistered_peer() {
        let healer = MeshHealer::new(30);
        // Never registered — must not panic, must remain absent.
        healer.mark_dead(&peer("ghost")).await;
        assert_eq!(healer.peer_status(&peer("ghost")).await, None);
    }

    #[tokio::test]
    async fn test_reconnect_exhausted() {
        let healer = MeshHealer::new(30);
        healer.register_peer(peer("p1")).await;

        let outcome = healer
            .heal_peer(&peer("p1"), DisconnectReason::HeartbeatTimeout, 2, || async { false })
            .await;

        assert_eq!(outcome, ReconnectOutcome::Exhausted);
        assert_eq!(healer.peer_status(&peer("p1")).await, Some(PeerStatus::Dead));
    }

    #[tokio::test]
    async fn test_traffic_rerouting() {
        let healer = MeshHealer::new(30);
        healer.register_peer(peer("p1")).await;
        healer.register_peer(peer("p2")).await;

        // Mark p1 as disconnecting
        let mut peers = healer.peers.lock().await;
        peers.get_mut(&peer("p1")).unwrap().status = PeerStatus::Reconnecting;
        drop(peers);

        let reroute = healer.select_reroute_target(&peer("p1")).await;
        assert_eq!(reroute, Some(peer("p2")));
    }

    #[tokio::test]
    async fn test_reroute_returns_none_when_no_alternates() {
        let healer = MeshHealer::new(30);
        healer.register_peer(peer("only")).await;

        let reroute = healer.select_reroute_target(&peer("only")).await;
        assert!(reroute.is_none());
    }
}
