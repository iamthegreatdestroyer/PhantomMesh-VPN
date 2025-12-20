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

# Configure structured logging
import logging.config
import yaml
import os

from .discovery import get_discovery_service
from .metrics import get_metrics_exporter

# Load logging configuration
log_config_path = "/etc/prometheus/logging.yml"
if os.path.exists(log_config_path):
    with open(log_config_path, 'r') as f:
        log_config = yaml.safe_load(f)
        logging.config.dictConfig(log_config)

logger = structlog.get_logger(__name__)

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
        self.metrics_exporter = get_metrics_exporter()
        self._metrics_task: asyncio.Task | None = None
        self.discovery_service = get_discovery_service()

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

            # Register agent with discovery service
            agent_id = f"phantom-{role.name.lower()}"
            await self._register_agent_with_discovery(agent_id, role.name.lower())

            logger.info("agent_spawned", role=role.name)

    async def _register_agent_with_discovery(self, agent_id: str, agent_type: str) -> None:
        """Register agent with the discovery service."""
        try:
            import aiohttp

            # Get the metrics server port (assuming it's running on the same host)
            metrics_port = 8000  # Same as the agent swarm metrics port

            async with aiohttp.ClientSession() as session:
                registration_data = {
                    "agent_id": agent_id,
                    "agent_type": agent_type,
                    "host": "agent-swarm",  # Docker service name
                    "port": metrics_port
                }

                # Register with discovery service
                async with session.post(
                    "http://discovery:8081/register",
                    json=registration_data,
                    timeout=5
                ) as response:
                    if response.status == 200:
                        logger.info("agent_registered_with_discovery",
                                   agent_id=agent_id,
                                   agent_type=agent_type)
                    else:
                        logger.warning("agent_registration_failed",
                                     agent_id=agent_id,
                                     status=response.status)

        except Exception as e:
            logger.error("discovery_registration_error",
                        agent_id=agent_id,
                        error=str(e))

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

    async def _collect_metrics_loop(self) -> None:
        """Periodically collect and update swarm metrics."""
        import psutil
        import time

        while self._running:
            try:
                # Collect agent metrics
                total_agents = len(self.agents)
                active_agents = sum(1 for agent in self.agents.values() if agent.state.is_active)

                # Calculate swarm efficiency (simplified)
                avg_performance = sum(agent.state.performance_score for agent in self.agents.values()) / max(total_agents, 1)
                swarm_efficiency = min(avg_performance, 1.0)

                # Get memory usage
                process = psutil.Process()
                memory_used = process.memory_info().rss
                memory_limit = 2 * 1024 * 1024 * 1024  # 2GB limit (example)

                # Update swarm metrics
                swarm_metrics = SwarmMetrics(
                    total_agents=total_agents,
                    active_agents=active_agents,
                    tasks_queued=sum(len(agent.state.task_queue) for agent in self.agents.values()),
                    tasks_processing=sum(1 for agent in self.agents.values() if agent.state.current_task),
                    memory_used=memory_used,
                    memory_limit=memory_limit,
                    swarm_efficiency=swarm_efficiency,
                    last_coordination=time.time()
                )
                self.metrics_exporter.update_swarm_metrics(swarm_metrics)

                # Update individual agent metrics
                for agent in self.agents.values():
                    agent_metrics = AgentMetrics(
                        tasks_completed=agent.state.tasks_completed,
                        tasks_failed=agent.state.tasks_failed,
                        memory_usage=agent.state.memory_usage,
                        cpu_usage=agent.state.cpu_usage,
                        active_time=agent.state.active_time,
                        last_heartbeat=time.time()
                    )

                    # Convert AgentRole to AgentType
                    agent_type = AgentType(agent.role.name.lower())
                    self.metrics_exporter.update_agent_metrics(
                        f"agent-{agent.role.name.lower()}",
                        agent_type,
                        agent_metrics
                    )

                await asyncio.sleep(30)  # Update every 30 seconds

            except Exception as e:
                logger.error("metrics_collection_error", error=str(e))
                await asyncio.sleep(30)

    async def run(self) -> None:
        """Main orchestrator loop."""
        self._running = True
        await self.spawn_swarm()

        # Start metrics collection task
        self._metrics_task = asyncio.create_task(self._collect_metrics_loop())

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