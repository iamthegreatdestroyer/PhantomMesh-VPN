//! Tunnel Engine v2.0
//! ==================
//! Real WireGuard-style tunnel with TUN/TAP device, proper Noise handshake,
//! nonce management, and bidirectional packet forwarding.
//!
//! Protocol: Custom Noise_IK with Kyber-768 hybrid post-quantum KEM
//! Transport: ChaCha20-Poly1305 with incrementing nonce counter
//! Device: Linux TUN interface for IP packet tunneling

use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use super::super::security_layer::crypto_manager::CryptoManager;
use crate::mesh::healer::{DisconnectReason, MeshHealer, PeerId};
use crate::security_layer::handshake::{self, HandshakeResult, InitiatorState, NodeIdentity};
use crate::security_layer::threat_engine::ThreatEngine;

// ============================================================================
// Constants
// ============================================================================

const MAX_PACKET_SIZE: usize = 65535;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16; // Poly1305 tag
const HEADER_SIZE: usize = 1 + 4 + 8; // type(1) + session_id(4) + nonce_counter(8) = 13
const REPLAY_WINDOW_SIZE: u64 = 2048;
const KEEPALIVE_INTERVAL_SECS: u64 = 25;

// Packet types
const PACKET_HANDSHAKE_INIT: u8 = 1;
const PACKET_HANDSHAKE_RESP: u8 = 2;
const PACKET_DATA: u8 = 4;
const PACKET_KEEPALIVE: u8 = 8;

// ============================================================================
// Rekey / session-lifecycle policy (Stage 3)
//
// Mirrors WireGuard: rehandshake at a time OR data-volume deadline, whichever
// comes first, with a bounded grace period afterward before the session is
// dropped entirely if rehandshaking hasn't succeeded.
// ============================================================================

/// Proactively rehandshake once a session has been alive this long.
const REKEY_AFTER_SECS: u64 = 120;
/// ...or once this many bytes have been transferred under the current keys,
/// whichever comes first.
const REKEY_AFTER_BYTES: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB
/// If a session crosses its rekey deadline (time OR bytes) and a new
/// handshake still hasn't completed this many additional seconds later, the
/// session is dropped entirely rather than kept around indefinitely waiting.
const REKEY_REJECT_AFTER_SECS: u64 = 180;
/// How long, after a NEW handshake's response arrives and send_key switches,
/// the PREVIOUS session's recv_key remains valid for still-in-flight incoming
/// packets encrypted before the peer itself switched. Deliberately much
/// shorter than REKEY_REJECT_AFTER_SECS — this covers real network
/// reordering/jitter around the moment of a rekey, not an extended grace
/// period for a peer that hasn't rehandshaked at all.
const DUAL_KEY_WINDOW_SECS: u64 = 10;
/// An in-flight (sent, no response yet) handshake we initiated is discarded
/// if no response arrives within this long, so a peer that never responds
/// cannot make us leak InitiatorState forever.
const PENDING_HANDSHAKE_TIMEOUT_SECS: u64 = 10;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Clone)]
pub enum TunnelEvent {
    PeerConnected { public_key: [u8; 32], endpoint: String },
    PeerDisconnected { public_key: [u8; 32] },
    PacketRouted { dimension: u8, bytes: usize },
    ThreatSignature { signature: Vec<u8>, source: String },
    HandshakeCompleted { peer: [u8; 32] },
    /// One of the tunnel's supervised tasks (UDP receive, TUN read, or
    /// keepalive) hit a fatal error and the engine is shutting down as a
    /// result. Emitted so callers (e.g. the CLI) can run the same cleanup
    /// path as an explicit stop, instead of the tunnel silently going
    /// half-alive (e.g. UDP/keepalive still running while TUN-read died).
    FatalError { task: &'static str, reason: String },
}

#[derive(Debug, Clone)]
pub struct PeerConfig {
    pub public_key: [u8; 32],
    pub endpoint: Option<SocketAddr>,
    pub allowed_ips: Vec<String>,
    pub preshared_key: Option<[u8; 32]>,
    pub persistent_keepalive: Option<u64>,
}

/// The previous session's receive-side material, kept around only for the
/// dual-key window immediately after a rekey: incoming packets still
/// arriving under the OLD session_id/recv_key (sent by a peer that hasn't
/// switched yet, or reordered in flight) remain decryptable until
/// `expires_at`. Outgoing traffic never uses this — send_key switches to the
/// new session immediately once the new session is installed.
#[derive(Debug)]
struct PrevSession {
    session_id: u32,
    recv_key: [u8; 32],
    recv_bitmap: Mutex<ReplayWindow>,
    expires_at: Instant,
}

/// A fully-established transport session resulting from a completed
/// handshake (initial or rekey). Everything here is specific to ONE set of
/// negotiated keys — a rekey installs a brand new `Session`, it does not
/// mutate an existing one in place (session_id, nonce counters, and the
/// rekey-deadline clock must all start fresh with new keys).
#[derive(Debug)]
struct Session {
    session_id: u32,
    send_key: [u8; 32],
    recv_key: [u8; 32],
    send_nonce: AtomicU64,
    recv_bitmap: Mutex<ReplayWindow>,
    /// When this session was established — the basis for the 120s
    /// time-based rekey trigger.
    established_at: Instant,
    /// Bytes transferred (sent + received) under THIS session's keys —
    /// the basis for the 4 GiB data-volume rekey trigger. Resets to 0 on
    /// every new session; does not accumulate across rekeys (that's what
    /// the engine-wide `TunnelStats` counters are for).
    bytes_transferred: AtomicU64,
    /// Present only during the dual-key window right after a rekey.
    prev: Mutex<Option<PrevSession>>,
}

impl Session {
    fn is_past_rekey_deadline(&self) -> bool {
        self.established_at.elapsed() >= Duration::from_secs(REKEY_AFTER_SECS)
            || self.bytes_transferred.load(Ordering::Relaxed) >= REKEY_AFTER_BYTES
    }
}

/// An in-flight handshake this side initiated and is waiting on a response
/// for. Not yet a session — no usable keys exist until `process_response`
/// succeeds.
struct PendingHandshake {
    initiator_state: InitiatorState,
    started_at: Instant,
}

impl std::fmt::Debug for PendingHandshake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // InitiatorState holds ephemeral secret key material and deliberately
        // does not implement Debug/Display itself; avoid ever formatting it
        // even indirectly through a derive here.
        f.debug_struct("PendingHandshake")
            .field("started_at", &self.started_at)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct PeerState {
    config: PeerConfig,
    /// `None` until a real handshake round-trip completes — a peer starts
    /// with NO usable session (no keys, no session_id) rather than the old
    /// static-KDF-derived key. Any packet that needs a session (data,
    /// keepalive) while this is `None` simply cannot be sent/decrypted yet.
    session: RwLock<Option<Session>>,
    /// Wall-clock marker of the most recent handshake completion (initial or
    /// rekey), independent of any one Session's own established_at — kept so
    /// the "reject after 180s past deadline" check has a stable point of
    /// reference even across the moment a new Session object replaces the
    /// old one.
    last_handshake: std::sync::Mutex<Option<Instant>>,
    /// First time we noticed the current session (if any) crossed its own
    /// rekey deadline without yet completing a new handshake. Cleared when a
    /// new session is installed. Drives the 180s reject-after-deadline drop
    /// independently of wall-clock time-since-established, since the
    /// deadline can be crossed via the bytes trigger well before 120s.
    rekey_deadline_crossed_at: std::sync::Mutex<Option<Instant>>,
    last_received: std::sync::Mutex<std::time::Instant>,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
}

#[derive(Debug, Default, Clone)]
pub struct TunnelStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub peers_connected: usize,
    pub handshakes_completed: u64,
}

// ============================================================================
// Anti-Replay Window
// ============================================================================

#[derive(Debug)]
struct ReplayWindow {
    bitmap: Vec<u64>,
    last_seq: u64,
}

impl Default for ReplayWindow {
    fn default() -> Self {
        Self {
            bitmap: vec![0u64; (REPLAY_WINDOW_SIZE / 64 + 1) as usize],
            last_seq: 0,
        }
    }
}

impl ReplayWindow {
    fn check_and_update(&mut self, seq: u64) -> bool {
        if seq == 0 {
            return false; // nonce 0 is reserved
        }

        if seq > self.last_seq {
            let shift = seq - self.last_seq;
            if shift >= REPLAY_WINDOW_SIZE {
                self.bitmap.fill(0);
            } else {
                for _ in 0..shift {
                    let idx = ((self.last_seq + 1) % REPLAY_WINDOW_SIZE) as usize;
                    self.bitmap[idx / 64] &= !(1u64 << (idx % 64));
                }
            }
            self.last_seq = seq;
            let idx = (seq % REPLAY_WINDOW_SIZE) as usize;
            self.bitmap[idx / 64] |= 1u64 << (idx % 64);
            true
        } else if self.last_seq - seq >= REPLAY_WINDOW_SIZE {
            false // too old
        } else {
            let idx = (seq % REPLAY_WINDOW_SIZE) as usize;
            let bit = 1u64 << (idx % 64);
            if self.bitmap[idx / 64] & bit != 0 {
                false // replay
            } else {
                self.bitmap[idx / 64] |= bit;
                true
            }
        }
    }
}

// ============================================================================
// Tunnel Engine
// ============================================================================

pub struct TunnelEngine {
    crypto: Arc<CryptoManager>,
    event_tx: mpsc::Sender<TunnelEvent>,
    peers: Arc<RwLock<HashMap<[u8; 32], Arc<PeerState>>>>,
    /// session_id -> owning peer's static public key. Entries are added for
    /// BOTH the current session and (during its dual-key window) the
    /// previous session of a peer, since incoming packets under either
    /// session_id must resolve back to the same PeerState. Stale entries
    /// (from an expired dual-key window, or a session that was ultimately
    /// dropped) are removed opportunistically rather than being load-bearing
    /// for correctness — decrypt() re-validates against the peer's actual
    /// current/prev session state after this lookup, so a lingering map
    /// entry alone can never resurrect a truly-expired key.
    session_map: Arc<RwLock<HashMap<u32, [u8; 32]>>>,
    /// Handshakes THIS side initiated and is waiting on a response for, keyed
    /// by peer static public key. Removed on: response received (success),
    /// PENDING_HANDSHAKE_TIMEOUT_SECS elapsed with no response (peer never
    /// answered), or peer removal.
    pending_handshakes: Arc<RwLock<HashMap<[u8; 32], PendingHandshake>>>,
    stats: Arc<Mutex<TunnelStats>>,
    running: Arc<AtomicBool>,
    local_private_key: [u8; 32],
    local_public_key: [u8; 32],
    /// This node's long-term identity used to actually perform handshakes.
    /// Built once in `new()` via `NodeIdentity::from_existing_keys` around
    /// the real, persistent X25519 keypair the caller supplied — see that
    /// constructor's doc comment for why Dilithium/Kyber are still
    /// per-process-random (no config-level source for them exists yet).
    identity: Arc<NodeIdentity>,
    mesh_healer: Option<Arc<MeshHealer>>,
    threat_engine: Option<Arc<ThreatEngine>>,
    /// Name of the TUN interface currently owned by this engine (set in
    /// `start()`, read by `stop()` to issue the real interface deletion).
    tun_name: Mutex<Option<String>>,
    /// JoinHandles for the three tasks spawned in `start()`. Stored so
    /// `stop()` can `.abort()` them directly instead of relying on them to
    /// notice the `running` flag — a task blocked in `tun.read()` or
    /// `recv_from().await` will not wake up just because an atomic bool
    /// changed elsewhere, so without an explicit abort the fd (and the
    /// interface it holds open) is never actually released.
    task_handles: Arc<Mutex<TaskHandles>>,
}

#[derive(Default)]
struct TaskHandles {
    udp_recv: Option<JoinHandle<()>>,
    tun_read: Option<JoinHandle<()>>,
    keepalive: Option<JoinHandle<()>>,
}

impl TaskHandles {
    /// Abort every stored handle and drop them. Safe to call multiple
    /// times (aborting an already-finished/aborted handle is a no-op).
    fn abort_all(&mut self) {
        if let Some(h) = self.udp_recv.take() { h.abort(); }
        if let Some(h) = self.tun_read.take() { h.abort(); }
        if let Some(h) = self.keepalive.take() { h.abort(); }
    }
}

