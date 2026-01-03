"""
PhantomMesh Orchestration Performance Testing Suite
====================================================

Comprehensive performance benchmarking for orchestration throughput,
latency, scalability, and fault recovery.

Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

from __future__ import annotations

import asyncio
import time
import statistics
from dataclasses import dataclass, field
from datetime import datetime, UTC
from typing import Any, Callable, Coroutine
from collections import defaultdict
import json

import structlog

logger = structlog.get_logger(__name__)


# ═══════════════════════════════════════════════════════════════════════════════
# PERFORMANCE METRICS
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class PerformanceMetrics:
    """Performance metrics for an operation."""
    operation: str
    duration_ms: float
    success: bool
    timestamp: datetime = field(default_factory=lambda: datetime.now(UTC))
    metadata: dict[str, Any] = field(default_factory=dict)
    
    def to_dict(self) -> dict[str, Any]:
        return {
            "operation": self.operation,
            "duration_ms": round(self.duration_ms, 2),
            "success": self.success,
            "timestamp": self.timestamp.isoformat(),
            "metadata": self.metadata
        }


@dataclass
class BenchmarkResults:
    """Aggregated benchmark results."""
    benchmark_name: str
    total_operations: int
    successful_operations: int
    failed_operations: int
    
    # Latency metrics (ms)
    min_latency_ms: float
    max_latency_ms: float
    mean_latency_ms: float
    median_latency_ms: float
    p95_latency_ms: float
    p99_latency_ms: float
    stddev_latency_ms: float
    
    # Throughput
    operations_per_second: float
    
    # Resource usage
    peak_memory_mb: float
    average_memory_mb: float
    
    # Results
    success_rate: float
    timestamp: datetime = field(default_factory=lambda: datetime.now(UTC))
    
    def to_dict(self) -> dict[str, Any]:
        return {
            "benchmark_name": self.benchmark_name,
            "total_operations": self.total_operations,
            "successful_operations": self.successful_operations,
            "failed_operations": self.failed_operations,
            "latency_ms": {
                "min": round(self.min_latency_ms, 2),
                "max": round(self.max_latency_ms, 2),
                "mean": round(self.mean_latency_ms, 2),
                "median": round(self.median_latency_ms, 2),
                "p95": round(self.p95_latency_ms, 2),
                "p99": round(self.p99_latency_ms, 2),
                "stddev": round(self.stddev_latency_ms, 2)
            },
            "throughput_ops_per_sec": round(self.operations_per_second, 2),
            "memory_mb": {
                "peak": round(self.peak_memory_mb, 2),
                "average": round(self.average_memory_mb, 2)
            },
            "success_rate": round(self.success_rate, 4),
            "timestamp": self.timestamp.isoformat()
        }


class PerformanceMonitor:
    """Monitor and aggregate performance metrics."""
    
    def __init__(self):
        self.metrics: list[PerformanceMetrics] = []
        self._lock = asyncio.Lock()
    
    async def record(self, metric: PerformanceMetrics) -> None:
        """Record performance metric."""
        async with self._lock:
            self.metrics.append(metric)
    
    def get_metrics_by_operation(self, operation: str) -> list[PerformanceMetrics]:
        """Get all metrics for specific operation."""
        return [m for m in self.metrics if m.operation == operation]
    
    def calculate_percentile(
        self,
        operation: str,
        percentile: float
    ) -> float:
        """Calculate percentile latency for operation."""
        metrics = self.get_metrics_by_operation(operation)
        if not metrics:
            return 0.0
        
        durations = sorted([m.duration_ms for m in metrics])
        index = int(len(durations) * percentile / 100)
        return durations[min(index, len(durations) - 1)]


# ═══════════════════════════════════════════════════════════════════════════════
# ORCHESTRATION BENCHMARKS
# ═══════════════════════════════════════════════════════════════════════════════

class OrchestrationBenchmark:
    """Benchmark suite for orchestration performance."""
    
    def __init__(self, monitor: PerformanceMonitor):
        self.monitor = monitor
        self.memory_samples: list[float] = []
    
    async def benchmark_workflow_execution(
        self,
        num_workflows: int = 100,
        steps_per_workflow: int = 5
    ) -> BenchmarkResults:
        """
        Benchmark workflow execution throughput and latency.
        
        Tests:
        - Workflow creation and state machine setup
        - Step execution
        - State transitions
        - Resource allocation
        """
        logger.info(
            "benchmark_workflow_execution_start",
            num_workflows=num_workflows,
            steps_per_workflow=steps_per_workflow
        )
        
        start_time = time.time()
        
        # Simulate workflow execution
        async def execute_workflow(workflow_id: int) -> float:
            workflow_start = time.time()
            
            try:
                # Simulate workflow execution
                for step_id in range(steps_per_workflow):
                    # Simulate step processing
                    await asyncio.sleep(0.01)  # 10ms per step
                
                elapsed = (time.time() - workflow_start) * 1000  # ms
                
                await self.monitor.record(
                    PerformanceMetrics(
                        operation="workflow_execution",
                        duration_ms=elapsed,
                        success=True,
                        metadata={
                            "workflow_id": workflow_id,
                            "steps": steps_per_workflow
                        }
                    )
                )
                
                return elapsed
                
            except Exception as e:
                elapsed = (time.time() - workflow_start) * 1000
                await self.monitor.record(
                    PerformanceMetrics(
                        operation="workflow_execution",
                        duration_ms=elapsed,
                        success=False,
                        metadata={"error": str(e)}
                    )
                )
                return elapsed
        
        # Run workflows concurrently
        tasks = [
            execute_workflow(i)
            for i in range(num_workflows)
        ]
        
        await asyncio.gather(*tasks)
        
        total_time = time.time() - start_time
        metrics = self.monitor.get_metrics_by_operation("workflow_execution")
        
        return self._compile_results(
            "workflow_execution",
            metrics,
            total_time,
            num_workflows
        )
    
    async def benchmark_state_transitions(
        self,
        num_transitions: int = 1000,
        guard_complexity: str = "simple"
    ) -> BenchmarkResults:
        """
        Benchmark state machine transition performance.
        
        Tests:
        - State transition validation
        - Guard evaluation
        - Callback execution
        - History maintenance
        """
        logger.info(
            "benchmark_state_transitions_start",
            num_transitions=num_transitions,
            guard_complexity=guard_complexity
        )
        
        start_time = time.time()
        
        # Simulate state transitions
        async def perform_transition(transition_id: int) -> float:
            trans_start = time.time()
            
            try:
                # Simulate guard evaluation
                if guard_complexity == "simple":
                    await asyncio.sleep(0.0001)  # 0.1ms
                elif guard_complexity == "complex":
                    await asyncio.sleep(0.001)   # 1ms
                
                # Simulate callback execution
                await asyncio.sleep(0.0005)  # 0.5ms
                
                elapsed = (time.time() - trans_start) * 1000
                
                await self.monitor.record(
                    PerformanceMetrics(
                        operation="state_transition",
                        duration_ms=elapsed,
                        success=True,
                        metadata={
                            "transition_id": transition_id,
                            "complexity": guard_complexity
                        }
                    )
                )
                
                return elapsed
                
            except Exception as e:
                elapsed = (time.time() - trans_start) * 1000
                await self.monitor.record(
                    PerformanceMetrics(
                        operation="state_transition",
                        duration_ms=elapsed,
                        success=False,
                        metadata={"error": str(e)}
                    )
                )
                return elapsed
        
        # Run transitions concurrently
        tasks = [perform_transition(i) for i in range(num_transitions)]
        await asyncio.gather(*tasks)
        
        total_time = time.time() - start_time
        metrics = self.monitor.get_metrics_by_operation("state_transition")
        
        return self._compile_results(
            "state_transitions",
            metrics,
            total_time,
            num_transitions
        )
    
    async def benchmark_threat_detection(
        self,
        num_threats: int = 500,
        pattern_complexity: str = "simple"
    ) -> BenchmarkResults:
        """
        Benchmark threat detection and response pattern matching.
        
        Tests:
        - Pattern matching performance
        - Threat classification
        - Response workflow initiation
        - Incident logging
        """
        logger.info(
            "benchmark_threat_detection_start",
            num_threats=num_threats,
            pattern_complexity=pattern_complexity
        )
        
        start_time = time.time()
        
        # Simulate threat detection
        async def detect_threat(threat_id: int) -> float:
            detect_start = time.time()
            
            try:
                # Simulate pattern matching
                if pattern_complexity == "simple":
                    await asyncio.sleep(0.001)   # 1ms
                elif pattern_complexity == "complex":
                    await asyncio.sleep(0.005)   # 5ms
                
                # Simulate incident logging
                await asyncio.sleep(0.0005)  # 0.5ms
                
                elapsed = (time.time() - detect_start) * 1000
                
                await self.monitor.record(
                    PerformanceMetrics(
                        operation="threat_detection",
                        duration_ms=elapsed,
                        success=True,
                        metadata={
                            "threat_id": threat_id,
                            "complexity": pattern_complexity
                        }
                    )
                )
                
                return elapsed
                
            except Exception as e:
                elapsed = (time.time() - detect_start) * 1000
                await self.monitor.record(
                    PerformanceMetrics(
                        operation="threat_detection",
                        duration_ms=elapsed,
                        success=False,
                        metadata={"error": str(e)}
                    )
                )
                return elapsed
        
        # Run threat detection
        tasks = [detect_threat(i) for i in range(num_threats)]
        await asyncio.gather(*tasks)
        
        total_time = time.time() - start_time
        metrics = self.monitor.get_metrics_by_operation("threat_detection")
        
        return self._compile_results(
            "threat_detection",
            metrics,
            total_time,
            num_threats
        )
    
    async def benchmark_concurrent_orchestration(
        self,
        num_concurrent_agents: int = 10,
        operations_per_agent: int = 100
    ) -> BenchmarkResults:
        """
        Benchmark concurrent agent orchestration and coordination.
        
        Tests:
        - Multi-agent coordination
        - Event dispatching
        - Resource contention
        - Scaling characteristics
        """
        logger.info(
            "benchmark_concurrent_orchestration_start",
            num_agents=num_concurrent_agents,
            ops_per_agent=operations_per_agent
        )
        
        start_time = time.time()
        
        # Simulate concurrent agent operations
        async def agent_operations(agent_id: int) -> None:
            for op_id in range(operations_per_agent):
                op_start = time.time()
                
                try:
                    # Simulate agent processing with contention
                    await asyncio.sleep(0.001)  # 1ms operation
                    
                    elapsed = (time.time() - op_start) * 1000
                    
                    await self.monitor.record(
                        PerformanceMetrics(
                            operation="concurrent_operation",
                            duration_ms=elapsed,
                            success=True,
                            metadata={
                                "agent_id": agent_id,
                                "operation_id": op_id
                            }
                        )
                    )
                    
                except Exception as e:
                    elapsed = (time.time() - op_start) * 1000
                    await self.monitor.record(
                        PerformanceMetrics(
                            operation="concurrent_operation",
                            duration_ms=elapsed,
                            success=False,
                            metadata={"error": str(e)}
                        )
                    )
        
        # Run concurrent agents
        tasks = [
            agent_operations(i)
            for i in range(num_concurrent_agents)
        ]
        
        await asyncio.gather(*tasks)
        
        total_time = time.time() - start_time
        total_ops = num_concurrent_agents * operations_per_agent
        metrics = self.monitor.get_metrics_by_operation("concurrent_operation")
        
        return self._compile_results(
            "concurrent_orchestration",
            metrics,
            total_time,
            total_ops
        )
    
    async def benchmark_rollback_performance(
        self,
        num_rollbacks: int = 100,
        workflow_depth: int = 10
    ) -> BenchmarkResults:
        """
        Benchmark workflow rollback and recovery performance.
        
        Tests:
        - Rollback execution time
        - State restoration
        - Cleanup operations
        - Error recovery
        """
        logger.info(
            "benchmark_rollback_performance_start",
            num_rollbacks=num_rollbacks,
            workflow_depth=workflow_depth
        )
        
        start_time = time.time()
        
        # Simulate rollback operations
        async def perform_rollback(rollback_id: int) -> float:
            rollback_start = time.time()
            
            try:
                # Simulate rolling back multiple steps
                for step in range(workflow_depth):
                    await asyncio.sleep(0.002)  # 2ms per rollback step
                
                elapsed = (time.time() - rollback_start) * 1000
                
                await self.monitor.record(
                    PerformanceMetrics(
                        operation="rollback",
                        duration_ms=elapsed,
                        success=True,
                        metadata={
                            "rollback_id": rollback_id,
                            "steps": workflow_depth
                        }
                    )
                )
                
                return elapsed
                
            except Exception as e:
                elapsed = (time.time() - rollback_start) * 1000
                await self.monitor.record(
                    PerformanceMetrics(
                        operation="rollback",
                        duration_ms=elapsed,
                        success=False,
                        metadata={"error": str(e)}
                    )
                )
                return elapsed
        
        # Run rollbacks
        tasks = [perform_rollback(i) for i in range(num_rollbacks)]
        await asyncio.gather(*tasks)
        
        total_time = time.time() - start_time
        metrics = self.monitor.get_metrics_by_operation("rollback")
        
        return self._compile_results(
            "rollback_performance",
            metrics,
            total_time,
            num_rollbacks
        )
    
    def _compile_results(
        self,
        benchmark_name: str,
        metrics: list[PerformanceMetrics],
        total_time: float,
        num_operations: int
    ) -> BenchmarkResults:
        """Compile benchmark results from metrics."""
        if not metrics:
            return BenchmarkResults(
                benchmark_name=benchmark_name,
                total_operations=num_operations,
                successful_operations=0,
                failed_operations=num_operations,
                min_latency_ms=0,
                max_latency_ms=0,
                mean_latency_ms=0,
                median_latency_ms=0,
                p95_latency_ms=0,
                p99_latency_ms=0,
                stddev_latency_ms=0,
                operations_per_second=0,
                peak_memory_mb=0,
                average_memory_mb=0,
                success_rate=0
            )
        
        durations = [m.duration_ms for m in metrics]
        successful = sum(1 for m in metrics if m.success)
        failed = len(metrics) - successful
        
        return BenchmarkResults(
            benchmark_name=benchmark_name,
            total_operations=num_operations,
            successful_operations=successful,
            failed_operations=failed,
            min_latency_ms=min(durations),
            max_latency_ms=max(durations),
            mean_latency_ms=statistics.mean(durations),
            median_latency_ms=statistics.median(durations),
            p95_latency_ms=self._percentile(durations, 95),
            p99_latency_ms=self._percentile(durations, 99),
            stddev_latency_ms=statistics.stdev(durations) if len(durations) > 1 else 0,
            operations_per_second=num_operations / total_time,
            peak_memory_mb=0,  # TODO: Implement memory tracking
            average_memory_mb=0,  # TODO: Implement memory tracking
            success_rate=successful / num_operations if num_operations > 0 else 0
        )
    
    @staticmethod
    def _percentile(data: list[float], percentile: float) -> float:
        """Calculate percentile value."""
        if not data:
            return 0.0
        sorted_data = sorted(data)
        index = int(len(sorted_data) * percentile / 100)
        return sorted_data[min(index, len(sorted_data) - 1)]


# ═══════════════════════════════════════════════════════════════════════════════
# BENCHMARK RUNNER
# ═══════════════════════════════════════════════════════════════════════════════

class BenchmarkRunner:
    """Execute full benchmark suite and generate report."""
    
    def __init__(self):
        self.monitor = PerformanceMonitor()
        self.benchmark = OrchestrationBenchmark(self.monitor)
        self.results: list[BenchmarkResults] = []
    
    async def run_full_suite(self) -> list[BenchmarkResults]:
        """Run complete benchmark suite."""
        logger.info("benchmark_suite_start")
        
        start_time = datetime.now(UTC)
        
        # Run all benchmarks
        results = []
        
        logger.info("running_workflow_execution_benchmark")
        results.append(
            await self.benchmark.benchmark_workflow_execution(
                num_workflows=100,
                steps_per_workflow=5
            )
        )
        
        logger.info("running_state_transitions_benchmark")
        results.append(
            await self.benchmark.benchmark_state_transitions(
                num_transitions=1000,
                guard_complexity="simple"
            )
        )
        
        logger.info("running_threat_detection_benchmark")
        results.append(
            await self.benchmark.benchmark_threat_detection(
                num_threats=500,
                pattern_complexity="simple"
            )
        )
        
        logger.info("running_concurrent_orchestration_benchmark")
        results.append(
            await self.benchmark.benchmark_concurrent_orchestration(
                num_concurrent_agents=10,
                operations_per_agent=100
            )
        )
        
        logger.info("running_rollback_performance_benchmark")
        results.append(
            await self.benchmark.benchmark_rollback_performance(
                num_rollbacks=100,
                workflow_depth=10
            )
        )
        
        self.results = results
        
        end_time = datetime.now(UTC)
        duration = (end_time - start_time).total_seconds()
        
        logger.info(
            "benchmark_suite_complete",
            num_benchmarks=len(results),
            total_duration_seconds=duration
        )
        
        return results
    
    def generate_report(self) -> dict[str, Any]:
        """Generate benchmark report."""
        return {
            "timestamp": datetime.now(UTC).isoformat(),
            "total_benchmarks": len(self.results),
            "benchmarks": [r.to_dict() for r in self.results],
            "summary": {
                "avg_throughput_ops_per_sec": statistics.mean(
                    r.operations_per_second for r in self.results
                ) if self.results else 0,
                "avg_latency_ms": statistics.mean(
                    r.mean_latency_ms for r in self.results
                ) if self.results else 0,
                "overall_success_rate": statistics.mean(
                    r.success_rate for r in self.results
                ) if self.results else 0
            }
        }
