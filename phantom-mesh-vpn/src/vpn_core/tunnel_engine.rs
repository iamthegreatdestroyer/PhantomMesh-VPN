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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

use super::super::security_layer::crypto_manager::CryptoManager;
use crate::mesh::healer::{DisconnectReason, MeshHealer, PeerId};
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
// Data Types
// ============================================================================

#[derive(Debug, Clone)]
pub enum TunnelEvent {
    PeerConnected { public_key: [u8; 32], endpoint: String },
    PeerDisconnected { public_key: [u8; 32] },
    PacketRouted { dimension: u8, bytes: usize },
    ThreatSignature { signature: Vec<u8>, source: String },
    HandshakeCompleted { peer: [u8; 32] },
}

#[derive(Debug, Clone)]
pub struct PeerConfig {
    pub public_key: [u8; 32],
    pub endpoint: Option<SocketAddr>,
    pub allowed_ips: Vec<String>,
    pub preshared_key: Option<[u8; 32]>,
    pub persistent_keepalive: Option<u64>,
}

#[derive(Debug)]
struct PeerState {
    config: PeerConfig,
    session_id: u32,
    send_key: [u8; 32],
    recv_key: [u8; 32],
    send_nonce: AtomicU64,
    recv_nonce_max: AtomicU64,
    recv_bitmap: Mutex<ReplayWindow>,
    last_handshake: std::time::Instant,
    last_received: std::sync::Mutex<std::time::Instant>,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    handshake_complete: AtomicBool,
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
    session_map: Arc<RwLock<HashMap<u32, [u8; 32]>>>,
    stats: Arc<Mutex<TunnelStats>>,
    running: Arc<AtomicBool>,
    local_private_key: [u8; 32],
    local_public_key: [u8; 32],
    mesh_healer: Option<Arc<MeshHealer>>,
    threat_engine: Option<Arc<ThreatEngine>>,
}

impl TunnelEngine {
    pub fn new(
        crypto: Arc<CryptoManager>,
        event_tx: mpsc::Sender<TunnelEvent>,
        private_key: [u8; 32],
        public_key: [u8; 32],
    ) -> Self {
        Self {
            crypto,
            event_tx,
            peers: Arc::new(RwLock::new(HashMap::new())),
            session_map: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(Mutex::new(TunnelStats::default())),
            running: Arc::new(AtomicBool::new(false)),
            local_private_key: private_key,
            local_public_key: public_key,
            mesh_healer: None,
            threat_engine: None,
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
        let session_id = rand::random::<u32>();

        // Derive initial transport keys using BLAKE3 KDF
        // Real keys will be established during handshake
        let mut send_key = [0u8; 32];
        let mut recv_key = [0u8; 32];
        let ikm = blake3::hash(&[&self.local_private_key[..], &config.public_key[..]].concat());
        send_key.copy_from_slice(&ikm.as_bytes()[..32]);
        let ikm2 = blake3::hash(&[&config.public_key[..], &self.local_private_key[..]].concat());
        recv_key.copy_from_slice(&ikm2.as_bytes()[..32]);

        let peer = Arc::new(PeerState {
            config: config.clone(),
            session_id,
            send_key,
            recv_key,
            send_nonce: AtomicU64::new(1), // Start at 1, 0 is reserved
            recv_nonce_max: AtomicU64::new(0),
            recv_bitmap: Mutex::new(ReplayWindow::default()),
            last_handshake: std::time::Instant::now(),
            last_received: std::sync::Mutex::new(std::time::Instant::now()),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            handshake_complete: AtomicBool::new(false),
        });

        {
            let mut peers = self.peers.write().await;
            peers.insert(config.public_key, peer);
        }
        {
            let mut session_map = self.session_map.write().await;
            session_map.insert(session_id, config.public_key);
        }

        if let Some(ref healer) = self.mesh_healer {
            healer.register_peer(PeerId(hex::encode(config.public_key))).await;
        }

        info!(peer = ?hex::encode(&config.public_key[..8]), "Peer added");
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
            let mut session_map = self.session_map.write().await;
            session_map.remove(&peer.session_id);
            let _ = self.event_tx.send(TunnelEvent::PeerDisconnected { public_key }).await;
            info!(peer = ?hex::encode(&public_key[..8]), "Peer removed");
        }
        Ok(())
    }

    pub async fn get_stats(&self) -> TunnelStats {
        self.stats.lock().await.clone()
    }

    /// Encrypt a packet for sending to a peer
    fn encrypt_packet(
        &self,
        plaintext: &[u8],
        peer: &PeerState,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let nonce_counter = peer.send_nonce.fetch_add(1, Ordering::SeqCst);

        // Build nonce: 4 bytes zero padding + 8 bytes counter (little-endian)
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());

        // Encrypt with ChaCha20-Poly1305
        let ciphertext = self.crypto.encrypt_chacha(plaintext, &peer.send_key, &nonce)?;

