"""
PhantomMesh Agent Swarm
=======================
Elite Agent Collective for autonomous VPN orchestration.

Copyright © 2025 Stephen Bilodeau. All rights reserved.
Licensed under GPL-3.0 with proprietary agent clauses.
"""

from .phantom_orchestrator import PhantomOrchestrator
from .vpn_hooks import get_hook_registry, VPNEvent, VPNEventType

__version__ = "0.1.0"
__author__ = "Stephen Bilodeau"
__license__ = "GPL-3.0"

__all__ = [
    "PhantomOrchestrator",
    "get_hook_registry",
    "VPNEvent",
    "VPNEventType",
]