"""Broker position fetching for IBKR, Alpaca, and TradeStation."""

from __future__ import annotations

from typing import Any, Dict, List, Optional


def fetch_ibkr_positions(account_id: Optional[str] = None) -> List[Dict[str, Any]]:
    """Fetch positions from IBKR Client Portal."""
    try:
        from .ibkr_portal_client import IBKRPortalClient

        client = IBKRPortalClient()
        positions = client.get_portfolio_positions(account_id)

        return [
            {
                "symbol": pos.get("ticker", ""),
                "quantity": float(pos.get("position", 0.0)),
                "avg_price": float(pos.get("averageCost", 0.0)),
                "current_price": float(pos.get("markPrice", 0.0) or pos.get("lastPrice", 0.0)),
                "market_value": float(pos.get("markValue", 0.0)),
                "unrealized_pl": float(pos.get("unrealizedPnl", 0.0)),
                "currency": pos.get("currency", "USD"),
            }
            for pos in positions
            if isinstance(pos, dict)
        ]
    except Exception:
        return []


def fetch_alpaca_positions() -> List[Dict[str, Any]]:
    """Fetch positions from Alpaca API."""
    try:
        from .alpaca_client import AlpacaClient

        client = AlpacaClient()
        positions = client.get_positions()

        return [
            {
                "symbol": pos.get("symbol", ""),
                "quantity": float(pos.get("qty", 0.0)),
                "avg_price": float(pos.get("avg_entry_price", 0.0)),
                "current_price": float(pos.get("current_price", 0.0)),
                "market_value": float(pos.get("market_value", 0.0)),
                "unrealized_pl": float(pos.get("unrealized_pl", 0.0)),
                "currency": pos.get("currency", "USD"),
            }
            for pos in positions
            if isinstance(pos, dict)
        ]
    except Exception:
        return []


def fetch_tradestation_positions(
    account_id: Optional[str] = None,
) -> List[Dict[str, Any]]:
    """Fetch positions from TradeStation API."""
    try:
        from .tradestation_client import TradeStationClient

        client = TradeStationClient()
        return client.get_positions(account_id)
    except Exception:
        return []