impl TunnelEngine {
    pub fn new(
        crypto: Arc<CryptoManager>,
        event_tx: mpsc::Sender<TunnelEvent>,
        private_key: [u8; 32],
        public_key: [u8; 32],
    ) -> Self {
        // Build a real handshake identity around the caller's (real, DH-valid
        // in every existing call site) X25519 keypair, rather than always
        // generating a fresh throwaway identity — see
        // NodeIdentity::from_existing_keys's doc comment for what is and
        // isn't persistent here (X25519 yes, Dilithium/Kyber not yet).
        //
        // from_existing_keys validates the pair and can fail if it doesn't
        // match; every real call site derives public_key directly from
        // private_key immediately beforehand so this should never happen in
        // practice. TunnelEngine::new() is infallible (all existing callers
        // use it as a plain expression, several inside Arc::new(...) with no
        // `?`/`.unwrap()`), so rather than changing that signature — or
        // panicking inside a constructor over a case that's a can't-happen
        // invariant today — this degrades to a freshly-generated identity
        // and logs loudly, which is a safe fallback (the node still works,
        // just without a persistent X25519 identity) rather than crashing
        // the whole daemon at startup.
        let identity = match NodeIdentity::from_existing_keys(private_key, public_key) {
            Ok(id) => id,
            Err(e) => {
                error!(
                    "NodeIdentity::from_existing_keys rejected the supplied keypair ({}) — \
                     falling back to a freshly-generated identity. This should not happen \
                     since callers derive public_key from private_key directly; if you see \
                     this, something upstream is passing a mismatched pair.",
                    e
                );
                NodeIdentity::generate().expect("NodeIdentity::generate must succeed (system RNG failure)")
            }
        };

        Self {
            crypto,
            event_tx,
            peers: Arc::new(RwLock::new(HashMap::new())),
            session_map: Arc::new(RwLock::new(HashMap::new())),
            pending_handshakes: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(Mutex::new(TunnelStats::default())),
            running: Arc::new(AtomicBool::new(false)),
            local_private_key: private_key,
            local_public_key: public_key,
            identity: Arc::new(identity),
            mesh_healer: None,
            threat_engine: None,
            tun_name: Mutex::new(None),
            task_handles: Arc::new(Mutex::new(TaskHandles::default())),
        }
    }

    pub fn with_mesh_healer(mut self, healer: Arc<MeshHealer>) -> Self {
        self.mesh_healer = Some(healer);
        self
    }

    pub fn with_threat_engine(mut self, engine: Arc<ThreatEngine>) -> Self {
        self.threat_engine = Some(engine);
        self
    }

    pub async fn add_peer(&self, config: PeerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // A peer starts with NO usable session at all — no keys, no
        // session_id — until a real handshake round-trip completes. The old
        // behavior here (blake3::hash(local_private_key || peer_public_key)
        // as an immediately-usable static key) is gone entirely: it had no
        // ephemeral component, no round-trip, and no forward secrecy, and
        // this is precisely the gap Stage 3 exists to close.
        let peer = Arc::new(PeerState {
            config: config.clone(),
            session: RwLock::new(None),
            last_handshake: std::sync::Mutex::new(None),
            rekey_deadline_crossed_at: std::sync::Mutex::new(None),
            last_received: std::sync::Mutex::new(std::time::Instant::now()),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
        });

        {
            let mut peers = self.peers.write().await;
            peers.insert(config.public_key, peer);
        }

        if let Some(ref healer) = self.mesh_healer {
            healer.register_peer(PeerId(hex::encode(config.public_key))).await;
        }

        info!(peer = ?hex::encode(&config.public_key[..8]), "Peer added (no session yet — awaiting handshake)");
        let _ = self.event_tx.send(TunnelEvent::PeerConnected {
            public_key: config.public_key,
            endpoint: config.endpoint.map(|e| e.to_string()).unwrap_or_default(),
        }).await;

        Ok(())
    }

    pub async fn remove_peer(&self, public_key: [u8; 32]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let removed = {
            let mut peers = self.peers.write().await;
            peers.remove(&public_key)
        };
        if let Some(peer) = removed {
            // Clean up every session_map entry this peer owns: its current
            // session_id, and — if a dual-key window happens to be active —
            // the previous session_id too. Both are looked up from the
            // peer's own state rather than assumed, since either or both may
            // be absent (peer never completed a handshake at all, or its
            // dual-key window already expired).
            let mut ids_to_remove = Vec::new();
            {
                let session = peer.session.read().await;
                if let Some(ref s) = *session {
                    ids_to_remove.push(s.session_id);
                    if let Some(ref prev) = *s.prev.lock().await {
                        ids_to_remove.push(prev.session_id);
                    }
                }
            }
            if !ids_to_remove.is_empty() {
                let mut session_map = self.session_map.write().await;
                for id in ids_to_remove {
                    session_map.remove(&id);
                }
            }
            self.pending_handshakes.write().await.remove(&public_key);

            let _ = self.event_tx.send(TunnelEvent::PeerDisconnected { public_key }).await;
            info!(peer = ?hex::encode(&public_key[..8]), "Peer removed");
        }
        Ok(())
    }

    pub async fn get_stats(&self) -> TunnelStats {
        self.stats.lock().await.clone()
    }

    /// Build a fresh `Session` from a completed handshake's derived keys.
    /// Used for both the initial handshake and every subsequent rekey — a
    /// rekey always installs a brand-new `Session` rather than mutating one
    /// in place, so nonce counters and the rekey-deadline clock both start
    /// clean with the new keys.
    ///
    /// IMPORTANT: `session_id` must be IDENTICAL on both sides of the same
    /// handshake, since the sender writes its own local session_id into
    /// every packet header and the receiver looks packets up by that exact
    /// value in ITS OWN session_map. A prior version of this function chose
    /// `rand::random::<u32>()` independently on each side — which, since
    /// this function runs separately on the initiator (in
    /// `process_response`'s caller) and the responder (in
    /// `process_init_and_respond`'s caller), produced two DIFFERENT random
    /// IDs for what was supposed to be one logical session. That bug was
    /// caught by `test_real_data_packet_round_trips_end_to_end`: the
    /// initiator's real handshake completed fine on both sides (nothing
    /// about session_id agreement is checked by the handshake itself), but
    /// the very first real data packet sent afterward came back
    /// `InboundOutcome::Dropped` — the receiving side's session_map had no
    /// entry for the sender's self-chosen session_id at all, so
    /// `DecryptHelper::decrypt` correctly (from its own perspective) reported
    /// "Unknown session". Deriving session_id deterministically from the
    /// handshake's own derived keys — which ARE guaranteed identical on both
    /// sides (`init_result.send_key == resp_result.recv_key` and vice versa,
    /// per `test_full_handshake`/`test_ikm_byte_identical_both_sides` in
    /// handshake.rs) — fixes this without touching handshake.rs's wire
    /// format at all: sort the two keys into a canonical order before
    /// hashing so BOTH sides, regardless of which one is "send" vs "recv"
    /// locally, hash the exact same two 32-byte values in the exact same
    /// order.
    fn session_from_handshake_result(result: &HandshakeResult) -> Session {
        let mut sorted_keys = [result.send_key.to_vec(), result.recv_key.to_vec()];
        sorted_keys.sort();
        let mut hash_input = Vec::with_capacity(64);
        hash_input.extend_from_slice(&sorted_keys[0]);
        hash_input.extend_from_slice(&sorted_keys[1]);
        let id_hash = blake3::hash(&hash_input);
        let session_id = u32::from_le_bytes(id_hash.as_bytes()[..4].try_into().unwrap());

        Session {
            session_id,
            send_key: result.send_key,
            recv_key: result.recv_key,
            send_nonce: AtomicU64::new(1), // Start at 1, 0 is reserved
            recv_bitmap: Mutex::new(ReplayWindow::default()),
            established_at: Instant::now(),
            bytes_transferred: AtomicU64::new(0),
            prev: Mutex::new(None),
        }
    }

    /// Encrypt a packet for sending to a peer under an established session.
    fn encrypt_packet(
        &self,
        plaintext: &[u8],
        session: &Session,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let nonce_counter = session.send_nonce.fetch_add(1, Ordering::SeqCst);

        // Build nonce: 4 bytes zero padding + 8 bytes counter (little-endian)
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());

        // Encrypt with ChaCha20-Poly1305
        let ciphertext = self.crypto.encrypt_chacha(plaintext, &session.send_key, &nonce)?;

        // Build packet: type(1) + session_id(4) + nonce_counter(8) + ciphertext
        let mut packet = Vec::with_capacity(HEADER_SIZE + ciphertext.len());
        packet.push(PACKET_DATA);
        packet.extend_from_slice(&session.session_id.to_le_bytes());
        packet.extend_from_slice(&nonce_counter.to_le_bytes());
        packet.extend_from_slice(&ciphertext);
        session.bytes_transferred.fetch_add(packet.len() as u64, Ordering::Relaxed);

