"""
alpaca_service.py - FastAPI service exposing broker-agnostic snapshot for TUI and PWA

Endpoints:
- GET /api/health
- GET /api/v1/snapshot

Environment:
- SYMBOLS: comma-separated underlyings (default: SPY,QQQ)
- SNAPSHOT_FILE_PATH: optional path to also write snapshot JSON (for TUI file polling)
- NATS_URL: when set, health (and optional snapshot) are published to NATS for unified dashboard
- ALPACA_HEALTH_REFRESH_SECONDS: seconds between background health refreshes (default 5; 1–60).
  Health returns cached state immediately so the server is up before Alpaca is connected.
"""
from __future__ import annotations

import asyncio
import json
import os
from contextlib import asynccontextmanager
from datetime import datetime, timedelta, timezone
from typing import Dict, List, Any, Optional, Tuple

from fastapi import FastAPI, Request
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

from .alpaca_client import AlpacaClient
from . import nats_client
from .shared_config_loader import load_shared_config


class ModeRequest(BaseModel):
    mode: str

class AccountRequest(BaseModel):
    account_id: Optional[str] = None


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _symbols_from_env() -> List[str]:
    raw = os.getenv("SYMBOLS", "SPY,QQQ")
    return [s.strip().upper() for s in raw.split(",") if s.strip()]


def _alpaca_client_kwargs_from_config() -> Dict[str, Any]:
    """Build AlpacaClient kwargs from shared config if present (e.g. base_url for live https://api.alpaca.markets)."""
    kwargs: Dict[str, Any] = {}
    try:
        config = load_shared_config()
        alpaca = (config.extra or {}).get("alpaca") or {}
        data_cfg = alpaca.get("data_client_config") or alpaca.get("dataClientConfig") or {}
        base_url = (data_cfg.get("base_url") or data_cfg.get("baseUrl") or "").strip()
        if base_url:
            kwargs["base_url"] = base_url
        data_base_url = (data_cfg.get("data_base_url") or data_cfg.get("dataBaseUrl") or "").strip()
        if data_base_url:
            kwargs["data_base_url"] = data_base_url
        paper = data_cfg.get("paper", True)
        os.environ["ALPACA_PAPER"] = "1" if paper else "0"
    except Exception:
        pass
    return kwargs


