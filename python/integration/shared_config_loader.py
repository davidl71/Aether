"""
Shared Configuration Loader

Implements the unified JSON configuration file format for TUI, PWA, and standalone applications.
Based on design from docs/research/architecture/SHARED_CONFIGURATION_SCHEMA.md

Supports:
- Multi-path configuration file discovery
- Environment variable overrides
- Data source selection with primary + fallback chain
- Application-specific configuration sections (TUI, PWA, broker)
- Backward compatibility with existing config formats
"""

import json
import os
import logging
import platform
from pathlib import Path
from typing import Any, Dict, List, Optional
from dataclasses import dataclass, field

logger = logging.getLogger(__name__)


@dataclass
class DataSourceConfig:
    """Configuration for a single data source."""
    type: str  # "alpaca", "ib", "tradestation", "mock", "static"
    enabled: bool = True
    api_key_id: Optional[str] = None
    api_secret_key: Optional[str] = None
    base_url: Optional[str] = None
    paper: bool = True
    timeout_ms: int = 30000
    rate_limit_per_minute: Optional[int] = None
    # IB-specific fields
    host: Optional[str] = None
    port: Optional[int] = None
    client_id: Optional[int] = None
    connection_type: Optional[str] = None  # "tws" or "clientPortal"
    # Additional fields stored as dict
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class TUIConfig:
    """TUI-specific configuration section."""
    provider_type: str = "rest"  # "mock", "rest", "file", "ibkr_rest", "livevol"
    update_interval_ms: int = 1000
    refresh_rate_ms: int = 500
    rest_endpoint: str = "http://localhost:8080/api/snapshot"
    rest_timeout_ms: int = 10000
    rest_verify_ssl: bool = False
    file_path: Optional[str] = None
    ibkr_rest: Dict[str, Any] = field(default_factory=dict)
    display: Dict[str, bool] = field(default_factory=lambda: {"showColors": True, "showFooter": True})


@dataclass
class PWAConfig:
    """PWA-specific configuration section."""
    service_ports: Dict[str, int] = field(default_factory=dict)
    default_service: str = "alpaca"
    service_urls: Dict[str, str] = field(default_factory=dict)


@dataclass
class BrokerConfig:
    """Broker/standalone configuration section."""
    primary: str = "ALPACA"
    priorities: List[str] = field(default_factory=lambda: ["alpaca", "ib", "mock"])


@dataclass
class SharedConfig:
    """Unified configuration structure."""
    version: str = "1.0.0"
    data_sources: Dict[str, Any] = field(default_factory=lambda: {
        "primary": "alpaca",
        "fallback": [],
        "sources": {}
    })
    services: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    tui: Optional[TUIConfig] = None
    pwa: Optional[PWAConfig] = None
    broker: Optional[BrokerConfig] = None
    # Additional sections stored as dict
    extra: Dict[str, Any] = field(default_factory=dict)


