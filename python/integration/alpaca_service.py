"""
alpaca_service.py - FastAPI service exposing broker-agnostic snapshot for TUI and PWA

Endpoints:
- GET /api/health
- GET /api/snapshot

Environment:
- SYMBOLS: comma-separated underlyings (default: SPY,QQQ)
- SNAPSHOT_FILE_PATH: optional path to also write snapshot JSON (for TUI file polling)
"""
from __future__ import annotations

import json
import os
from datetime import datetime, timedelta, timezone
from typing import Dict, List, Any, Optional

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

from .alpaca_client import AlpacaClient


class ModeRequest(BaseModel):
    mode: str

class AccountRequest(BaseModel):
    account_id: Optional[str] = None


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _symbols_from_env() -> List[str]:
    raw = os.getenv("SYMBOLS", "SPY,QQQ")
    return [s.strip().upper() for s in raw.split(",") if s.strip()]


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
        "orats_ok": False,
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
    app = FastAPI(title="IB Box Spread Alpaca Service", version="0.1.0")
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Support both OAuth and API key authentication
    # AlpacaClient will automatically detect which method to use based on available env vars
    client = AlpacaClient()
    # Store current mode and account in memory (in production, use Redis or database)
    current_mode: str = os.getenv("ALPACA_PAPER", "1").lower() in {"1", "true", "yes", "on"} and "PAPER" or "LIVE"
    current_account_id: Optional[str] = None

    @app.get("/api/health")
    def health() -> Dict[str, Any]:
        """Health check endpoint."""
        try:
            # Try to get account to verify credentials
            account = client.get_account()
            # Get OAuth status if using OAuth
            oauth_status = None
            if hasattr(client, "_use_oauth") and client._use_oauth:
                oauth_status = {
                    "enabled": True,
                    "has_token": client._access_token is not None,
                    "expires_at": client._token_expires_at.isoformat() if client._token_expires_at else None,
                    "expires_soon": client._token_expires_at and datetime.now() >= (client._token_expires_at - timedelta(minutes=1)) if client._token_expires_at else False,
                }
            else:
                oauth_status = {"enabled": False}

            return {
                "status": "ok",
                "ts": _now_iso(),
                "alpaca_connected": account is not None,
                "account_id": account.get("account_number") if account else None,
                "oauth": oauth_status,
            }
        except Exception as e:
            return {
                "status": "error",
                "ts": _now_iso(),
                "alpaca_connected": False,
                "error": str(e),
            }

    @app.get("/api/snapshot")
    def snapshot(mode: Optional[str] = None, account_id: Optional[str] = None) -> Dict[str, Any]:
        """Get complete snapshot with market data, positions, and orders."""
        try:
            # Use query parameter if provided, otherwise use stored mode/account
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
    def get_positions() -> List[Dict[str, Any]]:
        """Get all open positions."""
        try:
            return client.get_positions()
        except Exception as e:
            return [{"error": str(e)}]

    @app.get("/api/orders")
    def get_orders(status: str = "all", limit: int = 50) -> List[Dict[str, Any]]:
        """Get orders (default: all, can filter by status: open, closed, all)."""
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
    def list_accounts() -> Dict[str, Any]:
        """List all available Alpaca accounts"""
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
    def set_account(request: AccountRequest) -> Dict[str, Any]:
        """Set active account ID"""
        nonlocal current_account_id
        new_account_id = request.account_id

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
    def get_account() -> Dict[str, Any]:
        """Get current active account"""
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
