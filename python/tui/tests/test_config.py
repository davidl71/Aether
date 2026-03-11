"""
Tests for TUI configuration (python/tui/config.py).

Consolidated from test_tui_config.py and test_tui_config_unit.py.
Covers TUIConfig, load_config(), and _apply_env_overrides().
"""

import json
import os
from pathlib import Path
from unittest.mock import patch


from .. import config as config_module
from ..config import (
    TUIConfig,
    load_config,
    _apply_env_overrides,
    DEFAULT_BACKEND_PORTS,
    DEFAULT_TCP_BACKEND_PORTS,
    PRESET_REST_ENDPOINTS,
    DEFAULT_SHARED_API_BASE_URL,
    snapshot_endpoint_from_base,
)


# Rest endpoint default in TUIConfig (shared Rust origin)
DEFAULT_REST_ENDPOINT = "http://localhost:8080/api/v1/snapshot"


def test_default_backend_ports_includes_all_services():
    """DEFAULT_BACKEND_PORTS should include the active backend services."""
    assert DEFAULT_BACKEND_PORTS.get("discount_bank") == 8003
    assert DEFAULT_BACKEND_PORTS.get("rust") == 8080


def test_preset_rest_endpoints():
    """Rust owns both the shared snapshot path and the IB specialist proxy path."""
    assert "rest_ib" in PRESET_REST_ENDPOINTS
    assert "rest_rust" in PRESET_REST_ENDPOINTS
    assert PRESET_REST_ENDPOINTS["rest_rust"] == DEFAULT_REST_ENDPOINT
    assert "8080" in PRESET_REST_ENDPOINTS["rest_ib"]
    assert "/api/v1/ib/" in PRESET_REST_ENDPOINTS["rest_ib"]


def test_default_tcp_backend_ports_includes_tws():
    """DEFAULT_TCP_BACKEND_PORTS should include TWS/Gateway paper port."""
    assert DEFAULT_TCP_BACKEND_PORTS.get("tws") == 7497


# ---------------------------------------------------------------------------
# TUIConfig - defaults
# ---------------------------------------------------------------------------


class TestTUIConfigDefaults:
    def test_default_provider(self):
        cfg = TUIConfig()
        assert cfg.provider_type == "mock"

    def test_default_endpoint(self):
        cfg = TUIConfig()
        assert cfg.api_base_url is None
        assert cfg.rest_endpoint == ""

    def test_default_interval(self):
        cfg = TUIConfig()
        assert cfg.update_interval_ms == 1000

    def test_default_colors(self):
        cfg = TUIConfig()
        assert cfg.show_colors is True

    def test_default_ibkr_rest(self):
        cfg = TUIConfig()
        assert "5001" in cfg.ibkr_rest_base_url

    def test_custom_values(self):
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

    def test_default_api_base_roundtrips_to_snapshot(self):
        cfg = TUIConfig()
        assert snapshot_endpoint_from_base(cfg.api_base_url) == DEFAULT_REST_ENDPOINT


# ---------------------------------------------------------------------------
# TUIConfig - serialization
# ---------------------------------------------------------------------------


class TestTUIConfigSerialization:
    def test_to_dict(self):
        cfg = TUIConfig(provider_type="rest", update_interval_ms=2000)
        d = cfg.to_dict()
        assert d["provider_type"] == "rest"
        assert d["update_interval_ms"] == 2000

    def test_from_dict(self):
        d = {"provider_type": "file", "file_path": "/tmp/test.json"}
        cfg = TUIConfig.from_dict(d)
        assert cfg.provider_type == "file"
        assert cfg.file_path == "/tmp/test.json"

    def test_roundtrip(self):
        original = TUIConfig(
            provider_type="rest", rest_endpoint="http://example.com"
        )
        d = original.to_dict()
        restored = TUIConfig.from_dict(d)
        assert restored.provider_type == original.provider_type
        assert restored.rest_endpoint == original.rest_endpoint


# ---------------------------------------------------------------------------
# TUIConfig - file I/O
# ---------------------------------------------------------------------------


class TestTUIConfigFileIO:
    def test_save_and_load(self, tmp_path):
        fp = str(tmp_path / "config.json")
        cfg = TUIConfig(provider_type="ibkr_rest", update_interval_ms=500)
        cfg.save_to_file(fp)

        loaded = TUIConfig.load_from_file(fp)
        assert loaded.provider_type == "ibkr_rest"
        assert loaded.update_interval_ms == 500

    def test_load_missing(self, tmp_path):
        cfg = TUIConfig.load_from_file(str(tmp_path / "nope.json"))
        assert cfg.provider_type == "mock"

    def test_load_invalid(self, tmp_path):
        fp = tmp_path / "bad.json"
        fp.write_text("not json!")
        cfg = TUIConfig.load_from_file(str(fp))
        assert cfg.provider_type == "mock"

    def test_creates_parent_dirs(self, tmp_path):
        fp = str(tmp_path / "sub" / "dir" / "config.json")
        cfg = TUIConfig()
        cfg.save_to_file(fp)
        assert Path(fp).exists()


# ---------------------------------------------------------------------------
# TUIConfig - config path
# ---------------------------------------------------------------------------


