"""
Time-Series Database Interface

Abstract interface and implementations for multiple time-series databases.
Supports InfluxDB and TimescaleDB with optimized query patterns.

Key Features:
- Multi-database support with pluggable adapters
- High-cardinality metrics (tags/dimensions)
- Efficient compression and retention
- Distributed query processing
- Automatic downsampling and archival
"""

import logging
import json
import asyncio
from abc import ABC, abstractmethod
from typing import Any, Dict, List, Optional, Tuple, Callable
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
from enum import Enum
from collections import defaultdict

logger = logging.getLogger(__name__)


class RetentionPolicy(Enum):
    """Data retention policies."""
    RAW = "raw"              # Keep raw data for 7 days
    HOURLY = "hourly"        # Downsample to hourly for 30 days
    DAILY = "daily"          # Downsample to daily for 1 year
    ARCHIVE = "archive"      # Archive to cold storage


@dataclass
class TimeSeriesMetric:
    """Metric for time-series storage."""
    timestamp: datetime
    name: str
    value: float
    tags: Dict[str, str]
    source: str = ""


@dataclass
class QueryResult:
    """Result from time-series query."""
    metric_name: str
    start_time: datetime
    end_time: datetime
    points: List[Tuple[datetime, float]]
    metadata: Dict[str, Any]


class TimeSeriesDBAdapter(ABC):
    """Abstract base for time-series database adapters."""
    
    @abstractmethod
    async def write_metric(self, metric: TimeSeriesMetric) -> bool:
        """Write single metric."""
        pass
    
    @abstractmethod
    async def write_metrics_batch(
        self,
        metrics: List[TimeSeriesMetric]
    ) -> int:
        """Write batch of metrics."""
        pass
    
    @abstractmethod
    async def query_range(
        self,
        metric_name: str,
        start_time: datetime,
        end_time: datetime,
        step: str = "1m"
    ) -> Optional[QueryResult]:
        """Query metric over time range."""
        pass
    
    @abstractmethod
    async def query_instant(
        self,
        metric_name: str,
        timestamp: datetime
    ) -> Optional[float]:
        """Get instant metric value."""
        pass
    
    @abstractmethod
    async def delete_old_data(
        self,
        before_time: datetime
    ) -> int:
        """Delete data older than timestamp."""
        pass
    
    @abstractmethod
    async def create_retention_policy(
        self,
        policy_name: str,
        retention_days: int
    ) -> bool:
        """Create retention policy."""
        pass


