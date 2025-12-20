# PhantomMesh VPN: Master Class Initialization Prompt for GitHub Copilot

> **Directive Classification:** Phase 1 Execution | Autonomous Scaffold Generation
> **Target Environment:** VS Code with Copilot Agent Mode
> **Estimated Execution:** Days 1-2 of 14-Day Sprint

---

## PRIME DIRECTIVE [REF:PD-001]

You are initiating the **PhantomMesh VPN** project—a trans-dimensional, agent-swarm orchestrated privacy fortress. Your mission is to scaffold a production-grade repository structure optimized for autonomous development, leveraging the Elite Agent framework and Sigma integrations.

**Project Identity:**
- **Name:** `phantom-mesh-vpn`
- **Vision:** Self-adaptive VPN with semantic routing, quantum-resistant cryptography, and emergent threat intelligence
- **Tech Core:** Rust (protocol/crypto) | Python (AI/agents) | Docker (orchestration)
- **License:** GPL-3.0 with proprietary agent clauses
- **Copyright:** © 2025 Stephen Bilodeau

---

## SECTION 1: REPOSITORY SCAFFOLD GENERATION [REF:RS-100]

### 1.1 Initialize Repository Structure [REF:RS-101]

Create the complete directory tree with placeholder files. Execute sequentially:

```
phantom-mesh-vpn/
├── .devcontainer/
│   └── devcontainer.json
├── .github/
│   └── workflows/
│       ├── ci.yml
│       ├── security-scan.yml
│       └── release.yml
├── configs/
│   ├── node_profiles.yaml
│   └── mesh_configs.yaml
├── docs/
│   ├── architecture.md
│   ├── ip_protections.md
│   └── api_docs/
│       └── .gitkeep
├── src/
│   ├── vpn_core/
│   │   ├── mod.rs
│   │   ├── tunnel_engine.rs
│   │   ├── routing_manager.py
│   │   └── api_gateway.rs
│   ├── security_layer/
│   │   ├── mod.rs
│   │   ├── threat_engine.py
│   │   └── crypto_manager.rs
│   └── agent_swarm/
│       ├── __init__.py
│       ├── phantom_orchestrator.py
│       ├── vpn_hooks.py
│       └── sub_agents/
│           ├── __init__.py
│           ├── velocity.py
│           ├── cipher.py
│           ├── genesis.py
│           ├── aegis.py
│           ├── nexus.py
│           ├── stream.py
│           ├── apex.py
│           ├── omniscient.py
│           └── fortress.py
├── utils/
│   ├── mnemonics.rs
│   └── sigma_integrations/
│       ├── mod.rs
│       ├── sigma_lang_decoder.rs
│       └── sigma_vault_crypto.rs
├── tests/
│   ├── unit_tests/
│   │   ├── test_crypto.rs
│   │   ├── test_routing.py
│   │   └── test_agents.py
│   └── e2e_tests/
│       └── test_network_sim.py
├── deployment_scripts/
│   ├── k8s_cluster_setup.sh
│   └── docker_node_deploy.sh
├── Cargo.toml
├── pyproject.toml
├── docker-compose.yml
├── README.md
├── LICENSE
└── .gitignore
```

---

## SECTION 2: CORE CONFIGURATION FILES [REF:CF-200]

### 2.1 Cargo.toml — Rust Workspace [REF:CF-201]

```toml
[package]
name = "phantom-mesh-vpn"
version = "0.1.0"
edition = "2021"
authors = ["Stephen Bilodeau <stephen@phantommesh.io>"]
license = "GPL-3.0"
description = "Trans-dimensional agent-swarm VPN with quantum-resistant routing"
repository = "https://github.com/phantommesh/phantom-mesh-vpn"

[dependencies]
# WireGuard Protocol
boringtun = "0.6"
x25519-dalek = "2.0"

# Quantum-Resistant Cryptography (Kyber/CRYSTALS)
pqcrypto-kyber = "0.8"
pqcrypto-dilithium = "0.5"

# Networking
tokio = { version = "1.35", features = ["full"] }
libp2p = { version = "0.53", features = ["tcp", "noise", "yamux", "gossipsub"] }

# API Gateway
axum = "0.7"
tower = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Crypto Utilities
ring = "0.17"
rand = "0.8"
blake3 = "1.5"

# Logging & Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# FFI for Python Integration
pyo3 = { version = "0.20", features = ["extension-module"] }

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"

[lib]
name = "phantom_mesh"
crate-type = ["cdylib", "rlib"]

[[bin]]
name = "phantom-node"
path = "src/main.rs"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 2.2 pyproject.toml — Python Agent Framework [REF:CF-202]

```toml
[build-system]
requires = ["maturin>=1.4,<2.0"]
build-backend = "maturin"

[project]
name = "phantom-mesh-agents"
version = "0.1.0"
description = "Elite Agent Swarm for PhantomMesh VPN orchestration"
authors = [{ name = "Stephen Bilodeau", email = "stephen@phantommesh.io" }]
license = { text = "GPL-3.0" }
requires-python = ">=3.11"
classifiers = [
    "Development Status :: 3 - Alpha",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Rust",
    "Topic :: System :: Networking",
    "Topic :: Security :: Cryptography",
]
dependencies = [
    "asyncio>=3.4",
    "aiohttp>=3.9",
    "pydantic>=2.5",
    "structlog>=24.1",
    "prometheus-client>=0.19",
    "numpy>=1.26",
    "scikit-learn>=1.4",  # For threat pattern analysis
    "networkx>=3.2",       # Mesh topology analysis
    "cryptography>=42.0",
    "python-dotenv>=1.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.4",
    "pytest-asyncio>=0.23",
    "pytest-cov>=4.1",
    "mypy>=1.8",
    "ruff>=0.1",
    "black>=24.1",
]
simulation = [
    "mininet>=2.3",
    "scapy>=2.5",
]

