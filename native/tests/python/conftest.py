"""
conftest.py - pytest configuration and fixtures
"""
import pytest
import sys
from pathlib import Path

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "python"))


@pytest.fixture
def project_root_path():
    """Return project root path"""
    return Path(__file__).parent.parent.parent


@pytest.fixture
def sample_config():
    """Sample configuration for testing"""
    return {
        "tws": {
            "host": "127.0.0.1",
            "port": 7497,
            "client_id": 1,
        },
        "strategy": {
            "symbols": ["SPY", "QQQ"],
            "min_arbitrage_profit": 0.10,
            "min_roi_percent": 0.5,
        },
        "risk": {
            "max_total_exposure": 50000.0,
            "max_positions": 10,
        },
        "nautilus_trader": {
            "enabled": True,
            "data_client_config": {
                "host": "127.0.0.1",
                "port": 7497,
            },
            "exec_client_config": {
                "host": "127.0.0.1",
                "port": 7497,
            },
        },
    }



