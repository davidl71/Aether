"""
Health Dashboard - Unified health from NATS system.health (protobuf).

Subscribes to NATS subject `system.health`; backends publish BackendHealth (protobuf) there.
Serves aggregated JSON for services/dashboards to consume.

Endpoints:
- GET /api/health - Aggregated health: { "backends": { "ib": {...}, ... }, "source": "nats" }
- GET /api/health/dashboard - Same with optional summary
- GET /api/health/{backend_id} - Single backend health or 404

Environment:
- NATS_URL - NATS server (default nats://localhost:4222). Required for updates.
- HEALTH_DASHBOARD_PORT - Port to bind (default 8010).
"""

from __future__ import annotations

import asyncio
import logging
import os
import sys
from contextlib import asynccontextmanager
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware

project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.services.security_integration_helper import (
    add_security_to_app,
    add_security_headers_middleware,
)

logger = logging.getLogger(__name__)

# In-memory: last health per backend (updated by NATS subscriber)
_backends: Dict[str, Dict[str, Any]] = {}
_nats_connected = False

try:
    import nats
    from nats.aio.client import Client as NATS
    NATS_AVAILABLE = True
except ImportError:
    NATS = None
    NATS_AVAILABLE = False

try:
    from python.generated.ib.platform import v1 as pb_v1
    from python.tui.providers.proto_snapshot import backend_health_to_dict
    PROTO_AVAILABLE = True
except ImportError:
    pb_v1 = None
    backend_health_to_dict = None
    PROTO_AVAILABLE = False


def _parse_health_proto(data: bytes) -> Dict[str, Any] | None:
    """Deserialize system.health message as BackendHealth protobuf (or NatsEnvelope)."""
    if not PROTO_AVAILABLE or pb_v1 is None or backend_health_to_dict is None:
        return None
    try:
        envelope = pb_v1.NatsEnvelope().parse(data)
        payload_bytes = getattr(envelope, "payload", None)
        if payload_bytes:
            health = pb_v1.BackendHealth().parse(payload_bytes)
            return backend_health_to_dict(health)
        return None
    except Exception:
        try:
            health = pb_v1.BackendHealth().parse(data)
            return backend_health_to_dict(health)
        except Exception:
            return None


async def _nats_subscriber() -> None:
    """Background task: connect to NATS, subscribe to system.health, update _backends."""
    global _nats_connected
    url = os.environ.get("NATS_URL", "nats://localhost:4222").strip()
    if not url or not NATS_AVAILABLE:
        logger.warning("Health dashboard: NATS_URL not set or nats-py not installed; no live updates")
        return
    nc = NATS()
    try:
        await nc.connect(
            servers=[url],
            reconnect_time_wait=2,
            max_reconnect_attempts=-1,
        )
        _nats_connected = True
        logger.info("Health dashboard connected to NATS at %s", url)
    except Exception as e:
        logger.warning("Health dashboard NATS connect failed: %s", e)
        return

    async def on_health(msg):
        try:
            data = msg.data
            if isinstance(data, str):
                data = data.encode("utf-8")
            parsed = _parse_health_proto(data)
            if parsed is not None:
                backend = parsed.get("backend", "unknown")
                _backends[backend] = {**parsed, "updated_at": datetime.now(timezone.utc).isoformat()}
                logger.debug("Health update: %s -> %s", backend, parsed.get("status", ""))
            else:
                logger.debug("Health message: protobuf parse failed or proto unavailable")
        except Exception as e:
            logger.warning("Health message decode: %s", e)

    await nc.subscribe("system.health", cb=on_health)
    logger.info("Subscribed to system.health")

    while True:
        await asyncio.sleep(60)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Start NATS subscriber in background."""
    task = asyncio.create_task(_nats_subscriber())
    yield
    task.cancel()
    try:
        await task
    except asyncio.CancelledError:
        pass


app = FastAPI(
    title="Health Dashboard",
    description="Unified health JSON from NATS system.health for services and dashboards",
    version="1.0.0",
    lifespan=lifespan,
)

add_security_to_app(app, project_root=project_root)
add_security_headers_middleware(app)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


def _aggregated() -> Dict[str, Any]:
    """Build aggregated health response."""
    statuses = [b.get("status", "unknown") for b in _backends.values()]
    all_ok = all(s == "ok" for s in statuses) if statuses else False
    any_error = any(s in ("error", "disabled") for s in statuses)
    return {
        "status": "ok" if all_ok else ("error" if any_error else "degraded"),
        "source": "nats" if _nats_connected else "none",
        "nats_connected": _nats_connected,
        "backends": dict(_backends),
        "backends_list": list(_backends.keys()),
        "all_ok": all_ok,
        "any_error": any_error,
        "generated_at": datetime.now(timezone.utc).isoformat(),
    }


@app.get("/api/health")
def get_health() -> Dict[str, Any]:
    """Unified health JSON for services to consume. Updated via NATS system.health."""
    return _aggregated()


@app.get("/api/health/dashboard")
def get_health_dashboard() -> Dict[str, Any]:
    """Same as /api/health with explicit dashboard-friendly keys (backends, summary)."""
    return _aggregated()


@app.get("/api/health/{backend_id}")
def get_health_backend(backend_id: str) -> Dict[str, Any]:
    """Single backend health. 404 if not yet reported via NATS."""
    if backend_id not in _backends:
        raise HTTPException(status_code=404, detail=f"Backend {backend_id} not in health map (no NATS update yet)")
    return _backends[backend_id]


def _port() -> int:
    return int(os.environ.get("HEALTH_DASHBOARD_PORT", "8010"))


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=_port())
