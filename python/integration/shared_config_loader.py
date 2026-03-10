"""
Shared Configuration Loader

Implements the unified JSON configuration file format for TUI, PWA, and standalone applications.
Based on design from docs/research/architecture/SHARED_CONFIGURATION_SCHEMA.md

Config location (when IB_BOX_SPREAD_CONFIG is not set):
- The app always uses the home config: ~/.config/ib_box_spread/config.json
  (or on macOS: ~/Library/Application Support/ib_box_spread/config.json).
- Project config (config/config.json, config.example.json) is for reference and is used
  only as the source of defaults to generate the home config on first run (bootstrap).
- If home config does not exist, it is created by copying from the project's
  config.example.json (or config.json), then loading proceeds from home.

Supports:
- Multi-path configuration file discovery
- Environment variable overrides (IB_BOX_SPREAD_CONFIG for explicit path)
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

try:
    import jsonschema
except ImportError:
    jsonschema = None  # type: ignore[misc, assignment]

try:
    from .onepassword_sdk_helper import resolve_secret
except ImportError:
    resolve_secret = None  # type: ignore[misc, assignment]


def _strip_json_comments(text: str) -> str:
    """Strip // and /* */ comments from JSON-like text so stdlib json can parse it."""
    out: list[str] = []
    i = 0
    n = len(text)
    in_double = False
    in_line = False
    in_block = False
    block_depth = 0

    while i < n:
        c = text[i]
        if in_line:
            if c == "\n":
                in_line = False
                out.append(c)
            i += 1
            continue
        if in_block:
            if c == "*" and i + 1 < n and text[i + 1] == "/":
                block_depth -= 1
                if block_depth == 0:
                    in_block = False
                i += 2
            elif c == "/" and i + 1 < n and text[i + 1] == "*":
                block_depth += 1
                i += 2
            else:
                i += 1
            continue
        if in_double:
            if c == "\\" and i + 1 < n:
                out.append(c)
                out.append(text[i + 1])
                i += 2
            elif c == '"':
                in_double = False
                out.append(c)
                i += 1
            else:
                out.append(c)
                i += 1
            continue
        if c == '"':
            in_double = True
            out.append(c)
            i += 1
        elif c == "/" and i + 1 < n:
            nxt = text[i + 1]
            if nxt == "/":
                in_line = True
                i += 2
            elif nxt == "*":
                in_block = True
                block_depth = 1
                i += 2
            else:
                out.append(c)
                i += 1
        else:
            out.append(c)
            i += 1

    return "".join(out)


@dataclass
class DataSourceConfig:
    """Configuration for a single data source."""
    type: str  # "alpaca", "ib", "mock", "static"
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
    rest_timeout_ms: int = 15000
    rest_verify_ssl: bool = False
    file_path: Optional[str] = None
    ibkr_rest: Dict[str, Any] = field(default_factory=dict)
    display: Dict[str, bool] = field(default_factory=lambda: {"showColors": True, "showFooter": True})
    api_base_url: Optional[str] = None  # when set, TUI uses this base for snapshot, scenarios, bank-accounts, health


@dataclass
class PWAConfig:
    """PWA-specific configuration section."""
    service_ports: Dict[str, int] = field(default_factory=dict)
    default_service: str = "ib"
    service_urls: Dict[str, str] = field(default_factory=dict)


@dataclass
class BrokerConfig:
    """Broker/standalone configuration section."""
    primary: str = "IB"
    priorities: List[str] = field(default_factory=lambda: ["alpaca", "ib", "mock"])


@dataclass
class SharedConfig:
    """Unified configuration structure."""
    version: str = "1.0.0"
    data_sources: Dict[str, Any] = field(default_factory=lambda: {
        "primary": "ib",
        "fallback": [],
        "sources": {}
    })
    services: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    tui: Optional[TUIConfig] = None
    pwa: Optional[PWAConfig] = None
    broker: Optional[BrokerConfig] = None
    # Additional sections stored as dict
    extra: Dict[str, Any] = field(default_factory=dict)
    # Live/paper from top-level tws / alpaca (flat config)
    tws_port: Optional[int] = None  # 7497 = paper, 7496 = live
    alpaca_paper: Optional[bool] = None


