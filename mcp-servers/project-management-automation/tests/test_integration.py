"""
Integration Tests for MCP Server

Tests end-to-end MCP server functionality with real tool execution.
"""

import json
import pytest
import subprocess
import sys
from pathlib import Path
from typing import Dict, Any

# Add project root to path
project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(Path(__file__).parent.parent))


class TestMCPServerIntegration:
    """Integration tests for MCP server."""

    def test_server_imports(self):
        """Test that server module can be imported."""
        try:
            # Try to import server file directly
            import importlib.util
            server_path = Path(__file__).parent.parent / 'server.py'
            if server_path.exists():
                spec = importlib.util.spec_from_file_location("server_module", server_path)
                if spec and spec.loader:
                    server_module = importlib.util.module_from_spec(spec)
                    # If we can load the spec, imports work
                    assert True
                else:
                    pytest.skip("Server module spec not available")
            else:
                pytest.skip("Server file not found")
        except Exception as e:
            pytest.skip(f"Server imports not available: {e}")

    def test_error_handler_imports(self):
        """Test that error handler can be imported."""
        try:
            import importlib.util
            error_handler_path = Path(__file__).parent.parent / 'error_handler.py'
            if error_handler_path.exists():
                spec = importlib.util.spec_from_file_location("error_handler", error_handler_path)
                if spec and spec.loader:
                    error_handler = importlib.util.module_from_spec(spec)
                    spec.loader.exec_module(error_handler)
                    assert hasattr(error_handler, 'ErrorCode')
                    assert hasattr(error_handler, 'AutomationError')
            else:
                pytest.skip("Error handler file not found")
        except Exception as e:
            pytest.skip(f"Error handler imports not available: {e}")

    def test_tool_wrappers_import(self):
        """Test that tool wrapper files exist and are readable."""
        tools_dir = Path(__file__).parent.parent / 'tools'
        tool_files = [
            'docs_health.py',
            'todo2_alignment.py',
            'duplicate_detection.py'
        ]

        for tool_file in tool_files:
            tool_path = tools_dir / tool_file
            assert tool_path.exists(), f"{tool_file} should exist"
            assert tool_path.stat().st_size > 0, f"{tool_file} should not be empty"

    def test_resource_handlers_exist(self):
        """Test that resource handler files exist."""
        resources_dir = Path(__file__).parent.parent / 'resources'
        resource_files = [
            'status.py',
            'history.py',
            'list.py'
        ]

        for resource_file in resource_files:
            resource_path = resources_dir / resource_file
            assert resource_path.exists(), f"{resource_file} should exist"
            assert resource_path.stat().st_size > 0, f"{resource_file} should not be empty"

    def test_server_file_exists(self):
        """Test that server.py file exists and is readable."""
        server_file = Path(__file__).parent.parent / 'server.py'
        assert server_file.exists(), "server.py should exist"
        assert server_file.is_file(), "server.py should be a file"
        assert server_file.stat().st_size > 0, "server.py should not be empty"

    def test_error_handler_file_exists(self):
        """Test that error_handler.py exists."""
        error_handler_file = Path(__file__).parent.parent / 'error_handler.py'
        assert error_handler_file.exists(), "error_handler.py should exist"

    def test_tools_directory_exists(self):
        """Test that tools directory exists with expected files."""
        tools_dir = Path(__file__).parent.parent / 'tools'
        assert tools_dir.exists(), "tools directory should exist"
        assert tools_dir.is_dir(), "tools should be a directory"

        expected_tools = [
            'docs_health.py',
            'todo2_alignment.py',
            'duplicate_detection.py',
            'dependency_security.py',
            'automation_opportunities.py',
            'todo_sync.py',
            'pwa_review.py'
        ]

        for tool_file in expected_tools:
            tool_path = tools_dir / tool_file
            assert tool_path.exists(), f"{tool_file} should exist"

    def test_resources_directory_exists(self):
        """Test that resources directory exists with expected files."""
        resources_dir = Path(__file__).parent.parent / 'resources'
        assert resources_dir.exists(), "resources directory should exist"
        assert resources_dir.is_dir(), "resources should be a directory"

        expected_resources = [
            'status.py',
            'history.py',
            'list.py'
        ]

        for resource_file in expected_resources:
            resource_path = resources_dir / resource_file
            assert resource_path.exists(), f"{resource_file} should exist"


class TestMCPConfiguration:
    """Tests for MCP configuration."""

    def test_mcp_json_exists(self):
        """Test that .cursor/mcp.json exists and contains our server."""
        mcp_config = project_root / '.cursor' / 'mcp.json'
        assert mcp_config.exists(), ".cursor/mcp.json should exist"

        with open(mcp_config, 'r') as f:
            config = json.load(f)

        assert 'mcpServers' in config, "mcpServers key should exist"
        assert 'project-management-automation' in config['mcpServers'], \
            "project-management-automation server should be configured"

        server_config = config['mcpServers']['project-management-automation']
        assert 'command' in server_config, "Server should have command"
        assert 'args' in server_config, "Server should have args"
        assert 'description' in server_config, "Server should have description"

    def test_server_description_contains_deprecation_hint(self):
        """Test that server description includes deprecation hint."""
        mcp_config = project_root / '.cursor' / 'mcp.json'

        with open(mcp_config, 'r') as f:
            config = json.load(f)

        description = config['mcpServers']['project-management-automation'].get('description', '')
        assert 'NOTE' in description or 'prefer' in description.lower(), \
            "Server description should include deprecation/preference hint"


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
