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
from typing import Optional, Dict, List

from ..integration.shared_config_loader import SharedConfigLoader

# Prefer in-package helper; fallback for different run contexts (e.g. python -m tui.app from python/)
try:
    from ..integration.onepassword_sdk_helper import getenv_or_resolve
except ImportError:
    try:
        from integration.onepassword_sdk_helper import getenv_or_resolve
    except ImportError:
        def getenv_or_resolve(env_var: str, op_ref_env_var: str, default: str = "") -> str:
            # Stub when 1Password helper unavailable: only use env_var (never resolve op://)
            return os.getenv(env_var, default)

logger = logging.getLogger(__name__)

# Default ports when no shared config (so we still poll and show status for common backends)
# Keys match service names; display names (TWS/IBKR, TradeStation, etc.) applied in snapshot_display.
DEFAULT_BACKEND_PORTS: Dict[str, int] = {
    "ib": 8002,
    "alpaca": 8000,
    "tastytrade": 8005,
    "tradestation": 8001,
    "discount_bank": 8003,
    "risk_free_rate": 8004,
    "rust": 8080,  # Rust backend REST (matches config.services.rust_backend.rest_port)
}

# TWS/Gateway (socket, not HTTP). Port 7497 = paper, 7496 = live. Health is TCP connect only.
DEFAULT_TCP_BACKEND_PORTS: Dict[str, int] = {
    "tws": 7497,
}

# Api-gateway single entry point (P1-B). When gateway runs (default :9000), TUI and Web use same pipeline.
DEFAULT_GATEWAY_BASE_URL: str = "http://localhost:9000"

# Preset REST provider types -> snapshot URL. All presets route through api-gateway when gateway is running.
PRESET_REST_ENDPOINTS: Dict[str, str] = {
    "rest_rust": f"{DEFAULT_GATEWAY_BASE_URL}/api/v1/snapshot",
    "rest_ib": f"{DEFAULT_GATEWAY_BASE_URL}/api/v1/ib/snapshot",
    "rest_tws_gateway": f"{DEFAULT_GATEWAY_BASE_URL}/api/v1/ib/snapshot",
    "rest_alpaca": f"{DEFAULT_GATEWAY_BASE_URL}/api/v1/alpaca/snapshot",
    "rest_tradestation": f"{DEFAULT_GATEWAY_BASE_URL}/api/v1/tradestation/snapshot",
    "rest_tastytrade": f"{DEFAULT_GATEWAY_BASE_URL}/api/v1/tastytrade/snapshot",
}


