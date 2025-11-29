"""
tastytrade_service.py - FastAPI service exposing Tastytrade broker-agnostic snapshot for TUI and PWA

Endpoints:
- GET /api/health
- GET /api/snapshot
- GET /api/v1/snapshot
- GET /api/positions
- GET /api/accounts
- POST /api/account
- GET /api/account

Environment:
- SYMBOLS: comma-separated underlyings (default: SPY,QQQ)
- SNAPSHOT_FILE_PATH: optional path to also write snapshot JSON (for TUI file polling)
"""
from __future__ import annotations

import json
import os
from datetime import datetime, timezone, timedelta
from typing import Dict, List, Any, Optional

from fastapi import FastAPI, Response, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import asyncio
from pathlib import Path
import sys

# Add project root to path for security module
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.services.security_integration_helper import (
    add_security_to_app,
    add_security_headers_middleware
)

from .tastytrade_client import TastytradeClient, TastytradeError
from .config_loader import get_service_port

# Optional DXLink import
try:
    from .dxlink_client import DXLinkClient, DXLinkError
    DXLINK_AVAILABLE = True
except ImportError:
    DXLINK_AVAILABLE = False
    DXLinkClient = None
    DXLinkError = None


class AccountRequest(BaseModel):
    account_id: Optional[str] = None


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _symbols_from_env() -> List[str]:
    raw = os.getenv("SYMBOLS", "SPY,QQQ")
    return [s.strip().upper() for s in raw.split(",") if s.strip()]


def _extract_account_value(summary: Dict, key: str, default: float = 0.0) -> float:
    """Extract numeric value from Tastytrade account summary."""
    if not isinstance(summary, dict):
        return default

    # Tastytrade account summary structure may vary
    value = summary.get(key)
    if value is not None:
        try:
            return float(value)
        except (ValueError, TypeError):
            pass

    # Try nested structures
    if "account" in summary:
        account = summary["account"]
        if isinstance(account, dict):
            value = account.get(key)
            if value is not None:
                try:
                    return float(value)
                except (ValueError, TypeError):
                    pass

    return default


