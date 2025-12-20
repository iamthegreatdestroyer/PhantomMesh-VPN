"""
Elite Agent Sub-Agents
======================
Individual agent implementations for the PhantomMesh swarm.

Copyright © 2025 Stephen Bilodeau. All rights reserved.
Licensed under GPL-3.0 with proprietary agent clauses.
"""

from .apex import ApexAgent
from .phantom import PhantomAgent
from .cipher import CipherAgent
from .velocity import VelocityAgent
from .fortress import FortressAgent
from .genesis import GenesisAgent
from .aegis import AegisAgent
from .nexus import NexusAgent
from .stream import StreamAgent
from .omniscient import OmniscientAgent

__all__ = [
    "ApexAgent",
    "PhantomAgent",
    "CipherAgent",
    "VelocityAgent",
    "FortressAgent",
    "GenesisAgent",
    "AegisAgent",
    "NexusAgent",
    "StreamAgent",
    "OmniscientAgent",
]