//! Agent trait definition and lifecycle management

use crate::agent_framework::message::Message;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent identifier
    fn id(&self) -> &str;

    /// Agent name for display
    fn name(&self) -> &str;

    /// Agent capabilities/role description
    fn capabilities(&self) -> Vec<&str>;

    /// Initialize the agent
    /// Called once during agent startup
    async fn init(&self) -> Result<(), String>;

    /// Process an incoming message
    /// Called for each message the agent receives
    async fn process_message(&self, message: Message) -> Result<Option<Message>, String>;

    /// Periodic health check
    /// Called regularly to maintain agent state
    async fn health_check(&self) -> Result<(), String>;

    /// Shutdown the agent gracefully
    /// Called during system shutdown
    async fn shutdown(&self) -> Result<(), String>;

    /// Get agent metrics
    /// Return current performance and state metrics
    async fn get_metrics(&self) -> AgentMetrics;
}

/// Metrics for monitoring agent health and performance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_id: String,
    pub messages_processed: u64,
    pub messages_failed: u64,
    pub average_response_time_ms: f64,
    pub last_health_check: String,
    pub uptime_seconds: u64,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub is_healthy: bool,
}

/// Agent state information
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentState {
    Initializing,
    Ready,
    Processing,
    Paused,
    Failed,
    Shutdown,
}

/// Agent event for lifecycle and error tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentEvent {
    pub agent_id: String,
    pub timestamp: String,
    pub event_type: AgentEventType,
    pub details: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentEventType {
    Initialized,
    ProcessingMessage,
    MessageProcessed,
    MessageFailed,
    HealthCheckFailed,
    Shutdown,
    Error,
}
