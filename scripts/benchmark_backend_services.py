#!/usr/bin/env python3
"""
Benchmark backend services (IB, Alpaca, TradeStation, Tastytrade, etc.).

Measures latency for GET /api/health and GET /api/snapshot (or /api/v1/snapshot)
per service. Optional concurrent requests for throughput.

Usage:
  uv run python scripts/benchmark_backend_services.py
  uv run python scripts/benchmark_backend_services.py --requests 20 --concurrent 3
  uv run python scripts/benchmark_backend_services.py --config config/config.json
  uv run python scripts/benchmark_backend_services.py --services ib,alpaca

Requires: requests (or run from project venv with dependencies installed).
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple
from concurrent.futures import ThreadPoolExecutor, as_completed

try:
    import requests
except ImportError:
    print("Requires requests. Install with: uv pip install requests", file=sys.stderr)
    sys.exit(1)

# Default ports aligned with config/config.example.json "services"
DEFAULT_SERVICES: List[Dict[str, Any]] = [
    {"name": "ib", "port": 8002, "host": "127.0.0.1", "endpoints": [("/api/health", "health", 5), ("/api/v1/snapshot", "snapshot", 20)]},
    {"name": "alpaca", "port": 8000, "host": "127.0.0.1", "endpoints": [("/api/health", "health", 5), ("/api/snapshot", "snapshot", 20)]},
    {"name": "tradestation", "port": 8001, "host": "127.0.0.1", "endpoints": [("/api/health", "health", 5), ("/api/snapshot", "snapshot", 20)]},
    {"name": "tastytrade", "port": 8005, "host": "127.0.0.1", "endpoints": [("/api/health", "health", 5), ("/api/v1/snapshot", "snapshot", 20)]},
    {"name": "discount_bank", "port": 8003, "host": "127.0.0.1", "endpoints": [("/api/health", "health", 5)]},
    {"name": "risk_free_rate", "port": 8004, "host": "127.0.0.1", "endpoints": [("/api/health", "health", 5)]},
    {"name": "analytics", "port": 8007, "host": "127.0.0.1", "endpoints": [("/api/health", "health", 5)]},
]


def load_services_from_config(config_path: Path) -> List[Dict[str, Any]]:
    """Load service ports from config JSON; merge with default endpoints."""
    if not config_path.exists():
        return DEFAULT_SERVICES
    try:
        raw = config_path.read_text(encoding="utf-8")
        # Strip single-line // comments for JSON-with-comments configs
        lines = []
        for line in raw.splitlines():
            s = line.strip()
            if s.startswith("//"):
                continue
            lines.append(line)
        data = json.loads("\n".join(lines))
    except Exception as e:
        print(f"Warning: could not load {config_path}: {e}", file=sys.stderr)
        return DEFAULT_SERVICES

    services_config = data.get("services") or {}
    port_map = {
        "ib": ("ib", 8002),
        "alpaca": ("alpaca", 8000),
        "tradestation": ("tradestation", 8001),
        "tastytrade": ("tastytrade", 8005),
        "discount_bank": ("discount_bank", 8003),
        "risk_free_rate": ("risk_free_rate", 8004),
        "analytics": ("analytics", 8007),
    }
    result = []
    for key, (name, default_port) in port_map.items():
        port = default_port
        if isinstance(services_config.get(key), dict) and "port" in services_config[key]:
            port = int(services_config[key]["port"])
        elif isinstance(services_config.get(key), (int, float)):
            port = int(services_config[key])
        svc = next((s for s in DEFAULT_SERVICES if s["name"] == name), None)
        if svc:
            result.append({**svc, "port": port})
        else:
            endpoints = [("/api/health", "health", 5)]
            if key in ("ib", "alpaca", "tradestation", "tastytrade"):
                path = "/api/v1/snapshot" if key in ("ib", "tastytrade") else "/api/snapshot"
                endpoints.append((path, "snapshot", 20))
            result.append({"name": name, "port": port, "host": "127.0.0.1", "endpoints": endpoints})
    return result


def one_request(url: str, timeout_sec: int) -> Tuple[float, Optional[int], Optional[str]]:
    """Return (latency_sec, status_code, error_message)."""
    start = time.perf_counter()
    try:
        r = requests.get(url, timeout=timeout_sec)
        elapsed = time.perf_counter() - start
        return (elapsed, r.status_code, None if r.ok else r.text[:200])
    except Exception as e:
        elapsed = time.perf_counter() - start
        return (elapsed, None, str(e))


def run_benchmark(
    services: List[Dict[str, Any]],
    requests_per_endpoint: int = 10,
    concurrent: int = 1,
) -> List[Dict[str, Any]]:
    results = []
    for svc in services:
        name = svc["name"]
        host = svc.get("host", "127.0.0.1")
        port = svc["port"]
        base = f"http://{host}:{port}"
        for path, label, timeout_sec in svc["endpoints"]:
            url = base + path
            latencies: List[float] = []
            errors: List[str] = []
            status_codes: List[int] = []

            if concurrent <= 1:
                for _ in range(requests_per_endpoint):
                    elapsed, code, err = one_request(url, timeout_sec)
                    latencies.append(elapsed * 1000)  # ms
                    if code is not None:
                        status_codes.append(code)
                    if err:
                        errors.append(err)
            else:
                with ThreadPoolExecutor(max_workers=min(concurrent, requests_per_endpoint)) as ex:
                    futures = [ex.submit(one_request, url, timeout_sec) for _ in range(requests_per_endpoint)]
                    for f in as_completed(futures):
                        elapsed, code, err = f.result()
                        latencies.append(elapsed * 1000)
                        if code is not None:
                            status_codes.append(code)
                        if err:
                            errors.append(err)

            if not latencies:
                results.append({
                    "service": name,
                    "endpoint": label,
                    "url": url,
                    "ok": False,
                    "n": 0,
                    "min_ms": None,
                    "max_ms": None,
                    "avg_ms": None,
                    "p95_ms": None,
                    "errors": len(errors),
                    "sample_error": errors[0] if errors else None,
                })
                continue

            ok = len(errors) == 0 and all(c == 200 for c in status_codes)
            latencies_sorted = sorted(latencies)
            p95_idx = max(0, int(len(latencies_sorted) * 0.95) - 1)
            p95_ms = latencies_sorted[p95_idx] if latencies_sorted else None

            results.append({
                "service": name,
                "endpoint": label,
                "url": url,
                "ok": ok,
                "n": len(latencies),
                "min_ms": round(min(latencies), 2),
                "max_ms": round(max(latencies), 2),
                "avg_ms": round(statistics.mean(latencies), 2),
                "p95_ms": round(p95_ms, 2) if p95_ms is not None else None,
                "errors": len(errors),
                "sample_error": errors[0] if errors else None,
            })
    return results


def print_results(results: List[Dict[str, Any]], verbose: bool = False) -> None:
    print("\nBackend service benchmark\n" + "=" * 72)
    print(f"{'Service':<18} {'Endpoint':<10} {'Status':<6} {'N':>4} {'Min(ms)':>9} {'Avg(ms)':>9} {'P95(ms)':>9} {'Max(ms)':>9}")
    print("-" * 72)
    for r in results:
        status = "ok" if r["ok"] else "fail"
        n = r["n"]
        min_ms = f"{r['min_ms']:.2f}" if r["min_ms"] is not None else "—"
        avg_ms = f"{r['avg_ms']:.2f}" if r["avg_ms"] is not None else "—"
        p95_ms = f"{r['p95_ms']:.2f}" if r["p95_ms"] is not None else "—"
        max_ms = f"{r['max_ms']:.2f}" if r["max_ms"] is not None else "—"
        print(f"{r['service']:<18} {r['endpoint']:<10} {status:<6} {n:>4} {min_ms:>9} {avg_ms:>9} {p95_ms:>9} {max_ms:>9}")
        if verbose and r.get("sample_error"):
            print(f"  └─ {r['sample_error'][:80]}")
    print("=" * 72)


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark backend services (health + snapshot latency)")
    parser.add_argument("--config", type=Path, default=Path("config/config.json"), help="Config JSON (ports); falls back to config.example.json")
    parser.add_argument("--requests", "-n", type=int, default=10, help="Requests per endpoint (default 10)")
    parser.add_argument("--concurrent", "-c", type=int, default=1, help="Concurrent requests (default 1)")
    parser.add_argument("--services", type=str, default=None, help="Comma-separated service names (default: all)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Show error samples")
    parser.add_argument("--json", action="store_true", help="Output JSON instead of table")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    config_path = root / args.config if not args.config.is_absolute() else args.config
    if not config_path.exists():
        config_path = root / "config" / "config.example.json"

    services = load_services_from_config(config_path)
    if args.services:
        want = {s.strip().lower() for s in args.services.split(",")}
        services = [s for s in services if s["name"].lower() in want]
    if not services:
        print("No services to benchmark.", file=sys.stderr)
        return 1

    results = run_benchmark(services, requests_per_endpoint=args.requests, concurrent=args.concurrent)
    if args.json:
        print(json.dumps(results, indent=2))
    else:
        print_results(results, verbose=args.verbose)

    return 0 if all(r["ok"] for r in results) else 1


if __name__ == "__main__":
    sys.exit(main())
