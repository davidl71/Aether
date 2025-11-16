"""
alpaca_client.py - Minimal Alpaca Market Data v2 REST client (broker-agnostic feed)

Reads API credentials from environment variables:
- ALPACA_API_KEY_ID
-.ALPACA_API_SECRET_KEY
- ALPACA_BASE_URL (optional, defaults to https://paper-api.alpaca.markets)
- ALPACA_DATA_BASE_URL (optional, defaults to https://data.alpaca.markets)
- ALPACA_PAPER (optional, "1"/"true" uses paper endpoints)
"""
from __future__ import annotations

import os
import time
from typing import Dict, Optional

import requests


class AlpacaClient:
    def __init__(
        self,
        api_key_id: Optional[str] = None,
        api_secret_key: Optional[str] = None,
        base_url: Optional[str] = None,
        data_base_url: Optional[str] = None,
        session: Optional[requests.Session] = None,
    ) -> None:
        self.api_key_id = api_key_id or os.getenv("ALPACA_API_KEY_ID", "")
        self.api_secret_key = api_secret_key or os.getenv("ALPACA_API_SECRET_KEY", "")
        if not self.api_key_id or not self.api_secret_key:
            raise RuntimeError("Missing Alpaca credentials (ALPACA_API_KEY_ID/ALPACA_API_SECRET_KEY)")

        paper = os.getenv("ALPACA_PAPER", "1").lower() in {"1", "true", "yes", "on"}
        default_trading_base = "https://paper-api.alpaca.markets" if paper else "https://api.alpaca.markets"
        default_data_base = "https://data.alpaca.markets"

        self.base_url = base_url or os.getenv("ALPACA_BASE_URL", default_trading_base)
        self.data_base_url = data_base_url or os.getenv("ALPACA_DATA_BASE_URL", default_data_base)
        self._session = session or requests.Session()
        self._session.headers.update(
            {
                "APCA-API-KEY-ID": self.api_key_id,
                "APCA-API-SECRET-KEY": self.api_secret_key,
                "Accept": "application/json",
            }
        )

    def _get(self, url: str, params: Optional[Dict] = None) -> Dict:
        resp = self._session.get(url, params=params or {}, timeout=10)
        resp.raise_for_status()
        return resp.json()

    def get_latest_quote(self, symbol: str) -> Optional[Dict]:
        """
        Get the latest NBBO quote for a symbol.
        Returns dict with fields: ask_price, bid_price, ask_size, bid_size, timestamp
        """
        url = f"{self.data_base_url}/v2/stocks/{symbol}/quotes/latest"
        try:
            data = self._get(url)
        except requests.HTTPError:
            return None
        quote = data.get("quote") or {}
        if not quote:
            return None
        return {
            "ask_price": float(quote.get("ap") or 0.0),
            "bid_price": float(quote.get("bp") or 0.0),
            "ask_size": int(quote.get("as") or 0),
            "bid_size": int(quote.get("bs") or 0),
            "timestamp": quote.get("t") or "",
        }

    def get_latest_trade(self, symbol: str) -> Optional[Dict]:
        """
        Get the latest trade price for a symbol.
        Returns dict with fields: price, size, timestamp
        """
        url = f"{self.data_base_url}/v2/stocks/{symbol}/trades/latest"
        try:
            data = self._get(url)
        except requests.HTTPError:
            return None
        trade = data.get("trade") or {}
        if not trade:
            return None
        return {
            "price": float(trade.get("p") or 0.0),
            "size": int(trade.get("s") or 0),
            "timestamp": trade.get("t") or "",
        }

    def get_snapshot(self, symbol: str) -> Dict:
        """
        Convenience: combine latest trade and quote into a simple snapshot
        """
        quote = self.get_latest_quote(symbol) or {}
        trade = self.get_latest_trade(symbol) or {}
        bid = float(quote.get("bid_price") or 0.0)
        ask = float(quote.get("ask_price") or 0.0)
        last = float(trade.get("price") or 0.0) or (bid + ask) / 2.0 if (bid and ask) else (bid or ask)
        spread = (ask - bid) if (ask and bid) else 0.0
        return {
            "symbol": symbol,
            "last": last or 0.0,
            "bid": bid,
            "ask": ask,
            "spread": spread,
            "bid_size": int(quote.get("bid_size") or 0),
            "ask_size": int(quote.get("ask_size") or 0),
            "quote_ts": quote.get("timestamp") or "",
            "trade_ts": trade.get("timestamp") or "",
            "received_at": int(time.time()),
        }
