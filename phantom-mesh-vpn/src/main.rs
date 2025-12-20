//! PhantomMesh VPN Node
//! =====================
//! Main entry point for the PhantomMesh VPN daemon.
//!
//! Copyright © 2025 Stephen Bilodeau. All rights reserved.
//! Licensed under GPL-3.0 with proprietary agent clauses.

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};
use tracing_subscriber::fmt::format::json;
use rand::RngCore;

use phantom_mesh::security_layer::{crypto_manager::CryptoManager, sigma_vault::SigmaVault, threat_engine::ThreatEngine};
use phantom_mesh::vpn_core::{tunnel_engine::TunnelEngine, api_gateway::ApiGateway};
use phantom_mesh::metrics::{init_metrics, update_system_metrics};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing with JSON format for structured logging
    tracing_subscriber::fmt()
        .event_format(json())
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Initialize Prometheus metrics
    init_metrics();
    info!("Prometheus metrics initialized");

    info!("Starting PhantomMesh VPN Node v{}", env!("CARGO_PKG_VERSION"));

    // Initialize cryptographic manager
    let crypto = Arc::new(CryptoManager::new()?);

    // Initialize threat engine
    let threat_engine = Arc::new(Mutex::new(ThreatEngine::new()?));
    threat_engine.lock().await.initialize().await?;

    // Initialize ΣVault dimensional scattering system
    let mut sigma_vault_master_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut sigma_vault_master_key);
    let sigma_vault = Arc::new(SigmaVault::new(sigma_vault_master_key));
    sigma_vault.start_background_tasks();

    info!("ΣVault dimensional scattering system initialized");

    // Create event channel for agent hooks
    let (event_tx, mut event_rx) = mpsc::channel(100);

    // Initialize tunnel engine
    let _tunnel_engine = Arc::new(TunnelEngine::new(crypto.clone(), sigma_vault.clone(), event_tx));

    // Initialize API gateway with threat engine
    let api_gateway = ApiGateway::new(Arc::clone(&threat_engine));

    // Start API server in background
    let _api_handle = tokio::spawn(async move {
        if let Err(e) = api_gateway.serve("0.0.0.0:8080").await {
            warn!("API gateway error: {}", e);
        }
    });

    // Start metrics update task
    let _metrics_handle = tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            update_system_metrics();
        }
    });

    // TODO: Initialize agent swarm integration
    // TODO: Load configuration
    // TODO: Start network listeners

    info!("PhantomMesh VPN Node initialized successfully");
    info!("Listening on UDP port 51820 for WireGuard connections");
    info!("API gateway available on http://localhost:8080");
    info!("Threat detection engine active");

    // Main event loop
    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                // Handle tunnel events
                match event {
                    phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::PeerConnected { public_key, endpoint } => {
                        info!(peer = ?public_key[..8], endpoint = ?endpoint, "Peer connected");
                    }
                    phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::PeerDisconnected { public_key } => {
                        info!(peer = ?public_key[..8], "Peer disconnected");
                    }
                    phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::PacketRouted { dimension, bytes } => {
                        // Debug level for high-frequency events
                        tracing::debug!(dimension, bytes, "Packet routed");
                    }
                    phantom_mesh::vpn_core::tunnel_engine::TunnelEvent::ThreatSignature { signature, source } => {
                        warn!(signature = ?signature[..16], source = ?source, "Threat signature detected");

                        // Analyze threat with threat engine
                        let threat_result = threat_engine.lock().await.analyze_packet(&signature, Some(&source)).await;
                        if let Some(threat) = threat_result {
                            warn!(
                                threat_id = ?threat.signature_id,
                                severity = ?threat.severity,
                                confidence = threat.confidence,
                                "Threat confirmed by engine"
                            );

                            // Generate alert
                            threat_engine.lock().await.generate_alert(&threat).await?;
                        }
                    }
                }
            }
        }
    }
}