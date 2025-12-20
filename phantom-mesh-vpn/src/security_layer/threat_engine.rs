//! Threat Engine Module
//! ====================
//! Real-time threat detection and response.
//!
//! Copyright © 2025 Stephen Bilodeau. All rights reserved.
//! Licensed under GPL-3.0 with proprietary agent clauses.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use tracing::{debug, error, info};
use regex::Regex;
use aho_corasick::AhoCorasick;
use bloom::{BloomFilter, ASMS};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::metrics::{THREAT_EVENTS_TOTAL, THREAT_RESPONSE_TIME_SECONDS};

/// Threat signature database entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatSignature {
    pub id: String,
    pub pattern: Vec<u8>,
    pub description: String,
    pub severity: ThreatSeverity,
    pub category: ThreatCategory,
    pub regex_pattern: Option<String>,
    #[serde(skip)]
    pub compiled_regex: Option<Regex>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreatCategory {
    Malware,
    Exploit,
    Anomaly,
    DDoS,
    DataExfiltration,
    UnauthorizedAccess,
}

/// Anomaly detection statistics
#[derive(Debug, Clone)]
pub struct PacketStats {
    pub packet_count: u64,
    pub byte_count: u64,
    pub avg_packet_size: f64,
    pub std_dev_packet_size: f64,
    pub entropy: f64,
    pub last_updated: Instant,
}

/// Threat detection result
#[derive(Debug, Clone, Serialize)]
pub struct ThreatResult {
    pub signature_id: String,
    pub severity: ThreatSeverity,
    pub confidence: f64,
    pub description: String,
    pub source_ip: Option<String>,
    #[serde(skip)]
    pub timestamp: Instant,
}

/// Automated response action
#[derive(Debug, Clone)]
pub enum ResponseAction {
    QuarantinePeer { peer_key: [u8; 32] },
    BlockTraffic { source_ip: String, duration: Duration },
    Alert { message: String, severity: ThreatSeverity },
    Log { level: String, message: String },
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetrics {
    pub packets_analyzed: u64,
    pub threats_detected: u64,
    pub false_positives: u64,
    pub analysis_time_avg: Duration,
    pub memory_usage: usize,
    #[serde(skip)]
    pub last_updated: Instant,
}

/// Threat intelligence sharing data
#[derive(Debug, Clone)]
pub struct ThreatIntel {
    pub signature: ThreatSignature,
    pub source_agent: String,
    pub timestamp: Instant,
    pub confidence: f64,
}

/// Main threat detection and analysis engine
pub struct ThreatEngine {
    // Signature database
    signatures: Arc<RwLock<HashMap<String, ThreatSignature>>>,
    bloom_filter: Arc<RwLock<BloomFilter>>,
    aho_corasick: Arc<RwLock<Option<AhoCorasick>>>,

    // Anomaly detection
    packet_stats: Arc<DashMap<String, PacketStats>>, // keyed by peer IP or ID
    anomaly_threshold: f64,
    statistical_window: usize,

    // Performance monitoring
    metrics: Arc<RwLock<PerformanceMetrics>>,
    performance_window: Arc<RwLock<VecDeque<(Instant, Duration)>>>,

    // Response mechanisms
    response_tx: mpsc::Sender<ResponseAction>,
    quarantined_peers: Arc<RwLock<HashMap<[u8; 32], Instant>>>,

    // Threat intelligence sharing
    intel_tx: mpsc::Sender<ThreatIntel>,
    intel_rx: mpsc::Receiver<ThreatIntel>,

    // Configuration
    max_memory_mb: usize,
    update_interval: Duration,
}

impl ThreatEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (response_tx, _) = mpsc::channel(1000);
        let (intel_tx, intel_rx) = mpsc::channel(100);

