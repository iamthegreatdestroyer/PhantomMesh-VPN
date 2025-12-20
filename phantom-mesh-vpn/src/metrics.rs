//! Prometheus metrics for PhantomMesh VPN
//!
//! This module provides comprehensive metrics collection for:
//! - VPN tunnel statistics
//! - Threat detection events
//! - ΣVault operations
//! - System performance

use lazy_static::lazy_static;
use prometheus::{
    Counter, Encoder, Gauge, Histogram, HistogramOpts, IntCounterVec, IntGaugeVec, Opts,
    TextEncoder,
};
use std::sync::Mutex;

// Global metrics registry
lazy_static! {
    pub static ref METRICS_REGISTRY: prometheus::Registry = {
        let registry = prometheus::Registry::new();

        // Register all metrics with the registry
        registry.register(Box::new(VPN_PACKETS_TOTAL.clone())).unwrap();
        registry.register(Box::new(VPN_BYTES_TOTAL.clone())).unwrap();
        registry.register(Box::new(VPN_PACKETS_DROPPED_TOTAL.clone())).unwrap();
        registry.register(Box::new(VPN_TUNNEL_LATENCY_SECONDS.clone())).unwrap();
        registry.register(Box::new(VPN_ACTIVE_TUNNELS.clone())).unwrap();
        registry.register(Box::new(THREAT_EVENTS_TOTAL.clone())).unwrap();
        registry.register(Box::new(THREAT_RESPONSE_TIME_SECONDS.clone())).unwrap();
        registry.register(Box::new(THREAT_PATTERNS_ACTIVE.clone())).unwrap();
        registry.register(Box::new(SIGMA_VAULT_SCATTERING_EFFICIENCY.clone())).unwrap();
        registry.register(Box::new(SIGMA_VAULT_OPERATIONS_TOTAL.clone())).unwrap();
        registry.register(Box::new(SIGMA_VAULT_DIMENSIONS_ACTIVE.clone())).unwrap();
        registry.register(Box::new(SYSTEM_MEMORY_USAGE_BYTES.clone())).unwrap();
        registry.register(Box::new(SYSTEM_CPU_USAGE_PERCENT.clone())).unwrap();
        registry.register(Box::new(SYSTEM_NETWORK_CONNECTIONS.clone())).unwrap();

        registry
    };

    // VPN Tunnel Metrics
    pub static ref VPN_PACKETS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("phantom_vpn_packets_total", "Total number of VPN packets processed"),
        &["direction", "tunnel_id"]
    ).unwrap();

    pub static ref VPN_BYTES_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("phantom_vpn_bytes_total", "Total bytes transferred through VPN tunnels"),
        &["direction", "tunnel_id"]
    ).unwrap();

    pub static ref VPN_PACKETS_DROPPED_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("phantom_vpn_packets_dropped_total", "Total number of VPN packets dropped"),
        &["reason", "tunnel_id"]
    ).unwrap();

    pub static ref VPN_TUNNEL_LATENCY_SECONDS: Histogram = Histogram::with_opts(
        HistogramOpts::new("phantom_vpn_tunnel_latency_seconds", "VPN tunnel latency in seconds")
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
    ).unwrap();

    pub static ref VPN_ACTIVE_TUNNELS: IntGaugeVec = IntGaugeVec::new(
        Opts::new("phantom_vpn_active_tunnels", "Number of active VPN tunnels"),
        &["protocol", "state"]
    ).unwrap();

    // Threat Detection Metrics
    pub static ref THREAT_EVENTS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("phantom_threat_events_total", "Total number of threat detection events"),
        &["threat_type", "severity", "source"]
    ).unwrap();

    pub static ref THREAT_RESPONSE_TIME_SECONDS: Histogram = Histogram::with_opts(
        HistogramOpts::new("phantom_threat_response_time_seconds", "Time taken to respond to threats")
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0])
    ).unwrap();

    pub static ref THREAT_PATTERNS_ACTIVE: IntGaugeVec = IntGaugeVec::new(
        Opts::new("phantom_threat_patterns_active", "Number of active threat patterns being monitored"),
        &["pattern_type"]
    ).unwrap();

    // ΣVault Metrics
    pub static ref SIGMA_VAULT_SCATTERING_EFFICIENCY: Gauge = Gauge::new(
        "phantom_sigma_vault_scattering_efficiency",
        "Efficiency of ΣVault dimensional scattering (0.0-1.0)"
    ).unwrap();

    pub static ref SIGMA_VAULT_OPERATIONS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("phantom_sigma_vault_operations_total", "Total ΣVault operations performed"),
        &["operation_type", "result"]
    ).unwrap();

    pub static ref SIGMA_VAULT_DIMENSIONS_ACTIVE: IntGaugeVec = IntGaugeVec::new(
        Opts::new("phantom_sigma_vault_dimensions_active", "Number of active dimensional coordinates"),
        &["dimension_type"]
    ).unwrap();

    // System Metrics
    pub static ref SYSTEM_MEMORY_USAGE_BYTES: Gauge = Gauge::new(
        "phantom_system_memory_usage_bytes",
        "Current memory usage in bytes"
    ).unwrap();

    pub static ref SYSTEM_CPU_USAGE_PERCENT: Gauge = Gauge::new(
        "phantom_system_cpu_usage_percent",
        "Current CPU usage percentage"
    ).unwrap();

    pub static ref SYSTEM_NETWORK_CONNECTIONS: IntGaugeVec = IntGaugeVec::new(
        Opts::new("phantom_system_network_connections", "Number of active network connections"),
        &["protocol", "state"]
    ).unwrap();
}

/// Metrics server for exposing Prometheus metrics
pub struct MetricsServer {
    encoder: TextEncoder,
}

impl MetricsServer {
    pub fn new() -> Self {
        Self {
            encoder: TextEncoder::new(),
        }
    }

    /// Generate metrics output in Prometheus format
    pub fn encode_metrics(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = Vec::new();
        self.encoder
            .encode(&METRICS_REGISTRY.gather(), &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

impl Default for MetricsServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize metrics with default values
pub fn init_metrics() {
    // Set initial values
    SIGMA_VAULT_SCATTERING_EFFICIENCY.set(1.0);
    THREAT_PATTERNS_ACTIVE
        .with_label_values(&["signature"])
        .set(0);
    THREAT_PATTERNS_ACTIVE
        .with_label_values(&["behavioral"])
        .set(0);
    THREAT_PATTERNS_ACTIVE
        .with_label_values(&["anomaly"])
        .set(0);
}

/// Update system metrics
pub fn update_system_metrics() {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    // Memory usage
    let total_memory = sys.total_memory() as f64 * 1024.0; // Convert to bytes
    let used_memory = sys.used_memory() as f64 * 1024.0;
    SYSTEM_MEMORY_USAGE_BYTES.set(used_memory);

    // CPU usage
    let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
    SYSTEM_CPU_USAGE_PERCENT.set(cpu_usage);

    // Network connections (simplified)
    // In a real implementation, you'd track actual connections
    SYSTEM_NETWORK_CONNECTIONS
        .with_label_values(&["tcp", "established"])
        .set(0);
    SYSTEM_NETWORK_CONNECTIONS
        .with_label_values(&["udp", "listening"])
        .set(0);
}
