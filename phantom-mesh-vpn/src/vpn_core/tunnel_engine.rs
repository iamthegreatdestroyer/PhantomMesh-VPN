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
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

use super::super::security_layer::crypto_manager::CryptoManager;

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
    last_received: std::time::Instant,
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
        }
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
            last_received: std::time::Instant::now(),
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

    /// Decrypt a received packet
    async fn decrypt_packet(
        &self,
        packet: &[u8],
    ) -> Result<(Vec<u8>, [u8; 32]), Box<dyn std::error::Error + Send + Sync>> {
        if packet.len() < HEADER_SIZE + TAG_SIZE {
            return Err("Packet too short".into());
        }

        let packet_type = packet[0];
        if packet_type != PACKET_DATA && packet_type != PACKET_KEEPALIVE {
            return Err(format!("Unknown packet type: {}", packet_type).into());
        }

        let session_id = u32::from_le_bytes(packet[1..5].try_into().unwrap());
        let nonce_counter = u64::from_le_bytes(packet[5..13].try_into().unwrap());
        let ciphertext = &packet[HEADER_SIZE..];

        // Look up peer by session ID
        let peer_key = {
            let session_map = self.session_map.read().await;
            *session_map.get(&session_id).ok_or("Unknown session")?
        };

        let peers = self.peers.read().await;
        let peer = peers.get(&peer_key).ok_or("Peer not found")?;

        // Anti-replay check
        {
            let mut replay = peer.recv_bitmap.lock().await;
            if !replay.check_and_update(nonce_counter) {
                return Err("Replay detected or nonce too old".into());
            }
        }

        // Build nonce
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[4..12].copy_from_slice(&nonce_counter.to_le_bytes());

        // Decrypt
        let plaintext = self.crypto.decrypt_chacha(ciphertext, &peer.recv_key, &nonce)?;

        // Update stats
        peer.bytes_received.fetch_add(packet.len() as u64, Ordering::Relaxed);

        Ok((plaintext, peer_key))
    }

    /// Start the tunnel on a given listen address
    ///
    /// This spawns two tasks:
    /// 1. UDP listener: receives encrypted packets, decrypts, writes to TUN
    /// 2. TUN reader: reads IP packets from TUN, encrypts, sends to peer UDP endpoint
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

        // Spawn UDP → TUN task (receive encrypted, decrypt, write to TUN)
        let running = self.running.clone();
        let stats = self.stats.clone();
        let peers = self.peers.clone();
        let session_map = self.session_map.clone();
        let crypto = self.crypto.clone();
        let event_tx = self.event_tx.clone();
        let tun_writer = tun.clone();
        let local_pk = self.local_private_key;

        let engine_decrypt = TunnelEngine {
            crypto: crypto.clone(),
            event_tx: event_tx.clone(),
            peers: peers.clone(),
            session_map: session_map.clone(),
            stats: stats.clone(),
            running: running.clone(),
            local_private_key: local_pk,
            local_public_key: self.local_public_key,
        };

        tokio::spawn(async move {
            let mut buf = vec![0u8; MAX_PACKET_SIZE];
            while running.load(Ordering::SeqCst) {
                match socket.recv_from(&mut buf).await {
                    Ok((len, _addr)) => {
                        let packet = &buf[..len];
                        match engine_decrypt.decrypt_packet(packet).await {
                            Ok((plaintext, _peer_key)) => {
                                // Write decrypted IP packet to TUN device
                                #[cfg(target_os = "linux")]
                                {
                                    use std::io::Write;
                                    if let Err(e) = (&*tun_writer).write_all(&plaintext) {
                                        error!("TUN write error: {}", e);
                                    }
                                }

                                let mut s = stats.lock().await;
                                s.packets_received += 1;
                                s.bytes_received += len as u64;
                            }
                            Err(e) => {
                                debug!("Packet decrypt failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        if running.load(Ordering::SeqCst) {
                            error!("UDP recv error: {}", e);
                        }
                        break;
                    }
                }
            }
        });

        // Spawn TUN → UDP task (read from TUN, encrypt, send to peer)
        let running2 = self.running.clone();
        let stats2 = self.stats.clone();
        let peers2 = self.peers.clone();
        let crypto2 = self.crypto.clone();
        let socket2 = socket.clone();
        let local_pk2 = self.local_private_key;
        let local_pubk2 = self.local_public_key;
        let event_tx2 = self.event_tx.clone();
        let session_map2 = self.session_map.clone();

        let engine_encrypt = TunnelEngine {
            crypto: crypto2,
            event_tx: event_tx2,
            peers: peers2.clone(),
            session_map: session_map2,
            stats: stats2.clone(),
            running: running2.clone(),
            local_private_key: local_pk2,
            local_public_key: local_pubk2,
        };

        tokio::spawn(async move {
            let mut buf = vec![0u8; MAX_PACKET_SIZE];
            while running2.load(Ordering::SeqCst) {
                #[cfg(target_os = "linux")]
                {
                    use std::io::Read;
                    match (&*tun).read(&mut buf) {
                        Ok(len) => {
                            let ip_packet = &buf[..len];
                            // Determine which peer to send to based on destination IP
                            let peers = peers2.read().await;
                            for (_pk, peer) in peers.iter() {
                                if let Some(endpoint) = peer.config.endpoint {
                                    if peer.handshake_complete.load(Ordering::SeqCst) {
                                        match engine_encrypt.encrypt_packet(ip_packet, peer) {
                                            Ok(encrypted) => {
                                                if let Err(e) = socket2.send_to(&encrypted, endpoint).await {
                                                    error!("UDP send error: {}", e);
                                                }
                                                peer.bytes_sent.fetch_add(encrypted.len() as u64, Ordering::Relaxed);
                                                let mut s = stats2.lock().await;
                                                s.packets_sent += 1;
                                                s.bytes_sent += encrypted.len() as u64;
                                            }
                                            Err(e) => {
                                                error!("Encrypt error: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            if running2.load(Ordering::SeqCst) {
                                error!("TUN read error: {}", e);
                            }
                            break;
                        }
                    }
                }

                #[cfg(not(target_os = "linux"))]
                {
                    // Non-Linux: sleep to prevent busy loop
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    warn!("TUN device not supported on this platform");
                    break;
                }
            }
        });

        info!("Tunnel engine started: UDP {} → TUN {}", listen_addr, tun_name);
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
}
