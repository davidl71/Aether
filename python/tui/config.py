"""
Configuration management for Python TUI

MIGRATION NOTES FOR FUTURE C++ MIGRATION (pybind11):
- Configuration can be shared with C++ via JSON serialization
- Consider using a shared config format (JSON/YAML) between Python and C++
- C++ config loading can be exposed via pybind11

Uses SharedConfigLoader for unified configuration format (see T-112).
"""

from __future__ import annotations

import json
import logging
import os
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional

from ..integration.shared_config_loader import SharedConfigLoader

logger = logging.getLogger(__name__)


@dataclass
class TUIConfig:
    """
    TUI configuration matching C++ TUIConfig structure

    MIGRATION NOTE: This can be serialized to JSON and loaded in C++
    using nlohmann/json
    """
    provider_type: str = "mock"  # "mock", "rest", "file", "ibkr_rest", "livevol", "nautilus"
    rest_endpoint: str = "http://localhost:8080/api/v1/snapshot"
    update_interval_ms: int = 1000
    refresh_rate_ms: int = 500
    rest_timeout_ms: int = 5000
    rest_verify_ssl: bool = False
    file_path: Optional[str] = None

    # IBKR REST API settings
    ibkr_rest_base_url: str = "https://localhost:5000/v1/portal"
    ibkr_rest_account_id: str = ""
    ibkr_rest_verify_ssl: bool = False
    ibkr_rest_timeout_ms: int = 5000

    # Display settings
    show_colors: bool = True
    show_footer: bool = True

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization"""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict) -> TUIConfig:
        """Create from dictionary"""
        return cls(**data)

    def save_to_file(self, file_path: str) -> None:
        """Save configuration to JSON file"""
        config_dir = Path(file_path).parent
        config_dir.mkdir(parents=True, exist_ok=True)

        with open(file_path, 'w') as f:
            json.dump(self.to_dict(), f, indent=2)
        logger.info(f"Configuration saved to {file_path}")

    @classmethod
    def load_from_file(cls, file_path: str) -> TUIConfig:
        """Load configuration from JSON file"""
        if not Path(file_path).exists():
            logger.warning(f"Config file not found: {file_path}, using defaults")
            return cls()

        try:
            with open(file_path, 'r') as f:
                data = json.load(f)
            return cls.from_dict(data)
        except Exception as e:
            logger.error(f"Failed to load config: {e}, using defaults")
            return cls()

    @classmethod
    def get_config_path(cls) -> str:
        """Get default configuration file path"""
        home = os.getenv("HOME")
        if home:
            config_dir = Path(home) / ".config" / "ib_box_spread"
            config_dir.mkdir(parents=True, exist_ok=True)
            return str(config_dir / "tui_config.json")
        return "tui_config.json"

    @classmethod
    def load_default(cls) -> TUIConfig:
        """Load default configuration"""
        return cls()


def load_config() -> TUIConfig:
    """
    Load configuration from shared config file or fallback to legacy config.

    Uses SharedConfigLoader (T-112) for unified configuration format.
    Falls back to legacy TUI config file if shared config not available.

    Environment variables override config file:
    - TUI_BACKEND: provider type (mock, rest, file, etc.)
    - TUI_API_URL: REST endpoint URL
    - TUI_SNAPSHOT_FILE: file path for file provider
    - IB_BOX_SPREAD_CONFIG: path to shared config file
    """
    # Try shared config first
    try:
        shared_config = SharedConfigLoader.load_config()
        if shared_config.tui:
            # Convert shared config TUI section (dataclass) to TUI TUIConfig
            shared_tui = shared_config.tui  # This is a TUIConfig dataclass from shared_config_loader

            # Extract ibkr_rest dict fields (shared config uses Dict[str, Any])
            ibkr_rest_dict = shared_tui.ibkr_rest if hasattr(shared_tui, 'ibkr_rest') and isinstance(shared_tui.ibkr_rest, dict) else {}
            display_dict = shared_tui.display if hasattr(shared_tui, 'display') and isinstance(shared_tui.display, dict) else {}

            config = TUIConfig(
                provider_type=shared_tui.provider_type,
                rest_endpoint=shared_tui.rest_endpoint,
                update_interval_ms=shared_tui.update_interval_ms,
                refresh_rate_ms=shared_tui.refresh_rate_ms,
                rest_timeout_ms=shared_tui.rest_timeout_ms,
                rest_verify_ssl=shared_tui.rest_verify_ssl,
                file_path=shared_tui.file_path,
                ibkr_rest_base_url=ibkr_rest_dict.get("baseUrl", "https://localhost:5000/v1/portal"),
                ibkr_rest_account_id=ibkr_rest_dict.get("accountId", ""),
                ibkr_rest_verify_ssl=ibkr_rest_dict.get("verifySsl", False),
                ibkr_rest_timeout_ms=ibkr_rest_dict.get("timeoutMs", 5000),
                show_colors=display_dict.get("showColors", True),
                show_footer=display_dict.get("showFooter", True),
            )
            logger.info("Loaded TUI configuration from shared config file")
            # Apply environment variable overrides
            _apply_env_overrides(config)
            return config
    except Exception as e:
        logger.debug(f"Shared config not available ({e}), falling back to legacy config")

    # Fallback to legacy config file
    config_path = TUIConfig.get_config_path()
    config = TUIConfig.load_from_file(config_path)

    # Override with environment variables
    _apply_env_overrides(config)

    return config


def _apply_env_overrides(config: TUIConfig) -> None:
    """Apply environment variable overrides to config."""
    if os.getenv("TUI_BACKEND"):
        config.provider_type = os.getenv("TUI_BACKEND", "mock")

    env_api_url = os.getenv("TUI_API_URL")
    if env_api_url:
        config.rest_endpoint = env_api_url

    env_file = os.getenv("TUI_SNAPSHOT_FILE")
    if env_file:
        config.file_path = env_file
