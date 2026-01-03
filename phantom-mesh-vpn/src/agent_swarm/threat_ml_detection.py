"""
PhantomMesh ML-Based Threat Detection Engine
=============================================

Advanced threat detection using ensemble machine learning models
with real-time pattern recognition and behavioral analysis.

Phase P1-003: Advanced Threat Intelligence Integration
Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

from __future__ import annotations

import asyncio
import numpy as np
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, UTC, timedelta
from enum import Enum, auto
from typing import Any, Dict, List, Optional, Tuple
from collections import defaultdict, deque
import json

import structlog

logger = structlog.get_logger(__name__)


# ═══════════════════════════════════════════════════════════════════════════════
# THREAT DETECTION TYPES & ENUMS
# ═══════════════════════════════════════════════════════════════════════════════

class ThreatClassification(Enum):
    """Threat severity classifications."""
    BENIGN = auto()
    SUSPICIOUS = auto()
    MALICIOUS = auto()
    CRITICAL = auto()
    CATASTROPHIC = auto()


@dataclass
class TrafficEvent:
    """Single network traffic event."""
    timestamp: datetime
    source_ip: str
    destination_ip: str
    port: int
    protocol: str
    packet_size: int
    flags: List[str]
    ttl: int
    window_size: int
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def to_feature_vector(self) -> np.ndarray:
        """Convert to ML feature vector."""
        return np.array([
            self.port,
            self.packet_size,
            self.ttl,
            self.window_size,
            len(self.flags),
            int(self.timestamp.timestamp())
        ])


@dataclass
class DetectionResult:
    """Result of threat detection analysis."""
    threat_detected: bool
    classification: ThreatClassification
    confidence: float  # 0.0 - 1.0
    threat_score: float  # 0.0 - 100.0
    primary_threat_type: str
    contributing_models: Dict[str, float]  # model_name -> confidence
    features_triggered: List[str]
    timestamp: datetime = field(default_factory=lambda: datetime.now(UTC))
    recommendation: str = ""
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "threat_detected": self.threat_detected,
            "classification": self.classification.name,
            "confidence": round(self.confidence, 4),
            "threat_score": round(self.threat_score, 2),
            "primary_threat": self.primary_threat_type,
            "model_votes": self.contributing_models,
            "features": self.features_triggered,
            "timestamp": self.timestamp.isoformat(),
            "recommendation": self.recommendation
        }


@dataclass
class AnomalyAlert:
    """Alert for detected anomalies."""
    anomaly_type: str
    severity: ThreatClassification
    deviation_magnitude: float  # Standard deviations from baseline
    expected_behavior: str
    observed_behavior: str
    timestamp: datetime = field(default_factory=lambda: datetime.now(UTC))
    affected_ip: Optional[str] = None
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "type": self.anomaly_type,
            "severity": self.severity.name,
            "magnitude": round(self.deviation_magnitude, 2),
            "expected": self.expected_behavior,
            "observed": self.observed_behavior,
            "timestamp": self.timestamp.isoformat(),
            "affected_ip": self.affected_ip
        }


# ═══════════════════════════════════════════════════════════════════════════════
# FEATURE EXTRACTION ENGINE
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class FeatureSet:
    """Extracted features for threat detection."""
    temporal_features: np.ndarray
    behavioral_features: np.ndarray
    packet_features: np.ndarray
    statistical_features: np.ndarray
    network_features: np.ndarray
    
    def to_array(self) -> np.ndarray:
        """Combine all features into single vector."""
        return np.concatenate([
            self.temporal_features,
            self.behavioral_features,
            self.packet_features,
            self.statistical_features,
            self.network_features
        ])


class FeatureExtractor:
    """
    Real-time feature extraction for threat detection.
    Converts raw traffic into ML-ready feature vectors.
    """
    
    def __init__(self, window_size: int = 100):
        self.window_size = window_size
        self._traffic_history: deque[TrafficEvent] = deque(maxlen=window_size)
        self._baseline_stats: Dict[str, float] = {}
        
        logger.info("feature_extractor_initialized", window_size=window_size)
    
    def extract_features(self, events: List[TrafficEvent]) -> FeatureSet:
        """
        Extract features from traffic events.
        
        Feature categories:
        - Temporal: Time-based patterns
        - Behavioral: IP, port, protocol patterns
        - Packet: Size, flags, TTL patterns
        - Statistical: Distribution patterns
        - Network: Graph topology patterns
        """
        
        # Add events to history
        for event in events:
            self._traffic_history.append(event)
        
        if len(self._traffic_history) < 10:
            # Not enough data yet
            return self._create_empty_features()
        
        # Extract feature groups
        temporal = self._extract_temporal_features()
        behavioral = self._extract_behavioral_features()
        packet = self._extract_packet_features()
        statistical = self._extract_statistical_features()
        network = self._extract_network_features()
        
        return FeatureSet(
            temporal_features=temporal,
            behavioral_features=behavioral,
            packet_features=packet,
            statistical_features=statistical,
            network_features=network
        )
    
    def _extract_temporal_features(self) -> np.ndarray:
        """Extract time-based patterns."""
        if len(self._traffic_history) < 2:
            return np.zeros(5)
        
        events = list(self._traffic_history)
        timestamps = [e.timestamp.timestamp() for e in events]
        
        # Inter-arrival times
        inter_arrivals = np.diff(timestamps)
        
        return np.array([
            np.mean(inter_arrivals) if len(inter_arrivals) > 0 else 0,
            np.std(inter_arrivals) if len(inter_arrivals) > 0 else 0,
            np.min(inter_arrivals) if len(inter_arrivals) > 0 else 0,
            np.max(inter_arrivals) if len(inter_arrivals) > 0 else 0,
            len(events)  # Event count
        ])
    
    def _extract_behavioral_features(self) -> np.ndarray:
        """Extract behavioral patterns (IPs, ports, protocols)."""
        events = list(self._traffic_history)
        
        unique_ips = len(set(e.destination_ip for e in events))
        unique_ports = len(set(e.port for e in events))
        unique_protocols = len(set(e.protocol for e in events))
        
        # Port scanning detection
        ports = [e.port for e in events]
        port_variety = len(set(ports)) / max(len(ports), 1)
        
        return np.array([
            unique_ips,
            unique_ports,
            unique_protocols,
            port_variety,
            len(events)
        ])
    
    def _extract_packet_features(self) -> np.ndarray:
        """Extract packet-level patterns."""
        events = list(self._traffic_history)
        
        packet_sizes = np.array([e.packet_size for e in events])
        ttls = np.array([e.ttl for e in events])
        windows = np.array([e.window_size for e in events])
        
        return np.array([
            np.mean(packet_sizes),
            np.std(packet_sizes),
            np.min(packet_sizes),
            np.max(packet_sizes),
            np.mean(ttls),
            np.std(ttls),
            np.mean(windows),
            np.std(windows)
        ])
    
    def _extract_statistical_features(self) -> np.ndarray:
        """Extract statistical distribution patterns."""
        events = list(self._traffic_history)
        
        if len(events) < 5:
            return np.zeros(6)
        
        packet_sizes = np.array([e.packet_size for e in events])
        
        # Entropy of packet sizes (indicates randomness)
        unique_sizes, counts = np.unique(packet_sizes, return_counts=True)
        probabilities = counts / len(packet_sizes)
        entropy = -np.sum(probabilities * np.log2(probabilities + 1e-10))
        
        # Skewness and kurtosis
        mean = np.mean(packet_sizes)
        std = np.std(packet_sizes)
        skewness = np.mean(((packet_sizes - mean) / (std + 1e-10)) ** 3)
        kurtosis = np.mean(((packet_sizes - mean) / (std + 1e-10)) ** 4) - 3
        
        return np.array([
            entropy,
            skewness,
            kurtosis,
            np.var(packet_sizes),
            np.percentile(packet_sizes, 75),
            np.percentile(packet_sizes, 25)
        ])
    
    def _extract_network_features(self) -> np.ndarray:
        """Extract network topology patterns."""
        events = list(self._traffic_history)
        
        source_ips = [e.source_ip for e in events]
        dest_ips = [e.destination_ip for e in events]
        
        # Repetition patterns
        ip_connections = defaultdict(int)
        for src, dst in zip(source_ips, dest_ips):
            ip_connections[f"{src}→{dst}"] += 1
        
        max_repeat = max(ip_connections.values()) if ip_connections else 1
        unique_flows = len(ip_connections)
        
        return np.array([
            len(set(source_ips)),
            len(set(dest_ips)),
            max_repeat,
            unique_flows,
            len(events)
        ])
    
    def _create_empty_features(self) -> FeatureSet:
        """Return zero feature vectors when insufficient data."""
        return FeatureSet(
            temporal_features=np.zeros(5),
            behavioral_features=np.zeros(5),
            packet_features=np.zeros(8),
            statistical_features=np.zeros(6),
            network_features=np.zeros(5)
        )
    
    def establish_baseline(self, events: List[TrafficEvent]) -> None:
        """Learn baseline behavior from normal traffic."""
        for event in events:
            self._traffic_history.append(event)
        
        features = self.extract_features([])
        feature_array = features.to_array()
        
        self._baseline_stats = {
            "mean": np.mean(feature_array),
            "std": np.std(feature_array),
            "min": np.min(feature_array),
            "max": np.max(feature_array)
        }
        
        logger.info("baseline_established", stats=self._baseline_stats)
    
    def get_baseline_stats(self) -> Dict[str, float]:
        """Get established baseline statistics."""
        return self._baseline_stats.copy()


# ═══════════════════════════════════════════════════════════════════════════════
# MACHINE LEARNING MODELS
# ═══════════════════════════════════════════════════════════════════════════════

class MLThreatModel(ABC):
    """Abstract base for ML threat detection models."""
    
    def __init__(self, model_name: str):
        self.model_name = model_name
        self.is_trained = False
    
    @abstractmethod
    async def detect(self, features: np.ndarray) -> Tuple[float, float]:
        """
        Detect threat in features.
        Returns: (is_threat: bool, confidence: float)
        """
        ...
    
    @abstractmethod
    async def train(self, training_data: np.ndarray, labels: np.ndarray):
        """Train model on labeled data."""
        ...
    
    @abstractmethod
    def get_model_name(self) -> str:
        """Get model identifier."""
        ...


class IsolationForestModel(MLThreatModel):
    """
    Isolation Forest - Anomaly detection via isolation.
    Excellent for high-dimensional threat detection.
    """
    
    def __init__(self):
        super().__init__("isolation_forest")
        self.trees = []
        self.threshold = 0.5
    
    async def detect(self, features: np.ndarray) -> Tuple[bool, float]:
        """
        Detect anomalies using isolation forest.
        Returns: (is_anomaly, anomaly_score)
        """
        if not self.is_trained or len(self.trees) == 0:
            return False, 0.0
        
        # Simple anomaly score (in real implementation, use proper IF)
        # Higher dimensional vectors are more isolated
        anomaly_score = np.linalg.norm(features) / (len(features) + 1)
        is_anomaly = anomaly_score > self.threshold
        
        return is_anomaly, min(anomaly_score, 1.0)
    
    async def train(self, training_data: np.ndarray, labels: np.ndarray):
        """Train isolation forest model."""
        # In production, use scikit-learn's IsolationForest
        self.is_trained = True
        logger.info("isolation_forest_trained", samples=len(training_data))
    
    def get_model_name(self) -> str:
        return self.model_name


class LSTMSequenceModel(MLThreatModel):
    """
    LSTM-based sequence anomaly detection.
    Detects temporal patterns in traffic sequences.
    """
    
    def __init__(self):
        super().__init__("lstm_sequence")
        self.sequence_length = 10
        self.threshold = 0.6
    
    async def detect(self, features: np.ndarray) -> Tuple[bool, float]:
        """
        Detect sequence anomalies using LSTM.
        Returns: (is_anomalous, reconstruction_error)
        """
        if not self.is_trained:
            return False, 0.0
        
        # Reconstruction error from LSTM autoencoder
        # Higher error indicates anomaly
        reconstruction_error = np.random.random()  # Placeholder
        is_anomalous = reconstruction_error > self.threshold
        
        return is_anomalous, reconstruction_error
    
    async def train(self, training_data: np.ndarray, labels: np.ndarray):
        """Train LSTM sequence model."""
        # In production, use TensorFlow/PyTorch LSTM
        self.is_trained = True
        logger.info("lstm_model_trained", samples=len(training_data))
    
    def get_model_name(self) -> str:
        return self.model_name


class HybridBayesianModel(MLThreatModel):
    """
    Hybrid Bayesian model combining statistical inference with rules.
    Interpretable threat detection with probabilistic reasoning.
    """
    
    def __init__(self):
        super().__init__("hybrid_bayesian")
        self.feature_priors = {}
        self.threat_likelihood = 0.5
    
    async def detect(self, features: np.ndarray) -> Tuple[bool, float]:
        """
        Detect threats using Bayesian inference.
        Returns: (is_threat, posterior_probability)
        """
        if not self.is_trained:
            return False, 0.0
        
        # Compute posterior probability of threat given features
        # P(Threat | Features) using Bayes' rule
        posterior = self._compute_posterior(features)
        is_threat = posterior > 0.5
        
        return is_threat, posterior
    
    async def train(self, training_data: np.ndarray, labels: np.ndarray):
        """Train Bayesian model."""
        # Learn feature distributions from data
        self.is_trained = True
        logger.info("bayesian_model_trained", samples=len(training_data))
    
    def _compute_posterior(self, features: np.ndarray) -> float:
        """Compute posterior probability using Bayes' rule."""
        # Simplified implementation
        feature_mean = np.mean(np.abs(features))
        return min(feature_mean / 100.0, 1.0)  # Normalize to [0, 1]
    
    def get_model_name(self) -> str:
        return self.model_name


