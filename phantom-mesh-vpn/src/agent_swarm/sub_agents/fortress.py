"""
FORTRESS Agent — Threat Detection & Defense
===========================================
Elite agent for threat detection and intrusion prevention.

Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

from typing import Any, Dict, List, Optional
import asyncio
import structlog
import numpy as np
from datetime import datetime, timedelta

from ..threat_integration import ThreatEngineClient, IntegratedThreatSystem
from ..phantom_orchestrator import EliteAgent, PhantomOrchestrator, MnemonicCache, AgentRole

logger = structlog.get_logger(__name__)


class ThreatSignature:
    """Threat signature for pattern matching."""

    def __init__(self, signature_id: str, pattern: bytes, description: str,
                 severity: str, category: str):
        self.id = signature_id
        self.pattern = pattern
        self.description = description
        self.severity = severity
        self.category = category


class AnomalyDetector:
    """Statistical anomaly detection for network traffic."""

    def __init__(self, window_size: int = 1000, threshold: float = 3.0):
        self.window_size = window_size
        self.threshold = threshold
        self.packet_sizes: List[int] = []
        self.timestamps: List[datetime] = []

    def add_packet(self, size: int) -> None:
        """Add packet to analysis window."""
        now = datetime.now()
        self.packet_sizes.append(size)
        self.timestamps.append(now)

        # Maintain window size
        if len(self.packet_sizes) > self.window_size:
            self.packet_sizes.pop(0)
            self.timestamps.pop(0)

    def detect_anomaly(self, size: int) -> Optional[Dict[str, Any]]:
        """Detect if packet size is anomalous."""
        if len(self.packet_sizes) < 10:  # Need minimum samples
            return None

        mean = np.mean(self.packet_sizes)
        std = np.std(self.packet_sizes)

        if std == 0:
            return None

        z_score = abs(size - mean) / std

        if z_score > self.threshold:
            return {
                "type": "packet_size_anomaly",
                "z_score": z_score,
                "mean": mean,
                "std": std,
                "packet_size": size,
                "confidence": min(z_score / 5.0, 1.0)
            }

        return None

    def detect_ddos(self) -> Optional[Dict[str, Any]]:
        """Detect potential DDoS attacks based on traffic patterns."""
        if len(self.timestamps) < 50:
            return None

        # Calculate packets per second over last minute
        now = datetime.now()
        recent_packets = [t for t in self.timestamps
                         if (now - t).total_seconds() < 60]

        pps = len(recent_packets) / 60.0

        # Threshold for DDoS detection (adjust based on network capacity)
        if pps > 1000:  # More than 1000 packets/second
            return {
                "type": "ddos_suspicion",
                "packets_per_second": pps,
                "time_window": 60,
                "confidence": min(pps / 2000.0, 1.0)
            }

        return None


class FortressAgent(EliteAgent):
    """
    FORTRESS Agent: Threat detection and defense.

    Responsibilities:
    - Anomaly detection algorithms
    - Threat signature analysis
    - Intrusion prevention systems
    - Security incident response
    - Integration with Rust ThreatEngine
    """

    def __init__(self, role: AgentRole, orchestrator: PhantomOrchestrator, mnemonic: MnemonicCache):
        super().__init__(role, orchestrator, mnemonic)
        self.threat_patterns = {}
        self.defense_measures = []
        self.anomaly_detector = AnomalyDetector()
        self.threat_intel_cache = {}
        self.active_alerts = set()
        self.quarantined_peers = set()

        # Initialize threat integration
        self.threat_system = IntegratedThreatSystem()
        self.rust_engine_available = False

        # Initialize default threat signatures
        self._load_default_signatures()

    async def initialize(self):
        """Initialize the FORTRESS agent."""
        self.rust_engine_available = await self.threat_system.initialize()
        if self.rust_engine_available:
            logger.info("fortress_agent_initialized", rust_engine="available")
        else:
            logger.warning("fortress_agent_initialized", rust_engine="unavailable")

    async def execute_mission(self, directive: str, context: Dict[str, Any]) -> Dict[str, Any]:
        """Execute threat detection mission."""
        logger.info("fortress_mission_executed", directive=directive[:100])

        mission_type = context.get("mission_type", "analyze")

        if mission_type == "analyze_packet":
            return await self._analyze_packet(context)
        elif mission_type == "anomaly_detection":
            return await self._perform_anomaly_detection(context)
        elif mission_type == "threat_intel_update":
            return await self._update_threat_intelligence(context)
        elif mission_type == "incident_response":
            return await self._handle_incident_response(context)
        elif mission_type == "batch_analysis":
            return await self._analyze_traffic_batch(context)
        else:
            return await self._general_threat_analysis(context)

    def _load_default_signatures(self) -> None:
        """Load default threat signatures."""
        signatures = [
            ThreatSignature(
                "malware_pattern_1",
                b"\x90\x90\x90\x90",  # NOP sled
                "Potential shellcode with NOP sled",
                "high",
                "malware"
            ),
            ThreatSignature(
                "exploit_buffer_overflow",
                b"A" * 100,  # Long A sequence
                "Buffer overflow attempt",
                "critical",
                "exploit"
            ),
            ThreatSignature(
                "suspicious_command_injection",
                b"; rm -rf",  # Command injection
                "Command injection attempt",
                "high",
                "exploit"
            ),
        ]

        for sig in signatures:
            self.threat_patterns[sig.id] = sig
            # Cache in mnemonic system
            self.mnemonic.store(
                f"threat_sig_{sig.id}",
                {
                    "pattern": sig.pattern.hex(),
                    "description": sig.description,
                    "severity": sig.severity,
                    "category": sig.category
                },
                priority=3  # High priority for threat data
            )

    async def execute_mission(self, directive: str, context: Dict[str, Any]) -> Dict[str, Any]:
        """Execute threat detection mission."""
        logger.info("fortress_mission_executed", directive=directive[:100])

        mission_type = context.get("mission_type", "analyze")

        if mission_type == "analyze_packet":
            return await self._analyze_packet(context)
        elif mission_type == "anomaly_detection":
            return await self._perform_anomaly_detection(context)
        elif mission_type == "threat_intel_update":
            return await self._update_threat_intelligence(context)
        elif mission_type == "incident_response":
            return await self._handle_incident_response(context)
        else:
            return await self._general_threat_analysis(context)

    async def _analyze_packet(self, context: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze a packet for threats."""
        packet_data = context.get("packet", b"")
        source_ip = context.get("source_ip")

        # Update anomaly detector
        packet_size = len(packet_data)
        self.anomaly_detector.add_packet(packet_size)

        threats_detected = []

        # Use integrated threat system if available
        if self.rust_engine_available:
            try:
                # Convert packet to bytes if needed
                if isinstance(packet_data, str):
                    packet_bytes = packet_data.encode()
                elif isinstance(packet_data, bytes):
                    packet_bytes = packet_data
                else:
                    packet_bytes = str(packet_data).encode()

                # Analyze with Rust engine
                result = await self.threat_system.analyze_packet(packet_bytes, source_ip or "unknown")

                if result.get("threat_detected"):
                    threat_result = result["threat_result"]
                    threats_detected.append({
                        "signature_id": threat_result.get("signature_id", "rust_detected"),
                        "description": threat_result.get("description", "Threat detected by Rust engine"),
                        "severity": threat_result.get("severity", "medium"),
                        "category": threat_result.get("category", "unknown"),
                        "confidence": threat_result.get("confidence", 0.8),
                        "source_ip": source_ip,
                        "rust_engine": True
                    })

                    # Cache threat detection
                    self.mnemonic.store(
                        f"threat_detection_{threat_result.get('signature_id', 'rust')}_{datetime.now().isoformat()}",
                        threat_result,
                        priority=5
                    )

            except Exception as e:
                logger.warning("rust_engine_analysis_failed", error=str(e))
                # Fall back to Python analysis

        # Fallback: Signature-based detection
        for sig_id, signature in self.threat_patterns.items():
            if signature.pattern in packet_data:
                threat = {
                    "signature_id": sig_id,
                    "description": signature.description,
                    "severity": signature.severity,
                    "category": signature.category,
                    "confidence": 1.0,
                    "source_ip": source_ip,
                    "rust_engine": False
                }
                threats_detected.append(threat)

                # Cache threat detection
                self.mnemonic.store(
                    f"threat_detection_{sig_id}_{datetime.now().isoformat()}",
                    threat,
                    priority=5
                )

        # Anomaly detection
        anomaly = self.anomaly_detector.detect_anomaly(packet_size)
        if anomaly:
            anomaly["source_ip"] = source_ip
            anomaly["rust_engine"] = False
            threats_detected.append(anomaly)

        # DDoS detection
        ddos = self.anomaly_detector.detect_ddos()
        if ddos:
            ddos["source_ip"] = source_ip
            threats_detected.append(ddos)

        # Generate alerts for high-severity threats
        for threat in threats_detected:
            if threat.get("severity") in ["high", "critical"] or threat.get("type") == "ddos_suspicion":
                await self._generate_alert(threat)

        return {
            "success": True,
            "action": "packet_analysis",
            "threats_detected": threats_detected,
            "packet_size": packet_size,
            "analysis_timestamp": datetime.now().isoformat()
        }

    async def _perform_anomaly_detection(self, context: Dict[str, Any]) -> Dict[str, Any]:
        """Perform comprehensive anomaly detection."""
        traffic_data = context.get("traffic_data", [])
        time_window = context.get("time_window", 300)  # 5 minutes

        anomalies = []

        # Use integrated threat system for anomaly detection
        if self.rust_engine_available and traffic_data:
            try:
                # Convert traffic data to packet bytes
                packet_bytes = []
                source_ips = []

                for packet_info in traffic_data:
                    packet_data = packet_info.get("data", b"")
                    if isinstance(packet_data, str):
                        packet_bytes.append(packet_data.encode())
                    elif isinstance(packet_data, bytes):
                        packet_bytes.append(packet_data)
                    else:
                        packet_bytes.append(str(packet_data).encode())

                    source_ips.append(packet_info.get("source_ip", "unknown"))

                # Analyze batch with Rust engine
                results = await self.threat_system.analyze_traffic_batch(packet_bytes, source_ips)

                # Extract anomalies
                for result in results:
                    threat_result = result.get("threat_result", {})
                    if threat_result.get("signature_id", "").endswith("_anomaly"):
                        anomalies.append({
                            "type": threat_result.get("category", "anomaly"),
                            "description": threat_result.get("description", "Statistical anomaly detected"),
                            "confidence": threat_result.get("confidence", 0.0),
                            "source_ip": result.get("source_ip"),
                            "rust_engine": True
                        })

            except Exception as e:
                logger.warning("rust_anomaly_detection_failed", error=str(e))
                # Fall back to Python analysis

        # Fallback: Python-based anomaly detection
        if traffic_data:
            packet_sizes = [len(packet.get("data", b"")) for packet in traffic_data]
            if packet_sizes:
                mean_size = np.mean(packet_sizes)
                std_size = np.std(packet_sizes)

                # Detect burst traffic
                recent_packets = len([p for p in traffic_data
                                    if (datetime.now() - datetime.fromisoformat(p.get("timestamp", ""))).total_seconds() < time_window])

                if recent_packets > 1000:  # Threshold
                    anomalies.append({
                        "type": "traffic_burst",
                        "packet_count": recent_packets,
                        "time_window": time_window,
                        "confidence": min(recent_packets / 2000.0, 1.0),
                        "rust_engine": False
                    })

        return {
            "success": True,
            "action": "anomaly_detection",
            "anomalies_detected": anomalies,
            "analysis_window": time_window
        }

    async def _update_threat_intelligence(self, context: Dict[str, Any]) -> Dict[str, Any]:
        """Update threat intelligence from external sources."""
        new_signatures = context.get("new_signatures", [])

        updated_count = 0

        # Use integrated threat system for signature updates
        if self.rust_engine_available:
            try:
                # Convert signatures to the format expected by Rust engine
                rust_signatures = []
                for sig_data in new_signatures:
                    rust_sig = {
                        "id": sig_data["id"],
                        "pattern": sig_data["pattern"],  # Already hex string
                        "description": sig_data["description"],
                        "severity": sig_data["severity"],
                        "category": sig_data["category"]
                    }
                    rust_signatures.append(rust_sig)

                # Update signatures via REST API
                success = await self.threat_system.update_signatures(rust_signatures)
                if success:
                    updated_count = len(new_signatures)
                else:
                    logger.warning("rust_signature_update_failed")
                    # Fall back to Python update

            except Exception as e:
                logger.warning("rust_signature_update_error", error=str(e))
                # Fall back to Python update

        # Fallback: Python-based signature update
        if updated_count == 0:
            for sig_data in new_signatures:
                sig = ThreatSignature(
                    sig_data["id"],
                    bytes.fromhex(sig_data["pattern"]),
                    sig_data["description"],
                    sig_data["severity"],
                    sig_data["category"]
                )

                self.threat_patterns[sig.id] = sig
                self.mnemonic.store(
                    f"threat_sig_{sig.id}",
                    sig_data,
                    priority=4
                )
                updated_count += 1

        return {
            "success": True,
            "action": "threat_intel_update",
            "signatures_updated": updated_count
        }

    async def _handle_incident_response(self, context: Dict[str, Any]) -> Dict[str, Any]:
        """Handle security incident response."""
        incident_type = context.get("incident_type")
        target_peer = context.get("peer_key")

        response_actions = []

        if incident_type == "quarantine":
            if target_peer:
                self.quarantined_peers.add(target_peer)
                response_actions.append({
                    "action": "quarantine_peer",
                    "peer_key": target_peer,
                    "timestamp": datetime.now().isoformat()
                })

                # Cache quarantine action
                self.mnemonic.store(
                    f"quarantine_{target_peer}",
                    {
                        "action": "quarantine",
                        "timestamp": datetime.now().isoformat(),
                        "reason": context.get("reason", "threat_detected")
                    },
                    priority=5
                )

        elif incident_type == "alert":
            alert_data = {
                "message": context.get("message", "Security alert"),
                "severity": context.get("severity", "medium"),
                "timestamp": datetime.now().isoformat()
            }
            response_actions.append({
                "action": "generate_alert",
                "alert_data": alert_data
            })

        return {
            "success": True,
            "action": "incident_response",
            "response_actions": response_actions
        }

    async def _analyze_traffic_batch(self, context: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze a batch of traffic packets."""
        packets = context.get("packets", [])
        source_ips = context.get("source_ips", [])

        if not packets or not self.rust_engine_available:
            return {
                "success": False,
                "action": "batch_analysis",
                "error": "No packets provided or Rust engine unavailable",
                "packets_analyzed": 0
            }

        # Convert packets to bytes if they're not already
        packet_bytes = []
        for packet in packets:
            if isinstance(packet, bytes):
                packet_bytes.append(packet)
            elif isinstance(packet, str):
                packet_bytes.append(packet.encode())
            else:
                packet_bytes.append(str(packet).encode())

        # Analyze with integrated system
        results = await self.threat_system.analyze_traffic_batch(packet_bytes, source_ips)

        # Process results
        threats_found = [r for r in results if r.get("threat_detected")]
        anomalies = []

        for result in results:
            if result.get("threat_result", {}).get("signature_id", "").endswith("_anomaly"):
                anomalies.append(result["threat_result"])

        return {
            "success": True,
            "action": "batch_analysis",
            "packets_analyzed": len(packets),
            "threats_detected": len(threats_found),
            "anomalies_detected": len(anomalies),
            "results": results
        }

    async def _generate_alert(self, threat: Dict[str, Any]) -> None:
        """Generate security alert."""
        alert_id = f"alert_{threat.get('signature_id', 'unknown')}_{datetime.now().isoformat()}"

        if alert_id not in self.active_alerts:
            self.active_alerts.add(alert_id)

            # Log alert
            logger.warning(
                "security_alert_generated",
                alert_id=alert_id,
                threat_type=threat.get("type", "signature"),
                severity=threat.get("severity", "unknown"),
                confidence=threat.get("confidence", 0.0)
            )

            # Cache alert
            self.mnemonic.store(
                alert_id,
                threat,
                priority=5
            )

    async def share_threat_intelligence(self, threat_data: Dict[str, Any]) -> None:
        """Share threat intelligence with other agents."""
        # This would integrate with the broader agent swarm
        intel_key = f"shared_intel_{datetime.now().isoformat()}"
        self.mnemonic.store(
            intel_key,
            {
                "threat_data": threat_data,
                "source_agent": "FORTRESS",
                "shared_at": datetime.now().isoformat()
            },
            priority=4
        )

        logger.info("threat_intelligence_shared", intel_key=intel_key)

    def get_security_status(self) -> Dict[str, Any]:
        """Get current security status."""
        return {
            "threat_patterns_loaded": len(self.threat_patterns),
            "active_alerts": len(self.active_alerts),
            "quarantined_peers": len(self.quarantined_peers),
            "anomaly_detector_window": len(self.anomaly_detector.packet_sizes),
            "mnemonic_cache_size": len(self.mnemonic._cache)
        }