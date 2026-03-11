"""
ib_service.py - FastAPI service exposing broker-agnostic snapshot for TUI and PWA using IB Client Portal API

Endpoints:
- GET /api/health
- GET /api/v1/snapshot

Environment:
- SYMBOLS: comma-separated underlyings (default: SPY,QQQ)
- IB_PORTAL_URL: IB Client Portal base URL (default: https://localhost:5001/v1/portal)
- SNAPSHOT_FILE_PATH: optional path to also write snapshot JSON (for TUI file polling)
- SNAPSHOT_CACHE_SECONDS: seconds to cache snapshot response (default: 3; 0 = disable). Increase to 5
  to reduce Gateway load when freshness is less critical.
- REAUTH_SLEEP_SECONDS: seconds to sleep after triggering Gateway reauth (default 0.5; see ibkr_portal_client).
- IB_HEALTH_REFRESH_SECONDS: seconds between background gateway health refreshes (default 3; 1–30).
  Health endpoint returns cached state immediately so the server can be marked up before the gateway is connected.

Snapshot latency: Market data, account summary, and positions are fetched in parallel.
Session is ensured once per snapshot; conids are prewarmed on first use to avoid repeated search round-trips.

Web responsiveness: All endpoints that perform Gateway I/O run the blocking work in a thread pool
(asyncio.to_thread) so the FastAPI event loop is not blocked; other requests can be served while
one snapshot or account call is in progress. A future improvement is to use an async HTTP client
(e.g. httpx.AsyncClient) in the portal layer so many concurrent Gateway calls do not consume threads.
"""
from __future__ import annotations

import asyncio
import json
import logging
import os
from contextlib import asynccontextmanager
from datetime import datetime, timezone
from typing import Dict, List, Any, Optional, Tuple

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
    _extract_cash_by_currency_from_summary,
    _format_ibcid_display_name,
    _infer_ib_session_mode,
    _now_iso,
    build_snapshot_payload,
)
from . import nats_client


class ModeRequest(BaseModel):
    mode: str


class AccountRequest(BaseModel):
    account_id: Optional[str] = None


def _symbols_from_env() -> List[str]:
    raw = os.getenv("SYMBOLS", "SPY,QQQ")
    return [s.strip().upper() for s in raw.split(",") if s.strip()]


