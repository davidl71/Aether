"""
Analytics API - Unified service for calculations and risk-free rate.

Serves both in one process so callers can use one origin and the server can call
calculation and risk-free-rate logic in-process (no HTTP between them).

Endpoints:
- GET /api/health - Unified health (calculations + risk_free_rate)
- GET /health - Legacy calculations health
- POST /api/v1/cash-flow/timeline - Cash flow timeline (from calculations_api)
- POST /api/v1/opportunity-simulation/* - Opportunity simulation (from calculations_api)
- POST /api/v1/cash-flow/management - Cash management (from calculations_api)
- POST /api/extract-rate, /api/build-curve, /api/benchmarks/*, /api/compare, /api/yield-curve/comparison (from risk_free_rate_service)

Environment:
- ANALYTICS_API_PORT: Port to bind (default 8007). Also respects CALCULATIONS_API_PORT for compatibility.
"""

from __future__ import annotations

import asyncio
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.services.security_integration_helper import (
    add_security_to_app,
    add_security_headers_middleware,
)
from python.services.calculations_api import router_calculations
from python.integration.risk_free_rate_service import router_risk_free_rate

try:
    from python.integration import nats_client
except ImportError:
    nats_client = None  # type: ignore[assignment]

app = FastAPI(
    title="Analytics API",
    description="Unified API: cash flow, opportunity simulation, risk-free rate (SOFR/Treasury)",
    version="1.0.0",
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


@app.get("/api/health")
async def unified_health() -> Dict[str, Any]:
    """Unified health: both components available in-process. When NATS_URL is set, publishes to system.health."""
    result = {
        "status": "ok",
        "service": "analytics-api",
        "components": ["calculations", "risk_free_rate"],
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }
    if nats_client and os.environ.get("NATS_URL", "").strip():
        asyncio.create_task(nats_client.publish_health("analytics", result))
    return result


app.include_router(router_calculations)
app.include_router(router_risk_free_rate)


def _port() -> int:
    return int(
        os.getenv("ANALYTICS_API_PORT")
        or os.getenv("CALCULATIONS_API_PORT")
        or "8007"
    )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=_port())
