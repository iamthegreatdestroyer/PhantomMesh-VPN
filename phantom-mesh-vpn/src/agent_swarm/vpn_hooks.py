"""
VPN Event Hooks — Agent-VPN Integration Layer
==============================================
Bridges Rust VPN core with Python agent swarm.

Copyright © 2025 Stephen Bilodeau. All rights reserved.
"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from datetime import datetime, UTC
from enum import Enum, auto
from typing import Any, Callable, Coroutine

import structlog

logger = structlog.get_logger(__name__)


class VPNEventType(Enum):
    """VPN core event classifications."""
    # Connection Events
    PEER_CONNECTED = auto()
    PEER_DISCONNECTED = auto()
    HANDSHAKE_INITIATED = auto()
    HANDSHAKE_COMPLETED = auto()

    # Traffic Events
    PACKET_ROUTED = auto()
    BANDWIDTH_THRESHOLD = auto()
    LATENCY_SPIKE = auto()

    # Security Events
    THREAT_SIGNATURE = auto()
    ANOMALY_DETECTED = auto()
    KEY_ROTATION_DUE = auto()

    # Mesh Events
    TOPOLOGY_CHANGE = auto()
    NODE_DISCOVERY = auto()
    ROUTE_OPTIMIZATION = auto()

    # System Events
    RESOURCE_WARNING = auto()
    CONFIG_RELOAD = auto()


@dataclass(frozen=True)
class VPNEvent:
    """Immutable VPN event payload."""
    event_type: VPNEventType
    timestamp: datetime
    source_node: str
    payload: dict[str, Any]
    priority: int = 1  # 1-10, higher = more urgent


class VPNHookRegistry:
    """
    Central registry for VPN event hooks.
    Enables agent swarm to react to VPN core events.
    """

    def __init__(self):
        self._hooks: dict[VPNEventType, list[Callable[[VPNEvent], Coroutine[Any, Any, None]]]] = {
            evt: [] for evt in VPNEventType
        }
        self._global_hooks: list[Callable[[VPNEvent], Coroutine[Any, Any, None]]] = []

    def register(
        self,
        event_type: VPNEventType,
        handler: Callable[[VPNEvent], Coroutine[Any, Any, None]]
    ) -> None:
        """Register handler for specific event type."""
        self._hooks[event_type].append(handler)
        logger.debug("hook_registered", event_type=event_type.name)

    def register_global(
        self,
        handler: Callable[[VPNEvent], Coroutine[Any, Any, None]]
    ) -> None:
        """Register handler for all events."""
        self._global_hooks.append(handler)
        logger.debug("global_hook_registered")

    async def emit(self, event: VPNEvent) -> None:
        """Emit event to all registered handlers."""
        handlers = self._hooks[event.event_type] + self._global_hooks

        if not handlers:
            logger.debug("event_unhandled", event_type=event.event_type.name)
            return

        # Fire all handlers concurrently
        tasks = [handler(event) for handler in handlers]
        results = await asyncio.gather(*tasks, return_exceptions=True)

        for i, result in enumerate(results):
            if isinstance(result, Exception):
                logger.error(
                    "hook_error",
                    event_type=event.event_type.name,
                    error=str(result)
                )


# Global registry instance
_registry = VPNHookRegistry()


def get_hook_registry() -> VPNHookRegistry:
    """Get the global hook registry."""
    return _registry


# ═══════════════════════════════════════════════════════════════════════════════
# CONVENIENCE DECORATORS
# ═══════════════════════════════════════════════════════════════════════════════

def on_vpn_event(*event_types: VPNEventType):
    """Decorator to register a function as an event handler."""
    def decorator(func: Callable[[VPNEvent], Coroutine[Any, Any, None]]):
        for evt in event_types:
            _registry.register(evt, func)
        return func
    return decorator


def on_all_vpn_events(func: Callable[[VPNEvent], Coroutine[Any, Any, None]]):
    """Decorator to register a function for all events."""
    _registry.register_global(func)
    return func


# ═══════════════════════════════════════════════════════════════════════════════
# RUST FFI BRIDGE (called from Rust via PyO3)
# ═══════════════════════════════════════════════════════════════════════════════

def rust_emit_event(
    event_type_str: str,
    source_node: str,
    payload_json: str,
    priority: int = 1
) -> None:
    """
    Entry point for Rust VPN core to emit events.
    Called via PyO3 FFI bridge.
    """
    import json

    try:
        event_type = VPNEventType[event_type_str]
        payload = json.loads(payload_json)

        event = VPNEvent(
            event_type=event_type,
            timestamp=datetime.now(UTC),
            source_node=source_node,
            payload=payload,
            priority=priority
        )

        # Schedule async emission
        asyncio.create_task(_registry.emit(event))

    except (KeyError, json.JSONDecodeError) as e:
        logger.error("rust_event_parse_error", error=str(e))