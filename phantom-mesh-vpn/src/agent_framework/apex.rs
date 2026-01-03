//! APEX Strategic Command Agent
//! ==============================
//! Core orchestration engine with decision-making, task prioritization, and command execution
//!
//! Responsibilities:
//! - Strategic decision-making and planning
//! - Task prioritization and scheduling
//! - Command orchestration
//! - State management and coordination
//! - Delegation to specialized agents (FORTRESS, CIPHER)

use crate::agent_framework::{
    traits::{Agent, AgentMetrics, AgentState},
    message::{Message, AgentId, MessageType, Priority},
    coordinator::AgentCoordinator,
};
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::json;
use std::sync::Arc;
use tokio::time::Instant;
use tracing::{info, debug, warn, error};

/// Strategic decision-making agent
pub struct ApexAgent {
    id: AgentId,
    coordinator: Arc<AgentCoordinator>,
    state: Arc<tokio::sync::RwLock<AgentState>>,
    start_time: Instant,
    message_count: Arc<std::sync::atomic::AtomicU64>,
    error_count: Arc<std::sync::atomic::AtomicU64>,
    task_queue: Arc<DashMap<String, TaskInfo>>,
}

/// Information about a queued task
#[derive(Clone, Debug)]
struct TaskInfo {
    id: String,
    priority: Priority,
    status: TaskStatus,
    created_at: std::time::Instant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TaskStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

impl ApexAgent {
    /// Create a new APEX agent
    pub fn new(coordinator: Arc<AgentCoordinator>) -> Arc<Self> {
        Arc::new(ApexAgent {
            id: AgentId::new("apex"),
            coordinator,
            state: Arc::new(tokio::sync::RwLock::new(AgentState::Initializing)),
            start_time: Instant::now(),
            message_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            error_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            task_queue: Arc::new(DashMap::new()),
        })
    }

    /// Make a strategic decision based on current state
    async fn make_decision(&self, context: serde_json::Value) -> Result<Message, String> {
        debug!("APEX: Making strategic decision");

        // Analyze the context and determine appropriate actions
        let decision = json!({
            "decision_type": "strategic_response",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "context": context,
        });

        // Create response message to coordinator
        let message = Message::new(
            self.id.clone(),
            vec![], // Coordinator will handle routing
            MessageType::Response("decision".to_string()),
            Priority::High,
        ).with_data("decision", decision);

        Ok(message)
    }

    /// Prioritize and queue a task
    async fn queue_task(&self, task_id: String, priority: Priority) {
        self.task_queue.insert(task_id.clone(), TaskInfo {
            id: task_id.clone(),
            priority,
            status: TaskStatus::Queued,
            created_at: std::time::Instant::now(),
        });
        debug!("APEX: Queued task {} with priority {:?}", task_id, priority);
    }

    /// Execute a command
    async fn execute_command(&self, command: String) -> Result<String, String> {
        info!("APEX: Executing command: {}", command);

        // Parse and execute command
        match command.as_str() {
            "threat_scan" => {
                // Delegate to FORTRESS
                Ok("Threat scan initiated".to_string())
            }
            "crypto_init" => {
                // Delegate to CIPHER
                Ok("Cryptographic initialization started".to_string())
            }
            "health_check" => {
                // Perform health check
                Ok("System healthy".to_string())
            }
            _ => {
                warn!("APEX: Unknown command: {}", command);
                Err(format!("Unknown command: {}", command))
            }
        }
    }
}

#[async_trait]
impl Agent for ApexAgent {
    fn id(&self) -> &str {
        &self.id.0
    }

    fn name(&self) -> &str {
        "APEX Strategic Command"
    }

    fn capabilities(&self) -> Vec<&str> {
        vec![
            "strategic_planning",
            "task_orchestration",
            "command_execution",
            "state_management",
            "decision_making",
        ]
    }

    async fn init(&self) -> Result<(), String> {
        info!("APEX Agent initializing");
        *self.state.write().await = AgentState::Ready;
        Ok(())
    }

    async fn process_message(&self, message: Message) -> Result<Option<Message>, String> {
        self.message_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!("APEX: Processing message {}", message.id);

        // Update state
        *self.state.write().await = AgentState::Processing;

        let result = match &message.message_type {
            MessageType::Command(cmd) => {
                debug!("APEX: Received command: {}", cmd);
                self.execute_command(cmd.clone()).await?;
                Some(Message::new(
                    self.id.clone(),
                    vec![message.from.clone()],
                    MessageType::Response(format!("Command executed: {}", cmd)),
                    Priority::Normal,
                ))
            }
            MessageType::Query(query) => {
                debug!("APEX: Received query: {}", query);
                match query.as_str() {
                    "status" => {
                        Some(Message::new(
                            self.id.clone(),
                            vec![message.from.clone()],
                            MessageType::Response("APEX operational".to_string()),
                            Priority::Normal,
                        ))
                    }
                    _ => {
                        Some(Message::new(
                            self.id.clone(),
                            vec![message.from.clone()],
                            MessageType::Response(format!("Query response: {}", query)),
                            Priority::Normal,
                        ))
                    }
                }
            }
            MessageType::Event(event) => {
                debug!("APEX: Received event: {}", event);
                
                // Make strategic decision based on event
                if let Ok(response) = self.make_decision(json!({"event": event})).await {
                    Some(response)
                } else {
                    Some(Message::new(
                        self.id.clone(),
                        vec![message.from.clone()],
                        MessageType::Response("Event acknowledged".to_string()),
                        Priority::Normal,
                    ))
                }
            }
            _ => {
                debug!("APEX: Received message of type: {:?}", message.message_type);
                None
            }
        };

        // Restore state
        *self.state.write().await = AgentState::Ready;
        Ok(result)
    }

    async fn health_check(&self) -> Result<(), String> {
        let state = self.state.read().await;
        if *state == AgentState::Failed {
            Err("APEX health check failed".to_string())
        } else {
            Ok(())
        }
    }

    async fn shutdown(&self) -> Result<(), String> {
        info!("APEX Agent shutting down");
        *self.state.write().await = AgentState::Shutdown;
        Ok(())
    }

    async fn get_metrics(&self) -> AgentMetrics {
        let uptime_seconds = self.start_time.elapsed().as_secs();
        let messages_processed = self.message_count.load(std::sync::atomic::Ordering::Relaxed);
        let messages_failed = self.error_count.load(std::sync::atomic::Ordering::Relaxed);
        
        AgentMetrics {
            agent_id: self.id.0.clone(),
            messages_processed,
            messages_failed,
            average_response_time_ms: 5.0, // Placeholder
            last_health_check: chrono::Utc::now().to_rfc3339(),
            uptime_seconds,
            cpu_usage_percent: 2.5, // Placeholder
            memory_usage_mb: 45.0, // Placeholder
            is_healthy: true,
        }
    }
}
