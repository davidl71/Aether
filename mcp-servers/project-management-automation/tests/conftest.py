"""
Pytest Configuration for MCP Server Tests

Provides fixtures and test configuration.
"""

import pytest
import sys
from pathlib import Path

# Add project root to path for imports
project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(Path(__file__).parent.parent))


@pytest.fixture
def project_root_path():
    """Fixture providing project root path."""
    return project_root


@pytest.fixture
def server_path():
    """Fixture providing server.py path."""
    return Path(__file__).parent.parent / 'server.py'


@pytest.fixture
def mcp_config_path():
    """Fixture providing MCP config path."""
    return project_root / '.cursor' / 'mcp.json'