def _snapshot_cache_ttl_seconds() -> float:
    """Seconds to cache snapshot (0 = disabled)."""
    try:
        return max(0.0, float(os.getenv("SNAPSHOT_CACHE_SECONDS", "3")))
    except (ValueError, TypeError):
        return 3.0


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

    # Store current account in memory for now.
    current_account_id: Optional[str] = None
    # Snapshot cache: key (symbols_tuple, account_id) -> {"payload": dict, "cached_at": datetime}
    _snapshot_cache: Dict[Tuple[Tuple[str, ...], str], Dict[str, Any]] = {}

    @app.get("/api/health")
    async def health() -> Dict[str, Any]:
        """Health check endpoint. Returns 200 immediately from cached state; backend connection runs in background."""
        state = app.state.connection_state
        return dict(state)

    @app.get("/api/v1/snapshot")
    async def snapshot(
        account_id: Optional[str] = None,
        symbols: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Get complete snapshot with market data, positions, and orders.
        Optional query param: symbols=SYM1,SYM2 (e.g. symbols=MNQ,NQ for micro/nano futures).
        Responses are cached for SNAPSHOT_CACHE_SECONDS (default 3) to reduce IB API load.
        Blocking Gateway I/O runs in a thread pool so the event loop stays responsive.
        """
        try:
            effective_account_id = account_id if account_id else current_account_id
            if symbols and symbols.strip():
                symbol_list = [s.strip().upper() for s in symbols.split(",") if s.strip()]
            else:
                symbol_list = _symbols_from_env()
            cache_key: Tuple[Tuple[str, ...], str] = (
                tuple(sorted(symbol_list)),
                effective_account_id or "",
            )
            ttl = _snapshot_cache_ttl_seconds()
            if ttl > 0 and cache_key in _snapshot_cache:
                entry = _snapshot_cache[cache_key]
                age = (datetime.now(timezone.utc) - entry["cached_at"]).total_seconds()
                if age < ttl:
                    return entry["payload"]
            payload = await asyncio.to_thread(
                build_snapshot_payload, symbol_list, client, effective_account_id
            )
            if ttl > 0:
                _snapshot_cache[cache_key] = {
                    "payload": payload,
                    "cached_at": datetime.now(timezone.utc),
                }

            path = os.getenv("SNAPSHOT_FILE_PATH", "").strip()
            if path:
                try:
                    parent = os.path.dirname(path)
                    if parent:
                        os.makedirs(parent, exist_ok=True)
                    with open(path, "w", encoding="utf-8") as f:
                        json.dump(payload, f, indent=2)
                except Exception as e:
                    logger.warning("Failed to write snapshot file: %s", e)

            if os.environ.get("NATS_URL", "").strip():
                asyncio.create_task(nats_client.publish_snapshot("ib", payload))

            return payload
        except Exception as e:
            logger.error("Error building snapshot: %s", e, exc_info=True)
            # Return 200 with minimal valid payload so TUI does not see 500 / "unreachable"
            return {
                "error": str(e),
                "generated_at": _now_iso(),
                "mode": "LIVE",
                "strategy": "box_spread",
                "account_id": "IBKR",
                "metrics": {
                    "net_liq": 0.0,
                    "buying_power": 0.0,
                    "excess_liquidity": 0.0,
                    "margin_requirement": 0.0,
                    "commissions": 0.0,
                    "portal_ok": False,
                    "tws_ok": False,
                    "questdb_ok": False,
                },
                "symbols": [],
                "positions": [],
                "historic": [],
                "orders": [],
                "decisions": [],
                "alerts": [{"timestamp": _now_iso(), "text": f"Snapshot error: {e}", "severity": "error"}],
                "risk": {"allowed": True, "reason": str(e), "updated_at": _now_iso()},
            }

    @app.get("/api/positions")
    async def get_positions(account_id: Optional[str] = None) -> List[Dict[str, Any]]:
        """Get all open positions. Blocking Gateway call runs in thread pool."""
        try:
            effective_account_id = account_id if account_id else current_account_id
            positions = await asyncio.to_thread(
                client.get_portfolio_positions, effective_account_id
            )
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

    @app.get("/api/accounts")
    async def list_accounts() -> Dict[str, Any]:
        """List all available IB accounts. Blocking Gateway calls run in thread pool."""
        def _fetch() -> Dict[str, Any]:
            accounts = client.get_accounts()
            formatted = []
            for acc_id in accounts:
                try:
                    summary = client.get_account_summary(acc_id)
                    formatted.append({
                        "id": acc_id,
                        "account_id": acc_id,
                        "net_liquidation": _extract_account_value(summary, "NetLiquidation"),
                        "buying_power": _extract_account_value(summary, "BuyingPower"),
                        "excess_liquidity": _extract_account_value(summary, "ExcessLiquidity"),
                    })
                except IBKRPortalError:
                    formatted.append({"id": acc_id, "account_id": acc_id})
            return {"accounts": formatted, "ts": _now_iso()}

        try:
            return await asyncio.to_thread(_fetch)
        except IBKRPortalError as e:
            return {"accounts": [], "error": str(e), "ts": _now_iso()}

    @app.post("/api/account")
    async def set_account(request: AccountRequest) -> Dict[str, Any]:
        """Set active account ID. Blocking Gateway call runs in thread pool."""
        nonlocal current_account_id
        new_account_id = request.account_id

        if new_account_id:
            def _check() -> Dict[str, Any]:
                accounts = client.get_accounts()
                if new_account_id in accounts:
                    return {"ok": True, "account_id": new_account_id}
                return {"ok": False, "message": f"Account {new_account_id} not found"}
            result = await asyncio.to_thread(_check)
            if result["ok"]:
                current_account_id = result["account_id"]
                return {"status": "ok", "account_id": current_account_id, "ts": _now_iso()}
            return {"status": "error", "message": result["message"], "ts": _now_iso()}
        else:
            current_account_id = None
            return {"status": "ok", "account_id": None, "ts": _now_iso()}

    @app.get("/api/account")
    async def get_account() -> Dict[str, Any]:
        """Get current active account. Blocking Gateway calls run in thread pool."""
        def _fetch() -> Dict[str, Any]:
            accounts = client.get_accounts()
            account_id = current_account_id or (accounts[0] if accounts else None)
            if account_id:
                summary = client.get_account_summary(account_id)
                return {
                    "account_id": account_id,
                    "net_liquidation": _extract_account_value(summary, "NetLiquidation"),
                    "buying_power": _extract_account_value(summary, "BuyingPower"),
                    "excess_liquidity": _extract_account_value(summary, "ExcessLiquidity"),
                    "margin_requirement": _extract_account_value(summary, "MaintMarginReq"),
                    "ts": _now_iso(),
                }
            return {"account_id": None, "ts": _now_iso()}

        try:
            return await asyncio.to_thread(_fetch)
        except IBKRPortalError as e:
            return {"account_id": None, "error": str(e), "ts": _now_iso()}

    return app


app = create_app()
