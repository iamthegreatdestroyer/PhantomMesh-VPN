"""
Pytest configuration and shared fixtures
"""

import pytest
import asyncio
import sys

# Ensure src is in path
sys.path.insert(0, 'src')


@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
def sample_threat_data():
    """Sample threat detection data."""
    return {
        "signature": bytes.fromhex("90909090"),
        "source_ip": "192.168.1.100",
        "timestamp": "2025-01-01T00:00:00Z"
    }


@pytest.fixture
def sample_peer_config():
    """Sample peer configuration."""
    return {
        "public_key": "test_key_base64",
        "endpoint": "127.0.0.1:51820",
        "allowed_ips": ["10.0.0.0/24"],
        "persistent_keepalive": 25
    }