def build_snapshot_payload(
    symbols: List[str], client: TastytradeClient, account_id: Optional[str] = None
) -> Dict[str, Any]:
    """Build snapshot payload matching API_CONTRACT.md format."""
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
                    "roi": 0.0,
                    "maker_count": 0,
                    "taker_count": 0,
                    "volume": int(s.get("volume", 0.0)),
                    "candle": {
                        "open": float(s.get("close", 0.0) or last_price),
                        "high": float(last_price),
                        "low": float(last_price),
                        "close": float(s.get("close", 0.0) or last_price),
                        "volume": int(s.get("volume", 0.0)),
                        "entry": float(last_price),
                        "updated": _now_iso(),
                    },
                    "option_chains": [],
                }
            )
        except TastytradeError as e:
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
    display_account_id = "TASTYTRADE"
    account_summary = None
    try:
        accounts = client.get_accounts()
        if accounts:
            # Use provided account_id or first account
            target_account = account_id or (accounts[0].get("account-number") or accounts[0].get("id") if accounts else None)
            if target_account:
                display_account_id = target_account
                account_summary = client.get_account_summary(target_account)
    except TastytradeError as e:
        import logging
        logging.warning(f"Failed to get account summary: {e}")

    # Extract metrics from Tastytrade account summary
    metrics = {
        "net_liq": _extract_account_value(account_summary, "net-liquidating-value") if account_summary else 0.0,
        "buying_power": _extract_account_value(account_summary, "buying-power") if account_summary else 0.0,
        "excess_liquidity": _extract_account_value(account_summary, "excess-liquidity") if account_summary else 0.0,
        "margin_requirement": _extract_account_value(account_summary, "margin-requirement") if account_summary else 0.0,
        "commissions": 0.0,  # Tastytrade may provide this in transaction history
        "portal_ok": account_summary is not None,
        "tws_ok": False,
        "orats_ok": False,
        "questdb_ok": False,
    }

    # Get positions
    positions_data = []
    try:
        if display_account_id and display_account_id != "TASTYTRADE":
            positions = client.get_positions(display_account_id)
            for pos in positions:
                if isinstance(pos, dict):
                    positions_data.append(
                        {
                            "symbol": pos.get("symbol", "") or pos.get("instrument-symbol", ""),
                            "quantity": int(float(pos.get("quantity", 0.0) or pos.get("position", 0.0))),
                            "avg_price": float(pos.get("average-price", 0.0) or pos.get("average-cost", 0.0)),
                            "current_price": float(pos.get("mark-price", 0.0) or pos.get("last-price", 0.0)),
                            "market_value": float(pos.get("market-value", 0.0) or pos.get("value", 0.0)),
                            "unrealized_pl": float(pos.get("unrealized-pl", 0.0) or pos.get("unrealized-pnl", 0.0)),
                        }
                    )
    except TastytradeError as e:
        import logging
        logging.warning(f"Failed to get positions: {e}")

    # Tastytrade orders would require additional endpoint
    orders_data: List[Dict[str, Any]] = []

    payload: Dict[str, Any] = {
        "generated_at": _now_iso(),
        "mode": "LIVE",  # Tastytrade is always live trading
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


async def _init_dxlink(dxlink_client: DXLinkClient, connected_websockets: List[WebSocket]) -> None:
    """Initialize DXLink connection in background."""
    import logging
    logger = logging.getLogger(__name__)
    try:
        await dxlink_client.connect()
        # Subscribe to default symbols
        symbols = _symbols_from_env()
        if symbols:
            await dxlink_client.subscribe(symbols)

        # Register callback to broadcast quotes to all connected clients
        def broadcast_quote(quote: Dict[str, Any]) -> None:
            """Broadcast quote to all connected WebSocket clients."""
            quote_message = json.dumps({
                "type": "quote",
                "data": quote,
            })
            # Send to all connected clients
            for ws in connected_websockets[:]:  # Copy list to avoid modification during iteration
                try:
                    asyncio.create_task(_send_quote_async(ws, quote))
                except Exception as e:
                    logger.warning(f"Failed to queue quote for client: {e}")

        dxlink_client.on_quote(broadcast_quote)
        logger.info("DXLink client initialized and connected")
    except Exception as e:
        logger.error(f"Failed to initialize DXLink: {e}")


def create_app() -> FastAPI:
    app = FastAPI(title="IB Box Spread Tastytrade Service", version="0.1.0")
    
    # Add security components
    security_components = add_security_to_app(app, project_root=project_root)
    add_security_headers_middleware(app)

    # Initialize Tastytrade client with OAuth or session-based auth
    base_url = os.getenv("TASTYTRADE_BASE_URL", "https://api.tastytrade.com")
    sandbox_base_url = os.getenv("TASTYTRADE_SANDBOX_BASE_URL", "https://api.cert.tastyworks.com")
    client_secret = os.getenv("TASTYTRADE_CLIENT_SECRET", "")
    refresh_token = os.getenv("TASTYTRADE_REFRESH_TOKEN", "")
    username = os.getenv("TASTYTRADE_USERNAME", "")
    password = os.getenv("TASTYTRADE_PASSWORD", "")

    # Determine sandbox mode
    sandbox_env = os.getenv("TASTYTRADE_SANDBOX", "").lower()
    sandbox = sandbox_env in ("true", "1", "yes", "on")

    client = TastytradeClient(
        client_secret=client_secret if client_secret else None,
        refresh_token=refresh_token if refresh_token else None,
        username=username if username else None,
        password=password if password else None,
        base_url=base_url,
        sandbox=sandbox,
        sandbox_base_url=sandbox_base_url,
    )

    # Store current account in memory (in production, use Redis or database)
    current_account_id: Optional[str] = None

    # DXLink client for real-time streaming (optional)
    dxlink_client: Optional[DXLinkClient] = None
    connected_websockets: List[WebSocket] = []
    dxlink_connected = False

    # Initialize DXLink if available (will connect on first use)
    if DXLINK_AVAILABLE:
        try:
            dxlink_client = DXLinkClient(client, sandbox=sandbox)
        except Exception as e:
            logger.warning(f"Failed to initialize DXLink client: {e}")
            dxlink_client = None

    # Startup event to initialize DXLink connection
    @app.on_event("startup")
    async def startup_event() -> None:
        """Initialize DXLink connection on service startup."""
        if DXLINK_AVAILABLE and dxlink_client:
            try:
                await _init_dxlink(dxlink_client, connected_websockets)
            except Exception as e:
                logger.error(f"Failed to connect DXLink on startup: {e}")

    # Shutdown event to cleanup DXLink connection
    @app.on_event("shutdown")
    async def shutdown_event() -> None:
        """Cleanup DXLink connection on service shutdown."""
        if dxlink_client:
            try:
                await dxlink_client.disconnect()
            except Exception as e:
                logger.warning(f"Error disconnecting DXLink: {e}")

    @app.get("/api/health")
    def health() -> Dict[str, Any]:
        """Health check endpoint."""
        try:
            # Try to get accounts to verify connection
            accounts = client.get_accounts()

            # Get OAuth token status if using OAuth
            oauth_status = None
            if client._use_oauth:
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
                "tastytrade_connected": len(accounts) > 0,
                "accounts": [acc.get("account-number") or acc.get("id") for acc in accounts] if accounts else [],
                "oauth": oauth_status,
                "sandbox": {
                    "enabled": client.sandbox,
                    "base_url": client.base_url,
                    "note": "Sandbox resets every 24h, quotes are 15-min delayed" if client.sandbox else None,
                },
            }
        except TastytradeError as e:
            return {
                "status": "error",
                "ts": _now_iso(),
                "tastytrade_connected": False,
                "error": str(e),
                "oauth": {"enabled": client._use_oauth} if hasattr(client, "_use_oauth") else None,
                "sandbox": {"enabled": client.sandbox} if hasattr(client, "sandbox") else None,
                "dxlink": {
                    "available": DXLINK_AVAILABLE,
                    "connected": dxlink_client.connected if dxlink_client else False,
                    "subscribed_symbols": list(dxlink_client.subscribed_symbols) if dxlink_client and dxlink_client.connected else [],
                } if DXLINK_AVAILABLE else {"available": False},
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
            if not effective_account_id:
                # Get first account if none specified
                accounts = client.get_accounts()
                if accounts:
                    effective_account_id = accounts[0].get("account-number") or accounts[0].get("id")

            if not effective_account_id:
                return []

            positions = client.get_positions(effective_account_id)
            # Format positions for frontend
            formatted = []
            for pos in positions:
                if isinstance(pos, dict):
                    formatted.append(
                        {
                            "symbol": pos.get("symbol", "") or pos.get("instrument-symbol", ""),
                            "quantity": float(pos.get("quantity", 0.0) or pos.get("position", 0.0)),
                            "avg_price": float(pos.get("average-price", 0.0) or pos.get("average-cost", 0.0)),
                            "current_price": float(pos.get("mark-price", 0.0) or pos.get("last-price", 0.0)),
                            "market_value": float(pos.get("market-value", 0.0) or pos.get("value", 0.0)),
                            "unrealized_pl": float(pos.get("unrealized-pl", 0.0) or pos.get("unrealized-pnl", 0.0)),
                        }
                    )
            return formatted
        except TastytradeError as e:
            return [{"error": str(e)}]

    @app.get("/api/accounts")
    def list_accounts() -> Dict[str, Any]:
        """List all available Tastytrade accounts"""
        try:
            accounts = client.get_accounts()
            # Format accounts for frontend
            formatted = []
            for acc in accounts:
                account_number = acc.get("account-number") or acc.get("id") or acc.get("account_number")
                if account_number:
                    try:
                        summary = client.get_account_summary(account_number)
                        formatted.append({
                            "id": account_number,
                            "account_id": account_number,
                            "net_liquidation": _extract_account_value(summary, "net-liquidating-value"),
                            "buying_power": _extract_account_value(summary, "buying-power"),
                            "excess_liquidity": _extract_account_value(summary, "excess-liquidity"),
                        })
                    except TastytradeError:
                        # If summary fails, just add the account ID
                        formatted.append({"id": account_number, "account_id": account_number})

            return {"accounts": formatted, "ts": _now_iso()}
        except TastytradeError as e:
            return {"accounts": [], "error": str(e), "ts": _now_iso()}

    @app.post("/api/account")
    def set_account(request: AccountRequest) -> Dict[str, Any]:
        """Set active account ID"""
        nonlocal current_account_id
        new_account_id = request.account_id

        if new_account_id:
            # Verify account exists
            accounts = client.get_accounts()
            account_numbers = [acc.get("account-number") or acc.get("id") for acc in accounts]
            if new_account_id in account_numbers:
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
            account_id = current_account_id or (accounts[0].get("account-number") or accounts[0].get("id") if accounts else None)

            if account_id:
                summary = client.get_account_summary(account_id)
                return {
                    "account_id": account_id,
                    "net_liquidation": _extract_account_value(summary, "net-liquidating-value"),
                    "buying_power": _extract_account_value(summary, "buying-power"),
                    "excess_liquidity": _extract_account_value(summary, "excess-liquidity"),
                    "margin_requirement": _extract_account_value(summary, "margin-requirement"),
                    "ts": _now_iso(),
                }
            return {"account_id": None, "ts": _now_iso()}
        except TastytradeError as e:
            return {"account_id": None, "error": str(e), "ts": _now_iso()}

    @app.websocket("/api/stream/quotes")
    async def stream_quotes(websocket: WebSocket) -> None:
        """WebSocket endpoint for real-time quote streaming via DXLink."""
        if not DXLINK_AVAILABLE or not dxlink_client:
            await websocket.close(code=1003, reason="DXLink not available")
            return

        await websocket.accept()
        connected_websockets.append(websocket)
        logger.info(f"WebSocket client connected. Total clients: {len(connected_websockets)}")

        # Quotes are already being broadcast to all clients via the callback registered in startup

        try:
            # Keep connection alive and handle client messages
            while True:
                try:
                    # Receive messages from client (subscriptions, etc.)
                    data = await websocket.receive_text()
                    try:
                        message = json.loads(data)
                        msg_type = message.get("type")

                        if msg_type == "subscribe":
                            symbols = message.get("symbols", [])
                            if symbols:
                                await dxlink_client.subscribe(symbols)
                                await websocket.send_text(json.dumps({
                                    "type": "subscribed",
                                    "symbols": symbols,
                                }))
                        elif msg_type == "unsubscribe":
                            symbols = message.get("symbols", [])
                            if symbols:
                                await dxlink_client.unsubscribe(symbols)
                                await websocket.send_text(json.dumps({
                                    "type": "unsubscribed",
                                    "symbols": symbols,
                                }))
                    except json.JSONDecodeError:
                        await websocket.send_text(json.dumps({
                            "type": "error",
                            "message": "Invalid JSON message",
                        }))
                except WebSocketDisconnect:
                    break
        except WebSocketDisconnect:
            pass
        except Exception as e:
            logger.error(f"Error in WebSocket connection: {e}")
        finally:
            # Remove from connected clients
            if websocket in connected_websockets:
                connected_websockets.remove(websocket)
            logger.info(f"WebSocket client disconnected. Total clients: {len(connected_websockets)}")

    async def _send_quote_async(websocket: WebSocket, quote: Dict[str, Any]) -> None:
        """Send quote to WebSocket client (async wrapper)."""
        try:
            await websocket.send_text(json.dumps({
                "type": "quote",
                "data": quote,
            }))
        except Exception as e:
            logger.warning(f"Failed to send quote to WebSocket client: {e}")

    @app.post("/api/auth/refresh")
    def refresh_token() -> Dict[str, Any]:
        """Manually refresh OAuth access token."""
        try:
            if not client._use_oauth:
                return {
                    "status": "error",
                    "message": "OAuth not configured. This endpoint is only available with OAuth authentication.",
                    "ts": _now_iso(),
                }

            token_data = client.refresh_access_token()
            return {
                "status": "ok",
                "message": "Token refreshed successfully",
                "expires_at": token_data.get("expires_at"),
                "ts": _now_iso(),
            }
        except TastytradeError as e:
            return {
                "status": "error",
                "message": str(e),
                "ts": _now_iso(),
            }

    return app


app = create_app()