        Ok(packet)
    }

    /// Start the tunnel on a given listen address
    ///
    /// Spawns three tasks:
    /// 1. UDP listener: receives encrypted packets, decrypts, runs threat analysis, writes to TUN
    /// 2. TUN reader: reads IP packets from TUN, encrypts, sends to peer UDP endpoint
    /// 3. Keepalive: sends keepalives every 25s, detects peer timeouts, invokes MeshHealer
    pub async fn start(
        &self,
        listen_addr: SocketAddr,
        tun_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Tunnel already running".into());
        }

        // Startup-time stale-interface check: if an interface with this
        // name already exists (e.g. left behind by an unclean previous
        // run/crash), remove it first rather than failing TUNSETIFF or
        // silently ending up with a second, conflicting interface.
        if Self::interface_exists(tun_name) {
            warn!(name = tun_name, "Stale TUN interface found at startup, removing before recreate");
            Self::delete_interface(tun_name);
        }

        self.running.store(true, Ordering::SeqCst);

        let socket = Arc::new(UdpSocket::bind(listen_addr).await?);
        info!(addr = %listen_addr, "UDP socket bound");

        // Create TUN device
        let tun = Self::create_tun_device(tun_name)?;
        let tun = Arc::new(tun);
        info!(name = tun_name, "TUN device created");

        {
            let mut stored_name = self.tun_name.lock().await;
            *stored_name = Some(tun_name.to_string());
        }

        let recv_socket = socket.clone();
        let send_socket = socket.clone();

        // === Task 1: UDP → TUN (receive encrypted, decrypt, threat-analyze, write to TUN) ===
        let running1 = self.running.clone();
        let stats1 = self.stats.clone();
        let peers1 = self.peers.clone();
        let session_map1 = self.session_map.clone();
        let crypto1 = self.crypto.clone();
        let event_tx1 = self.event_tx.clone();
        let tun1 = tun.clone();
        let threat_engine1 = self.threat_engine.clone();
        let handles1 = self.task_handles.clone();
        let identity1 = self.identity.clone();
        let pending_handshakes1 = self.pending_handshakes.clone();

        let udp_recv_handle = tokio::spawn(async move {
            let decryptor = DecryptHelper {
                crypto: crypto1, peers: peers1.clone(), session_map: session_map1.clone(),
            };
            let handshaker = HandshakeHelper {
                identity: identity1,
                peers: peers1,
                session_map: session_map1,
                pending_handshakes: pending_handshakes1,
                stats: stats1.clone(),
                event_tx: event_tx1.clone(),
            };
            let mut buf = vec![0u8; MAX_PACKET_SIZE];
            while running1.load(Ordering::SeqCst) {
                match recv_socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        let outcome = process_inbound_datagram(
                            &recv_socket,
                            addr,
                            &buf[..len],
                            &decryptor,
                            &handshaker,
                            &threat_engine1,
                            &event_tx1,
                        ).await;

                        if let InboundOutcome::Data { plaintext } = outcome {
                            #[cfg(target_os = "linux")]
                            {
                                use std::io::Write;
                                if let Err(e) = (&*tun1).write_all(&plaintext) {
                                    error!("TUN write error: {}", e);
                                }
                            }
                            // On non-Linux, `plaintext` is otherwise
                            // unused in this branch (there's no TUN to
                            // write it to) — reference it explicitly rather
                            // than let an unused-binding warning appear on
                            // that target.
                            #[cfg(not(target_os = "linux"))]
                            {
                                let _ = &plaintext;
                            }
                            let mut s = stats1.lock().await;
                            s.packets_received += 1;
                            s.bytes_received += len as u64;
                        }
                    }
                    Err(e) => {
                        if running1.load(Ordering::SeqCst) {
                            error!("UDP recv: {} (fatal, shutting down tunnel)", e);
                            running1.store(false, Ordering::SeqCst);
                            let _ = event_tx1.send(TunnelEvent::FatalError {
                                task: "udp_recv",
                                reason: e.to_string(),
                            }).await;
                            // Abort sibling tasks immediately rather than
                            // leaving the tunnel half-alive (e.g. TUN read
                            // and keepalive still running with no one
                            // reading the socket anymore).
                            handles1.lock().await.abort_all();
                        }
                        break;
                    }
                }
            }
        });

        // === Task 2: TUN → UDP (read from TUN, encrypt, send to peer) ===
        let running2 = self.running.clone();
        let stats2 = self.stats.clone();
        let peers2 = self.peers.clone();
        let crypto2 = self.crypto.clone();
        let tun2 = tun.clone();
        let event_tx2 = self.event_tx.clone();
        let handles2 = self.task_handles.clone();
        let identity2 = self.identity.clone();
        let session_map2 = self.session_map.clone();
        let pending_handshakes2 = self.pending_handshakes.clone();

        let tun_read_handle = tokio::spawn(async move {
            let handshaker2 = HandshakeHelper {
                identity: identity2,
                peers: peers2.clone(),
                session_map: session_map2,
                pending_handshakes: pending_handshakes2,
                stats: stats2.clone(),
                event_tx: event_tx2.clone(),
            };
            let mut buf = vec![0u8; MAX_PACKET_SIZE];
            loop {
                if !running2.load(Ordering::SeqCst) { break; }

                #[cfg(target_os = "linux")]
                {
                    use std::io::Read;
                    match (&*tun2).read(&mut buf) {
                        Ok(len) => {
                            let ip_packet = &buf[..len];
                            let peers = peers2.read().await;
                            for (pk, peer) in peers.iter() {
                                let endpoint = match peer.config.endpoint {
                                    Some(e) => e,
                                    None => continue,
                                };

                                // A peer with no established session yet (no
                                // real handshake round-trip has completed)
                                // has no usable send_key at all — there is
                                // deliberately no static/fallback key to fall
                                // back to anymore, so outbound traffic to
                                // this peer is silently dropped until a
                                // handshake completes, rather than sent
                                // unencrypted or under some ad-hoc key.
                                //
                                // BUG FIX (found via manual two-instance
                                // verification -- no automated test caught
                                // this since none exercised a real start()):
                                // nothing anywhere ever initiated the FIRST
                                // handshake with a brand-new peer. add_peer()
                                // only logs and waits, and the keepalive
                                // task's rekey trigger requires an existing
                                // session (session_exists must already be
                                // true). A fresh two-peer setup never
                                // connected at all -- confirmed empirically:
                                // the kernel queued outbound ICMP packets to
                                // the TUN device, but zero ever reached the
                                // wire. Kicking off a handshake attempt here,
                                // on the first outbound-traffic attempt that
                                // finds no session, closes that gap.
                                // initiate_handshake is itself a no-op if one
                                // is already pending for this peer, so this
                                // is safe to call on every dropped packet
                                // while a handshake is in flight.
                                let session_guard = peer.session.read().await;
                                let session = match session_guard.as_ref() {
                                    Some(s) => s,
                                    None => {
                                        if let Err(e) = handshaker2.initiate_handshake(&send_socket, *pk, peer).await {
                                            debug!(peer = ?hex::encode(&pk[..8]), error = %e, "Handshake initiation (from outbound traffic) failed");
                                        }
                                        continue;
                                    }
                                };

                                let nonce_counter = session.send_nonce.fetch_add(1, Ordering::SeqCst);
                                let mut nonce = [0u8; NONCE_SIZE];
                                nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());

                                match crypto2.encrypt_chacha(ip_packet, &session.send_key, &nonce) {
                                    Ok(ciphertext) => {
                                        let mut pkt = Vec::with_capacity(HEADER_SIZE + ciphertext.len());
                                        pkt.push(PACKET_DATA);
                                        pkt.extend_from_slice(&session.session_id.to_le_bytes());
                                        pkt.extend_from_slice(&nonce_counter.to_le_bytes());
                                        pkt.extend_from_slice(&ciphertext);

                                        if let Err(e) = send_socket.send_to(&pkt, endpoint).await {
                                            error!("UDP send: {}", e);
                                        }
                                        session.bytes_transferred.fetch_add(pkt.len() as u64, Ordering::Relaxed);
                                        peer.bytes_sent.fetch_add(pkt.len() as u64, Ordering::Relaxed);
                                        let mut s = stats2.lock().await;
                                        s.packets_sent += 1;
                                        s.bytes_sent += pkt.len() as u64;
                                    }
                                    Err(e) => { error!("Encrypt: {}", e); }
                                }
                            }
                        }
                        Err(e) => {
                            if running2.load(Ordering::SeqCst) {
                                error!("TUN read: {} (fatal, shutting down tunnel)", e);
                                running2.store(false, Ordering::SeqCst);
                                let _ = event_tx2.send(TunnelEvent::FatalError {
                                    task: "tun_read",
                                    reason: e.to_string(),
                                }).await;
                                // Without this, the UDP-recv and keepalive
                                // tasks keep running with a dead TUN reader
                                // — the tunnel silently goes half-alive
                                // while `status` still reports it as up.
                                handles2.lock().await.abort_all();
                            }
                            break;
                        }
                    }
                }

                #[cfg(not(target_os = "linux"))]
                {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    warn!("TUN not supported on this platform");
                    break;
                }
            }
        });

        // === Task 3: Keepalive sender + peer timeout detector + rekey trigger ===
        let running3 = self.running.clone();
        let peers3 = self.peers.clone();
        let crypto3 = self.crypto.clone();
        let event_tx3 = self.event_tx.clone();
        let healer3 = self.mesh_healer.clone();
        let socket3 = socket.clone();
        let identity3 = self.identity.clone();
        let session_map3 = self.session_map.clone();
        let pending_handshakes3 = self.pending_handshakes.clone();
        let stats3 = self.stats.clone();
        let event_tx3b = self.event_tx.clone();

        let keepalive_handle = tokio::spawn(async move {
            let handshaker = HandshakeHelper {
                identity: identity3,
                peers: peers3.clone(),
                session_map: session_map3.clone(),
                pending_handshakes: pending_handshakes3,
                stats: stats3,
                event_tx: event_tx3b,
            };

            let mut interval = tokio::time::interval(Duration::from_secs(KEEPALIVE_INTERVAL_SECS));
            while running3.load(Ordering::SeqCst) {
                interval.tick().await;
                if !running3.load(Ordering::SeqCst) { break; }

                // Expire any pending (sent, no response yet) handshake older
                // than PENDING_HANDSHAKE_TIMEOUT_SECS. Without this, a peer
                // that never responds to an INIT would permanently block
                // initiate_handshake's "already pending, don't re-send"
                // guard for that peer — meaning a rekey (or even an initial
                // connection) would get exactly ONE attempt for the entire
                // REKEY_REJECT_AFTER_SECS grace period instead of being
                // retried. This also bounds how long InitiatorState (which
                // holds ephemeral key material) is kept around for a peer
                // that may simply be offline.
                {
                    let mut pending = handshaker.pending_handshakes.write().await;
                    let before = pending.len();
                    pending.retain(|_, p| {
                        p.started_at.elapsed() < Duration::from_secs(PENDING_HANDSHAKE_TIMEOUT_SECS)
                    });
                    let expired = before - pending.len();
                    if expired > 0 {
                        debug!(count = expired, "Expired {} stale pending handshake(s) with no response", expired);
                    }
                }

                let mut timed_out = Vec::new();
                // Peers that independently earned Dead status this tick:
                // BOTH missed 3 keepalives AND their session is past its own
                // rekey deadline without a successful rehandshake. These
                // skip heal_peer's retry loop entirely — see
                // MeshHealer::mark_dead's doc comment for why.
                let mut newly_dead = Vec::new();
                // Peers whose session crossed REKEY_REJECT_AFTER_SECS past
                // its own deadline with no successful rekey — dropped
                // entirely (not just flagged), independent of keepalive
                // status.
                let mut sessions_to_reject = Vec::new();

                {
                    let peers = peers3.read().await;
                    for (pk, peer) in peers.iter() {
                        // Send keepalive packet (encrypted empty payload) —
                        // only meaningful if a session actually exists; a
                        // peer still waiting on its very first handshake has
                        // no key to encrypt a keepalive under at all.
                        if let Some(endpoint) = peer.config.endpoint {
                            let session_guard = peer.session.read().await;
                            if let Some(ref session) = *session_guard {
                                let nonce_counter = session.send_nonce.fetch_add(1, Ordering::SeqCst);
                                let mut nonce = [0u8; NONCE_SIZE];
                                nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());

                                match crypto3.encrypt_chacha(&[], &session.send_key, &nonce) {
                                    Ok(ciphertext) => {
                                        let mut pkt = Vec::with_capacity(HEADER_SIZE + ciphertext.len());
                                        pkt.push(PACKET_KEEPALIVE);
                                        pkt.extend_from_slice(&session.session_id.to_le_bytes());
                                        pkt.extend_from_slice(&nonce_counter.to_le_bytes());
                                        pkt.extend_from_slice(&ciphertext);

                                        if let Err(e) = socket3.send_to(&pkt, endpoint).await {
                                            debug!("Keepalive send error: {}", e);
                                        }
                                        session.bytes_transferred.fetch_add(pkt.len() as u64, Ordering::Relaxed);
                                    }
                                    Err(e) => { debug!("Keepalive encrypt error: {}", e); }
                                }
                            }
                        }

                        // --- Rekey trigger & deadline/staleness bookkeeping ---
                        // Scoped so the session read-guard is dropped before
                        // we potentially need a write-guard later (dropping
                        // the session on reject-after) — holding a read lock
                        // across that would deadlock against ourselves.
                        let (past_deadline, session_exists) = {
                            let session_guard = peer.session.read().await;
                            match session_guard.as_ref() {
                                Some(session) => (session.is_past_rekey_deadline(), true),
                                None => (false, false),
                            }
                        };

                        if session_exists && past_deadline {
                            // Record the FIRST time we noticed this session
                            // is past its deadline (idempotent — does not
                            // overwrite an earlier timestamp on subsequent
                            // ticks where it's still past-deadline).
                            {
                                let mut crossed = peer.rekey_deadline_crossed_at.lock().unwrap();
                                if crossed.is_none() {
                                    *crossed = Some(Instant::now());
                                }
                            }

                            // Proactively rehandshake — reuses the exact
                            // same initiate_handshake path as the very first
                            // connection. initiate_handshake itself is a
                            // no-op if a handshake is already pending for
                            // this peer, so this is safe to call every tick
                            // while waiting on a response.
                            if let Err(e) = handshaker.initiate_handshake(&socket3, *pk, peer).await {
                                debug!(peer = ?hex::encode(&pk[..8]), error = %e, "Rekey handshake initiation failed");
                            }

                            let crossed_at = *peer.rekey_deadline_crossed_at.lock().unwrap();
                            let time_since_crossed = crossed_at.map(|t| t.elapsed()).unwrap_or(Duration::ZERO);

                            // Staleness policy: BOTH missed-3-keepalives AND
                            // past rekey deadline without a successful
                            // rehandshake => straight to Dead, skip
                            // heal_peer's retry loop. Checked here (not only
                            // in the missed-keepalive branch below) since
                            // this is the branch that actually knows the
                            // session is past-deadline.
                            let missed_keepalives = peer.last_received.lock()
                                .map(|last| last.elapsed() > Duration::from_secs(KEEPALIVE_INTERVAL_SECS * 3))
                                .unwrap_or(false);
                            if missed_keepalives {
                                newly_dead.push(*pk);
                            }

                            // Reject-after: 180s past the deadline with no
                            // successful rekey => drop the session entirely,
                            // independent of keepalive status.
                            if time_since_crossed >= Duration::from_secs(REKEY_REJECT_AFTER_SECS) {
                                sessions_to_reject.push(*pk);
                            }
                        }

                        // Check for timeout (75s = 3 missed keepalives) —
                        // pre-existing behavior, unchanged for a peer whose
                        // session is NOT past its rekey deadline (a
                        // perfectly healthy, freshly-established session
                        // that simply hasn't heard from its peer in 75s
                        // still goes through the normal heal_peer retry
                        // path, exactly as before this stage).
                        let elapsed = peer.last_received.lock()
                            .map(|last| last.elapsed())
                            .unwrap_or(Duration::from_secs(0));
                        if elapsed > Duration::from_secs(KEEPALIVE_INTERVAL_SECS * 3) && !newly_dead.contains(pk) {
                            timed_out.push((*pk, peer.config.endpoint));
                        }
                    }
                }

                // Drop sessions that crossed the reject-after grace period.
                // Done as a second pass (after releasing the read lock on
                // `peers` above) since this needs a write lock on the
                // individual peer's `session` slot.
                for pk in &sessions_to_reject {
                    let peers = peers3.read().await;
                    if let Some(peer) = peers.get(pk) {
                        // Collect BOTH the current session_id and (if a
                        // dual-key window happened to still be active) the
                        // previous one, so session_map doesn't accumulate a
                        // permanently-stale entry for either. Not required
                        // for correctness — DecryptHelper::decrypt always
                        // re-checks peer.session itself, so a lingering
                        // session_map entry pointing at a peer with no
                        // session can never decrypt anything — but left
                        // unbounded this HashMap would otherwise grow by one
                        // entry per rekey/reject-after-drop over a
                        // long-running daemon's lifetime.
                        let mut ids_to_remove = Vec::new();
                        let dropped_session_id = {
                            let mut session_guard = peer.session.write().await;
                            if let Some(s) = session_guard.take() {
                                ids_to_remove.push(s.session_id);
                                if let Some(prev) = s.prev.into_inner() {
                                    ids_to_remove.push(prev.session_id);
                                }
                                Some(s.session_id)
                            } else {
                                None
                            }
                        };
                        if !ids_to_remove.is_empty() {
                            let mut session_map = session_map3.write().await;
                            for id in ids_to_remove {
                                session_map.remove(&id);
                            }
                        }
                        *peer.rekey_deadline_crossed_at.lock().unwrap() = None;
                        if let Some(sid) = dropped_session_id {
                            warn!(
                                peer = ?hex::encode(&pk[..8]),
                                session_id = sid,
                                "Session dropped: {}s past its rekey deadline with no successful rehandshake",
                                REKEY_REJECT_AFTER_SECS
                            );
                        }
                    }
                }

                // Mark independently-stale peers Dead directly, bypassing
                // heal_peer's retry/back-off loop entirely (see
                // MeshHealer::mark_dead's doc comment).
                for pk in &newly_dead {
                    warn!(
                        peer = ?hex::encode(&pk[..8]),
                        "Peer marked Dead directly: missed 3 keepalives AND past its own rekey deadline \
                         without a successful rehandshake — a session past its own security expiry \
                         should not be kept alive by heartbeats alone"
                    );
                    let _ = event_tx3.send(TunnelEvent::PeerDisconnected { public_key: *pk }).await;
                    if let Some(ref healer) = healer3 {
                        healer.mark_dead(&PeerId(hex::encode(pk))).await;
                    }
                }

                // Ordinary heartbeat-timeout peers (NOT already handled via
                // the newly_dead fast path above) go through the pre-existing
                // heal_peer retry/back-off loop, unchanged from before this
                // stage.
                for (pk, _endpoint) in timed_out {
                    warn!(peer = ?hex::encode(&pk[..8]), "Peer timed out (no response for 75s)");
                    let _ = event_tx3.send(TunnelEvent::PeerDisconnected { public_key: pk }).await;

                    if let Some(ref healer) = healer3 {
                        let peer_id = PeerId(hex::encode(pk));
                        let healer = healer.clone();
                        tokio::spawn(async move {
                            healer.heal_peer(
                                &peer_id,
                                DisconnectReason::HeartbeatTimeout,
                                5,
                                || async { false },
                            ).await;
                        });
                    }
                }
            }
        });

        // Store handles immediately so stop() (or a sibling task's fatal-error
        // path) can abort them directly rather than trusting the `running`
        // flag to be noticed by a task blocked in a syscall.
        {
            let mut handles = self.task_handles.lock().await;
            handles.udp_recv = Some(udp_recv_handle);
            handles.tun_read = Some(tun_read_handle);
            handles.keepalive = Some(keepalive_handle);
        }

        info!("Tunnel started: UDP {} <-> TUN {}", listen_addr, tun_name);
        Ok(())
    }

    /// Stop the tunnel engine: abort all supervised tasks (even if they're
    /// blocked in a syscall) and delete the real TUN interface.
    ///
    /// Flipping `running` alone is not sufficient here: the UDP-recv task
    /// may be parked in `recv_from().await` and the TUN-read task may be
    /// parked in a blocking `tun.read()` syscall, neither of which wakes up
    /// just because an unrelated atomic changed elsewhere. Without the
    /// explicit `.abort()` calls below, those tasks — and the fd/interface
    /// they hold open — could outlive `stop()` indefinitely.
    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);

        // Abort every spawned task directly. This is the actual root-cause
        // fix: termination no longer depends on a task noticing a flag.
        self.task_handles.lock().await.abort_all();

        // Delete the real TUN interface so it doesn't leak (matching how
        // it was created: this codebase manages TUN via raw ioctl/no
        // netlink crate, and cli.rs already shells out to `ip` for
        // interface configuration, so `ip link delete` matches the
        // existing mechanism rather than introducing a new dependency).
        let tun_name = {
            let mut stored_name = self.tun_name.lock().await;
            stored_name.take()
        };
        if let Some(name) = tun_name {
            Self::delete_interface(&name);
        }

        info!("Tunnel engine stopped");
    }

    /// Check whether a network interface with this name currently exists.
    #[cfg(target_os = "linux")]
    fn interface_exists(name: &str) -> bool {
        Command::new("ip")
            .args(["link", "show", name])
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "linux"))]
    fn interface_exists(_name: &str) -> bool {
        false
    }

    /// Delete a network interface by name. Best-effort: logs on failure
    /// (e.g. the interface is already gone) rather than propagating an
    /// error, since this is called both from `stop()` (where the interface
    /// is expected to exist) and from the startup-time stale-interface
    /// check (where it may or may not still be there).
    #[cfg(target_os = "linux")]
    fn delete_interface(name: &str) {
        match Command::new("ip").args(["link", "delete", name]).output() {
            Ok(out) if out.status.success() => {
                info!(name = name, "TUN interface deleted");
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                warn!(name = name, error = %stderr.trim(), "ip link delete failed (interface may already be gone)");
            }
            Err(e) => {
                warn!(name = name, error = %e, "Failed to invoke `ip link delete`");
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn delete_interface(_name: &str) {}

    /// Create a TUN device (Linux only)
    #[cfg(target_os = "linux")]
    fn create_tun_device(name: &str) -> Result<std::fs::File, Box<dyn std::error::Error + Send + Sync>> {
        use std::fs::OpenOptions;
        use std::os::unix::io::AsRawFd;

        let tun_fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;

        // ioctl to create TUN device
        #[repr(C)]
        struct IfReq {
            ifr_name: [u8; 16],
            ifr_flags: i16,
            _pad: [u8; 22],
        }

        const IFF_TUN: i16 = 0x0001;
        const IFF_NO_PI: i16 = 0x1000;
        const TUNSETIFF: libc::c_ulong = 0x400454CA;

        let mut req = IfReq {
            ifr_name: [0u8; 16],
            ifr_flags: IFF_TUN | IFF_NO_PI,
            _pad: [0u8; 22],
        };

        let name_bytes = name.as_bytes();
        let copy_len = std::cmp::min(name_bytes.len(), 15);
        req.ifr_name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

        let ret = unsafe {
            libc::ioctl(tun_fd.as_raw_fd(), TUNSETIFF as _, &mut req as *mut IfReq)
        };

        if ret < 0 {
            return Err(format!("ioctl TUNSETIFF failed: {}", std::io::Error::last_os_error()).into());
        }

        info!(name = name, "TUN device created via ioctl");
        Ok(tun_fd)
    }

    #[cfg(not(target_os = "linux"))]
    fn create_tun_device(_name: &str) -> Result<std::fs::File, Box<dyn std::error::Error + Send + Sync>> {
        Err("TUN device only supported on Linux".into())
    }
}

// ============================================================================
// Decrypt Helper (Send-safe for tokio::spawn)
// ============================================================================

struct DecryptHelper {
    crypto: Arc<CryptoManager>,
    peers: Arc<RwLock<HashMap<[u8; 32], Arc<PeerState>>>>,
    session_map: Arc<RwLock<HashMap<u32, [u8; 32]>>>,
}

impl DecryptHelper {
    /// Decrypt a PACKET_DATA or PACKET_KEEPALIVE payload, resolving which
    /// key to use against EITHER the peer's current session OR — during the
    /// dual-key window right after a rekey — its previous session.
    ///
    /// Matching is done by session_id, not by "try both keys": the packet's
    /// session_id header tells us unambiguously which of the two sessions
    /// (if either) it was encrypted under, so there's no need to guess by
    /// attempting decryption twice. This also means a session_id that
    /// matches neither the current nor a still-valid previous session is
    /// rejected immediately as "Unknown session" without touching AEAD at
    /// all — same behavior as before this stage for a session_id that was
    /// never valid, now also covering "was valid but its dual-key window
    /// already expired".
    async fn decrypt(&self, packet: &[u8]) -> Result<(Vec<u8>, [u8; 32]), Box<dyn std::error::Error + Send + Sync>> {
        if packet.len() < HEADER_SIZE + TAG_SIZE {
            return Err("Packet too short".into());
        }
        let session_id = u32::from_le_bytes(packet[1..5].try_into().unwrap());
        let nonce_counter = u64::from_le_bytes(packet[5..13].try_into().unwrap());
        let ciphertext = &packet[HEADER_SIZE..];

        let peer_key = {
            let sm = self.session_map.read().await;
            *sm.get(&session_id).ok_or("Unknown session")?
        };
        let peers = self.peers.read().await;
        let peer = peers.get(&peer_key).ok_or("Peer not found")?;

        let session_guard = peer.session.read().await;
        let session = session_guard.as_ref().ok_or("Peer has no established session")?;

        // Case 1: packet's session_id matches the CURRENT session.
        if session.session_id == session_id {
            {
                let mut replay = session.recv_bitmap.lock().await;
                if !replay.check_and_update(nonce_counter) {
                    return Err("Replay detected".into());
                }
            }
            let mut nonce = [0u8; NONCE_SIZE];
            nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());
            let plaintext = self.crypto.decrypt_chacha(ciphertext, &session.recv_key, &nonce)?;
            session.bytes_transferred.fetch_add(packet.len() as u64, Ordering::Relaxed);
            peer.bytes_received.fetch_add(packet.len() as u64, Ordering::Relaxed);
            return Ok((plaintext, peer_key));
        }

        // Case 2: packet's session_id matches the PREVIOUS session, and its
        // dual-key window hasn't expired yet.
        {
            let mut prev_guard = session.prev.lock().await;
            if let Some(ref prev) = *prev_guard {
                if prev.session_id == session_id {
                    if Instant::now() >= prev.expires_at {
                        // Window expired — proactively clear it so we don't
                        // keep re-checking a dead key on every packet, and
                        // report the same "Unknown session" a caller would
                        // see for any other invalid session_id.
                        *prev_guard = None;
                        return Err("Unknown session (previous session's dual-key window has expired)".into());
                    }
                    {
                        let mut replay = prev.recv_bitmap.lock().await;
                        if !replay.check_and_update(nonce_counter) {
                            return Err("Replay detected".into());
                        }
                    }
                    let mut nonce = [0u8; NONCE_SIZE];
                    nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());
                    let plaintext = self.crypto.decrypt_chacha(ciphertext, &prev.recv_key, &nonce)?;
                    // Deliberately NOT counted toward the (new) current
                    // session's bytes_transferred rekey-trigger budget —
                    // this traffic was encrypted under keys already
                    // scheduled for retirement, it shouldn't extend the new
                    // session's deadline.
                    peer.bytes_received.fetch_add(packet.len() as u64, Ordering::Relaxed);
                    return Ok((plaintext, peer_key));
                }
            }
        }

        Err("Unknown session".into())
    }
}

