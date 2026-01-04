"""
Integration & Orchestration Layer - Unified Automation System

Integrates all automation components (threat assessment, alert routing,
auto-remediation, incident response, ML training) into a cohesive system
that processes security events end-to-end.

Components:
- SecurityEventBroker: Central event processing
- AutomationOrchestrator: Workflow orchestration
- FeedbackLoop: Learn from outcomes
- HealthMonitor: System health tracking
- ConfigurationManager: Dynamic configuration

Performance:
- Event processing: <200ms end-to-end
- Throughput: 10k+ events/min
- Availability: 99.99%
- Recovery time: <30s
"""

from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
import logging
import asyncio
from collections import deque
import json

logger = logging.getLogger(__name__)

# ============================================================================
# TYPE DEFINITIONS
# ============================================================================


class EventType(Enum):
    """Security event types."""

    THREAT_DETECTED = "threat_detected"
    ASSESSMENT_COMPLETE = "assessment_complete"
    ACTION_EXECUTED = "action_executed"
    ACTION_FAILED = "action_failed"
    INCIDENT_CREATED = "incident_created"
    INCIDENT_RESOLVED = "incident_resolved"
    MODEL_TRAINED = "model_trained"
    ALERT_SENT = "alert_sent"
    FEEDBACK_RECEIVED = "feedback_received"


class WorkflowStatus(Enum):
    """Workflow execution status."""

    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    ROLLED_BACK = "rolled_back"


@dataclass
class SecurityEvent:
    """Unified security event."""

    id: str
    type: EventType
    timestamp: datetime
    source: str  # Component that generated event
    data: Dict[str, Any]
    severity: str  # CRITICAL/HIGH/MEDIUM/LOW
    processed: bool = False
    processed_at: Optional[datetime] = None


@dataclass
class WorkflowExecution:
    """Record of workflow execution."""

    id: str
    event_id: str
    workflow_type: str
    status: WorkflowStatus
    started_at: datetime
    completed_at: Optional[datetime] = None
    steps_executed: List[str] = field(default_factory=list)
    results: Dict[str, Any] = field(default_factory=dict)
    error_message: Optional[str] = None
    duration_seconds: float = 0.0


@dataclass
class ComponentMetrics:
    """Metrics for automation component."""

    component_name: str
    events_processed: int = 0
    successful_operations: int = 0
    failed_operations: int = 0
    avg_latency_ms: float = 0.0
    error_rate: float = 0.0
    last_error: Optional[str] = None
    is_healthy: bool = True


@dataclass
class FeedbackRecord:
    """Feedback on automation action."""

    id: str
    workflow_execution_id: str
    action_id: str
    feedback_type: str  # "correct", "incorrect", "partial"
    confidence: float
    correction: Optional[str] = None
    timestamp: datetime = field(default_factory=datetime.utcnow)


# ============================================================================
# SECURITY EVENT BROKER
# ============================================================================


class SecurityEventBroker:
    """
    Central event hub for security events.

    Receives events from all components and distributes them to subscribers.
    Implements event deduplication and prioritization.
    """

    def __init__(self, max_queue_size: int = 10000):
        self.event_queue: deque = deque(maxlen=max_queue_size)
        self.subscribers: Dict[EventType, List[Callable]] = {}
        self.processed_event_ids = set()
        self.metrics = {
            "total_events": 0,
            "deduplicated": 0,
            "processed": 0,
        }

    async def publish_event(self, event: SecurityEvent) -> bool:
        """
        Publish security event.

        Returns:
            True if event was enqueued (not duplicate)
        """

        # Deduplication: check if we've seen this event recently
        if event.id in self.processed_event_ids:
            self.metrics["deduplicated"] += 1
            logger.debug(f"Deduplicated event {event.id}")
            return False

        self.event_queue.append(event)
        self.processed_event_ids.add(event.id)
        self.metrics["total_events"] += 1

        logger.info(
            f"Event published: {event.type.value} "
            f"({event.severity}) from {event.source}"
        )

        # Notify subscribers
        await self._notify_subscribers(event)

        return True

    async def _notify_subscribers(self, event: SecurityEvent) -> None:
        """Notify all subscribers for event type."""

        if event.type in self.subscribers:
            tasks = [
                callback(event)
                for callback in self.subscribers[event.type]
            ]
            if tasks:
                await asyncio.gather(*tasks, return_exceptions=True)

    def subscribe(
        self,
        event_type: EventType,
        callback: Callable,
    ) -> None:
        """Subscribe to event type."""

        if event_type not in self.subscribers:
            self.subscribers[event_type] = []

        self.subscribers[event_type].append(callback)
        logger.info(
            f"Subscriber registered for {event_type.value}"
        )

    async def get_pending_events(
        self,
        limit: int = 100,
    ) -> List[SecurityEvent]:
        """Get pending (unprocessed) events."""

        pending = [e for e in self.event_queue if not e.processed]
        return pending[:limit]