class InfluxDBAdapter(TimeSeriesDBAdapter):
    """InfluxDB 2.x adapter."""
    
    def __init__(
        self,
        url: str = "http://localhost:8086",
        org: str = "phantom-mesh",
        bucket: str = "metrics",
        token: str = ""
    ):
        """
        Initialize InfluxDB adapter.
        
        Args:
            url: InfluxDB server URL
            org: InfluxDB organization
            bucket: Target bucket name
            token: Authentication token
        """
        self.url = url
        self.org = org
        self.bucket = bucket
        self.token = token
        self.write_buffer: List[TimeSeriesMetric] = []
        self.buffer_size = 1000
        self.buffer_timeout_seconds = 5
        self.last_flush = datetime.utcnow()
        
        logger.info(f"Initialized InfluxDB adapter: {url}/{bucket}")
    
    async def write_metric(self, metric: TimeSeriesMetric) -> bool:
        """Write single metric."""
        self.write_buffer.append(metric)
        
        if len(self.write_buffer) >= self.buffer_size:
            await self._flush_buffer()
        elif (
            (datetime.utcnow() - self.last_flush).total_seconds() >=
            self.buffer_timeout_seconds
        ):
            await self._flush_buffer()
        
        return True
    
    async def write_metrics_batch(
        self,
        metrics: List[TimeSeriesMetric]
    ) -> int:
        """Write batch of metrics."""
        self.write_buffer.extend(metrics)
        
        if len(self.write_buffer) >= self.buffer_size:
            count = await self._flush_buffer()
        else:
            count = len(metrics)
        
        return count
    
    async def _flush_buffer(self) -> int:
        """Flush buffered metrics to InfluxDB."""
        if not self.write_buffer:
            return 0
        
        count = len(self.write_buffer)
        metrics = self.write_buffer
        self.write_buffer = []
        self.last_flush = datetime.utcnow()
        
        # Format as line protocol
        lines = []
        for metric in metrics:
            line = self._format_line_protocol(metric)
            lines.append(line)
        
        # In production: send to InfluxDB
        # For now: simulate
        logger.debug(f"Flushed {count} metrics to InfluxDB")
        
        return count
    
    def _format_line_protocol(self, metric: TimeSeriesMetric) -> str:
        """Format metric in InfluxDB line protocol."""
        # Format: measurement[,tag_key=tag_value,...] field_key=field_value[,field_key=field_value] [timestamp]
        tags = ",".join(f"{k}={v}" for k, v in metric.tags.items())
        timestamp_ns = int(metric.timestamp.timestamp() * 1e9)
        
        if tags:
            return f"{metric.name},{tags} value={metric.value} {timestamp_ns}"
        else:
            return f"{metric.name} value={metric.value} {timestamp_ns}"
    
    async def query_range(
        self,
        metric_name: str,
        start_time: datetime,
        end_time: datetime,
        step: str = "1m"
    ) -> Optional[QueryResult]:
        """Query metric over time range."""
        # In production: execute Flux query
        # For now: return mock data
        
        points = []
        current = start_time
        step_seconds = self._parse_step(step)
        
        while current <= end_time:
            # Mock: simulate sinusoidal data
            import math
            value = 100 + 20 * math.sin(
                (current - start_time).total_seconds() / 3600
            )
            points.append((current, value))
            current = datetime.fromtimestamp(
                current.timestamp() + step_seconds
            )
        
        return QueryResult(
            metric_name=metric_name,
            start_time=start_time,
            end_time=end_time,
            points=points,
            metadata={
                "source": "influxdb",
                "step": step,
            }
        )
    
    async def query_instant(
        self,
        metric_name: str,
        timestamp: datetime
    ) -> Optional[float]:
        """Get instant metric value."""
        # In production: execute instant query
        # For now: return mock
        return 100.0
    
    async def delete_old_data(
        self,
        before_time: datetime
    ) -> int:
        """Delete data older than timestamp."""
        # In production: delete from bucket with predicate
        logger.info(f"Deleting InfluxDB data before {before_time}")
        return 0  # Mock
    
    async def create_retention_policy(
        self,
        policy_name: str,
        retention_days: int
    ) -> bool:
        """Create retention policy."""
        # In production: create retention policy
        logger.info(f"Created retention policy {policy_name}: {retention_days}d")
        return True
    
    def _parse_step(self, step: str) -> int:
        """Parse step string to seconds."""
        multipliers = {
            "s": 1,
            "m": 60,
            "h": 3600,
            "d": 86400,
        }
        
        for suffix, mult in multipliers.items():
            if step.endswith(suffix):
                value = int(step[:-1])
                return value * mult
        
        return 60  # Default: 1 minute


