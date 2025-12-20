//! Tunnel Engine Module
//! ====================
//! WireGuard tunnel implementation with ΣVault integration.
//!
//! Copyright © 2025 Stephen Bilodeau. All rights reserved.
//! Licensed under GPL-3.0 with proprietary agent clauses.

use super::super::security_layer::sigma_vault::SigmaVault;use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::cmp::min;
use tokio::sync::{mpsc::Sender, Mutex};
use tokio::task;
use tracing::{error, info};
/// Tunnel events emitted to agent swarm
#[derive(Debug, Clone)]
pub enum TunnelEvent {
    PeerConnected { public_key: [u8; 32], endpoint: String },
    PeerDisconnected { public_key: [u8; 32] },
    PacketRouted { dimension: u8, bytes: usize },
    ThreatSignature { signature: Vec<u8>, source: String },
}

/// Peer information
#[derive(Debug, Clone)]
pub struct Peer {
    pub public_key: [u8; 32],
    pub endpoint: SocketAddr,
    pub allowed_ips: Vec<String>,
    pub preshared_key: Option<[u8; 32]>,
}

/// Tunnel statistics
#[derive(Debug, Default, Clone)]
pub struct TunnelStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub peers_connected: usize,
}

/// WireGuard tunnel engine with quantum-resistant crypto
pub struct TunnelEngine {
    crypto: Arc<super::super::security_layer::crypto_manager::CryptoManager>,
    sigma_vault: Arc<SigmaVault>,
    event_tx: Sender<TunnelEvent>,
    peers: Arc<Mutex<HashMap<[u8; 32], Peer>>>,
    stats: Arc<Mutex<TunnelStats>>,
    socket: Arc<Mutex<Option<Arc<UdpSocket>>>>,
    running: Arc<Mutex<bool>>,
    shared_keys: Arc<Mutex<HashMap<[u8; 32], [u8; 32]>>>, // For simplicity, pre-shared keys
}

impl TunnelEngine {
    pub fn new(
        crypto: Arc<super::super::security_layer::crypto_manager::CryptoManager>,
        sigma_vault: Arc<SigmaVault>,
        event_tx: Sender<TunnelEvent>,
    ) -> Self {
        Self {
            crypto,
            sigma_vault,
            event_tx,
            peers: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(TunnelStats::default())),
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            shared_keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a peer to the tunnel
    pub async fn add_peer(&self, peer: Peer) -> Result<(), Box<dyn std::error::Error>> {
        let mut peers = self.peers.lock().await;
        let mut shared_keys = self.shared_keys.lock().await;
        let key_bytes = peer.public_key;
        peers.insert(key_bytes, peer.clone());

        // For simplicity, use preshared key or generate one
        let shared_key = peer.preshared_key.unwrap_or_else(|| {
            // Generate a shared key using crypto
            self.crypto.generate_keypair().unwrap().1 // Use private key as shared for demo
        });
        shared_keys.insert(key_bytes, shared_key);

        let _ = self.event_tx.send(TunnelEvent::PeerConnected {
            public_key: key_bytes,
            endpoint: peer.endpoint.to_string(),
        }).await;

        info!("Peer added: {:?}", key_bytes);
        Ok(())
    }

    /// Remove a peer from the tunnel
    pub async fn remove_peer(&self, public_key: [u8; 32]) -> Result<(), Box<dyn std::error::Error>> {
        let mut peers = self.peers.lock().await;
        let mut shared_keys = self.shared_keys.lock().await;
        if let Some(_peer) = peers.remove(&public_key) {
            shared_keys.remove(&public_key);
            let _ = self.event_tx.send(TunnelEvent::PeerDisconnected { public_key }).await;
            info!("Peer removed: {:?}", public_key);
        }
        Ok(())
    }

    /// Get tunnel statistics
    pub async fn get_stats(&self) -> TunnelStats {
        self.stats.lock().await.clone()
    }

    pub async fn start(&self, listen_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.lock().await;
        if *running {
            return Err("Tunnel already running".into());
        }
        *running = true;

        // Create UDP socket
        let socket = Arc::new(UdpSocket::bind(listen_addr)?);
        *self.socket.lock().await = Some(socket.clone());

        info!("Tunnel started on {}", listen_addr);

        // Spawn packet handling task
        let peers = self.peers.clone();
        let stats = self.stats.clone();
        let event_tx = self.event_tx.clone();
        let crypto = self.crypto.clone();
        let sigma_vault = self.sigma_vault.clone();
        let shared_keys = self.shared_keys.clone();

        task::spawn(async move {
            Self::handle_packets(socket, peers, shared_keys, stats, event_tx, crypto, sigma_vault).await;
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.lock().await;
        if !*running {
            return Ok(());
        }
        *running = false;

        // Close socket
        *self.socket.lock().await = None;

        info!("Tunnel stopped");
        Ok(())
    }

    async fn handle_packets(
        socket: Arc<UdpSocket>,
        peers: Arc<Mutex<HashMap<[u8; 32], Peer>>>,
        shared_keys: Arc<Mutex<HashMap<[u8; 32], [u8; 32]>>>,
        stats: Arc<Mutex<TunnelStats>>,
        event_tx: Sender<TunnelEvent>,
        crypto: Arc<super::super::security_layer::crypto_manager::CryptoManager>,
        sigma_vault: Arc<SigmaVault>,
    ) {
        let mut buf = [0u8; 65535];

        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    let packet = &buf[..len];

                    // Update stats
                    {
                        let mut stats = stats.lock().await;
                        stats.bytes_received += len as u64;
                        stats.packets_received += 1;
                    }

                    // Process packet
                    if let Err(e) = Self::process_packet(packet, addr, &peers, &shared_keys, &stats, &event_tx, &crypto, &sigma_vault, &socket).await {
                        error!("Failed to process packet: {}", e);
                    }
                }
                Err(e) => {
                    error!("UDP recv error: {}", e);
                    break;
                }
            }
        }
    }