class SharedConfigLoader:
    """Loader for unified configuration format."""

    @staticmethod
    def _candidate_paths(config_path: Optional[str] = None) -> List[Path]:
        """Get candidate paths for configuration file search."""
        candidates: List[Path] = []

        def add(path: Path) -> None:
            normalized = path.expanduser()
            if normalized not in candidates:
                candidates.append(normalized)

        # 1. Explicit path (if provided)
        if config_path:
            requested = Path(config_path).expanduser()
            add(requested if requested.is_absolute() else (Path.cwd() / requested))
            if not requested.is_absolute():
                add(requested)

        # 2. Environment variable
        env_override = os.getenv("IB_BOX_SPREAD_CONFIG")
        if env_override:
            env_path = Path(env_override).expanduser()
            add(env_path if env_path.is_absolute() else (Path.cwd() / env_path))

        # 3. Home config
        home = Path.home()
        add(home / ".config" / "ib_box_spread" / "config.json")

        # 4. macOS Application Support
        if platform.system() == "Darwin":
            add(home / "Library" / "Application Support" / "ib_box_spread" / "config.json")

        # 5. System configs
        add(Path("/usr/local/etc/ib_box_spread/config.json"))
        add(Path("/etc/ib_box_spread/config.json"))

        # 6. Project config
        project_root = Path(__file__).parent.parent.parent
        add(project_root / "config" / "config.json")
        add(project_root / "config" / "config.example.json")

        return candidates

    @staticmethod
    def _resolve_env_placeholders(value: Any) -> Any:
        """Resolve ${VAR_NAME} placeholders in configuration values."""
        if isinstance(value, str) and value.startswith("${") and value.endswith("}"):
            var_name = value[2:-1]
            env_value = os.getenv(var_name)
            if env_value is not None:
                return env_value
            logger.warning(f"Environment variable {var_name} not found, using placeholder")
            return value
        elif isinstance(value, dict):
            return {k: SharedConfigLoader._resolve_env_placeholders(v) for k, v in value.items()}
        elif isinstance(value, list):
            return [SharedConfigLoader._resolve_env_placeholders(item) for item in value]
        return value

    @staticmethod
    def _apply_env_overrides(config: Dict[str, Any]) -> Dict[str, Any]:
        """Apply environment variable overrides to configuration."""
        # Data source primary
        if env_primary := os.getenv("DATA_SOURCE_PRIMARY"):
            if "dataSources" not in config:
                config["dataSources"] = {}
            config["dataSources"]["primary"] = env_primary

        # TUI provider type
        if env_tui_backend := os.getenv("TUI_BACKEND"):
            if "tui" not in config:
                config["tui"] = {}
            config["tui"]["providerType"] = env_tui_backend

        # Service ports
        for service_name in ["alpaca", "ib", "tradestation", "discountBank"]:
            env_port = os.getenv(f"{service_name.upper().replace('_', '')}_PORT")
            if env_port:
                if "services" not in config:
                    config["services"] = {}
                if service_name not in config["services"]:
                    config["services"][service_name] = {}
                try:
                    config["services"][service_name]["port"] = int(env_port)
                except ValueError:
                    logger.warning(f"Invalid port value for {service_name}: {env_port}")

        return config

    @staticmethod
    def load_config(config_path: Optional[str] = None) -> SharedConfig:
        """
        Load configuration from JSON file.

        Args:
            config_path: Optional explicit path to config file

        Returns:
            SharedConfig object with loaded configuration

        Raises:
            FileNotFoundError: If config file not found
            json.JSONDecodeError: If config file is invalid JSON
        """
        candidates = SharedConfigLoader._candidate_paths(config_path)
        last_error: Optional[Exception] = None

        for candidate in candidates:
            try:
                if not candidate.exists() or not candidate.is_file():
                    continue

                with candidate.open("r", encoding="utf-8") as fh:
                    config_dict = json.load(fh)

                logger.info(f"Loaded configuration from {candidate}")

                # Resolve environment variable placeholders
                config_dict = SharedConfigLoader._resolve_env_placeholders(config_dict)

                # Apply environment variable overrides
                config_dict = SharedConfigLoader._apply_env_overrides(config_dict)

                # Parse into SharedConfig dataclass
                return SharedConfigLoader._parse_config(config_dict)

            except json.JSONDecodeError as err:
                logger.error(f"Failed to parse configuration at {candidate}: {err}")
                raise
            except Exception as err:
                last_error = err
                logger.debug(f"Failed to load configuration at {candidate}: {err}")

        searched = "\n  - ".join(str(path) for path in candidates)
        message = f"Configuration file not found. Searched:\n  - {searched}"
        if last_error:
            raise FileNotFoundError(message) from last_error
        raise FileNotFoundError(message)

    @staticmethod
    def _parse_config(config_dict: Dict[str, Any]) -> SharedConfig:
        """Parse configuration dictionary into SharedConfig dataclass."""
        # Extract data sources
        data_sources = config_dict.get("dataSources", {
            "primary": "alpaca",
            "fallback": [],
            "sources": {}
        })

        # Extract services
        services = config_dict.get("services", {})

        # Extract TUI config
        tui_dict = config_dict.get("tui")
        tui_config = None
        if tui_dict:
            tui_config = TUIConfig(
                provider_type=tui_dict.get("providerType", "rest"),
                update_interval_ms=tui_dict.get("updateIntervalMs", 1000),
                refresh_rate_ms=tui_dict.get("refreshRateMs", 500),
                rest_endpoint=tui_dict.get("restEndpoint", "http://localhost:8080/api/snapshot"),
                rest_timeout_ms=tui_dict.get("restTimeoutMs", 10000),
                rest_verify_ssl=tui_dict.get("restVerifySsl", False),
                file_path=tui_dict.get("filePath"),
                ibkr_rest=tui_dict.get("ibkrRest", {}),
                display=tui_dict.get("display", {"showColors": True, "showFooter": True})
            )

        # Extract PWA config
        pwa_dict = config_dict.get("pwa")
        pwa_config = None
        if pwa_dict:
            pwa_config = PWAConfig(
                service_ports=pwa_dict.get("servicePorts", {}),
                default_service=pwa_dict.get("defaultService", "alpaca"),
                service_urls=pwa_dict.get("serviceUrls", {})
            )

        # Extract broker config
        broker_dict = config_dict.get("broker")
        broker_config = None
        if broker_dict:
            broker_config = BrokerConfig(
                primary=broker_dict.get("primary", "ALPACA"),
                priorities=broker_dict.get("priorities", ["alpaca", "ib", "mock"])
            )

        # Extract extra fields
        known_keys = {"version", "dataSources", "services", "tui", "pwa", "broker"}
        extra = {k: v for k, v in config_dict.items() if k not in known_keys}

        return SharedConfig(
            version=config_dict.get("version", "1.0.0"),
            data_sources=data_sources,
            services=services,
            tui=tui_config,
            pwa=pwa_config,
            broker=broker_config,
            extra=extra
        )

    @staticmethod
    def get_data_source_config(config: SharedConfig, source_name: str) -> Optional[Dict[str, Any]]:
        """Get configuration for a specific data source."""
        sources = config.data_sources.get("sources", {})
        return sources.get(source_name)

    @staticmethod
    def get_primary_data_source(config: SharedConfig) -> str:
        """Get primary data source name."""
        return config.data_sources.get("primary", "alpaca")

    @staticmethod
    def get_fallback_data_sources(config: SharedConfig) -> List[str]:
        """Get fallback data source names."""
        return config.data_sources.get("fallback", [])


# Convenience function
def load_shared_config(config_path: Optional[str] = None) -> SharedConfig:
    """Load shared configuration (convenience function)."""
    return SharedConfigLoader.load_config(config_path)
