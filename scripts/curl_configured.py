#!/usr/bin/env python3
"""
Curl (or request) using the same configured URLs as the TUI.

Loads config via TUI config (config/config.json + shared config + env),
then runs GET against the configured snapshot and/or health URLs.
Useful for debugging "is the backend up?" without opening the TUI.

Usage:
  uv run python scripts/curl_configured.py              # request snapshot + health, print JSON
  uv run python scripts/curl_configured.py --snapshot   # snapshot only
  uv run python scripts/curl_configured.py --health      # health only
  uv run python scripts/curl_configured.py --curl       # print curl commands only (no request)
  uv run python scripts/curl_configured.py --health --path /api/health/dashboard  # health dashboard summary

Run from project root so config paths resolve.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

# Project root = parent of scripts/
SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))


def _effective_health_url(config) -> str | None:
    """Unified health URL from config (same logic as TUI app)."""
    url = getattr(config, "health_dashboard_url", None)
    if url:
        return url
    base = getattr(config, "api_base_url", None)
    if base:
        return base.strip().rstrip("/") + "/api/health"
    return None


def _get_snapshot_url(config) -> str:
    """Snapshot URL from config (same as RestProvider)."""
    if getattr(config, "api_base_url", None):
        base = config.api_base_url.strip().rstrip("/")
        return f"{base}/api/v1/snapshot"
    return (
        getattr(config, "rest_endpoint", None)
        or os.getenv("TUI_API_URL")
        or "http://localhost:8002/api/v1/snapshot"
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Request snapshot/health using TUI-configured URLs.",
    )
    parser.add_argument(
        "--snapshot",
        action="store_true",
        help="Request snapshot URL only (default: snapshot + health)",
    )
    parser.add_argument(
        "--health",
        action="store_true",
        help="Request health URL only (default: snapshot + health)",
    )
    parser.add_argument(
        "--path",
        default=None,
        metavar="PATH",
        help="Append path to health URL (e.g. /api/health/dashboard). Ignored for --snapshot.",
    )
    parser.add_argument(
        "--curl",
        action="store_true",
        help="Print curl commands only; do not perform requests.",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=10.0,
        help="Request timeout in seconds (default 10).",
    )
    args = parser.parse_args()

    # Default: both snapshot and health
    do_snapshot = args.snapshot or (not args.snapshot and not args.health)
    do_health = args.health or (not args.snapshot and not args.health)

    try:
        from python.tui.config import load_config
    except ImportError:
        print("Error: run from project root (python.tui.config not found)", file=sys.stderr)
        return 1

    config = load_config()
    snapshot_url = _get_snapshot_url(config)
    health_url = _effective_health_url(config)
    if args.path and health_url:
        health_url = health_url.rstrip("/") + "/" + args.path.lstrip("/")

    if args.curl:
        lines = []
        if do_snapshot:
            lines.append(f"curl -sS '{snapshot_url}'")
        if do_health and health_url:
            lines.append(f"curl -sS '{health_url}'")
        if not lines:
            print("Nothing to curl (no health URL in config?). Snapshot:", snapshot_url, file=sys.stderr)
            return 1
        print("\n".join(lines))
        return 0

    # Use requests if available, else subprocess curl
    try:
        import requests
        use_requests = True
    except ImportError:
        use_requests = False

    exit_code = 0
    if do_snapshot:
        print(f"# Snapshot: {snapshot_url}")
        if use_requests:
            try:
                r = requests.get(snapshot_url, timeout=args.timeout)
                r.raise_for_status()
                print(json.dumps(r.json(), indent=2))
            except Exception as e:
                print(str(e), file=sys.stderr)
                exit_code = 1
        else:
            r = subprocess.run(
                ["curl", "-sS", "-m", str(int(args.timeout)), snapshot_url],
                capture_output=True,
                text=True,
            )
            if r.returncode != 0:
                print(r.stderr or r.stdout, file=sys.stderr)
                exit_code = 1
            else:
                try:
                    print(json.dumps(json.loads(r.stdout), indent=2))
                except json.JSONDecodeError:
                    print(r.stdout)

    if do_health and health_url:
        print(f"\n# Health: {health_url}")
        if use_requests:
            try:
                r = requests.get(health_url, timeout=args.timeout)
                r.raise_for_status()
                print(json.dumps(r.json(), indent=2))
            except Exception as e:
                print(str(e), file=sys.stderr)
                exit_code = 1
        else:
            r = subprocess.run(
                ["curl", "-sS", "-m", str(int(args.timeout)), health_url],
                capture_output=True,
                text=True,
            )
            if r.returncode != 0:
                print(r.stderr or r.stdout, file=sys.stderr)
                exit_code = 1
            else:
                try:
                    print(json.dumps(json.loads(r.stdout), indent=2))
                except json.JSONDecodeError:
                    print(r.stdout)
    elif do_health and not health_url:
        print("# Health: (no health_dashboard_url or api_base_url in config)", file=sys.stderr)
        print("# Per-backend health: curl -sS http://127.0.0.1:8000/api/health  # Alpaca", file=sys.stderr)
        print("#                     curl -sS http://127.0.0.1:8002/api/health  # IB", file=sys.stderr)

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
