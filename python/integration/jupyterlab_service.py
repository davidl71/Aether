#!/usr/bin/env python3
"""
jupyterlab_service.py - Wrapper script to launch JupyterLab as a service

This script launches JupyterLab following the existing service pattern.
It reads configuration from config.json and starts JupyterLab on the configured port.

Usage:
    python -m integration.jupyterlab_service

Environment Variables:
    JUPYTERLAB_PORT: Override port from config (default: 8888)
    JUPYTERLAB_DIR: Working directory for notebooks (default: project root)
    JUPYTERLAB_TOKEN: Authentication token (auto-generated if not set)
    JUPYTERLAB_PASSWORD: Password hash (alternative to token)
    JUPYTERLAB_ALLOW_ORIGIN: CORS origin (default: *)
    JUPYTERLAB_IP: IP to bind to (default: 127.0.0.1)
"""
from __future__ import annotations

import os
import sys
import subprocess
from pathlib import Path

# Add project root to path for imports
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "python"))

try:
    from integration.config_loader import ConfigLoader
except ImportError:
    # Fallback if config_loader not available
    ConfigLoader = None

try:
    from integration.onepassword_sdk_helper import getenv_or_resolve
except ImportError:
    def getenv_or_resolve(env_var: str, op_ref: str, default: str = "") -> str:
        return os.getenv(env_var, default)


def get_jupyterlab_port() -> int:
    """Get JupyterLab port from config or environment."""
    # Check environment variable first
    env_port = os.getenv("JUPYTERLAB_PORT")
    if env_port:
        try:
            return int(env_port)
        except ValueError:
            pass

    # Try to get from config
    if ConfigLoader:
        try:
            config = ConfigLoader.load()
            services = config.get("services", {})
            jupyterlab_config = services.get("jupyterlab", {})
            port = jupyterlab_config.get("port", 8888)
            return int(port)
        except Exception:
            pass

    # Default port
    return 8888


def get_jupyterlab_dir() -> Path:
    """Get JupyterLab working directory."""
    env_dir = os.getenv("JUPYTERLAB_DIR")
    if env_dir:
        return Path(env_dir).expanduser().resolve()

    # Default to project root
    return project_root


def main() -> int:
    """Launch JupyterLab server."""
    port = get_jupyterlab_port()
    notebook_dir = get_jupyterlab_dir()

    # Ensure notebook directory exists
    notebook_dir.mkdir(parents=True, exist_ok=True)

    # Build JupyterLab command
    cmd = [
        sys.executable,
        "-m",
        "jupyterlab",
        "--no-browser",
        "--ip", os.getenv("JUPYTERLAB_IP", "127.0.0.1"),
        "--port", str(port),
        "--notebook-dir", str(notebook_dir),
        "--allow-root",  # Allow running as root (for Docker)
    ]

    # Add token if provided; optional 1Password op:// ref via OP_JUPYTERLAB_TOKEN_SECRET
    token = getenv_or_resolve("JUPYTERLAB_TOKEN", "OP_JUPYTERLAB_TOKEN_SECRET", "")
    if token:
        cmd.extend(["--NotebookApp.token", token])

    # Add password if provided; optional 1Password op:// ref via OP_JUPYTERLAB_PASSWORD_SECRET
    password = getenv_or_resolve("JUPYTERLAB_PASSWORD", "OP_JUPYTERLAB_PASSWORD_SECRET", "")
    if password:
        cmd.extend(["--NotebookApp.password", password])

    # Add CORS origin
    allow_origin = os.getenv("JUPYTERLAB_ALLOW_ORIGIN", "*")
    cmd.extend([
        "--ServerApp.allow_origin", allow_origin,
        "--ServerApp.disable_check_xsrf", "True",  # Allow CORS
    ])

    print(f"Starting JupyterLab on http://127.0.0.1:{port}", file=sys.stderr)
    print(f"Notebook directory: {notebook_dir}", file=sys.stderr)
    print("Access token will be displayed in the output", file=sys.stderr)
    print("", file=sys.stderr)

    # Launch JupyterLab
    try:
        subprocess.run(cmd, check=True)
    except KeyboardInterrupt:
        print("\nJupyterLab stopped by user", file=sys.stderr)
        return 0
    except subprocess.CalledProcessError as e:
        print(f"Error: JupyterLab failed to start: {e}", file=sys.stderr)
        return 1
    except FileNotFoundError:
        print("Error: JupyterLab not installed. Install with: pip install jupyterlab", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