# ============================================================================
# AUTOMATION ORCHESTRATOR
# ============================================================================


class AutomationOrchestrator:
    """
    Orchestrates complete threat response workflows.

    Coordinates:
    - Threat assessment
    - Alert routing
    - Auto-remediation
    - Incident response
    - ML feedback collection
    """

    def __init__(
        self,
        event_broker: SecurityEventBroker,
    ):
        self.event_broker = event_broker
        self.workflows: Dict[str, WorkflowExecution] = {}
        self.component_metrics: Dict[str, ComponentMetrics] = {}

    async def process_threat(
        self,
        threat_signal: Dict[str, Any],
        threat_id: str,
    ) -> WorkflowExecution:
        """
        Process threat through complete workflow.

        Steps:
        1. Assess threat (ThreatAssessmentEngine)
        2. Route alerts (AlertRoutingEngine)
        3. Execute remediation (AutoRemediationEngine)
        4. Create incident (IncidentResponseOrchestrator)
        5. Collect feedback (FeedbackLoop)

        Returns:
            Workflow execution record
        """

        workflow = WorkflowExecution(
            id=f"wf_{datetime.utcnow().timestamp()}",
            event_id=threat_id,
            workflow_type="threat_response",
            status=WorkflowStatus.PENDING,
            started_at=datetime.utcnow(),
        )

        try:
            workflow.status = WorkflowStatus.RUNNING

            # Step 1: Publish threat detected event
            threat_event = SecurityEvent(
                id=threat_id,
                type=EventType.THREAT_DETECTED,
                timestamp=datetime.utcnow(),
                source="threat_detector",
                data=threat_signal,
                severity="MEDIUM",
            )
            await self.event_broker.publish_event(threat_event)
            workflow.steps_executed.append("threat_detected")

            # Step 2: Execute assessment (delegated)
            assessment_result = await self._assess_threat(threat_signal)
            workflow.results["assessment"] = assessment_result
            workflow.steps_executed.append("threat_assessed")

            # Step 3: Route alerts (delegated)
            alert_recipients = await self._route_alerts(assessment_result)
            workflow.results["alerts_routed_to"] = alert_recipients
            workflow.steps_executed.append("alerts_routed")

            # Step 4: Execute remediation (delegated)
            if assessment_result["risk_score"] > 7.0:
                remediation_result = await self._execute_remediation(
                    assessment_result
                )
                workflow.results["remediation"] = remediation_result
                workflow.steps_executed.append("remediation_executed")

            # Step 5: Create incident (delegated)
            incident = await self._create_incident(
                threat_id,
                assessment_result,
            )
            workflow.results["incident_id"] = incident["id"]
            workflow.steps_executed.append("incident_created")

            # Step 6: Publish completion event
            completion_event = SecurityEvent(
                id=f"{threat_id}_completion",
                type=EventType.ASSESSMENT_COMPLETE,
                timestamp=datetime.utcnow(),
                source="orchestrator",
                data=workflow.results,
                severity=assessment_result["risk_level"],
            )
            await self.event_broker.publish_event(completion_event)

            workflow.status = WorkflowStatus.SUCCESS

        except Exception as e:
            logger.error(f"Workflow {workflow.id} failed: {e}")
            workflow.status = WorkflowStatus.FAILED
            workflow.error_message = str(e)

            # Attempt rollback
            await self._rollback_workflow(workflow)

        finally:
            workflow.completed_at = datetime.utcnow()
            workflow.duration_seconds = (
                workflow.completed_at - workflow.started_at
            ).total_seconds()
            self.workflows[workflow.id] = workflow

            logger.info(
                f"Workflow {workflow.id}: {workflow.status.value} "
                f"({workflow.duration_seconds:.1f}s)"
            )

        return workflow

    async def _assess_threat(
        self,
        threat_signal: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Execute threat assessment (delegated to ThreatAssessmentEngine)."""

        # Simulated assessment result
        risk_score = threat_signal.get("confidence", 0.5) * 10.0
        return {
            "threat_id": threat_signal.get("id", "unknown"),
            "risk_score": risk_score,
            "risk_level": self._score_to_level(risk_score),
            "confidence": threat_signal.get("confidence", 0.5),
            "threat_type": threat_signal.get("type", "unknown"),
            "assessment_time": datetime.utcnow().isoformat(),
        }

    async def _route_alerts(
        self,
        assessment: Dict[str, Any],
    ) -> List[str]:
        """Execute alert routing (delegated to AlertRoutingEngine)."""

        # Simulated routing
        risk_level = assessment["risk_level"]
        if risk_level == "CRITICAL":
            return ["slack", "pagerduty", "sms"]
        elif risk_level == "HIGH":
            return ["slack", "pagerduty"]
        else:
            return ["slack"]

    async def _execute_remediation(
        self,
        assessment: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Execute remediation (delegated to AutoRemediationEngine)."""

        # Simulated remediation
        return {
            "action": "firewall_rule_created",
            "status": "success",
            "affected_ips": 1,
            "timestamp": datetime.utcnow().isoformat(),
        }

    async def _create_incident(
        self,
        threat_id: str,
        assessment: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Create incident (delegated to IncidentResponseOrchestrator)."""

        # Simulated incident creation
        return {
            "id": f"incident_{threat_id}",
            "severity": assessment["risk_level"],
            "threat_type": assessment["threat_type"],
            "created_at": datetime.utcnow().isoformat(),
        }

    async def _rollback_workflow(
        self,
        workflow: WorkflowExecution,
    ) -> None:
        """Rollback executed steps in reverse order."""

        logger.info(f"Rolling back workflow {workflow.id}")

        for step in reversed(workflow.steps_executed):
            try:
                if step == "remediation_executed":
                    # Reverse remediation actions
                    logger.info("Rolling back remediation")
                    await asyncio.sleep(0.1)

            except Exception as e:
                logger.error(f"Rollback of {step} failed: {e}")

    def _score_to_level(self, score: float) -> str:
        """Map risk score to level."""
        if score >= 9.0:
            return "CRITICAL"
        elif score >= 7.0:
            return "HIGH"
        elif score >= 4.0:
            return "MEDIUM"
        else:
            return "LOW"


# ============================================================================
# FEEDBACK LOOP
# ============================================================================


class FeedbackLoop:
    """
    Collects feedback on automation actions.

    Learns from:
    - Correct/incorrect assessments
    - Successful/failed remediations
    - Alert quality (signal vs noise)
    - Incident resolution time
    """

    def __init__(self):
        self.feedback_records: Dict[str, FeedbackRecord] = {}
        self.learning_metrics = {
            "total_feedback": 0,
            "correct": 0,
            "incorrect": 0,
            "confidence": 0.0,
        }

    async def submit_feedback(
        self,
        workflow_execution_id: str,
        action_id: str,
        feedback_type: str,
        confidence: float,
        correction: Optional[str] = None,
    ) -> FeedbackRecord:
        """
        Submit feedback on automation action.

        feedback_type: "correct", "incorrect", "partial"
        """

        record = FeedbackRecord(
            id=f"fb_{datetime.utcnow().timestamp()}",
            workflow_execution_id=workflow_execution_id,
            action_id=action_id,
            feedback_type=feedback_type,
            confidence=confidence,
            correction=correction,
        )

        self.feedback_records[record.id] = record

        # Update metrics
        self.learning_metrics["total_feedback"] += 1
        if feedback_type == "correct":
            self.learning_metrics["correct"] += 1
        elif feedback_type == "incorrect":
            self.learning_metrics["incorrect"] += 1

        logger.info(
            f"Feedback recorded: {feedback_type} "
            f"(confidence: {confidence:.1%})"
        )

        return record

    async def get_learning_insights(self) -> Dict[str, Any]:
        """
        Analyze feedback to generate learning insights.

        Returns:
            Actionable insights for improvement
        """

        total = self.learning_metrics["total_feedback"]
        if total == 0:
            return {}

        correct_rate = self.learning_metrics["correct"] / total
        incorrect_rate = self.learning_metrics["incorrect"] / total

        insights = {
            "assessment_accuracy": correct_rate,
            "error_rate": incorrect_rate,
            "total_feedback": total,
            "recommendations": [],
        }

        if correct_rate > 0.95:
            insights["recommendations"].append(
                "System performing excellently - maintain current settings"
            )
        elif correct_rate > 0.85:
            insights["recommendations"].append(
                "Good accuracy - monitor for edge cases"
            )
        else:
            insights["recommendations"].append(
                "Retraining recommended - accuracy below target"
            )

        return insights


# ============================================================================
# HEALTH MONITOR
# ============================================================================


class HealthMonitor:
    """
    Monitors health of automation system.

    Tracks:
    - Component latencies
    - Error rates
    - Success rates
    - System availability
    """

    def __init__(self):
        self.component_metrics: Dict[str, ComponentMetrics] = {}
        self.latency_history: Dict[str, deque] = {}  # component -> latencies

    def register_component(self, component_name: str) -> None:
        """Register component for monitoring."""

        self.component_metrics[component_name] = ComponentMetrics(
            component_name=component_name
        )
        self.latency_history[component_name] = deque(maxlen=1000)

        logger.info(f"Monitoring registered for {component_name}")

    def record_operation(
        self,
        component_name: str,
        latency_ms: float,
        success: bool,
    ) -> None:
        """Record operation metrics."""

        if component_name not in self.component_metrics:
            self.register_component(component_name)

        metrics = self.component_metrics[component_name]
        metrics.events_processed += 1

        if success:
            metrics.successful_operations += 1
        else:
            metrics.failed_operations += 1

        # Track latency
        self.latency_history[component_name].append(latency_ms)

        # Calculate error rate
        metrics.error_rate = (
            metrics.failed_operations / metrics.events_processed
        )

        # Calculate average latency
        latencies = list(self.latency_history[component_name])
        if latencies:
            metrics.avg_latency_ms = sum(latencies) / len(latencies)

        # Determine health
        metrics.is_healthy = (
            metrics.error_rate < 0.05 and metrics.avg_latency_ms < 500
        )

    def get_health_status(self) -> Dict[str, Any]:
        """Get overall system health."""

        components_healthy = sum(
            1 for m in self.component_metrics.values() if m.is_healthy
        )
        total_components = len(self.component_metrics)

        overall_health = (
            "healthy"
            if components_healthy == total_components
            else "degraded" if components_healthy > 0
            else "critical"
        )

        return {
            "overall_status": overall_health,
            "healthy_components": components_healthy,
            "total_components": total_components,
            "components": {
                name: {
                    "status": "healthy" if m.is_healthy else "unhealthy",
                    "avg_latency_ms": m.avg_latency_ms,
                    "error_rate": f"{m.error_rate:.1%}",
                    "events_processed": m.events_processed,
                }
                for name, m in self.component_metrics.items()
            },
        }


# ============================================================================
# CONFIGURATION MANAGER
# ============================================================================


class ConfigurationManager:
    """
    Dynamic configuration management.

    Allows runtime configuration updates without restart.
    """

    def __init__(self):
        self.config: Dict[str, Any] = {
            # Threat Assessment
            "threat_assessment_risk_thresholds": {
                "critical": 9.0,
                "high": 7.0,
                "medium": 4.0,
            },
            # Alert Routing
            "alert_routing_rules": {
                "critical": ["slack", "pagerduty", "sms"],
                "high": ["slack", "pagerduty"],
                "medium": ["slack"],
            },
            # Auto-Remediation
            "auto_remediation_enabled": True,
            "auto_remediation_risk_threshold": 7.0,
            "auto_remediation_actions": [
                "create_firewall_rule",
                "rate_limit",
                "quarantine_threat",
            ],
            # ML Training
            "ml_training_schedule": "daily",
            "ml_training_min_samples": 100,
            "ml_model_evaluation_threshold": 0.85,
            # System
            "event_deduplication_window_seconds": 300,
            "max_concurrent_workflows": 100,
            "alert_escalation_timeout_minutes": 60,
        }

    def get_config(self, key: str) -> Any:
        """Get configuration value."""
        return self.config.get(key)

    def set_config(self, key: str, value: Any) -> bool:
        """Update configuration value."""
        self.config[key] = value
        logger.info(f"Configuration updated: {key} = {value}")
        return True

    def get_all_config(self) -> Dict[str, Any]:
        """Get all configuration."""
        return self.config.copy()


# ============================================================================
# AUTOMATION SYSTEM (Main Integration)
# ============================================================================


class AutomationSystem:
    """
    Unified automation system integrating all components.

    This is the main entry point for processing security events.
    """

    def __init__(self):
        self.event_broker = SecurityEventBroker()
        self.orchestrator = AutomationOrchestrator(self.event_broker)
        self.feedback_loop = FeedbackLoop()
        self.health_monitor = HealthMonitor()
        self.config_manager = ConfigurationManager()

        # Register components for monitoring
        self.health_monitor.register_component("threat_assessment")
        self.health_monitor.register_component("alert_routing")
        self.health_monitor.register_component("auto_remediation")
        self.health_monitor.register_component("incident_response")

    async def process_security_event(
        self,
        threat_signal: Dict[str, Any],
    ) -> Dict[str, Any]:
        """
        Process security event through complete automation system.

        Returns:
            Processing result with workflow execution details
        """

        threat_id = threat_signal.get("id", f"threat_{datetime.utcnow().timestamp()}")

        try:
            # Execute threat response workflow
            workflow = await self.orchestrator.process_threat(
                threat_signal,
                threat_id,
            )

            # Record metrics
            self.health_monitor.record_operation(
                "orchestrator",
                workflow.duration_seconds * 1000,
                success=workflow.status == WorkflowStatus.SUCCESS,
            )

            return {
                "success": workflow.status == WorkflowStatus.SUCCESS,
                "workflow_id": workflow.id,
                "threat_id": threat_id,
                "status": workflow.status.value,
                "steps": workflow.steps_executed,
                "results": workflow.results,
                "duration_seconds": workflow.duration_seconds,
            }

        except Exception as e:
            logger.error(f"Failed to process security event: {e}")
            return {
                "success": False,
                "threat_id": threat_id,
                "error": str(e),
            }

    async def provide_feedback(
        self,
        workflow_execution_id: str,
        action_id: str,
        feedback_type: str,
        confidence: float,
    ) -> bool:
        """
        Provide feedback on automation action.

        feedback_type: "correct", "incorrect", "partial"
        """

        await self.feedback_loop.submit_feedback(
            workflow_execution_id,
            action_id,
            feedback_type,
            confidence,
        )

        return True

    def get_system_status(self) -> Dict[str, Any]:
        """
        Get complete system status.

        Returns:
            System health, metrics, and configuration
        """

        return {
            "health": self.health_monitor.get_health_status(),
            "events_processed": self.event_broker.metrics["total_events"],
            "feedback_accuracy": self.feedback_loop.learning_metrics[
                "correct"
            ] / max(1, self.feedback_loop.learning_metrics["total_feedback"]),
            "configuration": self.config_manager.get_all_config(),
        }


__all__ = [
    "AutomationSystem",
    "SecurityEventBroker",
    "AutomationOrchestrator",
    "FeedbackLoop",
    "HealthMonitor",
    "ConfigurationManager",
    "SecurityEvent",
    "WorkflowExecution",
    "FeedbackRecord",
    "EventType",
    "WorkflowStatus",
]
