#!/bin/bash
# P0-001: Test Infrastructure Scaffold
# PhantomMesh-VPN Automation Script
# 
# Execute with: bash scripts/p0-001-test-scaffold.sh
# 
# This script creates the complete test infrastructure for the project.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  P0-001: Test Infrastructure Scaffold                        ║"
echo "║  PhantomMesh-VPN Automation                                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd "$PROJECT_ROOT"

echo "📦 Creating test directory structure..."

# Create Rust test structure
mkdir -p tests/rust/{unit,integration}
mkdir -p tests/fixtures
mkdir -p tests/mocks

# Create Python test structure  
mkdir -p tests/python/{unit,integration}

# Create Python __init__.py files
touch tests/python/__init__.py
touch tests/python/unit/__init__.py
touch tests/python/integration/__init__.py

echo "📝 Creating Rust unit test modules..."

# Create Rust test module files
cat > tests/rust/unit/mod.rs << 'EOF'
//! Unit tests for PhantomMesh-VPN
//! 
//! Test organization:
//! - crypto_tests: ΣVault and cryptographic primitives
//! - tunnel_tests: Tunnel engine functionality
//! - threat_tests: Threat detection engine
//! - metrics_tests: Prometheus metrics

pub mod crypto_tests;
pub mod tunnel_tests;
pub mod threat_tests;
pub mod metrics_tests;
EOF

cat > tests/rust/unit/crypto_tests.rs << 'EOF'
//! Cryptographic module tests

use phantom_mesh::security_layer::crypto_manager::CryptoManager;

#[test]
fn test_kyber_keypair_generation() {
    let crypto = CryptoManager::new().expect("CryptoManager init failed");
    let (pk, sk) = crypto.generate_kyber_keypair().expect("Keypair gen failed");
    
    // Kyber-768 public key is 1184 bytes
    assert!(!pk.as_bytes().is_empty());
    assert!(!sk.as_bytes().is_empty());
}

#[test]
fn test_kyber_encapsulation_roundtrip() {
    let crypto = CryptoManager::new().expect("CryptoManager init failed");
    let (pk, sk) = crypto.generate_kyber_keypair().expect("Keypair gen failed");
    
    let (ciphertext, shared_secret_sender) = crypto.kyber_encapsulate(&pk).expect("Encapsulate failed");
    let shared_secret_receiver = crypto.kyber_decapsulate(&ciphertext, &sk).expect("Decapsulate failed");
    
    assert_eq!(shared_secret_sender, shared_secret_receiver);
}

#[test]
fn test_chacha_encryption_roundtrip() {
    let crypto = CryptoManager::new().expect("CryptoManager init failed");
    
    let key = crypto.random_bytes(32).expect("Random key failed");
    let nonce = crypto.random_bytes(12).expect("Random nonce failed");
    let plaintext = b"PhantomMesh VPN test message";
    
    let ciphertext = crypto.encrypt_chacha(plaintext, &key, &nonce).expect("Encrypt failed");
    let decrypted = crypto.decrypt_chacha(&ciphertext, &key, &nonce).expect("Decrypt failed");
    
    assert_eq!(plaintext.to_vec(), decrypted);
}

#[test]
fn test_dilithium_signature_verification() {
    let crypto = CryptoManager::new().expect("CryptoManager init failed");
    let (pk, sk) = crypto.generate_dilithium_keypair().expect("Keypair gen failed");
    
    let message = b"Sign this message";
    let signature = crypto.dilithium_sign(message, &sk).expect("Sign failed");
    let valid = crypto.dilithium_verify(message, &signature, &pk).expect("Verify failed");
    
    assert!(valid);
}

#[test]
fn test_signature_rejects_tampered_message() {
    let crypto = CryptoManager::new().expect("CryptoManager init failed");
    let (pk, sk) = crypto.generate_dilithium_keypair().expect("Keypair gen failed");
    
    let message = b"Original message";
    let tampered = b"Tampered message";
    let signature = crypto.dilithium_sign(message, &sk).expect("Sign failed");
    
    // Verification should fail for tampered message
    let valid = crypto.dilithium_verify(tampered, &signature, &pk).unwrap_or(false);
    assert!(!valid);
}
EOF

cat > tests/rust/unit/tunnel_tests.rs << 'EOF'
//! Tunnel engine tests

