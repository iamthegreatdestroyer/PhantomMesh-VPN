//! Message types for inter-agent communication

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for an agent
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(id: impl Into<String>) -> Self {
        AgentId(id.into())
    }
}

/// Message priority for routing and processing
#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Message action types for agent commands
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    // Command messages
    Command(String),

    // Query messages
    Query(String),

    // Response messages
    Response(String),

    // Event notifications
    Event(String),

    // Alerts
    Alert(String),

    // Coordination
    Coordination(String),
}

/// A message passed between agents
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: String,

    /// Source agent
    pub from: AgentId,

    /// Destination agent(s)
    pub to: Vec<AgentId>,

    /// Message type and content
    pub message_type: MessageType,

    /// Priority level
    pub priority: Priority,

    /// Message payload
    pub payload: HashMap<String, serde_json::Value>,

    /// Timestamp
    pub timestamp: String,

    /// Correlation ID for tracking related messages
    pub correlation_id: Option<String>,
}

impl Message {
    pub fn new(
        from: AgentId,
        to: Vec<AgentId>,
        message_type: MessageType,
        priority: Priority,
    ) -> Self {
        Message {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            message_type,
            priority,
            payload: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            correlation_id: None,
        }
    }

    /// Add data to message payload
    pub fn with_data(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.payload.insert(key.into(), value);
        self
    }

    /// Set correlation ID for message tracking
    pub fn with_correlation_id(mut self, id: String) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Get data from payload
    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.payload.get(key)
    }

    /// Check if message is critical priority
    pub fn is_critical(&self) -> bool {
        self.priority == Priority::Critical
    }
}

/// Builder for constructing complex messages
pub struct MessageBuilder {
    from: AgentId,
    to: Vec<AgentId>,
    message_type: MessageType,
    priority: Priority,
    payload: HashMap<String, serde_json::Value>,
    correlation_id: Option<String>,
}

impl MessageBuilder {
    pub fn new(from: AgentId, message_type: MessageType) -> Self {
        MessageBuilder {
            from,
            to: Vec::new(),
            message_type,
            priority: Priority::Normal,
            payload: HashMap::new(),
            correlation_id: None,
        }
    }

    pub fn to(mut self, agent: AgentId) -> Self {
        self.to.push(agent);
        self
    }

    pub fn to_multiple(mut self, agents: Vec<AgentId>) -> Self {
        self.to.extend(agents);
        self
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn data(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.payload.insert(key.into(), value);
        self
    }

    pub fn correlation_id(mut self, id: String) -> Self {
        self.correlation_id = Some(id);
        self
    }

    pub fn build(self) -> Message {
        Message {
            id: uuid::Uuid::new_v4().to_string(),
            from: self.from,
            to: self.to,
            message_type: self.message_type,
            priority: self.priority,
            payload: self.payload,
            timestamp: chrono::Utc::now().to_rfc3339(),
            correlation_id: self.correlation_id,
        }
    }
}