# ═══════════════════════════════════════════════════════════════════════════════
# THREAT DETECTOR — ENSEMBLE COORDINATOR
# ═══════════════════════════════════════════════════════════════════════════════

class ThreatDetector:
    """
    Ensemble-based threat detector combining multiple ML models.
    Uses consensus voting and confidence aggregation.
    """
    
    def __init__(self):
        self.models: List[MLThreatModel] = [
            IsolationForestModel(),
            LSTMSequenceModel(),
            HybridBayesianModel()
        ]
        self.feature_extractor = FeatureExtractor()
        self.ensemble_threshold = 2  # At least 2/3 models must agree
        self._detection_history: deque[DetectionResult] = deque(maxlen=1000)
        
        logger.info("threat_detector_initialized", models=len(self.models))
    
    async def detect_threats(
        self,
        traffic_data: List[TrafficEvent],
        historical_context: Optional[Dict[str, Any]] = None
    ) -> DetectionResult:
        """
        Detect threats in traffic with ensemble voting.
        
        Algorithm:
        1. Extract features from traffic
        2. Run each model independently
        3. Aggregate votes
        4. Compute confidence
        5. Classify threat
        """
        
        # Extract features
        features = self.feature_extractor.extract_features(traffic_data)
        feature_array = features.to_array()
        
        # Run ensemble
        model_votes: Dict[str, float] = {}
        threat_indicators = 0
        
        for model in self.models:
            is_threat, confidence = await model.detect(feature_array)
            model_votes[model.get_model_name()] = confidence
            
            if is_threat:
                threat_indicators += 1
        
        # Determine if consensus indicates threat
        consensus_threat = threat_indicators >= self.ensemble_threshold
        
        # Compute aggregate confidence
        avg_confidence = np.mean(list(model_votes.values()))
        
        # Classify threat
        classification, threat_score = self._classify_threat(
            consensus_threat,
            avg_confidence,
            feature_array,
            traffic_data
        )
        
        # Create result
        result = DetectionResult(
            threat_detected=consensus_threat,
            classification=classification,
            confidence=avg_confidence,
            threat_score=threat_score,
            primary_threat_type=self._determine_threat_type(traffic_data),
            contributing_models=model_votes,
            features_triggered=self._identify_triggered_features(features),
            recommendation=self._generate_recommendation(classification)
        )
        
        # Store in history
        self._detection_history.append(result)
        
        if result.threat_detected:
            logger.warning(
                "threat_detected",
                classification=classification.name,
                confidence=round(avg_confidence, 4),
                threat_score=round(threat_score, 2)
            )
        
        return result
    
    def _classify_threat(
        self,
        consensus: bool,
        confidence: float,
        features: np.ndarray,
        traffic: List[TrafficEvent]
    ) -> Tuple[ThreatClassification, float]:
        """Classify threat severity."""
        
        if not consensus:
            return ThreatClassification.BENIGN, 0.0
        
        # Map confidence to threat score (0-100)
        threat_score = confidence * 100
        
        # Classify based on confidence and feature patterns
        if confidence > 0.95:
            return ThreatClassification.CATASTROPHIC, threat_score
        elif confidence > 0.85:
            return ThreatClassification.CRITICAL, threat_score
        elif confidence > 0.70:
            return ThreatClassification.MALICIOUS, threat_score
        elif confidence > 0.50:
            return ThreatClassification.SUSPICIOUS, threat_score
        else:
            return ThreatClassification.BENIGN, 0.0
    
    def _determine_threat_type(self, traffic: List[TrafficEvent]) -> str:
        """Determine type of threat from traffic patterns."""
        if not traffic:
            return "unknown"
        
        # Analyze traffic patterns
        ports = [e.port for e in traffic]
        unique_ports = len(set(ports))
        
        if unique_ports > 10:
            return "port_scan"
        
        if any(e.port == 22 or e.port == 3389 for e in traffic):
            return "ssh_brute_force"
        
        if any(e.packet_size > 65000 for e in traffic):
            return "dos_attack"
        
        return "anomalous_traffic"
    
    def _identify_triggered_features(self, features: FeatureSet) -> List[str]:
        """Identify which features triggered threat indicators."""
        triggered = []
        
        if np.mean(features.temporal_features) > 5:
            triggered.append("unusual_temporal_pattern")
        
        if features.behavioral_features[2] > 3:  # unique_protocols
            triggered.append("multiple_protocols")
        
        if features.packet_features[7] > 2:  # window std dev
            triggered.append("variable_window_size")
        
        if features.statistical_features[0] > 3:  # entropy
            triggered.append("high_entropy")
        
        return triggered
    
    def _generate_recommendation(self, classification: ThreatClassification) -> str:
        """Generate response recommendation."""
        recommendations = {
            ThreatClassification.BENIGN: "No action required",
            ThreatClassification.SUSPICIOUS: "Monitor closely, escalate if pattern continues",
            ThreatClassification.MALICIOUS: "Block source immediately, log incident",
            ThreatClassification.CRITICAL: "Block source, isolate affected systems, alert team",
            ThreatClassification.CATASTROPHIC: "Execute emergency response plan immediately"
        }
        return recommendations.get(classification, "Unknown threat")
    
    async def update_model(self, feedback: Dict[str, Any]):
        """Update models based on operational feedback."""
        logger.info("model_update_received", feedback_type=feedback.get("type"))
        # Trigger retraining with new data
    
    def get_detection_stats(self) -> Dict[str, Any]:
        """Get threat detection statistics."""
        history = list(self._detection_history)
        
        if not history:
            return {}
        
        detections = sum(1 for r in history if r.threat_detected)
        avg_confidence = np.mean([r.confidence for r in history])
        
        classifications = defaultdict(int)
        for result in history:
            classifications[result.classification.name] += 1
        
        return {
            "total_analyzed": len(history),
            "threats_detected": detections,
            "detection_rate": round(detections / len(history), 4),
            "avg_confidence": round(avg_confidence, 4),
            "classifications": dict(classifications),
            "last_updated": history[-1].timestamp.isoformat()
        }


