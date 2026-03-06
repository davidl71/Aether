"""
nats_snapshot_subscriber.py - Subscribe to snapshot.* and system.health from NATS.

Proof-of-concept: keeps last snapshot per backend and last health per backend in memory.
Optional --serve-port exposes the latest snapshot via GET /api/snapshot (single backend)
or GET /api/snapshot/{backend_id}.

Run:
  uv run python -m python.integration.nats_snapshot_subscriber
  uv run python -m python.integration.nats_snapshot_subscriber --nats-url nats://localhost:4222 --serve-port 9010

Requires: nats-py (pip install nats-py)
"""

from __future__ import annotations

import argparse
import asyncio
import json
import logging
import os
import sys
from pathlib import Path
from typing import Any, Dict, Optional

# Add project root for imports when run as script
if __name__ == "__main__" and __package__ is None:
    _root = Path(__file__).resolve().parent.parent.parent
    sys.path.insert(0, str(_root))

try:
    import nats
    from nats.aio.client import Client as NATS
    NATS_AVAILABLE = True
except ImportError:
    NATS = None
    NATS_AVAILABLE = False

logger = logging.getLogger(__name__)

# In-memory state: last snapshot per backend, last health per backend
_last_snapshot: Dict[str, Dict[str, Any]] = {}
_last_health: Dict[str, Dict[str, Any]] = {}


def _subject_to_backend(subject: str, prefix: str) -> Optional[str]:
    """e.g. snapshot.ib -> ib."""
    if subject.startswith(prefix + "."):
        return subject[len(prefix) + 1:].strip() or None
    return None


async def run_subscriber(
    nats_url: str,
    serve_port: Optional[int] = None,
) -> None:
    if not NATS_AVAILABLE:
        logger.error("nats-py not installed - pip install nats-py")
        return

    nc = NATS()
    try:
        await nc.connect(
            servers=[nats_url],
            reconnect_time_wait=2,
            max_reconnect_attempts=-1,
        )
    except Exception as e:
        logger.error("NATS connect failed: %s", e)
        return

    logger.info("Connected to NATS at %s", nats_url)

    async def on_snapshot(msg):
        subject = msg.subject
        backend = _subject_to_backend(subject, "snapshot")
        if not backend:
            return
        try:
            data = json.loads(msg.data.decode("utf-8"))
            _last_snapshot[backend] = data
            logger.info("Snapshot updated: %s (keys: %s)", backend, list(data.keys())[:5])
        except Exception as e:
            logger.warning("Snapshot decode %s: %s", subject, e)

    async def on_health(msg):
        try:
            data = json.loads(msg.data.decode("utf-8"))
            backend = data.get("backend", "unknown")
            _last_health[backend] = data
            logger.info("Health updated: %s -> %s", backend, data.get("status", ""))
        except Exception as e:
            logger.warning("Health decode: %s", e)

    await nc.subscribe("snapshot.>", cb=on_snapshot)
    await nc.subscribe("system.health", cb=on_health)
    logger.info("Subscribed to snapshot.> and system.health")

    if serve_port and serve_port > 0:
        try:
            from aiohttp import web
        except ImportError:
            logger.warning("aiohttp not installed - skipping HTTP serve. pip install aiohttp")
        else:
            def handle_snapshot(request):
                backend = request.match_info.get("backend")
                if backend:
                    payload = _last_snapshot.get(backend)
                else:
                    payload = next(iter(_last_snapshot.values()), None) if _last_snapshot else None
                if payload is None:
                    return web.json_response(
                        {"error": "no snapshot yet", "backends": list(_last_snapshot.keys())}, status=404
                    )
                return web.json_response(payload)

            def handle_health(request):
                return web.json_response({"snapshot_backends": list(_last_snapshot.keys()), "health": _last_health})

            app = web.Application()
            app.router.add_get("/api/snapshot", handle_snapshot)
            app.router.add_get("/api/snapshot/{backend}", handle_snapshot)
            app.router.add_get("/api/health", handle_health)
            runner = web.AppRunner(app)
            await runner.setup()
            site = web.TCPSite(runner, "127.0.0.1", serve_port)
            await site.start()
            logger.info("Serving latest snapshot at http://127.0.0.1:%s/api/snapshot", serve_port)

    while True:
        await asyncio.sleep(60)


def main() -> None:
    parser = argparse.ArgumentParser(description="NATS snapshot/health subscriber (proof-of-concept)")
    parser.add_argument("--nats-url", default=os.environ.get("NATS_URL", "nats://localhost:4222"), help="NATS server URL")
    parser.add_argument("--serve-port", type=int, default=0, help="If > 0, serve GET /api/snapshot and /api/health on this port")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose logging")
    args = parser.parse_args()
    logging.basicConfig(level=logging.DEBUG if args.verbose else logging.INFO)
    asyncio.run(run_subscriber(args.nats_url, args.serve_port if args.serve_port > 0 else None))


if __name__ == "__main__":
    main()
