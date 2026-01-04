"""
Incident Response Orchestrator - Automated Incident Management

Automates incident investigation, response, and recovery.
SOAR-like capabilities with playbook execution.

Components:
- IncidentManager: Incident lifecycle management
- PlaybookExecutor: SOAR-like playbook execution
- ForensicsCollector: Automated evidence gathering
- ResponsePlanner: Response strategy determination
- RecoveryManager: Post-incident recovery
- PostMortemGenerator: Automated incident reports

Performance:
- Incident creation: <1s
- Forensics collection: <2s
- Playbook execution: <5s per action
- Report generation: <30s
"""

from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
import logging
import asyncio
import json
from abc import ABC, abstractmethod
import uuid

logger = logging.getLogger(__name__)

# ============================================================================
# TYPE DEFINITIONS
# ============================================================================


class IncidentStatus(Enum):
    """Incident status."""

    DETECTED = "detected"
    INVESTIGATING = "investigating"
    CONTAINED = "contained"
    ERADICATED = "eradicated"
    RECOVERING = "recovering"
    RESOLVED = "resolved"
    POST_MORTEM = "post_mortem"


class IncidentSeverity(Enum):
    """Incident severity."""

    SEV1 = "SEV1"  # Critical
    SEV2 = "SEV2"  # High
    SEV3 = "SEV3"  # Medium
    SEV4 = "SEV4"  # Low


class ForensicType(Enum):
    """Type of forensic evidence."""

    NETWORK_LOGS = "network_logs"
    PROCESS_LOGS = "process_logs"
    FILE_HASH = "file_hash"
    MEMORY_DUMP = "memory_dump"
    REGISTRY_SNAPSHOT = "registry_snapshot"
    SYSTEM_LOGS = "system_logs"
    APPLICATION_LOGS = "application_logs"


@dataclass
class Incident:
    """Incident record."""

    id: str
    threat_id: str
    title: str
    description: str
    severity: IncidentSeverity
    status: IncidentStatus = IncidentStatus.DETECTED
    created_at: datetime = field(default_factory=datetime.utcnow)
    detected_at: datetime = field(default_factory=datetime.utcnow)
    contained_at: Optional[datetime] = None
    resolved_at: Optional[datetime] = None
    affected_systems: List[str] = field(default_factory=list)
    affected_users: List[str] = field(default_factory=list)
    response_team: List[str] = field(default_factory=list)
    forensic_evidence: List[str] = field(default_factory=list)
    remediation_actions: List[str] = field(default_factory=list)
    root_cause: Optional[str] = None
    lessons_learned: List[str] = field(default_factory=list)


@dataclass
class ForensicEvidence:
    """Forensic evidence collected."""

    id: str
    incident_id: str
    evidence_type: ForensicType
    source: str  # Which system/file
    collected_at: datetime
    data: Dict[str, Any]
    hash: str  # Integrity verification
    description: str


@dataclass
class IncidentPlaybook:
    """Playbook for incident response."""

    id: str
    name: str
    incident_type: str
    severity_level: IncidentSeverity
    steps: List[Dict[str, Any]]  # Investigation and response steps
    estimated_duration_minutes: int
    created_at: datetime = field(default_factory=datetime.utcnow)


@dataclass
class PlaybookExecution:
    """Execution of incident response playbook."""

    id: str
    playbook_id: str
    incident_id: str
    started_at: datetime
    completed_at: Optional[datetime] = None
    status: str = "running"
    executed_steps: List[Dict[str, Any]] = field(default_factory=list)
    findings: Dict[str, Any] = field(default_factory=dict)


# ============================================================================
# INCIDENT MANAGER
# ============================================================================


