"""
alpaca_client.py - Alpaca client using official alpaca-py SDK

Reads API credentials from environment variables:
- ALPACA_API_KEY_ID
- ALPACA_API_SECRET_KEY
- ALPACA_BASE_URL (optional, defaults to https://paper-api.alpaca.markets)
- ALPACA_DATA_BASE_URL (optional, defaults to https://data.alpaca.markets)
- ALPACA_PAPER (optional, "1"/"true" uses paper endpoints)
"""
from __future__ import annotations

import os
import time
from typing import Dict, List, Optional

try:
    from alpaca.trading.client import TradingClient
    from alpaca.data.historical import StockHistoricalDataClient
    from alpaca.data.requests import StockLatestQuoteRequest, StockLatestTradeRequest
    ALPACA_PY_AVAILABLE = True
except ImportError:
    ALPACA_PY_AVAILABLE = False
    TradingClient = None
    StockHistoricalDataClient = None

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

        # Use official alpaca-py SDK if available
        if ALPACA_PY_AVAILABLE:
            try:
                # TradingClient for account and trading operations
                self._trading_client = TradingClient(
                    api_key=self.api_key_id,
                    secret_key=self.api_secret_key,
                    base_url=self.base_url,
                    paper=paper
                )
                # StockHistoricalDataClient for market data
                self._data_client = StockHistoricalDataClient(
                    api_key=self.api_key_id,
                    secret_key=self.api_secret_key
                )
                self._use_official_sdk = True
            except Exception as e:
                print(f"Warning: Failed to initialize official alpaca-py SDK: {e}")
                print("Falling back to REST API client")
                self._use_official_sdk = False
                self._trading_client = None
                self._data_client = None
        else:
            self._use_official_sdk = False
            self._trading_client = None
            self._data_client = None

        # Fallback REST client
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
        # Use official SDK if available
        if self._use_official_sdk and self._data_client:
            try:
                request = StockLatestQuoteRequest(symbol_or_symbols=[symbol])
                quotes = self._data_client.get_stock_latest_quote(request)
                if quotes and symbol in quotes:
                    quote = quotes[symbol]
                    return {
                        "ask_price": float(quote.ask_price) if quote.ask_price else 0.0,
                        "bid_price": float(quote.bid_price) if quote.bid_price else 0.0,
                        "ask_size": int(quote.ask_size) if quote.ask_size else 0,
                        "bid_size": int(quote.bid_size) if quote.bid_size else 0,
                        "timestamp": quote.timestamp.isoformat() if hasattr(quote.timestamp, 'isoformat') else str(quote.timestamp),
                    }
            except Exception as e:
                print(f"Warning: Official SDK get_latest_quote() failed: {e}")
                # Fall through to REST API fallback

        # Fallback to REST API
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
        # Use official SDK if available
        if self._use_official_sdk and self._data_client:
            try:
                request = StockLatestTradeRequest(symbol_or_symbols=[symbol])
                trades = self._data_client.get_stock_latest_trade(request)
                if trades and symbol in trades:
                    trade = trades[symbol]
                    return {
                        "price": float(trade.price) if trade.price else 0.0,
                        "size": int(trade.size) if trade.size else 0,
                        "timestamp": trade.timestamp.isoformat() if hasattr(trade.timestamp, 'isoformat') else str(trade.timestamp),
                    }
            except Exception as e:
                print(f"Warning: Official SDK get_latest_trade() failed: {e}")
                # Fall through to REST API fallback

        # Fallback to REST API
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

    def get_accounts(self) -> List[Dict]:
        """
        Get list of all accounts accessible with these credentials.

        For Trading API: Returns the single account associated with the API keys.
        For Broker API: Returns list of all managed accounts.

        Returns list of account dicts with id, account_number, status, etc.
        """
        # Use official SDK if available
        if self._use_official_sdk and self._trading_client:
            try:
                account = self._trading_client.get_account()
                if account:
                    # Convert Account model to dict
                    account_dict = {
                        "id": account.id,
                        "account_number": account.account_number,
                        "status": account.status.value if hasattr(account.status, 'value') else str(account.status),
                        "currency": account.currency,
                        "buying_power": float(account.buying_power) if account.buying_power else 0.0,
                        "cash": float(account.cash) if account.cash else 0.0,
                        "portfolio_value": float(account.portfolio_value) if account.portfolio_value else 0.0,
                        "pattern_day_trader": account.pattern_day_trader if hasattr(account, 'pattern_day_trader') else False,
                        "trading_blocked": account.trading_blocked if hasattr(account, 'trading_blocked') else False,
                    }
                    return [account_dict]
            except Exception as e:
                print(f"Warning: Official SDK get_account() failed: {e}")
                # Fall through to REST API fallback

        # Fallback to REST API
        try:
            account_data = self._get(f"{self.base_url}/v2/account")
            if account_data:
                return [account_data]
        except requests.HTTPError as e:
            # If 404, try the Broker API endpoint (plural) for multiple accounts
            if e.response.status_code == 404:
                try:
                    url = f"{self.base_url}/v2/accounts"
                    data = self._get(url)
                    # Broker API returns a list of accounts
                    if isinstance(data, list):
                        return data
                    # Or sometimes wrapped in an 'accounts' key
                    if isinstance(data, dict) and "accounts" in data:
                        return data["accounts"]
                except requests.HTTPError:
                    pass
        return []

    def get_account(self, account_id: Optional[str] = None) -> Optional[Dict]:
        """
        Get account information for a specific account.
        If account_id is None, gets the default account.
        """
        # Use official SDK if available
        if self._use_official_sdk and self._trading_client:
            try:
                account = self._trading_client.get_account()
                if account:
                    # If account_id specified, verify it matches
                    if account_id and account.account_number != account_id and account.id != account_id:
                        return None
                    # Convert Account model to dict
                    account_dict = {
                        "id": account.id,
                        "account_number": account.account_number,
                        "status": account.status.value if hasattr(account.status, 'value') else str(account.status),
                        "currency": account.currency,
                        "buying_power": float(account.buying_power) if account.buying_power else 0.0,
                        "cash": float(account.cash) if account.cash else 0.0,
                        "portfolio_value": float(account.portfolio_value) if account.portfolio_value else 0.0,
                        "pattern_day_trader": account.pattern_day_trader if hasattr(account, 'pattern_day_trader') else False,
                        "trading_blocked": account.trading_blocked if hasattr(account, 'trading_blocked') else False,
                    }
                    return account_dict
            except Exception as e:
                print(f"Warning: Official SDK get_account() failed: {e}")
                # Fall through to REST API fallback

        # Fallback to REST API
        if account_id:
            url = f"{self.base_url}/v2/accounts/{account_id}"
        else:
            url = f"{self.base_url}/v2/account"
        try:
            return self._get(url)
        except requests.HTTPError:
            return None