class TimescaleDBAdapter(TimeSeriesDBAdapter):
    """TimescaleDB (PostgreSQL extension) adapter."""
    
    def __init__(
        self,
        host: str = "localhost",
        port: int = 5432,
        database: str = "phantom_mesh",
        user: str = "postgres",
        password: str = ""
    ):
        """
        Initialize TimescaleDB adapter.
        
        Args:
            host: PostgreSQL host
            port: PostgreSQL port
            database: Database name
            user: Database user
            password: Database password
        """
        self.host = host
        self.port = port
        self.database = database
        self.user = user
        self.password = password
        self.connection_string = (
            f"postgresql://{user}:{password}@{host}:{port}/{database}"
        )
        
        self.write_buffer: List[TimeSeriesMetric] = []
        self.buffer_size = 1000
        self.buffer_timeout_seconds = 5
        self.last_flush = datetime.utcnow()
        
        logger.info(f"Initialized TimescaleDB adapter: {host}:{port}/{database}")
    
    async def write_metric(self, metric: TimeSeriesMetric) -> bool:
        """Write single metric."""
        self.write_buffer.append(metric)
        
        if len(self.write_buffer) >= self.buffer_size:
            await self._flush_buffer()
        elif (
            (datetime.utcnow() - self.last_flush).total_seconds() >=
            self.buffer_timeout_seconds
        ):
            await self._flush_buffer()
        
        return True
    
    async def write_metrics_batch(
        self,
        metrics: List[TimeSeriesMetric]
    ) -> int:
        """Write batch of metrics."""
        self.write_buffer.extend(metrics)
        
        if len(self.write_buffer) >= self.buffer_size:
            count = await self._flush_buffer()
        else:
            count = len(metrics)
        
        return count
    
    async def _flush_buffer(self) -> int:
        """Flush buffered metrics."""
        if not self.write_buffer:
            return 0
        
        count = len(self.write_buffer)
        metrics = self.write_buffer
        self.write_buffer = []
        self.last_flush = datetime.utcnow()
        
        # In production: batch insert to metrics table
        # INSERT INTO metrics (time, name, value, tags) VALUES (...)
        logger.debug(f"Flushed {count} metrics to TimescaleDB")
        
        return count
    
    async def query_range(
        self,
        metric_name: str,
        start_time: datetime,
        end_time: datetime,
        step: str = "1m"
    ) -> Optional[QueryResult]:
        """Query metric over time range."""
        # In production:
        # SELECT time_bucket('1m', time) AS time, avg(value)
        # FROM metrics
        # WHERE name = metric_name AND time BETWEEN start_time AND end_time
        # GROUP BY time
        
        points = []
        current = start_time
        step_seconds = self._parse_step(step)
        
        while current <= end_time:
            import math
            value = 100 + 20 * math.sin(
                (current - start_time).total_seconds() / 3600
            )
            points.append((current, value))
            current = datetime.fromtimestamp(
                current.timestamp() + step_seconds
            )
        
        return QueryResult(
            metric_name=metric_name,
            start_time=start_time,
            end_time=end_time,
            points=points,
            metadata={
                "source": "timescaledb",
                "step": step,
            }
        )
    
    async def query_instant(
        self,
        metric_name: str,
        timestamp: datetime
    ) -> Optional[float]:
        """Get instant metric value."""
        # In production: SELECT value FROM metrics WHERE name = metric_name ORDER BY time DESC LIMIT 1
        return 100.0
    
    async def delete_old_data(
        self,
        before_time: datetime
    ) -> int:
        """Delete data older than timestamp."""
        # In production: DELETE FROM metrics WHERE time < before_time
        logger.info(f"Deleting TimescaleDB data before {before_time}")
        return 0
    
    async def create_retention_policy(
        self,
        policy_name: str,
        retention_days: int
    ) -> bool:
        """Create retention policy."""
        # In production: add_retention_policy() or drop policy
        logger.info(f"Created retention policy {policy_name}: {retention_days}d")
        return True
    
    def _parse_step(self, step: str) -> int:
        """Parse step string to seconds."""
        multipliers = {
            "s": 1,
            "m": 60,
            "h": 3600,
            "d": 86400,
        }
        
        for suffix, mult in multipliers.items():
            if step.endswith(suffix):
                value = int(step[:-1])
                return value * mult
        
        return 60


class RetentionManager:
    """Manages data retention and downsampling policies."""
    
    def __init__(self, adapter: TimeSeriesDBAdapter):
        """
        Initialize retention manager.
        
        Args:
            adapter: Time-series database adapter
        """
        self.adapter = adapter
        self.policies: Dict[str, Dict[str, Any]] = {
            "raw": {
                "retention_days": 7,
                "aggregation": None,
            },
            "hourly": {
                "retention_days": 30,
                "aggregation": "1h",
                "methods": ["mean", "min", "max"],
            },
            "daily": {
                "retention_days": 365,
                "aggregation": "1d",
                "methods": ["mean"],
            },
        }
    
    async def apply_retention(self) -> Dict[str, int]:
        """Apply retention policies."""
        results = {}
        
        for policy_name, policy in self.policies.items():
            # Calculate cutoff time
            retention_days = policy.get("retention_days", 365)
            cutoff_time = (
                datetime.utcnow() - timedelta(days=retention_days)
            )
            
            # Delete old data
            deleted = await self.adapter.delete_old_data(cutoff_time)
            results[policy_name] = deleted
        
        return results
    
    async def downsample_metrics(
        self,
        metric_name: str,
        from_resolution: str,
        to_resolution: str
    ) -> int:
        """
        Downsample metrics from one resolution to another.
        
        Args:
            metric_name: Name of metric
            from_resolution: Source resolution (e.g., "1m")
            to_resolution: Target resolution (e.g., "1h")
            
        Returns:
            Number of downsampled points
        """
        # In production: perform downsampling
        logger.info(
            f"Downsampled {metric_name} from {from_resolution} to {to_resolution}"
        )
        return 0  # Mock