// ============================================================================
// Handshake Helper (Send-safe for tokio::spawn — mirrors DecryptHelper)
// ============================================================================

/// Bundles everything needed to drive handshakes (initiating, responding,
/// installing resulting sessions) from inside a spawned task, which only has
/// access to `Arc`-cloned fields rather than a live `&TunnelEngine` —
/// `start()` takes `&self`, not `self: Arc<Self>`, so a spawned task cannot
/// hold a `TunnelEngine` reference across `.await` points. Same shape/reason
/// as the pre-existing `DecryptHelper`.
struct HandshakeHelper {
    identity: Arc<NodeIdentity>,
    peers: Arc<RwLock<HashMap<[u8; 32], Arc<PeerState>>>>,
    session_map: Arc<RwLock<HashMap<u32, [u8; 32]>>>,
    pending_handshakes: Arc<RwLock<HashMap<[u8; 32], PendingHandshake>>>,
    stats: Arc<Mutex<TunnelStats>>,
    event_tx: mpsc::Sender<TunnelEvent>,
}

impl HandshakeHelper {
    /// Look up a configured peer by the UDP source address a packet arrived
    /// from. Peers are statically configured with known endpoints (NAT
    /// traversal / dynamic discovery is out of scope for this stage), so
    /// this is a simple linear match against each peer's configured
    /// endpoint rather than any kind of discovery mechanism.
    async fn find_peer_by_endpoint(&self, addr: SocketAddr) -> Option<([u8; 32], Arc<PeerState>)> {
        let peers = self.peers.read().await;
        peers.iter()
            .find(|(_, p)| p.config.endpoint == Some(addr))
            .map(|(k, p)| (*k, p.clone()))
    }

    /// Initiate a handshake with a peer: build the INIT message, record
    /// InitiatorState as a pending handshake, and send it over the given
    /// socket to the peer's statically-configured endpoint.
    ///
    /// Used for BOTH the very first connection to a peer and every
    /// subsequent proactive rekey — both reuse this exact same code path, as
    /// required (a rekey is not a structurally different operation from the
    /// initial handshake, just triggered by a different condition).
    ///
    /// Does nothing (returns Ok without sending) if a handshake is already
    /// pending for this peer, to avoid flooding a slow-to-respond peer with
    /// duplicate INIT messages every time this is called (e.g. once per
    /// keepalive tick while waiting on a response).
    async fn initiate_handshake(
        &self,
        socket: &UdpSocket,
        peer_key: [u8; 32],
        peer: &Arc<PeerState>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            let pending = self.pending_handshakes.read().await;
            if pending.contains_key(&peer_key) {
                debug!(peer = ?hex::encode(&peer_key[..8]), "Handshake already pending, not re-initiating");
                return Ok(());
            }
        }

        let endpoint = match peer.config.endpoint {
            Some(e) => e,
            None => {
                // NAT traversal / dynamic peer discovery is explicitly out of
                // scope for this stage — we only ever handshake with peers
                // at their already-known, statically-configured endpoint. A
                // peer with no endpoint configured simply cannot be
                // initiated to.
                return Err("Cannot initiate handshake: peer has no statically-configured endpoint".into());
            }
        };

        let (init_msg, initiator_state) = handshake::build_init_message(&self.identity, &peer_key)?;

        {
            let mut pending = self.pending_handshakes.write().await;
            pending.insert(peer_key, PendingHandshake {
                initiator_state,
                started_at: Instant::now(),
            });
        }

        socket.send_to(&init_msg, endpoint).await?;
        info!(peer = ?hex::encode(&peer_key[..8]), endpoint = %endpoint, "Handshake INIT sent");

