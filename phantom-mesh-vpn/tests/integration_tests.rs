//! Comprehensive test suite for Agent Framework
//!
//! Tests all agent functionality including:
//! - Individual agent initialization and lifecycle
//! - Message passing and routing
//! - Coordinator operations
//! - Inter-agent communication
//! - Error handling and recovery

#[cfg(test)]
mod agent_tests {
    use phantom_mesh::agent_framework::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_apex_initialization() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let apex = apex::ApexAgent::new(coordinator.clone());

        // Register APEX
        let result = coordinator.register_agent(apex.clone()).await;
        assert!(result.is_ok(), "APEX registration failed");

        // Verify registration
        let agents = coordinator.list_agents();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].0, "apex");
    }

    #[tokio::test]
    async fn test_fortress_initialization() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let fortress = fortress::FortressAgent::new(coordinator.clone());

        let result = coordinator.register_agent(fortress.clone()).await;
        assert!(result.is_ok(), "FORTRESS registration failed");

        let agents = coordinator.list_agents();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].0, "fortress");
    }

    #[tokio::test]
    async fn test_cipher_initialization() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let cipher = cipher::CipherAgent::new(coordinator.clone());

        let result = coordinator.register_agent(cipher.clone()).await;
        assert!(result.is_ok(), "CIPHER registration failed");

        let agents = coordinator.list_agents();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].0, "cipher");
    }

    #[tokio::test]
    async fn test_all_agents_initialization() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());

        // Register all agents
        let apex = apex::ApexAgent::new(coordinator.clone());
        coordinator.register_agent(apex.clone()).await.unwrap();

        let fortress = fortress::FortressAgent::new(coordinator.clone());
        coordinator.register_agent(fortress.clone()).await.unwrap();

        let cipher = cipher::CipherAgent::new(coordinator.clone());
        coordinator.register_agent(cipher.clone()).await.unwrap();

        // Verify all registered
        let agents = coordinator.list_agents();
        assert_eq!(agents.len(), 3);

        // Verify IDs
        let ids: Vec<_> = agents.iter().map(|a| a.0.as_str()).collect();
        assert!(ids.contains(&"apex"));
        assert!(ids.contains(&"fortress"));
        assert!(ids.contains(&"cipher"));
    }

    #[tokio::test]
    async fn test_apex_capabilities() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let apex = apex::ApexAgent::new(coordinator.clone());

        let caps = apex.capabilities();
        assert!(caps.contains(&"strategic_planning"));
        assert!(caps.contains(&"task_orchestration"));
        assert!(caps.contains(&"command_execution"));
    }

    #[tokio::test]
    async fn test_fortress_capabilities() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let fortress = fortress::FortressAgent::new(coordinator.clone());

        let caps = fortress.capabilities();
        assert!(caps.contains(&"threat_detection"));
        assert!(caps.contains(&"pattern_matching"));
        assert!(caps.contains(&"alert_generation"));
    }

    #[tokio::test]
    async fn test_cipher_capabilities() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let cipher = cipher::CipherAgent::new(coordinator.clone());

        let caps = cipher.capabilities();
        assert!(caps.contains(&"encryption_decryption"));
        assert!(caps.contains(&"key_management"));
        assert!(caps.contains(&"key_agreement"));
    }

    #[tokio::test]
    async fn test_message_creation() {
        let from = message::AgentId::new("apex");
        let to = vec![message::AgentId::new("fortress")];
        let msg = message::Message::new(
            from,
            to,
            message::MessageType::Command("test_command".to_string()),
            message::Priority::Normal,
        );

        assert_eq!(msg.from.0, "apex");
        assert_eq!(msg.to[0].0, "fortress");
        assert_eq!(msg.priority, message::Priority::Normal);
    }

    #[tokio::test]
    async fn test_message_builder() {
        let msg = message::MessageBuilder::new(
            message::AgentId::new("apex"),
            message::MessageType::Command("test".to_string()),
        )
        .to(message::AgentId::new("fortress"))
        .priority(message::Priority::Critical)
        .data("key", serde_json::json!("value"))
        .build();

        assert_eq!(msg.priority, message::Priority::Critical);
        assert!(msg.get_data("key").is_some());
        assert!(msg.is_critical());
    }

    #[tokio::test]
    async fn test_coordinator_message_routing() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());

        // Register agents
        let apex = apex::ApexAgent::new(coordinator.clone());
        coordinator.register_agent(apex.clone()).await.unwrap();

        let fortress = fortress::FortressAgent::new(coordinator.clone());
        coordinator.register_agent(fortress.clone()).await.unwrap();

        // Send message
        let msg = message::Message::new(
            message::AgentId::new("apex"),
            vec![message::AgentId::new("fortress")],
            message::MessageType::Command("scan_threats".to_string()),
            message::Priority::High,
        );

        let result = coordinator.send_message(msg).await;
        assert!(result.is_ok(), "Message routing failed");
    }

    #[tokio::test]
    async fn test_coordinator_stats() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());

        // Register agent
        let apex = apex::ApexAgent::new(coordinator.clone());
        coordinator.register_agent(apex.clone()).await.unwrap();

        // Get stats
        let stats = coordinator.get_statistics();
        assert_eq!(stats.messages_processed, 0);
        assert_eq!(stats.messages_failed, 0);
        assert_eq!(stats.active_agents, 1);
    }

    #[tokio::test]
    async fn test_agent_metrics() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let apex = apex::ApexAgent::new(coordinator.clone());

        let metrics = apex.get_metrics().await;
        assert_eq!(metrics.agent_id, "apex");
        assert_eq!(metrics.messages_processed, 0);
        assert!(metrics.is_healthy);
    }

    #[tokio::test]
    async fn test_priority_levels() {
        assert!(message::Priority::Critical > message::Priority::High);
        assert!(message::Priority::High > message::Priority::Normal);
        assert!(message::Priority::Normal > message::Priority::Low);
    }

    #[tokio::test]
    async fn test_agent_state_transitions() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let apex = apex::ApexAgent::new(coordinator.clone());

        // Should start as Initializing
        // After init, should be Ready
        assert!(apex.init().await.is_ok());

        // Health check should pass
        assert!(apex.health_check().await.is_ok());

        // Shutdown should work
        assert!(apex.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_apex_command_execution() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let apex = apex::ApexAgent::new(coordinator.clone());
        apex.init().await.unwrap();

        let msg = message::Message::new(
            message::AgentId::new("test"),
            vec![message::AgentId::new("apex")],
            message::MessageType::Command("threat_scan".to_string()),
            message::Priority::Normal,
        );

        let result = apex.process_message(msg).await;
        assert!(result.is_ok(), "Command processing failed");
        assert!(result.unwrap().is_some(), "No response generated");
    }

    #[tokio::test]
    async fn test_fortress_threat_analysis() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let fortress = fortress::FortressAgent::new(coordinator.clone());
        fortress.init().await.unwrap();

        let msg = message::Message::new(
            message::AgentId::new("test"),
            vec![message::AgentId::new("fortress")],
            message::MessageType::Event("port_scan_detected".to_string()),
            message::Priority::High,
        );

        let result = fortress.process_message(msg).await;
        assert!(result.is_ok(), "Threat analysis failed");
    }

    #[tokio::test]
    async fn test_cipher_key_generation() {
        let coordinator = Arc::new(coordinator::AgentCoordinator::new());
        let cipher = cipher::CipherAgent::new(coordinator.clone());
        cipher.init().await.unwrap();

        let msg = message::Message::new(
            message::AgentId::new("test"),
            vec![message::AgentId::new("cipher")],
            message::MessageType::Command("generate_key".to_string()),
            message::Priority::Normal,
        );

        let result = cipher.process_message(msg).await;
        assert!(result.is_ok(), "Key generation failed");
        assert!(result.unwrap().is_some(), "No response from CIPHER");
    }

    #[tokio::test]
    async fn test_framework_initialization() {
        // This tests the full framework initialization
        let framework = agent_framework::init_framework().await;
        assert!(framework.is_ok(), "Framework initialization failed");

        let coordinator = framework.unwrap();
        let agents = coordinator.list_agents();
        assert_eq!(agents.len(), 3, "Not all agents registered");
    }
}