@dataclass
class TUIConfig:
    """
    TUI configuration matching C++ TUIConfig structure

    MIGRATION NOTE: This can be serialized to JSON and loaded in C++
    using nlohmann/json
    """

    provider_type: str = (
        "mock"  # "mock", "rest", "file", "nats", "rest_ib", "rest_alpaca", ...
    )
    rest_endpoint: str = "http://localhost:9000/api/v1/snapshot"  # P1-B: default via api-gateway (Rust)
    update_interval_ms: int = 1000
    refresh_rate_ms: int = 500
    rest_timeout_ms: int = 15000
    rest_verify_ssl: bool = False
    file_path: Optional[str] = None

    # When set, REST provider uses this interval (ms) outside US regular market hours (9:30–16:00 ET).
    # 0 or unset = use update_interval_ms always (no slowdown when market closed).
    out_of_market_interval_ms: int = 60_000  # 1 minute when closed

    # NATS pub/sub (for provider_type "nats")
    nats_url: str = "nats://localhost:4222"
    nats_snapshot_backend: str = "ib"  # subscribe to snapshot.{backend}

    # IBKR REST API settings
    ibkr_rest_base_url: str = "https://localhost:5001/v1/portal"
    ibkr_rest_account_id: str = ""
    ibkr_rest_verify_ssl: bool = False
    ibkr_rest_timeout_ms: int = 5000

    # Display settings
    show_colors: bool = True
    show_footer: bool = True

    # Backend service ports for health checks (name -> port), e.g. {"ib": 8002, "alpaca": 8000}
    backend_ports: Dict[str, int] = field(default_factory=dict)
    # TCP-only backends (e.g. TWS/Gateway on 7497); no /api/health, just socket connect
    tcp_backend_ports: Dict[str, int] = field(default_factory=dict)

    # Backends disabled due to missing/placeholder credentials (name -> reason)
    disabled_backends: Dict[str, str] = field(default_factory=dict)

    # User-disabled backends (toggle in setup screen); merged with disabled_backends for display
    user_disabled_backends: List[str] = field(default_factory=list)

    # Optional: unified health dashboard URL (GET returns { backends: { name: health } }). When set, TUI uses it instead of polling each backend.
    health_dashboard_url: Optional[str] = None

    # Optional: API router base URL. When set, TUI uses this base for snapshot, scenarios, bank-accounts, and health
    # (e.g. http://localhost:9xxx with routes /api/v1/snapshot, /scenarios, /api/bank-accounts, /api/health).
    api_base_url: Optional[str] = None

    # Optional: SQLite path for persisting latest snapshot per backend (fallback when backend is down or on startup).
    # Default when unset: SNAPSHOT_CACHE_DB env or ~/.config/ib_box_spread/snapshot_cache.db. Set to "" to disable.
    snapshot_cache_path: Optional[str] = None

    # Live/paper per provider (from shared config tws.port / alpaca.data_client_config.paper)
    tws_port_override: Optional[int] = None  # 7497 = paper, 7496 = live; when set, overrides tcp_backend_ports["tws"]
    alpaca_paper: Optional[bool] = None  # True = paper, False = live

    # Symbol watchlist for dashboard; mock provider generates data for these symbols when provider is mock.
    watchlist: List[str] = field(
        default_factory=lambda: ["SPX", "XSP", "NANOS", "TLT", "DSP"]
    )

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
                out_of_market_interval_ms=getattr(shared_tui, "out_of_market_interval_ms", 60_000),
                nats_url=getattr(shared_tui, "nats_url", "nats://localhost:4222"),
                nats_snapshot_backend=getattr(shared_tui, "nats_snapshot_backend", "ib"),
                api_base_url=getattr(shared_tui, "api_base_url", None),
                ibkr_rest_base_url=ibkr_rest_dict.get(
                    "baseUrl", ibkr_rest_dict.get("base_url", "https://localhost:5001/v1/portal")
                ),
                ibkr_rest_account_id=ibkr_rest_dict.get("accountId", ibkr_rest_dict.get("account_id", "")),
                ibkr_rest_verify_ssl=ibkr_rest_dict.get("verifySsl", ibkr_rest_dict.get("verify_ssl", False)),
                ibkr_rest_timeout_ms=ibkr_rest_dict.get("timeoutMs", ibkr_rest_dict.get("timeout_ms", 5000)),
                show_colors=show_colors,
                show_footer=show_footer,
            )
            from_services = _backend_ports_from_services(shared_config.services)
            config.backend_ports = {**DEFAULT_BACKEND_PORTS, **from_services}
            config.tcp_backend_ports = {**DEFAULT_TCP_BACKEND_PORTS, **(config.tcp_backend_ports or {})}
            if getattr(shared_config, "tws_port", None) is not None:
                config.tcp_backend_ports["tws"] = shared_config.tws_port
            if getattr(shared_config, "alpaca_paper", None) is not None:
                config.alpaca_paper = shared_config.alpaca_paper
            config.disabled_backends = _disabled_backends_from_env(shared_config.services)
            # Unified health dashboard: one URL to get all backends (optional)
            hd = (shared_config.services or {}).get("health_dashboard")
            if isinstance(hd, dict) and isinstance(hd.get("port"), int):
                config.health_dashboard_url = f"http://127.0.0.1:{hd['port']}/api/health"
            logger.info("Loaded TUI configuration from shared config file")
            # Overlay user_disabled_backends from TUI config file if present
            tui_path = TUIConfig.get_config_path()
            if Path(tui_path).exists():
                try:
                    tui_only = TUIConfig.load_from_file(tui_path)
                    if getattr(tui_only, "user_disabled_backends", None):
                        config.user_disabled_backends = list(tui_only.user_disabled_backends)
                    # Persist TUI provider choice across restarts (mock/rest/file/nats and endpoint)
                    config.provider_type = tui_only.provider_type or config.provider_type
                    config.rest_endpoint = tui_only.rest_endpoint or config.rest_endpoint
                    config.file_path = tui_only.file_path if tui_only.file_path else config.file_path
                    config.nats_url = getattr(tui_only, "nats_url", None) or config.nats_url
                    config.nats_snapshot_backend = getattr(tui_only, "nats_snapshot_backend", None) or config.nats_snapshot_backend
                    if hasattr(tui_only, "snapshot_cache_path"):
                        config.snapshot_cache_path = tui_only.snapshot_cache_path
                    if getattr(tui_only, "out_of_market_interval_ms", None) is not None:
                        config.out_of_market_interval_ms = tui_only.out_of_market_interval_ms
                except Exception:
                    pass
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
        default_config.tcp_backend_ports = dict(DEFAULT_TCP_BACKEND_PORTS)
        default_config.disabled_backends = _disabled_backends_from_env(
            {"alpaca": {}, "tastytrade": {}, "tradestation": {}}
        )
        default_config.save_to_file(config_path)
        logger.info(f"Initialized TUI config at {config_path}")
        config = default_config
    else:
        config = TUIConfig.load_from_file(config_path)
        # Ensure we poll common backends so status line can show them (or "disabled")
        config.backend_ports = {**DEFAULT_BACKEND_PORTS, **(config.backend_ports or {})}
        config.tcp_backend_ports = {**DEFAULT_TCP_BACKEND_PORTS, **(config.tcp_backend_ports or {})}
        config.disabled_backends = _disabled_backends_from_env(
            {"alpaca": {}, "tastytrade": {}, "tradestation": {}}
        )

    # Override with environment variables
    _apply_env_overrides(config)

    return config


