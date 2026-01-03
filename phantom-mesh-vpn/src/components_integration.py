"""
Phase P1-004 Components Integration Layer

Provides integrated interface for:
- Component 1: Analytics Data Ingestion (Stream processing)
- Component 2: Advanced Analytics Processing (Real-time analysis)
- Component 3: Time-Series Database Interface (Data storage)
- Component 4: Analytics API Gateway (REST/WebSocket API)
- Component 5: Web Dashboard (React visualization)
- Component 6: Reporting Engine (Automated reports)

This module orchestrates all components into a cohesive system.
"""

from typing import Dict, Any, List, Optional, Callable
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import Enum
import asyncio
import logging
from abc import ABC, abstractmethod

logger = logging.getLogger(__name__)

# ============================================================================
# INTEGRATION MODELS
# ============================================================================


@dataclass
class SystemMetrics:
    """System-wide metrics snapshot."""

    timestamp: datetime
    total_threats_detected: int
    critical_threats: int
    avg_response_time_ms: float
    network_uptime_percent: float
    threat_detection_rate: float
    api_requests_per_second: float
    active_sessions: int
    database_query_latency_ms: float
    cache_hit_rate: float


@dataclass
class DashboardMetrics:
    """Dashboard-specific metrics."""

    threats: List[Dict[str, Any]]
    nodes: List[Dict[str, Any]]
    metrics: List[Dict[str, Any]]
    alerts: List[Dict[str, Any]]
    system_status: str
    last_update: datetime


# ============================================================================
# COMPONENT ORCHESTRATOR
# ============================================================================