[tool.maturin]
features = ["pyo3/extension-module"]
python-source = "src/agent_swarm"

[tool.ruff]
line-length = 100
target-version = "py311"

[tool.mypy]
python_version = "3.11"
strict = true
```

### 2.3 docker-compose.yml — Orchestration Layer [REF:CF-203]

```yaml
version: "3.9"

services:
  # Primary VPN Node
  phantom-node:
    build:
      context: .
      dockerfile: Dockerfile.node
    container_name: phantom-primary
    cap_add:
      - NET_ADMIN
      - SYS_MODULE
    sysctls:
      - net.ipv4.ip_forward=1
      - net.ipv4.conf.all.src_valid_mark=1
    volumes:
      - ./configs:/etc/phantom-mesh:ro
      - phantom-state:/var/lib/phantom-mesh
      - /dev/net/tun:/dev/net/tun
    networks:
      - phantom-mesh-net
    ports:
      - "51820:51820/udp"  # WireGuard
      - "8080:8080"         # API Gateway
    environment:
      - RUST_LOG=info,phantom_mesh=debug
      - PHANTOM_MODE=primary
      - AGENT_SWARM_ENABLED=true
    restart: unless-stopped

  # Agent Swarm Orchestrator
  agent-swarm:
    build:
      context: .
      dockerfile: Dockerfile.agents
    container_name: phantom-agents
    volumes:
      - ./src/agent_swarm:/app/agents:ro
      - agent-state:/var/lib/phantom-agents
    networks:
      - phantom-mesh-net
    environment:
      - PYTHONUNBUFFERED=1
      - PHANTOM_ORCHESTRATOR_MODE=autonomous
      - MNEMONIC_CACHE_SIZE=10000
      - SIGMA_VAULT_ENDPOINT=http://phantom-node:8080/sigma
    depends_on:
      - phantom-node
    restart: unless-stopped

  # Monitoring Stack
  prometheus:
    image: prom/prometheus:v2.48.0
    container_name: phantom-metrics
    volumes:
      - ./configs/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    networks:
      - phantom-mesh-net
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=30d'

  grafana:
    image: grafana/grafana:10.2.3
    container_name: phantom-dashboard
    volumes:
      - grafana-data:/var/lib/grafana
      - ./configs/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
    networks:
      - phantom-mesh-net
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=phantom_secure_2025
      - GF_USERS_ALLOW_SIGN_UP=false
    depends_on:
      - prometheus

networks:
  phantom-mesh-net:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16

volumes:
  phantom-state:
  agent-state:
  prometheus-data:
  grafana-data:
```

### 2.4 .devcontainer/devcontainer.json [REF:CF-204]

```json
{
  "name": "PhantomMesh VPN Development",
  "build": {
    "dockerfile": "Dockerfile.dev",
    "context": ".."
  },
  "features": {
    "ghcr.io/devcontainers/features/rust:1": {
      "version": "stable",
      "profile": "default"
    },
    "ghcr.io/devcontainers/features/python:1": {
      "version": "3.11"
    },
    "ghcr.io/devcontainers/features/docker-in-docker:2": {},
    "ghcr.io/devcontainers/features/github-cli:1": {}
  },
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer",
        "ms-python.python",
        "ms-python.vscode-pylance",
        "github.copilot",
        "github.copilot-chat",
        "tamasfe.even-better-toml",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb",
        "ms-azuretools.vscode-docker",
        "redhat.vscode-yaml",
        "eamodio.gitlens"
      ],
      "settings": {
        "rust-analyzer.checkOnSave.command": "clippy",
        "python.analysis.typeCheckingMode": "strict",
        "editor.formatOnSave": true,
        "[rust]": { "editor.defaultFormatter": "rust-lang.rust-analyzer" },
        "[python]": { "editor.defaultFormatter": "ms-python.black-formatter" }
      }
    }
  },
  "forwardPorts": [51820, 8080, 9090, 3000],
  "postCreateCommand": "cargo build && pip install -e '.[dev]'",
  "remoteUser": "vscode",
  "containerEnv": {
    "RUST_BACKTRACE": "1",
    "PHANTOM_DEV_MODE": "true"
  }
}
```

---

## SECTION 3: AGENT SWARM BOOTSTRAP [REF:AS-300]

### 3.1 Phantom Orchestrator Core [REF:AS-301]

Generate `src/agent_swarm/phantom_orchestrator.py`:

```python
"""
PhantomMesh Agent Swarm Orchestrator
=====================================
@PHANTOM Prime Controller with MNEMONIC Integration

Copyright © 2025 Stephen Bilodeau. All rights reserved.
Licensed under GPL-3.0 with proprietary agent clauses.
"""

from __future__ import annotations

