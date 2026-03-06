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

try:
    from .combo_detector import detect_box_spreads
except ImportError:
    detect_box_spreads = None  # type: ignore[misc, assignment]


def _infer_ib_session_mode(account_id: Optional[str]) -> str:
    """Infer Live vs Paper from IB account ID. Selection is made at Gateway login (5001).
    Paper accounts typically use 'DU' prefix; live use 'U' or numeric. Returns 'PAPER' or 'LIVE'."""
    if not account_id or not str(account_id).strip():
        return "LIVE"
    aid = str(account_id).strip().upper()
    if aid.startswith("DU"):
        return "PAPER"
    return "LIVE"


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
    """Extract numeric value from IB account summary.
    Handles both IB-style nested list format and flat/scalar format from Client Portal.
    """
    if not isinstance(summary, dict):
        return default

    # Try primary key and common aliases (Client Portal may use camelCase or different names)
    keys_to_try = [key]
    if key == "TotalCashValue":
        keys_to_try = ["TotalCashValue", "totalCashValue", "CashBalance", "cashBalance", "AvailableFunds"]
    elif key == "NetLiquidation":
        keys_to_try = ["NetLiquidation", "netLiquidation", "NetLiquidationValue"]
    elif key == "BuyingPower":
        keys_to_try = ["BuyingPower", "buyingPower"]
    elif key == "ExcessLiquidity":
        keys_to_try = ["ExcessLiquidity", "excessLiquidity"]
    elif key == "MaintMarginReq":
        keys_to_try = ["MaintMarginReq", "maintMarginReq", "MaintMarginRequirement"]

    for k in keys_to_try:
        raw = summary.get(k)
        if raw is None:
            continue
        # Nested format: {"TotalCashValue": [{"value": "123.45", ...}]}
        if isinstance(raw, list) and len(raw) > 0:
            first = raw[0]
            if isinstance(first, dict):
                value_str = first.get("value") or first.get("amount")
                if value_str is not None:
                    try:
                        return float(value_str)
                    except (ValueError, TypeError):
                        pass
            elif isinstance(first, (int, float)):
                return float(first)
        # Flat scalar
        if isinstance(raw, (int, float)):
            return float(raw)
        if isinstance(raw, str) and raw.strip():
            try:
                return float(raw)
            except (ValueError, TypeError):
                pass
    return default


def _float_or_none(val: Any) -> Optional[float]:
    """Return float(val) or None if missing/invalid."""
    if val is None:
        return None
    try:
        f = float(val)
        return f
    except (ValueError, TypeError):
        return None


def _expiry_str_to_date(expiry: str) -> str:
    """Convert IB-style expiry (e.g. MAR2027) to YYYY-MM-DD (third Friday of month)."""
    if not expiry or not isinstance(expiry, str):
        return ""
    import calendar
    s = expiry.strip().upper()
    months = {"JAN": 1, "FEB": 2, "MAR": 3, "APR": 4, "MAY": 5, "JUN": 6,
              "JUL": 7, "AUG": 8, "SEP": 9, "OCT": 10, "NOV": 11, "DEC": 12}
    for mon_name, mon_num in months.items():
        if s.startswith(mon_name):
            try:
                year = int(s[len(mon_name):])
                if year < 100:
                    year += 2000
                # Third Friday
                cal = calendar.Calendar(calendar.FRIDAY)
                fridays = [d for d in cal.itermonthdates(year, mon_num) if d.month == mon_num]
                if len(fridays) >= 3:
                    d = fridays[2]
                    return f"{d.year:04d}-{d.month:02d}-{d.day:02d}"
                # Fallback: last day of month
                _, last = calendar.monthrange(year, mon_num)
                return f"{year:04d}-{mon_num:02d}-{last:02d}"
            except (ValueError, TypeError):
                pass
            break
    return ""

