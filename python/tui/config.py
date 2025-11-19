"""
Configuration management for Python TUI

MIGRATION NOTES FOR FUTURE C++ MIGRATION (pybind11):
- Configuration can be shared with C++ via JSON serialization
- Consider using a shared config format (JSON/YAML) between Python and C++
- C++ config loading can be exposed via pybind11
"""

from __future__ import annotations

import json
import logging
import os
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)


@dataclass
class TUIConfig:
    """
    TUI configuration matching C++ TUIConfig structure

    MIGRATION NOTE: This can be serialized to JSON and loaded in C++
    using nlohmann/json
    """
    provider_type: str = "mock"  # "mock", "rest", "file", "ibkr_rest", "livevol", "nautilus"
    rest_endpoint: str = "http://localhost:8080/api/snapshot"
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
    Load configuration from file or environment variables

    Environment variables override config file:
    - TUI_BACKEND: provider type (mock, rest, file, etc.)
    - TUI_API_URL: REST endpoint URL
    - TUI_SNAPSHOT_FILE: file path for file provider
    """
    config_path = TUIConfig.get_config_path()
    config = TUIConfig.load_from_file(config_path)

    # Override with environment variables
    if os.getenv("TUI_BACKEND"):
        config.provider_type = os.getenv("TUI_BACKEND", "mock")

    if os.getenv("TUI_API_URL"):
        config.rest_endpoint = os.getenv("TUI_API_URL")

    if os.getenv("TUI_SNAPSHOT_FILE"):
        config.file_path = os.getenv("TUI_SNAPSHOT_FILE")

    return config
