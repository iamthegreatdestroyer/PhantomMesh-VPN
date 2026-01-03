//! Agent coordination and message routing system

use crate::agent_framework::{
    traits::{Agent, AgentMetrics},
    message::{Message, AgentId},
};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};
use std::time::Instant;

type AgentBox = Arc<dyn Agent>;

/// Coordinates multiple agents and routes messages between them
pub struct AgentCoordinator {
    agents: DashMap<AgentId, AgentBox>,
    message_queue: mpsc::UnboundedSender<Message>,
    message_receiver: tokio::sync::Mutex<mpsc::UnboundedReceiver<Message>>,
    statistics: Arc<CoordinatorStatistics>,
}

/// Statistics for coordinator performance monitoring
#[derive(Default)]
struct CoordinatorStatistics {
    messages_processed: std::sync::atomic::AtomicU64,
    messages_failed: std::sync::atomic::AtomicU64,
    total_processing_time_ms: std::sync::atomic::AtomicU64,
}

impl AgentCoordinator {
    /// Create a new coordinator
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        AgentCoordinator {
            agents: DashMap::new(),
            message_queue: tx,
            message_receiver: tokio::sync::Mutex::new(rx),
            statistics: Arc::new(CoordinatorStatistics::default()),
        }
    }

    /// Register an agent with the coordinator
    pub async fn register_agent(&self, agent: AgentBox) -> Result<(), String> {
        let agent_id = AgentId::new(agent.id());
        
        // Initialize the agent
        agent.init().await?;
        
        // Store the agent
        self.agents.insert(agent_id.clone(), agent.clone());
        
        info!("Agent registered: {} ({})", agent.id(), agent.name());
        Ok(())
    }

    /// Get an agent by ID
    pub fn get_agent(&self, id: &AgentId) -> Option<AgentBox> {
        self.agents.get(id).map(|entry| Arc::clone(&entry.value()))
    }

    /// List all registered agents
    pub fn list_agents(&self) -> Vec<AgentId> {
        self.agents
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Send a message to target agent(s)
    pub async fn send_message(&self, mut message: Message) -> Result<(), String> {
        debug!("Routing message {} from {} to {} target(s)", 
            message.id, message.from.0, message.to.len());

        // Validate targets exist
        for target in &message.to {
            if !self.agents.contains_key(target) {
                warn!("Target agent not found: {}", target.0);
                return Err(format!("Agent not found: {}", target.0));
            }
        }

        self.message_queue.send(message)
            .map_err(|e| format!("Failed to queue message: {}", e))
    }

    /// Process all pending messages
    pub async fn process_messages(&self) -> Result<usize, String> {
        let mut count = 0;
        let mut receiver = self.message_receiver.lock().await;

        while let Ok(message) = receiver.try_recv() {
            let start = Instant::now();
            
            match self.process_single_message(message).await {
                Ok(_) => {
                    count += 1;
                    let elapsed = start.elapsed().as_millis() as u64;
                    self.statistics.messages_processed
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    self.statistics.total_processing_time_ms
                        .fetch_add(elapsed, std::sync::atomic::Ordering::Relaxed);
                }
                Err(e) => {
                    warn!("Error processing message: {}", e);
                    self.statistics.messages_failed
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }

        Ok(count)
    }

    /// Process a single message
    async fn process_single_message(&self, message: Message) -> Result<(), String> {
        let from_id = message.from.clone();
        let msg_id = message.id.clone();
        let message_type = format!("{:?}", message.message_type);
        
        debug!("Processing message {} from {}: {}", msg_id, from_id.0, message_type);

        for target in message.to.iter() {
            if let Some(agent_entry) = self.agents.get(target) {
                let agent = Arc::clone(&agent_entry.value());
                drop(agent_entry); // Release lock

                match agent.process_message(message.clone()).await {
                    Ok(Some(response)) => {
                        debug!("Agent {} produced response to message {}", target.0, msg_id);
                        // Could queue response for further routing
                    }
                    Ok(None) => {
                        debug!("Agent {} processed message {} (no response)", target.0, msg_id);
                    }
                    Err(e) => {
                        error!("Agent {} failed to process message {}: {}", target.0, msg_id, e);
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Run the coordinator's main loop
    /// This continuously processes messages from agents
    pub async fn run(&self, shutdown_signal: tokio::sync::broadcast::Receiver<()>) -> Result<(), String> {
        let mut shutdown = shutdown_signal;
        
        info!("Agent coordinator started");

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Coordinator shutting down");
                    break;
                }
                result = self.process_messages() => {
                    match result {
                        Ok(count) if count > 0 => {
                            debug!("Processed {} messages", count);
                        }
                        Err(e) => {
                            error!("Error processing messages: {}", e);
                        }
                        _ => {}
                    }
                }
            }

            // Yield to other tasks
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Shutdown all agents
        for entry in self.agents.iter() {
            let agent = entry.value();
            if let Err(e) = agent.shutdown().await {
                error!("Error shutting down agent {}: {}", agent.id(), e);
            }
        }

        Ok(())
    }

    /// Get metrics for all agents
    pub async fn get_agent_metrics(&self) -> Vec<AgentMetrics> {
        let mut metrics = Vec::new();

        for entry in self.agents.iter() {
            let agent = entry.value();
            let agent_metrics = agent.get_metrics().await;
            metrics.push(agent_metrics);
        }

        metrics
    }

    /// Get coordinator statistics
    pub fn get_statistics(&self) -> CoordinatorStats {
        CoordinatorStats {
            messages_processed: self.statistics.messages_processed
                .load(std::sync::atomic::Ordering::Relaxed),
            messages_failed: self.statistics.messages_failed
                .load(std::sync::atomic::Ordering::Relaxed),
            average_processing_time_ms: {
                let total = self.statistics.total_processing_time_ms
                    .load(std::sync::atomic::Ordering::Relaxed);
                let processed = self.statistics.messages_processed
                    .load(std::sync::atomic::Ordering::Relaxed);
                if processed > 0 {
                    total as f64 / processed as f64
                } else {
                    0.0
                }
            },
            active_agents: self.agents.len(),
        }
    }
}

/// Statistics about coordinator performance
#[derive(Clone, Debug)]
pub struct CoordinatorStats {
    pub messages_processed: u64,
    pub messages_failed: u64,
    pub average_processing_time_ms: f64,
    pub active_agents: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = AgentCoordinator::new();
        assert_eq!(coordinator.list_agents().len(), 0);
    }
}