def build_snapshot_payload(symbols: List[str], client: AlpacaClient, mode: str = "PAPER", account_id: Optional[str] = None) -> Dict[str, Any]:
    # Map to the web/src/types/snapshot.ts SnapshotPayload shape
    symbol_snapshots: List[Dict[str, Any]] = []
    for sym in symbols:
        s = client.get_snapshot(sym)
        symbol_snapshots.append(
            {
                "symbol": sym,
                "last": float(s.get("last") or 0.0),
                "bid": float(s.get("bid") or 0.0),
                "ask": float(s.get("ask") or 0.0),
                "spread": float(s.get("spread") or 0.0),
                "roi": 0.0,
                "maker_count": 0,
                "taker_count": 0,
                "volume": int(s.get("bid_size") or 0) + int(s.get("ask_size") or 0),
                "candle": {
                    "open": float(s.get("last") or 0.0),
                    "high": float(s.get("last") or 0.0),
                    "low": float(s.get("last") or 0.0),
                    "close": float(s.get("last") or 0.0),
                    "volume": int(s.get("bid_size") or 0) + int(s.get("ask_size") or 0),
                    "entry": float(s.get("last") or 0.0),
                    "updated": _now_iso(),
                },
                "option_chains": [],  # Populate via separate option chain endpoint if needed
            }
        )

    # Normalize mode
    if mode.upper() in ("LIVE", "LIVE_TRADING", "PRODUCTION"):
        mode_display = "LIVE"
    else:
        mode_display = "PAPER"

    # Get account info if account_id is specified
    account_info = None
    display_account_id = "ALPACA"
    if account_id:
        # Try to get the specific account by matching account_number
        accounts = client.get_accounts()
        for acc in accounts:
            account_number = acc.get("account_number") or acc.get("id")
            if account_number == account_id:
                account_info = acc
                display_account_id = account_number
                break
        # If not found in list, try direct lookup
        if not account_info:
            account_info = client.get_account(account_id)
            if account_info:
                display_account_id = account_info.get("account_number") or account_info.get("id") or account_id
    else:
        # Get default account (first account from list or direct call)
        accounts = client.get_accounts()
        if accounts:
            account_info = accounts[0]
            display_account_id = account_info.get("account_number") or account_info.get("id") or "ALPACA"
        else:
            # Fallback to direct account call
            account_info = client.get_account()
            if account_info:
                display_account_id = account_info.get("account_number") or account_info.get("id") or "ALPACA"

    # Fetch positions and orders
    positions = client.get_positions()
    orders = client.get_orders(status="open", limit=20)

    # Extract account metrics if available
    metrics = {
        "net_liq": float(account_info.get("portfolio_value", 0.0)) if account_info else 0.0,
        "buying_power": float(account_info.get("buying_power", 0.0)) if account_info else 0.0,
        "excess_liquidity": float(account_info.get("excess_liquidity", 0.0)) if account_info else 0.0,
        "margin_requirement": float(account_info.get("day_trading_buying_power", 0.0)) if account_info else 0.0,
        "commissions": 0.0,  # Alpaca doesn't provide commission history in account endpoint
        "portal_ok": True,
        "tws_ok": False,
        "questdb_ok": False,
    }

    payload: Dict[str, Any] = {
        "generated_at": _now_iso(),
        "mode": mode_display,
        "strategy": "box_spread",
        "account_id": display_account_id,
        "metrics": metrics,
        "symbols": symbol_snapshots,
        "positions": [
            {
                "symbol": pos.get("symbol", ""),
                "quantity": int(float(pos.get("qty", 0))),
                "avg_price": float(pos.get("avg_entry_price", 0.0)),
                "current_price": float(pos.get("current_price", 0.0)),
                "market_value": float(pos.get("market_value", 0.0)),
                "unrealized_pl": float(pos.get("unrealized_pl", 0.0)),
            }
            for pos in positions
        ],
        "historic": [],
        "orders": [
            {
                "id": order.get("id", ""),
                "symbol": order.get("symbol", ""),
                "side": order.get("side", ""),
                "quantity": int(float(order.get("qty", 0))),
                "filled_quantity": int(float(order.get("filled_qty", 0))),
                "order_type": order.get("order_type", ""),
                "status": order.get("status", ""),
                "limit_price": float(order.get("limit_price", 0.0)) if order.get("limit_price") else None,
                "stop_price": float(order.get("stop_price", 0.0)) if order.get("stop_price") else None,
            }
            for order in orders
        ],
        "alerts": [],
    }
    return payload