        Ok(())
    }

    /// Install a newly-completed session for a peer, handling the dual-key
    /// transition if a session already existed (rekey case) vs. installing
    /// the very first session for this peer (initial handshake case).
    ///
    /// Per the settled dual-key policy: the OLD session's recv_key remains
    /// valid for DUAL_KEY_WINDOW_SECS so still-in-flight incoming packets
    /// encrypted under it aren't dropped, while send_key switches to the NEW
    /// session immediately (this function is what makes the new key the one
    /// `encrypt` paths see from this point on, by installing it as `peer`'s
    /// current session before returning).
    async fn install_session(
        &self,
        peer_key: [u8; 32],
        peer: &Arc<PeerState>,
        result: &HandshakeResult,
    ) {
        let new_session = TunnelEngine::session_from_handshake_result(result);
        let new_session_id = new_session.session_id;

        let old_session_map_entry = {
            let mut session_slot = peer.session.write().await;
            let old = session_slot.take();
            *session_slot = Some(new_session);
            old
        };

        // Register the new session_id so incoming packets under it resolve
        // back to this peer.
        {
            let mut session_map = self.session_map.write().await;
            session_map.insert(new_session_id, peer_key);
        }

        if let Some(old_session) = old_session_map_entry {
            // Rekey case: carry the old session's recv side forward into the
            // new session's `prev` slot for the dual-key window. The old
            // session_id stays resolvable in session_map for exactly that
            // long too (DecryptHelper::decrypt re-validates against
            // `prev.expires_at` itself, so a lingering map entry past expiry
            // is harmless — see session_map's field doc on TunnelEngine).
            let old_session_id = old_session.session_id;
            let prev = PrevSession {
                session_id: old_session.session_id,
                recv_key: old_session.recv_key,
                recv_bitmap: old_session.recv_bitmap,
                expires_at: Instant::now() + Duration::from_secs(DUAL_KEY_WINDOW_SECS),
            };
            let session_slot = peer.session.read().await;
            if let Some(ref s) = *session_slot {
                *s.prev.lock().await = Some(prev);
            }
            info!(
                peer = ?hex::encode(&peer_key[..8]),
                old_session = old_session_id,
                new_session = new_session_id,
                "Rekey completed — dual-key window active for {}s",
                DUAL_KEY_WINDOW_SECS
            );
        } else {
            info!(peer = ?hex::encode(&peer_key[..8]), session = new_session_id, "Initial handshake completed — session established");
        }

        *peer.last_handshake.lock().unwrap() = Some(Instant::now());
        *peer.rekey_deadline_crossed_at.lock().unwrap() = None;

        self.pending_handshakes.write().await.remove(&peer_key);

        {
            let mut stats = self.stats.lock().await;
            stats.handshakes_completed += 1;
        }

        let _ = self.event_tx.send(TunnelEvent::HandshakeCompleted { peer: peer_key }).await;
    }

    /// Handle an incoming PACKET_HANDSHAKE_INIT: process it, and — ONLY if
    /// the claimed initiator identity matches an already-configured peer —
    /// build and send a response, installing the new session on our side.
    ///
    /// IMPORTANT authorization note: `handshake::process_init_and_respond`
    /// verifies the Dilithium signature is SELF-CONSISTENT (i.e. it really
    /// was produced by whichever Dilithium key the message claims belongs to
    /// the sender) but does NOT check that claimed static key against any
    /// allowlist — that would let literally any node that can complete a
    /// valid-looking handshake with its own freshly-generated keypair obtain
    /// a session, which defeats the entire point of PhantomMesh only
    /// trusting statically-configured peers. That authorization check is
    /// done HERE, immediately after signature verification succeeds and
    /// BEFORE a response is built or sent at all — an unrecognized peer gets
    /// no response, not just no installed session.
    async fn handle_handshake_init(
        &self,
        socket: &UdpSocket,
        src_addr: SocketAddr,
        init_msg: &[u8],
    ) {
        let (resp_msg, result) = match handshake::process_init_and_respond(&self.identity, init_msg) {
            Ok(v) => v,
            Err(e) => {
                // Malformed/tampered/unsigned-by-anyone-we'd-trust INIT from
                // an unauthenticated remote source. Log and drop — never
                // panic, never respond. This is precisely the boundary the
                // prompt calls out: a malformed/truncated/short UDP packet
                // from an unauthenticated remote peer must never panic the
                // whole tunnel engine.
                debug!(source = %src_addr, error = %e, "Rejected handshake INIT (parse/signature failure)");
                return;
            }
        };

        let claimed_initiator_key = result.peer_identity;

        // Authorization check: is this claimed static key one of OUR
        // configured peers? (See doc comment above for why this can't be
        // skipped or merged into process_init_and_respond itself.)
        let peer = {
            let peers = self.peers.read().await;
            peers.get(&claimed_initiator_key).cloned()
        };
        let peer = match peer {
            Some(p) => p,
            None => {
                warn!(
                    source = %src_addr,
                    claimed_peer = ?hex::encode(&claimed_initiator_key[..8]),
                    "Rejected handshake INIT: claimed identity is not a configured peer"
                );
                return;
            }
        };

        // Simultaneous-handshake tie-break. If we ALSO have a pending
        // outbound handshake to this exact peer right now, both sides
        // triggered initiate_handshake at nearly the same moment (a real,
        // plausible race -- found via manual two-instance testing, not a
        // hypothetical: two freshly-started peers, or two peers recovering
        // from an outage together, can easily both decide to connect within
        // the same few milliseconds). Left unhandled, each side processes
        // the OTHER's INIT as responder and independently completes a
        // DIFFERENT handshake with different ephemeral keys -- both sides
        // end up with non-matching sessions and the tunnel never actually
        // passes traffic, even though both sides log "handshake completed".
        //
        // Deterministic tie-break (mirrors the intent of WireGuard's own
        // handling of simultaneous initiation): the side with the
        // numerically smaller static public key always wins and keeps its
        // own outbound attempt going; the other side defers, abandoning its
        // own pending outbound handshake and processing the winner's INIT
        // normally as responder. This guarantees exactly one handshake
        // actually completes, so both sides converge on the same
        // session_from_handshake_result-derived session_id.
        {
            let mut pending = self.pending_handshakes.write().await;
            if pending.contains_key(&claimed_initiator_key) {
                if self.identity.x25519_public < claimed_initiator_key {
                    debug!(
                        peer = ?hex::encode(&claimed_initiator_key[..8]),
                        "Simultaneous handshake detected -- we win the tie-break (smaller static key), ignoring peer's INIT"
                    );
                    return;
                }
                pending.remove(&claimed_initiator_key);
                debug!(
                    peer = ?hex::encode(&claimed_initiator_key[..8]),
                    "Simultaneous handshake detected -- we lose the tie-break (larger static key), abandoning our own INIT and processing theirs"
                );
            }
        }

        // Send the response before installing the session locally — if the
        // send fails there's no point holding a session the peer will never
        // see a response for (they'll just time out and retry the INIT).
        if let Err(e) = socket.send_to(&resp_msg, src_addr).await {
            error!(peer = ?hex::encode(&claimed_initiator_key[..8]), error = %e, "Failed to send handshake RESP");
            return;
        }
        debug!(peer = ?hex::encode(&claimed_initiator_key[..8]), "Handshake RESP sent");

        self.install_session(claimed_initiator_key, &peer, &result).await;
    }

    /// Handle an incoming PACKET_HANDSHAKE_RESP: match it to a pending
    /// handshake WE initiated (by source address, since peers are
    /// statically configured with known endpoints — see
    /// find_peer_by_endpoint), process it, and install the resulting
    /// session.
    async fn handle_handshake_resp(&self, src_addr: SocketAddr, resp_msg: &[u8]) {
        let (peer_key, peer) = match self.find_peer_by_endpoint(src_addr).await {
            Some(v) => v,
            None => {
                debug!(source = %src_addr, "Received handshake RESP from an address matching no configured peer, dropping");
                return;
            }
        };

        let initiator_state = {
            let mut pending = self.pending_handshakes.write().await;
            match pending.remove(&peer_key) {
                Some(p) => p.initiator_state,
                None => {
                    debug!(peer = ?hex::encode(&peer_key[..8]), "Received handshake RESP but no handshake is pending for this peer, dropping");
                    return;
                }
            }
        };

        let result = match handshake::process_response(&self.identity, &initiator_state, resp_msg) {
            Ok(r) => r,
            Err(e) => {
                // Malformed/tampered RESP, or a RESP that doesn't verify
                // against the identity we were expecting. Log and drop —
                // never panic. The pending handshake was already removed
                // above; if the real peer's response arrives late/retried,
                // it will no longer match a pending entry and will itself be
                // dropped by the `None` branch above, which is acceptable —
                // the peer will notice via its own INIT timeout and can
                // re-initiate.
                warn!(peer = ?hex::encode(&peer_key[..8]), error = %e, "Rejected handshake RESP (parse/signature failure)");
                return;
            }
        };

        self.install_session(peer_key, &peer, &result).await;
    }
}

/// Result of processing one inbound UDP datagram via
/// `process_inbound_datagram`. Callers use this to decide what to do with
/// decrypted data (e.g. `start()`'s real Task 1 writes it to the TUN device;
/// tests can just assert on it directly without needing a TUN device at
/// all).
///
/// Deliberately private (not `pub`/`pub(crate)`): this, `process_inbound_datagram`,
/// `DecryptHelper`, and `HandshakeHelper` are only ever constructed/called
/// from within this file — including the live-handshake integration tests
/// added in this stage, which live in an inline `#[cfg(test)] mod` in THIS
/// file rather than a separate `tests/*.rs` file specifically so they can
/// reach these internals directly. A `tests/*.rs` integration test compiles
/// as a separate crate and can only see genuinely `pub` items, which would
/// have forced these into the crate's public API just to be testable —
/// undesirable for a security-sensitive internal dispatch/session-lifecycle
/// surface that callers outside this module have no business touching.
#[derive(Debug)]
enum InboundOutcome {
    /// A PACKET_DATA packet was successfully decrypted.
    Data { plaintext: Vec<u8> },
    /// A PACKET_KEEPALIVE was received and processed (last_received
    /// updated); nothing further to do.
    KeepaliveReceived,
    /// A PACKET_HANDSHAKE_INIT or PACKET_HANDSHAKE_RESP was dispatched to
    /// the handshake helper (accepted or rejected — both cases are handled
    /// entirely inside the helper, including sending any response).
    HandshakeHandled,
    /// The datagram was too short, had an unrecognized session_id, failed
    /// AEAD decryption, or otherwise didn't parse — logged and dropped.
    /// Never a panic, by construction (every fallible step here returns
    /// `Result`/`Option` and is matched explicitly).
    Dropped,
}