class SharedConfigLoader:
    """Loader for unified configuration format.

    Config is always loaded from the home directory when possible. Project config
    (config/config.json, config.example.json) is for reference and is used only
    as the source of defaults to generate the home config on first run.
    """

    # Preferred home config path (used for bootstrap and "always use home" behavior)
    @staticmethod
    def get_home_config_path() -> Path:
        """Return the primary home config file path (~/.config/ib_box_spread/config.json)."""
        return Path.home() / ".config" / "ib_box_spread" / "config.json"

    @staticmethod
    def _home_config_paths() -> List[Path]:
        """Return candidate paths for the user's home config (preferred order)."""
        home = Path.home()
        paths = [home / ".config" / "ib_box_spread" / "config.json"]
        if platform.system() == "Darwin":
            paths.append(home / "Library" / "Application Support" / "ib_box_spread" / "config.json")
        return paths

    @staticmethod
    def _project_default_config_path() -> Optional[Path]:
        """Return project config file to use as default source (for bootstrap). Prefer config.json, then config.example.json."""
        project_root = Path(__file__).parent.parent.parent
        for name in ("config.json", "config.example.json"):
            p = project_root / "config" / name
            if p.exists() and p.is_file():
                return p
        return None

    @staticmethod
    def _bootstrap_home_config_from_project() -> Optional[Path]:
        """If home config does not exist, create it from project default. Returns home path written, or None on failure."""
        home_paths = SharedConfigLoader._home_config_paths()
        if not home_paths:
            return None
        primary_home = home_paths[0]
        if primary_home.exists() and primary_home.is_file():
            return primary_home
        project_src = SharedConfigLoader._project_default_config_path()
        if not project_src:
            return None
        try:
            primary_home.parent.mkdir(parents=True, exist_ok=True)
            raw = project_src.read_text(encoding="utf-8")
            primary_home.write_text(raw, encoding="utf-8")
            logger.info("Bootstrapped home config from %s to %s", project_src, primary_home)
            return primary_home
        except Exception as e:
            logger.debug("Could not bootstrap home config: %s", e)
            return None

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
    def _resolve_env_placeholders(value: Any, quiet: bool = False) -> Any:
        """Resolve ${VAR_NAME} placeholders and op:// secret refs in configuration values."""
        if isinstance(value, str):
            s = value.strip()
            # 1Password op:// refs: resolve when SDK available
            if s.startswith("op://") and resolve_secret is not None:
                resolved = resolve_secret(s)
                if resolved is not None:
                    return resolved
                if not quiet:
                    logger.debug("1Password ref not resolved (SDK missing or auth failed), leaving as-is")
                return value
            if s.startswith("${") and s.endswith("}"):
                var_name = s[2:-1]
                env_value = os.getenv(var_name)
                if env_value is not None:
                    return env_value
                if quiet:
                    logger.debug(f"Environment variable {var_name} not found, using placeholder")
                else:
                    logger.warning(f"Environment variable {var_name} not found, using placeholder")
                return value
        elif isinstance(value, dict):
            return {k: SharedConfigLoader._resolve_env_placeholders(v, quiet) for k, v in value.items()}
        elif isinstance(value, list):
            return [SharedConfigLoader._resolve_env_placeholders(item, quiet) for item in value]
        return value

    @staticmethod
    def _validate_schema(config_dict: Dict[str, Any], config_path: Path) -> None:
        """If jsonschema is available and config has unified shape (dataSources.primary), validate against config/schema.json."""
        if jsonschema is None:
            return
        ds = config_dict.get("dataSources") or {}
        if not isinstance(ds, dict) or "primary" not in ds:
            logger.debug("Config missing dataSources.primary, skipping schema validation")
            return
        project_root = Path(__file__).parent.parent.parent
        schema_path = project_root / "config" / "schema.json"
        if not schema_path.exists() or not schema_path.is_file():
            logger.debug("Config schema not found at %s, skipping validation", schema_path)
            return
        try:
            with schema_path.open("r", encoding="utf-8") as f:
                schema = json.load(f)
            jsonschema.validate(config_dict, schema)
            logger.debug("Config validated against %s", schema_path)
        except jsonschema.ValidationError as e:
            logger.warning("Config at %s failed schema validation: %s", config_path, str(e))
        except Exception as e:
            logger.debug("Schema validation skipped: %s", e)

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
        for service_name in ["alpaca", "ib", "discountBank"]:
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
    def load_config(config_path: Optional[str] = None, quiet_placeholder_warnings: bool = False) -> SharedConfig:
        """
        Load configuration from JSON file.

        Args:
            config_path: Optional explicit path to config file
            quiet_placeholder_warnings: If True, log missing env var placeholders at DEBUG instead of WARNING

        Returns:
            SharedConfig object with loaded configuration

        Raises:
            FileNotFoundError: If config file not found
            json.JSONDecodeError: If config file is invalid JSON
        """
        candidates = SharedConfigLoader._candidate_paths(config_path)
        last_error: Optional[Exception] = None

        # When no explicit path: prefer home config. Bootstrap from project if home config is missing.
        use_only_home = not config_path and not os.getenv("IB_BOX_SPREAD_CONFIG")
        if use_only_home:
            home_paths = SharedConfigLoader._home_config_paths()
            any_home_exists = any(p.exists() and p.is_file() for p in home_paths)
            if not any_home_exists:
                SharedConfigLoader._bootstrap_home_config_from_project()
            if any(p.exists() and p.is_file() for p in home_paths):
                candidates = home_paths
        # Else: explicit path, or IB_BOX_SPREAD_CONFIG set, or no home config (use full list including project)

        for candidate in candidates:
            try:
                if not candidate.exists() or not candidate.is_file():
                    continue

                with candidate.open("r", encoding="utf-8") as fh:
                    raw = fh.read()
                config_dict = json.loads(_strip_json_comments(raw))

                logger.info(f"Loaded configuration from {candidate}")

                # Resolve environment variable placeholders
                config_dict = SharedConfigLoader._resolve_env_placeholders(
                    config_dict, quiet=quiet_placeholder_warnings
                )

                # Apply environment variable overrides
                config_dict = SharedConfigLoader._apply_env_overrides(config_dict)

                # Optional: validate against config/schema.json when jsonschema available
                SharedConfigLoader._validate_schema(config_dict, candidate)

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
            "primary": "ib",
            "fallback": [],
            "sources": {}
        })

        # Extract services
        services = config_dict.get("services", {})

        # Extract TUI config
        tui_dict = config_dict.get("tui")
        tui_config = None
        if tui_dict:
            def _tui(key_camel: str, key_snake: str, default: Any) -> Any:
                return tui_dict.get(key_camel, tui_dict.get(key_snake, default))

            tui_config = TUIConfig(
                provider_type=_tui("providerType", "provider_type", "rest"),
                update_interval_ms=_tui("updateIntervalMs", "update_interval_ms", 1000),
                refresh_rate_ms=_tui("refreshRateMs", "refresh_rate_ms", 500),
                rest_endpoint=_tui("restEndpoint", "rest_endpoint", "http://localhost:8080/api/snapshot"),
                rest_timeout_ms=_tui("restTimeoutMs", "rest_timeout_ms", 15000),
                rest_verify_ssl=_tui("restVerifySsl", "rest_verify_ssl", False),
                file_path=_tui("filePath", "file_path", None),
                ibkr_rest=_tui("ibkrRest", "ibkr_rest", {}),
                display=_tui("display", "display", {"showColors": True, "showFooter": True}),
                api_base_url=_tui("apiBaseUrl", "api_base_url", None),
            )

        # Extract PWA config
        pwa_dict = config_dict.get("pwa")
        pwa_config = None
        if pwa_dict:
            pwa_config = PWAConfig(
                service_ports=pwa_dict.get("servicePorts", {}),
                default_service=pwa_dict.get("defaultService", "ib"),
                service_urls=pwa_dict.get("serviceUrls", {})
            )

        # Extract broker config
        broker_dict = config_dict.get("broker")
        broker_config = None
        if broker_dict:
            broker_config = BrokerConfig(
                primary=broker_dict.get("primary", "IB"),
                priorities=broker_dict.get("priorities", ["alpaca", "ib", "mock"])
            )

        # Extract extra fields
        known_keys = {"version", "dataSources", "services", "tui", "pwa", "broker"}
        extra = {k: v for k, v in config_dict.items() if k not in known_keys}

        # Top-level tws / alpaca (flat config) for live vs paper
        tws_port = None
        if "tws" in config_dict and isinstance(config_dict["tws"], dict):
            p = config_dict["tws"].get("port")
            if isinstance(p, int) and 0 < p < 65536:
                tws_port = p
        alpaca_paper = None
        if "alpaca" in config_dict and isinstance(config_dict["alpaca"], dict):
            dcc = config_dict["alpaca"].get("data_client_config") or config_dict["alpaca"].get("dataClientConfig")
            if isinstance(dcc, dict) and "paper" in dcc:
                alpaca_paper = bool(dcc["paper"])

        return SharedConfig(
            version=config_dict.get("version", "1.0.0"),
            data_sources=data_sources,
            services=services,
            tui=tui_config,
            pwa=pwa_config,
            broker=broker_config,
            extra=extra,
            tws_port=tws_port,
            alpaca_paper=alpaca_paper,
        )

    @staticmethod
    def get_data_source_config(config: SharedConfig, source_name: str) -> Optional[Dict[str, Any]]:
        """Get configuration for a specific data source."""
        sources = config.data_sources.get("sources", {})
        return sources.get(source_name)

    @staticmethod
    def get_primary_data_source(config: SharedConfig) -> str:
        """Get primary data source name."""
        return config.data_sources.get("primary", "ib")

    @staticmethod
    def get_fallback_data_sources(config: SharedConfig) -> List[str]:
        """Get fallback data source names."""
        return config.data_sources.get("fallback", [])

    @staticmethod
    def patch_home_config(updates: Dict[str, Any]) -> None:
        """
        Read home config JSON, apply deep updates (e.g. tws.port, alpaca.data_client_config.paper),
        and write back. Use for live/paper switching and port updates.
        """
        path = SharedConfigLoader.get_home_config_path()
        if not path.exists() or not path.is_file():
            raise FileNotFoundError(f"Home config not found: {path}")
        raw = path.read_text(encoding="utf-8")
        config = json.loads(_strip_json_comments(raw))

        def deep_merge(base: Dict, patch: Dict) -> None:
            for k, v in patch.items():
                if isinstance(v, dict) and isinstance(base.get(k), dict):
                    deep_merge(base[k], v)
                else:
                    base[k] = v

        deep_merge(config, updates)
        path.write_text(json.dumps(config, indent=2), encoding="utf-8")
        logger.info("Updated home config at %s", path)


# Convenience function
def load_shared_config(config_path: Optional[str] = None) -> SharedConfig:
    """Load shared configuration (convenience function)."""
    return SharedConfigLoader.load_config(config_path)
