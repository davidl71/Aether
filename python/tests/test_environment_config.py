"""Tests for python/services/environment_config.py."""

import json

import pytest

from python.services.environment_config import EnvironmentConfig


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture
def config_file(tmp_path):
    """Create a temporary environment.json config file."""
    cfg = {
        "services": {"calculations_port": 8081, "snapshot_port": 8082},
        "security": {
            "rate_limit_per_minute": 120,
            "rate_limit_per_second": 20,
            "api_key": "test-key-123",
            "require_auth": True,
            "allowed_origins": ["http://localhost:4000"],
        },
        "nested": {"deep": {"value": 42}},
    }
    fp = tmp_path / "environment.json"
    fp.write_text(json.dumps(cfg))
    return fp


@pytest.fixture
def empty_config(tmp_path):
    return tmp_path / "missing.json"


# ---------------------------------------------------------------------------
# Initialization
# ---------------------------------------------------------------------------


class TestInit:
    def test_load_existing_file(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec.get("services.calculations_port") == 8081

    def test_load_missing_file(self, empty_config):
        ec = EnvironmentConfig(empty_config)
        assert ec.get("nonexistent", "fallback") == "fallback"

    def test_invalid_json(self, tmp_path):
        fp = tmp_path / "bad.json"
        fp.write_text("not json")
        ec = EnvironmentConfig(fp)
        assert ec.get("key", "default") == "default"


# ---------------------------------------------------------------------------
# get()
# ---------------------------------------------------------------------------


class TestGet:
    def test_simple_key(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec.get("services.snapshot_port") == 8082

    def test_nested_key(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec.get("nested.deep.value") == 42

    def test_default_fallback(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec.get("nonexistent.key", "default") == "default"

    def test_none_default(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec.get("nonexistent") is None

    def test_env_var_override(self, config_file, monkeypatch):
        monkeypatch.setenv("MY_PORT", "9999")
        ec = EnvironmentConfig(config_file)
        val = ec.get("services.calculations_port", default=8081, env_var="MY_PORT")
        assert val == 9999

    def test_env_var_not_set(self, config_file):
        ec = EnvironmentConfig(config_file)
        val = ec.get("services.calculations_port", default=8081, env_var="UNLIKELY_VAR_42")
        assert val == 8081


# ---------------------------------------------------------------------------
# _convert_type
# ---------------------------------------------------------------------------


class TestConvertType:
    def test_bool_true(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec._convert_type("true", None) is True

    def test_bool_false(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec._convert_type("false", None) is False

    def test_int(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec._convert_type("42", None) == 42

    def test_float(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec._convert_type("3.14", None) == pytest.approx(3.14)

    def test_string(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec._convert_type("hello", None) == "hello"

    def test_bool_default(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec._convert_type("yes", False) is True
        assert ec._convert_type("no", False) is False

    def test_int_default(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec._convert_type("100", 0) == 100
        assert ec._convert_type("not_int", 5) == 5

    def test_float_default(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec._convert_type("2.5", 0.0) == pytest.approx(2.5)

    def test_list_default(self, config_file):
        ec = EnvironmentConfig(config_file)
        result = ec._convert_type("a, b, c", [])
        assert result == ["a", "b", "c"]


# ---------------------------------------------------------------------------
# get_security_config
# ---------------------------------------------------------------------------


class TestSecurityConfig:
    def test_from_file(self, config_file):
        ec = EnvironmentConfig(config_file)
        sec = ec.get_security_config()
        assert sec["rate_limit_per_minute"] == 120
        assert sec["api_key"] == "test-key-123"
        assert sec["require_auth"] is True

    def test_defaults(self, empty_config):
        ec = EnvironmentConfig(empty_config)
        sec = ec.get_security_config()
        assert sec["rate_limit_per_minute"] == 60
        assert sec["require_auth"] is False


# ---------------------------------------------------------------------------
# get_service_port
# ---------------------------------------------------------------------------


class TestServicePort:
    def test_from_file(self, config_file):
        ec = EnvironmentConfig(config_file)
        assert ec.get_service_port("calculations", default=8080) == 8081

    def test_default(self, empty_config):
        ec = EnvironmentConfig(empty_config)
        assert ec.get_service_port("calculations", default=8080) == 8080

    def test_env_override(self, config_file, monkeypatch):
        monkeypatch.setenv("CALCULATIONS_PORT", "7777")
        ec = EnvironmentConfig(config_file)
        assert ec.get_service_port("calculations", default=8080) == 7777


# ---------------------------------------------------------------------------
# reload
# ---------------------------------------------------------------------------


class TestReload:
    def test_reload(self, tmp_path):
        fp = tmp_path / "env.json"
        fp.write_text(json.dumps({"key": "old"}))
        ec = EnvironmentConfig(fp)
        assert ec.get("key") == "old"
        fp.write_text(json.dumps({"key": "new"}))
        ec.reload()
        assert ec.get("key") == "new"