/// Process exactly one already-received UDP datagram: dispatch handshake
/// packets to `handshaker`, or attempt to decrypt as data/keepalive via
/// `decryptor`. This is the SAME dispatch logic `start()`'s real UDP-receive
/// task runs (factored out here rather than duplicated) so that:
///
///   1. Tests can drive the actual production dispatch path over real
///      loopback UDP sockets without needing a TUN device (which requires
///      CAP_NET_ADMIN, unavailable in a plain `cargo test` process — see
///      the existing precedent/reasoning in
///      `test_stop_aborts_blocked_tasks_within_timeout` for the same
///      constraint in Stage 2).
///   2. There is exactly one copy of this logic, so a test exercising it
///      really does prove something about `start()`'s real behavior rather
///      than about a re-implementation that could silently drift from it.
///
/// Never panics on malformed/truncated/garbage input — every fallible
/// operation here (`decryptor.decrypt`, the handshake helper's internal
/// parsing) is routed through an explicit `Result`/`Option` match, never an
/// `.unwrap()`/`.expect()` on attacker-controlled data.
async fn process_inbound_datagram(
    socket: &UdpSocket,
    src_addr: SocketAddr,
    datagram: &[u8],
    decryptor: &DecryptHelper,
    handshaker: &HandshakeHelper,
    threat_engine: &Option<Arc<ThreatEngine>>,
    event_tx: &mpsc::Sender<TunnelEvent>,
) -> InboundOutcome {
    if datagram.is_empty() {
        return InboundOutcome::Dropped;
    }
    let packet_type = datagram[0];

    // Handshake packets are dispatched BEFORE any attempt to decrypt as data
    // — they arrive in plaintext (the handshake protocol itself is what
    // establishes keys in the first place) and must never be run through
    // DecryptHelper::decrypt, which would just fail with "Unknown
    // session"/AEAD-failure noise for what is actually a well-formed
    // handshake message.
    if packet_type == PACKET_HANDSHAKE_INIT {
        handshaker.handle_handshake_init(socket, src_addr, datagram).await;
        return InboundOutcome::HandshakeHandled;
    }
    if packet_type == PACKET_HANDSHAKE_RESP {
        handshaker.handle_handshake_resp(src_addr, datagram).await;
        return InboundOutcome::HandshakeHandled;
    }

    match decryptor.decrypt(datagram).await {
        Ok((plaintext, peer_key)) => {
            // Update last_received timestamp for this peer.
            {
                let peers = decryptor.peers.read().await;
                if let Some(peer) = peers.get(&peer_key) {
                    if let Ok(mut last) = peer.last_received.lock() {
                        *last = std::time::Instant::now();
                    }
                }
            }

            if packet_type == PACKET_KEEPALIVE {
                debug!(peer = ?hex::encode(&peer_key[..8]), "Keepalive received");
                return InboundOutcome::KeepaliveReceived;
            }

            // Threat analysis (detection only — never drops packets).
            if let Some(ref te) = threat_engine {
                let source = src_addr.to_string();
                if let Some(threat) = te.analyze_packet(&plaintext, Some(&source)).await {
                    warn!(
                        threat_id = %threat.signature_id,
                        severity = ?threat.severity,
                        source = %source,
                        "Threat detected in decrypted packet"
                    );
                    let _ = event_tx.send(TunnelEvent::ThreatSignature {
                        signature: plaintext[..plaintext.len().min(64)].to_vec(),
                        source,
                    }).await;
                }
            }

            InboundOutcome::Data { plaintext }
        }
        Err(e) => {
            debug!("Decrypt failed: {}", e);
            InboundOutcome::Dropped
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_layer::handshake::NodeIdentity;

    /// Generate a real X25519 keypair for test-only TunnelEngine construction.
    /// Replaces the old CryptoManager::generate_keypair() (deleted in the
    /// Stage 1 crypto fix — it produced two independent random values with
    /// no cryptographic relationship between them at all). NodeIdentity's
    /// x25519_private/x25519_public fields ARE a real DH keypair.
    fn test_keypair() -> ([u8; 32], [u8; 32]) {
        let identity = NodeIdentity::generate().unwrap();
        (identity.x25519_private, identity.x25519_public)
    }

    #[test]
    fn test_replay_window_accepts_new() {
        let mut w = ReplayWindow::default();
        assert!(w.check_and_update(1));
        assert!(w.check_and_update(2));
        assert!(w.check_and_update(3));
    }

    #[test]
    fn test_replay_window_rejects_duplicate() {
        let mut w = ReplayWindow::default();
        assert!(w.check_and_update(1));
        assert!(!w.check_and_update(1)); // duplicate
    }

    #[test]
    fn test_replay_window_rejects_zero() {
        let mut w = ReplayWindow::default();
        assert!(!w.check_and_update(0)); // reserved
    }

    #[test]
    fn test_replay_window_accepts_out_of_order() {
        let mut w = ReplayWindow::default();
        assert!(w.check_and_update(5));
        assert!(w.check_and_update(3)); // out of order but within window
        assert!(w.check_and_update(1));
        assert!(!w.check_and_update(3)); // already seen
    }

    #[test]
    fn test_replay_window_rejects_too_old() {
        let mut w = ReplayWindow::default();
        assert!(w.check_and_update(REPLAY_WINDOW_SIZE + 100));
        assert!(!w.check_and_update(1)); // too old, outside window
    }

    #[test]
    fn test_replay_window_large_jump() {
        let mut w = ReplayWindow::default();
        assert!(w.check_and_update(1));
        assert!(w.check_and_update(10000)); // large jump
        assert!(w.check_and_update(9999)); // within new window
        assert!(!w.check_and_update(1)); // now too old
    }

    #[tokio::test]
    async fn test_tunnel_engine_creation() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, _rx) = mpsc::channel(100);
        let keys = test_keypair();
        let engine = TunnelEngine::new(crypto, tx, keys.0, keys.1);
        let stats = engine.get_stats().await;
        assert_eq!(stats.packets_sent, 0);
        assert_eq!(stats.peers_connected, 0);
    }

    #[tokio::test]
    async fn test_add_remove_peer() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, _rx) = mpsc::channel(100);
        let keys = test_keypair();
        let engine = TunnelEngine::new(crypto, tx, keys.0, keys.1);

        let peer_config = PeerConfig {
            public_key: [42u8; 32],
            endpoint: Some("127.0.0.1:51820".parse().unwrap()),
            allowed_ips: vec!["10.0.0.0/24".into()],
            preshared_key: None,
            persistent_keepalive: Some(25),
        };

        engine.add_peer(peer_config).await.unwrap();
        let peers = engine.peers.read().await;
        assert_eq!(peers.len(), 1);
        drop(peers);

        engine.remove_peer([42u8; 32]).await.unwrap();
        let peers = engine.peers.read().await;
        assert_eq!(peers.len(), 0);
    }

    /// Drive a real handshake between two fresh identities and return just
    /// the initiator-side HandshakeResult. Used by tests that need a
    /// genuinely-derived session's keys (rather than fabricated arbitrary
    /// bytes) to exercise encrypt/packet-format logic, without each such
    /// test having to spin up two real TunnelEngine instances over a real
    /// socket (that's what the dedicated live-handshake integration tests
    /// added in this stage are for).
    fn real_handshake_result() -> HandshakeResult {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();
        let (init_msg, init_state) = handshake::build_init_message(&initiator, &responder.x25519_public).unwrap();
        let (resp_msg, _resp_result) = handshake::process_init_and_respond(&responder, &init_msg).unwrap();
        handshake::process_response(&initiator, &init_state, &resp_msg).unwrap()
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, _rx) = mpsc::channel(100);
        let keys = test_keypair();
        let engine = TunnelEngine::new(crypto, tx, keys.0, keys.1);

        let peer_config = PeerConfig {
            public_key: [42u8; 32],
            endpoint: Some("127.0.0.1:51820".parse().unwrap()),
            allowed_ips: vec![],
            preshared_key: Some([99u8; 32]),
            persistent_keepalive: None,
        };

        engine.add_peer(peer_config).await.unwrap();

        // A peer has NO usable session immediately after add_peer() as of
        // this stage — install one directly here (simulating a completed
        // handshake) with real, genuinely-derived keys, since this test is
        // specifically about encrypt_packet's framing/nonce logic, not
        // about handshake completion itself.
        let result = real_handshake_result();
        {
            let peers = engine.peers.read().await;
            let peer = peers.get(&[42u8; 32]).unwrap();
            let session = TunnelEngine::session_from_handshake_result(&result);
            *peer.session.write().await = Some(session);
        }

        let peers = engine.peers.read().await;
        let peer = peers.get(&[42u8; 32]).unwrap();
        let session_guard = peer.session.read().await;
        let session = session_guard.as_ref().unwrap();

        let plaintext = b"Hello PhantomMesh VPN!";
        let encrypted = engine.encrypt_packet(plaintext, session).unwrap();

        // Verify header
        assert_eq!(encrypted[0], PACKET_DATA);
        assert!(encrypted.len() > HEADER_SIZE + TAG_SIZE);

        // Verify nonce incremented
        let nonce = session.send_nonce.load(Ordering::SeqCst);
        assert_eq!(nonce, 2); // Started at 1, incremented to 2

        // Verify decryption round-trips correctly under the SAME session's
        // recv_key (using the counterpart key derived on the other side of
        // the same real handshake would require the full two-engine
        // integration test — this specifically checks encrypt_packet's own
        // framing is internally consistent: the nonce/session_id it wrote
        // into the header are exactly what's needed to decrypt it back).
        let nonce_counter = u64::from_le_bytes(encrypted[5..13].try_into().unwrap());
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        nonce_bytes[4..12].copy_from_slice(&nonce_counter.to_le_bytes());
        let ciphertext = &encrypted[HEADER_SIZE..];
        let decrypted = crypto_manager_decrypt(&session.send_key, ciphertext, &nonce_bytes);
        assert_eq!(decrypted, plaintext);
    }

    /// Local helper mirroring CryptoManager::decrypt_chacha exactly, used
    /// only so test_encrypt_decrypt_roundtrip can verify encrypt_packet's
    /// output is genuinely decryptable (not just "looks like a packet") —
    /// encrypt_packet is a private method on TunnelEngine and its internal
    /// CryptoManager instance isn't exposed to tests, so this stands in
    /// with a fresh CryptoManager (ChaCha20-Poly1305 is a pure function of
    /// key+nonce+ciphertext, so a second instance decrypts identically).
    fn crypto_manager_decrypt(key: &[u8; 32], ciphertext: &[u8], nonce: &[u8; NONCE_SIZE]) -> Vec<u8> {
        let crypto = CryptoManager::new().unwrap();
        crypto.decrypt_chacha(ciphertext, key, nonce).unwrap()
    }

    #[tokio::test]
    async fn test_keepalive_packet_format() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, _rx) = mpsc::channel(100);
        let keys = test_keypair();
        let engine = TunnelEngine::new(crypto.clone(), tx, keys.0, keys.1);

        let peer_config = PeerConfig {
            public_key: [42u8; 32],
            endpoint: Some("127.0.0.1:51820".parse().unwrap()),
            allowed_ips: vec![],
            preshared_key: None,
            persistent_keepalive: Some(25),
        };
        engine.add_peer(peer_config).await.unwrap();

        let result = real_handshake_result();
        {
            let peers = engine.peers.read().await;
            let peer = peers.get(&[42u8; 32]).unwrap();
            let session = TunnelEngine::session_from_handshake_result(&result);
            *peer.session.write().await = Some(session);
        }

        let peers = engine.peers.read().await;
        let peer = peers.get(&[42u8; 32]).unwrap();
        let session_guard = peer.session.read().await;
        let session = session_guard.as_ref().unwrap();

        // Build a keepalive: encrypted empty payload with PACKET_KEEPALIVE type
        let nonce_counter = session.send_nonce.fetch_add(1, Ordering::SeqCst);
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());
        let ciphertext = crypto.encrypt_chacha(&[], &session.send_key, &nonce).unwrap();

        let mut pkt = Vec::with_capacity(HEADER_SIZE + ciphertext.len());
        pkt.push(PACKET_KEEPALIVE);
        pkt.extend_from_slice(&session.session_id.to_le_bytes());
        pkt.extend_from_slice(&nonce_counter.to_le_bytes());
        pkt.extend_from_slice(&ciphertext);

        assert_eq!(pkt[0], PACKET_KEEPALIVE);
        // Keepalive = header + Poly1305 tag only (empty plaintext)
        assert_eq!(pkt.len(), HEADER_SIZE + TAG_SIZE);
    }

    #[tokio::test]
    async fn test_mesh_healer_wiring() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, mut rx) = mpsc::channel(100);
        let keys = test_keypair();
        let healer = Arc::new(MeshHealer::new(0)); // 0s timeout for testing
        let engine = TunnelEngine::new(crypto, tx, keys.0, keys.1)
            .with_mesh_healer(healer.clone());

        let peer_config = PeerConfig {
            public_key: [42u8; 32],
            endpoint: Some("127.0.0.1:51820".parse().unwrap()),
            allowed_ips: vec![],
            preshared_key: None,
            persistent_keepalive: Some(25),
        };
        engine.add_peer(peer_config).await.unwrap();

        // Drain the PeerConnected event
        let _ = rx.recv().await;

        // Verify peer was registered with healer
        let peer_id = PeerId(hex::encode([42u8; 32]));
        let status = healer.peer_status(&peer_id).await;
        assert!(status.is_some());
    }

    #[tokio::test]
    async fn test_threat_engine_wiring() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, _rx) = mpsc::channel(100);
        let keys = test_keypair();
        let threat = Arc::new(ThreatEngine::new().unwrap());
        let engine = TunnelEngine::new(crypto, tx, keys.0, keys.1)
            .with_threat_engine(threat.clone());

        assert!(engine.threat_engine.is_some());
    }

    #[tokio::test]
    async fn test_peer_timeout_detection() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, mut rx) = mpsc::channel(100);
        let keys = test_keypair();
        let healer = Arc::new(MeshHealer::new(0));
        let engine = TunnelEngine::new(crypto, tx, keys.0, keys.1)
            .with_mesh_healer(healer.clone());

        let peer_config = PeerConfig {
            public_key: [42u8; 32],
            endpoint: Some("127.0.0.1:51820".parse().unwrap()),
            allowed_ips: vec![],
            preshared_key: None,
            persistent_keepalive: Some(25),
        };
        engine.add_peer(peer_config).await.unwrap();

        // Drain PeerConnected event
        let _ = rx.recv().await;

        // Set last_received far in the past to simulate timeout
        {
            let peers = engine.peers.read().await;
            let peer = peers.get(&[42u8; 32]).unwrap();
            let mut last = peer.last_received.lock().unwrap();
            *last = std::time::Instant::now() - Duration::from_secs(100);
        }

        // Verify peer is timed out by checking elapsed time
        let peers = engine.peers.read().await;
        let peer = peers.get(&[42u8; 32]).unwrap();
        let elapsed = peer.last_received.lock().unwrap().elapsed();
        assert!(elapsed > Duration::from_secs(75));
    }

    /// Proves the actual root-cause fix for Stage 2: tasks terminate on
    /// `.abort()` even when they're "blocked" in a loop that never checks
    /// any flag — i.e. termination does NOT depend on a task noticing that
    /// an AtomicBool changed elsewhere.
    ///
    /// This deliberately does NOT call `TunnelEngine::start()` — doing so
    /// requires TUNSETIFF (CAP_NET_ADMIN), which a plain `cargo test`
    /// process does not have (verified directly against this box: an
    /// unprivileged ioctl(TUNSETIFF) call returns `Operation not
    /// permitted`). Gating this test on root would make it fail for
    /// privilege reasons unrelated to the abort logic being tested, so
    /// instead it exercises `TaskHandles::abort_all()` — the exact
    /// mechanism `stop()` calls — against stand-in tasks that reproduce
    /// the real failure mode: a `loop { }` with no `running`-flag check
    /// and no natural await-point-driven cancellation opportunity beyond
    /// what `.abort()` itself forces.
    #[tokio::test]
    async fn test_stop_aborts_blocked_tasks_within_timeout() {
        // Exercise the exact Arc<Mutex<TaskHandles>> shape TunnelEngine
        // itself uses, so this test covers the real storage/locking path
        // and not just a bare TaskHandles value.
        let handles = Arc::new(Mutex::new(TaskHandles::default()));

        // Three stand-ins for the UDP-recv, TUN-read, and keepalive tasks.
        // Each spins forever and would never exit on its own — exactly the
        // failure mode described in the Stage 2 plan (blocked in a
        // "syscall" that won't wake up just because a flag changed). Each
        // holds a oneshot sender that only fires when the task itself is
        // torn down, so we can prove real termination, not just that
        // abort() was called.
        let (done_tx1, done_rx1) = tokio::sync::oneshot::channel::<()>();
        let (done_tx2, done_rx2) = tokio::sync::oneshot::channel::<()>();
        let (done_tx3, done_rx3) = tokio::sync::oneshot::channel::<()>();

        let udp_stub = tokio::spawn(async move {
            let _guard = done_tx1;
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        });
        let tun_stub = tokio::spawn(async move {
            let _guard = done_tx2;
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        });
        let keepalive_stub = tokio::spawn(async move {
            let _guard = done_tx3;
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        });

        {
            let mut h = handles.lock().await;
            h.udp_recv = Some(udp_stub);
            h.tun_read = Some(tun_stub);
            h.keepalive = Some(keepalive_stub);
        }

        // This is what TunnelEngine::stop() calls.
        handles.lock().await.abort_all();

        // The real assertion: each task must actually terminate within a
        // bounded timeout after abort_all() — not merely that a flag was
        // set. Each oneshot::Receiver only resolves when its task's
        // `_guard` is dropped, i.e. the task has actually torn down.
        let r1 = tokio::time::timeout(Duration::from_secs(2), done_rx1).await;
        let r2 = tokio::time::timeout(Duration::from_secs(2), done_rx2).await;
        let r3 = tokio::time::timeout(Duration::from_secs(2), done_rx3).await;

        assert!(r1.is_ok(), "UDP-recv stand-in did not terminate within 2s of abort_all()");
        assert!(r2.is_ok(), "TUN-read stand-in did not terminate within 2s of abort_all()");
        assert!(r3.is_ok(), "Keepalive stand-in did not terminate within 2s of abort_all()");

        // Storage must be empty afterward too.
        let h = handles.lock().await;
        assert!(h.udp_recv.is_none());
        assert!(h.tun_read.is_none());
        assert!(h.keepalive.is_none());
    }
}

