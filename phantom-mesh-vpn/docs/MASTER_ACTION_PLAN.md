# 🎯 PhantomMesh-VPN Master Action Plan

> **Document Version:** 1.0  
> **Created:** 2025-01-XX  
> **Architect:** @ARCHITECT (Elite Agent Collective)  
> **Execution Mode:** Maximum Autonomy & Automation  
> **Reference:** [EXECUTIVE_SUMMARY.md](./EXECUTIVE_SUMMARY.md)

---

## 📋 Table of Contents

1. [Execution Philosophy](#1-execution-philosophy)
2. [Phase Overview](#2-phase-overview)
3. [Phase 0: Foundation & Tooling](#3-phase-0-foundation--tooling)
4. [Phase 1: Core VPN Implementation](#4-phase-1-core-vpn-implementation)
5. [Phase 2: Agent Intelligence](#5-phase-2-agent-intelligence)
6. [Phase 3: Testing & Validation](#6-phase-3-testing--validation)
7. [Phase 4: Documentation & Polish](#7-phase-4-documentation--polish)
8. [Phase 5: Release Preparation](#8-phase-5-release-preparation)
9. [Automation Triggers](#9-automation-triggers)
10. [Validation Checkpoints](#10-validation-checkpoints)
11. [Risk Mitigation Protocols](#11-risk-mitigation-protocols)
12. [Appendix: Task Specifications](#12-appendix-task-specifications)

---

## 1. Execution Philosophy

### 1.1 Autonomy Principles

```
┌─────────────────────────────────────────────────────────────────┐
│                   AUTONOMOUS EXECUTION MODEL                    │
├─────────────────────────────────────────────────────────────────┤
│  PRINCIPLE 1: Self-Contained Tasks                              │
│    • Each task has complete specifications                      │
│    • No external dependencies unless explicitly stated          │
│    • Clear success criteria and validation steps                │
│                                                                 │
│  PRINCIPLE 2: Automated Validation                              │
│    • Every task includes automated verification commands        │
│    • CI/CD pipeline validates all changes                       │
│    • Continuous integration on every commit                     │
│                                                                 │
│  PRINCIPLE 3: Parallel Execution                                │
│    • Independent tasks can be executed simultaneously           │
│    • Dependencies are explicitly marked                         │
│    • Critical path is clearly identified                        │
│                                                                 │
│  PRINCIPLE 4: Rollback Safety                                   │
│    • Each change is atomic and reversible                       │
│    • Feature flags for gradual rollout                          │
│    • Git branching strategy enforced                            │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Agent Assignment Matrix

| Agent     | Primary Responsibility   | Tasks                  |
| --------- | ------------------------ | ---------------------- |
| @APEX     | Core Rust implementation | P0-001 through P0-008  |
| @CIPHER   | Cryptographic validation | P0-003, P0-005, P1-007 |
| @TENSOR   | AI/ML agent intelligence | P1-001 through P1-010  |
| @ECLIPSE  | Testing & verification   | P2-001 through P2-008  |
| @FLUX     | DevOps & automation      | P0-001, P3-003, P4-001 |
| @SCRIBE   | Documentation            | P3-001, P3-002, P4-002 |
| @FORTRESS | Security hardening       | P1-007, P2-007, P4-003 |

---

## 2. Phase Overview

```
╔═══════════════════════════════════════════════════════════════════════════╗
║                        EXECUTION TIMELINE                                  ║
╠═══════════════════════════════════════════════════════════════════════════╣
║                                                                            ║
║  PHASE 0: Foundation        ████░░░░░░░░░░░░░░░░░░░░░░░░░░  Week 1-2      ║
║  ─────────────────────────────────────────────────────────────────────    ║
║  • Dev environment setup                                                   ║
║  • Test infrastructure scaffold                                            ║
║  • CI/CD enhancements                                                      ║
║                                                                            ║
║  PHASE 1: Core VPN          ░░░░████████████░░░░░░░░░░░░░░  Week 2-5      ║
║  ─────────────────────────────────────────────────────────────────────    ║
║  • WireGuard-style tunnel implementation                                   ║
║  • Rust-Python FFI bridge                                                  ║
║  • Network socket management                                               ║
║  • Peer discovery & handshake                                              ║
║                                                                            ║
║  PHASE 2: Agent Intelligence ░░░░░░░░░░░░░░████████░░░░░░░  Week 5-7      ║
║  ─────────────────────────────────────────────────────────────────────    ║
║  • 10 sub-agent implementations                                            ║
║  • Semantic routing engine                                                 ║
║  • Threat response automation                                              ║
║                                                                            ║
║  PHASE 3: Testing           ░░░░░░░░░░░░░░░░░░░░░░████████  Week 7-9      ║
║  ─────────────────────────────────────────────────────────────────────    ║
║  • Unit test suite (90%+ coverage)                                         ║
║  • Integration tests                                                       ║
║  • Performance benchmarks                                                  ║
║                                                                            ║
║  PHASE 4: Documentation     ░░░░░░░░░░░░░░░░░░░░░░░░░░████  Week 9-10     ║
║  ─────────────────────────────────────────────────────────────────────    ║
║  • API documentation                                                       ║
║  • User guides                                                             ║
║  • Architecture refinement                                                 ║
║                                                                            ║
║  PHASE 5: Release           ░░░░░░░░░░░░░░░░░░░░░░░░░░░░██  Week 10-11    ║
║  ─────────────────────────────────────────────────────────────────────    ║
║  • Security audit                                                          ║
║  • Release packaging                                                       ║
║  • Beta deployment                                                         ║
║                                                                            ║
╚═══════════════════════════════════════════════════════════════════════════╝

LEGEND: ████ = Active Phase    ░░░░ = Inactive
```

### 2.1 Critical Path

```
P0-001 (Test Scaffold) ──┐
                         ├──▶ P0-003 (WireGuard Core) ──┐
P0-002 (DevContainer) ───┘                              │
                                                        ├──▶ P0-005 (Peer Handshake)
P0-004 (FFI Bridge) ────────────────────────────────────┘            │
                                                                     │
                                                                     ▼
P1-001 (NetworkIntel) ◀──────────────────────────────────────────────┘
        │
        ├──▶ P1-002 through P1-010 (Sub-Agents) ──┐
        │                                          │
        └──▶ P1-011 (Semantic Router) ─────────────┤
                                                   │
                                                   ▼
                                            P2-001 (Unit Tests)
                                                   │
                                                   ▼
                                            P4-001 (Release)
```

---

## 3. Phase 0: Foundation & Tooling

> **Duration:** Week 1-2  
> **Priority:** P0 (CRITICAL)  
> **Parallel Execution:** YES (P0-001 and P0-002 are independent)

### Task P0-001: Test Infrastructure Scaffold

**Assignee:** @FLUX + @ECLIPSE  
**Estimated Effort:** 4 hours  
**Dependencies:** None  
**Blocking:** All Phase 3 tasks

#### Specification

```yaml
task_id: P0-001
title: Create Test Infrastructure Scaffold
objective: Establish comprehensive test directory structure with Rust and Python test frameworks

deliverables:
  - tests/rust/unit/mod.rs
  - tests/rust/integration/mod.rs
  - tests/python/unit/__init__.py
  - tests/python/integration/__init__.py
  - tests/fixtures/sample_configs.yaml
  - tests/mocks/mock_network.rs
  - pytest.ini
  - .cargo/config.toml (test settings)

validation_commands:
  - cargo test --no-run # Verify Rust tests compile
  - pytest --collect-only # Verify Python tests discovered
  - cargo tarpaulin --out Html # Verify coverage tooling works
```

#### Autonomous Execution Script

```bash
#!/bin/bash
# P0-001: Test Infrastructure Scaffold
# Execute with: bash scripts/p0-001-test-scaffold.sh

set -euo pipefail

echo "📦 Creating test directory structure..."

# Create Rust test structure
mkdir -p tests/rust/{unit,integration}
mkdir -p tests/fixtures
mkdir -p tests/mocks

# Create Python test structure
mkdir -p tests/python/{unit,integration}
touch tests/python/__init__.py
touch tests/python/unit/__init__.py
touch tests/python/integration/__init__.py

# Create Rust test module files
cat > tests/rust/unit/mod.rs << 'EOF'
//! Unit tests for PhantomMesh-VPN
//!
//! Test organization:
//! - crypto_tests: ΣVault and cryptographic primitives
//! - tunnel_tests: Tunnel engine functionality
//! - threat_tests: Threat detection engine
//! - metrics_tests: Prometheus metrics

mod crypto_tests;
mod tunnel_tests;
mod threat_tests;
mod metrics_tests;
EOF

cat > tests/rust/integration/mod.rs << 'EOF'
//! Integration tests for PhantomMesh-VPN
//!
//! These tests verify end-to-end functionality:
//! - Full tunnel establishment
//! - Multi-peer communication
//! - Threat detection and response

mod tunnel_integration;
mod peer_mesh_integration;
mod threat_response_integration;
EOF

# Create test fixtures
cat > tests/fixtures/sample_configs.yaml << 'EOF'
# Test configuration fixtures
test_node:
  id: "test-node-001"
  tier: "relay"
  capabilities: ["route", "cache"]

test_peer:
  public_key: "test_key_base64_encoded"
  endpoint: "127.0.0.1:51820"
  allowed_ips: ["10.0.0.0/24"]
EOF

# Create mock network module
cat > tests/mocks/mock_network.rs << 'EOF'
//! Mock network layer for testing
//!
//! Provides simulated network operations without actual socket I/O

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockNetwork {
    packets: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl MockNetwork {
    pub fn new() -> Self {
        Self {
            packets: Arc::new(Mutex::new(HashMap::new())),
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
}
EOF

# Create pytest.ini
cat > pytest.ini << 'EOF'
[pytest]
testpaths = tests/python
python_files = test_*.py
python_classes = Test*
python_functions = test_*
addopts = -v --tb=short --strict-markers
markers =
    unit: Unit tests
    integration: Integration tests
    slow: Slow running tests
    security: Security-related tests
EOF

# Update Cargo.toml for test configuration
echo ""
echo "✅ Test scaffold created successfully!"
echo "📊 Run validation: cargo test --no-run && pytest --collect-only"
```

#### Success Criteria

- [ ] `cargo test --no-run` passes without errors
- [ ] `pytest --collect-only` discovers test modules
- [ ] Directory structure matches specification
- [ ] CI pipeline recognizes new test targets

---

### Task P0-002: Development Container Setup

**Assignee:** @FLUX  
**Estimated Effort:** 2 hours  
**Dependencies:** None  
**Blocking:** Developer onboarding

#### Specification

```yaml
task_id: P0-002
title: Create DevContainer Configuration
objective: Standardized development environment with all dependencies pre-installed

deliverables:
  - .devcontainer/devcontainer.json
  - .devcontainer/Dockerfile
  - .devcontainer/post-create.sh
  - .vscode/settings.json
  - .vscode/extensions.json
  - .vscode/launch.json

validation_commands:
  - docker build -f .devcontainer/Dockerfile -t phantommesh-dev .
  - code --list-extensions | grep -E "rust-analyzer|python"
```

#### Autonomous Execution Script

```bash
#!/bin/bash
# P0-002: DevContainer Setup
# Execute with: bash scripts/p0-002-devcontainer.sh

set -euo pipefail

echo "🐳 Creating DevContainer configuration..."

mkdir -p .devcontainer
mkdir -p .vscode

# Create DevContainer Dockerfile
cat > .devcontainer/Dockerfile << 'EOF'
# PhantomMesh-VPN Development Container
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Install Python 3.11+ and system dependencies
RUN apt-get update && apt-get install -y \
    python3.11 \
    python3.11-venv \
    python3-pip \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    wireguard-tools \
    iproute2 \
    iptables \
    tcpdump \
    && rm -rf /var/lib/apt/lists/*

# Install Rust components
RUN rustup component add rustfmt clippy llvm-tools-preview
RUN cargo install cargo-tarpaulin cargo-audit cargo-watch

# Install Python dependencies
COPY pyproject.toml /tmp/
RUN pip3 install --break-system-packages -e /tmp/ || true
RUN pip3 install --break-system-packages pytest pytest-cov pytest-asyncio black mypy

# Set up environment
ENV RUST_BACKTRACE=1
ENV PYTHONDONTWRITEBYTECODE=1

WORKDIR /workspace
EOF

# Create devcontainer.json
cat > .devcontainer/devcontainer.json << 'EOF'
{
    "name": "PhantomMesh-VPN Dev",
    "build": {
        "dockerfile": "Dockerfile",
        "context": ".."
    },
    "features": {
        "ghcr.io/devcontainers/features/docker-in-docker:2": {},
        "ghcr.io/devcontainers/features/kubectl-helm-minikube:1": {}
    },
    "customizations": {
        "vscode": {
            "extensions": [
                "rust-lang.rust-analyzer",
                "ms-python.python",
                "ms-python.vscode-pylance",
                "tamasfe.even-better-toml",
                "serayuzgur.crates",
                "vadimcn.vscode-lldb",
                "ms-azuretools.vscode-docker",
                "redhat.vscode-yaml",
                "GitHub.copilot"
            ],
            "settings": {
                "rust-analyzer.checkOnSave.command": "clippy",
                "python.defaultInterpreterPath": "/usr/bin/python3.11",
                "editor.formatOnSave": true
            }
        }
    },
    "postCreateCommand": "bash .devcontainer/post-create.sh",
    "remoteUser": "vscode",
    "mounts": [
        "source=${localWorkspaceFolder}/.cargo-cache,target=/usr/local/cargo/registry,type=bind,consistency=cached"
    ],
    "forwardPorts": [51820, 8080, 9090, 3000],
    "portsAttributes": {
        "51820": {"label": "WireGuard", "protocol": "udp"},
        "8080": {"label": "API Gateway"},
        "9090": {"label": "Prometheus"},
        "3000": {"label": "Grafana"}
    }
}
EOF

# Create post-create script
cat > .devcontainer/post-create.sh << 'EOF'
#!/bin/bash
set -e

echo "🚀 Setting up PhantomMesh-VPN development environment..."

# Install Python dependencies
pip3 install --break-system-packages -e ".[dev]" || pip3 install -e ".[dev]" || true

# Build Rust project
cargo build

# Initialize git hooks
if [ -d .git ]; then
    echo "Setting up git hooks..."
    cat > .git/hooks/pre-commit << 'HOOK'
#!/bin/bash
cargo fmt --check
cargo clippy -- -D warnings
python -m black --check src/agent_swarm/
HOOK
    chmod +x .git/hooks/pre-commit
fi

echo "✅ Development environment ready!"
echo "📖 Run 'cargo test' to verify setup"
EOF
chmod +x .devcontainer/post-create.sh

# Create VS Code settings
cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.procMacro.enable": true,
    "python.defaultInterpreterPath": "${workspaceFolder}/.venv/bin/python",
    "python.formatting.provider": "black",
    "python.linting.enabled": true,
    "python.linting.mypyEnabled": true,
    "editor.formatOnSave": true,
    "editor.rulers": [100],
    "files.exclude": {
        "**/target": true,
        "**/__pycache__": true,
        "**/.pytest_cache": true
    },
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    },
    "[python]": {
        "editor.defaultFormatter": "ms-python.black-formatter"
    }
}
EOF

# Create VS Code extensions list
cat > .vscode/extensions.json << 'EOF'
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "ms-python.python",
        "ms-python.vscode-pylance",
        "ms-python.black-formatter",
        "tamasfe.even-better-toml",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb",
        "ms-azuretools.vscode-docker",
        "redhat.vscode-yaml",
        "GitHub.copilot",
        "GitHub.copilot-chat"
    ]
}
EOF

# Create VS Code launch configuration
cat > .vscode/launch.json << 'EOF'
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug PhantomMesh Node",
            "cargo": {
                "args": ["build", "--bin=phantom-node"],
                "filter": {
                    "name": "phantom-node",
                    "kind": "bin"
                }
            },
            "args": ["--config", "configs/mesh_configs.yaml"],
            "cwd": "${workspaceFolder}/phantom-mesh-vpn",
            "env": {
                "RUST_BACKTRACE": "1",
                "RUST_LOG": "debug"
            }
        },
        {
            "name": "Debug Python Orchestrator",
            "type": "debugpy",
            "request": "launch",
            "module": "agent_swarm.phantom_orchestrator",
            "cwd": "${workspaceFolder}/phantom-mesh-vpn/src",
            "env": {
                "PYTHONPATH": "${workspaceFolder}/phantom-mesh-vpn/src"
            }
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Unit Tests",
            "cargo": {
                "args": ["test", "--no-run"],
                "filter": {
                    "kind": "lib"
                }
            },
            "cwd": "${workspaceFolder}/phantom-mesh-vpn"
        }
    ]
}
EOF

echo "✅ DevContainer configuration created!"
echo "📖 Open in VS Code and select 'Reopen in Container'"
```

#### Success Criteria

- [ ] DevContainer builds successfully
- [ ] All VS Code extensions install automatically
- [ ] `cargo build` succeeds inside container
- [ ] Python environment configured correctly

---

### Task P0-003: WireGuard-Style Tunnel Core

**Assignee:** @APEX + @CIPHER  
**Estimated Effort:** 16 hours  
**Dependencies:** None (can start immediately)  
**Blocking:** P0-005, P1-\* tasks

#### Specification

```yaml
task_id: P0-003
title: Implement WireGuard-Style Tunnel Core
objective: Complete tunnel_engine.rs with manual WireGuard-compatible implementation

deliverables:
  - src/vpn_core/tunnel_engine.rs (complete implementation)
  - src/vpn_core/handshake.rs (new file)
  - src/vpn_core/noise_protocol.rs (new file)
  - src/vpn_core/packet.rs (new file)

technical_requirements:
  - Use x25519-dalek for key exchange (already in Cargo.toml)
  - Use chacha20poly1305 for AEAD encryption
  - Implement Noise_IK handshake pattern
  - Support keep-alive and roaming
  - Handle MTU discovery

validation_commands:
  - cargo test tunnel_engine::tests
  - cargo clippy -- -D warnings
  - cargo doc --no-deps
```

#### Implementation Outline

```rust
// src/vpn_core/noise_protocol.rs
//! Noise Protocol Framework implementation for WireGuard-compatible tunnels
//!
//! Implements Noise_IK pattern:
//! <- s
//! ...
//! -> e, es, s, ss
//! <- e, ee, se

use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, aead::{Aead, NewAead}};
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};
use blake2::{Blake2s256, Digest};

/// Noise protocol handshake state
pub struct NoiseHandshake {
    local_static: StaticSecret,
    local_ephemeral: Option<EphemeralSecret>,
    remote_static: Option<PublicKey>,
    remote_ephemeral: Option<PublicKey>,
    chaining_key: [u8; 32],
    hash: [u8; 32],
    symmetric_state: SymmetricState,
}

impl NoiseHandshake {
    /// Initialize handshake as initiator
    pub fn initiate(local_static: StaticSecret, remote_static: PublicKey) -> Self {
        // Implementation follows Noise_IK pattern
        todo!("Implement Noise_IK initiator")
    }

    /// Initialize handshake as responder
    pub fn respond(local_static: StaticSecret) -> Self {
        todo!("Implement Noise_IK responder")
    }

    /// Process handshake message and potentially derive transport keys
    pub fn process_message(&mut self, msg: &[u8]) -> Result<Option<TransportKeys>, HandshakeError> {
        todo!("Implement message processing")
    }

    /// Generate next handshake message
    pub fn generate_message(&mut self) -> Result<Vec<u8>, HandshakeError> {
        todo!("Implement message generation")
    }
}

/// Transport keys derived from completed handshake
pub struct TransportKeys {
    pub send_key: [u8; 32],
    pub recv_key: [u8; 32],
    pub send_nonce: u64,
    pub recv_nonce: u64,
}

// src/vpn_core/packet.rs
//! WireGuard-compatible packet format

/// Message types as per WireGuard specification
#[repr(u8)]
pub enum MessageType {
    HandshakeInit = 1,
    HandshakeResponse = 2,
    CookieReply = 3,
    TransportData = 4,
}

/// Transport data packet
#[derive(Debug)]
pub struct TransportPacket {
    pub msg_type: u8,           // Always 4
    pub reserved: [u8; 3],       // Zero
    pub receiver_index: u32,     // Peer's sender index
    pub counter: u64,            // Nonce counter
    pub encrypted_payload: Vec<u8>,
}

impl TransportPacket {
    pub fn encrypt(
        payload: &[u8],
        key: &[u8; 32],
        counter: u64,
        receiver_index: u32,
    ) -> Result<Self, PacketError> {
        todo!("Implement packet encryption")
    }

    pub fn decrypt(&self, key: &[u8; 32]) -> Result<Vec<u8>, PacketError> {
        todo!("Implement packet decryption")
    }
}
```

#### Success Criteria

- [ ] Noise_IK handshake completes successfully in tests
- [ ] Encrypted tunnel can transmit data bidirectionally
- [ ] Performance: >1 Gbps on modern hardware
- [ ] Compatible with WireGuard packet format

---

### Task P0-004: Rust-Python FFI Bridge

**Assignee:** @APEX  
**Estimated Effort:** 8 hours  
**Dependencies:** P0-003 (partial - needs basic tunnel types)  
**Blocking:** P1-\* (Python agents need Rust access)

#### Specification

```yaml
task_id: P0-004
title: Implement Rust-Python FFI Bridge
objective: Enable Python agent swarm to call Rust VPN core functions

deliverables:
  - src/ffi/mod.rs (new module)
  - src/ffi/python_bindings.rs
  - src/agent_swarm/rust_bridge.py
  - Cargo.toml (add pyo3 dependency)
  - pyproject.toml (update with native extension)

technical_requirements:
  - Use PyO3 for safe Rust-Python interop
  - Expose key functions: tunnel_status, encrypt_packet, threat_check
  - Async support via pyo3-asyncio
  - Proper error propagation

validation_commands:
  - cargo build --features python-bindings
  - python -c "import phantom_mesh_core; print(phantom_mesh_core.version())"
  - pytest tests/python/test_ffi.py
```

#### Implementation Outline

```rust
// src/ffi/python_bindings.rs
use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;

/// Python-accessible wrapper for tunnel status
#[pyclass]
pub struct PyTunnelStatus {
    #[pyo3(get)]
    pub connected: bool,
    #[pyo3(get)]
    pub peer_count: usize,
    #[pyo3(get)]
    pub bytes_sent: u64,
    #[pyo3(get)]
    pub bytes_received: u64,
}

/// Python-accessible threat detection
#[pyfunction]
fn check_threat(packet_data: &[u8]) -> PyResult<bool> {
    use crate::security_layer::threat_engine::ThreatEngine;

    let engine = ThreatEngine::global()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    Ok(engine.is_threat(packet_data))
}

/// Python-accessible encryption
#[pyfunction]
fn encrypt_for_peer(data: &[u8], peer_id: &str) -> PyResult<Vec<u8>> {
    use crate::vpn_core::tunnel_engine::TunnelEngine;

    TunnelEngine::encrypt(data, peer_id)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Module initialization
#[pymodule]
fn phantom_mesh_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_threat, m)?)?;
    m.add_function(wrap_pyfunction!(encrypt_for_peer, m)?)?;
    m.add_class::<PyTunnelStatus>()?;

    // Version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
```

```python
# src/agent_swarm/rust_bridge.py
"""
Rust-Python FFI Bridge for PhantomMesh-VPN

Provides Python access to high-performance Rust core functions.
"""

from typing import Optional
import phantom_mesh_core  # Native Rust module

class RustBridge:
    """Bridge to Rust VPN core functionality."""

    @staticmethod
    def check_threat(packet_data: bytes) -> bool:
        """Check if packet matches known threat signatures."""
        return phantom_mesh_core.check_threat(packet_data)

    @staticmethod
    def encrypt_packet(data: bytes, peer_id: str) -> bytes:
        """Encrypt data for specific peer using tunnel keys."""
        return phantom_mesh_core.encrypt_for_peer(data, peer_id)

    @staticmethod
    def get_tunnel_status() -> dict:
        """Get current tunnel status."""
        status = phantom_mesh_core.get_tunnel_status()
        return {
            "connected": status.connected,
            "peer_count": status.peer_count,
            "bytes_sent": status.bytes_sent,
            "bytes_received": status.bytes_received,
        }

    @staticmethod
    def version() -> str:
        """Get Rust core version."""
        return phantom_mesh_core.__version__
```

#### Success Criteria

- [ ] Python can import `phantom_mesh_core` module
- [ ] All exposed functions work correctly
- [ ] No memory leaks (verified with valgrind)
- [ ] Async operations don't block Python event loop

---

### Task P0-005: Peer Handshake & Session Keys

**Assignee:** @APEX + @CIPHER  
**Estimated Effort:** 12 hours  
**Dependencies:** P0-003  
**Blocking:** Full mesh connectivity

#### Specification

```yaml
task_id: P0-005
title: Implement Peer Handshake with Session Keys
objective: Complete peer discovery, authentication, and session key derivation

deliverables:
  - src/vpn_core/peer_manager.rs (complete implementation)
  - src/vpn_core/session.rs (new file)
  - src/vpn_core/discovery.rs (enhanced)

technical_requirements:
  - Integrate with Noise handshake from P0-003
  - Post-quantum key encapsulation (Kyber) for long-term keys
  - Session key rotation every 2 minutes
  - Replay attack prevention with nonce tracking

validation_commands:
  - cargo test peer_manager::tests
  - cargo test session::tests
  - cargo bench handshake_bench
```

#### Success Criteria

- [ ] Two peers can complete handshake successfully
- [ ] Session keys rotate without connection drop
- [ ] Replay attacks are detected and blocked
- [ ] Handshake completes in <50ms on LAN

---

## 4. Phase 1: Core VPN Implementation

> **Duration:** Week 2-5  
> **Priority:** P0-P1  
> **Dependencies:** Phase 0 tasks

### Task P0-006: Network Socket Management

**Assignee:** @APEX  
**Estimated Effort:** 8 hours  
**Dependencies:** P0-003, P0-005

```yaml
task_id: P0-006
title: Implement Network Socket Management
objective: UDP socket handling for WireGuard-compatible communication

deliverables:
  - src/vpn_core/socket_manager.rs
  - src/vpn_core/endpoint.rs

technical_requirements:
  - Async UDP with tokio
  - Support for multiple interfaces
  - NAT traversal (STUN/TURN client)
  - Connection migration (roaming support)
```

### Task P0-007: TUN/TAP Interface Integration

**Assignee:** @APEX + @CORE  
**Estimated Effort:** 12 hours  
**Dependencies:** P0-006

```yaml
task_id: P0-007
title: TUN/TAP Virtual Interface
objective: Create and manage virtual network interface for tunneled traffic

deliverables:
  - src/vpn_core/tun_device.rs
  - Platform-specific implementations (Linux, macOS, Windows)

technical_requirements:
  - Cross-platform TUN device creation
  - Efficient packet routing
  - MTU handling
  - IPv4 and IPv6 support
```

### Task P0-008: API Gateway Completion

**Assignee:** @APEX + @SYNAPSE  
**Estimated Effort:** 6 hours  
**Dependencies:** P0-003, P0-005

```yaml
task_id: P0-008
title: Complete API Gateway Endpoints
objective: Finish REST API for tunnel management

deliverables:
  - src/vpn_core/api_gateway.rs (enhanced)
  - OpenAPI specification

new_endpoints:
  - POST /api/v1/tunnel/connect
  - DELETE /api/v1/tunnel/disconnect
  - GET /api/v1/tunnel/status
  - POST /api/v1/peers/add
  - DELETE /api/v1/peers/{peer_id}
  - GET /api/v1/peers
  - POST /api/v1/config/reload
```

---

## 5. Phase 2: Agent Intelligence

> **Duration:** Week 5-7  
> **Priority:** P1  
> **Dependencies:** P0-004 (FFI Bridge)

### Tasks P1-001 through P1-010: Sub-Agent Implementations

Each sub-agent follows this template:

```yaml
task_template:
  assignee: "@TENSOR"
  estimated_effort: "4 hours each"
  dependencies: ["P0-004"]

  implementation_pattern:
    - Complete execute_mission() method
    - Add specialized decision logic
    - Integrate with Rust core via FFI
    - Include unit tests
    - Add Prometheus metrics
```

#### P1-001: NetworkIntelAgent

```python
# src/agent_swarm/sub_agents/network_intel.py
"""
NetworkIntelAgent: Real-time network analysis and intelligence gathering.

Responsibilities:
- Traffic pattern analysis
- Peer behavior profiling
- Bandwidth utilization monitoring
- Anomaly detection
"""

from dataclasses import dataclass
from typing import Dict, List, Optional
import asyncio

from .base_agent import SubAgent
from ..rust_bridge import RustBridge

@dataclass
class NetworkInsight:
    """Analyzed network intelligence."""
    peer_id: str
    traffic_pattern: str  # "normal", "suspicious", "malicious"
    bandwidth_mbps: float
    packet_loss_percent: float
    latency_ms: float
    anomaly_score: float  # 0.0 to 1.0

class NetworkIntelAgent(SubAgent):
    """Real-time network intelligence agent."""

    def __init__(self):
        super().__init__(
            name="NetworkIntel",
            role="network_analysis",
            description="Real-time network pattern analysis"
        )
        self._peer_profiles: Dict[str, NetworkInsight] = {}
        self._analysis_interval = 5.0  # seconds

    async def execute_mission(self, context: Dict) -> Dict:
        """
        Execute network intelligence gathering mission.

        Args:
            context: Mission context with target parameters

        Returns:
            Analysis results with insights and recommendations
        """
        target_peers = context.get("peers", [])
        analysis_depth = context.get("depth", "standard")

        insights = []
        for peer_id in target_peers:
            insight = await self._analyze_peer(peer_id, analysis_depth)
            insights.append(insight)
            self._peer_profiles[peer_id] = insight

        # Aggregate analysis
        threat_level = self._calculate_threat_level(insights)
        recommendations = self._generate_recommendations(insights)

        return {
            "status": "complete",
            "insights": [vars(i) for i in insights],
            "threat_level": threat_level,
            "recommendations": recommendations,
            "analyzed_peers": len(insights),
        }

    async def _analyze_peer(self, peer_id: str, depth: str) -> NetworkInsight:
        """Analyze individual peer behavior."""
        # Get traffic stats from Rust core
        stats = RustBridge.get_peer_stats(peer_id)

        # Pattern analysis
        pattern = self._classify_traffic_pattern(stats)

        # Anomaly detection
        anomaly_score = self._detect_anomalies(stats, peer_id)

        return NetworkInsight(
            peer_id=peer_id,
            traffic_pattern=pattern,
            bandwidth_mbps=stats.get("bandwidth_mbps", 0.0),
            packet_loss_percent=stats.get("packet_loss", 0.0),
            latency_ms=stats.get("latency_ms", 0.0),
            anomaly_score=anomaly_score,
        )

    def _classify_traffic_pattern(self, stats: Dict) -> str:
        """Classify traffic pattern using ML model or heuristics."""
        # Placeholder for ML-based classification
        bandwidth = stats.get("bandwidth_mbps", 0)
        packet_size_variance = stats.get("packet_size_variance", 0)

        if packet_size_variance > 1000 and bandwidth > 100:
            return "suspicious"
        return "normal"

    def _detect_anomalies(self, stats: Dict, peer_id: str) -> float:
        """Calculate anomaly score based on historical baseline."""
        if peer_id not in self._peer_profiles:
            return 0.0  # No baseline yet

        baseline = self._peer_profiles[peer_id]
        # Compare current stats to baseline
        # Simplified Z-score calculation
        return min(1.0, abs(stats.get("bandwidth_mbps", 0) - baseline.bandwidth_mbps) / 100)

    def _calculate_threat_level(self, insights: List[NetworkInsight]) -> str:
        """Calculate overall threat level from insights."""
        if not insights:
            return "unknown"

        avg_anomaly = sum(i.anomaly_score for i in insights) / len(insights)
        malicious_count = sum(1 for i in insights if i.traffic_pattern == "malicious")

        if malicious_count > 0 or avg_anomaly > 0.8:
            return "critical"
        elif avg_anomaly > 0.5:
            return "high"
        elif avg_anomaly > 0.2:
            return "medium"
        return "low"

    def _generate_recommendations(self, insights: List[NetworkInsight]) -> List[str]:
        """Generate actionable recommendations."""
        recommendations = []

        for insight in insights:
            if insight.traffic_pattern == "malicious":
                recommendations.append(f"BLOCK peer {insight.peer_id} immediately")
            elif insight.traffic_pattern == "suspicious":
                recommendations.append(f"MONITOR peer {insight.peer_id} closely")
            if insight.latency_ms > 100:
                recommendations.append(f"OPTIMIZE routing to peer {insight.peer_id}")

        return recommendations
```

#### P1-002 through P1-010: Remaining Sub-Agents

| Task ID | Agent             | Primary Function                      |
| ------- | ----------------- | ------------------------------------- |
| P1-002  | ThreatNeutralizer | Active threat response and mitigation |
| P1-003  | PeerValidator     | Peer authentication and trust scoring |
| P1-004  | RoutingOptimizer  | Dynamic route optimization            |
| P1-005  | CryptoAuditor     | Cryptographic health monitoring       |
| P1-006  | HealthMonitor     | System health and resource monitoring |
| P1-007  | ConfigHarmonizer  | Configuration consistency management  |
| P1-008  | MetricsCollector  | Prometheus metrics aggregation        |
| P1-009  | AlertDispatcher   | Alert routing and notification        |
| P1-010  | PolicyEnforcer    | Security policy enforcement           |

### Task P1-011: Semantic Routing Engine

**Assignee:** @TENSOR + @LINGUA  
**Estimated Effort:** 12 hours  
**Dependencies:** P1-001 through P1-010

```yaml
task_id: P1-011
title: Implement Semantic Routing Engine
objective: Route tasks to appropriate sub-agents based on semantic analysis

deliverables:
  - src/agent_swarm/semantic_router.py
  - src/agent_swarm/intent_classifier.py
  - configs/routing_rules.yaml
```

---

## 6. Phase 3: Testing & Validation

> **Duration:** Week 7-9  
> **Priority:** P2  
> **Dependencies:** Phases 0-1

### Task P2-001: Rust Unit Test Suite

**Assignee:** @ECLIPSE  
**Estimated Effort:** 16 hours  
**Dependencies:** All P0 tasks

```yaml
task_id: P2-001
title: Comprehensive Rust Unit Tests
objective: Achieve 90%+ code coverage on Rust codebase

deliverables:
  - tests/rust/unit/crypto_tests.rs
  - tests/rust/unit/tunnel_tests.rs
  - tests/rust/unit/threat_tests.rs
  - tests/rust/unit/metrics_tests.rs

coverage_targets:
  - sigma_vault.rs: 95%
  - threat_engine.rs: 90%
  - tunnel_engine.rs: 90%
  - noise_protocol.rs: 95%

validation_commands:
  - cargo tarpaulin --out Html --output-dir coverage/
  - cargo test --all-features
```

### Task P2-002: Python Unit Test Suite

**Assignee:** @ECLIPSE  
**Estimated Effort:** 8 hours  
**Dependencies:** All P1 tasks

```yaml
task_id: P2-002
title: Python Agent Unit Tests
objective: Full test coverage for Python agent swarm

deliverables:
  - tests/python/unit/test_orchestrator.py
  - tests/python/unit/test_sub_agents.py
  - tests/python/unit/test_rust_bridge.py

coverage_target: 85%

validation_commands:
  - pytest --cov=src/agent_swarm --cov-report=html
```

### Task P2-003: Integration Test Suite

**Assignee:** @ECLIPSE  
**Estimated Effort:** 12 hours  
**Dependencies:** P2-001, P2-002

```yaml
task_id: P2-003
title: End-to-End Integration Tests
objective: Verify complete system functionality

test_scenarios:
  - Two-node tunnel establishment
  - Multi-peer mesh formation
  - Threat detection and response
  - Configuration hot-reload
  - Failover and recovery

deliverables:
  - tests/integration/test_tunnel_e2e.py
  - tests/integration/test_mesh_formation.py
  - tests/integration/test_threat_response.py
```

### Task P2-004: Performance Benchmarks

**Assignee:** @VELOCITY  
**Estimated Effort:** 8 hours  
**Dependencies:** P0-003, P0-005

```yaml
task_id: P2-004
title: Performance Benchmark Suite
objective: Establish performance baselines and regression detection

benchmarks:
  - Handshake latency (target: <50ms)
  - Throughput (target: >1 Gbps)
  - Encryption overhead (target: <5%)
  - Memory usage under load
  - CPU utilization

deliverables:
  - benches/handshake_bench.rs
  - benches/throughput_bench.rs
  - benches/crypto_bench.rs
```

### Task P2-005: Fuzz Testing

**Assignee:** @FORTRESS  
**Estimated Effort:** 8 hours  
**Dependencies:** P0-003

```yaml
task_id: P2-005
title: Fuzz Testing for Security-Critical Paths
objective: Discover edge cases and potential vulnerabilities

targets:
  - Packet parsing
  - Handshake message processing
  - Configuration parsing
  - API input validation

tools:
  - cargo-fuzz
  - AFL++
  - libFuzzer
```

---

## 7. Phase 4: Documentation & Polish

> **Duration:** Week 9-10  
> **Priority:** P3  
> **Dependencies:** Phases 0-2

### Task P3-001: API Documentation

**Assignee:** @SCRIBE  
**Estimated Effort:** 8 hours

```yaml
task_id: P3-001
title: Complete API Documentation
objective: OpenAPI specification and interactive documentation

deliverables:
  - docs/api_docs/openapi.yaml
  - docs/api_docs/index.html (ReDoc/Swagger)
  - README updates with API quickstart
```

### Task P3-002: User Guide

**Assignee:** @SCRIBE  
**Estimated Effort:** 12 hours

```yaml
task_id: P3-002
title: Comprehensive User Guide
objective: End-user documentation for deployment and operation

deliverables:
  - docs/user_guide/installation.md
  - docs/user_guide/configuration.md
  - docs/user_guide/troubleshooting.md
  - docs/user_guide/best_practices.md
```

### Task P3-003: Helm Chart Completion

**Assignee:** @FLUX  
**Estimated Effort:** 6 hours

```yaml
task_id: P3-003
title: Complete Helm Chart Templates
objective: Production-ready Kubernetes deployment

deliverables:
  - k8s/helm/phantommesh/templates/*.yaml (all resources)
  - k8s/helm/phantommesh/values.yaml (comprehensive)
  - k8s/helm/phantommesh/README.md
```

---

## 8. Phase 5: Release Preparation

> **Duration:** Week 10-11  
> **Priority:** P4  
> **Dependencies:** All previous phases

### Task P4-001: Security Audit

**Assignee:** @FORTRESS + @CIPHER  
**Estimated Effort:** 16 hours

```yaml
task_id: P4-001
title: Pre-Release Security Audit
objective: Comprehensive security review before release

audit_areas:
  - Cryptographic implementation review
  - Memory safety verification
  - Input validation audit
  - Dependency vulnerability scan
  - Configuration security
```

### Task P4-002: Release Packaging

**Assignee:** @FLUX  
**Estimated Effort:** 8 hours

```yaml
task_id: P4-002
title: Release Packaging and Distribution
objective: Create distributable packages

deliverables:
  - Binary releases (Linux, macOS, Windows)
  - Docker images (pushed to registry)
  - Helm chart package
  - Release notes
```

### Task P4-003: Beta Deployment

**Assignee:** @FLUX + @SENTRY  
**Estimated Effort:** 8 hours

```yaml
task_id: P4-003
title: Beta Environment Deployment
objective: Deploy to staging/beta environment for validation

activities:
  - Deploy to Kubernetes beta cluster
  - Configure monitoring and alerting
  - Establish feedback collection
  - Document deployment runbook
```

---

## 9. Automation Triggers

### 9.1 CI/CD Automation

```yaml
# .github/workflows/ci.yml enhancements
automation_triggers:
  on_push_to_main:
    - cargo fmt --check
    - cargo clippy -- -D warnings
    - cargo test --all-features
    - cargo tarpaulin --out Xml
    - pytest --cov=src/agent_swarm
    - docker build & push

  on_pull_request:
    - All push triggers
    - cargo audit
    - cargo bench (compare to baseline)
    - Security scan (Trivy, Snyk)

  on_release_tag:
    - Build release binaries
    - Push to container registry
    - Publish Helm chart
    - Generate changelog

  nightly:
    - Full integration tests
    - Fuzz testing (1 hour)
    - Dependency update check
    - Security scan
```

### 9.2 Automated Quality Gates

```yaml
quality_gates:
  code_coverage:
    rust: 90%
    python: 85%
    action: block_merge

  security_scan:
    critical_vulnerabilities: 0
    high_vulnerabilities: 0
    action: block_merge

  performance:
    handshake_latency_p99: 50ms
    throughput_min: 1Gbps
    action: warn_on_regression

  linting:
    rust_clippy: no_warnings
    python_black: formatted
    python_mypy: no_errors
    action: block_merge
```

---

## 10. Validation Checkpoints

Reference: Master Prompt [REF:EX-603]

### Checkpoint 1: Foundation Complete (End of Phase 0)

```yaml
checkpoint_1:
  name: "Foundation Validation"
  criteria:
    - tests/ directory structure exists
    - DevContainer builds and runs
    - Basic tunnel encryption works
    - FFI bridge passes hello-world test
    - Peer handshake succeeds in test

  validation_script: |
    #!/bin/bash
    set -e
    cargo test --lib
    pytest --collect-only
    python -c "import phantom_mesh_core"
    echo "✅ Checkpoint 1 PASSED"
```

### Checkpoint 2: Core VPN Functional (End of Phase 1)

```yaml
checkpoint_2:
  name: "Core VPN Validation"
  criteria:
    - Two nodes can establish tunnel
    - Encrypted traffic flows bidirectionally
    - Session key rotation works
    - At least 3 sub-agents functional

  validation_script: |
    #!/bin/bash
    set -e
    cargo test --test integration
    pytest tests/integration/
    ./scripts/two-node-test.sh
    echo "✅ Checkpoint 2 PASSED"
```

### Checkpoint 3: Full Feature Set (End of Phase 2)

```yaml
checkpoint_3:
  name: "Feature Complete Validation"
  criteria:
    - All 10 sub-agents implemented
    - 90%+ test coverage
    - Performance benchmarks met
    - No critical/high vulnerabilities

  validation_script: |
    #!/bin/bash
    set -e
    cargo tarpaulin --fail-under 90
    pytest --cov-fail-under=85
    cargo bench
    cargo audit
    echo "✅ Checkpoint 3 PASSED"
```

### Checkpoint 4: Release Ready (End of Phase 4)

```yaml
checkpoint_4:
  name: "Release Validation"
  criteria:
    - Documentation complete
    - Security audit passed
    - Beta deployment successful
    - Release packages built

  validation_script: |
    #!/bin/bash
    set -e
    helm lint k8s/helm/phantommesh
    docker run phantommesh:latest --version
    ./scripts/smoke-test.sh
    echo "✅ Checkpoint 4 PASSED - READY FOR RELEASE"
```

---

## 11. Risk Mitigation Protocols

### 11.1 Technical Risks

| Risk                             | Probability | Impact | Mitigation                                                            |
| -------------------------------- | ----------- | ------ | --------------------------------------------------------------------- |
| WireGuard integration complexity | Medium      | High   | Detailed Noise protocol implementation; fallback to simpler handshake |
| Cross-platform TUN issues        | High        | Medium | Abstract platform layer; focus Linux first                            |
| FFI memory safety                | Medium      | High   | Extensive fuzzing; memory sanitizers in CI                            |
| Performance regression           | Low         | Medium | Automated benchmarks with regression detection                        |

### 11.2 Rollback Procedures

```yaml
rollback_procedures:
  code_rollback:
    trigger: "Test failure on main branch"
    action: "git revert to last passing commit"
    automation: "GitHub Actions auto-revert"

  deployment_rollback:
    trigger: "Production health check failure"
    action: "Helm rollback to previous release"
    automation: "ArgoCD automatic rollback"

  configuration_rollback:
    trigger: "Config validation failure"
    action: "Restore from ConfigMap backup"
    automation: "Kubernetes rollback"
```

---

## 12. Appendix: Task Specifications

### Quick Reference: All Tasks

| ID     | Title                         | Effort | Priority | Dependencies   |
| ------ | ----------------------------- | ------ | -------- | -------------- |
| P0-001 | Test Infrastructure Scaffold  | 4h     | P0       | None           |
| P0-002 | DevContainer Setup            | 2h     | P0       | None           |
| P0-003 | WireGuard-Style Tunnel Core   | 16h    | P0       | None           |
| P0-004 | Rust-Python FFI Bridge        | 8h     | P0       | P0-003         |
| P0-005 | Peer Handshake & Session Keys | 12h    | P0       | P0-003         |
| P0-006 | Network Socket Management     | 8h     | P0       | P0-003, P0-005 |
| P0-007 | TUN/TAP Interface             | 12h    | P0       | P0-006         |
| P0-008 | API Gateway Completion        | 6h     | P0       | P0-003, P0-005 |
| P1-001 | NetworkIntelAgent             | 4h     | P1       | P0-004         |
| P1-002 | ThreatNeutralizer             | 4h     | P1       | P0-004         |
| P1-003 | PeerValidator                 | 4h     | P1       | P0-004         |
| P1-004 | RoutingOptimizer              | 4h     | P1       | P0-004         |
| P1-005 | CryptoAuditor                 | 4h     | P1       | P0-004         |
| P1-006 | HealthMonitor                 | 4h     | P1       | P0-004         |
| P1-007 | ConfigHarmonizer              | 4h     | P1       | P0-004         |
| P1-008 | MetricsCollector              | 4h     | P1       | P0-004         |
| P1-009 | AlertDispatcher               | 4h     | P1       | P0-004         |
| P1-010 | PolicyEnforcer                | 4h     | P1       | P0-004         |
| P1-011 | Semantic Routing Engine       | 12h    | P1       | P1-001:010     |
| P2-001 | Rust Unit Test Suite          | 16h    | P2       | All P0         |
| P2-002 | Python Unit Test Suite        | 8h     | P2       | All P1         |
| P2-003 | Integration Test Suite        | 12h    | P2       | P2-001, P2-002 |
| P2-004 | Performance Benchmarks        | 8h     | P2       | P0-003, P0-005 |
| P2-005 | Fuzz Testing                  | 8h     | P2       | P0-003         |
| P3-001 | API Documentation             | 8h     | P3       | P0-008         |
| P3-002 | User Guide                    | 12h    | P3       | All            |
| P3-003 | Helm Chart Completion         | 6h     | P3       | None           |
| P4-001 | Security Audit                | 16h    | P4       | All            |
| P4-002 | Release Packaging             | 8h     | P4       | P4-001         |
| P4-003 | Beta Deployment               | 8h     | P4       | P4-002         |

### Total Effort Estimation

| Phase     | Tasks  | Total Hours   |
| --------- | ------ | ------------- |
| Phase 0   | 8      | 68 hours      |
| Phase 1   | 11     | 52 hours      |
| Phase 2   | 5      | 52 hours      |
| Phase 3   | 3      | 26 hours      |
| Phase 4   | 3      | 32 hours      |
| **TOTAL** | **30** | **230 hours** |

### Parallel Execution Opportunities

```
PARALLEL GROUP A (Week 1):
├── P0-001 (Test Scaffold)
├── P0-002 (DevContainer)
└── P0-003 (Tunnel Core - start)

PARALLEL GROUP B (Week 2-3):
├── P0-003 (Tunnel Core - continue)
├── P0-004 (FFI Bridge)
└── P0-005 (Peer Handshake)

PARALLEL GROUP C (Week 4-5):
├── P0-006 (Sockets)
├── P0-007 (TUN/TAP)
├── P0-008 (API Gateway)
└── P1-001:010 (Sub-Agents - can start after P0-004)

PARALLEL GROUP D (Week 6-8):
├── P1-011 (Semantic Router)
├── P2-001 (Rust Tests)
├── P2-002 (Python Tests)
└── P2-004 (Benchmarks)

PARALLEL GROUP E (Week 9-10):
├── P2-003 (Integration Tests)
├── P2-005 (Fuzz Testing)
├── P3-001 (API Docs)
├── P3-002 (User Guide)
└── P3-003 (Helm Chart)

SEQUENTIAL (Week 10-11):
└── P4-001 → P4-002 → P4-003
```

---

## 📌 Execution Start Commands

### Begin Phase 0 Immediately

```bash
# Clone or navigate to project
cd phantom-mesh-vpn

# Start with parallel foundational tasks
# Terminal 1: Test Infrastructure
bash scripts/p0-001-test-scaffold.sh

# Terminal 2: DevContainer
bash scripts/p0-002-devcontainer.sh

# Terminal 3: Begin WireGuard Core (P0-003)
# This is the critical path - start immediately
cargo new --lib src/vpn_core/noise_protocol
```

### Automated Execution Mode

For maximum autonomy, use the GitHub Copilot Agent or similar to execute tasks:

```
@APEX Execute P0-003: Implement WireGuard-style tunnel core
@ECLIPSE Execute P0-001: Create test infrastructure scaffold
@FLUX Execute P0-002: Set up development container
```

---

**Document Status:** READY FOR EXECUTION  
**Next Review:** After Phase 0 completion  
**Owner:** @ARCHITECT (Elite Agent Collective)

---

_"Architecture is the art of making complexity manageable and change inevitable."_
