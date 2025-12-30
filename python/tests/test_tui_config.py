"""
Tests for TUI configuration management.

Tests TUIConfig class and load_config() function.
"""

import unittest
import json
import os
import tempfile
from pathlib import Path
from unittest.mock import patch, mock_open

import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from tui.config import TUIConfig, load_config


class TestTUIConfig(unittest.TestCase):
    """Tests for TUIConfig dataclass."""

    def test_default_values(self):
        """Test TUIConfig with default values."""
        config = TUIConfig()

        assert config.provider_type == "mock"
        assert config.rest_endpoint == "http://localhost:8080/api/snapshot"
        assert config.update_interval_ms == 1000
        assert config.refresh_rate_ms == 500
        assert config.rest_timeout_ms == 5000
        assert config.rest_verify_ssl is False
        assert config.file_path is None
        assert config.ibkr_rest_base_url == "https://localhost:5000/v1/portal"
        assert config.ibkr_rest_account_id == ""
        assert config.ibkr_rest_verify_ssl is False
        assert config.ibkr_rest_timeout_ms == 5000
        assert config.show_colors is True
        assert config.show_footer is True

    def test_custom_values(self):
        """Test TUIConfig with custom values."""
        config = TUIConfig(
            provider_type="rest",
            rest_endpoint="https://api.example.com",
            update_interval_ms=2000,
            show_colors=False,
        )

        assert config.provider_type == "rest"
        assert config.rest_endpoint == "https://api.example.com"
        assert config.update_interval_ms == 2000
        assert config.show_colors is False

    def test_to_dict(self):
        """Test to_dict() method."""
        config = TUIConfig(provider_type="rest", update_interval_ms=2000)
        config_dict = config.to_dict()

        assert isinstance(config_dict, dict)
        assert config_dict["provider_type"] == "rest"
        assert config_dict["update_interval_ms"] == 2000
        assert config_dict["show_colors"] is True

    def test_from_dict(self):
        """Test from_dict() class method."""
        data = {
            "provider_type": "file",
            "rest_endpoint": "http://test.com",
            "update_interval_ms": 3000,
            "file_path": "/path/to/file.json",
            "show_colors": False,
        }

        config = TUIConfig.from_dict(data)

        assert config.provider_type == "file"
        assert config.rest_endpoint == "http://test.com"
        assert config.update_interval_ms == 3000
        assert config.file_path == "/path/to/file.json"
        assert config.show_colors is False

    def test_save_to_file(self):
        """Test save_to_file() method."""
        config = TUIConfig(provider_type="rest", update_interval_ms=2000)

        with tempfile.TemporaryDirectory() as tmpdir:
            config_path = Path(tmpdir) / "test_config.json"
            config.save_to_file(str(config_path))

            # Verify file was created
            assert config_path.exists()

            # Verify file contents
            with open(config_path, "r") as f:
                loaded_data = json.load(f)

            assert loaded_data["provider_type"] == "rest"
            assert loaded_data["update_interval_ms"] == 2000

    def test_save_to_file_creates_directory(self):
        """Test that save_to_file() creates parent directory if needed."""
        config = TUIConfig()

        with tempfile.TemporaryDirectory() as tmpdir:
            config_path = Path(tmpdir) / "nested" / "dir" / "config.json"
            config.save_to_file(str(config_path))

            # Verify directory and file were created
            assert config_path.exists()
            assert config_path.parent.exists()

    def test_load_from_file_exists(self):
        """Test load_from_file() when file exists."""
        config_data = {
            "provider_type": "rest",
            "rest_endpoint": "https://api.example.com",
            "update_interval_ms": 2000,
            "show_colors": False,
        }

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            config = TUIConfig.load_from_file(temp_path)

            assert config.provider_type == "rest"
            assert config.rest_endpoint == "https://api.example.com"
            assert config.update_interval_ms == 2000
            assert config.show_colors is False
        finally:
            Path(temp_path).unlink()

    def test_load_from_file_not_exists(self):
        """Test load_from_file() when file doesn't exist."""
        config = TUIConfig.load_from_file("/nonexistent/path/config.json")

        # Should return default config
        assert config.provider_type == "mock"
        assert config.rest_endpoint == "http://localhost:8080/api/snapshot"

    def test_load_from_file_invalid_json(self):
        """Test load_from_file() with invalid JSON."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            f.write("{invalid json}")
            temp_path = f.name

        try:
            config = TUIConfig.load_from_file(temp_path)

            # Should return default config on error
            assert config.provider_type == "mock"
        finally:
            Path(temp_path).unlink()

    def test_get_config_path(self):
        """Test get_config_path() class method."""
        with tempfile.TemporaryDirectory() as tmpdir:
            with patch.dict(os.environ, {"HOME": tmpdir}):
                config_path = TUIConfig.get_config_path()

                expected_path = (
                    Path(tmpdir) / ".config" / "ib_box_spread" / "tui_config.json"
                )
                assert config_path == str(expected_path)
                # Verify directory was created
                assert expected_path.parent.exists()

    def test_get_config_path_no_home(self):
        """Test get_config_path() when HOME env var is not set."""
        with patch.dict(os.environ, {}, clear=True):
            # Remove HOME if it exists
            if "HOME" in os.environ:
                del os.environ["HOME"]

            config_path = TUIConfig.get_config_path()

            # Should return default filename
            assert config_path == "tui_config.json"

    def test_load_default(self):
        """Test load_default() class method."""
        config = TUIConfig.load_default()

        # Should return default config
        assert config.provider_type == "mock"
        assert config.rest_endpoint == "http://localhost:8080/api/snapshot"


class TestLoadConfig(unittest.TestCase):
    """Tests for load_config() function."""

    def test_load_config_from_file(self):
        """Test load_config() loads from file."""
        config_data = {
            "provider_type": "rest",
            "rest_endpoint": "https://api.example.com",
            "update_interval_ms": 2000,
        }

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch("tui.config.TUIConfig.get_config_path", return_value=temp_path):
                config = load_config()

                assert config.provider_type == "rest"
                assert config.rest_endpoint == "https://api.example.com"
                assert config.update_interval_ms == 2000
        finally:
            Path(temp_path).unlink()

    def test_load_config_env_override_provider_type(self):
        """Test that TUI_BACKEND env var overrides provider_type."""
        config_data = {"provider_type": "file"}

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch("tui.config.TUIConfig.get_config_path", return_value=temp_path):
                with patch.dict(os.environ, {"TUI_BACKEND": "rest"}):
                    config = load_config()

                    # Environment variable should override file
                    assert config.provider_type == "rest"
        finally:
            Path(temp_path).unlink()

    def test_load_config_env_override_rest_endpoint(self):
        """Test that TUI_API_URL env var overrides rest_endpoint."""
        config_data = {"rest_endpoint": "http://file.example.com"}

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch("tui.config.TUIConfig.get_config_path", return_value=temp_path):
                with patch.dict(os.environ, {"TUI_API_URL": "https://env.example.com"}):
                    config = load_config()

                    # Environment variable should override file
                    assert config.rest_endpoint == "https://env.example.com"
        finally:
            Path(temp_path).unlink()

    def test_load_config_env_override_file_path(self):
        """Test that TUI_SNAPSHOT_FILE env var overrides file_path."""
        config_data = {"file_path": "/file/path.json"}

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch("tui.config.TUIConfig.get_config_path", return_value=temp_path):
                with patch.dict(os.environ, {"TUI_SNAPSHOT_FILE": "/env/path.json"}):
                    config = load_config()

                    # Environment variable should override file
                    assert config.file_path == "/env/path.json"
        finally:
            Path(temp_path).unlink()

    def test_load_config_no_file_uses_defaults(self):
        """Test load_config() uses defaults when file doesn't exist."""
        with patch(
            "tui.config.TUIConfig.get_config_path",
            return_value="/nonexistent/config.json",
        ):
            config = load_config()

            # Should use defaults
            assert config.provider_type == "mock"
            assert config.rest_endpoint == "http://localhost:8080/api/snapshot"


if __name__ == "__main__":
    unittest.main()
