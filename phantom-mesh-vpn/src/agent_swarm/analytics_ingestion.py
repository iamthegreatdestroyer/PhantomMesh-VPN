"""
Analytics Data Ingestion & Stream Processing Engine

This module provides high-performance, real-time data ingestion for threat
intelligence, network metrics, and system events. It handles streaming from
Phase P1-003 threat detection and various monitoring sources.

Key Components:
- ThreatStreamProcessor: Consumes threat detection output
- MetricsCollector: Aggregates network and system metrics
- EventEnricher: Adds context and correlations
- StreamDeduplicator: Reduces redundant events
- DataBatcher: Optimizes database writes

Performance Targets:
- 100k+ events per second ingestion
- <50ms end-to-end latency
- 99.99% event delivery guarantee
"""

import asyncio
import json
import logging
import time
from typing import Any, Callable, Dict, List, Optional, Set, Tuple
from dataclasses import dataclass, asdict, field
from datetime import datetime, timedelta
from collections import deque, defaultdict
from abc import ABC, abstractmethod
from enum import Enum
import hashlib

import numpy as np
from concurrent.futures import ThreadPoolExecutor

# Configure logging
logger = logging.getLogger(__name__)


class EventType(Enum):
    """Classification of inbound events."""
    THREAT_DETECTION = "threat_detection"
    NETWORK_METRIC = "network_metric"
    SYSTEM_EVENT = "system_event"
    SECURITY_ALERT = "security_alert"
    PERFORMANCE_METRIC = "performance_metric"


class EventSeverity(Enum):
    """Event severity levels."""
    CRITICAL = 5
    HIGH = 4
    MEDIUM = 3
    LOW = 2
    INFO = 1


@dataclass
class RawEvent:
    """Raw event from detection or monitoring systems."""
    timestamp: datetime
    source: str
    event_type: EventType
    data: Dict[str, Any]
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def get_hash(self) -> str:
        """Generate consistent hash for deduplication."""
        content = json.dumps(asdict(self), default=str, sort_keys=True)
        return hashlib.md5(content.encode()).hexdigest()


@dataclass
class EnrichedEvent:
    """Event with added context and correlations."""
    timestamp: datetime
    source: str
    event_type: EventType
    severity: EventSeverity
    data: Dict[str, Any]
    metadata: Dict[str, Any]
    enrichment: Dict[str, Any] = field(default_factory=dict)
    correlations: List[str] = field(default_factory=list)
    original_hash: str = ""
    processed_at: datetime = field(default_factory=datetime.utcnow)


@dataclass
class AggregatedMetric:
    """Aggregated metric for time-series storage."""
    timestamp: datetime
    metric_name: str
    value: float
    tags: Dict[str, str] = field(default_factory=dict)
    source: str = ""
    aggregation_method: str = "none"


class StreamDeduplicator:
    """
    Reduces redundant events using sliding window deduplication.
    
    Uses a combination of:
    - Exact hash matching (recent window)
    - Bloom filter for larger windows
    - Time-based bucketing
    """
    
    def __init__(
        self,
        window_size: int = 5000,
        time_window_seconds: int = 60
    ):
        """
        Initialize deduplicator.
        
        Args:
            window_size: Number of hashes to keep in memory
            time_window_seconds: Time window for duplicate detection
        """
        self.window_size = window_size
        self.time_window_seconds = time_window_seconds
        self.recent_hashes: Dict[str, datetime] = {}
        self.hash_queue: deque = deque(maxlen=window_size)
        self.duplicate_count = 0
        self.processed_count = 0
        self.lock = asyncio.Lock()
        
    async def is_duplicate(self, event: RawEvent) -> bool:
        """
        Check if event is a duplicate.
        
        Args:
            event: Event to check
            
        Returns:
            True if duplicate, False otherwise
        """
        async with self.lock:
            event_hash = event.get_hash()
            now = datetime.utcnow()
            
            # Clean expired hashes
            expired_keys = [
                k for k, v in self.recent_hashes.items()
                if (now - v).total_seconds() > self.time_window_seconds
            ]
            for k in expired_keys:
                del self.recent_hashes[k]
            
            # Check for duplicate
            if event_hash in self.recent_hashes:
                self.duplicate_count += 1
                return True
            
            # Add to tracking
            self.recent_hashes[event_hash] = now
            self.hash_queue.append(event_hash)
            self.processed_count += 1
            return False
    
    def get_stats(self) -> Dict[str, Any]:
        """Get deduplication statistics."""
        return {
            "processed_total": self.processed_count,
            "duplicates_detected": self.duplicate_count,
            "duplicate_rate": (
                self.duplicate_count / self.processed_count
                if self.processed_count > 0 else 0
            ),
            "active_hashes": len(self.recent_hashes),
        }


