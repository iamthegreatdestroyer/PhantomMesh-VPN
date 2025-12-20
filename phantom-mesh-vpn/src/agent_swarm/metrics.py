#!/usr/bin/env python3
"""
PhantomMesh Agent Swarm Metrics Exporter
========================================

Prometheus metrics exporter for the Python agent swarm orchestrator.
Provides comprehensive monitoring of agent performance, task execution,
and swarm coordination metrics.

Copyright © 2025 Stephen Bilodeau. All rights reserved.
Licensed under GPL-3.0 with proprietary agent clauses.
"""

import asyncio
import time
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from enum import Enum

from prometheus_client import (
    CollectorRegistry, Counter, Gauge, Histogram, generate_latest,
    CONTENT_TYPE_LATEST
)
from aiohttp import web
import structlog

# Configure structured logging
import logging.config
import yaml
import os

# Load logging configuration
log_config_path = "/etc/phantom-mesh/logging.yml"
if os.path.exists(log_config_path):
    with open(log_config_path, 'r') as f:
        log_config = yaml.safe_load(f)
        logging.config.dictConfig(log_config)

logger = structlog.get_logger(__name__)


class AgentType(Enum):
    """Types of agents in the swarm."""
    APEX = "apex"
    CIPHER = "cipher"
    ARCHITECT = "architect"
    FORTRESS = "fortress"
    GENESIS = "genesis"
    NEXUS = "nexus"
    OMNISCIENT = "omniscient"
    PHANTOM = "phantom"
    STREAM = "stream"
    VELOCITY = "velocity"


@dataclass
class AgentMetrics:
    """Metrics for individual agents."""
    tasks_completed: int = 0
    tasks_failed: int = 0
    memory_usage: float = 0.0
    cpu_usage: float = 0.0
    active_time: float = 0.0
    last_heartbeat: float = 0.0


@dataclass
class SwarmMetrics:
    """Global swarm metrics."""
    total_agents: int = 0
    active_agents: int = 0
    tasks_queued: int = 0
    tasks_processing: int = 0
    memory_limit: float = 0.0
    memory_used: float = 0.0
    swarm_efficiency: float = 1.0
    last_coordination: float = 0.0


