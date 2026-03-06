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
from concurrent.futures import ThreadPoolExecutor
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
from . import nats_client


class ModeRequest(BaseModel):
    mode: str


class AccountRequest(BaseModel):
    account_id: Optional[str] = None


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _symbols_from_env() -> List[str]:
    raw = os.getenv("SYMBOLS", "SPY,QQQ")
    return [s.strip().upper() for s in raw.split(",") if s.strip()]


# Symbol sets we have already prewarmed (conid cache) once per process.
_prewarmed_symbol_keys: set = set()


def _ensure_conids_prewarmed(client: IBKRPortalClient, symbols: List[str]) -> None:
    """Call prewarm_conids once per distinct symbol set so get_snapshots_batch has warm conids."""
    key = tuple(sorted(symbols))
    if key in _prewarmed_symbol_keys:
        return
    client.prewarm_conids(symbols)
    _prewarmed_symbol_keys.add(key)


def _snapshot_cache_ttl_seconds() -> float:
    """Seconds to cache snapshot (0 = disabled)."""
    try:
        return max(0.0, float(os.getenv("SNAPSHOT_CACHE_SECONDS", "3")))
    except (ValueError, TypeError):
        return 3.0


def _extract_account_value(summary: Dict, key: str, default: float = 0.0) -> float:
    """Extract numeric value from IB account summary."""
    if not isinstance(summary, dict):
        return default

    # IB account summary has a nested structure: {"key": [{"value": "123.45", ...}]}
    items = summary.get(key, [])
    if isinstance(items, list) and len(items) > 0:
        value_str = items[0].get("value") if isinstance(items[0], dict) else None
        if value_str:
            try:
                return float(value_str)
            except (ValueError, TypeError):
                pass

    return default