def _build_cash_flow_timeline(
    positions: List[Dict[str, Any]],
    reported_future_events: Optional[List[Dict[str, Any]]] = None,
) -> Optional[Dict[str, Any]]:
    """Generate cash flow timeline from positions and reported future events (dividend, expiry, etc.)."""
    try:
        from .cash_flow_timeline import calculate_cash_flow_timeline
    except ImportError:
        return None

    if not positions and not (reported_future_events and len(reported_future_events) > 0):
        return None

    try:
        result = calculate_cash_flow_timeline(
            positions=positions,
            bank_accounts=[],
            projection_months=12,
            reported_future_events=reported_future_events or [],
        )
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
        ledger_rows: List[Dict] = []

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

        def fetch_ledger() -> List[Dict]:
            try:
                return client.get_account_ledger(effective_account_id)
            except IBKRPortalError as e:
                logger.debug("Ledger not available: %s", e)
                return []

        with ThreadPoolExecutor(max_workers=4) as pool:
            fut_snap = pool.submit(fetch_market_data)
            fut_summary = pool.submit(fetch_summary)
            fut_pos = pool.submit(fetch_positions)
            fut_ledger = pool.submit(fetch_ledger)
            snapshots_list = fut_snap.result()
            account_summary = fut_summary.result()
            positions_raw = fut_pos.result()
            ledger_rows = fut_ledger.result()
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

    # Build positions from parallel fetch result. Client Portal returns contractDesc, position,
    # mktValue, unrealizedPnl, avgCost, assetClass (and sometimes ticker/symbol).
    positions_data = []
    future_events: List[Dict[str, Any]] = []
    if detect_box_spreads:
        try:
            combos, remaining = detect_box_spreads(positions_raw)
            for c in combos:
                if c.get("type") == "box_spread":
                    name = f"Box: {c.get('underlying', '')} {c.get('expiry', '')} {c.get('k1')}/{c.get('k2')} box"
                    qty = int(c.get("quantity", 0))
                    mkt = float(c.get("mktValue", 0) or 0)
                    pnl = float(c.get("unrealizedPnl", 0) or 0)
                    side = c.get("side", "long" if qty > 0 else "short")
                    exp_cash = c.get("expected_cash_at_expiry")
                    positions_data.append({
                        "name": name,
                        "symbol": name,
                        "quantity": qty,
                        "avg_price": 0.0,
                        "current_price": mkt / qty if qty else 0.0,
                        "market_value": mkt,
                        "unrealized_pl": pnl,
                        "roi": (pnl / mkt * 100.0) if mkt else 0.0,
                        "instrument_type": "box_spread",
                        "side": side,
                        "expected_cash_at_expiry": float(exp_cash) if exp_cash is not None else None,
                        "currency": "USD",
                        "dividend": None,
                    })
                    # Reported future event: option/box expiry
                    if exp_cash is not None and qty != 0:
                        exp_date = _expiry_str_to_date(c.get("expiry") or "")
                        future_events.append({
                            "event_type": "expiry",
                            "date": exp_date,
                            "amount": float(exp_cash),
                            "currency": "USD",
                            "position_name": name,
                            "description": "Box spread expiry",
                        })
            positions_raw = remaining
        except Exception as e:
            logger.debug("Combo detection skipped: %s", e)
    for pos in positions_raw:
        if isinstance(pos, dict):
            name = (pos.get("ticker") or pos.get("symbol") or pos.get("contractDesc") or "").strip()
            if not name:
                name = str(pos.get("conid", ""))
            qty = int(float(pos.get("position", 0.0)))
            avg_cost = float(pos.get("avgCost", 0.0) or pos.get("averageCost", 0.0))
            mkt_val = float(pos.get("mktValue", 0.0) or pos.get("markValue", 0.0))
            cur = float(pos.get("markPrice", 0.0) or pos.get("lastPrice", 0.0))
            if not cur and qty:
                cur = mkt_val / qty
            pnl = float(pos.get("unrealizedPnl", 0.0))
            roi = (pnl / mkt_val * 100.0) if mkt_val else 0.0
            # Optional bid/ask/last from Client Portal (some Gateways include them)
            bid = _float_or_none(pos.get("bidPrice") or pos.get("bid"))
            ask = _float_or_none(pos.get("askPrice") or pos.get("ask"))
            last = _float_or_none(pos.get("lastPrice") or pos.get("last") or pos.get("markPrice"))
            spread = (ask - bid) if (bid is not None and ask is not None) else None
            price = last or cur
            side = "long" if qty > 0 else "short" if qty < 0 else None
            curr = (pos.get("currency") or "USD").strip() or "USD"
            # Expected cash at expiry: optional for bonds/bills (could be set when we have face/maturity)
            exp_cash = pos.get("expected_cash_at_expiry")
            # Dividend: next/expected total for position (Client Portal or enrichment may provide)
            div = _float_or_none(pos.get("dividend") or pos.get("dividendAmount") or pos.get("nextDividendAmount"))
            if div is None and pos.get("dividendPerShare") is not None and qty:
                try:
                    dps = float(pos.get("dividendPerShare", 0) or 0)
                    if dps:
                        div = dps * abs(qty)
                except (ValueError, TypeError):
                    pass
            # Maturity / principal (bonds, bills, loans)
            maturity_date_str = (pos.get("maturityDate") or pos.get("maturity_date") or "").strip()
            if isinstance(maturity_date_str, (int, float)):
                maturity_date_str = str(int(maturity_date_str))
            asset_class = (pos.get("assetClass") or "").strip().upper()
            inst_type = "bond" if asset_class == "BOND" else "t_bill" if asset_class in ("BILL", "TBILL") else None
            positions_data.append({
                "name": name,
                "symbol": name,
                "quantity": qty,
                "avg_price": avg_cost,
                "current_price": cur,
                "market_value": mkt_val,
                "unrealized_pl": pnl,
                "roi": roi,
                "bid": bid,
                "ask": ask,
                "last": last,
                "spread": spread,
                "price": price,
                "side": side,
                "currency": curr,
                "expected_cash_at_expiry": float(exp_cash) if exp_cash is not None else None,
                "dividend": div,
                "instrument_type": inst_type,
                "maturity_date": maturity_date_str[:10] if maturity_date_str else None,
            })
            # Reported future events: dividend, principal/expiry at maturity
            if div is not None and div != 0:
                future_events.append({
                    "event_type": "dividend",
                    "date": (pos.get("exDate") or pos.get("nextDividendDate") or "").strip()[:10] or "",
                    "amount": float(div),
                    "currency": curr,
                    "position_name": name,
                    "description": "Dividend",
                })
            if maturity_date_str and (exp_cash is not None or mkt_val):
                amt = float(exp_cash) if exp_cash is not None else mkt_val
                if amt != 0:
                    future_events.append({
                        "event_type": "principal_repayment",
                        "date": maturity_date_str[:10] if len(maturity_date_str) >= 10 else maturity_date_str,
                        "amount": amt,
                        "currency": curr,
                        "position_name": name,
                        "description": "Principal at maturity",
                    })

    # Cash positions: from ledger (all currencies) when available, else single USD from TotalCashValue
    if ledger_rows and isinstance(ledger_rows, list):
        for row in ledger_rows:
            if not isinstance(row, dict):
                continue
            curr = (row.get("currency") or "").strip().upper()
            bal = row.get("balance")
            if curr is None:
                continue
            if curr == "":
                curr = "USD"
            try:
                balance = float(bal) if bal is not None else 0.0
            except (ValueError, TypeError):
                balance = 0.0
            positions_data.append({
                "name": f"Cash ({curr})",
                "symbol": "Cash",
                "quantity": 0,
                "avg_price": 0.0,
                "current_price": balance,
                "market_value": balance,
                "unrealized_pl": 0.0,
                "roi": 0.0,
                "instrument_type": "cash",
                "currency": curr,
                "dividend": None,
            })
    else:
        total_cash = _extract_account_value(account_summary, "TotalCashValue") if account_summary else 0.0
        if account_summary and total_cash == 0.0:
            logger.debug(
                "Account summary present but TotalCashValue is 0; summary keys: %s. "
                "Ledger was empty (use /v1/api/portfolio/{id}/ledger if available).",
                list(account_summary.keys()) if isinstance(account_summary, dict) else type(account_summary),
            )
        positions_data.append({
            "name": "Cash (USD)",
            "symbol": "Cash",
            "quantity": 0,
            "avg_price": 0.0,
            "current_price": total_cash,
            "market_value": total_cash,
            "unrealized_pl": 0.0,
            "roi": 0.0,
            "instrument_type": "cash",
            "currency": "USD",
            "dividend": None,
        })

    # IB doesn't have a simple orders endpoint via Client Portal, so we'll leave it empty
    # Orders would require TWS API or more complex Client Portal integration
    orders_data: List[Dict[str, Any]] = []

    # Generate cash flow timeline from positions (includes reported future_events)
    cash_flow_timeline = _build_cash_flow_timeline(positions_data, future_events)

    payload: Dict[str, Any] = {
        "generated_at": _now_iso(),
        "mode": _infer_ib_session_mode(display_account_id),
        "strategy": "box_spread",
        "account_id": display_account_id,
        "metrics": metrics,
        "symbols": symbol_snapshots,
        "positions": positions_data,
        "historic": [],
        "orders": orders_data,
        "decisions": [],
        "alerts": [],
        "future_events": future_events,
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
                    name = (pos.get("ticker") or pos.get("symbol") or pos.get("contractDesc") or "").strip() or str(pos.get("conid", ""))
                    formatted.append(
                        {
                            "symbol": name,
                            "name": name,
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