// ============================================================================
// Stage 3 live-handshake integration tests
//
// Two real TunnelEngine instances, each bound to a real loopback UDP socket,
// configured as each other's statically-configured peer. Deliberately does
// NOT call TunnelEngine::start() (which requires a real TUN device via
// TUNSETIFF/CAP_NET_ADMIN, unavailable in a plain `cargo test` process — see
// test_stop_aborts_blocked_tasks_within_timeout's doc comment in `mod tests`
// above for the same constraint, verified directly against this box in
// Stage 2). Instead, these tests drive `process_inbound_datagram` directly —
// the SAME dispatch function start()'s real Task 1 calls — over real
// sockets, so what's being exercised is the actual production
// handshake/session/decrypt code path, not a re-implementation of it. TUN
// itself (reading/writing IP packets to a kernel interface) is a separate,
// already-covered concern from Stage 2 and is not what this stage is about.
//
// This lives as an inline #[cfg(test)] module in THIS file rather than a
// separate tests/*.rs integration test specifically so it can reach
// DecryptHelper/HandshakeHelper/process_inbound_datagram/PeerState/Session
// directly — those are deliberately kept private (not pub/pub(crate)) since
// nothing outside this module has legitimate reason to touch them, and a
// tests/*.rs file (a separate crate) can only see genuinely `pub` items.
// ============================================================================

#[cfg(test)]
mod live_handshake_tests {
    use super::*;
    use crate::security_layer::handshake::NodeIdentity;
    use tokio::net::UdpSocket as TokioUdpSocket;

    /// One fully-set-up side of a two-engine test: a real TunnelEngine, a
    /// real loopback UDP socket bound to `port`, and the event receiver so
    /// tests can drain/assert on TunnelEvents (HandshakeCompleted, etc.).
    struct TestNode {
        engine: Arc<TunnelEngine>,
        socket: Arc<TokioUdpSocket>,
        addr: SocketAddr,
        event_rx: mpsc::Receiver<TunnelEvent>,
    }

    async fn make_test_node() -> TestNode {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let identity = NodeIdentity::generate().unwrap();
        let (event_tx, event_rx) = mpsc::channel(256);
        let engine = Arc::new(TunnelEngine::new(
            crypto,
            event_tx,
            identity.x25519_private,
            identity.x25519_public,
        ));
        // Bind to port 0 (OS-assigned free port) rather than a hardcoded
        // port — avoids any flakiness/collision from a fixed port already
        // being in use on a shared test box, and this box's 2-core CPU
        // means multiple test binaries could plausibly run with some
        // overlap.
        let socket = Arc::new(TokioUdpSocket::bind("127.0.0.1:0").await.unwrap());
        let addr = socket.local_addr().unwrap();
        TestNode { engine, socket, addr, event_rx }
    }

    /// Configure `a` and `b` as each other's statically-configured peer
    /// (matching this stage's explicit scope: only already-known, static
    /// endpoints — no discovery).
    async fn configure_as_peers(a: &TestNode, b: &TestNode) {
        a.engine.add_peer(PeerConfig {
            public_key: b.engine.local_public_key,
            endpoint: Some(b.addr),
            allowed_ips: vec![],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }).await.unwrap();

        b.engine.add_peer(PeerConfig {
            public_key: a.engine.local_public_key,
            endpoint: Some(a.addr),
            allowed_ips: vec![],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }).await.unwrap();
    }

    /// Build the same DecryptHelper + HandshakeHelper pair Task 1 builds
    /// inside `start()`, for a given node — used by test-driven receive
    /// loops below so they exercise the real `process_inbound_datagram`
    /// dispatch rather than a re-implementation.
    fn helpers_for(node: &TestNode) -> (DecryptHelper, HandshakeHelper) {
        let decryptor = DecryptHelper {
            crypto: node.engine.crypto.clone(),
            peers: node.engine.peers.clone(),
            session_map: node.engine.session_map.clone(),
        };
        let handshaker = HandshakeHelper {
            identity: node.engine.identity.clone(),
            peers: node.engine.peers.clone(),
            session_map: node.engine.session_map.clone(),
            pending_handshakes: node.engine.pending_handshakes.clone(),
            stats: node.engine.stats.clone(),
            event_tx: node.engine.event_tx.clone(),
        };
        (decryptor, handshaker)
    }