class ComponentOrchestrator:
    """
    Orchestrates all Phase P1-004 components.

    Manages:
    - Data flow between components
    - Component lifecycle
    - Error handling and recovery
    - Performance monitoring
    """

    def __init__(
        self,
        ingestion_engine: Any,  # StreamDeduplicator + EventEnricher
        analytics_engine: Any,  # RealTimeAggregator + AnomalyDetector
        tsdb_interface: Any,  # InfluxDBAdapter / TimescaleDBAdapter
        api_gateway: Any,  # APIGateway
        reporting_service: Any,  # ReportingService
    ):
        self.ingestion = ingestion_engine
        self.analytics = analytics_engine
        self.tsdb = tsdb_interface
        self.api_gateway = api_gateway
        self.reporting = reporting_service

        self._is_running = False
        self._component_health = {}
        self._performance_stats = {}
        self._error_handlers: List[Callable] = []

    async def initialize(self):
        """Initialize all components."""
        logger.info("Initializing Phase P1-004 components...")

        try:
            # Initialize in dependency order
            await self.tsdb.connect()
            self._component_health["tsdb"] = "healthy"

            await self.ingestion.start()
            self._component_health["ingestion"] = "healthy"

            await self.analytics.start()
            self._component_health["analytics"] = "healthy"

            await self.api_gateway.start()
            self._component_health["api_gateway"] = "healthy"

            await self.reporting.start()
            self._component_health["reporting"] = "healthy"

            logger.info("All components initialized successfully")
            return True

        except Exception as e:
            logger.error(f"Component initialization failed: {e}")
            return False

    async def shutdown(self):
        """Shutdown all components gracefully."""
        logger.info("Shutting down Phase P1-004 components...")

        tasks = [
            self.reporting.stop(),
            self.api_gateway.stop(),
            self.analytics.stop(),
            self.ingestion.stop(),
            self.tsdb.disconnect(),
        ]

        results = await asyncio.gather(*tasks, return_exceptions=True)

        for result in results:
            if isinstance(result, Exception):
                logger.error(f"Shutdown error: {result}")

        logger.info("All components shut down")

    async def start_data_pipeline(self):
        """Start the data processing pipeline."""

        self._is_running = True

        while self._is_running:
            try:
                # Step 1: Ingest raw data
                raw_events = await self.ingestion.get_pending_events()

                if raw_events:
                    logger.debug(f"Ingested {len(raw_events)} events")

                    # Step 2: Process through analytics
                    analyzed_data = await self.analytics.process_events(
                        raw_events
                    )

                    # Step 3: Store in time-series database
                    await self.tsdb.write_metrics(analyzed_data["metrics"])
                    await self.tsdb.write_threats(analyzed_data["threats"])

                    # Step 4: Broadcast to API gateway (for WebSocket)
                    await self.api_gateway.broadcast_update(analyzed_data)

                    # Step 5: Check if reporting is needed
                    if self._should_generate_scheduled_reports():
                        logger.info("Triggering scheduled report generation")

                # Check component health
                await self._check_component_health()

                # Brief pause to prevent CPU spinning
                await asyncio.sleep(0.1)

            except Exception as e:
                logger.error(f"Pipeline error: {e}")
                await self._handle_pipeline_error(e)

                # Back off briefly on error
                await asyncio.sleep(1)

    async def get_dashboard_data(
        self,
        time_range: tuple[datetime, datetime],
    ) -> DashboardMetrics:
        """Get aggregated data for dashboard display."""

        try:
            # Query all data sources in parallel
            threats_task = self.tsdb.query_threats(
                time_range[0],
                time_range[1],
            )
            nodes_task = self.tsdb.query_nodes(time_range[0], time_range[1])
            metrics_task = self.tsdb.query_metrics(
                time_range[0],
                time_range[1],
            )
            alerts_task = self.analytics.get_recent_alerts()

            (
                threats,
                nodes,
                metrics,
                alerts,
            ) = await asyncio.gather(
                threats_task,
                nodes_task,
                metrics_task,
                alerts_task,
            )

            return DashboardMetrics(
                threats=threats,
                nodes=nodes,
                metrics=metrics,
                alerts=alerts,
                system_status=self._get_system_status(),
                last_update=datetime.utcnow(),
            )

        except Exception as e:
            logger.error(f"Error retrieving dashboard data: {e}")
            raise

    def _get_system_status(self) -> str:
        """Get overall system status."""

        healthy = sum(
            1
            for status in self._component_health.values()
            if status == "healthy"
        )
        total = len(self._component_health)

        if healthy == total:
            return "HEALTHY"
        elif healthy >= total * 0.75:
            return "DEGRADED"
        else:
            return "CRITICAL"

    async def _check_component_health(self):
        """Check health of all components."""

        health_checks = {
            "tsdb": self.tsdb.health_check(),
            "ingestion": self.ingestion.health_check(),
            "analytics": self.analytics.health_check(),
            "api_gateway": self.api_gateway.health_check(),
            "reporting": self.reporting.health_check() if hasattr(self.reporting, 'health_check') else None,
        }

        results = await asyncio.gather(
            *[v for v in health_checks.values() if v],
            return_exceptions=True,
        )

        for name, result in zip(health_checks.keys(), results):
            if isinstance(result, Exception):
                self._component_health[name] = "unhealthy"
                logger.warning(f"Component {name} unhealthy: {result}")
            else:
                self._component_health[name] = (
                    "healthy" if result else "degraded"
                )

    def _should_generate_scheduled_reports(self) -> bool:
        """Check if scheduled reports should run."""
        # Simplified: could implement actual scheduling logic
        return False

    async def _handle_pipeline_error(self, error: Exception):
        """Handle pipeline errors."""

        for handler in self._error_handlers:
            try:
                await handler(error)
            except Exception as e:
                logger.error(f"Error handler failed: {e}")

    def register_error_handler(self, handler: Callable):
        """Register error handler callback."""
        self._error_handlers.append(handler)


# ============================================================================
# UNIFIED ANALYTICS FACADE
# ============================================================================