    async fn process_packet(
        packet: &[u8],
        _addr: SocketAddr,
        _peers: &Arc<Mutex<HashMap<[u8; 32], Peer>>>,
        shared_keys: &Arc<Mutex<HashMap<[u8; 32], [u8; 32]>>>,
        stats: &Arc<Mutex<TunnelStats>>,
        event_tx: &Sender<TunnelEvent>,
        crypto: &Arc<super::super::security_layer::crypto_manager::CryptoManager>,
        sigma_vault: &Arc<SigmaVault>,
        _socket: &Arc<UdpSocket>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if packet.len() < 32 {
            return Err("Packet too short".into());
        }

        let peer_public_key: [u8; 32] = packet[..32].try_into().unwrap();
        let encrypted_payload = &packet[32..];

        let shared_key = {
            let shared_keys = shared_keys.lock().await;
            *shared_keys.get(&peer_public_key).ok_or("Unknown peer")?
        };

        // Decrypt payload
        let nonce = [0u8; 12]; // TODO: proper nonce handling
        let decrypted = crypto.decrypt_chacha(encrypted_payload, &shared_key, &nonce)?;

        // Scatter packet through ΣVault dimensional space
        sigma_vault.scatter_packet(&decrypted, &peer_public_key).await?;

        // Try to gather reassembled packets
        let gathered_packets = sigma_vault.gather_packets().await?;

        for reassembled_packet in gathered_packets {
            // For now, just log the reassembled data
            // In a real VPN, this would be sent to the virtual interface
            info!("Reassembled packet from ΣVault: {:?}", &reassembled_packet[..min(100, reassembled_packet.len())]);

            // Update stats for sent (though not actually sending)
            {
                let mut stats = stats.lock().await;
                stats.bytes_sent += reassembled_packet.len() as u64;
                stats.packets_sent += 1;
            }

            // Emit event with dimensional information
            let _ = event_tx.send(TunnelEvent::PacketRouted {
                dimension: 7, // ΣVault uses 7 dimensions
                bytes: reassembled_packet.len(),
            }).await;
        }

        Ok(())
    }
}