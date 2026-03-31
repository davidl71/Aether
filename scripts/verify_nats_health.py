#!/usr/bin/env python3
"""
Verify backend_service NATS health surfaces via the REST snapshot server.

This is meant to partially automate the "NATS health surfaces in TUI" manual checks by
asserting the backend publishes a reasonable transport health payload at `GET /health`.

Defaults assume the REST snapshot server is enabled (REST_SNAPSHOT_PORT) and reachable at:
  http://localhost:8002/health

Usage:
  python scripts/verify_nats_health.py
  python scripts/verify_nats_health.py --base-url http://localhost:8002
  python scripts/verify_nats_health.py --expect connected
  python scripts/verify_nats_health.py --expect disconnected
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request


def _get_json(url: str, timeout_s: float) -> dict:
    req = urllib.request.Request(url, headers={"Accept": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=timeout_s) as resp:
            body = resp.read()
            return json.loads(body.decode("utf-8"))
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"HTTP {e.code} for {url}: {e.read().decode('utf-8', 'replace')}") from e
    except urllib.error.URLError as e:
        raise RuntimeError(f"Request failed for {url}: {e}") from e
    except json.JSONDecodeError as e:
        raise RuntimeError(f"Invalid JSON from {url}: {e}") from e


def _field(obj: dict, key: str) -> object:
    if key not in obj:
        raise KeyError(key)
    return obj[key]


def main() -> int:
    p = argparse.ArgumentParser(description="Verify NATS transport health via GET /health")
    p.add_argument(
        "--base-url",
        default="http://localhost:8002",
        help="Base URL for REST snapshot server (default: http://localhost:8002)",
    )
    p.add_argument(
        "--timeout",
        type=float,
        default=5.0,
        help="HTTP timeout in seconds (default: 5)",
    )
    p.add_argument(
        "--expect",
        choices=["any", "connected", "disconnected"],
        default="any",
        help="Expected transport state (default: any)",
    )
    args = p.parse_args()

    health_url = args.base_url.rstrip("/") + "/health"
    payload = _get_json(health_url, timeout_s=args.timeout)

    try:
        transport = _field(payload, "transport")
        if not isinstance(transport, dict):
            raise TypeError("transport is not an object")

        status = _field(transport, "status")
        if not isinstance(status, str) or not status.strip():
            raise TypeError("transport.status is not a non-empty string")

        subject = transport.get("subject")
        role = transport.get("role")
        last_seen_at = transport.get("last_seen_at")

        if args.expect == "connected" and status != "ok":
            raise AssertionError(f"expected connected (status=ok), got status={status!r}")
        if args.expect == "disconnected" and status == "ok":
            raise AssertionError(f"expected disconnected (status!=ok), got status={status!r}")

    except Exception as e:
        print(f"FAIL: {e}", file=sys.stderr)
        print(f"# url: {health_url}", file=sys.stderr)
        try:
            print(json.dumps(payload, indent=2), file=sys.stderr)
        except Exception:
            pass
        return 2

    print("OK")
    print(f"- url: {health_url}")
    print(f"- transport.status: {status}")
    if role:
        print(f"- transport.role: {role}")
    if subject:
        print(f"- transport.subject: {subject}")
    if last_seen_at:
        print(f"- transport.last_seen_at: {last_seen_at}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