// TODO: Implement after tunnel_engine.rs is complete
// use phantom_mesh::vpn_core::tunnel_engine::{TunnelEngine, TunnelEvent};

#[test]
fn test_tunnel_placeholder() {
    // Placeholder test
    assert!(true);
}

#[test]
fn test_peer_lifecycle() {
    // TODO: Test peer add/remove
    assert!(true);
}

#[test]
fn test_packet_routing() {
    // TODO: Test packet routing through dimensions
    assert!(true);
}
EOF

cat > tests/rust/unit/threat_tests.rs << 'EOF'
//! Threat engine tests

// use phantom_mesh::security_layer::threat_engine::ThreatEngine;

#[tokio::test]
async fn test_threat_engine_initialization() {
    // TODO: Uncomment when ThreatEngine is accessible
    // let engine = ThreatEngine::new().expect("ThreatEngine init failed");
    // engine.initialize().await.expect("Init failed");
    assert!(true);
}

#[test]
fn test_signature_matching() {
    // TODO: Test threat signature matching
    assert!(true);
}

#[test]
fn test_anomaly_detection() {
    // TODO: Test statistical anomaly detection
    assert!(true);
}
EOF

cat > tests/rust/unit/metrics_tests.rs << 'EOF'
//! Prometheus metrics tests

use phantom_mesh::metrics::{init_metrics, METRICS_REGISTRY};

#[test]
fn test_metrics_initialization() {
    init_metrics();
    
    // Registry should have metrics registered
    let metrics = METRICS_REGISTRY.gather();
    assert!(!metrics.is_empty());
}

#[test]
fn test_vpn_metrics_exist() {
    init_metrics();
    
    let metrics = METRICS_REGISTRY.gather();
    let metric_names: Vec<&str> = metrics.iter().map(|m| m.get_name()).collect();
    
    // Check that key metrics are registered
    assert!(metric_names.iter().any(|n| n.contains("phantom_vpn")));
    assert!(metric_names.iter().any(|n| n.contains("phantom_threat")));
    assert!(metric_names.iter().any(|n| n.contains("phantom_sigma_vault")));
}
EOF

cat > tests/rust/integration/mod.rs << 'EOF'
//! Integration tests for PhantomMesh-VPN
//!
//! These tests verify end-to-end functionality:
//! - Full tunnel establishment
//! - Multi-peer communication
//! - Threat detection and response

pub mod tunnel_integration;
pub mod peer_mesh_integration;
pub mod threat_response_integration;
EOF

cat > tests/rust/integration/tunnel_integration.rs << 'EOF'
//! End-to-end tunnel integration tests

#[tokio::test]
async fn test_tunnel_establishment() {
    // TODO: Test full tunnel setup between two nodes
    assert!(true);
}

#[tokio::test]
async fn test_encrypted_data_transfer() {
    // TODO: Test data transfer through established tunnel
    assert!(true);
}
EOF

cat > tests/rust/integration/peer_mesh_integration.rs << 'EOF'
//! Mesh formation integration tests

#[tokio::test]
async fn test_multi_peer_mesh() {
    // TODO: Test mesh with 3+ peers
    assert!(true);
}

#[tokio::test]
async fn test_mesh_routing_optimization() {
    // TODO: Test dynamic route optimization
    assert!(true);
}
EOF

cat > tests/rust/integration/threat_response_integration.rs << 'EOF'
//! Threat detection and response integration tests

#[tokio::test]
async fn test_threat_detection_in_tunnel() {
    // TODO: Test threat detection during active tunnel
    assert!(true);
}

#[tokio::test]
async fn test_automatic_threat_response() {
    // TODO: Test automatic quarantine/block response
    assert!(true);
}
EOF

echo "📝 Creating test fixtures..."

cat > tests/fixtures/sample_configs.yaml << 'EOF'
# Test configuration fixtures
test_node:
  id: "test-node-001"
  tier: "relay"
  capabilities: ["route", "cache"]
  
test_peer:
  public_key: "dGVzdF9wdWJsaWNfa2V5X2Jhc2U2NA=="
  endpoint: "127.0.0.1:51820"
  allowed_ips: ["10.0.0.0/24"]
  
test_mesh:
  network_id: "test-mesh"
  subnet: "10.100.0.0/16"
  max_peers: 10
EOF

