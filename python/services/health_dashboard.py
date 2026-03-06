"""
Health Dashboard - Unified health JSON from NATS system.health.

Subscribes to NATS subject `system.health`; backends publish their health there.
Serves aggregated JSON for services/dashboards to consume.

Endpoints:
- GET /api/health - Aggregated health: { "backends": { "ib": {...}, "alpaca": {...} }, "generated_at": "...", "source": "nats" }
- GET /api/health/dashboard - Same with optional summary (all_ok, any_error, backends_list)
- GET /api/health/{backend_id} - Single backend health or 404
- GET /api/config - Shared config slice (services, broker) so PWA can use same config as TUI (single config file/schema usage)

Environment:
- NATS_URL - NATS server (default nats://localhost:4222). Required for updates.
- HEALTH_DASHBOARD_PORT - Port to bind (default 8010).
"""

from __future__ import annotations

import asyncio
import json
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
            data = json.loads(msg.data.decode("utf-8"))
            backend = data.get("backend", "unknown")
            _backends[backend] = {**data, "updated_at": datetime.now(timezone.utc).isoformat()}
            logger.debug("Health update: %s -> %s", backend, data.get("status", ""))
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


@app.get("/api/config")
def get_config() -> Dict[str, Any]:
    """Shared config slice (services, broker, pwa) from same home config as TUI. PWA can fetch this to use same services and broker.priorities."""
    try:
        from python.integration.shared_config_loader import SharedConfigLoader
        config = SharedConfigLoader.load_config(quiet_placeholder_warnings=True)
        out = {
            "version": config.version,
            "services": config.services or {},
            "broker": {
                "primary": config.broker.primary if config.broker else "ALPACA",
                "priorities": list(config.broker.priorities) if config.broker else ["alpaca", "ib", "mock"],
            },
            "pwa": {},
        }
        if config.pwa:
            out["pwa"] = {
                "servicePorts": config.pwa.service_ports,
                "defaultService": config.pwa.default_service,
                "serviceUrls": config.pwa.service_urls,
            }
        return out
    except Exception as e:
        logger.warning("Failed to load shared config for /api/config: %s", e)
        raise HTTPException(status_code=503, detail="Config not available") from e


def _port() -> int:
    return int(os.environ.get("HEALTH_DASHBOARD_PORT", "8010"))


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=_port())
