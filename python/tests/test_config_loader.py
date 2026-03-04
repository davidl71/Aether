"""
Tests for config loader module.

Tests configuration loading functions with various file locations and environment variables.
"""
import unittest
import json
import os
import tempfile
from pathlib import Path
from unittest.mock import patch, MagicMock

import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from integration.config_loader import (
    _find_config_file,
    get_service_port,
    check_port_available,
    get_config_value
)


class TestFindConfigFile(unittest.TestCase):
    """Tests for _find_config_file() function."""

    def test_find_config_file_explicit_path(self):
        """Test _find_config_file() with explicit path."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write('{"test": "data"}')
            temp_path = f.name

        try:
            result = _find_config_file(temp_path)
            assert result == Path(temp_path)
        finally:
            os.unlink(temp_path)

    def test_find_config_file_env_override(self):
        """Test _find_config_file() with environment variable override."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write('{"test": "data"}')
            temp_path = f.name

        try:
            with patch.dict(os.environ, {"IB_BOX_SPREAD_CONFIG": temp_path}):
                result = _find_config_file()
                assert result == Path(temp_path)
        finally:
            os.unlink(temp_path)

    def test_find_config_file_not_found(self):
        """Test _find_config_file() when no config file exists."""
        with patch.dict(os.environ, {}, clear=True):
            # Mock Path.exists() to return False for all candidates
            with patch('integration.config_loader.Path.exists', return_value=False):
                result = _find_config_file()
                assert result is None

    def test_find_config_file_home_config(self):
        """Test _find_config_file() finds config in home directory."""
        with tempfile.TemporaryDirectory() as tmpdir:
            config_dir = Path(tmpdir) / ".config" / "ib_box_spread"
            config_dir.mkdir(parents=True)
            config_file = config_dir / "config.json"
            config_file.write_text('{"test": "data"}')

            with patch('integration.config_loader.Path.home', return_value=Path(tmpdir)):
                with patch.dict(os.environ, {}, clear=True):
                    result = _find_config_file()
                    assert result == config_file


class TestGetServicePort(unittest.TestCase):
    """Tests for get_service_port() function."""

    def test_get_service_port_env_override(self):
        """Test get_service_port() with environment variable override."""
        with patch.dict(os.environ, {"IB_PORT": "7497"}):
            port = get_service_port("ib")
            assert port == 7497

    def test_get_service_port_env_override_invalid(self):
        """Test get_service_port() with invalid environment variable."""
        with patch.dict(os.environ, {"IB_PORT": "invalid"}):
            with self.assertRaises(ValueError):
                get_service_port("ib")

    def test_get_service_port_from_config(self):
        """Test get_service_port() reads from config file."""
        config_data = {
            "services": {
                "ib": {"port": 7497},
                "alpaca": {"port": 8080}
            }
        }

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch('integration.config_loader._find_config_file', return_value=Path(temp_path)):
                with patch.dict(os.environ, {}, clear=True):
                    port = get_service_port("ib")
                    assert port == 7497
        finally:
            os.unlink(temp_path)

    def test_get_service_port_default(self):
        """Test get_service_port() uses default when not found."""
        with patch('integration.config_loader._find_config_file', return_value=None):
            with patch.dict(os.environ, {}, clear=True):
                port = get_service_port("ib", default_port=7497)
                assert port == 7497

    def test_get_service_port_no_default_error(self):
        """Test get_service_port() raises error when no port found and no default."""
        with patch('integration.config_loader._find_config_file', return_value=None):
            with patch.dict(os.environ, {}, clear=True):
                with self.assertRaises(ValueError) as cm:
                    get_service_port("ib")
                assert "Port not found" in str(cm.exception)

    def test_get_service_port_env_priority(self):
        """Test get_service_port() prioritizes environment variable over config."""
        config_data = {"services": {"ib": {"port": 7496}}}

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch('integration.config_loader._find_config_file', return_value=Path(temp_path)):
                with patch.dict(os.environ, {"IB_PORT": "7497"}):
                    port = get_service_port("ib")
                    assert port == 7497  # Env var takes priority
        finally:
            os.unlink(temp_path)

    def test_get_service_port_service_name_normalization(self):
        """Test get_service_port() handles service name with underscores."""
        with patch.dict(os.environ, {"WEBBACKEND_PORT": "8080"}):
            port = get_service_port("web_backend")
            assert port == 8080