cat > tests/fixtures/threat_signatures.json << 'EOF'
{
  "signatures": [
    {
      "id": "test_sig_001",
      "pattern": "90909090",
      "description": "Test NOP sled signature",
      "severity": "high",
      "category": "malware"
    },
    {
      "id": "test_sig_002", 
      "pattern": "414141414141",
      "description": "Test buffer overflow pattern",
      "severity": "critical",
      "category": "exploit"
    }
  ]
}
EOF

echo "📝 Creating mock modules..."

cat > tests/mocks/mock_network.rs << 'EOF'
//! Mock network layer for testing
//! 
//! Provides simulated network operations without actual socket I/O

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockNetwork {
    packets: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    latency_ms: u64,
}

impl MockNetwork {
    pub fn new() -> Self {
        Self {
            packets: Arc::new(Mutex::new(HashMap::new())),
            latency_ms: 0,
        }
    }
    
    pub fn with_latency(latency_ms: u64) -> Self {
        Self {
            packets: Arc::new(Mutex::new(HashMap::new())),
            latency_ms,
        }
    }
    
    pub fn send(&self, dest: &str, data: &[u8]) -> Result<(), String> {
        let mut packets = self.packets.lock().unwrap();
        packets.insert(dest.to_string(), data.to_vec());
        Ok(())
    }
    
    pub fn receive(&self, src: &str) -> Option<Vec<u8>> {
        let packets = self.packets.lock().unwrap();
        packets.get(src).cloned()
    }
    
    pub fn clear(&self) {
        let mut packets = self.packets.lock().unwrap();
        packets.clear();
    }
}

impl Default for MockNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_send_receive() {
        let network = MockNetwork::new();
        network.send("peer1", b"hello").unwrap();
        let received = network.receive("peer1");
        assert_eq!(received, Some(b"hello".to_vec()));
    }
    
    #[test]
    fn test_mock_clear() {
        let network = MockNetwork::new();
        network.send("peer1", b"data").unwrap();
        network.clear();
        assert!(network.receive("peer1").is_none());
    }
}
EOF

cat > tests/mocks/mod.rs << 'EOF'
//! Test mocks and fakes for PhantomMesh-VPN

pub mod mock_network;

pub use mock_network::MockNetwork;
EOF

echo "📝 Creating Python test files..."

cat > tests/python/unit/test_orchestrator.py << 'EOF'
"""
Unit tests for PhantomMesh Agent Swarm Orchestrator
"""

import pytest
import asyncio
from unittest.mock import Mock, AsyncMock, patch

import sys
sys.path.insert(0, 'src')

from agent_swarm.phantom_orchestrator import (
    MnemonicCache,
    AgentRole,
    AgentState,
    PhantomOrchestrator,
)


class TestMnemonicCache:
    """Tests for MNEMONIC cache system."""
    
    def test_store_and_recall(self):
        cache = MnemonicCache(max_size=100)
        cache.store("test_key", "test_value", priority=5)
        result = cache.recall("test_key")
        assert result == "test_value"
    
    def test_recall_boosts_priority(self):
        cache = MnemonicCache(max_size=100)
        cache.store("key", "value", priority=1)
        
        # Access multiple times
        for _ in range(5):
            cache.recall("key")
        
        # Priority should be boosted (capped at 10)
        _, _, priority = cache._cache["key"]
        assert priority > 1
        assert priority <= 10
    
    def test_eviction_on_overflow(self):
        cache = MnemonicCache(max_size=3)
        
        cache.store("key1", "val1", priority=1)
        cache.store("key2", "val2", priority=2)
        cache.store("key3", "val3", priority=3)
        cache.store("key4", "val4", priority=4)  # Should evict key1
        
        assert cache.recall("key1") is None  # Evicted
        assert cache.recall("key2") is not None
        assert cache.recall("key4") is not None
    
    def test_recall_missing_key_returns_none(self):
        cache = MnemonicCache(max_size=100)
        result = cache.recall("nonexistent")
        assert result is None


class TestAgentState:
    """Tests for agent state management."""
    
    def test_default_state(self):
        state = AgentState(role=AgentRole.APEX)
        assert state.active is True
        assert state.performance_score == 1.0
        assert len(state.task_queue) == 0
    
    def test_task_queue(self):
        state = AgentState(role=AgentRole.FORTRESS)
        state.task_queue.append("task1")
        state.task_queue.append("task2")
        
        assert len(state.task_queue) == 2
        assert state.task_queue[0] == "task1"