class UnifiedAnalyticsFacade:
    """
    Unified interface for all analytics functionality.

    Provides high-level methods for:
    - Threat intelligence queries
    - Metrics analysis
    - Report generation
    - Real-time streaming
    """

    def __init__(self, orchestrator: ComponentOrchestrator):
        self.orchestrator = orchestrator

    async def query_threats(
        self,
        start_time: datetime,
        end_time: datetime,
        severity_filter: Optional[str] = None,
        region_filter: Optional[str] = None,
    ) -> List[Dict[str, Any]]:
        """
        Query threats with advanced filtering.

        Supports filtering by:
        - Severity (CRITICAL, HIGH, MEDIUM, LOW)
        - Region (us-east, eu-west, etc.)
        - Time range
        """

        filters = {}
        if severity_filter:
            filters["severity"] = severity_filter
        if region_filter:
            filters["region"] = region_filter

        return await self.orchestrator.tsdb.query_threats(
            start_time,
            end_time,
            filters=filters,
        )

    async def get_threat_summary(
        self,
        hours: int = 24,
    ) -> Dict[str, Any]:
        """Get summary of recent threats."""

        end_time = datetime.utcnow()
        start_time = end_time - timedelta(hours=hours)

        threats = await self.query_threats(start_time, end_time)

        severity_counts = {
            "CRITICAL": 0,
            "HIGH": 0,
            "MEDIUM": 0,
            "LOW": 0,
        }

        for threat in threats:
            severity = threat.get("severity", "UNKNOWN")
            if severity in severity_counts:
                severity_counts[severity] += 1

        return {
            "total_threats": len(threats),
            "severity_breakdown": severity_counts,
            "critical_percentage": (
                severity_counts["CRITICAL"] / len(threats) * 100
                if threats
                else 0
            ),
            "time_range": {
                "start": start_time.isoformat(),
                "end": end_time.isoformat(),
            },
        }

    async def get_metrics_summary(
        self,
        metric_names: List[str],
        hours: int = 1,
    ) -> Dict[str, Dict[str, float]]:
        """Get summary statistics for metrics."""

        end_time = datetime.utcnow()
        start_time = end_time - timedelta(hours=hours)

        metrics = await self.orchestrator.tsdb.query_metrics(
            start_time,
            end_time,
            metric_names=metric_names,
        )

        summary = {}
        for metric_name, values in metrics.items():
            if values:
                summary[metric_name] = {
                    "min": min(values),
                    "max": max(values),
                    "avg": sum(values) / len(values),
                    "latest": values[-1],
                }

        return summary

    async def generate_report_async(
        self,
        report_type: str,
        export_format: str,
        hours: int = 24,
    ) -> bytes:
        """
        Generate a report asynchronously.

        Supports:
        - report_type: 'executive', 'technical', 'forensic'
        - export_format: 'pdf', 'excel', 'json', 'html'
        """

        from phantom_mesh_vpn.src.reporting.reporting_engine import (
            ReportConfig,
            ReportType,
            ExportFormat,
        )

        end_time = datetime.utcnow()
        start_time = end_time - timedelta(hours=hours)

        config = ReportConfig(
            name=f"{report_type}_report_{datetime.utcnow().isoformat()}",
            report_type=ReportType[report_type.upper()],
            export_format=ExportFormat[export_format.upper()],
        )

        # Generate report
        report_data = await self.orchestrator.reporting.generate_report(
            config,
            (start_time, end_time),
        )

        # Export to requested format
        return await self.orchestrator.reporting.export_report(
            report_data,
            config.export_format,
        )

    async def stream_live_metrics(
        self,
        callback: Callable[[Dict[str, Any]], None],
    ):
        """
        Stream live metrics as they arrive.

        Callback will be invoked with metric updates.
        """

        # Register with API gateway for WebSocket updates
        await self.orchestrator.api_gateway.register_live_stream(callback)

    async def get_system_health(self) -> Dict[str, Any]:
        """Get overall system health snapshot."""

        return {
            "components": self.orchestrator._component_health,
            "status": self.orchestrator._get_system_status(),
            "uptime_hours": 0,  # Would calculate from start time
            "timestamp": datetime.utcnow().isoformat(),
        }


# ============================================================================
# METRICS COLLECTION & EXPORT
# ============================================================================


class MetricsCollector:
    """Collects and exports performance metrics."""

    def __init__(self):
        self.metrics = {}

    def record_metric(
        self,
        name: str,
        value: float,
        tags: Optional[Dict[str, str]] = None,
    ):
        """Record a metric."""
        key = f"{name}:{tags}" if tags else name

        if key not in self.metrics:
            self.metrics[key] = []

        self.metrics[key].append({
            "value": value,
            "timestamp": datetime.utcnow(),
            "tags": tags,
        })

    def get_summary(self) -> Dict[str, Any]:
        """Get metrics summary."""
        summary = {}

        for key, values in self.metrics.items():
            if values:
                metric_values = [v["value"] for v in values]
                summary[key] = {
                    "min": min(metric_values),
                    "max": max(metric_values),
                    "avg": sum(metric_values) / len(metric_values),
                    "count": len(metric_values),
                    "latest": values[-1]["value"],
                }

        return summary


# ============================================================================
# EXPORTS
# ============================================================================

__all__ = [
    "ComponentOrchestrator",
    "UnifiedAnalyticsFacade",
    "MetricsCollector",
    "SystemMetrics",
    "DashboardMetrics",
]