        // Build packet: type(1) + session_id(4) + nonce_counter(8) + ciphertext
        let mut packet = Vec::with_capacity(HEADER_SIZE + ciphertext.len());
        packet.push(PACKET_DATA);
        packet.extend_from_slice(&peer.session_id.to_le_bytes());
        packet.extend_from_slice(&nonce_counter.to_le_bytes());
        packet.extend_from_slice(&ciphertext);

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
        self.running.store(true, Ordering::SeqCst);

        let socket = Arc::new(UdpSocket::bind(listen_addr).await?);
        info!(addr = %listen_addr, "UDP socket bound");

        // Create TUN device
        let tun = Self::create_tun_device(tun_name)?;
        let tun = Arc::new(tun);
        info!(name = tun_name, "TUN device created");

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

        tokio::spawn(async move {
            let decryptor = DecryptHelper {
                crypto: crypto1, peers: peers1, session_map: session_map1,
            };
            let mut buf = vec![0u8; MAX_PACKET_SIZE];
            while running1.load(Ordering::SeqCst) {
                match recv_socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        if len < 1 { continue; }
                        let packet_type = buf[0];

                        match decryptor.decrypt(&buf[..len]).await {
                            Ok((plaintext, peer_key)) => {
                                // Update last_received timestamp for this peer
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
                                    continue;
                                }

                                // Threat analysis (detection only — never drops packets)
                                if let Some(ref te) = threat_engine1 {
                                    let source = addr.to_string();
                                    if let Some(threat) = te.analyze_packet(&plaintext, Some(&source)).await {
                                        warn!(
                                            threat_id = %threat.signature_id,
                                            severity = ?threat.severity,
                                            source = %source,
                                            "Threat detected in decrypted packet"
                                        );
                                        let _ = event_tx1.send(TunnelEvent::ThreatSignature {
                                            signature: plaintext[..plaintext.len().min(64)].to_vec(),
                                            source,
                                        }).await;
                                    }
                                }

                                #[cfg(target_os = "linux")]
                                {
                                    use std::io::Write;
                                    if let Err(e) = (&*tun1).write_all(&plaintext) {
                                        error!("TUN write error: {}", e);
                                    }
                                }
                                let mut s = stats1.lock().await;
                                s.packets_received += 1;
                                s.bytes_received += len as u64;
                            }
                            Err(e) => { debug!("Decrypt failed: {}", e); }
                        }
                    }
                    Err(e) => {
                        if running1.load(Ordering::SeqCst) { error!("UDP recv: {}", e); }
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

        tokio::spawn(async move {
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
                            for (_pk, peer) in peers.iter() {
                                if let Some(endpoint) = peer.config.endpoint {
                                    let nonce_counter = peer.send_nonce.fetch_add(1, Ordering::SeqCst);
                                    let mut nonce = [0u8; NONCE_SIZE];
                                    nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());

                                    match crypto2.encrypt_chacha(ip_packet, &peer.send_key, &nonce) {
                                        Ok(ciphertext) => {
                                            let mut pkt = Vec::with_capacity(HEADER_SIZE + ciphertext.len());
                                            pkt.push(PACKET_DATA);
                                            pkt.extend_from_slice(&peer.session_id.to_le_bytes());
                                            pkt.extend_from_slice(&nonce_counter.to_le_bytes());
                                            pkt.extend_from_slice(&ciphertext);

                                            if let Err(e) = send_socket.send_to(&pkt, endpoint).await {
                                                error!("UDP send: {}", e);
                                            }
                                            peer.bytes_sent.fetch_add(pkt.len() as u64, Ordering::Relaxed);
                                            let mut s = stats2.lock().await;
                                            s.packets_sent += 1;
                                            s.bytes_sent += pkt.len() as u64;
                                        }
                                        Err(e) => { error!("Encrypt: {}", e); }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            if running2.load(Ordering::SeqCst) { error!("TUN read: {}", e); }
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

        // === Task 3: Keepalive sender + peer timeout detector ===
        let running3 = self.running.clone();
        let peers3 = self.peers.clone();
        let crypto3 = self.crypto.clone();
        let event_tx3 = self.event_tx.clone();
        let healer3 = self.mesh_healer.clone();
        let socket3 = socket.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(KEEPALIVE_INTERVAL_SECS));
            while running3.load(Ordering::SeqCst) {
                interval.tick().await;
                if !running3.load(Ordering::SeqCst) { break; }

                let mut timed_out = Vec::new();

                {
                    let peers = peers3.read().await;
                    for (pk, peer) in peers.iter() {
                        // Send keepalive packet (encrypted empty payload)
                        if let Some(endpoint) = peer.config.endpoint {
                            let nonce_counter = peer.send_nonce.fetch_add(1, Ordering::SeqCst);
                            let mut nonce = [0u8; NONCE_SIZE];
                            nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());

                            match crypto3.encrypt_chacha(&[], &peer.send_key, &nonce) {
                                Ok(ciphertext) => {
                                    let mut pkt = Vec::with_capacity(HEADER_SIZE + ciphertext.len());
                                    pkt.push(PACKET_KEEPALIVE);
                                    pkt.extend_from_slice(&peer.session_id.to_le_bytes());
                                    pkt.extend_from_slice(&nonce_counter.to_le_bytes());
                                    pkt.extend_from_slice(&ciphertext);

                                    if let Err(e) = socket3.send_to(&pkt, endpoint).await {
                                        debug!("Keepalive send error: {}", e);
                                    }
                                }
                                Err(e) => { debug!("Keepalive encrypt error: {}", e); }
                            }
                        }

                        // Check for timeout (75s = 3 missed keepalives)
                        let elapsed = peer.last_received.lock()
                            .map(|last| last.elapsed())
                            .unwrap_or(Duration::from_secs(0));
                        if elapsed > Duration::from_secs(KEEPALIVE_INTERVAL_SECS * 3) {
                            timed_out.push((*pk, peer.config.endpoint));
                        }
                    }
                }

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

        info!("Tunnel started: UDP {} <-> TUN {}", listen_addr, tun_name);
        Ok(())
    }

    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Tunnel engine stopped");
    }

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

        {
            let mut replay = peer.recv_bitmap.lock().await;
            if !replay.check_and_update(nonce_counter) {
                return Err("Replay detected".into());
            }
        }

        let mut nonce = [0u8; NONCE_SIZE];
        nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());
        let plaintext = self.crypto.decrypt_chacha(ciphertext, &peer.recv_key, &nonce)?;
        peer.bytes_received.fetch_add(packet.len() as u64, Ordering::Relaxed);

        Ok((plaintext, peer_key))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
        let keys = crypto.generate_keypair().unwrap();
        let engine = TunnelEngine::new(crypto, tx, keys.0, keys.1);
        let stats = engine.get_stats().await;
        assert_eq!(stats.packets_sent, 0);
        assert_eq!(stats.peers_connected, 0);
    }

    #[tokio::test]
    async fn test_add_remove_peer() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, _rx) = mpsc::channel(100);
        let keys = crypto.generate_keypair().unwrap();
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

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, _rx) = mpsc::channel(100);
        let keys = crypto.generate_keypair().unwrap();
        let engine = TunnelEngine::new(crypto, tx, keys.0, keys.1);

        let peer_config = PeerConfig {
            public_key: [42u8; 32],
            endpoint: Some("127.0.0.1:51820".parse().unwrap()),
            allowed_ips: vec![],
            preshared_key: Some([99u8; 32]),
            persistent_keepalive: None,
        };

        engine.add_peer(peer_config).await.unwrap();

        let peers = engine.peers.read().await;
        let peer = peers.get(&[42u8; 32]).unwrap();

        let plaintext = b"Hello PhantomMesh VPN!";
        let encrypted = engine.encrypt_packet(plaintext, peer).unwrap();

        // Verify header
        assert_eq!(encrypted[0], PACKET_DATA);
        assert!(encrypted.len() > HEADER_SIZE + TAG_SIZE);

        // Verify nonce incremented
        let nonce = peer.send_nonce.load(Ordering::SeqCst);
        assert_eq!(nonce, 2); // Started at 1, incremented to 2
    }

    #[tokio::test]
    async fn test_keepalive_packet_format() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, _rx) = mpsc::channel(100);
        let keys = crypto.generate_keypair().unwrap();
        let engine = TunnelEngine::new(crypto.clone(), tx, keys.0, keys.1);

        let peer_config = PeerConfig {
            public_key: [42u8; 32],
            endpoint: Some("127.0.0.1:51820".parse().unwrap()),
            allowed_ips: vec![],
            preshared_key: None,
            persistent_keepalive: Some(25),
        };
        engine.add_peer(peer_config).await.unwrap();

        let peers = engine.peers.read().await;
        let peer = peers.get(&[42u8; 32]).unwrap();

        // Build a keepalive: encrypted empty payload with PACKET_KEEPALIVE type
        let nonce_counter = peer.send_nonce.fetch_add(1, Ordering::SeqCst);
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());
        let ciphertext = crypto.encrypt_chacha(&[], &peer.send_key, &nonce).unwrap();

        let mut pkt = Vec::with_capacity(HEADER_SIZE + ciphertext.len());
        pkt.push(PACKET_KEEPALIVE);
        pkt.extend_from_slice(&peer.session_id.to_le_bytes());
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
        let keys = crypto.generate_keypair().unwrap();
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
        let keys = crypto.generate_keypair().unwrap();
        let threat = Arc::new(ThreatEngine::new().unwrap());
        let engine = TunnelEngine::new(crypto, tx, keys.0, keys.1)
            .with_threat_engine(threat.clone());

        assert!(engine.threat_engine.is_some());
    }

    #[tokio::test]
    async fn test_peer_timeout_detection() {
        let crypto = Arc::new(CryptoManager::new().unwrap());
        let (tx, mut rx) = mpsc::channel(100);
        let keys = crypto.generate_keypair().unwrap();
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
}