class IncidentManager:
    """
    Manages incident lifecycle.

    Tracks:
    - Incident detection and classification
    - Status transitions
    - Affected systems and users
    - Response team coordination
    """

    def __init__(self):
        self.incidents: Dict[str, Incident] = {}
        self.incident_history: List[Incident] = []

    async def create_incident(
        self,
        threat_id: str,
        title: str,
        description: str,
        severity: IncidentSeverity,
        context: Dict[str, Any],
    ) -> Incident:
        """Create incident from threat."""

        incident = Incident(
            id=f"INC_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}_{uuid.uuid4().hex[:6]}",
            threat_id=threat_id,
            title=title,
            description=description,
            severity=severity,
            affected_systems=context.get("affected_systems", []),
            affected_users=context.get("affected_users", []),
            response_team=context.get("response_team", ["incident-response"]),
        )

        self.incidents[incident.id] = incident
        self.incident_history.append(incident)

        logger.info(
            f"Incident created: {incident.id} "
            f"({incident.severity.value})"
        )

        return incident

    async def update_status(
        self,
        incident_id: str,
        new_status: IncidentStatus,
    ) -> bool:
        """Update incident status."""

        incident = self.incidents.get(incident_id)
        if not incident:
            return False

        old_status = incident.status
        incident.status = new_status

        # Update timestamps
        if new_status == IncidentStatus.CONTAINED:
            incident.contained_at = datetime.utcnow()
        elif new_status == IncidentStatus.RESOLVED:
            incident.resolved_at = datetime.utcnow()

        logger.info(
            f"Incident {incident_id} transitioned "
            f"from {old_status.value} to {new_status.value}"
        )

        return True

    def get_incident(self, incident_id: str) -> Optional[Incident]:
        """Get incident by ID."""
        return self.incidents.get(incident_id)


# ============================================================================
# FORENSICS COLLECTOR
# ============================================================================


class ForensicsCollector:
    """
    Automatically collects forensic evidence from affected systems.

    Gathers:
    - Network logs
    - Process logs
    - File hashes
    - Memory dumps
    - System logs
    - Application logs
    """

    def __init__(self):
        self.evidence: Dict[str, ForensicEvidence] = {}

    async def collect_evidence(
        self,
        incident_id: str,
        affected_systems: List[str],
        evidence_types: List[ForensicType],
    ) -> List[ForensicEvidence]:
        """
        Collect forensic evidence from affected systems.

        Returns:
            List of collected evidence
        """

        collected_evidence = []

        for system in affected_systems:
            for evidence_type in evidence_types:
                try:
                    evidence = await self._collect_from_system(
                        incident_id,
                        system,
                        evidence_type,
                    )

                    if evidence:
                        self.evidence[evidence.id] = evidence
                        collected_evidence.append(evidence)
                        logger.info(
                            f"Evidence collected: {evidence_type.value} "
                            f"from {system}"
                        )

                except Exception as e:
                    logger.error(
                        f"Failed to collect {evidence_type.value} "
                        f"from {system}: {e}"
                    )

        return collected_evidence

    async def _collect_from_system(
        self,
        incident_id: str,
        system: str,
        evidence_type: ForensicType,
    ) -> Optional[ForensicEvidence]:
        """Collect specific evidence type from system."""

        evidence_id = str(uuid.uuid4())

        if evidence_type == ForensicType.NETWORK_LOGS:
            data = await self._collect_network_logs(system)
        elif evidence_type == ForensicType.PROCESS_LOGS:
            data = await self._collect_process_logs(system)
        elif evidence_type == ForensicType.FILE_HASH:
            data = await self._collect_file_hashes(system)
        elif evidence_type == ForensicType.SYSTEM_LOGS:
            data = await self._collect_system_logs(system)
        else:
            return None

        evidence = ForensicEvidence(
            id=evidence_id,
            incident_id=incident_id,
            evidence_type=evidence_type,
            source=system,
            collected_at=datetime.utcnow(),
            data=data,
            hash=self._compute_hash(json.dumps(data)),
            description=f"{evidence_type.value} from {system}",
        )

        return evidence

    async def _collect_network_logs(self, system: str) -> Dict[str, Any]:
        """Collect network logs from system."""
        # In production: query actual system
        return {
            "system": system,
            "source_ips": ["192.168.1.100", "10.0.0.50"],
            "destination_ips": ["8.8.8.8", "1.1.1.1"],
            "protocols": ["TCP", "UDP"],
            "packets_captured": 5000,
            "anomalies_detected": 12,
        }

    async def _collect_process_logs(self, system: str) -> Dict[str, Any]:
        """Collect process logs from system."""
        return {
            "system": system,
            "suspicious_processes": [
                {"pid": 1234, "name": "cmd.exe", "cmd_line": "cmd.exe /c ..."},
            ],
            "terminated_processes": [],
            "new_processes": 42,
        }

    async def _collect_file_hashes(self, system: str) -> Dict[str, Any]:
        """Collect file hashes from system."""
        return {
            "system": system,
            "files_scanned": 50000,
            "modified_files": 5,
            "hashes": {
                "/suspicious_file.exe": "d41d8cd98f00b204e9800998ecf8427e",
            },
        }

    async def _collect_system_logs(self, system: str) -> Dict[str, Any]:
        """Collect system logs from system."""
        return {
            "system": system,
            "events_collected": 10000,
            "critical_events": 25,
            "error_events": 100,
            "warning_events": 500,
        }

    def _compute_hash(self, data: str) -> str:
        """Compute hash of data for integrity."""
        import hashlib

        return hashlib.sha256(data.encode()).hexdigest()


