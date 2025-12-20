"""
Threat Engine Integration
==========================
Python interface to the Rust ThreatEngine for agent swarm integration.

Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

import asyncio
import aiohttp
import json
import structlog
from typing import Dict, Any, List, Optional
from datetime import datetime, timedelta

logger = structlog.get_logger(__name__)


class ThreatEngineClient:
    """Client for communicating with the Rust ThreatEngine via REST API."""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.session: Optional[aiohttp.ClientSession] = None

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def analyze_packet(self, packet_data: bytes, source_ip: Optional[str] = None) -> Dict[str, Any]:
        """Analyze a packet for threats."""
        if not self.session:
            raise RuntimeError("Client not initialized. Use async context manager.")

        payload = {
            "packet_data": list(packet_data),
            "source_ip": source_ip
        }

        try:
            async with self.session.post(
                f"{self.base_url}/threat/analyze",
                json=payload,
                timeout=aiohttp.ClientTimeout(total=5.0)
            ) as response:
                if response.status == 200:
                    result = await response.json()
                    return result
                else:
                    logger.error("threat_analysis_failed", status=response.status)
                    return {"threat_detected": False, "error": f"HTTP {response.status}"}
        except Exception as e:
            logger.error("threat_analysis_error", error=str(e))
            return {"threat_detected": False, "error": str(e)}

    async def get_metrics(self) -> Dict[str, Any]:
        """Get performance metrics from the threat engine."""
        if not self.session:
            raise RuntimeError("Client not initialized. Use async context manager.")

        try:
            async with self.session.get(
                f"{self.base_url}/threat/metrics",
                timeout=aiohttp.ClientTimeout(total=5.0)
            ) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    logger.error("metrics_fetch_failed", status=response.status)
                    return {}
        except Exception as e:
            logger.error("metrics_fetch_error", error=str(e))
            return {}

    async def update_signatures(self, signatures: List[Dict[str, Any]]) -> bool:
        """Update threat signatures."""
        if not self.session:
            raise RuntimeError("Client not initialized. Use async context manager.")

        try:
            async with self.session.post(
                f"{self.base_url}/threat/signatures",
                json={"signatures": signatures},
                timeout=aiohttp.ClientTimeout(total=10.0)
            ) as response:
                if response.status == 200:
                    result = await response.json()
                    return result.get("success", False)
                else:
                    logger.error("signature_update_failed", status=response.status)
                    return False
        except Exception as e:
            logger.error("signature_update_error", error=str(e))
            return False


class ThreatIntelligenceHub:
    """Central hub for threat intelligence sharing between agents."""

    def __init__(self):
        self.shared_intel: Dict[str, Dict[str, Any]] = {}
        self.agent_subscriptions: Dict[str, List[str]] = {}

    def share_intelligence(self, agent_id: str, intel: Dict[str, Any]) -> str:
        """Share threat intelligence from an agent."""
        intel_id = f"{agent_id}_{datetime.now().isoformat()}"
        intel["shared_by"] = agent_id
        intel["shared_at"] = datetime.now().isoformat()
        intel["id"] = intel_id

        self.shared_intel[intel_id] = intel
        logger.info("threat_intel_shared", intel_id=intel_id, agent=agent_id)

        return intel_id

    def get_intelligence(self, agent_id: str, since: Optional[datetime] = None) -> List[Dict[str, Any]]:
        """Get threat intelligence for an agent."""
        relevant_intel = []

        for intel_id, intel in self.shared_intel.items():
            # Filter by time if specified
            if since:
                shared_at = datetime.fromisoformat(intel["shared_at"])
                if shared_at < since:
                    continue

            # Filter out intel from the same agent (agents don't need their own intel)
            if intel.get("shared_by") != agent_id:
                relevant_intel.append(intel)

        return relevant_intel

    def subscribe_agent(self, agent_id: str, intel_types: List[str]):
        """Subscribe agent to specific intelligence types."""
        self.agent_subscriptions[agent_id] = intel_types

    def cleanup_old_intel(self, max_age_hours: int = 24):
        """Clean up old threat intelligence."""
        cutoff = datetime.now() - timedelta(hours=max_age_hours)
        to_remove = []

        for intel_id, intel in self.shared_intel.items():
            shared_at = datetime.fromisoformat(intel["shared_at"])
            if shared_at < cutoff:
                to_remove.append(intel_id)

        for intel_id in to_remove:
            del self.shared_intel[intel_id]

        if to_remove:
            logger.info("cleaned_old_intel", count=len(to_remove))


class IntegratedThreatSystem:
    """Integrated threat detection system combining Rust engine and Python agents."""

    def __init__(self, threat_engine_url: str = "http://localhost:8080"):
        self.threat_client = ThreatEngineClient(threat_engine_url)
        self.intel_hub = ThreatIntelligenceHub()
        self.active_scans = set()
        self.last_health_check = datetime.now()

    async def initialize(self):
        """Initialize the integrated threat system."""
        try:
            async with self.threat_client:
                # Test connection
                metrics = await self.threat_client.get_metrics()
                if metrics:
                    logger.info("threat_engine_connected", metrics=metrics)
                    return True
                else:
                    logger.error("threat_engine_connection_failed")
                    return False
        except Exception as e:
            logger.error("threat_system_init_failed", error=str(e))
            return False

    async def analyze_traffic_batch(self, packets: List[bytes], source_ips: List[str]) -> List[Dict[str, Any]]:
        """Analyze a batch of packets for threats."""
        results = []

        async with self.threat_client:
            for packet, source_ip in zip(packets, source_ips):
                result = await self.threat_client.analyze_packet(packet, source_ip)
                results.append(result)

                # Share high-confidence threats
                if result.get("threat_detected") and result.get("threat_result", {}).get("confidence", 0) > 0.8:
                    self.intel_hub.share_intelligence("integrated_system", result["threat_result"])

        return results

    async def perform_anomaly_scan(self, traffic_data: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Perform anomaly detection on traffic patterns."""
        # This would integrate with the FORTRESS agent's anomaly detection
        # For now, return a basic analysis

        packet_sizes = [len(p.get("data", b"")) for p in traffic_data]
        timestamps = [datetime.fromisoformat(p.get("timestamp", datetime.now().isoformat())) for p in traffic_data]

        analysis = {
            "total_packets": len(traffic_data),
            "avg_packet_size": sum(packet_sizes) / len(packet_sizes) if packet_sizes else 0,
            "time_span_seconds": (max(timestamps) - min(timestamps)).total_seconds() if timestamps else 0,
            "packets_per_second": len(traffic_data) / max((max(timestamps) - min(timestamps)).total_seconds(), 1),
            "anomalies_detected": [],  # Would be filled by FORTRESS agent
            "scan_timestamp": datetime.now().isoformat()
        }

        return analysis

    async def update_threat_signatures(self, new_signatures: List[Dict[str, Any]]) -> bool:
        """Update threat signatures in the Rust engine."""
        async with self.threat_client:
            return await self.threat_client.update_signatures(new_signatures)

    async def get_system_status(self) -> Dict[str, Any]:
        """Get overall system status."""
        async with self.threat_client:
            metrics = await self.threat_client.get_metrics()

        intel_count = len(self.intel_hub.shared_intel)
        active_scans = len(self.active_scans)

        return {
            "threat_engine_metrics": metrics,
            "shared_intelligence_count": intel_count,
            "active_scans": active_scans,
            "last_health_check": self.last_health_check.isoformat(),
            "system_status": "operational" if metrics else "degraded"
        }

    async def share_agent_intelligence(self, agent_id: str, intel: Dict[str, Any]) -> str:
        """Allow agents to share intelligence."""
        return self.intel_hub.share_intelligence(agent_id, intel)

    async def get_agent_intelligence(self, agent_id: str) -> List[Dict[str, Any]]:
        """Get intelligence for a specific agent."""
        since = datetime.now() - timedelta(hours=1)  # Last hour
        return self.intel_hub.get_intelligence(agent_id, since)

    async def health_check(self) -> bool:
        """Perform health check on the threat system."""
        try:
            async with self.threat_client:
                metrics = await self.threat_client.get_metrics()
                self.last_health_check = datetime.now()
                return bool(metrics)
        except Exception as e:
            logger.error("health_check_failed", error=str(e))
            return False