class TestPhantomOrchestrator:
    """Tests for main orchestrator."""
    
    def test_orchestrator_initialization(self):
        orchestrator = PhantomOrchestrator()
        assert orchestrator.mnemonic is not None
        assert len(orchestrator.agents) == 0  # Before spawn
    
    @pytest.mark.asyncio
    async def test_spawn_swarm(self):
        orchestrator = PhantomOrchestrator()
        await orchestrator.spawn_swarm()
        
        # All 10 agents should be spawned
        assert len(orchestrator.agents) == 10
        assert AgentRole.APEX in orchestrator.agents
        assert AgentRole.FORTRESS in orchestrator.agents
    
    @pytest.mark.asyncio
    async def test_dispatch_directive(self):
        orchestrator = PhantomOrchestrator()
        await orchestrator.spawn_swarm()
        
        await orchestrator.dispatch_directive(
            AgentRole.APEX,
            "test_directive",
            priority=5
        )
        
        apex_agent = orchestrator.agents[AgentRole.APEX]
        assert len(apex_agent.state.task_queue) == 1
        assert apex_agent.state.task_queue[0] == "test_directive"
    
    @pytest.mark.asyncio
    async def test_broadcast_event(self):
        orchestrator = PhantomOrchestrator()
        await orchestrator.spawn_swarm()
        
        test_event = {"type": "test_event", "data": "test_data"}
        await orchestrator.broadcast(test_event)
        
        # Event should be in the queue
        assert not orchestrator.event_bus.empty()
EOF

cat > tests/python/unit/test_sub_agents.py << 'EOF'
"""
Unit tests for Elite Agent sub-agents
"""

import pytest
from unittest.mock import Mock, AsyncMock

import sys
sys.path.insert(0, 'src')

from agent_swarm.phantom_orchestrator import (
    AgentRole,
    MnemonicCache,
    PhantomOrchestrator,
)
from agent_swarm.sub_agents import (
    ApexAgent,
    FortressAgent,
    CipherAgent,
    VelocityAgent,
)


@pytest.fixture
def mock_orchestrator():
    """Create a mock orchestrator for testing."""
    orchestrator = Mock(spec=PhantomOrchestrator)
    orchestrator.mnemonic = MnemonicCache(max_size=100)
    orchestrator.request_agent_reset = AsyncMock()
    return orchestrator


@pytest.fixture
def mock_mnemonic():
    """Create a mock mnemonic cache."""
    return MnemonicCache(max_size=100)


class TestApexAgent:
    """Tests for APEX strategic command agent."""
    
    def test_initialization(self, mock_orchestrator, mock_mnemonic):
        agent = ApexAgent(AgentRole.APEX, mock_orchestrator, mock_mnemonic)
        assert agent.role == AgentRole.APEX
        assert agent.state.active is True
    
    @pytest.mark.asyncio
    async def test_execute_mission(self, mock_orchestrator, mock_mnemonic):
        agent = ApexAgent(AgentRole.APEX, mock_orchestrator, mock_mnemonic)
        
        result = await agent.execute_mission(
            "coordinate_swarm",
            {"target": "all_agents"}
        )
        
        assert result["success"] is True
        assert "action" in result


class TestFortressAgent:
    """Tests for FORTRESS threat detection agent."""
    
    def test_initialization(self, mock_orchestrator, mock_mnemonic):
        agent = FortressAgent(AgentRole.FORTRESS, mock_orchestrator, mock_mnemonic)
        assert agent.role == AgentRole.FORTRESS
    
    @pytest.mark.asyncio
    async def test_execute_mission(self, mock_orchestrator, mock_mnemonic):
        agent = FortressAgent(AgentRole.FORTRESS, mock_orchestrator, mock_mnemonic)
        
        result = await agent.execute_mission(
            "scan_threats",
            {"scope": "network"}
        )
        
        assert result["success"] is True


class TestCipherAgent:
    """Tests for CIPHER cryptographic operations agent."""
    
    def test_initialization(self, mock_orchestrator, mock_mnemonic):
        agent = CipherAgent(AgentRole.CIPHER, mock_orchestrator, mock_mnemonic)
        assert agent.role == AgentRole.CIPHER