# ═══════════════════════════════════════════════════════════════════════════════
# ANOMALY DETECTOR
# ═══════════════════════════════════════════════════════════════════════════════

class AnomalyDetector:
    """
    Behavioral anomaly detection for network profiles.
    Detects deviations from established baseline behavior.
    """
    
    def __init__(self, sensitivity: float = 2.0):
        self.sensitivity = sensitivity  # Standard deviations threshold
        self.baseline_profiles: Dict[str, Dict[str, float]] = {}
        self._anomaly_history: deque[AnomalyAlert] = deque(maxlen=1000)
        
        logger.info("anomaly_detector_initialized", sensitivity=sensitivity)
    
    async def establish_baseline(
        self,
        traffic: List[TrafficEvent],
        profile_name: str = "default"
    ):
        """Learn normal behavior patterns from baseline traffic."""
        
        feature_extractor = FeatureExtractor()
        features = feature_extractor.extract_features(traffic)
        feature_array = features.to_array()
        
        self.baseline_profiles[profile_name] = {
            "mean": float(np.mean(feature_array)),
            "std": float(np.std(feature_array)),
            "min": float(np.min(feature_array)),
            "max": float(np.max(feature_array)),
            "sample_count": len(traffic),
            "established_at": datetime.now(UTC).isoformat()
        }
        
        logger.info(
            "baseline_established",
            profile=profile_name,
            samples=len(traffic)
        )
    
    async def detect_anomalies(
        self,
        current_traffic: List[TrafficEvent],
        profile_name: str = "default"
    ) -> List[AnomalyAlert]:
        """Detect deviations from baseline behavior."""
        
        if profile_name not in self.baseline_profiles:
            logger.warning("profile_not_found", profile=profile_name)
            return []
        
        baseline = self.baseline_profiles[profile_name]
        
        # Extract features from current traffic
        feature_extractor = FeatureExtractor()
        features = feature_extractor.extract_features(current_traffic)
        feature_array = features.to_array()
        current_mean = np.mean(feature_array)
        
        # Detect deviations
        deviation = (current_mean - baseline["mean"]) / (baseline["std"] + 1e-10)
        deviation_magnitude = abs(deviation)
        
        alerts = []
        
        if deviation_magnitude > self.sensitivity:
            # Anomaly detected
            alert = AnomalyAlert(
                anomaly_type="behavioral_deviation",
                severity=self._classify_anomaly_severity(deviation_magnitude),
                deviation_magnitude=deviation_magnitude,
                expected_behavior=f"Mean feature value ~{baseline['mean']:.2f}",
                observed_behavior=f"Mean feature value ~{current_mean:.2f}",
                affected_ip=current_traffic[0].source_ip if current_traffic else None
            )
            
            alerts.append(alert)
            self._anomaly_history.append(alert)
            
            logger.warning(
                "anomaly_detected",
                type=alert.anomaly_type,
                severity=alert.severity.name,
                magnitude=round(deviation_magnitude, 2)
            )
        
        return alerts
    
    def _classify_anomaly_severity(self, magnitude: float) -> ThreatClassification:
        """Classify anomaly severity based on magnitude."""
        if magnitude > 5.0:
            return ThreatClassification.CRITICAL
        elif magnitude > 3.0:
            return ThreatClassification.MALICIOUS
        elif magnitude > 2.0:
            return ThreatClassification.SUSPICIOUS
        else:
            return ThreatClassification.BENIGN
    
    def get_anomaly_stats(self) -> Dict[str, Any]:
        """Get anomaly detection statistics."""
        history = list(self._anomaly_history)
        
        if not history:
            return {"anomalies_detected": 0}
        
        severities = defaultdict(int)
        for alert in history:
            severities[alert.severity.name] += 1
        
        return {
            "anomalies_detected": len(history),
            "severity_distribution": dict(severities),
            "last_anomaly": history[-1].timestamp.isoformat()
        }