        Ok(Self {
            signatures: Arc::new(RwLock::new(HashMap::new())),
            bloom_filter: Arc::new(RwLock::new(BloomFilter::with_rate(0.01, 10000))),
            aho_corasick: Arc::new(RwLock::new(None)),
            packet_stats: Arc::new(DashMap::new()),
            anomaly_threshold: 3.0, // 3 standard deviations
            statistical_window: 1000,
            metrics: Arc::new(RwLock::new(PerformanceMetrics {
                packets_analyzed: 0,
                threats_detected: 0,
                false_positives: 0,
                analysis_time_avg: Duration::from_micros(10),
                memory_usage: 0,
                last_updated: Instant::now(),
            })),
            performance_window: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            response_tx,
            quarantined_peers: Arc::new(RwLock::new(HashMap::new())),
            intel_tx,
            intel_rx,
            max_memory_mb: 100,
            update_interval: Duration::from_secs(3600), // 1 hour
        })
    }

    /// Initialize the threat engine with default signatures
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.load_default_signatures().await?;
        self.rebuild_search_structures().await?;
        self.start_background_tasks();
        info!("Threat engine initialized");
        Ok(())
    }

    /// Load default threat signatures
    async fn load_default_signatures(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut signatures = self.signatures.write().await;

        // Malware signatures
        signatures.insert("malware_shellcode".to_string(), ThreatSignature {
            id: "malware_shellcode".to_string(),
            pattern: vec![0x90, 0x90, 0x90, 0x90], // NOP sled example
            description: "Potential shellcode with NOP sled".to_string(),
            severity: ThreatSeverity::High,
            category: ThreatCategory::Malware,
            regex_pattern: None,
            compiled_regex: None,
        });

        // Exploit signatures
        signatures.insert("buffer_overflow".to_string(), ThreatSignature {
            id: "buffer_overflow".to_string(),
            pattern: b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(), // Long A sequence
            description: "Potential buffer overflow attempt".to_string(),
            severity: ThreatSeverity::Critical,
            category: ThreatCategory::Exploit,
            regex_pattern: None,
            compiled_regex: None,
        });

        // DDoS patterns
        signatures.insert("syn_flood".to_string(), ThreatSignature {
            id: "syn_flood".to_string(),
            pattern: vec![], // Will be detected by anomaly analysis
            description: "SYN flood detected by packet rate anomaly".to_string(),
            severity: ThreatSeverity::High,
            category: ThreatCategory::DDoS,
            regex_pattern: Some(r"SYN\s+packets?".to_string()),
            compiled_regex: Some(Regex::new(r"SYN\s+packets?")?),
        });

        info!("Loaded {} default signatures", signatures.len());
        Ok(())
    }

    /// Rebuild search structures (Bloom filter, Aho-Corasick)
    async fn rebuild_search_structures(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let signatures = self.signatures.read().await;
        let patterns: Vec<&[u8]> = signatures.values().map(|s| s.pattern.as_slice()).collect();

        // Rebuild Bloom filter
        let mut bloom = self.bloom_filter.write().await;
        *bloom = BloomFilter::with_rate(0.01, patterns.len() as u32 * 10);
        for pattern in &patterns {
            bloom.insert(pattern);
        }

        // Rebuild Aho-Corasick for multi-pattern matching
        if !patterns.is_empty() {
            let ac = AhoCorasick::new(&patterns).map_err(|e| format!("Aho-Corasick build failed: {}", e))?;
            *self.aho_corasick.write().await = Some(ac);
        }

        debug!("Rebuilt search structures for {} patterns", patterns.len());
        Ok(())
    }

    /// Analyze packet for threats (signature-based and anomaly-based)
    pub async fn analyze_packet(&self, packet: &[u8], source_ip: Option<&str>) -> Option<ThreatResult> {
        let start_time = Instant::now();

        // Update performance metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.packets_analyzed += 1;
        }

        // Signature-based detection
        if let Some(result) = self.signature_based_detection(packet).await {
            self.record_threat_metrics(&result, start_time).await;
            self.update_metrics(start_time, true).await;
            return Some(result);
        }

        // Anomaly-based detection
        if let Some(result) = self.anomaly_based_detection(packet, source_ip).await {
            self.record_threat_metrics(&result, start_time).await;
            self.update_metrics(start_time, true).await;
            return Some(result);
        }

        self.update_metrics(start_time, false).await;
        None
    }

    /// Record threat detection metrics
    async fn record_threat_metrics(&self, threat: &ThreatResult, start_time: Instant) {
        let response_time = start_time.elapsed().as_secs_f64();

        // Record threat event
        THREAT_EVENTS_TOTAL
            .with_label_values(&[
                &format!("{:?}", threat.severity).to_lowercase(),
                &format!("{:?}", threat.severity).to_lowercase(),
                &threat.source_ip.clone().unwrap_or_else(|| "unknown".to_string()),
            ])
            .inc();

        // Record response time
        THREAT_RESPONSE_TIME_SECONDS.observe(response_time);
    }

    /// Signature-based threat detection
    async fn signature_based_detection(&self, packet: &[u8]) -> Option<ThreatResult> {
        // Quick Bloom filter check - use packet as bytes directly
        {
            let bloom = self.bloom_filter.read().await;
            if !bloom.contains(&packet.to_vec()) {
                return None;
            }
        }

        // Detailed Aho-Corasick matching
        if let Some(ac) = self.aho_corasick.read().await.as_ref() {
            if let Some(mat) = ac.find_iter(packet).next() {
                let signatures = self.signatures.read().await;
                for (id, sig) in signatures.iter() {
                    if sig.pattern.len() <= packet.len() &&
                       packet[mat.start()..mat.start() + sig.pattern.len()] == sig.pattern[..] {
                        return Some(ThreatResult {
                            signature_id: id.clone(),
                            severity: sig.severity.clone(),
                            confidence: 1.0,
                            description: sig.description.clone(),
                            source_ip: None,
                            timestamp: Instant::now(),
                        });
                    }
                }
            }
        }

        // Regex-based detection
        let signatures = self.signatures.read().await;
        for (id, sig) in signatures.iter() {
            if let Some(regex) = &sig.compiled_regex {
                if let Ok(packet_str) = std::str::from_utf8(packet) {
                    if regex.is_match(packet_str) {
                        return Some(ThreatResult {
                            signature_id: id.clone(),
                            severity: sig.severity.clone(),
                            confidence: 0.9,
                            description: sig.description.clone(),
                            source_ip: None,
                            timestamp: Instant::now(),
                        });
                    }
                }
            }
        }

        None
    }

    /// Anomaly-based threat detection using statistical analysis
    async fn anomaly_based_detection(&self, packet: &[u8], source_ip: Option<&str>) -> Option<ThreatResult> {
        let key = source_ip.unwrap_or("unknown").to_string();

        // Update packet statistics
        self.update_packet_stats(&key, packet.len()).await;

        // Get current stats
        if let Some(stats) = self.packet_stats.get(&key) {
            let packet_size = packet.len() as f64;

            // Check for packet size anomaly
            if stats.std_dev_packet_size > 0.0 {
                let z_score = (packet_size - stats.avg_packet_size) / stats.std_dev_packet_size;
                if z_score.abs() > self.anomaly_threshold {
                    return Some(ThreatResult {
                        signature_id: "packet_size_anomaly".to_string(),
                        severity: ThreatSeverity::Medium,
                        confidence: (z_score.abs() / 5.0).min(1.0), // Normalize confidence
                        description: format!("Packet size anomaly detected: {} bytes (z-score: {:.2})", packet_size, z_score),
                        source_ip: Some(key.clone()),
                        timestamp: Instant::now(),
                    });
                }
            }

            // Check for entropy anomaly (potential encrypted data exfiltration)
            let entropy = self.calculate_entropy(packet);
            if entropy > 7.5 { // High entropy threshold
                return Some(ThreatResult {
                    signature_id: "high_entropy_traffic".to_string(),
                    severity: ThreatSeverity::Medium,
                    confidence: 0.7,
                    description: format!("High entropy traffic detected: {:.2} bits", entropy),
                    source_ip: Some(key.clone()),
                    timestamp: Instant::now(),
                });
            }
        }

        None
    }

    /// Update packet statistics for anomaly detection
    async fn update_packet_stats(&self, key: &str, packet_size: usize) {
        self.packet_stats
            .entry(key.to_string())
            .and_modify(|stats| {
                stats.packet_count += 1;
                stats.byte_count += packet_size as u64;

                // Rolling average and standard deviation
                let old_avg = stats.avg_packet_size;
                let new_count = stats.packet_count as f64;
                stats.avg_packet_size = old_avg + (packet_size as f64 - old_avg) / new_count;

                // Welford's online algorithm for standard deviation
                if new_count > 1.0 {
                    let delta = packet_size as f64 - old_avg;
                    let delta2 = packet_size as f64 - stats.avg_packet_size;
                    stats.std_dev_packet_size = ((new_count - 2.0) * stats.std_dev_packet_size.powi(2) + delta * delta2) / (new_count - 1.0);
                    stats.std_dev_packet_size = stats.std_dev_packet_size.sqrt();
                }

                stats.entropy = self.calculate_entropy(&vec![packet_size as u8; packet_size]); // Simplified
                stats.last_updated = Instant::now();
            })
            .or_insert(PacketStats {
                packet_count: 1,
                byte_count: packet_size as u64,
                avg_packet_size: packet_size as f64,
                std_dev_packet_size: 0.0,
                entropy: self.calculate_entropy(&vec![packet_size as u8; packet_size]),
                last_updated: Instant::now(),
            });
    }

    /// Calculate Shannon entropy of data
    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        let mut freq = [0u32; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &freq {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Update performance metrics
    async fn update_metrics(&self, start_time: Instant, threat_detected: bool) {
        let analysis_time = start_time.elapsed();

        let mut metrics = self.metrics.write().await;
        if threat_detected {
            metrics.threats_detected += 1;
        }

        // Update rolling average analysis time
        let mut performance_window = self.performance_window.write().await;
        performance_window.push_back((Instant::now(), analysis_time));
        if performance_window.len() > 100 {
            performance_window.pop_front();
        }

        let total_time: Duration = performance_window.iter().map(|(_, duration)| *duration).sum();
        metrics.analysis_time_avg = total_time / performance_window.len() as u32;
        metrics.last_updated = Instant::now();
    }

    /// Quarantine a peer
    pub async fn quarantine_peer(&self, peer_key: &[u8; 32]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut quarantined = self.quarantined_peers.write().await;
        quarantined.insert(*peer_key, Instant::now());

        // Send response action
        let action = ResponseAction::QuarantinePeer { peer_key: *peer_key };
        if let Err(e) = self.response_tx.send(action).await {
            error!("Failed to send quarantine action: {}", e);
        }

        info!("Peer quarantined: {:?}", peer_key);
        Ok(())
    }

    /// Check if peer is quarantined
    pub async fn is_peer_quarantined(&self, peer_key: &[u8; 32]) -> bool {
        let quarantined = self.quarantined_peers.read().await;
        quarantined.contains_key(peer_key)
    }

    /// Generate alert for detected threat
    pub async fn generate_alert(&self, threat: &ThreatResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let action = ResponseAction::Alert {
            message: format!("Threat detected: {} - {}", threat.signature_id, threat.description),
            severity: threat.severity.clone(),
        };

        if let Err(e) = self.response_tx.send(action).await {
            error!("Failed to send alert: {}", e);
        }

        Ok(())
    }

    /// Share threat intelligence with other agents
    pub async fn share_threat_intel(&self, signature: ThreatSignature, confidence: f64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let intel = ThreatIntel {
            signature,
            source_agent: "ThreatEngine".to_string(),
            timestamp: Instant::now(),
            confidence,
        };

        if let Err(e) = self.intel_tx.send(intel).await {
            error!("Failed to share threat intel: {}", e);
        }

        Ok(())
    }

    /// Receive threat intelligence from other agents
    pub async fn receive_threat_intel(&mut self) -> Option<ThreatIntel> {
        match self.intel_rx.try_recv() {
            Ok(intel) => Some(intel),
            Err(_) => None,
        }
    }

    /// Update signature database
    pub async fn update_signatures(&self, new_signatures: Vec<ThreatSignature>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sig_count = new_signatures.len();
        let mut signatures = self.signatures.write().await;

        for sig in new_signatures {
            // Compile regex if present
            let compiled_regex = if let Some(pattern) = &sig.regex_pattern {
                Some(Regex::new(pattern)?)
            } else {
                None
            };

            let mut sig_with_regex = sig.clone();
            sig_with_regex.compiled_regex = compiled_regex;

            signatures.insert(sig.id.clone(), sig_with_regex);
        }

        // Rebuild search structures
        drop(signatures);
        self.rebuild_search_structures().await?;

        info!("Updated signature database with {} new signatures", sig_count);
        Ok(())
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Start background tasks (signature updates, cleanup, etc.)
    fn start_background_tasks(&self) {
        let _signatures = Arc::clone(&self.signatures);
        let quarantined_peers = Arc::clone(&self.quarantined_peers);
        let performance_window = Arc::clone(&self.performance_window);
        let update_interval = self.update_interval;

        // Signature update task
        tokio::spawn(async move {
            let mut interval = time::interval(update_interval);
            loop {
                interval.tick().await;
                // TODO: Implement signature update from threat intelligence
                // This would typically fetch from a threat intelligence feed
            }
        });

        // Cleanup task
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;

                // Clean up old performance metrics
                let mut window = performance_window.write().await;
                let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
                window.retain(|(timestamp, _)| *timestamp > cutoff);

                // Clean up quarantined peers after 1 hour
                let mut quarantined = quarantined_peers.write().await;
                let cutoff = Instant::now() - Duration::from_secs(3600);
                quarantined.retain(|_, &mut timestamp| timestamp > cutoff);
            }
        });
    }

    /// Update signatures from received threat intelligence
    async fn update_signatures_from_intel(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This would integrate with the FORTRESS agent
        // For now, just process any received intel
        while let Some(intel) = self.receive_threat_intel().await {
            if intel.confidence > 0.8 {
                let mut signatures = self.signatures.write().await;
                let sig_id = intel.signature.id.clone();
                signatures.insert(sig_id.clone(), intel.signature);
                info!("Added signature from intel: {}", sig_id);
            }
        }
        Ok(())
    }

    /// Cleanup expired quarantined peers
    async fn cleanup_quarantined_peers(&self) {
        let mut quarantined = self.quarantined_peers.write().await;
        let now = Instant::now();
        let expired: Vec<_> = quarantined.iter()
            .filter(|(_, &time)| now.duration_since(time) > Duration::from_secs(3600)) // 1 hour
            .map(|(key, _)| *key)
            .collect();

        for key in expired {
            quarantined.remove(&key);
            info!("Removed expired quarantine for peer: {:?}", key);
        }
    }

    /// Get response action receiver
    pub fn get_response_receiver(&self) -> mpsc::Receiver<ResponseAction> {
        let (_tx, rx) = mpsc::channel(1000);
        // This is a simplified implementation - in practice, you'd need to manage the channel properly
        rx
    }
}

impl Clone for ThreatEngine {
    fn clone(&self) -> Self {
        // This is a simplified clone - in practice, you'd need proper Arc cloning
        Self {
            signatures: Arc::clone(&self.signatures),
            bloom_filter: Arc::clone(&self.bloom_filter),
            aho_corasick: Arc::clone(&self.aho_corasick),
            packet_stats: Arc::clone(&self.packet_stats),
            anomaly_threshold: self.anomaly_threshold,
            statistical_window: self.statistical_window,
            metrics: Arc::clone(&self.metrics),
            performance_window: self.performance_window.clone(),
            response_tx: self.response_tx.clone(),
            quarantined_peers: Arc::clone(&self.quarantined_peers),
            intel_tx: self.intel_tx.clone(),
            intel_rx: tokio::sync::mpsc::channel(100).1, // New receiver
            max_memory_mb: self.max_memory_mb,
            update_interval: self.update_interval,
        }
    }
}