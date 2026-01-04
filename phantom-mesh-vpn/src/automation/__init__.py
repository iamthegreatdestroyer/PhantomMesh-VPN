"""
PhantomMesh VPN - Automation & AI Agent Integration Layer

P1-005 Phase: AI Agent Integration & Automation Layer

This module provides autonomous security operations capabilities including:
- Threat assessment and risk scoring
- Intelligent alert routing and escalation
- Auto-remediation with reversible actions
- Incident response orchestration
- ML model training pipeline
- Integrated agent coordination

Components:
1. threat_assessment.py     - Risk scoring and confidence estimation
2. alert_routing.py         - Multi-channel alert routing and escalation
3. auto_remediation.py      - Reversible remediation actions
4. incident_response.py     - Incident lifecycle and forensics
5. ml_training.py           - ML model training pipeline (coming)
6. orchestration.py         - Unified orchestration layer (coming)

Architecture:
    Analytics (P1-004)
        ↓
    Threat Assessment → Risk Scoring (1-10)
        ↓
    Alert Router → Multi-channel Notification
        ↓
    Auto-Remediation → Reversible Actions
        ↓
    Incident Response → SOAR-like Automation
        ↓
    ML Training → Continuous Improvement

Performance Targets:
- Threat assessment: <50ms
- Alert routing: <100ms
- Remediation: <500ms
- Incident response: <1s creation
- ML inference: <10ms

Quality:
- 100% type hints
- 100% documentation (public APIs)
- Comprehensive error handling
- Full async implementation
- Complete audit trails
- Enterprise-grade architecture
"""

# Phase P1-005 Components
from .threat_assessment import (
    ThreatAssessor,
    ThreatAssessment,
    RiskScoreCalculator,
    ConfidenceEstimator,
    ImpactAnalyzer,
    ContextualAnalyzer,
    RiskLevel,
    ConfidenceLevel,
    ThreatSignal,
)

from .alert_routing import (
    AlertRoutingOrchestrator,
    AlertRouter,
    EscalationManager,
    NotificationService,
    AlertEnricher,
    AlertSuppressionFilter,
    AlertRoute,
    EscalationPolicy,
    AlertNotification,
    RoutedAlert,
    NotificationChannel,
    EscalationLevel,
)

from .auto_remediation import (
    RemediationOrchestrator,
    RemediationPlaybook,
    RemediationStep,
    RemediationExecution,
    ActionRecord,
    RemediationAction,
    ActionStatus,
)

from .incident_response import (
    IncidentResponseOrchestrator,
    IncidentManager,
    ForensicsCollector,
    PlaybookExecutor,
    ResponsePlanner,
    PostMortemGenerator,
    Incident,
    ForensicEvidence,
    IncidentPlaybook,
    PlaybookExecution,
    IncidentStatus,
    IncidentSeverity,
    ForensicType,
)

# Component 5: ML Training Pipeline
from .ml_training import (
    MLTrainingOrchestrator,
    DataPipeline,
    FeatureEngineer,
    ModelTrainer,
    ModelEvaluator,
    ModelRegistry,
    OnlinePredictor,
    ModelVersion,
    PredictionResult,
    TrainingJob,
    FeatureVector,
    TrainingExample,
    ModelType,
    TrainingStatus,
)

# Component 6: Integration & Orchestration
from .integration import (
    AutomationSystem,
    SecurityEventBroker,
    AutomationOrchestrator,
    FeedbackLoop,
    HealthMonitor,
    ConfigurationManager,
    SecurityEvent,
    WorkflowExecution,
    ComponentMetrics,
    FeedbackRecord,
    EventType,
    WorkflowStatus,
)

__version__ = "1.0.0"
__author__ = "PhantomMesh Security"

__all__ = [
    # Threat Assessment
    "ThreatAssessor",
    "ThreatAssessment",
    "RiskScoreCalculator",
    "ConfidenceEstimator",
    "ImpactAnalyzer",
    "ContextualAnalyzer",
    "RiskLevel",
    "ConfidenceLevel",
    "ThreatSignal",
    # Alert Routing
    "AlertRoutingOrchestrator",
    "AlertRouter",
    "EscalationManager",
    "NotificationService",
    "AlertEnricher",
    "AlertSuppressionFilter",
    "AlertRoute",
    "EscalationPolicy",
    "AlertNotification",
    "RoutedAlert",
    "NotificationChannel",
    "EscalationLevel",
    # Auto-Remediation
    "RemediationOrchestrator",
    "RemediationPlaybook",
    "RemediationStep",
    "RemediationExecution",
    "ActionRecord",
    "RemediationAction",
    "ActionStatus",
    # Incident Response
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
    "IncidentStatus",
    "IncidentSeverity",
    "ForensicType",
    # ML Training Pipeline (Component 5)
    "MLTrainingOrchestrator",
    "DataPipeline",
    "FeatureEngineer",
    "ModelTrainer",
    "ModelEvaluator",
    "ModelRegistry",
    "OnlinePredictor",
    "ModelVersion",
    "PredictionResult",
    "TrainingJob",
    "FeatureVector",
    "TrainingExample",
    "ModelType",
    "TrainingStatus",
    # Integration & Orchestration (Component 6)
    "AutomationSystem",
    "SecurityEventBroker",
    "AutomationOrchestrator",
    "FeedbackLoop",
    "HealthMonitor",
    "ConfigurationManager",
    "SecurityEvent",
    "WorkflowExecution",
    "ComponentMetrics",
    "FeedbackRecord",
    "EventType",
    "WorkflowStatus",
]