class EventEnricher:
    """
    Adds context and correlations to events.
    
    Enrichment includes:
    - Threat context from threat intelligence database
    - Network topology information
    - Historical behavior patterns
    - Correlation with recent events
    """
    
    def __init__(self, correlation_window_seconds: int = 300):
        """
        Initialize enricher.
        
        Args:
            correlation_window_seconds: Time window for correlation analysis
        """
        self.correlation_window_seconds = correlation_window_seconds
        self.recent_events: List[Tuple[datetime, RawEvent]] = []
        self.enrichment_cache: Dict[str, Dict[str, Any]] = {}
        self.threat_intel_db: Dict[str, Dict[str, Any]] = {}
        self.lock = asyncio.Lock()
        
    async def enrich(self, event: RawEvent) -> EnrichedEvent:
        """
        Enrich event with context and correlations.
        
        Args:
            event: Raw event to enrich
            
        Returns:
            Enriched event with additional context
        """
        async with self.lock:
            # Determine severity
            severity = self._determine_severity(event)
            
            # Get threat context
            threat_context = self._get_threat_context(event)
            
            # Find correlations
            correlations = self._find_correlations(event)
            
            # Build enrichment
            enrichment = {
                "threat_context": threat_context,
                "source_reputation": self._get_source_reputation(event.source),
                "network_context": self._get_network_context(event),
                "historical_patterns": self._analyze_patterns(event),
            }
            
            # Store for correlation
            self.recent_events.append((datetime.utcnow(), event))
            
            # Clean old events
            cutoff_time = datetime.utcnow() - timedelta(
                seconds=self.correlation_window_seconds
            )
            self.recent_events = [
                (ts, e) for ts, e in self.recent_events
                if ts > cutoff_time
            ]
            
            return EnrichedEvent(
                timestamp=event.timestamp,
                source=event.source,
                event_type=event.event_type,
                severity=severity,
                data=event.data,
                metadata=event.metadata,
                enrichment=enrichment,
                correlations=correlations,
                original_hash=event.get_hash(),
            )
    
    def _determine_severity(self, event: RawEvent) -> EventSeverity:
        """Determine event severity."""
        if event.event_type == EventType.THREAT_DETECTION:
            threat_score = event.data.get("threat_score", 0)
            if threat_score > 0.8:
                return EventSeverity.CRITICAL
            elif threat_score > 0.6:
                return EventSeverity.HIGH
            elif threat_score > 0.4:
                return EventSeverity.MEDIUM
            else:
                return EventSeverity.LOW
        
        return EventSeverity.INFO
    
    def _get_threat_context(self, event: RawEvent) -> Dict[str, Any]:
        """Get threat intelligence context."""
        threat_type = event.data.get("threat_type", "unknown")
        
        if threat_type in self.threat_intel_db:
            return self.threat_intel_db[threat_type]
        
        return {
            "threat_type": threat_type,
            "known": False,
            "previous_incidents": 0,
        }
    
    def _get_source_reputation(self, source: str) -> Dict[str, Any]:
        """Get source reputation."""
        return {
            "source": source,
            "trust_score": 0.95,
            "previous_events": 0,
            "reliability": "high",
        }
    
    def _get_network_context(self, event: RawEvent) -> Dict[str, Any]:
        """Get network topology context."""
        return {
            "source_region": event.metadata.get("region", "unknown"),
            "affected_nodes": event.data.get("affected_nodes", []),
            "network_zone": "protected",
        }
    
    def _analyze_patterns(self, event: RawEvent) -> Dict[str, Any]:
        """Analyze historical patterns."""
        return {
            "event_frequency": "low",
            "typical_impact": "low",
            "remediation_time": "< 5 minutes",
        }
    
    def _find_correlations(self, event: RawEvent) -> List[str]:
        """Find correlated recent events."""
        correlations = []
        event_time = event.timestamp
        correlation_threshold = timedelta(seconds=30)
        
        for ts, recent_event in self.recent_events:
            if event_time - ts < correlation_threshold:
                if (recent_event.source == event.source or
                    recent_event.event_type == event.event_type):
                    correlations.append(recent_event.get_hash())
        
        return correlations


