//! API Gateway Module
//! ===================
//! REST API for VPN management and agent coordination.
//!
//! Copyright © 2025 Stephen Bilodeau. All rights reserved.
//! Licensed under GPL-3.0 with proprietary agent clauses.

use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
    response::Json as AxumJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::metrics::{MetricsServer, METRICS_REGISTRY};
use super::super::security_layer::threat_engine::{ThreatEngine, ThreatResult, PerformanceMetrics};
use prometheus::Encoder;

/// REST API gateway for VPN management
pub struct ApiGateway {
    threat_engine: Arc<Mutex<ThreatEngine>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
}

#[derive(Deserialize, Serialize)]
struct ThreatAnalysisRequest {
    packet_data: Vec<u8>,
    source_ip: Option<String>,
}

#[derive(Serialize)]
struct ThreatAnalysisResponse {
    threat_detected: bool,
    threat_result: Option<ThreatResult>,
    analysis_time_us: u128,
}

#[derive(Serialize)]
struct MetricsResponse {
    metrics: PerformanceMetrics,
}

#[derive(Deserialize)]
struct UpdateSignaturesRequest {
    signatures: Vec<super::super::security_layer::threat_engine::ThreatSignature>,
}

#[derive(Serialize)]
struct UpdateSignaturesResponse {
    success: bool,
    signatures_added: usize,
}

impl ApiGateway {
    pub fn new(threat_engine: Arc<Mutex<ThreatEngine>>) -> Self {
        Self { threat_engine }
    }

    pub fn router(&self) -> Router {
        let threat_engine = Arc::clone(&self.threat_engine);

        Router::new()
            .route("/health", get(health_check))
            .route("/metrics", get(prometheus_metrics))
            .route("/threat/analyze", post(analyze_threat))
            .route("/threat/metrics", get(get_metrics))
            .route("/threat/signatures", post(update_signatures))
            .with_state(threat_engine)
    }

    pub async fn serve(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.router();
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: 0, // TODO: Implement uptime tracking
    })
}

#[axum::debug_handler]
async fn analyze_threat(
    State(threat_engine): State<Arc<Mutex<ThreatEngine>>>,
    Json(request): Json<ThreatAnalysisRequest>,
) -> AxumJson<ThreatAnalysisResponse> {
    let start_time = std::time::Instant::now();
    let engine = threat_engine.lock().await;

    let threat_result = engine.analyze_packet(&request.packet_data, request.source_ip.as_deref()).await;

    let analysis_time = start_time.elapsed().as_micros();

    AxumJson(ThreatAnalysisResponse {
        threat_detected: threat_result.is_some(),
        threat_result,
        analysis_time_us: analysis_time,
    })
}

async fn get_metrics(
    State(threat_engine): State<Arc<Mutex<ThreatEngine>>>,
) -> AxumJson<MetricsResponse> {
    let engine = threat_engine.lock().await;
    let metrics = engine.get_metrics().await;

    AxumJson(MetricsResponse { metrics })
}

async fn update_signatures(
    State(threat_engine): State<Arc<Mutex<ThreatEngine>>>,
    Json(request): Json<UpdateSignaturesRequest>,
) -> AxumJson<UpdateSignaturesResponse> {
    let sig_count = request.signatures.len();
    let engine = threat_engine.lock().await;

    match engine.update_signatures(request.signatures).await {
        Ok(_) => AxumJson(UpdateSignaturesResponse {
            success: true,
            signatures_added: sig_count,
        }),
        Err(e) => {
            tracing::error!("Failed to update signatures: {}", e);
            AxumJson(UpdateSignaturesResponse {
                success: false,
                signatures_added: 0,
            })
        }
    }
}

async fn prometheus_metrics() -> String {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    encoder.encode_to_string(&metric_families).unwrap_or_else(|_| "# Error encoding metrics".to_string())
}