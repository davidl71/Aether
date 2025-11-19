"""
ib_service.py - FastAPI service exposing broker-agnostic snapshot for TUI and PWA using IB Client Portal API

Endpoints:
- GET /api/health
- GET /api/v1/snapshot

Environment:
- SYMBOLS: comma-separated underlyings (default: SPY,QQQ)
- IB_PORTAL_URL: IB Client Portal base URL (default: https://localhost:5000/v1/portal)
- SNAPSHOT_FILE_PATH: optional path to also write snapshot JSON (for TUI file polling)
"""
from __future__ import annotations

import json
import os
from datetime import datetime, timezone
from typing import Dict, List, Any, Optional

from fastapi import FastAPI, Response
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

from .ibkr_portal_client import IBKRPortalClient, IBKRPortalError


class ModeRequest(BaseModel):
    mode: str


class AccountRequest(BaseModel):
    account_id: Optional[str] = None


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _symbols_from_env() -> List[str]:
    raw = os.getenv("SYMBOLS", "SPY,QQQ")
    return [s.strip().upper() for s in raw.split(",") if s.strip()]


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


def build_snapshot_payload(
    symbols: List[str], client: IBKRPortalClient, account_id: Optional[str] = None
) -> Dict[str, Any]:
    """Build snapshot payload matching API_CONTRACT.md format."""
    # Map to the web/src/types/snapshot.ts SnapshotPayload shape
    symbol_snapshots: List[Dict[str, Any]] = []

    for sym in symbols:
        try:
            s = client.get_snapshot(sym)
            # Calculate spread
            spread = s.get("ask", 0.0) - s.get("bid", 0.0) if s.get("ask") and s.get("bid") else 0.0
            # Use last if available, otherwise close
            last_price = s.get("last", 0.0) or s.get("close", 0.0)

            symbol_snapshots.append(
                {
                    "symbol": sym,
                    "last": float(last_price),
                    "bid": float(s.get("bid", 0.0)),
                    "ask": float(s.get("ask", 0.0)),
                    "spread": float(spread),
                    "roi": 0.0,  # IB doesn't provide ROI in snapshot
                    "maker_count": 0,
                    "taker_count": 0,
                    "volume": int(s.get("volume", 0.0)),
                    "candle": {
                        "open": float(s.get("close", 0.0)),  # Use close as open if no history
                        "high": float(s.get("last", 0.0) or s.get("close", 0.0)),
                        "low": float(s.get("last", 0.0) or s.get("close", 0.0)),
                        "close": float(s.get("close", 0.0) or s.get("last", 0.0)),
                        "volume": int(s.get("volume", 0.0)),
                        "entry": float(s.get("last", 0.0) or s.get("close", 0.0)),
                        "updated": _now_iso(),
                    },
                    "option_chains": [],  # Populate via separate option chain endpoint if needed
                }
            )
        except IBKRPortalError as e:
            # Log error but continue with other symbols
            import logging
            logging.warning(f"Failed to get snapshot for {sym}: {e}")
            # Add empty snapshot for failed symbol
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

    # Get account summary
    display_account_id = "IBKR"
    account_summary = None
    try:
        account_summary = client.get_account_summary(account_id)
        accounts = client.get_accounts()
        if accounts:
            display_account_id = accounts[0]
    except IBKRPortalError as e:
        import logging
        logging.warning(f"Failed to get account summary: {e}")

    # Extract metrics from IB account summary
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

    # Get positions
    positions_data = []
    try:
        positions = client.get_portfolio_positions(account_id)
        for pos in positions:
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
    except IBKRPortalError as e:
        import logging
        logging.warning(f"Failed to get positions: {e}")

    # IB doesn't have a simple orders endpoint via Client Portal, so we'll leave it empty
    # Orders would require TWS API or more complex Client Portal integration
    orders_data: List[Dict[str, Any]] = []

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
    return payload


def create_app() -> FastAPI:
    app = FastAPI(title="IB Box Spread IB Service", version="0.1.0")
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Initialize IB Client Portal client
    portal_url = os.getenv("IB_PORTAL_URL", "https://localhost:5000/v1/portal")
    client = IBKRPortalClient(base_url=portal_url, verify_ssl=False, timeout_seconds=10)

    # Store current account in memory (in production, use Redis or database)
    current_account_id: Optional[str] = None

    @app.get("/api/health")
    def health() -> Dict[str, Any]:
        """Health check endpoint."""
        try:
            # Try to get accounts to verify connection
            accounts = client.get_accounts()
            return {
                "status": "ok",
                "ts": _now_iso(),
                "ib_connected": len(accounts) > 0,
                "accounts": accounts,
            }
        except IBKRPortalError as e:
            return {
                "status": "error",
                "ts": _now_iso(),
                "ib_connected": False,
                "error": str(e),
            }

    @app.get("/api/snapshot")
    @app.get("/api/v1/snapshot")  # Alias for API contract compatibility
    def snapshot(account_id: Optional[str] = None) -> Dict[str, Any]:
        """Get complete snapshot with market data, positions, and orders."""
        try:
            # Use query parameter if provided, otherwise use stored account
            effective_account_id = account_id if account_id else current_account_id
            symbols = _symbols_from_env()
            payload = build_snapshot_payload(symbols, client, effective_account_id)

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
    def get_positions(account_id: Optional[str] = None) -> List[Dict[str, Any]]:
        """Get all open positions."""
        try:
            effective_account_id = account_id if account_id else current_account_id
            positions = client.get_portfolio_positions(effective_account_id)
            # Format positions for frontend
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
    def list_accounts() -> Dict[str, Any]:
        """List all available IB accounts"""
        try:
            accounts = client.get_accounts()
            # Format accounts for frontend
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
                    # If summary fails, just add the account ID
                    formatted.append({"id": acc_id, "account_id": acc_id})

            return {"accounts": formatted, "ts": _now_iso()}
        except IBKRPortalError as e:
            return {"accounts": [], "error": str(e), "ts": _now_iso()}

    @app.post("/api/account")
    def set_account(request: AccountRequest) -> Dict[str, Any]:
        """Set active account ID"""
        nonlocal current_account_id
        new_account_id = request.account_id

        if new_account_id:
            # Verify account exists
            accounts = client.get_accounts()
            if new_account_id in accounts:
                current_account_id = new_account_id
                return {"status": "ok", "account_id": new_account_id, "ts": _now_iso()}
            return {"status": "error", "message": f"Account {new_account_id} not found", "ts": _now_iso()}
        else:
            # Clear account (use default)
            current_account_id = None
            return {"status": "ok", "account_id": None, "ts": _now_iso()}

    @app.get("/api/account")
    def get_account() -> Dict[str, Any]:
        """Get current active account"""
        try:
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
        except IBKRPortalError as e:
            return {"account_id": None, "error": str(e), "ts": _now_iso()}

    return app


app = create_app()