def _build_cash_flow_timeline(positions: List[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
    """Generate cash flow timeline from positions for inclusion in snapshot."""
    try:
        from .cash_flow_timeline import calculate_cash_flow_timeline
    except ImportError:
        return None

    if not positions:
        return None

    try:
        result = calculate_cash_flow_timeline(positions=positions, bank_accounts=[], projection_months=12)
        return {
            "events": [
                {"date": e.date, "amount": e.amount, "description": e.description,
                 "position_name": e.position_name, "type": e.type}
                for e in result.events
            ],
            "monthly_flows": {
                month: {"month": m.month, "inflows": m.inflows, "outflows": m.outflows,
                        "net": m.net, "events": [
                            {"date": ev.date, "amount": ev.amount, "description": ev.description,
                             "position_name": ev.position_name, "type": ev.type}
                            for ev in m.events
                        ]}
                for month, m in result.monthly_flows.items()
            },
            "total_inflows": result.total_inflows,
            "total_outflows": result.total_outflows,
            "net_cash_flow": result.net_cash_flow,
        }
    except Exception:
        import logging
        logger.warning("Failed to generate cash flow timeline", exc_info=True)
        return None


def build_snapshot_payload(
    symbols: List[str], client: IBKRPortalClient, account_id: Optional[str] = None
) -> Dict[str, Any]:
    """Build snapshot payload matching API_CONTRACT.md format.
    Fetches market data, account summary, and positions in parallel to reduce latency.
    Ensures session once before the parallel block and skips per-worker ensure_session.
    """
    try:
        accounts = client.get_accounts()
        effective_account_id = account_id or (accounts[0] if accounts else None)
    except IBKRPortalError:
        effective_account_id = account_id
    display_account_id = effective_account_id if effective_account_id is not None else "IBKR"

    # Ensure session once so parallel workers do not each call ensure_session().
    client.ensure_session()
    client.set_session_ensured_for_request(True)
    try:
        # Lazy prewarm: populate conid cache so get_snapshots_batch does not do N search_contracts.
        _ensure_conids_prewarmed(client, symbols)

        def fetch_market_data() -> List[Dict]:
            try:
                return client.get_snapshots_batch(symbols)
            except IBKRPortalError:
                return [{}] * len(symbols)

        def fetch_summary() -> Optional[Dict]:
            try:
                return client.get_account_summary(effective_account_id)
            except IBKRPortalError as e:
                logger.warning("Failed to get account summary: %s", e)
                return None

        def fetch_positions() -> List[Dict]:
            try:
                return client.get_portfolio_positions(effective_account_id)
            except IBKRPortalError as e:
                logger.warning("Failed to get positions: %s", e)
                return []

        with ThreadPoolExecutor(max_workers=3) as pool:
            fut_snap = pool.submit(fetch_market_data)
            fut_summary = pool.submit(fetch_summary)
            fut_pos = pool.submit(fetch_positions)
            snapshots_list = fut_snap.result()
            account_summary = fut_summary.result()
            positions_raw = fut_pos.result()
    finally:
        client.set_session_ensured_for_request(False)

    symbol_snapshots = []

    for i, sym in enumerate(symbols):
        try:
            s = snapshots_list[i] if i < len(snapshots_list) else {}
            spread = s.get("ask", 0.0) - s.get("bid", 0.0) if s.get("ask") and s.get("bid") else 0.0
            last_price = s.get("last", 0.0) or s.get("close", 0.0)

            symbol_snapshots.append(
                {
                    "symbol": sym,
                    "last": float(last_price),
                    "bid": float(s.get("bid", 0.0)),
                    "ask": float(s.get("ask", 0.0)),
                    "spread": float(spread),
                    "roi": 0.0,
                    "maker_count": 0,
                    "taker_count": 0,
                    "volume": int(s.get("volume", 0.0)),
                    "candle": {
                        "open": float(s.get("close", 0.0)),
                        "high": float(s.get("last", 0.0) or s.get("close", 0.0)),
                        "low": float(s.get("last", 0.0) or s.get("close", 0.0)),
                        "close": float(s.get("close", 0.0) or s.get("last", 0.0)),
                        "volume": int(s.get("volume", 0.0)),
                        "entry": float(s.get("last", 0.0) or s.get("close", 0.0)),
                        "updated": _now_iso(),
                    },
                    "option_chains": [],
                }
            )
        except (KeyError, TypeError, ValueError) as e:
            import logging
            logger.warning("Failed to format snapshot for %s: %s", sym, e)
            symbol_snapshots.append(
                {
                    "symbol": sym,
                    "last": 0.0,
                    "bid": 0.0,
                    "ask": 0.0,
                    "spread": 0.0,
                    "roi": 0.0,
                    "maker_count": 0,
                    "taker_count": 0,
                    "volume": 0,
                    "candle": {
                        "open": 0.0,
                        "high": 0.0,
                        "low": 0.0,
                        "close": 0.0,
                        "volume": 0,
                        "entry": 0.0,
                        "updated": _now_iso(),
                    },
                    "option_chains": [],
                }
            )

    # Extract metrics from IB account summary (account_summary from parallel fetch)
    # IB account summary format: {"NetLiquidation": [{"value": "100523.45", ...}], ...}
    metrics = {
        "net_liq": _extract_account_value(account_summary, "NetLiquidation") if account_summary else 0.0,
        "buying_power": _extract_account_value(account_summary, "BuyingPower") if account_summary else 0.0,
        "excess_liquidity": _extract_account_value(account_summary, "ExcessLiquidity") if account_summary else 0.0,
        "margin_requirement": _extract_account_value(account_summary, "MaintMarginReq") if account_summary else 0.0,
        "commissions": 0.0,  # IB doesn't provide commission history in summary
        "portal_ok": account_summary is not None,
        "tws_ok": False,  # Client Portal is separate from TWS
        "orats_ok": False,
        "questdb_ok": False,
    }

    # Build positions from parallel fetch result
    positions_data = []
    for pos in positions_raw:
        if isinstance(pos, dict):
            positions_data.append(
                {
                    "symbol": pos.get("ticker", ""),
                    "quantity": int(float(pos.get("position", 0.0))),
                    "avg_price": float(pos.get("averageCost", 0.0)),
                    "current_price": float(pos.get("markPrice", 0.0) or pos.get("lastPrice", 0.0)),
                    "market_value": float(pos.get("markValue", 0.0) or 0.0),
                    "unrealized_pl": float(pos.get("unrealizedPnl", 0.0)),
                }
            )

    # IB doesn't have a simple orders endpoint via Client Portal, so we'll leave it empty
    # Orders would require TWS API or more complex Client Portal integration
    orders_data: List[Dict[str, Any]] = []

    # Generate cash flow timeline from positions
    cash_flow_timeline = _build_cash_flow_timeline(positions_data)

    payload: Dict[str, Any] = {
        "generated_at": _now_iso(),
        "mode": "LIVE",  # IB Client Portal is always live trading
        "strategy": "box_spread",
        "account_id": display_account_id,
        "metrics": metrics,
        "symbols": symbol_snapshots,
        "positions": positions_data,
        "historic": [],
        "orders": orders_data,
        "decisions": [],
        "alerts": [],
        "risk": {
            "allowed": True,
            "reason": None,
            "updated_at": _now_iso(),
        },
    }
    if cash_flow_timeline:
        payload["cash_flow_timeline"] = cash_flow_timeline
    return payload


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
        if error is not None:
            out["error"] = str(error)
        if accounts is not None:
            out["accounts"] = list(accounts) if accounts else []
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
                except IBKRPortalError as e:
                    state["status"] = "error"
                    state["ib_connected"] = False
                    state["gateway_logged_in"] = False
                    state["ts"] = _now_iso()
                    state["error"] = str(e)
                    state["accounts"] = []
                except Exception as e:
                    logger.debug("Background health refresh failed: %s", e)
                    state["status"] = "error"
                    state["ib_connected"] = False
                    state["gateway_logged_in"] = False
                    state["ts"] = _now_iso()
                    state["error"] = str(e)
                    state["accounts"] = []
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

    # Store current account in memory (in production, use Redis or database)
    current_account_id: Optional[str] = None
    # Snapshot cache: key (symbols_tuple, account_id) -> {"payload": dict, "cached_at": datetime}
    _snapshot_cache: Dict[Tuple[Tuple[str, ...], str], Dict[str, Any]] = {}

    @app.get("/api/health")
    async def health() -> Dict[str, Any]:
        """Health check endpoint. Returns 200 immediately from cached state; backend connection runs in background."""
        state = app.state.connection_state
        return dict(state)

    @app.get("/api/snapshot")
    @app.get("/api/v1/snapshot")  # Alias for API contract compatibility
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
                    "orats_ok": False,
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
                    formatted.append(
                        {
                            "symbol": pos.get("ticker", ""),
                            "quantity": float(pos.get("position", 0.0)),
                            "avg_price": float(pos.get("averageCost", 0.0)),
                            "current_price": float(pos.get("markPrice", 0.0) or pos.get("lastPrice", 0.0)),
                            "market_value": float(pos.get("markValue", 0.0)),
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