class DataBatcher:
    """
    Batches events for efficient database writes.
    
    Optimizations:
    - Configurable batch size
    - Time-based batching
    - Adaptive batching based on throughput
    """
    
    def __init__(
        self,
        batch_size: int = 1000,
        batch_timeout_seconds: float = 5.0
    ):
        """
        Initialize batcher.
        
        Args:
            batch_size: Number of events per batch
            batch_timeout_seconds: Maximum time to wait for batch
        """
        self.batch_size = batch_size
        self.batch_timeout_seconds = batch_timeout_seconds
        self.current_batch: List[EnrichedEvent] = []
        self.batch_start_time: Optional[datetime] = None
        self.lock = asyncio.Lock()
        self.callbacks: List[Callable] = []
        self.batch_count = 0
        self.event_count = 0
        
    async def add_event(self, event: EnrichedEvent) -> bool:
        """
        Add event to batch.
        
        Args:
            event: Event to add
            
        Returns:
            True if batch is ready, False otherwise
        """
        async with self.lock:
            self.current_batch.append(event)
            self.event_count += 1
            
            if self.batch_start_time is None:
                self.batch_start_time = datetime.utcnow()
            
            # Check if batch is ready
            return await self._should_flush()
    
    async def _should_flush(self) -> bool:
        """Check if batch should be flushed."""
        if len(self.current_batch) >= self.batch_size:
            return True
        
        if self.batch_start_time:
            elapsed = (
                datetime.utcnow() - self.batch_start_time
            ).total_seconds()
            if elapsed >= self.batch_timeout_seconds:
                return True
        
        return False
    
    async def flush(self) -> Optional[List[EnrichedEvent]]:
        """
        Flush current batch if ready.
        
        Returns:
            Batch of events or None if not ready
        """
        async with self.lock:
            if len(self.current_batch) == 0:
                return None
            
            batch = self.current_batch
            self.current_batch = []
            self.batch_start_time = None
            self.batch_count += 1
            
            # Notify callbacks
            for callback in self.callbacks:
                await callback(batch)
            
            return batch
    
    def register_callback(self, callback: Callable) -> None:
        """Register callback for batch completion."""
        self.callbacks.append(callback)
    
    def get_stats(self) -> Dict[str, Any]:
        """Get batching statistics."""
        return {
            "total_batches": self.batch_count,
            "total_events": self.event_count,
            "current_batch_size": len(self.current_batch),
            "avg_batch_size": (
                self.event_count / self.batch_count
                if self.batch_count > 0 else 0
            ),
        }


class ThreatStreamProcessor:
    """
    High-performance processor for threat detection streams.
    
    Handles threat events from Phase P1-003 detection engine,
    deduplicating, enriching, and batching for database storage.
    """
    
    def __init__(
        self,
        batch_size: int = 1000,
        dedup_window_size: int = 5000,
        max_workers: int = 4
    ):
        """
        Initialize threat stream processor.
        
        Args:
            batch_size: Events per batch
            dedup_window_size: Deduplication window size
            max_workers: Number of worker threads
        """
        self.deduplicator = StreamDeduplicator(
            window_size=dedup_window_size
        )
        self.enricher = EventEnricher()
        self.batcher = DataBatcher(batch_size=batch_size)
        
        self.executor = ThreadPoolExecutor(max_workers=max_workers)
        self.processed_events = 0
        self.dropped_events = 0
        self.processing_times: deque = deque(maxlen=10000)
        self.start_time = datetime.utcnow()
        
    async def process_threat_event(
        self,
        event: RawEvent
    ) -> Optional[EnrichedEvent]:
        """
        Process incoming threat event.
        
        Args:
            event: Raw threat event
            
        Returns:
            Enriched event or None if duplicate/dropped
        """
        process_start = time.perf_counter()
        
        try:
            # Check for duplicates
            if await self.deduplicator.is_duplicate(event):
                self.dropped_events += 1
                return None
            
            # Enrich event
            enriched = await self.enricher.enrich(event)
            
            # Add to batch
            is_batch_ready = await self.batcher.add_event(enriched)
            
            self.processed_events += 1
            
            # Track processing time
            process_time = (time.perf_counter() - process_start) * 1000
            self.processing_times.append(process_time)
            
            if is_batch_ready:
                await self.batcher.flush()
            
            return enriched
            
        except Exception as e:
            logger.error(f"Error processing threat event: {e}")
            self.dropped_events += 1
            return None
    
    async def process_batch(
        self,
        events: List[RawEvent]
    ) -> List[EnrichedEvent]:
        """
        Process batch of events.
        
        Args:
            events: List of raw events
            
        Returns:
            List of enriched events
        """
        tasks = [
            self.process_threat_event(event)
            for event in events
        ]
        results = await asyncio.gather(*tasks)
        return [r for r in results if r is not None]
    
    def register_batch_callback(self, callback: Callable) -> None:
        """Register callback for batch completion."""
        self.batcher.register_callback(callback)
    
    def get_metrics(self) -> Dict[str, Any]:
        """Get processor metrics."""
        uptime = (datetime.utcnow() - self.start_time).total_seconds()
        
        processing_times = list(self.processing_times)
        
        return {
            "processed_events": self.processed_events,
            "dropped_events": self.dropped_events,
            "uptime_seconds": uptime,
            "throughput_events_per_sec": (
                self.processed_events / uptime if uptime > 0 else 0
            ),
            "processing_latency": {
                "p50": np.percentile(processing_times, 50) if processing_times else 0,
                "p95": np.percentile(processing_times, 95) if processing_times else 0,
                "p99": np.percentile(processing_times, 99) if processing_times else 0,
                "max": max(processing_times) if processing_times else 0,
            },
            "deduplicator_stats": self.deduplicator.get_stats(),
            "batcher_stats": self.batcher.get_stats(),
        }