def create_app() -> FastAPI:
    @asynccontextmanager
    async def lifespan(app: FastAPI):
        """Start server immediately; create client and refresh health in background."""
        state = app.state.connection_state
        refresh_interval = max(1.0, min(60.0, float(os.getenv("ALPACA_HEALTH_REFRESH_SECONDS", "5"))))

        def _try_create_client() -> Tuple[Optional[AlpacaClient], Optional[str]]:
            try:
                kwargs = _alpaca_client_kwargs_from_config()
                return AlpacaClient(**kwargs), None
            except Exception as e:
                msg = str(e).split("\n")[0].strip() or "Missing API key or OAuth credentials"
                if "credentials" not in msg.lower() and "api key" not in msg.lower():
                    msg = "Missing API key or OAuth credentials"
                return None, msg

        async def refresh_connection() -> None:
            while True:
                client = app.state.alpaca_client
                if client is None:
                    client, err = await asyncio.to_thread(_try_create_client)
                    app.state.alpaca_client = client
                    if client is None:
                        state["status"] = "disabled"
                        state["alpaca_connected"] = False
                        state["ts"] = _now_iso()
                        state["error"] = err or "Missing API key or OAuth credentials"
                else:
                    try:
                        account = await asyncio.to_thread(client.get_account)
                        oauth_status = None
                        if hasattr(client, "_use_oauth") and client._use_oauth:
                            oauth_status = {
                                "enabled": True,
                                "has_token": client._access_token is not None,
                                "expires_at": client._token_expires_at.isoformat() if client._token_expires_at else None,
                                "expires_soon": bool(client._token_expires_at and datetime.now() >= (client._token_expires_at - timedelta(minutes=1))),
                            }
                        else:
                            oauth_status = {"enabled": False}
                        state["status"] = "ok"
                        state["alpaca_connected"] = account is not None
                        state["ts"] = _now_iso()
                        state["account_id"] = account.get("account_number") if account else None
                        state["oauth"] = oauth_status
                        state["error"] = None
                    except Exception as e:
                        state["status"] = "error"
                        state["alpaca_connected"] = False
                        state["ts"] = _now_iso()
                        state["error"] = str(e)
                if os.environ.get("NATS_URL", "").strip():
                    asyncio.create_task(nats_client.publish_health("alpaca", dict(state)))
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

    app = FastAPI(title="IB Box Spread Alpaca Service", version="0.1.0", lifespan=lifespan)

    # Add security components
    add_security_to_app(app, project_root=project_root)
    add_security_headers_middleware(app)

    # Server starts immediately; client created in background
    app.state.alpaca_client = None
    app.state.connection_state = {
        "status": "starting",
        "ts": _now_iso(),
        "alpaca_connected": False,
        "error": None,
    }

    # Store current mode and account in memory for now.
    current_mode: str = os.getenv("ALPACA_PAPER", "1").lower() in {"1", "true", "yes", "on"} and "PAPER" or "LIVE"
    current_account_id: Optional[str] = None

    @app.get("/api/health")
    async def health(request: Request) -> Dict[str, Any]:
        """Health check endpoint. Returns 200 immediately from cached state; backend connection runs in background."""
        return dict(request.app.state.connection_state)

    @app.get("/api/v1/snapshot")
    def snapshot(request: Request, mode: Optional[str] = None, account_id: Optional[str] = None) -> Dict[str, Any]:
        """Get complete snapshot with market data, positions, and orders."""
        client = request.app.state.alpaca_client
        if client is None:
            return {
                "generated_at": _now_iso(),
                "mode": current_mode,
                "strategy": "DISABLED",
                "account_id": "",
                "symbols": [],
                "positions": [],
                "orders": [],
                "alerts": [],
                "error": request.app.state.connection_state.get("error") or "Missing credentials",
            }
        try:
            effective_mode = mode.upper() if mode else current_mode
            effective_account_id = account_id if account_id else current_account_id
            symbols = _symbols_from_env()
            payload = build_snapshot_payload(symbols, client, effective_mode, effective_account_id)
            # Optional file write for TUI file-based polling
            path = os.getenv("SNAPSHOT_FILE_PATH", "").strip()
            if path:
                try:
                    os.makedirs(os.path.dirname(path), exist_ok=True)
                    with open(path, "w", encoding="utf-8") as f:
                        json.dump(payload, f, indent=2)
                except Exception as e:
                    # Non-fatal, but log it
                    import logging
                    logging.warning(f"Failed to write snapshot file: {e}")
            return payload
        except Exception as e:
            import logging
            logging.error(f"Error building snapshot: {e}")
            return {
                "error": str(e),
                "generated_at": _now_iso(),
                "symbols": [],
                "positions": [],
                "orders": [],
            }

    @app.get("/api/positions")
    def get_positions(request: Request) -> List[Dict[str, Any]]:
        """Get all open positions."""
        client = request.app.state.alpaca_client
        if client is None:
            return []
        try:
            return client.get_positions()
        except Exception as e:
            return [{"error": str(e)}]

    @app.get("/api/orders")
    def get_orders(request: Request, status: str = "all", limit: int = 50) -> List[Dict[str, Any]]:
        """Get orders (default: all, can filter by status: open, closed, all)."""
        client = request.app.state.alpaca_client
        if client is None:
            return []
        try:
            return client.get_orders(status=status, limit=limit)
        except Exception as e:
            return [{"error": str(e)}]

    @app.post("/api/mode")
    def set_mode(request: ModeRequest) -> Dict[str, str]:
        """Set trading mode (PAPER or LIVE)"""
        nonlocal current_mode
        new_mode = request.mode.upper()
        if new_mode in ("PAPER", "LIVE", "LIVE_TRADING"):
            current_mode = "LIVE" if new_mode in ("LIVE", "LIVE_TRADING") else "PAPER"
            return {"status": "ok", "mode": current_mode, "ts": _now_iso()}
        return {"status": "error", "message": "Invalid mode. Use 'PAPER' or 'LIVE'", "ts": _now_iso()}

    @app.get("/api/mode")
    def get_mode() -> Dict[str, str]:
        """Get current trading mode"""
        return {"mode": current_mode, "ts": _now_iso()}

    @app.get("/api/accounts")
    def list_accounts(request: Request) -> Dict[str, Any]:
        """List all available Alpaca accounts"""
        client = request.app.state.alpaca_client
        if client is None:
            return {"accounts": [], "error": request.app.state.connection_state.get("error") or "Missing credentials", "ts": _now_iso()}
        try:
            accounts = client.get_accounts()
            # Format accounts for frontend
            formatted_accounts = []
            for acc in accounts:
                # Alpaca Trading API returns account_number as the main identifier
                # Paper accounts start with "PA", live accounts are UUIDs or other formats
                account_number = acc.get("account_number") or acc.get("id")
                account_id = acc.get("id") or acc.get("account_number")

                formatted_accounts.append({
                    "id": account_id,
                    "account_number": account_number,
                    "status": acc.get("status", "unknown"),
                    "currency": acc.get("currency", "USD"),
                    "buying_power": float(acc.get("buying_power", 0.0)),
                    "cash": float(acc.get("cash", 0.0)),
                    "portfolio_value": float(acc.get("portfolio_value", 0.0)),
                    "pattern_day_trader": acc.get("pattern_day_trader", False),
                    "trading_blocked": acc.get("trading_blocked", False),
                })
            return {"accounts": formatted_accounts, "ts": _now_iso()}
        except Exception as e:
            import traceback
            error_details = traceback.format_exc()
            return {"accounts": [], "error": str(e), "traceback": error_details, "ts": _now_iso()}

    @app.post("/api/account")
    def set_account(request: Request, body: AccountRequest) -> Dict[str, Any]:
        """Set active account ID"""
        nonlocal current_account_id
        client = request.app.state.alpaca_client
        if client is None:
            return {"status": "error", "message": request.app.state.connection_state.get("error") or "Missing credentials", "ts": _now_iso()}
        new_account_id = body.account_id

        if new_account_id:
            # For Trading API, we typically only have one account, so we can match by account_number
            # First, get all accounts to find a match
            accounts = client.get_accounts()
            matched_account = None
            for acc in accounts:
                account_number = acc.get("account_number") or acc.get("id")
                account_id = acc.get("id") or acc.get("account_number")
                # Match by account_number (e.g., PA3RWI1D1527) or id
                if account_number == new_account_id or account_id == new_account_id:
                    matched_account = acc
                    break

            if matched_account:
                # Store the account_number as the identifier
                account_number = matched_account.get("account_number") or matched_account.get("id")
                current_account_id = account_number
                return {"status": "ok", "account_id": account_number, "ts": _now_iso()}
            return {"status": "error", "message": f"Account {new_account_id} not found", "ts": _now_iso()}
        else:
            # Clear account (use default)
            current_account_id = None
            return {"status": "ok", "account_id": None, "ts": _now_iso()}

    @app.get("/api/account")
    def get_account(request: Request) -> Dict[str, Any]:
        """Get current active account"""
        client = request.app.state.alpaca_client
        if client is None:
            return {"account_id": None, "error": request.app.state.connection_state.get("error") or "Missing credentials", "ts": _now_iso()}
        account_info = None
        if current_account_id:
            account_info = client.get_account(current_account_id)
        else:
            account_info = client.get_account()

        if account_info:
            return {
                "account_id": account_info.get("account_number") or account_info.get("id"),
                "status": account_info.get("status"),
                "currency": account_info.get("currency", "USD"),
                "buying_power": float(account_info.get("buying_power", 0.0)),
                "cash": float(account_info.get("cash", 0.0)),
                "portfolio_value": float(account_info.get("portfolio_value", 0.0)),
                "ts": _now_iso()
            }
        return {"account_id": None, "ts": _now_iso()}

    return app


app = create_app()