class TestConfigPath:
    def test_get_config_path(self, tmp_path):
        """Path contains tui_config.json; use tmp_path to avoid creating real dirs."""
        with patch.dict(os.environ, {"HOME": str(tmp_path)}):
            path = TUIConfig.get_config_path()
        assert "tui_config.json" in path

    def test_get_config_path_with_home(self, tmp_path):
        with patch.dict(os.environ, {"HOME": str(tmp_path)}):
            config_path = TUIConfig.get_config_path()
            expected = tmp_path / ".config" / "ib_box_spread" / "tui_config.json"
            assert config_path == str(expected)
            assert expected.parent.exists()

    def test_get_config_path_no_home(self):
        with patch.dict(os.environ, {}, clear=True):
            if "HOME" in os.environ:
                del os.environ["HOME"]
            config_path = TUIConfig.get_config_path()
            assert config_path == "tui_config.json"

    def test_load_default(self):
        cfg = TUIConfig.load_default()
        assert cfg.provider_type == "mock"


# ---------------------------------------------------------------------------
# Environment overrides (_apply_env_overrides)
# ---------------------------------------------------------------------------


class TestEnvOverrides:
    def test_backend_override(self, monkeypatch):
        monkeypatch.setenv("TUI_BACKEND", "rest")
        cfg = TUIConfig()
        _apply_env_overrides(cfg)
        assert cfg.provider_type == "rest"

    def test_api_url_override(self, monkeypatch):
        monkeypatch.setenv("TUI_API_URL", "http://custom:9090")
        cfg = TUIConfig()
        _apply_env_overrides(cfg)
        assert cfg.rest_endpoint == "http://custom:9090"
        assert cfg.api_base_url == "http://custom:9090"

    def test_api_base_override(self, monkeypatch):
        monkeypatch.setenv("TUI_API_BASE_URL", "http://custom:9999")
        cfg = TUIConfig()
        _apply_env_overrides(cfg)
        assert cfg.api_base_url == "http://custom:9999"

    def test_snapshot_file_override(self, monkeypatch):
        monkeypatch.setenv("TUI_SNAPSHOT_FILE", "/data/snap.json")
        cfg = TUIConfig()
        _apply_env_overrides(cfg)
        assert cfg.file_path == "/data/snap.json"

    def test_no_override_when_unset(self):
        cfg = TUIConfig(provider_type="mock")
        _apply_env_overrides(cfg)
        assert cfg.provider_type == "mock"


# ---------------------------------------------------------------------------
# load_config()
# ---------------------------------------------------------------------------


class TestLoadConfig:
    def test_load_config_from_file(self, tmp_path):
        config_data = {
            "provider_type": "rest",
            "rest_endpoint": "https://api.example.com",
            "update_interval_ms": 2000,
        }
        config_path = tmp_path / "config.json"
        config_path.write_text(json.dumps(config_data))

        with patch.object(
            config_module.SharedConfigLoader,
            "load_config",
            side_effect=Exception("no shared config"),
        ):
            with patch.object(
                config_module.TUIConfig,
                "get_config_path",
                return_value=str(config_path),
            ):
                config = load_config()

        assert config.provider_type == "rest"
        assert config.rest_endpoint == "https://api.example.com"
        assert config.update_interval_ms == 2000
        assert config.api_base_url == "https://api.example.com"

    def test_load_config_env_override_provider_type(self, tmp_path):
        config_path = tmp_path / "config.json"
        config_path.write_text(json.dumps({"provider_type": "file"}))

        with patch.object(
            config_module.SharedConfigLoader,
            "load_config",
            side_effect=Exception("no shared config"),
        ):
            with patch.object(
                config_module.TUIConfig,
                "get_config_path",
                return_value=str(config_path),
            ):
                with patch.dict(os.environ, {"TUI_BACKEND": "rest"}):
                    config = load_config()

        assert config.provider_type == "rest"

    def test_load_config_env_override_rest_endpoint(self, tmp_path):
        config_path = tmp_path / "config.json"
        config_path.write_text(json.dumps({"rest_endpoint": "http://file.example.com"}))

        with patch.object(
            config_module.SharedConfigLoader,
            "load_config",
            side_effect=Exception("no shared config"),
        ):
            with patch.object(
                config_module.TUIConfig,
                "get_config_path",
                return_value=str(config_path),
            ):
                with patch.dict(os.environ, {"TUI_API_URL": "https://env.example.com"}):
                    config = load_config()

        assert config.rest_endpoint == "https://env.example.com"
        assert config.api_base_url == "https://env.example.com"

    def test_load_config_env_override_file_path(self, tmp_path):
        config_path = tmp_path / "config.json"
        config_path.write_text(json.dumps({"file_path": "/file/path.json"}))

        with patch.object(
            config_module.SharedConfigLoader,
            "load_config",
            side_effect=Exception("no shared config"),
        ):
            with patch.object(
                config_module.TUIConfig,
                "get_config_path",
                return_value=str(config_path),
            ):
                with patch.dict(os.environ, {"TUI_SNAPSHOT_FILE": "/env/path.json"}):
                    config = load_config()

        assert config.file_path == "/env/path.json"

    def test_load_config_no_file_uses_defaults(self, tmp_path):
        """When no config exists, load_config returns defaults and persists to get_config_path()."""
        config_path = str(tmp_path / "config.json")
        with patch.object(
            config_module.SharedConfigLoader,
            "load_config",
            side_effect=Exception("no shared config"),
        ):
            with patch.object(
                config_module.TUIConfig,
                "get_config_path",
                return_value=config_path,
            ):
                config = load_config()

        assert config.provider_type == "mock"
        assert config.api_base_url == DEFAULT_SHARED_API_BASE_URL
