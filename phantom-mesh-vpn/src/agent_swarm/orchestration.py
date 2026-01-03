"""
PhantomMesh Orchestration Engine — State Machine & Coordination
================================================================

Hierarchical state machine for agent lifecycle, workflow orchestration,
and coordinated threat response patterns.

@FORTRESS Protocol Implementation
Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

from __future__ import annotations

import asyncio
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, UTC, timedelta
from enum import Enum, auto
from typing import Any, Callable, Coroutine, Optional
import json

import structlog

logger = structlog.get_logger(__name__)


# ═══════════════════════════════════════════════════════════════════════════════
# ORCHESTRATION STATE MACHINE
# ═══════════════════════════════════════════════════════════════════════════════

class AgentLifecycleState(Enum):
    """Agent lifecycle states."""
    INITIALIZING = auto()
    IDLE = auto()
    EXECUTING = auto()
    WAITING = auto()
    DEGRADED = auto()
    ERROR = auto()
    SHUTTING_DOWN = auto()


class WorkflowState(Enum):
    """Workflow execution states."""
    CREATED = auto()
    QUEUED = auto()
    RUNNING = auto()
    PAUSED = auto()
    COMPLETED = auto()
    FAILED = auto()
    CANCELLED = auto()
    ROLLED_BACK = auto()


class ThreatLevel(Enum):
    """Threat severity levels."""
    NONE = auto()
    LOW = auto()
    MEDIUM = auto()
    HIGH = auto()
    CRITICAL = auto()
    CATASTROPHIC = auto()


@dataclass
class StateTransition:
    """Immutable state transition record."""
    from_state: Any
    to_state: Any
    timestamp: datetime = field(default_factory=lambda: datetime.now(UTC))
    reason: str = ""
    context: dict[str, Any] = field(default_factory=dict)
    
    def to_dict(self) -> dict[str, Any]:
        return {
            "from_state": self.from_state.name if hasattr(self.from_state, 'name') else str(self.from_state),
            "to_state": self.to_state.name if hasattr(self.to_state, 'name') else str(self.to_state),
            "timestamp": self.timestamp.isoformat(),
            "reason": self.reason,
            "context": self.context
        }


@dataclass
class StateGuard:
    """Preconditions for state transitions."""
    condition: Callable[..., bool]
    message: str = "Guard condition failed"
    
    def evaluate(self, *args, **kwargs) -> bool:
        return self.condition(*args, **kwargs)


class StateMachine:
    """
    Hierarchical state machine with guards, transitions, and callbacks.
    Implements temporal logic for workflow orchestration.
    """
    
    def __init__(self, initial_state: Any, entity_id: str = ""):
        self.current_state = initial_state
        self.entity_id = entity_id
        self._transitions: dict[tuple[Any, Any], list[StateGuard]] = {}
        self._callbacks: dict[tuple[Any, Any], list[Callable[[StateTransition], Coroutine[Any, Any, Any]]]] = {}
        self._history: list[StateTransition] = []
        self._state_lock = asyncio.Lock()
        
        logger.info("state_machine_created", entity=entity_id, initial_state=str(initial_state))
    
    def register_transition(
        self,
        from_state: Any,
        to_state: Any,
        guards: Optional[list[StateGuard]] = None
    ) -> None:
        """Register valid state transition with optional guards."""
        key = (from_state, to_state)
        self._transitions[key] = guards or []
        logger.debug("transition_registered", from_state=str(from_state), to_state=str(to_state))
    
    def on_transition(
        self,
        from_state: Any,
        to_state: Any,
        callback: Callable[[StateTransition], Coroutine[Any, Any, Any]]
    ) -> None:
        """Register callback for transition."""
        key = (from_state, to_state)
        if key not in self._callbacks:
            self._callbacks[key] = []
        self._callbacks[key].append(callback)
    
    async def transition(
        self,
        to_state: Any,
        reason: str = "",
        context: Optional[dict[str, Any]] = None
    ) -> bool:
        """
        Attempt state transition with guard evaluation and callbacks.
        Returns True if successful, False otherwise.
        """
        async with self._state_lock:
            # Check if transition is valid
            key = (self.current_state, to_state)
            if key not in self._transitions:
                logger.warning(
                    "invalid_transition",
                    entity=self.entity_id,
                    from_state=str(self.current_state),
                    to_state=str(to_state)
                )
                return False
            
            # Evaluate guards
            guards = self._transitions[key]
            for guard in guards:
                if not guard.evaluate(context or {}):
                    logger.warning(
                        "guard_failed",
                        entity=self.entity_id,
                        guard=guard.message,
                        context=context
                    )
                    return False
            
            # Perform transition
            from_state = self.current_state
            self.current_state = to_state
            
            # Record transition
            transition = StateTransition(
                from_state=from_state,
                to_state=to_state,
                reason=reason,
                context=context or {}
            )
            self._history.append(transition)
            
            logger.info(
                "state_transition",
                entity=self.entity_id,
                from_state=str(from_state),
                to_state=str(to_state),
                reason=reason
            )
            
            # Execute callbacks
            if key in self._callbacks:
                for callback in self._callbacks[key]:
                    try:
                        await callback(transition)
                    except Exception as e:
                        logger.error(
                            "transition_callback_error",
                            entity=self.entity_id,
                            error=str(e)
                        )
            
            return True
    
    def get_history(self, limit: Optional[int] = None) -> list[StateTransition]:
        """Get state transition history."""
        if limit:
            return self._history[-limit:]
        return self._history.copy()
    
    def can_transition_to(self, to_state: Any) -> bool:
        """Check if transition is possible without executing it."""
        return (self.current_state, to_state) in self._transitions


# ═══════════════════════════════════════════════════════════════════════════════
# WORKFLOW ORCHESTRATION
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class WorkflowStep:
    """Single step in a workflow."""
    id: str
    name: str
    agent_role: str
    action: str
    timeout: float = 30.0
    retries: int = 3
    rollback_action: Optional[str] = None
    dependencies: list[str] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class WorkflowExecution:
    """Execution context for a workflow."""
    workflow_id: str
    execution_id: str
    steps: list[WorkflowStep]
    state_machine: StateMachine
    start_time: datetime = field(default_factory=lambda: datetime.now(UTC))
    end_time: Optional[datetime] = None
    results: dict[str, Any] = field(default_factory=dict)
    failed_steps: list[str] = field(default_factory=list)


class OrchestrationEngine:
    """
    Master orchestration engine for workflow execution and agent coordination.
    Implements FORTRESS threat response patterns.
    """
    
    def __init__(self):
        self._workflows: dict[str, WorkflowExecution] = {}
        self._step_handlers: dict[str, Callable[[WorkflowStep, dict[str, Any]], Coroutine[Any, Any, Any]]] = {}
        self._threat_level = ThreatLevel.NONE
        self._active_incidents: dict[str, dict[str, Any]] = {}
        
        logger.info("orchestration_engine_initialized")
    
    def register_step_handler(
        self,
        step_type: str,
        handler: Callable[[WorkflowStep, dict[str, Any]], Coroutine[Any, Any, Any]]
    ) -> None:
        """Register handler for specific workflow step type."""
        self._step_handlers[step_type] = handler
        logger.debug("step_handler_registered", step_type=step_type)
    
    async def execute_workflow(
        self,
        workflow_id: str,
        steps: list[WorkflowStep],
        context: Optional[dict[str, Any]] = None
    ) -> WorkflowExecution:
        """
        Execute workflow with state machine tracking and automatic rollback.
        """
        execution_id = f"{workflow_id}_{datetime.now(UTC).timestamp()}"
        
        # Create state machine
        sm = StateMachine(WorkflowState.CREATED, entity_id=workflow_id)
        
        # Register transitions
        sm.register_transition(WorkflowState.CREATED, WorkflowState.QUEUED)
        sm.register_transition(WorkflowState.QUEUED, WorkflowState.RUNNING)
        sm.register_transition(WorkflowState.RUNNING, WorkflowState.COMPLETED)
        sm.register_transition(WorkflowState.RUNNING, WorkflowState.FAILED)
        sm.register_transition(WorkflowState.RUNNING, WorkflowState.PAUSED)
        sm.register_transition(WorkflowState.PAUSED, WorkflowState.RUNNING)
        sm.register_transition(WorkflowState.FAILED, WorkflowState.ROLLED_BACK)
        
        # Create execution context
        execution = WorkflowExecution(
            workflow_id=workflow_id,
            execution_id=execution_id,
            steps=steps,
            state_machine=sm
        )
        
        self._workflows[execution_id] = execution
        
        try:
            # Transition to queued
            await sm.transition(WorkflowState.QUEUED, reason="Workflow enqueued")
            
            # Transition to running
            await sm.transition(WorkflowState.RUNNING, reason="Workflow started")
            
            # Execute steps in order
            for step in steps:
                try:
                    result = await self._execute_step(step, context or {})
                    execution.results[step.id] = result
                    
                    logger.info(
                        "workflow_step_completed",
                        workflow=workflow_id,
                        step=step.id,
                        result_keys=list(result.keys())
                    )
                    
                except Exception as e:
                    logger.error(
                        "workflow_step_failed",
                        workflow=workflow_id,
                        step=step.id,
                        error=str(e)
                    )
                    
                    execution.failed_steps.append(step.id)
                    
                    # Attempt rollback
                    await self._rollback_workflow(execution)
                    await sm.transition(WorkflowState.FAILED, reason=f"Step {step.id} failed: {str(e)}")
                    
                    execution.end_time = datetime.now(UTC)
                    return execution
            
            # Mark as completed
            await sm.transition(WorkflowState.COMPLETED, reason="All steps completed successfully")
            execution.end_time = datetime.now(UTC)
            
            logger.info(
                "workflow_completed",
                workflow=workflow_id,
                execution=execution_id,
                duration_seconds=(execution.end_time - execution.start_time).total_seconds()
            )
            
        except Exception as e:
            logger.error(
                "workflow_execution_error",
                workflow=workflow_id,
                error=str(e)
            )
            
            await sm.transition(WorkflowState.FAILED, reason=f"Execution error: {str(e)}")
            await self._rollback_workflow(execution)
            execution.end_time = datetime.now(UTC)
        
        return execution
    
    async def _execute_step(
        self,
        step: WorkflowStep,
        context: dict[str, Any]
    ) -> dict[str, Any]:
        """Execute single workflow step with retry logic."""
        handler = self._step_handlers.get(step.name)
        
        if not handler:
            raise ValueError(f"No handler registered for step: {step.name}")
        
        last_error = None
        
        for attempt in range(step.retries):
            try:
                result = await asyncio.wait_for(
                    handler(step, context),
                    timeout=step.timeout
                )
                return result
                
            except asyncio.TimeoutError:
                last_error = f"Step timeout ({step.timeout}s)"
                logger.warning(
                    "step_timeout",
                    step=step.id,
                    attempt=attempt + 1,
                    timeout=step.timeout
                )
                
            except Exception as e:
                last_error = str(e)
                logger.warning(
                    "step_error",
                    step=step.id,
                    attempt=attempt + 1,
                    error=str(e)
                )
            
            # Backoff before retry
            if attempt < step.retries - 1:
                await asyncio.sleep(2 ** attempt)
        
        raise RuntimeError(f"Step {step.id} failed after {step.retries} attempts: {last_error}")
    
    async def _rollback_workflow(self, execution: WorkflowExecution) -> None:
        """Rollback workflow on failure."""
        logger.warning(
            "workflow_rollback_started",
            workflow=execution.workflow_id,
            failed_steps=execution.failed_steps
        )
        
        # Rollback in reverse order
        for step in reversed(execution.steps):
            if step.id in execution.results and step.rollback_action:
                try:
                    logger.info(
                        "step_rollback",
                        step=step.id,
                        action=step.rollback_action
                    )
                    # TODO: Execute rollback action
                    
                except Exception as e:
                    logger.error(
                        "rollback_error",
                        step=step.id,
                        error=str(e)
                    )
    
    async def record_threat(
        self,
        threat_id: str,
        level: ThreatLevel,
        description: str,
        context: Optional[dict[str, Any]] = None
    ) -> None:
        """Record detected threat and escalate if necessary."""
        self._active_incidents[threat_id] = {
            "level": level,
            "description": description,
            "timestamp": datetime.now(UTC).isoformat(),
            "context": context or {}
        }
        
        # Update threat level
        if level.value > self._threat_level.value:
            self._threat_level = level
            logger.critical(
                "threat_level_escalated",
                threat_id=threat_id,
                level=level.name,
                description=description
            )
    
    def get_threat_status(self) -> dict[str, Any]:
        """Get current threat status and active incidents."""
        return {
            "threat_level": self._threat_level.name,
            "active_incidents": len(self._active_incidents),
            "incidents": self._active_incidents
        }


# ═══════════════════════════════════════════════════════════════════════════════
# THREAT RESPONSE PATTERNS
# ═══════════════════════════════════════════════════════════════════════════════

class ThreatResponsePattern:
    """Automated threat response pattern definition."""
    
    def __init__(
        self,
        pattern_id: str,
        threat_signature: dict[str, Any],
        response_workflow: list[WorkflowStep],
        escalation_level: ThreatLevel
    ):
        self.pattern_id = pattern_id
        self.threat_signature = threat_signature
        self.response_workflow = response_workflow
        self.escalation_level = escalation_level
    
    def matches(self, threat_data: dict[str, Any]) -> bool:
        """Check if threat matches this pattern."""
        for key, expected_value in self.threat_signature.items():
            actual_value = threat_data.get(key)
            
            if isinstance(expected_value, dict) and isinstance(actual_value, dict):
                # Recursive matching
                for sub_key, sub_value in expected_value.items():
                    if actual_value.get(sub_key) != sub_value:
                        return False
            elif actual_value != expected_value:
                return False
        
        return True


class ThreatResponseEngine:
    """Engine for automated threat response and incident handling."""
    
    def __init__(self, orchestration_engine: OrchestrationEngine):
        self.orchestration = orchestration_engine
        self.patterns: dict[str, ThreatResponsePattern] = {}
        self._incident_handlers: dict[str, Callable[[dict[str, Any]], Coroutine[Any, Any, Any]]] = {}
        
        logger.info("threat_response_engine_initialized")
    
    def register_pattern(self, pattern: ThreatResponsePattern) -> None:
        """Register threat response pattern."""
        self.patterns[pattern.pattern_id] = pattern
        logger.debug("threat_pattern_registered", pattern=pattern.pattern_id)
    
    async def handle_threat(self, threat_data: dict[str, Any]) -> bool:
        """
        Detect threat and execute automated response.
        Returns True if threat was handled, False if no matching pattern.
        """
        threat_id = threat_data.get("id", f"threat_{datetime.now(UTC).timestamp()}")
        
        # Find matching patterns
        matching_patterns = [
            p for p in self.patterns.values()
            if p.matches(threat_data)
        ]
        
        if not matching_patterns:
            logger.warning("no_matching_threat_pattern", threat_id=threat_id)
            return False
        
        # Execute response for highest severity pattern
        pattern = max(matching_patterns, key=lambda p: p.escalation_level.value)
        
        logger.critical(
            "executing_threat_response",
            threat_id=threat_id,
            pattern=pattern.pattern_id,
            level=pattern.escalation_level.name
        )
        
        # Record threat
        await self.orchestration.record_threat(
            threat_id,
            pattern.escalation_level,
            f"Matched pattern: {pattern.pattern_id}",
            context=threat_data
        )
        
        # Execute response workflow
        execution = await self.orchestration.execute_workflow(
            f"response_{threat_id}",
            pattern.response_workflow,
            context=threat_data
        )
        
        return execution.state_machine.current_state == WorkflowState.COMPLETED


# ═══════════════════════════════════════════════════════════════════════════════
# INCIDENT PLAYBOOKS
# ═══════════════════════════════════════════════════════════════════════════════

def create_port_scan_playbook() -> ThreatResponsePattern:
    """Playbook for port scanning detection and response."""
    return ThreatResponsePattern(
        pattern_id="port_scan_detected",
        threat_signature={
            "event_type": "port_scan",
            "severity": "high"
        },
        response_workflow=[
            WorkflowStep(
                id="block_source",
                name="block_ip_source",
                agent_role="FORTRESS",
                action="Block source IP address",
                rollback_action="Unblock source IP address"
            ),
            WorkflowStep(
                id="alert_team",
                name="send_alert",
                agent_role="OMNISCIENT",
                action="Alert security team"
            ),
            WorkflowStep(
                id="log_incident",
                name="log_incident",
                agent_role="APEX",
                action="Log security incident"
            )
        ],
        escalation_level=ThreatLevel.HIGH
    )


def create_brute_force_playbook() -> ThreatResponsePattern:
    """Playbook for brute force attack detection and response."""
    return ThreatResponsePattern(
        pattern_id="brute_force_attack",
        threat_signature={
            "event_type": "failed_auth",
            "failed_attempts": {"$gte": 5}
        },
        response_workflow=[
            WorkflowStep(
                id="lock_account",
                name="lock_user_account",
                agent_role="AEGIS",
                action="Temporarily lock affected account",
                timeout=60
            ),
            WorkflowStep(
                id="reset_creds",
                name="reset_credentials",
                agent_role="CIPHER",
                action="Reset user credentials",
                timeout=30
            ),
            WorkflowStep(
                id="notify_user",
                name="notify_user",
                agent_role="APEX",
                action="Notify user of security incident"
            )
        ],
        escalation_level=ThreatLevel.HIGH
    )


def create_anomalous_traffic_playbook() -> ThreatResponsePattern:
    """Playbook for anomalous network traffic detection."""
    return ThreatResponsePattern(
        pattern_id="anomalous_traffic",
        threat_signature={
            "event_type": "traffic_anomaly",
            "anomaly_score": {"$gte": 0.8}
        },
        response_workflow=[
            WorkflowStep(
                id="capture_traffic",
                name="capture_network_traffic",
                agent_role="STREAM",
                action="Capture suspicious traffic for analysis",
                timeout=120
            ),
            WorkflowStep(
                id="isolate_system",
                name="isolate_system",
                agent_role="PHANTOM",
                action="Isolate affected system from network",
                rollback_action="Reconnect system to network"
            ),
            WorkflowStep(
                id="analyze_malware",
                name="analyze_malware",
                agent_role="FORTRESS",
                action="Analyze captured traffic for malware",
                timeout=300
            )
        ],
        escalation_level=ThreatLevel.CRITICAL
    )
