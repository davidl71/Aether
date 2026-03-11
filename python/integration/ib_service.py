"""
ib_service.py - FastAPI service for IB transport/session/account access via Client Portal API.

Public shared snapshot and health routes are owned by the Rust backend. This service remains
as a narrower IB-specific adapter for the remaining transport-facing endpoints.

Environment:
- SYMBOLS: comma-separated underlyings (default: SPY,QQQ)
- IB_PORTAL_URL: IB Client Portal base URL (default: https://localhost:5001/v1/portal)
- REAUTH_SLEEP_SECONDS: seconds to sleep after triggering Gateway reauth (default 0.5; see ibkr_portal_client).
- IB_HEALTH_REFRESH_SECONDS: seconds between background gateway health refreshes (default 3; 1–30).
  Internal connection state is refreshed in the background for status publishing.

Web responsiveness: Endpoints that perform Gateway I/O run the blocking work in a thread pool
(asyncio.to_thread) so the FastAPI event loop is not blocked; other requests can be served while
one account call is in progress. A future improvement is to use an async HTTP client
(e.g. httpx.AsyncClient) in the portal layer so many concurrent Gateway calls do not consume threads.
"""
from __future__ import annotations

import asyncio
import logging
import os
from contextlib import asynccontextmanager
from typing import Dict, List, Any, Optional

logger = logging.getLogger(__name__)

from fastapi import FastAPI
from pydantic import BaseModel
from pathlib import Path
import sys

# Add project root to path for security module
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.services.security_integration_helper import (
    add_security_to_app,
    add_security_headers_middleware
)

from .ibkr_portal_client import IBKRPortalClient, IBKRPortalError
from .ib_snapshot_builder import (
    _extract_account_value,
    _format_ibcid_display_name,
    _infer_ib_session_mode,
    _now_iso,
)
from . import nats_client


class ModeRequest(BaseModel):
    mode: str


def _symbols_from_env() -> List[str]:
    raw = os.getenv("SYMBOLS", "SPY,QQQ")
    return [s.strip().upper() for s in raw.split(",") if s.strip()]


def create_app() -> FastAPI:
    # Initialize IB Client Portal client (no network in __init__)
    portal_url = os.getenv("IB_PORTAL_URL", "https://localhost:5001/v1/portal")
    client = IBKRPortalClient(base_url=portal_url, verify_ssl=False, timeout_seconds=5)
    try:
        from urllib.parse import urlparse
        gateway_port = urlparse(portal_url).port or 5001
    except Exception:
        gateway_port = 5001

    def safe_health_payload(
        status: str,
        ib_connected: bool,
        gateway_logged_in: bool,
        error: Optional[str] = None,
        accounts: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        try:
            port = int(gateway_port) if gateway_port is not None else 5001
        except (TypeError, ValueError):
            port = 5001
        out: Dict[str, Any] = {
            "status": status,
            "ts": _now_iso(),
            "ib_connected": bool(ib_connected),
            "gateway_logged_in": bool(gateway_logged_in),
            "gateway_port": port,
        }
        if accounts:
            out["accounts"] = list(accounts)
            out["session_mode"] = _infer_ib_session_mode(accounts[0])
        elif accounts is not None:
            out["accounts"] = []
        if error is not None:
            out["error"] = str(error)
        return out

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        """Start server immediately; refresh backend connection in background."""
        state = app.state.connection_state
        ib_client = app.state.ib_client
        refresh_interval = max(1.0, min(30.0, float(os.getenv("IB_HEALTH_REFRESH_SECONDS", "3"))))

        async def refresh_connection() -> None:
            while True:
                try:
                    accounts = await asyncio.to_thread(ib_client.get_accounts, 1)
                    state["status"] = "ok"
                    state["ib_connected"] = len(accounts) > 0
                    state["gateway_logged_in"] = len(accounts) > 0
                    state["ts"] = _now_iso()
                    state["error"] = None
                    state["accounts"] = list(accounts) if accounts else []
                    state["session_mode"] = _infer_ib_session_mode(accounts[0]) if accounts else "LIVE"
                except IBKRPortalError as e:
                    state["status"] = "error"
                    state["ib_connected"] = False
                    state["gateway_logged_in"] = False
                    state["ts"] = _now_iso()
                    state["error"] = str(e)
                    state["accounts"] = []
                    state["session_mode"] = "LIVE"
                except Exception as e:
                    logger.debug("Background health refresh failed: %s", e)
                    state["status"] = "error"
                    state["ib_connected"] = False
                    state["gateway_logged_in"] = False
                    state["ts"] = _now_iso()
                    state["error"] = str(e)
                    state["accounts"] = []
                    state["session_mode"] = "LIVE"
                if os.environ.get("NATS_URL", "").strip():
                    asyncio.create_task(nats_client.publish_health("ib", dict(state)))
                await asyncio.sleep(refresh_interval)

        task = asyncio.create_task(refresh_connection())
        try:
            yield
        finally:
            task.cancel()
            try:
                await task
            except asyncio.CancelledError:
                pass

    app = FastAPI(title="IB Box Spread IB Service", version="0.1.0", lifespan=lifespan)

    # Add security components
    add_security_to_app(app, project_root=project_root)
    add_security_headers_middleware(app)

    # In-memory state: health returns this immediately (no gateway call in request path)
    app.state.ib_client = client
    app.state.connection_state = dict(safe_health_payload(
        status="starting",
        ib_connected=False,
        gateway_logged_in=False,
        error=None,
        accounts=[],
    ))
    if "session_mode" not in app.state.connection_state:
        app.state.connection_state["session_mode"] = "LIVE"

    @app.get("/api/positions")
    async def get_positions(account_id: Optional[str] = None) -> List[Dict[str, Any]]:
        """Get all open positions. Blocking Gateway call runs in thread pool."""
        try:
            positions = await asyncio.to_thread(client.get_portfolio_positions, account_id)
            formatted = []
            for pos in positions:
                if isinstance(pos, dict):
                    name = (pos.get("ticker") or pos.get("symbol") or pos.get("contractDesc") or "").strip() or str(pos.get("conid", ""))
                    conid_val = pos.get("conid")
                    try:
                        conid_val = int(conid_val) if conid_val is not None else None
                    except (ValueError, TypeError):
                        conid_val = None
                    asset_class = (pos.get("assetClass") or "").strip().upper()
                    maturity_date_str = (pos.get("maturityDate") or pos.get("maturity_date") or "").strip() or None
                    if maturity_date_str and isinstance(maturity_date_str, (int, float)):
                        maturity_date_str = str(int(maturity_date_str))
                    name = _format_ibcid_display_name(
                        name if name else str(conid_val or ""), asset_class, conid_val, maturity_date_str
                    )
                    formatted.append(
                        {
                            "symbol": name,
                            "name": name,
                            "conid": conid_val,
                            "quantity": float(pos.get("position", 0.0)),
                            "avg_price": float(pos.get("avgCost", 0.0) or pos.get("averageCost", 0.0)),
                            "current_price": float(pos.get("markPrice", 0.0) or pos.get("lastPrice", 0.0)),
                            "market_value": float(pos.get("mktValue", 0.0) or pos.get("markValue", 0.0)),
                            "unrealized_pl": float(pos.get("unrealizedPnl", 0.0)),
                        }
                    )
            return formatted
        except IBKRPortalError as e:
            return [{"error": str(e)}]

    return app


app = create_app()
