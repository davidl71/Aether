"""
tradestation_client.py - TradeStation API v3 REST client (broker-agnostic feed)

Reads API credentials from environment variables:
- TRADESTATION_CLIENT_ID
- TRADESTATION_CLIENT_SECRET
- TRADESTATION_ACCOUNT_ID (optional, for account-specific operations)
- TRADESTATION_BASE_URL (optional, defaults to https://api.tradestation.com/v3)
- TRADESTATION_SIM (optional, "1"/"true" uses SIM environment)
"""
from __future__ import annotations

import logging
import os
import time
from typing import Dict, List, Optional

try:
    from .onepassword_sdk_helper import getenv_or_resolve
except ImportError:
    def getenv_or_resolve(env_var: str, op_ref: str, default: str = "") -> str:
        return os.getenv(env_var, default)

import requests

logger = logging.getLogger(__name__)


class TradeStationClient:
    def __init__(
        self,
        client_id: Optional[str] = None,
        client_secret: Optional[str] = None,
        account_id: Optional[str] = None,
        base_url: Optional[str] = None,
        session: Optional[requests.Session] = None,
    ) -> None:
        # Optional 1Password op:// refs via OP_TRADESTATION_*_SECRET when SDK available
        self.client_id = client_id or getenv_or_resolve("TRADESTATION_CLIENT_ID", "OP_TRADESTATION_CLIENT_ID_SECRET", "")
        self.client_secret = client_secret or getenv_or_resolve("TRADESTATION_CLIENT_SECRET", "OP_TRADESTATION_CLIENT_SECRET_SECRET", "")
        if not self.client_id or not self.client_secret:
            raise RuntimeError(
                "Missing TradeStation credentials (TRADESTATION_CLIENT_ID/TRADESTATION_CLIENT_SECRET)"
            )

        self.account_id = account_id or getenv_or_resolve("TRADESTATION_ACCOUNT_ID", "OP_TRADESTATION_ACCOUNT_ID_SECRET", "")

        # TradeStation uses SIM environment for paper trading
        sim = os.getenv("TRADESTATION_SIM", "1").lower() in {"1", "true", "yes", "on"}
        default_base = "https://sim-api.tradestation.com/v3" if sim else "https://api.tradestation.com/v3"

        self.base_url = base_url or os.getenv("TRADESTATION_BASE_URL", default_base)
        self._session = session or requests.Session()
        self._access_token: Optional[str] = None
        self._token_expires_at: float = 0.0

    def _get_access_token(self) -> str:
        """
        Get OAuth access token using client credentials flow.
        TradeStation uses OAuth 2.0 client credentials grant.
        """
        # Return cached token if still valid (with 60s buffer)
        if self._access_token and time.time() < (self._token_expires_at - 60):
            return self._access_token

        # Request new token
        # Note: TradeStation OAuth endpoint may vary - adjust based on actual API docs
        token_url = f"{self.base_url}/security/authorize"
        # Alternative common OAuth endpoints:
        # - https://signin.tradestation.com/oauth/token
        # - https://api.tradestation.com/v2/authorize

        # Try common OAuth token endpoint pattern
        auth_base = self.base_url.replace("/v3", "").replace("/v2", "")
        token_url = f"{auth_base}/oauth/token"

        try:
            resp = self._session.post(
                token_url,
                data={
                    "grant_type": "client_credentials",
                    "client_id": self.client_id,
                    "client_secret": self.client_secret,
                },
                headers={"Content-Type": "application/x-www-form-urlencoded"},
                timeout=10,
            )
            resp.raise_for_status()
            data = resp.json()
            self._access_token = data.get("access_token", "")
            expires_in = data.get("expires_in", 3600)  # Default 1 hour
            self._token_expires_at = time.time() + expires_in
            return self._access_token
        except requests.RequestException as e:
            raise RuntimeError(f"Failed to obtain TradeStation access token: {e}") from e

    def _get_headers(self) -> Dict[str, str]:
        """Get headers with current access token."""
        token = self._get_access_token()
        return {
            "Authorization": f"Bearer {token}",
            "Accept": "application/json",
        }

    def _get(self, url: str, params: Optional[Dict] = None) -> Dict:
        """Make authenticated GET request."""
        resp = self._session.get(
            url, params=params or {}, headers=self._get_headers(), timeout=10
        )
        resp.raise_for_status()
        return resp.json()

    def get_quote(self, symbol: str) -> Optional[Dict]:
        """
        Get the latest quote for a symbol.
        Returns dict with fields: Ask, Bid, AskSize, BidSize, Last, etc.
        """
        # TradeStation v3 quote endpoint (adjust based on actual API spec)
        url = f"{self.base_url}/marketdata/quotes/{symbol}"
        try:
            data = self._get(url)
        except requests.HTTPError:
            return None

        # TradeStation response format may vary - adjust based on actual response
        # Common fields: Ask, Bid, AskSize, BidSize, Last, LastSize, etc.
        quote = data if isinstance(data, dict) else data.get("Quotes", [{}])[0] if isinstance(data, dict) and "Quotes" in data else {}
        if not quote:
            return None

        return {
            "ask_price": float(quote.get("Ask") or quote.get("ask") or 0.0),
            "bid_price": float(quote.get("Bid") or quote.get("bid") or 0.0),
            "ask_size": int(quote.get("AskSize") or quote.get("ask_size") or 0),
            "bid_size": int(quote.get("BidSize") or quote.get("bid_size") or 0),
            "last": float(quote.get("Last") or quote.get("last") or 0.0),
            "timestamp": quote.get("Timestamp") or quote.get("timestamp") or "",
        }

    def get_snapshot(self, symbol: str) -> Dict:
        """
        Convenience: get quote snapshot in standardized format.
        """
        quote = self.get_quote(symbol) or {}
        bid = float(quote.get("bid_price") or 0.0)
        ask = float(quote.get("ask_price") or 0.0)
        last = float(quote.get("last") or 0.0) or (bid + ask) / 2.0 if (bid and ask) else (bid or ask)
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
            "received_at": int(time.time()),
        }

    def get_accounts(self) -> List[Dict]:
        """Return a list of account dicts (id, type, status)."""
        url = f"{self.base_url}/brokerage/accounts"
        try:
            data = self._get(url)
        except requests.HTTPError as exc:
            logger.warning("Failed to fetch TradeStation accounts: %s", exc)
            return []

        accounts = data.get("Accounts", data) if isinstance(data, dict) else data
        if not isinstance(accounts, list):
            accounts = [accounts] if accounts else []
        return accounts

    def get_positions(self, account_id: Optional[str] = None) -> List[Dict]:
        """Fetch positions for the given (or default) account.

        Returns a list of dicts with standardized keys:
        ``symbol``, ``quantity``, ``avg_price``, ``current_price``,
        ``market_value``, ``unrealized_pl``, ``currency``.
        """
        acct = account_id or self.account_id
        if not acct:
            accounts = self.get_accounts()
            if accounts:
                acct = accounts[0].get("AccountID") or accounts[0].get("Key", "")
            if not acct:
                logger.error("No TradeStation account ID available")
                return []

        url = f"{self.base_url}/brokerage/accounts/{acct}/positions"
        try:
            data = self._get(url)
        except requests.HTTPError as exc:
            logger.warning("Failed to fetch TradeStation positions: %s", exc)
            return []

        raw_positions = data.get("Positions", data) if isinstance(data, dict) else data
        if not isinstance(raw_positions, list):
            raw_positions = [raw_positions] if raw_positions else []

        formatted: List[Dict] = []
        for pos in raw_positions:
            if not isinstance(pos, dict):
                continue
            quantity = float(pos.get("Quantity") or pos.get("LongShort", 0))
            if pos.get("LongShort", "").upper() == "SHORT":
                quantity = -abs(quantity)

            formatted.append({
                "symbol": pos.get("Symbol", ""),
                "quantity": quantity,
                "avg_price": float(pos.get("AveragePrice") or 0.0),
                "current_price": float(
                    pos.get("Last") or pos.get("MarketPrice") or 0.0
                ),
                "market_value": float(pos.get("MarketValue") or 0.0),
                "unrealized_pl": float(
                    pos.get("UnrealizedProfitLoss")
                    or pos.get("UnrealizedPL")
                    or 0.0
                ),
                "currency": pos.get("Currency", "USD"),
            })

        return formatted
