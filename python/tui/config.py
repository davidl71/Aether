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
from dataclasses import dataclass, asdict, field
from pathlib import Path
from typing import Optional, Dict

from ..integration.shared_config_loader import SharedConfigLoader

logger = logging.getLogger(__name__)

# Default ports when no shared config (so we still poll and show status for common backends)
DEFAULT_BACKEND_PORTS: Dict[str, int] = {
    "ib": 8002,
    "alpaca": 8000,
    "tastytrade": 8003,
}


@dataclass
class TUIConfig:
    """
    TUI configuration matching C++ TUIConfig structure

    MIGRATION NOTE: This can be serialized to JSON and loaded in C++
    using nlohmann/json
    """

    provider_type: str = (
        "rest"  # "mock", "rest", "file", "ibkr_rest", "livevol", "nautilus"
    )
    rest_endpoint: str = "http://localhost:8002/api/v1/snapshot"  # IB service port (changed from 8080 Rust backend to 8002 IB)
    update_interval_ms: int = 1000
    refresh_rate_ms: int = 500
    rest_timeout_ms: int = 15000
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

    # Backend service ports for health checks (name -> port), e.g. {"ib": 8002, "alpaca": 8000}
    backend_ports: Dict[str, int] = field(default_factory=dict)

    # Backends disabled due to missing/placeholder credentials (name -> reason)
    disabled_backends: Dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization"""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict) -> TUIConfig:
        """Create from dictionary (ignores unknown keys for backward compatibility)."""
        allowed = {f for f in cls.__dataclass_fields__}
        return cls(**{k: v for k, v in data.items() if k in allowed})

    def save_to_file(self, file_path: str) -> None:
        """Save configuration to JSON file"""
        config_dir = Path(file_path).parent
        config_dir.mkdir(parents=True, exist_ok=True)

        with open(file_path, "w") as f:
            json.dump(self.to_dict(), f, indent=2)
        logger.info(f"Configuration saved to {file_path}")

    @classmethod
    def load_from_file(cls, file_path: str) -> TUIConfig:
        """Load configuration from JSON file"""
        if not Path(file_path).exists():
            logger.warning(f"Config file not found: {file_path}, using defaults")
            return cls()

        try:
            with open(file_path, "r") as f:
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
        shared_config = SharedConfigLoader.load_config(quiet_placeholder_warnings=True)
        if shared_config.tui:
            # Convert shared config TUI section (dataclass) to TUI TUIConfig
            shared_tui = (
                shared_config.tui
            )  # This is a TUIConfig dataclass from shared_config_loader

            # Extract ibkr_rest dict fields (shared config uses Dict[str, Any])
            ibkr_rest_dict = (
                shared_tui.ibkr_rest
                if hasattr(shared_tui, "ibkr_rest")
                and isinstance(shared_tui.ibkr_rest, dict)
                else {}
            )
            display_dict = (
                shared_tui.display
                if hasattr(shared_tui, "display")
                and isinstance(shared_tui.display, dict)
                else {}
            )
            show_colors = display_dict.get("showColors", display_dict.get("show_colors", True))
            show_footer = display_dict.get("showFooter", display_dict.get("show_footer", True))

            config = TUIConfig(
                provider_type=shared_tui.provider_type,
                rest_endpoint=shared_tui.rest_endpoint,
                update_interval_ms=shared_tui.update_interval_ms,
                refresh_rate_ms=shared_tui.refresh_rate_ms,
                rest_timeout_ms=shared_tui.rest_timeout_ms,
                rest_verify_ssl=shared_tui.rest_verify_ssl,
                file_path=shared_tui.file_path,
                ibkr_rest_base_url=ibkr_rest_dict.get(
                    "baseUrl", ibkr_rest_dict.get("base_url", "https://localhost:5000/v1/portal")
                ),
                ibkr_rest_account_id=ibkr_rest_dict.get("accountId", ibkr_rest_dict.get("account_id", "")),
                ibkr_rest_verify_ssl=ibkr_rest_dict.get("verifySsl", ibkr_rest_dict.get("verify_ssl", False)),
                ibkr_rest_timeout_ms=ibkr_rest_dict.get("timeoutMs", ibkr_rest_dict.get("timeout_ms", 5000)),
                show_colors=show_colors,
                show_footer=show_footer,
            )
            from_services = _backend_ports_from_services(shared_config.services)
            config.backend_ports = {**DEFAULT_BACKEND_PORTS, **from_services}
            config.disabled_backends = _disabled_backends_from_env(shared_config.services)
            logger.info("Loaded TUI configuration from shared config file")
            # Apply environment variable overrides
            _apply_env_overrides(config)
            return config
    except Exception as e:
        logger.debug(
            f"Shared config not available ({e}), falling back to legacy config"
        )

    # Fallback: no shared config — use default backend ports and env-based disabled backends
    config_path = TUIConfig.get_config_path()
    if not Path(config_path).exists():
        # Auto-initialize: write default TUI config so it exists for future runs
        default_config = TUIConfig()
        default_config.backend_ports = dict(DEFAULT_BACKEND_PORTS)
        default_config.disabled_backends = _disabled_backends_from_env(
            {"alpaca": {}, "tastytrade": {}}
        )
        default_config.save_to_file(config_path)
        logger.info(f"Initialized TUI config at {config_path}")
        config = default_config
    else:
        config = TUIConfig.load_from_file(config_path)
        # Ensure we poll common backends so status line can show them (or "disabled")
        config.backend_ports = {**DEFAULT_BACKEND_PORTS, **(config.backend_ports or {})}
        config.disabled_backends = _disabled_backends_from_env(
            {"alpaca": {}, "tastytrade": {}}
        )

    # Override with environment variables
    _apply_env_overrides(config)

    return config


def _backend_ports_from_services(services: dict) -> Dict[str, int]:
    """Extract backend name -> port from config services for health checks."""
    out: Dict[str, int] = {}
    # Services that expose /api/health on a single port
    for name in ("ib", "alpaca", "tradestation", "discount_bank", "risk_free_rate", "tastytrade"):
        svc = services.get(name)
        if isinstance(svc, dict):
            port = svc.get("port")
            if isinstance(port, int) and 0 < port < 65536:
                out[name] = port
    return out


def _is_placeholder_or_empty(value: Optional[str]) -> bool:
    """True if value is missing, empty, or a placeholder (e.g. ${VAR} or 'placeholder')."""
    if value is None or not value.strip():
        return True
    v = value.strip().lower()
    if v in ("placeholder", "missing", "optional"):
        return True
    if v.startswith("${") and v.endswith("}"):
        return True
    return False


def _disabled_backends_from_env(services: dict) -> Dict[str, str]:
    """
    For backends present in services, detect missing/placeholder credentials
    and return backend name -> reason (e.g. "Missing API key").
    """
    out: Dict[str, str] = {}
    if not isinstance(services, dict):
        return out

    # Alpaca: API keys or OAuth
    if services.get("alpaca") is not None:
        has_oauth = not _is_placeholder_or_empty(os.getenv("ALPACA_CLIENT_ID")) and not _is_placeholder_or_empty(
            os.getenv("ALPACA_CLIENT_SECRET")
        )
        has_api_key = not _is_placeholder_or_empty(os.getenv("ALPACA_API_KEY_ID")) and not _is_placeholder_or_empty(
            os.getenv("ALPACA_API_SECRET_KEY")
        )
        if not has_oauth and not has_api_key:
            out["alpaca"] = "Missing API key or OAuth credentials"

    # Tastytrade: session or OAuth
    if services.get("tastytrade") is not None:
        has_oauth = not _is_placeholder_or_empty(os.getenv("TASTYTRADE_CLIENT_SECRET")) and not _is_placeholder_or_empty(
            os.getenv("TASTYTRADE_REFRESH_TOKEN")
        )
        has_session = not _is_placeholder_or_empty(os.getenv("TASTYTRADE_USERNAME")) and not _is_placeholder_or_empty(
            os.getenv("TASTYTRADE_PASSWORD")
        )
        if not has_oauth and not has_session:
            out["tastytrade"] = "Missing credentials"

    return out


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
