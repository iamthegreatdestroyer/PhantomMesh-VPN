#!/usr/bin/env python3
"""
PhantomMesh Dynamic Agent Service Discovery
===========================================

Automatically registers active agents with Prometheus service discovery.
Updates the dynamic_agents.yml file with current agent endpoints.

Copyright © 2025 Stephen Bilodeau. All rights reserved.
Licensed under GPL-3.0 with proprietary agent clauses.
"""

import asyncio
import time
import yaml
import os
from typing import Dict, List
from dataclasses import dataclass

import structlog
from aiohttp import web

# Configure structured logging
import logging.config
import yaml
import os

# Load logging configuration
log_config_path = "/etc/prometheus/logging.yml"
if os.path.exists(log_config_path):
    with open(log_config_path, 'r') as f:
        log_config = yaml.safe_load(f)
        logging.config.dictConfig(log_config)

logger = structlog.get_logger(__name__)


@dataclass
class AgentEndpoint:
    """Represents a dynamically discovered agent endpoint."""
    agent_id: str
    agent_type: str
    host: str
    port: int
    last_seen: float
    health_status: str = "unknown"


class AgentDiscoveryService:
    """Service for dynamic agent registration and health monitoring."""

    def __init__(self, discovery_file: str = "/etc/prometheus/dynamic_agents.yml"):
        self.discovery_file = discovery_file
        self.agents: Dict[str, AgentEndpoint] = {}
        self._running = False

    def update_discovery_file(self) -> None:
        """Update the Prometheus service discovery file with current agents."""
        # Filter to only healthy agents seen within last 5 minutes
        cutoff_time = time.time() - 300  # 5 minutes
        active_agents = [
            agent for agent in self.agents.values()
            if agent.last_seen > cutoff_time and agent.health_status == "healthy"
        ]

        # Create Prometheus service discovery format
        discovery_config = {
            "targets": [f"{agent.host}:{agent.port}" for agent in active_agents],
            "labels": {
                "job": "phantom-dynamic-agents",
                "service": "agent-sub-swarm"
            }
        }

        # Write to file
        os.makedirs(os.path.dirname(self.discovery_file), exist_ok=True)
        with open(self.discovery_file, 'w') as f:
            yaml.dump([discovery_config], f, default_flow_style=False)

        logger.info("updated_discovery_file",
                   active_agents=len(active_agents),
                   total_registered=len(self.agents))

    async def register_agent(self, request: web.Request) -> web.Response:
        """HTTP endpoint for agents to register themselves."""
        try:
            data = await request.json()

            agent = AgentEndpoint(
                agent_id=data['agent_id'],
                agent_type=data['agent_type'],
                host=data.get('host', request.remote),
                port=data['port'],
                last_seen=time.time(),
                health_status="healthy"
            )

            self.agents[agent.agent_id] = agent
            self.update_discovery_file()

            logger.info("agent_registered",
                       agent_id=agent.agent_id,
                       agent_type=agent.agent_type,
                       endpoint=f"{agent.host}:{agent.port}")

            return web.json_response({"status": "registered"})

        except Exception as e:
            logger.error("agent_registration_failed", error=str(e))
            return web.json_response({"error": str(e)}, status=400)

    async def unregister_agent(self, request: web.Request) -> web.Response:
        """HTTP endpoint for agents to unregister themselves."""
        try:
            agent_id = request.match_info['agent_id']

            if agent_id in self.agents:
                del self.agents[agent_id]
                self.update_discovery_file()
                logger.info("agent_unregistered", agent_id=agent_id)
                return web.json_response({"status": "unregistered"})
            else:
                return web.json_response({"error": "agent not found"}, status=404)

        except Exception as e:
            logger.error("agent_unregistration_failed", error=str(e))
            return web.json_response({"error": str(e)}, status=400)

    async def health_check_loop(self) -> None:
        """Periodically check health of registered agents."""
        while self._running:
            try:
                async with ClientSession() as session:
                    agents_to_remove = []

                    for agent_id, agent in self.agents.items():
                        try:
                            # Check agent health
                            async with session.get(f"http://{agent.host}:{agent.port}/health",
                                                 timeout=5) as response:
                                if response.status == 200:
                                    agent.health_status = "healthy"
                                    agent.last_seen = time.time()
                                else:
                                    agent.health_status = "unhealthy"
                        except Exception:
                            agent.health_status = "unreachable"

                            # Mark for removal if not seen for 10 minutes
                            if time.time() - agent.last_seen > 600:
                                agents_to_remove.append(agent_id)

                    # Remove stale agents
                    for agent_id in agents_to_remove:
                        logger.warning("removing_stale_agent", agent_id=agent_id)
                        del self.agents[agent_id]

                    self.update_discovery_file()

            except Exception as e:
                logger.error("health_check_error", error=str(e))

            await asyncio.sleep(60)  # Check every minute

    async def list_agents(self, request: web.Request) -> web.Response:
        """Return list of currently registered agents."""
        agents_list = [
            {
                "agent_id": agent.agent_id,
                "agent_type": agent.agent_type,
                "endpoint": f"{agent.host}:{agent.port}",
                "last_seen": agent.last_seen,
                "health_status": agent.health_status
            }
            for agent in self.agents.values()
        ]

        return web.json_response({
            "agents": agents_list,
            "total_count": len(agents_list)
        })

    def create_app(self) -> web.Application:
        """Create aiohttp application for the discovery service."""
        app = web.Application()
        app.router.add_post('/register', self.register_agent)
        app.router.add_delete('/unregister/{agent_id}', self.unregister_agent)
        app.router.add_get('/agents', self.list_agents)
        app.router.add_get('/health', lambda r: web.json_response({"status": "healthy"}))
        return app

    async def start_server(self, host: str = '0.0.0.0', port: int = 8081):
        """Start the discovery service."""
        self._running = True

        app = self.create_app()
        runner = web.AppRunner(app)
        await runner.setup()

        site = web.TCPSite(runner, host, port)
        await site.start()

        logger.info("agent_discovery_service_started", host=host, port=port)

        # Start health check loop
        health_task = asyncio.create_task(self.health_check_loop())

        try:
            # Keep running
            while self._running:
                await asyncio.sleep(1)
        except KeyboardInterrupt:
            logger.info("shutting_down_discovery_service")
        finally:
            self._running = False
            health_task.cancel()
            await runner.cleanup()


# Global discovery service instance
discovery_service = AgentDiscoveryService()


def get_discovery_service() -> AgentDiscoveryService:
    """Get the global discovery service instance."""
    return discovery_service


if __name__ == "__main__":
    # Start the discovery service
    asyncio.run(discovery_service.start_server())