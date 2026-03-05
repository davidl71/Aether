"""
tradestation_service.py - FastAPI service exposing TradeStation market data for TUI and PWA

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
from datetime import datetime, timezone
from typing import Dict, List, Any, Optional

from fastapi import FastAPI
from pathlib import Path
import sys

# Add project root to path for security module
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.services.security_integration_helper import (
    add_security_to_app,
    add_security_headers_middleware
)

from .tradestation_client import TradeStationClient


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _symbols_from_env() -> List[str]:
    raw = os.getenv("SYMBOLS", "SPY,QQQ")
    return [s.strip().upper() for s in raw.split(",") if s.strip()]


def build_snapshot_payload(symbols: List[str], client: Optional[TradeStationClient]) -> Dict[str, Any]:
    # Map to the web/src/types/snapshot.ts SnapshotPayload shape
    if client is None:
        return {
            "generated_at": _now_iso(),
            "mode": "SIM",
            "strategy": "box_spread",
            "account_id": "TRADESTATION",
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
            "alerts": [],
        }
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

    payload: Dict[str, Any] = {
        "generated_at": _now_iso(),
        "mode": "SIM" if os.getenv("TRADESTATION_SIM", "1").lower() in {"1", "true", "yes", "on"} else "LIVE",
        "strategy": "box_spread",
        "account_id": "TRADESTATION",
        "metrics": {
            "net_liq": 0.0,
            "buying_power": 0.0,
            "excess_liquidity": 0.0,
            "margin_requirement": 0.0,
            "commissions": 0.0,
            "portal_ok": True,
            "tws_ok": False,
            "orats_ok": False,
            "questdb_ok": False,
        },
        "symbols": symbol_snapshots,
        "positions": [],
        "historic": [],
        "orders": [],
        "alerts": [],
    }
    return payload


def create_app() -> FastAPI:
    app = FastAPI(title="IB Box Spread TradeStation Service", version="0.1.0")
    
    # Add security components
    add_security_to_app(app, project_root=project_root)
    add_security_headers_middleware(app)

    # Support start without credentials; allow disabled mode for TUI
    client: Optional[TradeStationClient] = None
    client_disabled_reason: Optional[str] = None
    try:
        client = TradeStationClient()
    except Exception as e:
        client_disabled_reason = str(e).split("\n")[0].strip() or "Missing credentials"

    @app.get("/api/health")
    def health() -> Dict[str, Any]:
        """Health check. Returns status 'disabled' when credentials are missing."""
        if client is None:
            return {
                "status": "disabled",
                "ts": _now_iso(),
                "error": client_disabled_reason or "Missing credentials",
            }
        return {"status": "ok", "ts": _now_iso()}

    @app.get("/api/snapshot")
    def snapshot() -> Dict[str, Any]:
        symbols = _symbols_from_env()
        payload = build_snapshot_payload(symbols, client)
        # Optional file write for TUI file-based polling
        path = os.getenv("SNAPSHOT_FILE_PATH", "").strip()
        if path:
            try:
                os.makedirs(os.path.dirname(path), exist_ok=True)
                with open(path, "w", encoding="utf-8") as f:
                    json.dump(payload, f, indent=2)
            except Exception:
                # Non-fatal
                pass
        return payload

    @app.get("/api/accounts")
    def list_accounts() -> Dict[str, Any]:
        """Account list for PWA/TUI compatibility. TradeStation SIM often has one account."""
        if client is None:
            return {"accounts": [], "error": client_disabled_reason or "Missing credentials", "ts": _now_iso()}
        try:
            # TradeStation API may expose account list; return stub until integrated
            return {"accounts": [], "ts": _now_iso()}
        except Exception as e:
            return {"accounts": [], "error": str(e), "ts": _now_iso()}

    return app


app = create_app()
