"""
Tests for environment configuration management.

Tests EnvironmentConfig class, configuration loading, and global config functions.
"""
import unittest
import json
import os
import tempfile
from pathlib import Path
from unittest.mock import patch

import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

from services.environment_config import (
    EnvironmentConfig,
    get_config,
    reload_config,
)


class TestEnvironmentConfig(unittest.TestCase):
    """Tests for EnvironmentConfig class."""

    def test_init_with_default_path(self):
        """Test EnvironmentConfig initialization with default path."""
        config = EnvironmentConfig()
        assert config.config_file is not None
        assert isinstance(config.config_file, Path)

    def test_init_with_custom_path(self):
        """Test EnvironmentConfig initialization with custom path."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write('{"test": "value"}')
            temp_path = Path(f.name)

        try:
            config = EnvironmentConfig(config_file=temp_path)
            assert config.config_file == temp_path
        finally:
            temp_path.unlink()

    def test_load_config_file_exists(self):
        """Test loading configuration from existing file."""
        config_data = {
            "security": {
                "rate_limit_per_minute": 120,
                "rate_limit_per_second": 20
            },
            "test_key": "test_value"
        }

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = Path(f.name)

        try:
            config = EnvironmentConfig(config_file=temp_path)
            assert config.get("test_key") == "test_value"
            assert config.get("security.rate_limit_per_minute") == 120
        finally:
            temp_path.unlink()

    def test_load_config_file_not_exists(self):
        """Test loading configuration when file doesn't exist."""
        temp_path = Path("/tmp/nonexistent_config.json")
        config = EnvironmentConfig(config_file=temp_path)
        # Should not raise error, just use empty config
        assert config.get("any_key", "default") == "default"

    def test_load_config_invalid_json(self):
        """Test loading configuration with invalid JSON."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write('{invalid json}')
            temp_path = Path(f.name)

        try:
            config = EnvironmentConfig(config_file=temp_path)
            # Should not raise error, just use empty config
            assert config.get("any_key", "default") == "default"
        finally:
            temp_path.unlink()

    def test_get_with_default(self):
        """Test get() method with default value."""
        config = EnvironmentConfig()
        assert config.get("nonexistent_key", "default_value") == "default_value"

    def test_get_with_nested_key(self):
        """Test get() method with nested key (dot notation)."""
        config_data = {
            "security": {
                "rate_limit_per_minute": 120
            }
        }

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = Path(f.name)

        try:
            config = EnvironmentConfig(config_file=temp_path)
            assert config.get("security.rate_limit_per_minute") == 120
        finally:
            temp_path.unlink()

    def test_get_with_env_var_priority(self):
        """Test that environment variables take priority over config file."""
        config_data = {"test_key": "file_value"}

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = Path(f.name)

        try:
            with patch.dict(os.environ, {"TEST_ENV_VAR": "env_value"}):
                config = EnvironmentConfig(config_file=temp_path)
                # Environment variable should take priority
                value = config.get("test_key", "default", env_var="TEST_ENV_VAR")
                assert value == "env_value"
        finally:
            temp_path.unlink()

    def test_get_security_config(self):
        """Test get_security_config() method."""
        config_data = {
            "security": {
                "rate_limit_per_minute": 120,
                "rate_limit_per_second": 20,
                "api_key": "test-key",
                "require_auth": True
            }
        }

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = Path(f.name)

        try:
            config = EnvironmentConfig(config_file=temp_path)
            security_config = config.get_security_config()

            assert security_config["rate_limit_per_minute"] == 120
            assert security_config["rate_limit_per_second"] == 20
            assert security_config["api_key"] == "test-key"
            assert security_config["require_auth"] is True
        finally:
            temp_path.unlink()

    def test_get_security_config_defaults(self):
        """Test get_security_config() with defaults."""
        config = EnvironmentConfig()
        security_config = config.get_security_config()

        assert security_config["rate_limit_per_minute"] == 60
        assert security_config["rate_limit_per_second"] == 10
        assert security_config["api_key"] is None
        assert security_config["require_auth"] is False

    def test_get_service_port(self):
        """Test get_service_port() method."""
        config_data = {
            "services": {
                "swiftness_api_port": 8081
            }
        }

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_path = Path(f.name)

        try:
            config = EnvironmentConfig(config_file=temp_path)
            port = config.get_service_port("swiftness_api", 8080)
            assert port == 8081
        finally:
            temp_path.unlink()

    def test_get_service_port_default(self):
        """Test get_service_port() with default."""
        config = EnvironmentConfig()
        port = config.get_service_port("nonexistent_service", 9000)
        assert port == 9000

    def test_reload(self):
        """Test reload() method."""
        config_data1 = {"test_key": "value1"}
        config_data2 = {"test_key": "value2"}

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data1, f)
            temp_path = Path(f.name)

        try:
            config = EnvironmentConfig(config_file=temp_path)
            assert config.get("test_key") == "value1"

            # Update file
            with open(temp_path, 'w') as f:
                json.dump(config_data2, f)

            # Reload and verify new value
            config.reload()
            assert config.get("test_key") == "value2"
        finally:
            temp_path.unlink()

    def test_convert_type_bool(self):
        """Test type conversion for boolean values."""
        config = EnvironmentConfig()

        # Test boolean conversion
        assert config._convert_type("true", False) is True
        assert config._convert_type("false", False) is False
        assert config._convert_type("1", False) is True
        assert config._convert_type("0", False) is False

    def test_convert_type_int(self):
        """Test type conversion for integer values."""
        config = EnvironmentConfig()

        assert config._convert_type("123", 0) == 123
        assert config._convert_type("456", 0) == 456

    def test_convert_type_float(self):
        """Test type conversion for float values."""
        config = EnvironmentConfig()

        assert config._convert_type("123.45", 0.0) == 123.45
        assert config._convert_type("456.78", 0.0) == 456.78

    def test_convert_type_list(self):
        """Test type conversion for list values."""
        config = EnvironmentConfig()

        result = config._convert_type("a, b, c", [])
        assert result == ["a", "b", "c"]


class TestGlobalConfigFunctions(unittest.TestCase):
    """Tests for global config functions."""

    def test_get_config_returns_singleton(self):
        """Test that get_config() returns the same instance."""
        config1 = get_config()
        config2 = get_config()
        assert config1 is config2

    def test_reload_config(self):
        """Test reload_config() function."""
        config = get_config()

        # Should not raise error
        reload_config()

        # Verify config still works
        assert config is not None


if __name__ == "__main__":
    unittest.main()