    /// Receive and dispatch exactly one datagram on `node`'s socket, via the
    /// real `process_inbound_datagram` path, with a timeout so a test that's
    /// wrong about whether a packet is coming doesn't hang forever.
    async fn recv_one(node: &TestNode, timeout: Duration) -> Result<InboundOutcome, &'static str> {
        let (decryptor, handshaker) = helpers_for(node);
        let mut buf = vec![0u8; MAX_PACKET_SIZE];
        let (len, addr) = tokio::time::timeout(timeout, node.socket.recv_from(&mut buf))
            .await
            .map_err(|_| "timed out waiting for a datagram")?
            .map_err(|_| "socket recv_from error")?;
        Ok(process_inbound_datagram(
            &node.socket,
            addr,
            &buf[..len],
            &decryptor,
            &handshaker,
            &node.engine.threat_engine,
            &node.engine.event_tx,
        ).await)
    }

    /// Drive a full, real handshake to completion between two nodes:
    /// `initiator` sends INIT to `responder`, `responder` processes it and
    /// sends RESP, `initiator` processes the RESP. Every step goes over the
    /// REAL loopback sockets via `process_inbound_datagram` — nothing here
    /// calls handshake.rs functions directly.
    async fn drive_handshake_to_completion(initiator: &TestNode, responder: &TestNode) {
        // initiator -> responder: send INIT (uses initiator's real
        // TunnelEngine, via the same HandshakeHelper.initiate_handshake
        // start()'s keepalive task calls for the initial connection AND
        // every rekey).
        let (_dec, initiator_handshaker) = helpers_for(initiator);
        let peer_pubkey = responder.engine.local_public_key;
        let peer_arc = {
            let peers = initiator.engine.peers.read().await;
            peers.get(&peer_pubkey).unwrap().clone()
        };
        initiator_handshaker
            .initiate_handshake(&initiator.socket, peer_pubkey, &peer_arc)
            .await
            .expect("initiate_handshake should succeed with a real configured peer+endpoint");

        // responder receives the INIT, processes it, sends RESP back —
        // all inside process_inbound_datagram/handle_handshake_init.
        let outcome = recv_one(responder, Duration::from_secs(5)).await
            .expect("responder should receive the INIT within 5s over real loopback");
        assert!(matches!(outcome, InboundOutcome::HandshakeHandled), "expected responder to handle a handshake packet, got {:?}", outcome);

        // initiator receives the RESP, completes the handshake on its side.
        let outcome = recv_one(initiator, Duration::from_secs(5)).await
            .expect("initiator should receive the RESP within 5s over real loopback");
        assert!(matches!(outcome, InboundOutcome::HandshakeHandled), "expected initiator to handle a handshake packet, got {:?}", outcome);
    }

    /// Assert a node has reached a real, established session for the given
    /// peer public key — i.e. actually completed a handshake, not merely
    /// "no error was returned anywhere".
    async fn assert_has_established_session(node: &TestNode, peer_pubkey: [u8; 32]) -> u32 {
        let peers = node.engine.peers.read().await;
        let peer = peers.get(&peer_pubkey).expect("peer should be configured");
        let session_guard = peer.session.read().await;
        let session = session_guard.as_ref().expect(
            "peer should have an established session after a completed handshake — \
             finding None here would mean the handshake round-trip did not actually \
             install usable keys, i.e. exactly the gap this stage exists to close"
        );
        session.session_id
    }

    #[tokio::test]
    async fn test_real_handshake_completes_on_both_sides() {
        let node_a = make_test_node().await;
        let node_b = make_test_node().await;
        configure_as_peers(&node_a, &node_b).await;

        drive_handshake_to_completion(&node_a, &node_b).await;

        // Both sides must have a REAL session installed — not just "no
        // error was returned". Different assertions than "handshake
        // returned Ok" deliberately, since that alone wouldn't catch a bug
        // where install_session silently failed to actually store the
        // session.
        let session_id_a = assert_has_established_session(&node_a, node_b.engine.local_public_key).await;
        let session_id_b = assert_has_established_session(&node_b, node_a.engine.local_public_key).await;

        // Sanity: the two sides' session IDs are independently
        // randomly-generated (see session_from_handshake_result), so
        // there's no reason to expect them equal — but each side's own ID
        // should be nonzero/consistent across repeated reads (i.e. actually
        // stored, not re-generated on every access).
        let session_id_a_again = assert_has_established_session(&node_a, node_b.engine.local_public_key).await;
        assert_eq!(session_id_a, session_id_a_again, "session_id changed across reads with no rekey — session is not stably stored");
        let _ = session_id_b;

        // Both engines' handshake-completed stats incremented.
        assert_eq!(node_a.engine.get_stats().await.handshakes_completed, 1);
        assert_eq!(node_b.engine.get_stats().await.handshakes_completed, 1);
    }

    #[tokio::test]
    async fn test_real_handshake_emits_handshake_completed_event() {
        let mut node_a = make_test_node().await;
        let node_b = make_test_node().await;
        configure_as_peers(&node_a, &node_b).await;

        drive_handshake_to_completion(&node_a, &node_b).await;

        // Drain node_a's event channel looking for HandshakeCompleted —
        // proves the previously-declared-but-never-emitted event variant is
        // now actually fired on a real completed handshake.
        let mut saw_handshake_completed = false;
        for _ in 0..10 {
            match tokio::time::timeout(Duration::from_millis(200), node_a.event_rx.recv()).await {
                Ok(Some(TunnelEvent::HandshakeCompleted { peer })) => {
                    assert_eq!(peer, node_b.engine.local_public_key);
                    saw_handshake_completed = true;
                    break;
                }
                Ok(Some(_)) => continue,
                _ => break,
            }
        }
        assert!(saw_handshake_completed, "expected a real TunnelEvent::HandshakeCompleted after a completed handshake");
    }

    #[tokio::test]
    async fn test_real_data_packet_round_trips_end_to_end() {
        let node_a = make_test_node().await;
        let node_b = make_test_node().await;
        configure_as_peers(&node_a, &node_b).await;
        drive_handshake_to_completion(&node_a, &node_b).await;

        // Encrypt a real payload on A's side using A's REAL established
        // session (the exact code path encrypt_packet/Task 2 would use).
        let plaintext = b"real end-to-end PhantomMesh data packet, stage 3";
        let encrypted = {
            let peers = node_a.engine.peers.read().await;
            let peer = peers.get(&node_b.engine.local_public_key).unwrap();
            let session_guard = peer.session.read().await;
            let session = session_guard.as_ref().unwrap();
            node_a.engine.encrypt_packet(plaintext, session).unwrap()
        };

        // Send it over the REAL loopback socket to B.
        node_a.socket.send_to(&encrypted, node_b.addr).await.unwrap();

        // B receives and decrypts it via the real dispatch path.
        let outcome = recv_one(&node_b, Duration::from_secs(5)).await
            .expect("node_b should receive the data packet within 5s over real loopback");
        match outcome {
            InboundOutcome::Data { plaintext: decrypted } => {
                assert_eq!(decrypted, plaintext, "decrypted payload does not match what was sent");
            }
            other => panic!("expected InboundOutcome::Data, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_forced_rekey_succeeds_and_dual_key_window_lets_data_through() {
        let node_a = make_test_node().await;
        let node_b = make_test_node().await;
        configure_as_peers(&node_a, &node_b).await;
        drive_handshake_to_completion(&node_a, &node_b).await;

        let old_session_id = assert_has_established_session(&node_a, node_b.engine.local_public_key).await;

        // Force the rekey deadline without waiting 120 real seconds: back-date
        // A's session's established_at far enough that is_past_rekey_deadline()
        // is true via the time-based trigger. (An equally valid way to force
        // it would be bumping bytes_transferred past REKEY_AFTER_BYTES — this
        // test exercises the time-based trigger specifically; both share the
        // exact same is_past_rekey_deadline()/rekey-initiation code path.)
        {
            let peers = node_a.engine.peers.read().await;
            let peer = peers.get(&node_b.engine.local_public_key).unwrap();
            let mut session_guard = peer.session.write().await;
            let session = session_guard.as_mut().unwrap();
            // Session is a plain struct (not behind further interior
            // mutability for established_at), so replace it directly via a
            // fresh Session sharing the same keys/session_id but a
            // back-dated clock — mirrors exactly what "artificially set a
            // session's start time far in the past" means.
            let backdated = Session {
                session_id: session.session_id,
                send_key: session.send_key,
                recv_key: session.recv_key,
                send_nonce: AtomicU64::new(session.send_nonce.load(Ordering::SeqCst)),
                recv_bitmap: Mutex::new(ReplayWindow::default()),
                established_at: Instant::now() - Duration::from_secs(REKEY_AFTER_SECS + 5),
                bytes_transferred: AtomicU64::new(session.bytes_transferred.load(Ordering::Relaxed)),
                prev: Mutex::new(None),
            };
            *session_guard = Some(backdated);
        }
        assert!(
            {
                let peers = node_a.engine.peers.read().await;
                let peer = peers.get(&node_b.engine.local_public_key).unwrap();
                let session_guard = peer.session.read().await;
                session_guard.as_ref().unwrap().is_past_rekey_deadline()
            },
            "session should report past its rekey deadline after back-dating established_at"
        );

        // Trigger the SAME rekey-initiation path the keepalive task uses
        // (initiate_handshake — a rekey is not structurally different from
        // the initial handshake, just triggered by a different condition).
        let (_dec_a, handshaker_a) = helpers_for(&node_a);
        let peer_arc = {
            let peers = node_a.engine.peers.read().await;
            peers.get(&node_b.engine.local_public_key).unwrap().clone()
        };
        handshaker_a.initiate_handshake(&node_a.socket, node_b.engine.local_public_key, &peer_arc).await
            .expect("rekey initiate_handshake should succeed");

        // B receives the (re)INIT and responds.
        let outcome = recv_one(&node_b, Duration::from_secs(5)).await.unwrap();
        assert!(matches!(outcome, InboundOutcome::HandshakeHandled));
        // A receives the RESP and installs the new session.
        let outcome = recv_one(&node_a, Duration::from_secs(5)).await.unwrap();
        assert!(matches!(outcome, InboundOutcome::HandshakeHandled));

        let new_session_id = assert_has_established_session(&node_a, node_b.engine.local_public_key).await;
        assert_ne!(old_session_id, new_session_id, "rekey should install a brand-new session_id, not reuse the old one");

        // The core dual-key-window assertion: a data packet sent
        // IMMEDIATELY after the forced rekey (i.e. under A's brand new
        // session) must still be decryptable by B, exercising the real
        // send/recv path end-to-end right after a rekey — not the dual-key
        // window's fallback branch specifically (that's the next
        // assertion), but proving the rekey didn't break ordinary traffic.
        let plaintext = b"data sent immediately after forced rekey";
        let encrypted = {
            let peers = node_a.engine.peers.read().await;
            let peer = peers.get(&node_b.engine.local_public_key).unwrap();
            let session_guard = peer.session.read().await;
            let session = session_guard.as_ref().unwrap();
            node_a.engine.encrypt_packet(plaintext, session).unwrap()
        };
        node_a.socket.send_to(&encrypted, node_b.addr).await.unwrap();
        let outcome = recv_one(&node_b, Duration::from_secs(5)).await.unwrap();
        match outcome {
            InboundOutcome::Data { plaintext: decrypted } => {
                assert_eq!(decrypted, plaintext, "post-rekey data packet did not decrypt correctly — connection was effectively dropped by the rekey");
            }
            other => panic!("expected InboundOutcome::Data after rekey, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_dual_key_window_decrypts_packet_under_previous_session() {
        // Exercises DecryptHelper::decrypt's SPECIFIC previous-session
        // fallback branch directly: install a session, manually attach a
        // `prev` (as install_session would during a real rekey), and prove
        // a packet encrypted under the OLD key is still accepted within the
        // window — this is the actual in-flight-packets-during-rehandshake
        // guarantee, distinct from (and a stronger check than) "a
        // post-rekey packet under the NEW key works", which the previous
        // test already covers.
        let node_a = make_test_node().await;
        let node_b = make_test_node().await;
        configure_as_peers(&node_a, &node_b).await;
        drive_handshake_to_completion(&node_a, &node_b).await;

        // Capture A's current (about-to-become-"previous") session keys,
        // then install a brand new session as the "current" one with the
        // old one attached as `prev` — mirrors exactly the state
        // install_session leaves behind mid-rekey.
        let (old_session_id, old_send_key) = {
            let peers = node_a.engine.peers.read().await;
            let peer = peers.get(&node_b.engine.local_public_key).unwrap();
            let session_guard = peer.session.read().await;
            let s = session_guard.as_ref().unwrap();
            (s.session_id, s.send_key)
        };

        // Encrypt a packet under the OLD session BEFORE swapping it out —
        // simulates a packet that was in flight at the moment the peer
        // (here, ourselves, for test simplicity) switched sessions.
        let plaintext = b"in-flight packet encrypted under the old session";
        let old_encrypted = {
            let peers = node_a.engine.peers.read().await;
            let peer = peers.get(&node_b.engine.local_public_key).unwrap();
            let session_guard = peer.session.read().await;
            node_a.engine.encrypt_packet(plaintext, session_guard.as_ref().unwrap()).unwrap()
        };

        // Now install a new "current" session with the old one attached as
        // `prev`, matching what a real rekey leaves behind on the
        // RECEIVING side (node_b, whose recv_key for the old session must
        // stay valid during the window). We manipulate node_b's state
        // directly here since that's the side whose decrypt() path we're
        // testing.
        let peer_b_pubkey = node_a.engine.local_public_key;
        let (b_old_session_id, b_old_recv_key, b_new_session) = {
            let peers = node_b.engine.peers.read().await;
            let peer = peers.get(&peer_b_pubkey).unwrap();
            let mut session_guard = peer.session.write().await;
            let old = session_guard.take().unwrap();
            let old_id = old.session_id;
            let old_recv_key = old.recv_key;
            // Fresh new "current" session (arbitrary fresh keys — its
            // content doesn't matter for this test, only that `prev` is
            // populated and unexpired).
            let new_session = Session {
                session_id: rand::random::<u32>(),
                send_key: [7u8; 32],
                recv_key: [8u8; 32],
                send_nonce: AtomicU64::new(1),
                recv_bitmap: Mutex::new(ReplayWindow::default()),
                established_at: Instant::now(),
                bytes_transferred: AtomicU64::new(0),
                prev: Mutex::new(Some(PrevSession {
                    session_id: old_id,
                    recv_key: old_recv_key,
                    recv_bitmap: old.recv_bitmap,
                    expires_at: Instant::now() + Duration::from_secs(DUAL_KEY_WINDOW_SECS),
                })),
            };
            *session_guard = Some(new_session);
            (old_id, old_recv_key, ())
        };
        let _ = (b_old_session_id, b_old_recv_key, b_new_session);
        // Re-register the OLD session_id in node_b's session_map too (this
        // is what install_session does during a real rekey — the old
        // session_id must still resolve to the same peer for the dual-key
        // window to work at all).
        {
            let mut sm = node_b.engine.session_map.write().await;
            sm.insert(old_session_id, peer_b_pubkey);
        }

        // Sanity: old_encrypted's session_id header matches old_session_id
        // (i.e. A really did encrypt this under the pre-rekey session).
        let hdr_session_id = u32::from_le_bytes(old_encrypted[1..5].try_into().unwrap());
        assert_eq!(hdr_session_id, old_session_id);

        // Deliver the OLD-session packet to B over the real socket.
        node_a.socket.send_to(&old_encrypted, node_b.addr).await.unwrap();
        let outcome = recv_one(&node_b, Duration::from_secs(5)).await.unwrap();
        match outcome {
            InboundOutcome::Data { plaintext: decrypted } => {
                assert_eq!(decrypted, plaintext, "packet encrypted under the PREVIOUS session should still decrypt correctly during the dual-key window");
            }
            other => panic!("expected InboundOutcome::Data via the dual-key window fallback, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_malformed_handshake_packet_does_not_crash_engine_and_subsequent_traffic_still_works() {
        // The other core Stage 3 safety requirement: a malformed/truncated
        // handshake packet from an unauthenticated source must never panic
        // the engine, and legitimate traffic (including from OTHER peers)
        // must continue to work afterward.
        let node_a = make_test_node().await;
        let node_b = make_test_node().await;
        let node_c = make_test_node().await; // a second, well-behaved peer of B
        configure_as_peers(&node_a, &node_b).await;
        configure_as_peers(&node_c, &node_b).await;

        // Send a batch of deliberately-malformed "handshake" packets
        // straight at B: right packet-type byte, but truncated/garbage
        // payloads at every length-prefix boundary handshake.rs's bounds
        // checks are supposed to catch.
        let malformed_payloads: Vec<Vec<u8>> = vec![
            vec![PACKET_HANDSHAKE_INIT], // just the type byte, nothing else
            vec![PACKET_HANDSHAKE_INIT, HANDSHAKE_VERSION_FOR_TEST], // + version, nothing else
            { let mut v = vec![PACKET_HANDSHAKE_INIT, HANDSHAKE_VERSION_FOR_TEST]; v.extend_from_slice(&[0u8; 10]); v }, // truncated ephemeral key
            { let mut v = vec![PACKET_HANDSHAKE_INIT, HANDSHAKE_VERSION_FOR_TEST]; v.extend_from_slice(&[0u8; 32]); v.extend_from_slice(&(u32::MAX / 2).to_le_bytes()); v }, // absurd claimed kyber_pub_len
            vec![PACKET_HANDSHAKE_RESP], // malformed RESP too, not just INIT
            vec![], // Task1's caller guards len<1 before dispatch, but exercise directly anyway
        ];

        for payload in &malformed_payloads {
            node_a.socket.send_to(payload, node_b.addr).await.unwrap();
            // B must process this without panicking. We can't directly
            // assert "no panic" from outside the async task in this
            // in-process test structure, but recv_one running to completion
            // and returning normally (rather than the test task itself
            // panicking, which `cargo test` would report as a hard
            // failure) is exactly that proof — process_inbound_datagram/
            // handle_handshake_init route every parse failure through Err,
            // never .unwrap(), so a panic here WOULD surface as this test
            // failing with a panic message, not silently passing.
            let outcome = recv_one(&node_b, Duration::from_secs(2)).await;
            // Either it timed out (possible for the len=0 case, since
            // process_inbound_datagram's is_empty() check returns Dropped
            // without needing a recv at all — actually recv_from itself
            // would still receive a 0-byte datagram, so this should not
            // time out) or it completed with Dropped/HandshakeHandled
            // (a malformed RESP with no matching pending handshake also
            // resolves to HandshakeHandled, since handle_handshake_resp
            // itself decides internally to drop it) — the only
            // unacceptable outcome is the test task panicking, which
            // `.await` on a panicking spawned future would surface as an
            // Err(JoinError) here if this were spawned, or directly as a
            // test failure since recv_one runs in the test's own task.
            match outcome {
                Ok(InboundOutcome::Dropped) | Ok(InboundOutcome::HandshakeHandled) => {}
                Ok(other) => panic!("malformed handshake payload unexpectedly produced {:?}", other),
                Err(e) => panic!("malformed handshake payload caused recv_one to error/timeout: {}", e),
            }
        }

        // NOW prove B is still fully functional: a real handshake from a
        // DIFFERENT, well-behaved peer (C) still completes correctly.
        drive_handshake_to_completion(&node_c, &node_b).await;
        assert_has_established_session(&node_b, node_c.engine.local_public_key).await;

        // And a real data packet from C still round-trips.
        let plaintext = b"B is still alive after the malformed-packet barrage";
        let encrypted = {
            let peers = node_c.engine.peers.read().await;
            let peer = peers.get(&node_b.engine.local_public_key).unwrap();
            let session_guard = peer.session.read().await;
            node_c.engine.encrypt_packet(plaintext, session_guard.as_ref().unwrap()).unwrap()
        };
        node_c.socket.send_to(&encrypted, node_b.addr).await.unwrap();
        let outcome = recv_one(&node_b, Duration::from_secs(5)).await.unwrap();
        match outcome {
            InboundOutcome::Data { plaintext: decrypted } => assert_eq!(decrypted, plaintext),
            other => panic!("expected InboundOutcome::Data from C after the malformed-packet barrage, got {:?}", other),
        }
    }

    /// Local copy of handshake.rs's private HANDSHAKE_VERSION constant
    /// (value 1) — only used to build realistic-looking malformed test
    /// payloads above; not a real dependency on handshake.rs's internals.
    const HANDSHAKE_VERSION_FOR_TEST: u8 = 1;
}