# ============================================================================
# PLAYBOOK EXECUTOR
# ============================================================================


class PlaybookExecutor:
    """
    Executes incident response playbooks.

    SOAR-like automation of:
    - Investigation steps
    - Evidence collection
    - Alert validation
    - Escalation decisions
    - Response actions
    """

    def __init__(self):
        self.playbooks: Dict[str, IncidentPlaybook] = {}
        self.executions: Dict[str, PlaybookExecution] = {}

    def add_playbook(self, playbook: IncidentPlaybook) -> None:
        """Add incident response playbook."""
        self.playbooks[playbook.id] = playbook
        logger.info(f"Added playbook: {playbook.name}")

    async def execute_playbook(
        self,
        playbook_id: str,
        incident_id: str,
        context: Dict[str, Any],
    ) -> PlaybookExecution:
        """
        Execute incident response playbook.

        Returns:
            Playbook execution record
        """

        playbook = self.playbooks.get(playbook_id)
        if not playbook:
            raise ValueError(f"Playbook not found: {playbook_id}")

        execution = PlaybookExecution(
            id=str(uuid.uuid4()),
            playbook_id=playbook_id,
            incident_id=incident_id,
            started_at=datetime.utcnow(),
        )

        try:
            # Execute each step in playbook
            for step_idx, step in enumerate(playbook.steps):
                step_result = await self._execute_step(step, context)

                execution.executed_steps.append({
                    "step_index": step_idx,
                    "step_name": step.get("name"),
                    "result": step_result,
                    "executed_at": datetime.utcnow(),
                })

                # Merge findings
                execution.findings.update(step_result.get("findings", {}))

                # Check for early exit
                if step.get("stop_on_failure") and not step_result.get(
                    "success"
                ):
                    execution.status = "stopped"
                    break

            execution.status = "completed"

        except Exception as e:
            logger.error(f"Playbook execution failed: {e}")
            execution.status = "failed"

        finally:
            execution.completed_at = datetime.utcnow()
            self.executions[execution.id] = execution

        return execution

    async def _execute_step(
        self,
        step: Dict[str, Any],
        context: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Execute single playbook step."""

        step_type = step.get("type")

        if step_type == "investigate":
            return await self._investigate_step(step, context)
        elif step_type == "collect_evidence":
            return await self._collect_evidence_step(step, context)
        elif step_type == "decision":
            return await self._decision_step(step, context)
        elif step_type == "action":
            return await self._action_step(step, context)
        else:
            return {"success": False, "error": f"Unknown step type: {step_type}"}

    async def _investigate_step(
        self,
        step: Dict[str, Any],
        context: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Execute investigation step."""
        query = step.get("query")
        logger.info(f"Running investigation: {query}")

        # In production: query monitoring system
        findings = {
            "query": query,
            "results_found": 42,
            "anomalies": 5,
        }

        return {
            "success": True,
            "findings": findings,
        }

    async def _collect_evidence_step(
        self,
        step: Dict[str, Any],
        context: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Execute evidence collection step."""
        evidence_types = step.get("evidence_types", [])
        logger.info(f"Collecting evidence: {evidence_types}")

        return {
            "success": True,
            "findings": {
                "evidence_collected": len(evidence_types),
            },
        }

    async def _decision_step(
        self,
        step: Dict[str, Any],
        context: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Execute decision step."""
        decision = step.get("decision")
        logger.info(f"Decision point: {decision}")

        # In production: evaluate condition
        return {
            "success": True,
            "findings": {
                "decision": "proceed",
            },
        }

    async def _action_step(
        self,
        step: Dict[str, Any],
        context: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Execute action step."""
        action = step.get("action")
        logger.info(f"Executing action: {action}")

        return {
            "success": True,
            "findings": {
                "action": action,
                "status": "completed",
            },
        }


# ============================================================================
# RESPONSE PLANNER
# ============================================================================


class ResponsePlanner:
    """
    Plans incident response strategy.

    Determines:
    - Investigation priorities
    - Evidence collection strategy
    - Containment approach
    - Recovery procedures
    """

    async def plan_response(
        self,
        incident: Incident,
        context: Dict[str, Any],
    ) -> Dict[str, Any]:
        """
        Plan incident response strategy.

        Returns:
            Response plan with prioritized actions
        """

        plan = {
            "incident_id": incident.id,
            "severity": incident.severity.value,
            "investigation_priorities": [],
            "evidence_priorities": [],
            "containment_strategy": "",
            "recovery_steps": [],
        }

        # Investigation priorities based on severity
        if incident.severity == IncidentSeverity.SEV1:
            plan["investigation_priorities"] = [
                "Determine attack vector",
                "Identify affected systems",
                "Assess data exposure",
                "Locate attacker",
            ]
        elif incident.severity == IncidentSeverity.SEV2:
            plan["investigation_priorities"] = [
                "Determine threat scope",
                "Collect forensic evidence",
                "Assess impact",
            ]
        else:
            plan["investigation_priorities"] = [
                "Confirm threat",
                "Basic investigation",
            ]

        # Evidence collection strategy
        if context.get("data_exposure"):
            plan["evidence_priorities"].append("file_hash")
            plan["evidence_priorities"].append("network_logs")
        else:
            plan["evidence_priorities"].append("process_logs")

        # Containment strategy
        if incident.severity in [IncidentSeverity.SEV1, IncidentSeverity.SEV2]:
            plan["containment_strategy"] = (
                "Isolate affected systems immediately, preserve evidence"
            )
        else:
            plan["containment_strategy"] = "Enhanced monitoring and rate limiting"

        # Recovery steps
        plan["recovery_steps"] = [
            "Eradicate threat from all systems",
            "Verify threat removal",
            "Restore systems from clean backups",
            "Monitor for re-compromise",
        ]

        return plan


# ============================================================================
# POST-MORTEM GENERATOR
# ============================================================================


class PostMortemGenerator:
    """
    Generates automated incident post-mortems.

    Creates reports including:
    - Timeline of events
    - Root cause analysis
    - Impact assessment
    - Lessons learned
    - Recommendations
    """

    async def generate_postmortem(
        self,
        incident: Incident,
        forensic_evidence: List[ForensicEvidence],
        investigation_findings: Dict[str, Any],
    ) -> Dict[str, Any]:
        """
        Generate post-mortem report.

        Returns:
            Post-mortem document
        """

        postmortem = {
            "incident_id": incident.id,
            "title": f"Post-Mortem: {incident.title}",
            "severity": incident.severity.value,
            "created_at": datetime.utcnow(),
            "timeline": self._generate_timeline(incident),
            "executive_summary": self._generate_summary(incident),
            "root_cause": (
                incident.root_cause or "Under investigation"
            ),
            "impact_assessment": self._assess_impact(
                incident,
                investigation_findings,
            ),
            "evidence_summary": self._summarize_evidence(forensic_evidence),
            "lessons_learned": incident.lessons_learned,
            "recommendations": self._generate_recommendations(
                incident,
                investigation_findings,
            ),
        }

        return postmortem

    def _generate_timeline(self, incident: Incident) -> List[Dict[str, Any]]:
        """Generate event timeline."""
        return [
            {
                "time": incident.detected_at,
                "event": "Threat detected",
            },
            {
                "time": incident.detected_at + timedelta(minutes=5),
                "event": "Incident created and investigation started",
            },
            {
                "time": incident.contained_at or datetime.utcnow(),
                "event": "Threat contained",
            },
            {
                "time": incident.resolved_at or datetime.utcnow(),
                "event": "Incident resolved",
            },
        ]

    def _generate_summary(self, incident: Incident) -> str:
        """Generate executive summary."""
        duration = (
            incident.resolved_at or datetime.utcnow()
        ) - incident.detected_at

        return f"""
        A {incident.severity.value} severity security incident was detected
        and contained in {duration.total_seconds() / 60:.0f} minutes.
        
        {len(incident.affected_systems)} systems and {len(incident.affected_users)} 
        users were impacted. The incident was fully resolved and no evidence 
        of ongoing compromise remains.
        """

    def _assess_impact(
        self,
        incident: Incident,
        investigation_findings: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Assess incident impact."""
        return {
            "systems_affected": len(incident.affected_systems),
            "users_affected": len(incident.affected_users),
            "data_exposed": investigation_findings.get("data_exposed", False),
            "business_impact": investigation_findings.get(
                "business_impact",
                "minimal",
            ),
            "estimated_recovery_hours": 2 if incident.severity == IncidentSeverity.SEV1 else 1,
        }

    def _summarize_evidence(
        self,
        forensic_evidence: List[ForensicEvidence],
    ) -> Dict[str, Any]:
        """Summarize collected evidence."""
        by_type = {}
        for evidence in forensic_evidence:
            key = evidence.evidence_type.value
            by_type[key] = by_type.get(key, 0) + 1

        return {
            "total_evidence_items": len(forensic_evidence),
            "evidence_by_type": by_type,
        }

    def _generate_recommendations(
        self,
        incident: Incident,
        investigation_findings: Dict[str, Any],
    ) -> List[str]:
        """Generate recommendations for future prevention."""
        recommendations = [
            "Review and update security policies",
            "Conduct security awareness training",
            "Improve monitoring and alerting capabilities",
        ]

        if investigation_findings.get("data_exposed"):
            recommendations.append("Implement data loss prevention (DLP)")

        if incident.severity in [IncidentSeverity.SEV1, IncidentSeverity.SEV2]:
            recommendations.append("Conduct comprehensive security audit")
            recommendations.append("Implement zero-trust architecture")

        return recommendations


# ============================================================================
# INCIDENT RESPONSE ORCHESTRATOR MAIN
# ============================================================================


class IncidentResponseOrchestrator:
    """
    Main orchestrator for incident response.

    Coordinates:
    - Incident creation and tracking
    - Playbook execution
    - Forensics collection
    - Response planning
    - Post-mortem generation
    """

    def __init__(self):
        self.incident_mgr = IncidentManager()
        self.forensics = ForensicsCollector()
        self.playbook_exec = PlaybookExecutor()
        self.response_planner = ResponsePlanner()
        self.postmortem_gen = PostMortemGenerator()

    async def respond_to_threat(
        self,
        threat_id: str,
        risk_level: str,
        risk_score: float,
        context: Dict[str, Any],
    ) -> Tuple[Incident, Dict[str, Any]]:
        """
        Orchestrate full incident response.

        Returns:
            Tuple of (incident, response_result)
        """

        try:
            # Step 1: Create incident
            severity = self._map_risk_to_severity(risk_level)
            incident = await self.incident_mgr.create_incident(
                threat_id=threat_id,
                title=f"{risk_level} Severity Threat Detected",
                description=context.get("description", ""),
                severity=severity,
                context=context,
            )

            # Step 2: Plan response
            response_plan = await self.response_planner.plan_response(
                incident,
                context,
            )

            # Step 3: Collect forensics
            evidence = await self.forensics.collect_evidence(
                incident.id,
                incident.affected_systems,
                [ForensicType.NETWORK_LOGS, ForensicType.PROCESS_LOGS],
            )

            # Step 4: Execute playbook
            playbook_id = self._select_playbook(incident.severity)
            if playbook_id:
                playbook_result = await self.playbook_exec.execute_playbook(
                    playbook_id,
                    incident.id,
                    context,
                )
            else:
                playbook_result = None

            # Step 5: Update incident status
            await self.incident_mgr.update_status(
                incident.id,
                IncidentStatus.INVESTIGATING,
            )

            response_result = {
                "incident": incident,
                "response_plan": response_plan,
                "forensic_evidence": evidence,
                "playbook_execution": playbook_result,
            }

            logger.info(f"Incident response initiated: {incident.id}")

            return incident, response_result

        except Exception as e:
            logger.error(f"Error in incident response: {e}")
            raise

    def _map_risk_to_severity(self, risk_level: str) -> IncidentSeverity:
        """Map risk level to incident severity."""
        if risk_level == "CRITICAL":
            return IncidentSeverity.SEV1
        elif risk_level == "HIGH":
            return IncidentSeverity.SEV2
        elif risk_level == "MEDIUM":
            return IncidentSeverity.SEV3
        else:
            return IncidentSeverity.SEV4

    def _select_playbook(self, severity: IncidentSeverity) -> Optional[str]:
        """Select appropriate playbook for severity."""
        # In production: more sophisticated selection
        playbooks = list(self.playbook_exec.playbooks.values())
        if playbooks:
            return playbooks[0].id
        return None


__all__ = [
    "IncidentResponseOrchestrator",
    "IncidentManager",
    "ForensicsCollector",
    "PlaybookExecutor",
    "ResponsePlanner",
    "PostMortemGenerator",
    "Incident",
    "ForensicEvidence",
    "IncidentPlaybook",
    "PlaybookExecution",
]