class TestCheckPortAvailable(unittest.TestCase):
    """Tests for check_port_available() function."""

    @patch('socket.socket')
    def test_check_port_available_true(self, mock_socket_class):
        """Test check_port_available() returns True for available port."""
        mock_socket = MagicMock()
        mock_socket.connect_ex.return_value = 1  # Connection failed = port available
        mock_socket.__enter__ = MagicMock(return_value=mock_socket)
        mock_socket.__exit__ = MagicMock(return_value=False)
        mock_socket_class.return_value = mock_socket

        result = check_port_available(8080)
        assert result is True

    @patch('socket.socket')
    def test_check_port_available_false(self, mock_socket_class):
        """Test check_port_available() returns False for port in use."""
        mock_socket = MagicMock()
        mock_socket.connect_ex.return_value = 0  # Connection succeeded = port in use
        mock_socket.__enter__ = MagicMock(return_value=mock_socket)
        mock_socket.__exit__ = MagicMock(return_value=False)
        mock_socket_class.return_value = mock_socket

        result = check_port_available(8080)
        assert result is False

    @patch('socket.socket')
    def test_check_port_available_exception(self, mock_socket_class):
        """Test check_port_available() returns True on exception (assumes available)."""
        mock_socket_class.side_effect = Exception("Socket error")

        result = check_port_available(8080)
        assert result is True  # Assumes available on error


class TestGetConfigValue(unittest.TestCase):
    """Tests for get_config_value() function."""

    def test_get_config_value_simple_path(self):
        """Test get_config_value() with simple JSON path."""
        config_data = {"tws": {"port": 7497}}

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch('integration.config_loader._find_config_file', return_value=Path(temp_path)):
                value = get_config_value("tws.port")
                assert value == 7497
        finally:
            os.unlink(temp_path)

    def test_get_config_value_nested_path(self):
        """Test get_config_value() with nested JSON path."""
        config_data = {
            "services": {
                "ib": {
                    "tws": {"port": 7497}
                }
            }
        }

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch('integration.config_loader._find_config_file', return_value=Path(temp_path)):
                value = get_config_value("services.ib.tws.port")
                assert value == 7497
        finally:
            os.unlink(temp_path)

    def test_get_config_value_not_found(self):
        """Test get_config_value() returns default when path not found."""
        config_data = {"other": "data"}

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch('integration.config_loader._find_config_file', return_value=Path(temp_path)):
                value = get_config_value("tws.port", default_value=7497)
                assert value == 7497
        finally:
            os.unlink(temp_path)

    def test_get_config_value_no_config_file(self):
        """Test get_config_value() returns default when no config file."""
        with patch('integration.config_loader._find_config_file', return_value=None):
            value = get_config_value("tws.port", default_value=7497)
            assert value == 7497

    def test_get_config_value_invalid_json(self):
        """Test get_config_value() returns default on invalid JSON."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write("invalid json")
            temp_path = f.name

        try:
            with patch('integration.config_loader._find_config_file', return_value=Path(temp_path)):
                value = get_config_value("tws.port", default_value=7497)
                assert value == 7497
        finally:
            os.unlink(temp_path)

    def test_get_config_value_path_with_leading_dot(self):
        """Test get_config_value() handles path with leading dot."""
        config_data = {"tws": {"port": 7497}}

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch('integration.config_loader._find_config_file', return_value=Path(temp_path)):
                value = get_config_value(".tws.port")
                assert value == 7497
        finally:
            os.unlink(temp_path)

    def test_get_config_value_none_value(self):
        """Test get_config_value() returns default when value is None."""
        config_data = {"tws": {"port": None}}

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = f.name

        try:
            with patch('integration.config_loader._find_config_file', return_value=Path(temp_path)):
                value = get_config_value("tws.port", default_value=7497)
                assert value == 7497
        finally:
            os.unlink(temp_path)


if __name__ == "__main__":
    unittest.main()
