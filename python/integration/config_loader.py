"""
config_loader.py - Shared configuration loader for Python services
Provides functions to read port assignments and other settings from config.json
Supports environment variable overrides
"""
import json
import os
from pathlib import Path
from typing import Optional, Any, Dict


def _find_config_file(config_path: Optional[str] = None) -> Optional[Path]:
    """Find config file using same logic as config_adapter."""
    candidates = []

    # If explicit path provided, use it
    if config_path:
        candidates.append(Path(config_path).expanduser())

    # Check environment variable
    env_override = os.getenv("IB_BOX_SPREAD_CONFIG")
    if env_override:
        candidates.append(Path(env_override).expanduser())

    # Standard locations
    home = Path.home()
    candidates.append(home / ".config" / "ib_box_spread" / "config.json")
    if os.name == "posix" and Path("/System/Library").exists():  # macOS
        candidates.append(home / "Library" / "Application Support" / "ib_box_spread" / "config.json")

    # Project root config
    # Try to find project root (look for config/ directory)
    current = Path(__file__).resolve()
    for parent in [current.parent, current.parent.parent, current.parent.parent.parent]:
        config_dir = parent / "config"
        if config_dir.exists():
            candidates.append(config_dir / "config.json")
            candidates.append(config_dir / "config.example.json")
            break

    # System locations
    candidates.append(Path("/usr/local/etc/ib_box_spread/config.json"))
    candidates.append(Path("/etc/ib_box_spread/config.json"))

    # Return first existing file
    for candidate in candidates:
        if candidate.exists() and candidate.is_file():
            return candidate

    return None


def get_service_port(service_name: str, default_port: Optional[int] = None) -> int:
    """
    Get service port from config with environment variable override.

    Args:
        service_name: Service name (e.g., "ib", "alpaca", "web")
        default_port: Default port if not found in config

    Returns:
        Port number

    Raises:
        ValueError: If port not found and no default provided
    """
    # Check environment variable first (highest priority)
    env_var_name = f"{service_name.upper().replace('_', '')}_PORT"
    env_port = os.getenv(env_var_name)
    if env_port:
        try:
            return int(env_port)
        except ValueError:
            raise ValueError(f"Invalid port in {env_var_name}: {env_port}")

    # Try config file
    config_file = _find_config_file()
    if config_file:
        try:
            with open(config_file, "r") as f:
                config = json.load(f)
                port = config.get("services", {}).get(service_name, {}).get("port")
                if port is not None:
                    return int(port)
        except (json.JSONDecodeError, KeyError, ValueError, OSError):
            pass  # Fall through to default

    # Fall back to default
    if default_port is not None:
        return default_port

    raise ValueError(f"Port not found for service '{service_name}' and no default provided")


def check_port_available(port: int) -> bool:
    """
    Check if a port is available (not in use).

    Args:
        port: Port number to check

    Returns:
        True if port is available, False if in use
    """
    import socket

    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.settimeout(0.1)
            result = s.connect_ex(("127.0.0.1", port))
            return result != 0  # 0 means connection succeeded (port in use)
    except Exception:
        # If we can't check, assume available
        return True


def get_config_value(json_path: str, default_value: Optional[Any] = None) -> Any:
    """
    Get config value using JSON path.

    Args:
        json_path: JSON path (e.g., ".tws.port")
        default_value: Default value if not found

    Returns:
        Config value or default
    """
    config_file = _find_config_file()
    if not config_file:
        return default_value

    try:
        with open(config_file, "r") as f:
            config = json.load(f)

        # Simple path traversal (supports .key.key format)
        keys = json_path.strip(".").split(".")
        value = config
        for key in keys:
            if isinstance(value, dict):
                value = value.get(key)
            else:
                return default_value

        return value if value is not None else default_value
    except (json.JSONDecodeError, KeyError, OSError):
        return default_value