import asyncio
import hashlib
from abc import ABC, abstractmethod
from collections.abc import Callable, Coroutine
from dataclasses import dataclass, field
from datetime import datetime, UTC
from enum import Enum, auto
from functools import lru_cache
from typing import Any, TypeVar, Generic

import structlog
from pydantic import BaseModel, Field

logger = structlog.get_logger(__name__)

# ═══════════════════════════════════════════════════════════════════════════════
# MNEMONIC CACHE — O(1) Recall System
# ═══════════════════════════════════════════════════════════════════════════════

class MnemonicCache:
    """
    Holographic memory system for sub-linear threat pattern recall.
    Implements temporal decay with priority elevation for active threats.
    """
    
    def __init__(self, max_size: int = 10_000):
        self._cache: dict[str, tuple[Any, float, int]] = {}  # key -> (value, timestamp, priority)
        self._max_size = max_size
        self._access_log: list[str] = []
    
    def store(self, key: str, value: Any, priority: int = 1) -> None:
        """Store with O(1) insertion, automatic eviction on overflow."""
        if len(self._cache) >= self._max_size:
            self._evict_lowest_priority()
        
        self._cache[key] = (value, datetime.now(UTC).timestamp(), priority)
        logger.debug("mnemonic_store", key=self._hash_key(key), priority=priority)
    
    def recall(self, key: str) -> Any | None:
        """O(1) recall with priority boost on access."""
        if key in self._cache:
            value, ts, priority = self._cache[key]
            # Boost priority on recall (reinforcement learning signal)
            self._cache[key] = (value, datetime.now(UTC).timestamp(), min(priority + 1, 10))
            self._access_log.append(key)
            return value
        return None
    
    def _evict_lowest_priority(self) -> None:
        """Evict oldest low-priority entries."""
        if not self._cache:
            return
        
        # Find lowest priority, oldest entry
        evict_key = min(
            self._cache.keys(),
            key=lambda k: (self._cache[k][2], -self._cache[k][1])  # (priority, -timestamp)
        )
        del self._cache[evict_key]
        logger.debug("mnemonic_evict", key=self._hash_key(evict_key))
    
    @staticmethod
    def _hash_key(key: str) -> str:
        return hashlib.sha256(key.encode()).hexdigest()[:12]


# ═══════════════════════════════════════════════════════════════════════════════
# AGENT DEFINITIONS — Elite Squad
# ═══════════════════════════════════════════════════════════════════════════════

class AgentRole(Enum):
    """Elite Agent role classifications."""
    APEX = auto()        # Strategic command & coordination
    PHANTOM = auto()     # Stealth operations & cloaking
    CIPHER = auto()      # Cryptographic operations
    VELOCITY = auto()    # Performance optimization & routing
    FORTRESS = auto()    # Threat detection & defense
    GENESIS = auto()     # Evolution & adaptation
    AEGIS = auto()       # Security scanning & hardening
    NEXUS = auto()       # Integration & deployment
    STREAM = auto()      # Traffic analysis & flow control
    OMNISCIENT = auto()  # Global awareness & monitoring


@dataclass
class AgentState:
    """Immutable agent state snapshot."""
    role: AgentRole
    active: bool = True
    task_queue: list[str] = field(default_factory=list)
    last_action: datetime = field(default_factory=lambda: datetime.now(UTC))
    performance_score: float = 1.0
    mnemonic_keys: list[str] = field(default_factory=list)


class EliteAgent(ABC):
    """
    Abstract base for all Elite Agents.
    Implements autonomous operation loop with MNEMONIC integration.
    """
    
    def __init__(
        self,
        role: AgentRole,
        orchestrator: PhantomOrchestrator,
        mnemonic: MnemonicCache,
    ):
        self.role = role
        self.orchestrator = orchestrator
        self.mnemonic = mnemonic
        self.state = AgentState(role=role)
        self._running = False
        self._task_handlers: dict[str, Callable[..., Coroutine[Any, Any, Any]]] = {}
        
        logger.info("agent_initialized", role=role.name)
    
    @abstractmethod
    async def execute_mission(self, directive: str, context: dict[str, Any]) -> dict[str, Any]:
        """Execute primary mission directive. Must be implemented by subclass."""
        ...
    
    async def run_loop(self) -> None:
        """Autonomous operation loop with self-healing."""
        self._running = True
        logger.info("agent_loop_started", role=self.role.name)
        
        while self._running:
            try:
                # Check for pending tasks
                if self.state.task_queue:
                    task = self.state.task_queue.pop(0)
                    await self._process_task(task)
                
                # Autonomous scanning interval
                await asyncio.sleep(0.1)
                
            except Exception as e:
                logger.error("agent_loop_error", role=self.role.name, error=str(e))
                await self._self_heal(e)
    
    async def _process_task(self, task: str) -> None:
        """Process queued task with mnemonic recall."""
        # Check mnemonic for similar past tasks
        cached_result = self.mnemonic.recall(f"{self.role.name}:{task}")
        if cached_result:
            logger.debug("mnemonic_hit", role=self.role.name, task=task[:50])
        
        result = await self.execute_mission(task, {"cached": cached_result})
        
        # Store successful patterns
        if result.get("success"):
            self.mnemonic.store(f"{self.role.name}:{task}", result, priority=2)
        
        self.state.last_action = datetime.now(UTC)
    
    async def _self_heal(self, error: Exception) -> None:
        """Autonomous recovery from errors."""
        self.state.performance_score *= 0.9  # Decay on error
        await asyncio.sleep(1)  # Backoff
        
        if self.state.performance_score < 0.3:
            logger.warning("agent_degraded", role=self.role.name)
            await self.orchestrator.request_agent_reset(self)
    
    def stop(self) -> None:
        """Graceful shutdown."""
        self._running = False
        logger.info("agent_stopped", role=self.role.name)


