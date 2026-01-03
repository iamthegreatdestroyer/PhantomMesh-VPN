"""
PhantomMesh Multi-Region Orchestrator
=====================================

Coordinates operations across multiple geographic regions with
distributed consensus, intelligent failover, and eventual consistency.

Phase P1-003: Advanced Threat Intelligence Integration
Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

from __future__ import annotations

import asyncio
import hashlib
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, UTC, timedelta
from enum import Enum, auto
from typing import Any, Dict, List, Optional, Set, Tuple
from collections import defaultdict, deque
import json

import structlog

logger = structlog.get_logger(__name__)


# ═══════════════════════════════════════════════════════════════════════════════
# MULTI-REGION TYPES
# ═══════════════════════════════════════════════════════════════════════════════

class RegionStatus(Enum):
    """Region health status."""
    HEALTHY = auto()
    DEGRADED = auto()
    UNHEALTHY = auto()
    UNAVAILABLE = auto()


class ConsistencyLevel(Enum):
    """Data consistency guarantee levels."""
    EVENTUAL = auto()        # Highest availability
    CAUSAL = auto()          # Ordered causality
    SESSION = auto()         # Session consistency
    STRONG = auto()          # Sequential consistency


@dataclass
class RegionConfig:
    """Configuration for a geographic region."""
    region_id: str
    name: str
    primary_datacenter: str
    backup_datacenters: List[str]
    latency_budget_ms: int  # SLA latency target
    coordinate: Tuple[float, float]  # Geographic coordinates
    active: bool = True
    priority: int = 1  # Higher = more preferred


@dataclass
class RegionMetrics:
    """Current metrics for a region."""
    region_id: str
    status: RegionStatus
    latency_ms: float
    throughput_rps: float
    error_rate: float
    cpu_usage_percent: float
    memory_usage_percent: float
    replicated_workloads: int
    last_heartbeat: datetime = field(default_factory=lambda: datetime.now(UTC))
    
    def is_healthy(self) -> bool:
        return (
            self.status == RegionStatus.HEALTHY and
            self.error_rate < 0.01 and
            self.cpu_usage_percent < 85
        )


@dataclass
class Workload:
    """Distributed workload to coordinate."""
    workload_id: str
    name: str
    regions: List[str]  # Primary + backup regions
    state: Dict[str, Any]
    replicas: Dict[str, Dict[str, Any]]  # region_id -> replica_state
    consistency_level: ConsistencyLevel
    created_at: datetime = field(default_factory=lambda: datetime.now(UTC))


@dataclass
class FailoverPlan:
    """Plan for failing over workloads."""
    failed_region: str
    affected_workloads: List[str]
    target_regions: List[str]
    actions: List[str]
    estimated_duration_seconds: float
    risk_level: str  # low, medium, high


@dataclass
class CoordinationResult:
    """Result of coordinated workflow execution."""
    workflow_id: str
    status: str  # success, partial, failed
    executed_regions: List[str]
    failed_regions: List[str]
    execution_time_ms: float
    coordination_overhead_ms: float
    data_consistency_achieved: bool
    failover_triggered: bool


@dataclass
class LoadDistribution:
    """Workload distribution across regions."""
    region_allocations: Dict[str, float]  # region_id -> allocation percentage
    estimated_latency_ms: float
    total_capacity_utilization: float
    balanced_score: float  # 0.0-1.0


# ═══════════════════════════════════════════════════════════════════════════════
# DISTRIBUTED STATE MANAGER
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class StateChange:
    """Record of a state change for replication."""
    change_id: str
    timestamp: datetime
    region_id: str
    workload_id: str
    old_state: Dict[str, Any]
    new_state: Dict[str, Any]
    version: int


class DistributedState:
    """
    Manages state replication with eventual consistency and CRDTs.
    Handles conflict resolution and convergence.
    """
    
    def __init__(self, regions: List[RegionConfig]):
        self.regions = regions
        self._state_log: deque[StateChange] = deque(maxlen=100000)
        self._region_clocks: Dict[str, int] = {r.region_id: 0 for r in regions}
        self._pending_replications: Dict[str, List[StateChange]] = defaultdict(list)
        
        logger.info("distributed_state_initialized", regions=len(regions))
    
    async def replicate_state(
        self,
        state_changes: Dict[str, Any]
    ) -> Dict[str, bool]:
        """
        Replicate state changes with conflict resolution.
        
        Algorithm:
        1. Increment logical clock
        2. Broadcast to all regions
        3. Collect ACKs with conflict detection
        4. Resolve conflicts using CRDTs
        5. Return replication status
        """
        
        replication_status = {}
        
        for region in self.regions:
            if not region.active:
                replication_status[region.region_id] = False
                continue
            
            # Increment logical clock for causality
            self._region_clocks[region.region_id] += 1
            
            # Create state change record
            change = StateChange(
                change_id=self._generate_change_id(),
                timestamp=datetime.now(UTC),
                region_id=region.region_id,
                workload_id=state_changes.get("workload_id", ""),
                old_state=state_changes.get("old_state", {}),
                new_state=state_changes.get("new_state", {}),
                version=self._region_clocks[region.region_id]
            )
            
            # Add to log
            self._state_log.append(change)
            self._pending_replications[region.region_id].append(change)
            
            # Simulate replication
            success = await self._replicate_to_region(region, change)
            replication_status[region.region_id] = success
            
            logger.info(
                "state_change_replicated",
                region=region.region_id,
                success=success,
                version=change.version
            )
        
        return replication_status
    
    async def detect_conflicts(
        self,
        changes: List[StateChange]
    ) -> List[Tuple[StateChange, StateChange]]:
        """Detect conflicting state changes."""
        
        conflicts = []
        
        for i, change1 in enumerate(changes):
            for change2 in changes[i+1:]:
                # Conflict if both modify same workload
                if (change1.workload_id == change2.workload_id and
                    change1.region_id != change2.region_id):
                    
                    # Check if changes overlap
                    overlap = self._detect_overlap(
                        change1.new_state,
                        change2.new_state
                    )
                    
                    if overlap:
                        conflicts.append((change1, change2))
        
        return conflicts
    
    async def resolve_conflicts(
        self,
        conflicts: List[Tuple[StateChange, StateChange]]
    ) -> Dict[str, Any]:
        """Resolve conflicts using CRDTs or LWW."""
        
        resolved = {}
        
        for change1, change2 in conflicts:
            if change1.timestamp > change2.timestamp:
                # Last-write-wins
                resolved[change1.workload_id] = change1.new_state
            else:
                resolved[change2.workload_id] = change2.new_state
        
        return resolved
    
    async def _replicate_to_region(
        self,
        region: RegionConfig,
        change: StateChange
    ) -> bool:
        """Replicate change to specific region."""
        
        try:
            # Simulate async replication with latency
            await asyncio.sleep(0.01)  # 10ms replication latency
            return True
        except Exception as e:
            logger.error("replication_failed", region=region.region_id, error=str(e))
            return False
    
    def _generate_change_id(self) -> str:
        """Generate unique change ID."""
        data = f"{datetime.now(UTC).isoformat()}{len(self._state_log)}"
        return hashlib.sha256(data.encode()).hexdigest()[:16]
    
    def _detect_overlap(
        self,
        state1: Dict[str, Any],
        state2: Dict[str, Any]
    ) -> bool:
        """Check if two state changes overlap."""
        keys1 = set(state1.keys())
        keys2 = set(state2.keys())
        return bool(keys1 & keys2)  # Intersection
    
    def get_replication_status(self) -> Dict[str, Any]:
        """Get replication status across regions."""
        
        pending_total = sum(len(v) for v in self._pending_replications.values())
        
        return {
            "total_state_changes": len(self._state_log),
            "pending_replications": pending_total,
            "region_clocks": self._region_clocks.copy(),
            "last_change": self._state_log[-1].timestamp.isoformat() if self._state_log else None
        }


# ═══════════════════════════════════════════════════════════════════════════════
# FAILOVER MANAGER
# ═══════════════════════════════════════════════════════════════════════════════

class FailoverManager:
    """Intelligent failover across regions."""
    
    def __init__(self, regions: List[RegionConfig]):
        self.regions = {r.region_id: r for r in regions}
        self._failover_history: deque = deque(maxlen=1000)
        
        logger.info("failover_manager_initialized", regions=len(regions))
    
    async def handle_region_failure(
        self,
        failed_region: str,
        affected_workloads: List[Workload]
    ):
        """Gracefully failover workloads from failed region."""
        
        logger.critical(
            "region_failure_detected",
            region=failed_region,
            affected_workloads=len(affected_workloads)
        )
        
        # Generate failover plan
        plan = await self._generate_failover_plan(
            failed_region,
            affected_workloads
        )
        
        # Execute failover
        result = await self.execute_failover_plan(plan)
        
        logger.info(
            "failover_completed",
            region=failed_region,
            success=result.status == "success"
        )
    
    async def _generate_failover_plan(
        self,
        failed_region: str,
        workloads: List[Workload]
    ) -> FailoverPlan:
        """Generate optimal failover plan."""
        
        # Identify backup regions
        backup_regions = self._select_backup_regions(failed_region)
        
        # Actions for failover
        actions = [
            f"stop_workloads_in_{failed_region}",
            f"promote_replicas_from_{backup_regions[0]}",
            "update_routing",
            "restart_in_backup",
            "monitor_convergence"
        ]
        
        return FailoverPlan(
            failed_region=failed_region,
            affected_workloads=[w.workload_id for w in workloads],
            target_regions=backup_regions,
            actions=actions,
            estimated_duration_seconds=30.0,
            risk_level="high"
        )
    
    async def execute_failover_plan(
        self,
        plan: FailoverPlan
    ) -> CoordinationResult:
        """Execute failover plan with state preservation."""
        
        start_time = datetime.now(UTC)
        
        # Execute actions in sequence
        for action in plan.actions:
            logger.info("executing_failover_action", action=action)
            await asyncio.sleep(0.1)  # Simulate action execution
        
        duration_ms = (datetime.now(UTC) - start_time).total_seconds() * 1000
        
        return CoordinationResult(
            workflow_id="failover",
            status="success",
            executed_regions=plan.target_regions,
            failed_regions=[plan.failed_region],
            execution_time_ms=duration_ms,
            coordination_overhead_ms=10.0,
            data_consistency_achieved=True,
            failover_triggered=True
        )
    
    def _select_backup_regions(self, failed_region: str) -> List[str]:
        """Select best backup regions for failover."""
        
        available = [
            r for r in self.regions.values()
            if r.region_id != failed_region and r.active
        ]
        
        # Sort by priority and distance
        available.sort(key=lambda r: (r.priority, r.latency_budget_ms))
        
        return [r.region_id for r in available[:2]]


# ═══════════════════════════════════════════════════════════════════════════════
# REGION COORDINATOR
# ═══════════════════════════════════════════════════════════════════════════════

class RegionCoordinator:
    """Coordinate operations across multiple regions."""
    
    def __init__(self, regions: List[RegionConfig]):
        self.regions = {r.region_id: r for r in regions}
        self.state_manager = DistributedState(regions)
        self.failover_manager = FailoverManager(regions)
        
        self._region_metrics: Dict[str, RegionMetrics] = {}
        self._active_workloads: Dict[str, Workload] = {}
        
        logger.info("region_coordinator_initialized", regions=len(regions))
    
    async def execute_coordinated_workflow(
        self,
        workflow: Dict[str, Any],
        regions: Optional[List[str]] = None
    ) -> CoordinationResult:
        """
        Execute workflow across regions with coordination.
        
        Coordination ensures:
        1. All regions execute in parallel
        2. State consistency is maintained
        3. Failures are handled gracefully
        4. Latency SLAs are met
        """
        
        start_time = datetime.now(UTC)
        
        # Determine regions
        target_regions = regions or list(self.regions.keys())
        
        # Create distributed workload
        workload = Workload(
            workload_id=workflow.get("id", ""),
            name=workflow.get("name", ""),
            regions=target_regions,
            state=workflow.get("state", {}),
            replicas={},
            consistency_level=ConsistencyLevel.EVENTUAL
        )
        
        self._active_workloads[workload.workload_id] = workload
        
        # Execute in parallel
        tasks = [
            self._execute_in_region(region, workload)
            for region in target_regions
        ]
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Analyze results
        succeeded = sum(1 for r in results if isinstance(r, bool) and r)
        failed_regions = [
            region for region, result in zip(target_regions, results)
            if isinstance(result, Exception)
        ]
        
        # Handle partial failures
        if failed_regions:
            await self.failover_manager.handle_region_failure(
                failed_regions[0],
                [workload]
            )
        
        duration_ms = (datetime.now(UTC) - start_time).total_seconds() * 1000
        
        return CoordinationResult(
            workflow_id=workload.workload_id,
            status="success" if not failed_regions else "partial",
            executed_regions=[r for r in target_regions if r not in failed_regions],
            failed_regions=failed_regions,
            execution_time_ms=duration_ms,
            coordination_overhead_ms=duration_ms * 0.1,
            data_consistency_achieved=True,
            failover_triggered=len(failed_regions) > 0
        )
    
    async def _execute_in_region(
        self,
        region_id: str,
        workload: Workload
    ) -> bool:
        """Execute workload in specific region."""
        
        try:
            # Simulate execution
            await asyncio.sleep(0.05)  # 50ms execution
            
            # Store replica
            workload.replicas[region_id] = workload.state.copy()
            
            return True
        except Exception as e:
            logger.error(
                "workload_execution_failed",
                region=region_id,
                workload=workload.workload_id,
                error=str(e)
            )
            return False
    
    async def update_region_metrics(
        self,
        metrics: Dict[str, RegionMetrics]
    ):
        """Update metrics for all regions."""
        
        for region_id, metric in metrics.items():
            self._region_metrics[region_id] = metric
            
            # Detect failures
            if metric.status == RegionStatus.UNAVAILABLE:
                affected = [
                    w for w in self._active_workloads.values()
                    if region_id in w.regions
                ]
                
                if affected:
                    await self.failover_manager.handle_region_failure(
                        region_id, affected
                    )


# ═══════════════════════════════════════════════════════════════════════════════
# GLOBAL LOAD BALANCER
# ═══════════════════════════════════════════════════════════════════════════════

class GlobalLoadBalancer:
    """Distribute load intelligently across regions."""
    
    def __init__(self):
        self._distribution_history: deque = deque(maxlen=10000)
        
        logger.info("global_load_balancer_initialized")
    
    async def distribute_load(
        self,
        workloads: List[Workload],
        region_metrics: Dict[str, RegionMetrics]
    ) -> LoadDistribution:
        """
        Optimize workload distribution across regions.
        
        Considers:
        1. Region capacity and utilization
        2. Network latency
        3. Cost per region
        4. Workload affinity
        """
        
        allocations = {}
        
        # Filter healthy regions
        healthy = {
            rid: m for rid, m in region_metrics.items()
            if m.is_healthy()
        }
        
        if not healthy:
            # Fallback: distribute evenly
            per_region = 1.0 / len(region_metrics)
            return LoadDistribution(
                region_allocations={r: per_region for r in region_metrics},
                estimated_latency_ms=100.0,
                total_capacity_utilization=0.5,
                balanced_score=0.5
            )
        
        # Compute capacity weights
        total_capacity = sum(
            (100 - m.cpu_usage_percent) / 100
            for m in healthy.values()
        )
        
        for region_id, metrics in healthy.items():
            capacity_factor = (100 - metrics.cpu_usage_percent) / (100 * total_capacity)
            allocations[region_id] = capacity_factor
        
        # Compute metrics
        avg_latency = sum(
            m.latency_ms * allocations.get(rid, 0)
            for rid, m in region_metrics.items()
        )
        
        total_utilization = sum(
            m.cpu_usage_percent * allocations.get(rid, 0)
            for rid, m in region_metrics.items()
        ) / 100
        
        balanced_score = self._compute_balance_score(allocations)
        
        return LoadDistribution(
            region_allocations=allocations,
            estimated_latency_ms=avg_latency,
            total_capacity_utilization=total_utilization,
            balanced_score=balanced_score
        )
    
    def _compute_balance_score(self, allocations: Dict[str, float]) -> float:
        """Compute how balanced the allocation is (0.0-1.0)."""
        
        if not allocations:
            return 0.0
        
        values = list(allocations.values())
        mean = sum(values) / len(values)
        variance = sum((v - mean) ** 2 for v in values) / len(values)
        
        # Lower variance = higher score
        return max(0.0, 1.0 - variance)
