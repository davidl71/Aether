"""Tests for python/tui/config.py - TUI configuration."""

import json
import os
from pathlib import Path
from unittest.mock import patch, MagicMock

import pytest

from python.tui.config import TUIConfig, _apply_env_overrides


# ---------------------------------------------------------------------------
# TUIConfig - defaults
# ---------------------------------------------------------------------------


class TestTUIConfigDefaults:
    def test_default_provider(self):
        cfg = TUIConfig()
        assert cfg.provider_type == "mock"

    def test_default_endpoint(self):
        cfg = TUIConfig()
        assert "localhost" in cfg.rest_endpoint

    def test_default_interval(self):
        cfg = TUIConfig()
        assert cfg.update_interval_ms == 1000

    def test_default_colors(self):
        cfg = TUIConfig()
        assert cfg.show_colors is True

    def test_default_ibkr_rest(self):
        cfg = TUIConfig()
        assert "5000" in cfg.ibkr_rest_base_url


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
        original = TUIConfig(provider_type="rest", rest_endpoint="http://example.com")
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
    def test_get_config_path(self):
        path = TUIConfig.get_config_path()
        assert "tui_config.json" in path

    def test_load_default(self):
        cfg = TUIConfig.load_default()
        assert cfg.provider_type == "mock"


# ---------------------------------------------------------------------------
# Environment overrides
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

    def test_snapshot_file_override(self, monkeypatch):
        monkeypatch.setenv("TUI_SNAPSHOT_FILE", "/data/snap.json")
        cfg = TUIConfig()
        _apply_env_overrides(cfg)
        assert cfg.file_path == "/data/snap.json"

    def test_no_override_when_unset(self):
        cfg = TUIConfig(provider_type="mock")
        _apply_env_overrides(cfg)
        assert cfg.provider_type == "mock"