# ═══════════════════════════════════════════════════════════════════════════════
# PHANTOM ORCHESTRATOR — Swarm Controller
# ═══════════════════════════════════════════════════════════════════════════════

class PhantomOrchestrator:
    """
    Central swarm controller for Elite Agent coordination.
    Implements ΣLANG directive parsing and ΣVault integration.
    """
    
    def __init__(self):
        self.mnemonic = MnemonicCache(max_size=10_000)
        self.agents: dict[AgentRole, EliteAgent] = {}
        self.event_bus: asyncio.Queue[dict[str, Any]] = asyncio.Queue()
        self._running = False
        
        logger.info("orchestrator_initialized")
    
    async def spawn_swarm(self) -> None:
        """Initialize all Elite Agents."""
        from .sub_agents import (
            ApexAgent, PhantomAgent, CipherAgent, VelocityAgent,
            FortressAgent, GenesisAgent, AegisAgent, NexusAgent,
            StreamAgent, OmniscientAgent
        )
        
        agent_classes = {
            AgentRole.APEX: ApexAgent,
            AgentRole.PHANTOM: PhantomAgent,
            AgentRole.CIPHER: CipherAgent,
            AgentRole.VELOCITY: VelocityAgent,
            AgentRole.FORTRESS: FortressAgent,
            AgentRole.GENESIS: GenesisAgent,
            AgentRole.AEGIS: AegisAgent,
            AgentRole.NEXUS: NexusAgent,
            AgentRole.STREAM: StreamAgent,
            AgentRole.OMNISCIENT: OmniscientAgent,
        }
        
        for role, cls in agent_classes.items():
            self.agents[role] = cls(role, self, self.mnemonic)
            logger.info("agent_spawned", role=role.name)
    
    async def dispatch_directive(
        self,
        target_role: AgentRole,
        directive: str,
        priority: int = 1
    ) -> None:
        """Dispatch directive to specific agent."""
        if target_role in self.agents:
            agent = self.agents[target_role]
            agent.state.task_queue.append(directive)
            
            # Store in mnemonic for pattern learning
            self.mnemonic.store(
                f"directive:{target_role.name}:{directive[:50]}",
                {"timestamp": datetime.now(UTC).isoformat(), "priority": priority},
                priority=priority
            )
            
            logger.info("directive_dispatched", target=target_role.name, directive=directive[:100])
    
    async def broadcast(self, event: dict[str, Any]) -> None:
        """Broadcast event to all agents via event bus."""
        await self.event_bus.put(event)
        logger.debug("event_broadcast", event_type=event.get("type"))
    
    async def request_agent_reset(self, agent: EliteAgent) -> None:
        """Handle agent reset request from degraded agent."""
        logger.warning("agent_reset_requested", role=agent.role.name)
        agent.state = AgentState(role=agent.role)
        agent.state.performance_score = 0.5  # Partial recovery
    
    async def run(self) -> None:
        """Main orchestrator loop."""
        self._running = True
        await self.spawn_swarm()
        
        # Start all agent loops
        tasks = [
            asyncio.create_task(agent.run_loop())
            for agent in self.agents.values()
        ]
        
        # Event bus processor
        async def process_events():
            while self._running:
                try:
                    event = await asyncio.wait_for(self.event_bus.get(), timeout=1.0)
                    await self._handle_event(event)
                except asyncio.TimeoutError:
                    continue
        
        tasks.append(asyncio.create_task(process_events()))
        
        logger.info("orchestrator_running", agent_count=len(self.agents))
        await asyncio.gather(*tasks, return_exceptions=True)
    
    async def _handle_event(self, event: dict[str, Any]) -> None:
        """Route events to appropriate agents."""
        event_type = event.get("type", "unknown")
        
        routing_table = {
            "threat_detected": AgentRole.FORTRESS,
            "route_optimization": AgentRole.VELOCITY,
            "crypto_rotation": AgentRole.CIPHER,
            "cloak_request": AgentRole.PHANTOM,
            "deploy_request": AgentRole.NEXUS,
            "scan_request": AgentRole.AEGIS,
            "evolution_trigger": AgentRole.GENESIS,
            "traffic_analysis": AgentRole.STREAM,
            "global_status": AgentRole.OMNISCIENT,
            "strategic_decision": AgentRole.APEX,
        }
        
        target_role = routing_table.get(event_type)
        if target_role and target_role in self.agents:
            await self.dispatch_directive(target_role, str(event))
    
    def shutdown(self) -> None:
        """Graceful swarm shutdown."""
        self._running = False
        for agent in self.agents.values():
            agent.stop()
        logger.info("orchestrator_shutdown")


# ═══════════════════════════════════════════════════════════════════════════════
# ENTRY POINT
# ═══════════════════════════════════════════════════════════════════════════════