class TestVelocityAgent:
    """Tests for VELOCITY performance optimization agent."""
    
    def test_initialization(self, mock_orchestrator, mock_mnemonic):
        agent = VelocityAgent(AgentRole.VELOCITY, mock_orchestrator, mock_mnemonic)
        assert agent.role == AgentRole.VELOCITY
EOF

cat > tests/python/integration/test_agent_integration.py << 'EOF'
"""
Integration tests for agent swarm
"""

import pytest
import asyncio

import sys
sys.path.insert(0, 'src')


class TestAgentSwarmIntegration:
    """Integration tests for full agent swarm."""
    
    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_full_swarm_lifecycle(self):
        """Test spawning and shutting down full agent swarm."""
        from agent_swarm.phantom_orchestrator import PhantomOrchestrator
        
        orchestrator = PhantomOrchestrator()
        await orchestrator.spawn_swarm()
        
        assert len(orchestrator.agents) == 10
        
        orchestrator.shutdown()
        for agent in orchestrator.agents.values():
            assert agent._running is False
    
    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_inter_agent_communication(self):
        """Test agents communicating via event bus."""
        from agent_swarm.phantom_orchestrator import PhantomOrchestrator, AgentRole
        
        orchestrator = PhantomOrchestrator()
        await orchestrator.spawn_swarm()
        
        # Broadcast threat event
        await orchestrator.broadcast({
            "type": "threat_detected",
            "severity": "high",
            "source": "test"
        })
        
        # Event should be queued
        assert not orchestrator.event_bus.empty()
        
        orchestrator.shutdown()
EOF

echo "📝 Creating pytest configuration..."

cat > pytest.ini << 'EOF'
[pytest]
testpaths = tests/python
python_files = test_*.py
python_classes = Test*
python_functions = test_*
asyncio_mode = auto
addopts = -v --tb=short --strict-markers
markers =
    unit: Unit tests (fast, isolated)
    integration: Integration tests (may be slower)
    slow: Slow running tests
    security: Security-related tests

filterwarnings =
    ignore::DeprecationWarning
    ignore::PendingDeprecationWarning
EOF

echo "📝 Creating conftest.py for pytest fixtures..."

cat > tests/python/conftest.py << 'EOF'
"""
Pytest configuration and shared fixtures
"""

import pytest
import asyncio
import sys

# Ensure src is in path
sys.path.insert(0, 'src')


@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
def sample_threat_data():
    """Sample threat detection data."""
    return {
        "signature": bytes.fromhex("90909090"),
        "source_ip": "192.168.1.100",
        "timestamp": "2025-01-01T00:00:00Z"
    }


@pytest.fixture
def sample_peer_config():
    """Sample peer configuration."""
    return {
        "public_key": "test_key_base64",
        "endpoint": "127.0.0.1:51820",
        "allowed_ips": ["10.0.0.0/24"],
        "persistent_keepalive": 25
    }
EOF

echo ""
echo "✅ Test infrastructure scaffold created successfully!"
echo ""
echo "📊 Created structure:"
echo "   tests/"
echo "   ├── rust/"
echo "   │   ├── unit/"
echo "   │   │   ├── mod.rs"
echo "   │   │   ├── crypto_tests.rs"
echo "   │   │   ├── tunnel_tests.rs"
echo "   │   │   ├── threat_tests.rs"
echo "   │   │   └── metrics_tests.rs"
echo "   │   └── integration/"
echo "   │       ├── mod.rs"
echo "   │       ├── tunnel_integration.rs"
echo "   │       ├── peer_mesh_integration.rs"
echo "   │       └── threat_response_integration.rs"
echo "   ├── python/"
echo "   │   ├── unit/"
echo "   │   │   ├── test_orchestrator.py"
echo "   │   │   └── test_sub_agents.py"
echo "   │   ├── integration/"
echo "   │   │   └── test_agent_integration.py"
echo "   │   └── conftest.py"
echo "   ├── fixtures/"
echo "   │   ├── sample_configs.yaml"
echo "   │   └── threat_signatures.json"
echo "   └── mocks/"
echo "       ├── mod.rs"
echo "       └── mock_network.rs"
echo ""
echo "🔍 Validation commands:"
echo "   cargo test --no-run           # Verify Rust tests compile"
echo "   pytest --collect-only         # Verify Python tests discovered"
echo ""
