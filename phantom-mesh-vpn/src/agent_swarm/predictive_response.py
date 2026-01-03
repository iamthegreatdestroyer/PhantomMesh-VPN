"""
PhantomMesh Predictive Response Engine
======================================

Forecasts threats and optimizes automated response strategies
using ML-driven incident prediction and resource allocation.

Phase P1-003: Advanced Threat Intelligence Integration
Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

from __future__ import annotations

import asyncio
import numpy as np
from dataclasses import dataclass, field
from datetime import datetime, UTC, timedelta
from enum import Enum, auto
from typing import Any, Dict, List, Optional, Tuple
from collections import defaultdict, deque
import json

import structlog

logger = structlog.get_logger(__name__)


# ═══════════════════════════════════════════════════════════════════════════════
# THREAT FORECAST TYPES
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class ThreatEvent:
    """Recorded threat event for forecasting."""
    timestamp: datetime
    threat_type: str
    severity: float  # 0.0-1.0
    duration_minutes: float
    agent_response_time_ms: float
    resources_used: Dict[str, float]
    success: bool
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ThreatForecast:
    """Forecast of future threat likelihood."""
    forecast_start: datetime
    forecast_end: datetime
    threat_probability: float  # 0.0-1.0
    expected_threat_type: str
    expected_severity: float  # 0.0-1.0
    confidence: float
    critical_time_windows: List[Tuple[datetime, datetime]]
    resource_requirements: Dict[str, float]
    recommended_actions: List[str]
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "forecast_period": f"{self.forecast_start.isoformat()}/{self.forecast_end.isoformat()}",
            "threat_probability": round(self.threat_probability, 4),
            "expected_type": self.expected_threat_type,
            "expected_severity": round(self.expected_severity, 4),
            "confidence": round(self.confidence, 4),
            "critical_windows": [
                f"{start.isoformat()}/{end.isoformat()}"
                for start, end in self.critical_time_windows
            ],
            "resources": self.resource_requirements,
            "recommendations": self.recommended_actions
        }


@dataclass
class CriticalTimeWindow:
    """High-risk time period identified in forecast."""
    start_time: datetime
    end_time: datetime
    threat_probability: float
    recommended_preparedness_level: str  # low, medium, high, critical


@dataclass
class ResourceSnapshot:
    """Current resource availability."""
    cpu_available: float  # 0.0-1.0
    memory_available_mb: float
    network_bandwidth_mbps: float
    agents_available: int
    orchestration_capacity: float  # 0.0-1.0


@dataclass
class OptimizedResponse:
    """Optimized response strategy."""
    primary_action: str
    secondary_actions: List[str]
    resource_allocation: Dict[str, float]
    estimated_duration_minutes: float
    success_probability: float
    cost_estimate: float


@dataclass
class ResponseOutcome:
    """Outcome of executed response."""
    response_id: str
    threat_type: str
    start_time: datetime
    end_time: datetime
    success: bool
    threat_mitigated: bool
    resources_used: Dict[str, float]
    incidents_prevented: int
    false_positive: bool
    feedback_score: float  # 0.0-1.0


# ═══════════════════════════════════════════════════════════════════════════════
# THREAT FORECASTER
# ═══════════════════════════════════════════════════════════════════════════════

class ThreatForecaster:
    """
    Forecasts threats 24-72 hours in advance using time series analysis.
    Identifies critical time windows and risk periods.
    """
    
    def __init__(self):
        self._threat_history: deque[ThreatEvent] = deque(maxlen=10000)
        self._historical_patterns: Dict[str, List[ThreatEvent]] = defaultdict(list)
        self._seasonality_factors: Dict[str, float] = {}
        
        logger.info("threat_forecaster_initialized")
    
    async def forecast_threats(
        self,
        historical_events: List[ThreatEvent],
        current_threat_level: float,
        time_horizon: timedelta = timedelta(hours=48)
    ) -> ThreatForecast:
        """
        Forecast future threats using ARIMA/Prophet-style analysis.
        
        Forecasting approach:
        1. Analyze historical threat patterns
        2. Detect seasonal/cyclical patterns
        3. Compute trend components
        4. Forecast threat probability
        5. Identify critical time windows
        """
        
        # Add events to history
        for event in historical_events:
            self._threat_history.append(event)
            self._historical_patterns[event.threat_type].append(event)
        
        # Extract components
        trend = self._compute_trend()
        seasonality = self._detect_seasonality()
        current_momentum = current_threat_level
        
        # Forecast threat probability
        forecast_prob = self._forecast_probability(
            trend, seasonality, current_momentum, time_horizon
        )
        
        # Identify critical windows
        critical_windows = self._identify_critical_windows(
            forecast_prob, time_horizon
        )
        
        # Determine expected threat type
        expected_type = self._forecast_threat_type()
        
        # Estimate resource requirements
        resources = self._estimate_resource_requirements(
            forecast_prob, expected_type
        )
        
        # Generate recommendations
        recommendations = self._generate_recommendations(
            forecast_prob, expected_type, critical_windows
        )
        
        return ThreatForecast(
            forecast_start=datetime.now(UTC),
            forecast_end=datetime.now(UTC) + time_horizon,
            threat_probability=min(forecast_prob, 1.0),
            expected_threat_type=expected_type,
            expected_severity=forecast_prob,
            confidence=self._compute_forecast_confidence(),
            critical_time_windows=critical_windows,
            resource_requirements=resources,
            recommended_actions=recommendations
        )
    
    async def identify_critical_windows(
        self,
        forecast: ThreatForecast
    ) -> List[CriticalTimeWindow]:
        """Identify specific high-risk time periods."""
        
        critical = []
        
        for start, end in forecast.critical_time_windows:
            # Calculate threat probability during window
            window_prob = forecast.threat_probability
            
            if window_prob > 0.8:
                level = "critical"
            elif window_prob > 0.6:
                level = "high"
            elif window_prob > 0.4:
                level = "medium"
            else:
                level = "low"
            
            critical.append(CriticalTimeWindow(
                start_time=start,
                end_time=end,
                threat_probability=window_prob,
                recommended_preparedness_level=level
            ))
        
        return critical
    
    def _compute_trend(self) -> float:
        """Compute trend component from historical data."""
        if len(self._threat_history) < 2:
            return 0.0
        
        # Look at recent trend
        recent_events = list(self._threat_history)[-100:]
        severities = [e.severity for e in recent_events]
        
        if len(severities) < 2:
            return 0.0
        
        # Simple linear trend
        x = np.arange(len(severities))
        coefficients = np.polyfit(x, severities, 1)
        trend = coefficients[0]  # Slope
        
        return float(trend)
    
    def _detect_seasonality(self) -> Dict[str, float]:
        """Detect cyclical patterns (daily, weekly, monthly)."""
        if len(self._threat_history) < 24:
            return {}
        
        # Group by hour of day
        hourly_counts = defaultdict(int)
        for event in self._threat_history:
            hour = event.timestamp.hour
            hourly_counts[hour] += 1
        
        # Compute seasonality
        total = sum(hourly_counts.values())
        seasonality = {
            f"hour_{h}": count / total
            for h, count in hourly_counts.items()
        }
        
        self._seasonality_factors = seasonality
        return seasonality
    
    def _forecast_probability(
        self,
        trend: float,
        seasonality: Dict[str, float],
        current: float,
        horizon: timedelta
    ) -> float:
        """Forecast threat probability."""
        
        # Base probability from current threat level
        base_prob = current
        
        # Add trend component
        trend_component = trend * (horizon.total_seconds() / 3600)
        
        # Add seasonal component
        current_hour = datetime.now(UTC).hour
        seasonal_factor = seasonality.get(f"hour_{current_hour}", 0.5)
        
        # Combine components
        forecast = base_prob + trend_component + (seasonal_factor - 0.5) * 0.1
        
        return float(np.clip(forecast, 0.0, 1.0))
    
    def _identify_critical_windows(
        self,
        threat_prob: float,
        horizon: timedelta
    ) -> List[Tuple[datetime, datetime]]:
        """Identify time windows with elevated threat probability."""
        
        windows = []
        
        # Identify high-risk periods
        if threat_prob > 0.5:
            # Entire period is critical
            start = datetime.now(UTC)
            end = start + horizon
            windows.append((start, end))
        elif threat_prob > 0.3:
            # 50% of period is critical
            start = datetime.now(UTC) + horizon / 2
            end = start + horizon / 2
            windows.append((start, end))
        
        return windows
    
    def _forecast_threat_type(self) -> str:
        """Forecast most likely threat type."""
        
        if not self._historical_patterns:
            return "unknown"
        
        # Return most frequent threat type
        threat_counts = {
            threat_type: len(events)
            for threat_type, events in self._historical_patterns.items()
        }
        
        return max(threat_counts, key=threat_counts.get)
    
    def _estimate_resource_requirements(
        self,
        threat_prob: float,
        threat_type: str
    ) -> Dict[str, float]:
        """Estimate resources needed to handle forecasted threat."""
        
        base_resources = {
            "cpu_percent": 10.0,
            "memory_mb": 256.0,
            "agents_needed": 1.0,
            "network_mbps": 50.0
        }
        
        # Scale by threat probability
        scaling_factor = threat_prob
        
        return {
            k: v * scaling_factor
            for k, v in base_resources.items()
        }
    
    def _generate_recommendations(
        self,
        threat_prob: float,
        threat_type: str,
        critical_windows: List[Tuple[datetime, datetime]]
    ) -> List[str]:
        """Generate recommendations based on forecast."""
        
        recommendations = []
        
        if threat_prob > 0.8:
            recommendations.append("Increase monitoring intensity")
            recommendations.append("Pre-allocate response resources")
            recommendations.append("Alert security team")
        elif threat_prob > 0.6:
            recommendations.append("Enable enhanced logging")
            recommendations.append("Prepare incident playbook")
        elif threat_prob > 0.4:
            recommendations.append("Monitor threat indicators")
        
        if threat_type == "port_scan":
            recommendations.append("Enable port monitoring")
        elif threat_type == "brute_force":
            recommendations.append("Strengthen authentication")
        
        return recommendations
    
    def _compute_forecast_confidence(self) -> float:
        """Compute confidence in forecast."""
        
        # Confidence depends on data availability
        if len(self._threat_history) < 10:
            return 0.3
        elif len(self._threat_history) < 100:
            return 0.5
        elif len(self._threat_history) < 1000:
            return 0.7
        else:
            return 0.9


# ═══════════════════════════════════════════════════════════════════════════════
# RESPONSE OPTIMIZER
# ═══════════════════════════════════════════════════════════════════════════════

class ResponseOptimizer:
    """
    Optimizes threat response strategies using historical outcomes
    and current resource availability.
    """
    
    def __init__(self):
        self._response_history: Dict[str, List[ResponseOutcome]] = defaultdict(list)
        self._success_rates: Dict[str, float] = {}
        self._resource_efficiency: Dict[str, float] = {}
        
        logger.info("response_optimizer_initialized")
    
    async def optimize_response(
        self,
        threat: Dict[str, Any],
        available_resources: ResourceSnapshot,
        historical_outcomes: List[ResponseOutcome]
    ) -> OptimizedResponse:
        """
        Generate optimized response plan using ML.
        
        Optimization considers:
        1. Historical success rates of different responses
        2. Current resource availability
        3. Threat type and severity
        4. Time constraints
        """
        
        # Store outcomes
        for outcome in historical_outcomes:
            self._response_history[outcome.threat_type].append(outcome)
        
        # Compute success rates
        self._update_success_rates()
        
        threat_type = threat.get("type", "unknown")
        severity = threat.get("severity", 0.5)
        
        # Determine optimal response
        primary_action = self._select_primary_action(
            threat_type, available_resources
        )
        
        secondary_actions = self._select_secondary_actions(
            threat_type, severity, available_resources
        )
        
        # Allocate resources
        resource_allocation = self._allocate_resources(
            primary_action, secondary_actions, available_resources
        )
        
        # Estimate success
        success_prob = self._estimate_success_probability(
            primary_action, threat_type
        )
        
        return OptimizedResponse(
            primary_action=primary_action,
            secondary_actions=secondary_actions,
            resource_allocation=resource_allocation,
            estimated_duration_minutes=self._estimate_duration(primary_action),
            success_probability=success_prob,
            cost_estimate=self._estimate_cost(resource_allocation)
        )
    
    async def learn_from_outcome(
        self,
        response: OptimizedResponse,
        outcome: ResponseOutcome
    ):
        """Learn from response execution to improve future optimization."""
        
        self._response_history[outcome.threat_type].append(outcome)
        self._update_success_rates()
        
        logger.info(
            "response_outcome_recorded",
            threat_type=outcome.threat_type,
            success=outcome.success,
            feedback_score=round(outcome.feedback_score, 4)
        )
    
    def _select_primary_action(
        self,
        threat_type: str,
        resources: ResourceSnapshot
    ) -> str:
        """Select primary response action."""
        
        if threat_type == "port_scan":
            return "block_source_ip"
        elif threat_type == "brute_force":
            return "lock_account"
        elif threat_type == "dos_attack":
            return "enable_rate_limiting"
        elif threat_type == "malware":
            return "isolate_host"
        else:
            return "generic_investigation"
    
    def _select_secondary_actions(
        self,
        threat_type: str,
        severity: float,
        resources: ResourceSnapshot
    ) -> List[str]:
        """Select secondary response actions."""
        
        actions = ["capture_logs", "alert_team"]
        
        if severity > 0.7:
            actions.append("backup_critical_data")
        
        if resources.agents_available > 2:
            actions.append("parallel_investigation")
        
        return actions
    
    def _allocate_resources(
        self,
        primary: str,
        secondary: List[str],
        available: ResourceSnapshot
    ) -> Dict[str, float]:
        """Allocate resources optimally."""
        
        allocation = {
            "cpu_percent": 10.0,
            "memory_mb": 256.0,
            "agents": 1.0,
            "network_mbps": 50.0
        }
        
        # Adjust for action severity
        if primary in ["isolate_host", "block_source_ip"]:
            allocation["agents"] = 2.0
        
        if len(secondary) > 2:
            allocation["cpu_percent"] = 20.0
        
        # Ensure we don't exceed available resources
        allocation["agents"] = min(
            allocation["agents"],
            available.agents_available
        )
        
        return allocation
    
    def _estimate_success_probability(
        self,
        action: str,
        threat_type: str
    ) -> float:
        """Estimate success probability from historical data."""
        
        if threat_type not in self._response_history:
            return 0.7  # Default estimate
        
        outcomes = self._response_history[threat_type]
        if not outcomes:
            return 0.7
        
        successes = sum(1 for o in outcomes if o.success)
        return successes / len(outcomes)
    
    def _estimate_duration(self, action: str) -> float:
        """Estimate response duration in minutes."""
        
        durations = {
            "block_source_ip": 0.5,
            "lock_account": 2.0,
            "enable_rate_limiting": 1.0,
            "isolate_host": 5.0,
            "generic_investigation": 15.0
        }
        
        return durations.get(action, 10.0)
    
    def _estimate_cost(self, resources: Dict[str, float]) -> float:
        """Estimate operational cost of response."""
        
        # Simple cost model
        cost = 0.0
        cost += resources.get("cpu_percent", 0) * 0.01
        cost += resources.get("memory_mb", 0) * 0.001
        cost += resources.get("agents", 0) * 10.0
        cost += resources.get("network_mbps", 0) * 0.1
        
        return cost
    
    def _update_success_rates(self):
        """Update success rates from response history."""
        
        for threat_type, outcomes in self._response_history.items():
            if not outcomes:
                self._success_rates[threat_type] = 0.7
                continue
            
            successes = sum(1 for o in outcomes if o.success)
            rate = successes / len(outcomes)
            self._success_rates[threat_type] = rate


# ═══════════════════════════════════════════════════════════════════════════════
# PLAYBOOK SELECTOR
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class SelectedPlaybook:
    """Selected incident playbook."""
    playbook_id: str
    playbook_name: str
    steps: List[str]
    estimated_execution_time: float
    required_roles: List[str]
    success_rate: float


class PlaybookSelector:
    """Intelligent incident playbook selection based on history."""
    
    def __init__(self):
        self.playbooks = {
            "port_scan_response": {
                "steps": [
                    "Identify scanner source",
                    "Block source IP",
                    "Capture traffic samples",
                    "Log incident",
                    "Alert team"
                ],
                "agents": ["FORTRESS"],
                "estimated_time": 2.0
            },
            "brute_force_response": {
                "steps": [
                    "Lock target account",
                    "Reset credentials",
                    "Enable MFA",
                    "Notify user",
                    "Monitor account"
                ],
                "agents": ["AEGIS", "CIPHER"],
                "estimated_time": 5.0
            },
            "dos_mitigation": {
                "steps": [
                    "Enable rate limiting",
                    "Increase capacity",
                    "Filter traffic",
                    "Monitor metrics",
                    "Escalate if needed"
                ],
                "agents": ["VELOCITY", "STREAM"],
                "estimated_time": 3.0
            }
        }
        
        self._success_history: Dict[str, List[bool]] = defaultdict(list)
    
    async def select_playbook(
        self,
        threat: Dict[str, Any],
        context: Dict[str, Any],
        success_history: Dict[str, float]
    ) -> SelectedPlaybook:
        """Select best playbook for incident."""
        
        threat_type = threat.get("type", "unknown")
        
        # Map threat to playbook
        playbook_mapping = {
            "port_scan": "port_scan_response",
            "brute_force": "brute_force_response",
            "ssh_brute_force": "brute_force_response",
            "dos_attack": "dos_mitigation"
        }
        
        playbook_id = playbook_mapping.get(threat_type, "generic_response")
        
        if playbook_id not in self.playbooks:
            playbook_id = "generic_response"
        
        playbook_data = self.playbooks[playbook_id]
        success_rate = success_history.get(playbook_id, 0.7)
        
        return SelectedPlaybook(
            playbook_id=playbook_id,
            playbook_name=playbook_id.replace("_", " ").title(),
            steps=playbook_data["steps"],
            estimated_execution_time=playbook_data["estimated_time"],
            required_roles=playbook_data["agents"],
            success_rate=success_rate
        )
    
    async def record_execution(
        self,
        playbook_id: str,
        success: bool
    ):
        """Record playbook execution result."""
        self._success_history[playbook_id].append(success)