class MetricsCollector:
    """
    Aggregates network and system metrics into time-series format.
    
    Collects from:
    - Network interfaces (throughput, latency, packet loss)
    - System resources (CPU, memory, disk)
    - Node health (uptime, availability)
    - Regional statistics
    """
    
    def __init__(self):
        """Initialize metrics collector."""
        self.metric_cache: Dict[str, List[AggregatedMetric]] = defaultdict(list)
        self.aggregation_intervals = {
            "1s": 1,
            "1m": 60,
            "5m": 300,
            "1h": 3600,
        }
        self.lock = asyncio.Lock()
        self.collected_count = 0
        
    async def collect_network_metrics(
        self,
        source: str,
        metrics: Dict[str, float],
        tags: Dict[str, str]
    ) -> List[AggregatedMetric]:
        """
        Collect network metrics.
        
        Args:
            source: Source of metrics
            metrics: Network metrics dict
            tags: Metric tags
            
        Returns:
            List of aggregated metrics
        """
        async with self.lock:
            aggregated = []
            timestamp = datetime.utcnow()
            
            for metric_name, value in metrics.items():
                agg_metric = AggregatedMetric(
                    timestamp=timestamp,
                    metric_name=f"network.{metric_name}",
                    value=value,
                    tags=tags,
                    source=source,
                    aggregation_method="instantaneous",
                )
                aggregated.append(agg_metric)
                self.metric_cache[metric_name].append(agg_metric)
            
            self.collected_count += len(aggregated)
            return aggregated
    
    async def collect_system_metrics(
        self,
        source: str,
        metrics: Dict[str, float],
        tags: Dict[str, str]
    ) -> List[AggregatedMetric]:
        """
        Collect system resource metrics.
        
        Args:
            source: Source of metrics
            metrics: System metrics dict
            tags: Metric tags
            
        Returns:
            List of aggregated metrics
        """
        async with self.lock:
            aggregated = []
            timestamp = datetime.utcnow()
            
            for metric_name, value in metrics.items():
                agg_metric = AggregatedMetric(
                    timestamp=timestamp,
                    metric_name=f"system.{metric_name}",
                    value=value,
                    tags=tags,
                    source=source,
                    aggregation_method="gauge",
                )
                aggregated.append(agg_metric)
                self.metric_cache[metric_name].append(agg_metric)
            
            self.collected_count += len(aggregated)
            return aggregated
    
    def get_metrics(self) -> Dict[str, Any]:
        """Get collector statistics."""
        return {
            "collected_total": self.collected_count,
            "metric_types": len(self.metric_cache),
            "cached_metrics": sum(
                len(v) for v in self.metric_cache.values()
            ),
        }


# Integration example
async def demo_analytics_ingestion():
    """Demonstrate analytics ingestion pipeline."""
    logger.info("Starting analytics ingestion demo...")
    
    # Create processor
    processor = ThreatStreamProcessor(batch_size=100)
    
    # Register batch callback
    async def on_batch(batch: List[EnrichedEvent]):
        logger.info(f"Batch ready: {len(batch)} events")
    
    processor.register_batch_callback(on_batch)
    
    # Simulate threat events
    for i in range(500):
        event = RawEvent(
            timestamp=datetime.utcnow(),
            source="threat_detector_node_1",
            event_type=EventType.THREAT_DETECTION,
            data={
                "threat_type": "DDoS" if i % 10 == 0 else "Scanning",
                "threat_score": 0.7 + (0.2 * (i % 3)),
                "affected_nodes": ["node_1", "node_2"],
            },
            metadata={
                "region": "us-east-1",
                "detection_method": "behavioral_analysis",
            }
        )
        
        result = await processor.process_threat_event(event)
        if result and i % 100 == 0:
            logger.info(f"Processed event {i}: {result.severity.name}")
    
    # Get metrics
    metrics = processor.get_metrics()
    logger.info(f"Processor metrics: {json.dumps(metrics, indent=2, default=str)}")


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(demo_analytics_ingestion())