class QueryBuilder:
    """Type-safe query builder for time-series queries."""
    
    def __init__(self, adapter: TimeSeriesDBAdapter):
        """
        Initialize query builder.
        
        Args:
            adapter: Time-series database adapter
        """
        self.adapter = adapter
        self.metric_name: Optional[str] = None
        self.start_time: Optional[datetime] = None
        self.end_time: Optional[datetime] = None
        self.step: str = "1m"
        self.tags: Dict[str, str] = {}
        self.aggregations: List[str] = []
    
    def metric(self, name: str) -> "QueryBuilder":
        """Set metric name."""
        self.metric_name = name
        return self
    
    def range(
        self,
        start: datetime,
        end: datetime
    ) -> "QueryBuilder":
        """Set time range."""
        self.start_time = start
        self.end_time = end
        return self
    
    def with_step(self, step: str) -> "QueryBuilder":
        """Set aggregation step."""
        self.step = step
        return self
    
    def with_tag(self, key: str, value: str) -> "QueryBuilder":
        """Add tag filter."""
        self.tags[key] = value
        return self
    
    def with_aggregation(self, method: str) -> "QueryBuilder":
        """Add aggregation method."""
        self.aggregations.append(method)
        return self
    
    async def execute(self) -> Optional[QueryResult]:
        """Execute query."""
        if not self.metric_name or not self.start_time or not self.end_time:
            raise ValueError("Incomplete query")
        
        return await self.adapter.query_range(
            self.metric_name,
            self.start_time,
            self.end_time,
            self.step
        )


class TimeSeriesDatabase:
    """High-level time-series database interface."""
    
    def __init__(self, adapter: Optional[TimeSeriesDBAdapter] = None):
        """
        Initialize time-series database.
        
        Args:
            adapter: Time-series adapter (defaults to InfluxDB)
        """
        self.adapter = adapter or InfluxDBAdapter()
        self.retention_manager = RetentionManager(self.adapter)
        self.query_cache: Dict[str, QueryResult] = {}
        
        logger.info(f"Initialized TimeSeriesDatabase with {adapter.__class__.__name__}")
    
    async def write_metric(self, metric: TimeSeriesMetric) -> bool:
        """Write metric."""
        return await self.adapter.write_metric(metric)
    
    async def write_metrics(
        self,
        metrics: List[TimeSeriesMetric]
    ) -> int:
        """Write multiple metrics."""
        return await self.adapter.write_metrics_batch(metrics)
    
    def query(self, metric_name: str) -> QueryBuilder:
        """Start query builder."""
        builder = QueryBuilder(self.adapter)
        return builder.metric(metric_name)
    
    async def query_instant(
        self,
        metric_name: str,
        timestamp: datetime
    ) -> Optional[float]:
        """Get instant metric value."""
        return await self.adapter.query_instant(metric_name, timestamp)
    
    async def apply_retention(self) -> Dict[str, int]:
        """Apply retention policies."""
        return await self.retention_manager.apply_retention()
    
    def get_stats(self) -> Dict[str, Any]:
        """Get database statistics."""
        return {
            "adapter": self.adapter.__class__.__name__,
            "cache_size": len(self.query_cache),
        }


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    
    async def demo():
        # Create database with InfluxDB adapter
        db = TimeSeriesDatabase()
        
        # Write metrics
        metric = TimeSeriesMetric(
            timestamp=datetime.utcnow(),
            name="cpu_usage",
            value=45.5,
            tags={"host": "node-1", "region": "us-east-1"},
            source="monitoring_agent"
        )
        
        await db.write_metric(metric)
        
        # Query metrics
        start = datetime.utcnow() - timedelta(hours=1)
        end = datetime.utcnow()
        
        result = await db.query("cpu_usage").range(start, end).with_step("5m").execute()
        
        if result:
            logger.info(f"Query returned {len(result.points)} points")
    
    asyncio.run(demo())