def _backend_ports_from_services(services: dict) -> Dict[str, int]:
    """Extract backend name -> port from config services for health checks."""
    out: Dict[str, int] = {}
    # Services that expose /api/health on a single port
    # Map config keys to TUI backend names (e.g. rust_backend -> rust)
    for name, backend_key in (
        ("ib", "ib"),
        ("alpaca", "alpaca"),
        ("tradestation", "tradestation"),
        ("discount_bank", "discount_bank"),
        ("risk_free_rate", "risk_free_rate"),
        ("tastytrade", "tastytrade"),
        ("rust", "rust_backend"),
    ):
        svc = services.get(backend_key)
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


def _is_op_ref(value: Optional[str]) -> bool:
    """True if value looks like a 1Password op:// secret reference (configured, resolve at runtime)."""
    return bool(value and str(value).strip().startswith("op://"))


def _disabled_backends_from_env(services: dict) -> Dict[str, str]:
    """
    For backends present in services, detect missing/placeholder credentials
    and return backend name -> reason (e.g. "Missing API key").
    Uses getenv_or_resolve so 1Password op:// refs (OP_*_SECRET) count as configured when SDK available.
    """
    out: Dict[str, str] = {}
    if not isinstance(services, dict):
        return out

    # Alpaca: API keys or OAuth (env or optional 1Password OP_*_SECRET refs, or SDK discovery)
    if services.get("alpaca") is not None:
        has_oauth = not _is_placeholder_or_empty(
            getenv_or_resolve("ALPACA_CLIENT_ID", "OP_ALPACA_CLIENT_ID_SECRET", "")
        ) and not _is_placeholder_or_empty(
            getenv_or_resolve("ALPACA_CLIENT_SECRET", "OP_ALPACA_CLIENT_SECRET_SECRET", "")
        )
        has_api_key = not _is_placeholder_or_empty(
            getenv_or_resolve("ALPACA_API_KEY_ID", "OP_ALPACA_API_KEY_ID_SECRET", "")
        ) and not _is_placeholder_or_empty(
            getenv_or_resolve("ALPACA_API_SECRET_KEY", "OP_ALPACA_API_SECRET_KEY_SECRET", "")
        )
        # Treat exported 1Password op:// refs as configured (resolution happens when client runs)
        if not has_api_key:
            op_key = (os.getenv("OP_ALPACA_API_KEY_ID_SECRET") or "").strip()
            op_secret = (os.getenv("OP_ALPACA_API_SECRET_KEY_SECRET") or "").strip()
            if _is_op_ref(op_key) and _is_op_ref(op_secret):
                has_api_key = True
        if not has_oauth and not has_api_key:
            # SDK/CLI discovery: find Alpaca item in 1Password and resolve (no env vars required)
            try:
                from ..integration.onepassword_sdk_helper import get_alpaca_credentials_from_1password
                if get_alpaca_credentials_from_1password():
                    has_api_key = True
            except Exception:
                pass
        if not has_oauth and not has_api_key:
            out["alpaca"] = "Missing API key or OAuth credentials"

    # Tastytrade: session or OAuth (env or optional 1Password OP_*_SECRET refs)
    if services.get("tastytrade") is not None:
        has_oauth = not _is_placeholder_or_empty(
            getenv_or_resolve("TASTYTRADE_CLIENT_SECRET", "OP_TASTYTRADE_CLIENT_SECRET_SECRET", "")
        ) and not _is_placeholder_or_empty(
            getenv_or_resolve("TASTYTRADE_REFRESH_TOKEN", "OP_TASTYTRADE_REFRESH_TOKEN_SECRET", "")
        )
        has_session = not _is_placeholder_or_empty(
            getenv_or_resolve("TASTYTRADE_USERNAME", "OP_TASTYTRADE_USERNAME_SECRET", "")
        ) and not _is_placeholder_or_empty(
            getenv_or_resolve("TASTYTRADE_PASSWORD", "OP_TASTYTRADE_PASSWORD_SECRET", "")
        )
        if not has_oauth and not has_session:
            out["tastytrade"] = "Missing credentials"

    # TradeStation: OAuth client credentials (env or optional 1Password OP_*_SECRET refs)
    if services.get("tradestation") is not None:
        has_creds = not _is_placeholder_or_empty(
            getenv_or_resolve("TRADESTATION_CLIENT_ID", "OP_TRADESTATION_CLIENT_ID_SECRET", "")
        ) and not _is_placeholder_or_empty(
            getenv_or_resolve("TRADESTATION_CLIENT_SECRET", "OP_TRADESTATION_CLIENT_SECRET_SECRET", "")
        )
        if not has_creds:
            out["tradestation"] = "Missing client ID or secret"

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

    env_health_url = os.getenv("TUI_HEALTH_DASHBOARD_URL")
    if env_health_url:
        config.health_dashboard_url = env_health_url

    env_api_base = os.getenv("TUI_API_BASE_URL")
    if env_api_base:
        config.api_base_url = env_api_base.strip().rstrip("/") or None
