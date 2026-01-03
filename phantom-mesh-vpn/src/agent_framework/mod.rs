//! Agent Framework for PhantomMesh
//! ================================
//! Provides the core infrastructure for autonomous agents (APEX, FORTRESS, CIPHER)
//! and their coordination through a shared message-passing system.
//!
//! # Architecture
//!
//! The agent framework implements:
//! - **Trait-based agents** with standard lifecycle (init, process, shutdown)
//! - **Message routing** for inter-agent communication
//! - **State management** with atomic updates
//! - **Event-driven processing** using tokio channels
//! - **Metrics collection** for monitoring and debugging
//!
//! # Agents
//!
//! - **APEX**: Strategic command orchestration, decision-making, task scheduling
//! - **FORTRESS**: Threat detection, security analysis, alert generation
//! - **CIPHER**: Cryptographic operations, key management, secure communication

pub mod apex;
pub mod cipher;
pub mod coordinator;
pub mod fortress;
pub mod message;
pub mod traits;

pub use apex::ApexAgent;
pub use cipher::CipherAgent;
pub use coordinator::AgentCoordinator;
pub use fortress::FortressAgent;
pub use message::{AgentId, Message, Priority};
pub use traits::Agent;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Initialize all agents and prepare the framework
pub async fn init_framework() -> Result<Arc<AgentCoordinator>, String> {
    info!("Initializing PhantomMesh Agent Framework");

    let coordinator = Arc::new(AgentCoordinator::new());

    // Initialize APEX Agent
    let apex = ApexAgent::new(coordinator.clone());
    coordinator.register_agent(apex).await?;

    // Initialize FORTRESS Agent
    let fortress = FortressAgent::new(coordinator.clone());
    coordinator.register_agent(fortress).await?;

    // Initialize CIPHER Agent
    let cipher = CipherAgent::new(coordinator.clone());
    coordinator.register_agent(cipher).await?;

    info!("Agent Framework initialized successfully");
    Ok(coordinator)
}