class PhantomMetricsExporter:
    """Prometheus metrics exporter for PhantomMesh agent swarm."""

    def __init__(self):
        self.registry = CollectorRegistry()

        # Agent-specific metrics
        self.agent_tasks_completed = Counter(
            'phantom_agent_tasks_completed_total',
            'Total number of tasks completed by agents',
            ['agent_type', 'agent_id'],
            registry=self.registry
        )

        self.agent_tasks_failed = Counter(
            'phantom_agent_tasks_failed_total',
            'Total number of tasks failed by agents',
            ['agent_type', 'agent_id', 'failure_reason'],
            registry=self.registry
        )

        self.agent_task_duration = Histogram(
            'phantom_agent_task_duration_seconds',
            'Time taken to complete agent tasks',
            ['agent_type', 'task_type'],
            buckets=[0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0],
            registry=self.registry
        )

        self.agent_memory_usage = Gauge(
            'phantom_agent_memory_usage_bytes',
            'Current memory usage of agents',
            ['agent_type', 'agent_id'],
            registry=self.registry
        )

        self.agent_memory_limit = Gauge(
            'phantom_agent_memory_limit_bytes',
            'Memory limit for agents',
            ['agent_type', 'agent_id'],
            registry=self.registry
        )

        self.agent_cpu_usage = Gauge(
            'phantom_agent_cpu_usage_percent',
            'Current CPU usage of agents',
            ['agent_type', 'agent_id'],
            registry=self.registry
        )

        self.agent_active = Gauge(
            'phantom_agent_active',
            'Whether agent is currently active (1) or inactive (0)',
            ['agent_type', 'agent_id'],
            registry=self.registry
        )

        self.agent_heartbeat_age = Gauge(
            'phantom_agent_heartbeat_age_seconds',
            'Time since last agent heartbeat',
            ['agent_type', 'agent_id'],
            registry=self.registry
        )

        # Swarm-level metrics
        self.swarm_total_agents = Gauge(
            'phantom_swarm_total_agents',
            'Total number of agents in the swarm',
            registry=self.registry
        )

        self.swarm_active_agents = Gauge(
            'phantom_swarm_active_agents',
            'Number of currently active agents',
            registry=self.registry
        )

        self.swarm_tasks_queued = Gauge(
            'phantom_swarm_tasks_queued',
            'Number of tasks currently queued',
            registry=self.registry
        )

        self.swarm_tasks_processing = Gauge(
            'phantom_swarm_tasks_processing',
            'Number of tasks currently being processed',
            registry=self.registry
        )

        self.swarm_memory_usage = Gauge(
            'phantom_swarm_memory_usage_bytes',
            'Total memory usage of the swarm',
            registry=self.registry
        )

        self.swarm_memory_limit = Gauge(
            'phantom_swarm_memory_limit_bytes',
            'Total memory limit of the swarm',
            registry=self.registry
        )

        self.swarm_efficiency = Gauge(
            'phantom_swarm_efficiency',
            'Overall efficiency of the agent swarm (0.0-1.0)',
            registry=self.registry
        )

        self.swarm_coordination_time = Gauge(
            'phantom_swarm_coordination_time_seconds',
            'Time since last swarm coordination',
            registry=self.registry
        )

        # Elite Agent Collective metrics
        self.elite_experience_retrieval = Counter(
            'phantom_elite_experience_retrieval_total',
            'Total experience retrievals from MNEMONIC memory',
            ['agent_type', 'retrieval_type'],
            registry=self.registry
        )

        self.elite_experience_stored = Counter(
            'phantom_elite_experience_stored_total',
            'Total experiences stored in MNEMONIC memory',
            ['agent_type', 'fitness_score'],
            registry=self.registry
        )

        self.elite_memory_hit_rate = Gauge(
            'phantom_elite_memory_hit_rate',
            'Hit rate for experience retrieval (0.0-1.0)',
            ['agent_type'],
            registry=self.registry
        )

        # Agent state tracking
        self.agent_states: Dict[str, AgentMetrics] = {}
        self.swarm_state = SwarmMetrics()

        logger.info("PhantomMetricsExporter initialized")

    def update_agent_metrics(self, agent_id: str, agent_type: AgentType, metrics: AgentMetrics):
        """Update metrics for a specific agent."""
        type_str = agent_type.value

        # Update counters
        self.agent_tasks_completed.labels(agent_type=type_str, agent_id=agent_id).inc_by(metrics.tasks_completed)
        self.agent_tasks_failed.labels(agent_type=type_str, agent_id=agent_id, failure_reason="unknown").inc_by(metrics.tasks_failed)

        # Update gauges
        self.agent_memory_usage.labels(agent_type=type_str, agent_id=agent_id).set(metrics.memory_usage)
        self.agent_cpu_usage.labels(agent_type=type_str, agent_id=agent_id).set(metrics.cpu_usage)
        self.agent_active.labels(agent_type=type_str, agent_id=agent_id).set(1 if metrics.active_time > 0 else 0)

        # Calculate heartbeat age
        heartbeat_age = time.time() - metrics.last_heartbeat
        self.agent_heartbeat_age.labels(agent_type=type_str, agent_id=agent_id).set(heartbeat_age)

        # Store state
        self.agent_states[agent_id] = metrics

    def update_swarm_metrics(self, metrics: SwarmMetrics):
        """Update global swarm metrics."""
        self.swarm_total_agents.set(metrics.total_agents)
        self.swarm_active_agents.set(metrics.active_agents)
        self.swarm_tasks_queued.set(metrics.tasks_queued)
        self.swarm_tasks_processing.set(metrics.tasks_processing)
        self.swarm_memory_usage.set(metrics.memory_used)
        self.swarm_memory_limit.set(metrics.memory_limit)
        self.swarm_efficiency.set(metrics.swarm_efficiency)

        coordination_age = time.time() - metrics.last_coordination
        self.swarm_coordination_time.set(coordination_age)

        self.swarm_state = metrics

    def record_task_completion(self, agent_type: AgentType, task_type: str, duration: float, success: bool):
        """Record task completion metrics."""
        self.agent_task_duration.labels(
            agent_type=agent_type.value,
            task_type=task_type
        ).observe(duration)

        if not success:
            # This would be handled by update_agent_metrics for failures
            pass

    def record_experience_retrieval(self, agent_type: str, retrieval_type: str):
        """Record experience retrieval from MNEMONIC memory."""
        self.elite_experience_retrieval.labels(
            agent_type=agent_type,
            retrieval_type=retrieval_type
        ).inc()

    def record_experience_storage(self, agent_type: str, fitness_score: float):
        """Record experience storage in MNEMONIC memory."""
        fitness_bucket = f"{int(fitness_score * 10) * 10}"  # Bucket by 10% increments
        self.elite_experience_stored.labels(
            agent_type=agent_type,
            fitness_score=fitness_bucket
        ).inc()

    def update_memory_hit_rate(self, agent_type: str, hit_rate: float):
        """Update memory hit rate for an agent type."""
        self.elite_memory_hit_rate.labels(agent_type=agent_type).set(hit_rate)

    async def metrics_handler(self, request: web.Request) -> web.Response:
        """HTTP handler for Prometheus metrics endpoint."""
        try:
            output = generate_latest(self.registry)
            return web.Response(
                body=output,
                content_type=CONTENT_TYPE_LATEST
            )
        except Exception as e:
            logger.error("Failed to generate metrics", error=str(e))
            return web.Response(status=500, text="Internal server error")

    async def health_handler(self, request: web.Request) -> web.Response:
        """Health check endpoint."""
        return web.json_response({
            "status": "healthy",
            "timestamp": time.time(),
            "agents_tracked": len(self.agent_states),
            "swarm_efficiency": self.swarm_state.swarm_efficiency
        })

    def create_app(self) -> web.Application:
        """Create aiohttp application with metrics endpoints."""
        app = web.Application()
        app.router.add_get('/metrics', self.metrics_handler)
        app.router.add_get('/health', self.health_handler)
        return app

    async def start_server(self, host: str = '0.0.0.0', port: int = 8000):
        """Start the metrics server."""
        app = self.create_app()
        runner = web.AppRunner(app)
        await runner.setup()

        site = web.TCPSite(runner, host, port)
        await site.start()

        logger.info("Metrics server started", host=host, port=port)

        # Keep the server running
        try:
            while True:
                await asyncio.sleep(1)
        except KeyboardInterrupt:
            logger.info("Shutting down metrics server")
            await runner.cleanup()


# Global metrics exporter instance
metrics_exporter = PhantomMetricsExporter()


def get_metrics_exporter() -> PhantomMetricsExporter:
    """Get the global metrics exporter instance."""
    return metrics_exporter


if __name__ == "__main__":
    # Example usage
    import sys

    async def main():
        exporter = get_metrics_exporter()

        # Example agent metrics update
        agent_metrics = AgentMetrics(
            tasks_completed=5,
            memory_usage=128 * 1024 * 1024,  # 128MB
            cpu_usage=15.5,
            active_time=3600.0,
            last_heartbeat=time.time()
        )
        exporter.update_agent_metrics("agent-001", AgentType.APEX, agent_metrics)

        # Example swarm metrics update
        swarm_metrics = SwarmMetrics(
            total_agents=10,
            active_agents=8,
            tasks_queued=3,
            memory_used=1024 * 1024 * 1024,  # 1GB
            memory_limit=2048 * 1024 * 1024,  # 2GB
            swarm_efficiency=0.92,
            last_coordination=time.time()
        )
        exporter.update_swarm_metrics(swarm_metrics)

        # Start server
        await exporter.start_server(port=int(sys.argv[1]) if len(sys.argv) > 1 else 8000)

    asyncio.run(main())