async def main() -> None:
    """Bootstrap the PhantomMesh Agent Swarm."""
    orchestrator = PhantomOrchestrator()
    
    try:
        await orchestrator.run()
    except KeyboardInterrupt:
        logger.info("shutdown_signal_received")
    finally:
        orchestrator.shutdown()


if __name__ == "__main__":
    asyncio.run(main())
```

### 3.2 VPN Event Hooks [REF:AS-302]

Generate `src/agent_swarm/vpn_hooks.py`:

```python
"""
VPN Event Hooks — Agent-VPN Integration Layer
==============================================
Bridges Rust VPN core with Python agent swarm.

Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from datetime import datetime, UTC
from enum import Enum, auto
from typing import Any, Callable, Coroutine

import structlog

logger = structlog.get_logger(__name__)


class VPNEventType(Enum):
    """VPN core event classifications."""
    # Connection Events
    PEER_CONNECTED = auto()
    PEER_DISCONNECTED = auto()
    HANDSHAKE_INITIATED = auto()
    HANDSHAKE_COMPLETED = auto()
    
    # Traffic Events
    PACKET_ROUTED = auto()
    BANDWIDTH_THRESHOLD = auto()
    LATENCY_SPIKE = auto()
    
    # Security Events
    THREAT_SIGNATURE = auto()
    ANOMALY_DETECTED = auto()
    KEY_ROTATION_DUE = auto()
    
    # Mesh Events
    TOPOLOGY_CHANGE = auto()
    NODE_DISCOVERY = auto()
    ROUTE_OPTIMIZATION = auto()
    
    # System Events
    RESOURCE_WARNING = auto()
    CONFIG_RELOAD = auto()


@dataclass(frozen=True)
class VPNEvent:
    """Immutable VPN event payload."""
    event_type: VPNEventType
    timestamp: datetime
    source_node: str
    payload: dict[str, Any]
    priority: int = 1  # 1-10, higher = more urgent


class VPNHookRegistry:
    """
    Central registry for VPN event hooks.
    Enables agent swarm to react to VPN core events.
    """
    
    def __init__(self):
        self._hooks: dict[VPNEventType, list[Callable[[VPNEvent], Coroutine[Any, Any, None]]]] = {
            evt: [] for evt in VPNEventType
        }
        self._global_hooks: list[Callable[[VPNEvent], Coroutine[Any, Any, None]]] = []
    
    def register(
        self,
        event_type: VPNEventType,
        handler: Callable[[VPNEvent], Coroutine[Any, Any, None]]
    ) -> None:
        """Register handler for specific event type."""
        self._hooks[event_type].append(handler)
        logger.debug("hook_registered", event_type=event_type.name)
    
    def register_global(
        self,
        handler: Callable[[VPNEvent], Coroutine[Any, Any, None]]
    ) -> None:
        """Register handler for all events."""
        self._global_hooks.append(handler)
        logger.debug("global_hook_registered")
    
    async def emit(self, event: VPNEvent) -> None:
        """Emit event to all registered handlers."""
        handlers = self._hooks[event.event_type] + self._global_hooks
        
        if not handlers:
            logger.debug("event_unhandled", event_type=event.event_type.name)
            return
        
        # Fire all handlers concurrently
        tasks = [handler(event) for handler in handlers]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                logger.error(
                    "hook_error",
                    event_type=event.event_type.name,
                    error=str(result)
                )


# Global registry instance
_registry = VPNHookRegistry()


def get_hook_registry() -> VPNHookRegistry:
    """Get the global hook registry."""
    return _registry


# ═══════════════════════════════════════════════════════════════════════════════
# CONVENIENCE DECORATORS
# ═══════════════════════════════════════════════════════════════════════════════

def on_vpn_event(*event_types: VPNEventType):
    """Decorator to register a function as an event handler."""
    def decorator(func: Callable[[VPNEvent], Coroutine[Any, Any, None]]):
        for evt in event_types:
            _registry.register(evt, func)
        return func
    return decorator


def on_all_vpn_events(func: Callable[[VPNEvent], Coroutine[Any, Any, None]]):
    """Decorator to register a function for all events."""
    _registry.register_global(func)
    return func


# ═══════════════════════════════════════════════════════════════════════════════
# RUST FFI BRIDGE (called from Rust via PyO3)
# ═══════════════════════════════════════════════════════════════════════════════

def rust_emit_event(
    event_type_str: str,
    source_node: str,
    payload_json: str,
    priority: int = 1
) -> None:
    """
    Entry point for Rust VPN core to emit events.
    Called via PyO3 FFI bridge.
    """
    import json
    
    try:
        event_type = VPNEventType[event_type_str]
        payload = json.loads(payload_json)
        
        event = VPNEvent(
            event_type=event_type,
            timestamp=datetime.now(UTC),
            source_node=source_node,
            payload=payload,
            priority=priority
        )
        
        # Schedule async emission
        asyncio.create_task(_registry.emit(event))
        
    except (KeyError, json.JSONDecodeError) as e:
        logger.error("rust_event_parse_error", error=str(e))
```

---

## SECTION 4: RUST VPN CORE STUBS [REF:RC-400]

### 4.1 Tunnel Engine Foundation [REF:RC-401]

Generate `src/vpn_core/tunnel_engine.rs`:

```rust
//! PhantomMesh Tunnel Engine
//! ==========================
//! WireGuard-based tunneling with ΣVault dimensional scattering.
//!
//! Copyright © 2025 Stephen Bilodeau. All rights reserved.
//! Licensed under GPL-3.0 with proprietary agent clauses.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, instrument, warn};

use crate::security_layer::crypto_manager::CryptoManager;

/// Maximum transmission unit for tunnel packets
const TUNNEL_MTU: usize = 1420;

/// Dimensional scatter factor for ΣVault routing
const SIGMA_SCATTER_DIMENSIONS: u8 = 7;

/// Peer state in the mesh network
#[derive(Debug, Clone)]
pub struct PeerState {
    pub public_key: [u8; 32],
    pub endpoint: Option<SocketAddr>,
    pub last_handshake: Option<std::time::Instant>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub keepalive_interval: u16,
}

/// Tunnel packet with dimensional metadata
#[derive(Debug)]
pub struct TunnelPacket {
    pub dimension: u8,
    pub sequence: u64,
    pub payload: Vec<u8>,
    pub scatter_proof: [u8; 32],
}

/// Core tunnel engine managing WireGuard sessions
pub struct TunnelEngine {
    /// Active peer sessions
    peers: Arc<RwLock<HashMap<[u8; 32], PeerState>>>,
    
    /// Cryptographic operations handler
    crypto: Arc<CryptoManager>,
    
    /// Event channel for agent hooks
    event_tx: mpsc::Sender<TunnelEvent>,
    
    /// Dimensional scatter state for ΣVault
    scatter_state: Arc<RwLock<ScatterState>>,
}

/// Events emitted to agent swarm
#[derive(Debug, Clone)]
pub enum TunnelEvent {
    PeerConnected { public_key: [u8; 32], endpoint: SocketAddr },
    PeerDisconnected { public_key: [u8; 32] },
    PacketRouted { dimension: u8, bytes: usize },
    ThreatSignature { signature: Vec<u8>, source: SocketAddr },
}

/// ΣVault dimensional scatter state
#[derive(Debug, Default)]
struct ScatterState {
    active_dimensions: [bool; SIGMA_SCATTER_DIMENSIONS as usize],
    dimension_loads: [u64; SIGMA_SCATTER_DIMENSIONS as usize],
    last_rotation: std::time::Instant,
}

impl TunnelEngine {
    /// Create a new tunnel engine instance
    pub fn new(crypto: Arc<CryptoManager>, event_tx: mpsc::Sender<TunnelEvent>) -> Self {
        info!("Initializing PhantomMesh Tunnel Engine");
        
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            crypto,
            event_tx,
            scatter_state: Arc::new(RwLock::new(ScatterState {
                active_dimensions: [true; SIGMA_SCATTER_DIMENSIONS as usize],
                dimension_loads: [0; SIGMA_SCATTER_DIMENSIONS as usize],
                last_rotation: std::time::Instant::now(),
            })),
        }
    }
    
    /// Add a peer to the mesh
    #[instrument(skip(self))]
    pub async fn add_peer(&self, public_key: [u8; 32], endpoint: Option<SocketAddr>) -> Result<(), TunnelError> {
        let mut peers = self.peers.write().await;
        
        let state = PeerState {
            public_key,
            endpoint,
            last_handshake: None,
            rx_bytes: 0,
            tx_bytes: 0,
            keepalive_interval: 25,
        };
        
        peers.insert(public_key, state);
        
        if let Some(ep) = endpoint {
            let _ = self.event_tx.send(TunnelEvent::PeerConnected {
                public_key,
                endpoint: ep,
            }).await;
        }
        
        info!(peer = ?public_key[..8], "Peer added to mesh");
        Ok(())
    }
    
    /// Route packet through dimensional scatter
    #[instrument(skip(self, packet))]
    pub async fn route_packet(&self, packet: &[u8], destination: [u8; 32]) -> Result<(), TunnelError> {
        // Select optimal dimension based on load
        let dimension = self.select_dimension().await;
        
        // Apply ΣVault scatter transformation
        let scattered = self.apply_scatter(packet, dimension).await?;
        
        // Encrypt with quantum-resistant cipher
        let encrypted = self.crypto.encrypt_packet(&scattered, &destination)?;
        
        // Update metrics
        {
            let mut state = self.scatter_state.write().await;
            state.dimension_loads[dimension as usize] += encrypted.len() as u64;
        }
        
        let _ = self.event_tx.send(TunnelEvent::PacketRouted {
            dimension,
            bytes: encrypted.len(),
        }).await;
        
        debug!(dimension, bytes = encrypted.len(), "Packet routed");
        Ok(())
    }
    
    /// Select least-loaded dimension for routing
    async fn select_dimension(&self) -> u8 {
        let state = self.scatter_state.read().await;
        
        state.dimension_loads
            .iter()
            .enumerate()
            .filter(|(i, _)| state.active_dimensions[*i])
            .min_by_key(|(_, load)| *load)
            .map(|(i, _)| i as u8)
            .unwrap_or(0)
    }
    
    /// Apply ΣVault dimensional scatter transformation
    async fn apply_scatter(&self, data: &[u8], dimension: u8) -> Result<Vec<u8>, TunnelError> {
        // TODO: Implement full ΣVault scatter algorithm
        // For now, prepend dimension marker
        let mut scattered = Vec::with_capacity(data.len() + 1);
        scattered.push(dimension);
        scattered.extend_from_slice(data);
        Ok(scattered)
    }
}

/// Tunnel operation errors
#[derive(Debug, thiserror::Error)]
pub enum TunnelError {
    #[error("Peer not found: {0:?}")]
    PeerNotFound([u8; 32]),
    
    #[error("Encryption failed: {0}")]
    EncryptionError(String),
    
    #[error("Scatter transformation failed: {0}")]
    ScatterError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dimension_selection() {
        // TODO: Implement dimension selection tests
    }
    
    #[tokio::test]
    async fn test_peer_lifecycle() {
        // TODO: Implement peer lifecycle tests
    }
}
```

### 4.2 Crypto Manager Stub [REF:RC-402]

Generate `src/security_layer/crypto_manager.rs`:

```rust
//! PhantomMesh Crypto Manager
//! ===========================
//! Quantum-resistant cryptography with Kyber/CRYSTALS and temporal key evolution.
//!
//! Copyright © 2025 Stephen Bilodeau. All rights reserved.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Key evolution interval in seconds
const KEY_EVOLUTION_INTERVAL: u64 = 300;

/// Cryptographic key pair with temporal metadata
#[derive(Clone)]
pub struct KeyPair {
    pub public: Vec<u8>,
    pub secret: Vec<u8>,
    pub created_at: std::time::Instant,
    pub evolution_count: u32,
}

/// Manages all cryptographic operations with quantum resistance
pub struct CryptoManager {
    /// Current Kyber keypair
    kyber_keys: Arc<RwLock<KeyPair>>,
    
    /// Current Dilithium signing keys
    dilithium_keys: Arc<RwLock<KeyPair>>,
    
    /// Session keys for active peers
    session_keys: Arc<RwLock<std::collections::HashMap<[u8; 32], Vec<u8>>>>,
}

impl CryptoManager {
    /// Initialize crypto manager with fresh key generation
    pub fn new() -> Result<Self, CryptoError> {
        info!("Initializing quantum-resistant crypto manager");
        
        // Generate initial Kyber keypair
        let (kyber_pk, kyber_sk) = Self::generate_kyber_keypair()?;
        
        // Generate initial Dilithium keypair
        let (dilithium_pk, dilithium_sk) = Self::generate_dilithium_keypair()?;
        
        Ok(Self {
            kyber_keys: Arc::new(RwLock::new(KeyPair {
                public: kyber_pk,
                secret: kyber_sk,
                created_at: std::time::Instant::now(),
                evolution_count: 0,
            })),
            dilithium_keys: Arc::new(RwLock::new(KeyPair {
                public: dilithium_pk,
                secret: dilithium_sk,
                created_at: std::time::Instant::now(),
                evolution_count: 0,
            })),
            session_keys: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }
    
    /// Generate Kyber-1024 keypair
    fn generate_kyber_keypair() -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        // TODO: Integrate pqcrypto-kyber
        // Placeholder with dummy keys
        debug!("Generating Kyber-1024 keypair");
        Ok((vec![0u8; 1568], vec![0u8; 3168]))
    }
    
    /// Generate Dilithium-3 keypair
    fn generate_dilithium_keypair() -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        // TODO: Integrate pqcrypto-dilithium
        debug!("Generating Dilithium-3 keypair");
        Ok((vec![0u8; 1952], vec![0u8; 4000]))
    }
    
    /// Encrypt packet for specific peer
    #[instrument(skip(self, data))]
    pub fn encrypt_packet(&self, data: &[u8], peer_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        // TODO: Implement hybrid encryption
        // Kyber KEM + ChaCha20-Poly1305
        debug!(data_len = data.len(), "Encrypting packet");
        
        // Placeholder: just return data with auth tag placeholder
        let mut encrypted = data.to_vec();
        encrypted.extend_from_slice(&[0u8; 16]); // Placeholder auth tag
        Ok(encrypted)
    }
    
    /// Decrypt packet from peer
    #[instrument(skip(self, data))]
    pub fn decrypt_packet(&self, data: &[u8], peer_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        // TODO: Implement hybrid decryption
        debug!(data_len = data.len(), "Decrypting packet");
        
        if data.len() < 16 {
            return Err(CryptoError::InvalidCiphertext("Too short".into()));
        }
        
        Ok(data[..data.len() - 16].to_vec())
    }
    
    /// Evolve keys based on temporal schedule (ΣVault integration)
    #[instrument(skip(self))]
    pub async fn evolve_keys(&self) -> Result<(), CryptoError> {
        let mut kyber = self.kyber_keys.write().await;
        
        if kyber.created_at.elapsed().as_secs() > KEY_EVOLUTION_INTERVAL {
            let (pk, sk) = Self::generate_kyber_keypair()?;
            *kyber = KeyPair {
                public: pk,
                secret: sk,
                created_at: std::time::Instant::now(),
                evolution_count: kyber.evolution_count + 1,
            };
            info!(evolution = kyber.evolution_count, "Keys evolved");
        }
        
        Ok(())
    }
    
    /// Get current public key for peer advertisement
    pub async fn get_public_key(&self) -> Vec<u8> {
        self.kyber_keys.read().await.public.clone()
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new().expect("Crypto initialization failed")
    }
}

/// Cryptographic operation errors
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    
    #[error("Encryption failed: {0}")]
    Encryption(String),
    
    #[error("Decryption failed: {0}")]
    Decryption(String),
    
    #[error("Invalid ciphertext: {0}")]
    InvalidCiphertext(String),
    
    #[error("Key evolution failed: {0}")]
    KeyEvolution(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_crypto_init() {
        let manager = CryptoManager::new();
        assert!(manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let manager = CryptoManager::new().unwrap();
        let data = b"test payload";
        let peer_key = [0u8; 32];
        
        let encrypted = manager.encrypt_packet(data, &peer_key).unwrap();
        let decrypted = manager.decrypt_packet(&encrypted, &peer_key).unwrap();
        
        assert_eq!(data.to_vec(), decrypted);
    }
}
```

---

## SECTION 5: CI/CD AUTOMATION [REF:CI-500]

### 5.1 GitHub Actions CI Pipeline [REF:CI-501]

Generate `.github/workflows/ci.yml`:

```yaml
name: PhantomMesh CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  rust-build:
    name: Rust Build & Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        with:
          components: clippy, rustfmt
      
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Check formatting
        run: cargo fmt --all -- --check
      
      - name: Clippy lints
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: Run tests
        run: cargo test --all-features --no-fail-fast
      
      - name: Build release
        run: cargo build --release

  python-test:
    name: Python Agent Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Python 3.11
        uses: actions/setup-python@v5
        with:
          python-version: "3.11"
          cache: "pip"
      
      - name: Install dependencies
        run: |
          pip install -e ".[dev]"
      
      - name: Lint with ruff
        run: ruff check src/agent_swarm
      
      - name: Type check with mypy
        run: mypy src/agent_swarm
      
      - name: Run pytest
        run: pytest tests/ -v --cov=src/agent_swarm --cov-report=xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          files: ./coverage.xml
          fail_ci_if_error: true

  docker-build:
    name: Docker Build
    runs-on: ubuntu-latest
    needs: [rust-build, python-test]
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Build VPN node image
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile.node
          push: false
          tags: phantom-mesh-vpn:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
      
      - name: Build agent image
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile.agents
          push: false
          tags: phantom-mesh-agents:${{ github.sha }}

  security-scan:
    name: Security Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run cargo-audit
        uses: rustsec/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: '.'
          severity: 'CRITICAL,HIGH'
```

---

## SECTION 6: EXECUTION DIRECTIVES [REF:EX-600]

### 6.1 Immediate Actions for Copilot [REF:EX-601]

Execute the following in sequence:

1. **Repository Initialization**
   ```bash
   mkdir -p phantom-mesh-vpn && cd phantom-mesh-vpn
   git init
   ```

2. **Generate All Directory Structure**
   - Create all directories as specified in [REF:RS-100]
   - Place configuration files from [REF:CF-200]

3. **Bootstrap Agent Swarm**
   - Generate `phantom_orchestrator.py` from [REF:AS-301]
   - Generate `vpn_hooks.py` from [REF:AS-302]
   - Create stub files for each sub-agent in `sub_agents/`

4. **Initialize Rust Core**
   - Generate `tunnel_engine.rs` from [REF:RC-401]
   - Generate `crypto_manager.rs` from [REF:RC-402]
   - Create module files (`mod.rs`) for proper imports

5. **Setup CI/CD**
   - Generate `.github/workflows/ci.yml` from [REF:CI-501]

### 6.2 Agent Activation Sequence [REF:EX-602]

After scaffold generation, activate agents in order:

| Order | Agent | Directive |
|-------|-------|-----------|
| 1 | @APEX | Coordinate scaffold validation |
| 2 | @OMNISCIENT | Configure monitoring hooks |
| 3 | @NEXUS | Validate Docker compose |
| 4 | @CIPHER | Initialize crypto stubs |
| 5 | @VELOCITY | Configure mesh routing defaults |

### 6.3 Validation Checkpoints [REF:EX-603]

Before proceeding to Phase 2, verify:

- [ ] `cargo check` passes without errors
- [ ] `pip install -e .` completes
- [ ] `docker-compose config` validates
- [ ] All 9 sub-agent files exist
- [ ] CI workflow syntax is valid

---

## APPENDIX: AGENT REFERENCE [REF:AP-700]

Quick reference for Elite Agent responsibilities:

| Agent | Symbol | Primary Function |
|-------|--------|------------------|
| APEX | @APEX | Strategic command, cross-agent coordination |
| PHANTOM | @PHANTOM | Stealth ops, dynamic cloaking |
| CIPHER | @CIPHER | Crypto operations, key management |
| VELOCITY | @VELOCITY | Performance tuning, route optimization |
| FORTRESS | @FORTRESS | Threat detection, analogy-based defense |
| GENESIS | @GENESIS | Evolutionary adaptation, feature mutation |
| AEGIS | @AEGIS | Security scanning, vulnerability assessment |
| NEXUS | @NEXUS | CI/CD, deployment orchestration |
| STREAM | @STREAM | Traffic analysis, flow control |
| OMNISCIENT | @OMNISCIENT | Global monitoring, state awareness |

---

**END OF MASTER CLASS PROMPT**

*Execute with: Copy entire prompt to Copilot Chat in VS Code Agent Mode*
*Estimated generation time: 10-15 minutes for full scaffold*

---
© 2025 Stephen Bilodeau. All rights reserved.
PhantomMesh VPN — Proprietary Architecture
