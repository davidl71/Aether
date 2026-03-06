"""
alpaca_client.py - Alpaca client using official alpaca-py SDK

Supports both API key authentication and OAuth 2.0 authentication.

API Key Authentication (default):
- ALPACA_API_KEY_ID
- ALPACA_API_SECRET_KEY

OAuth 2.0 Authentication:
- ALPACA_CLIENT_ID
- ALPACA_CLIENT_SECRET
- ALPACA_ACCESS_TOKEN (optional, will be obtained via OAuth if not provided)
- ALPACA_REFRESH_TOKEN (optional, for token refresh)

Other environment variables:
- ALPACA_BASE_URL (optional, defaults to https://paper-api.alpaca.markets or https://api.alpaca.markets when ALPACA_PAPER=0)
- ALPACA_DATA_BASE_URL (optional, defaults to https://data.alpaca.markets)
- ALPACA_PAPER (optional, "1"/"true" uses paper endpoints)
"""

from __future__ import annotations

import os
import time
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional

try:
    from .onepassword_sdk_helper import getenv_or_resolve, get_alpaca_credentials_from_1password
except ImportError:
    def getenv_or_resolve(env_var: str, op_ref: str, default: str = "") -> str:
        return os.getenv(env_var, default)

    def get_alpaca_credentials_from_1password():
        return None

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

logger = logging.getLogger(__name__)


class AlpacaError(RuntimeError):
    """Generic error raised for Alpaca API failures."""


class AlpacaClient:
    def __init__(
        self,
        api_key_id: Optional[str] = None,
        api_secret_key: Optional[str] = None,
        client_id: Optional[str] = None,
        client_secret: Optional[str] = None,
        access_token: Optional[str] = None,
        refresh_token: Optional[str] = None,
        base_url: Optional[str] = None,
        data_base_url: Optional[str] = None,
        session: Optional[requests.Session] = None,
    ) -> None:
        # OAuth credentials (preferred if available); optional 1Password op:// refs via SDK
        self.client_id = client_id or getenv_or_resolve("ALPACA_CLIENT_ID", "OP_ALPACA_CLIENT_ID_SECRET", "")
        self.client_secret = client_secret or getenv_or_resolve("ALPACA_CLIENT_SECRET", "OP_ALPACA_CLIENT_SECRET_SECRET", "")
        self._use_oauth = bool(self.client_id and self.client_secret)

        # API key credentials (fallback); optional 1Password op:// refs via SDK, or SDK discovery
        self.api_key_id = api_key_id or getenv_or_resolve("ALPACA_API_KEY_ID", "OP_ALPACA_API_KEY_ID_SECRET", "")
        self.api_secret_key = api_secret_key or getenv_or_resolve("ALPACA_API_SECRET_KEY", "OP_ALPACA_API_SECRET_KEY_SECRET", "")
        if not self.api_key_id or not self.api_secret_key:
            discovered = get_alpaca_credentials_from_1password()
            if discovered:
                key_val, secret_val = discovered
                if not self.api_key_id:
                    self.api_key_id = key_val
                if not self.api_secret_key:
                    self.api_secret_key = secret_val

        # Require at least one authentication method
        if not self._use_oauth and (not self.api_key_id or not self.api_secret_key):
            raise RuntimeError(
                "Missing Alpaca credentials. Provide either:\n"
                "  OAuth: ALPACA_CLIENT_ID and ALPACA_CLIENT_SECRET\n"
                "  API Keys: ALPACA_API_KEY_ID and ALPACA_API_SECRET_KEY"
            )

        # OAuth token management; optional 1Password op:// refs via SDK
        self._access_token = access_token or getenv_or_resolve("ALPACA_ACCESS_TOKEN", "OP_ALPACA_ACCESS_TOKEN_SECRET", "")
        self._refresh_token = refresh_token or getenv_or_resolve("ALPACA_REFRESH_TOKEN", "OP_ALPACA_REFRESH_TOKEN_SECRET", "")
        self._token_expires_at: Optional[datetime] = None

        paper = os.getenv("ALPACA_PAPER", "1").lower() in {"1", "true", "yes", "on"}
        default_trading_base = (
            "https://paper-api.alpaca.markets"
            if paper
            else "https://api.alpaca.markets"
        )
        default_data_base = "https://data.alpaca.markets"

        self.base_url = base_url or os.getenv("ALPACA_BASE_URL", default_trading_base)
        self.data_base_url = data_base_url or os.getenv("ALPACA_DATA_BASE_URL", default_data_base)
        self.paper = paper

        # Use official alpaca-py SDK if available (only with API keys, not OAuth)
        if ALPACA_PY_AVAILABLE and not self._use_oauth:
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
                logger.warning(f"Failed to initialize official alpaca-py SDK: {e}")
                logger.warning("Falling back to REST API client")
                self._use_official_sdk = False
                self._trading_client = None
                self._data_client = None
        else:
            self._use_official_sdk = False
            self._trading_client = None
            self._data_client = None

        # REST client
        self._session = session or requests.Session()
        self._session.headers.update({"Accept": "application/json"})

        # Set authentication headers based on auth method
        if self._use_oauth:
            self._ensure_oauth_authenticated()
        else:
            self._session.headers.update(
                {
                    "APCA-API-KEY-ID": self.api_key_id,
                    "APCA-API-SECRET-KEY": self.api_secret_key,
                }
            )

    def _ensure_oauth_authenticated(self) -> None:
        """Ensure we have a valid OAuth access token."""
        if not self._use_oauth:
            return

        # Check if access token is valid (not expired or expiring soon)
        if self._access_token and self._token_expires_at:
            # Refresh if expiring within 1 minute
            if datetime.now() < self._token_expires_at - timedelta(minutes=1):
                self._session.headers["Authorization"] = f"Bearer {self._access_token}"
                return

        # Get or refresh OAuth token
        self._oauth_authenticate()

    def _oauth_authenticate(self) -> None:
        """
        Authenticate using OAuth client credentials flow.

        Raises:
            AlpacaError: If authentication fails
        """
        if not self._use_oauth:
            raise AlpacaError("OAuth not configured")

        # Alpaca OAuth endpoint (authorization server)
        # For paper trading, use paper OAuth endpoint
        oauth_base = "https://api.alpaca.markets" if not self.paper else "https://paper-api.alpaca.markets"
        token_url = f"{oauth_base}/oauth/token"

        # Client credentials grant flow
        payload = {
            "grant_type": "client_credentials",
            "client_id": self.client_id,
            "client_secret": self.client_secret,
        }

        try:
            response = self._session.post(
                token_url,
                data=payload,  # Use form data, not JSON
                headers={"Content-Type": "application/x-www-form-urlencoded"},
                timeout=10
            )
            response.raise_for_status()
            data = response.json()

            access_token = data.get("access_token")
            expires_in = data.get("expires_in", 3600)  # Default 1 hour

            if not access_token:
                raise AlpacaError("No access token in OAuth response")

            self._access_token = access_token
            # Set expiration time (subtract 1 minute for safety margin)
            self._token_expires_at = datetime.now() + timedelta(seconds=expires_in - 60)

            # Update session headers
            self._session.headers["Authorization"] = f"Bearer {self._access_token}"

            logger.info("Alpaca OAuth authentication successful")
        except requests.exceptions.RequestException as e:
            # Clear tokens on failure
            self._access_token = None
            self._token_expires_at = None
            raise AlpacaError(f"OAuth authentication failed: {e}") from e

    def refresh_access_token(self) -> Dict:
        """
        Manually refresh the OAuth access token.

        Returns:
            Dict with token information
        """
        if not self._use_oauth:
            raise AlpacaError("OAuth not configured. Cannot refresh token.")

        self._oauth_authenticate()
        return {
            "access_token": self._access_token,
            "expires_at": self._token_expires_at.isoformat() if self._token_expires_at else None,
        }

    def _get(self, url: str, params: Optional[Dict] = None) -> Dict:
        """Make GET request and return JSON response as Dict."""
        # Ensure OAuth token is valid if using OAuth
        if self._use_oauth:
            self._ensure_oauth_authenticated()

        resp = self._session.get(url, params=params or {}, timeout=10)

        # Handle 401 errors - might need to refresh token
        if resp.status_code == 401 and self._use_oauth:
            # Try refreshing token once
            try:
                self._oauth_authenticate()
                resp = self._session.get(url, params=params or {}, timeout=10)
            except AlpacaError:
                pass  # Re-raise the original 401

        resp.raise_for_status()
        result = resp.json()
        # Ensure we return a Dict (API sometimes returns lists, wrap them)
        if isinstance(result, list):
            return {"items": result}
        return result if isinstance(result, dict) else {}

    def _get_list(self, url: str, params: Optional[Dict] = None) -> List[Dict]:
        """Make GET request and return JSON response as List[Dict]."""
        data = self._get(url, params=params)
        # Handle both list and dict responses
        if isinstance(data, list):
            return data
        if isinstance(data, dict) and "items" in data:
            return data["items"]
        return []

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
        last = (
            float(trade.get("price") or 0.0) or (bid + ask) / 2.0
            if (bid and ask)
            else (bid or ask)
        )
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
                        accounts_list = data["accounts"]
                        return accounts_list if isinstance(accounts_list, list) else []
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

    def get_positions(self) -> List[Dict]:
        """
        Get all open positions from Alpaca.
        Returns list of position dicts.
        """
        url = f"{self.base_url}/v2/positions"
        try:
            return self._get_list(url)
        except requests.HTTPError:
            return []

    def get_orders(self, status: str = "all", limit: int = 50) -> List[Dict]:
        """
        Get orders from Alpaca.
        Args:
            status: Order status filter (all, open, closed)
            limit: Maximum number of orders to return
        Returns list of order dicts.
        """
        url = f"{self.base_url}/v2/orders"
        params = {"status": status, "limit": limit}
        try:
            return self._get_list(url, params=params)
        except requests.HTTPError:
            return []

    # ========================================================================
    # Option Chain
    # ========================================================================

    def get_option_contracts(
        self,
        underlying_symbol: str,
        expiration_date: Optional[str] = None,
        option_type: Optional[str] = None,
        strike_price_gte: Optional[float] = None,
        strike_price_lte: Optional[float] = None,
        limit: int = 100,
    ) -> List[Dict]:
        """Fetch option contracts from Alpaca Options API.

        Args:
            underlying_symbol: Underlying ticker (e.g. "SPY").
            expiration_date: Filter by expiration (YYYY-MM-DD).
            option_type: "call" or "put".
            strike_price_gte: Minimum strike price.
            strike_price_lte: Maximum strike price.
            limit: Max contracts to return (API caps at 10000).

        Returns:
            List of option contract dicts with keys: id, symbol,
            underlying_symbol, expiration_date, strike_price, type,
            status, tradable, size, style.
        """
        url = f"{self.base_url}/v2/options/contracts"
        params: Dict = {
            "underlying_symbols": underlying_symbol,
            "limit": limit,
        }
        if expiration_date:
            params["expiration_date"] = expiration_date
        if option_type:
            params["type"] = option_type
        if strike_price_gte is not None:
            params["strike_price_gte"] = str(strike_price_gte)
        if strike_price_lte is not None:
            params["strike_price_lte"] = str(strike_price_lte)

        if self._use_oauth:
            self._ensure_oauth_authenticated()

        try:
            resp = self._session.get(url, params=params, timeout=10)
            if resp.status_code == 401 and self._use_oauth:
                self._oauth_authenticate()
                resp = self._session.get(url, params=params, timeout=10)
            resp.raise_for_status()
            data = resp.json()
        except requests.HTTPError:
            return []

        contracts = data.get("option_contracts") or data.get("items") or []
        if isinstance(contracts, list):
            return contracts
        return []

    def get_option_chain(
        self,
        underlying_symbol: str,
        expiration_date: Optional[str] = None,
    ) -> Dict[str, List[Dict]]:
        """Return option chain grouped by expiration date.

        Returns:
            Dict mapping expiration date strings to lists of contracts.
        """
        contracts = self.get_option_contracts(
            underlying_symbol,
            expiration_date=expiration_date,
            limit=10000,
        )
        chain: Dict[str, List[Dict]] = {}
        for c in contracts:
            exp = c.get("expiration_date", "unknown")
            chain.setdefault(exp, []).append(c)
        return chain

    # ========================================================================
    # Order Placement & Cancellation
    # ========================================================================

    def _post(self, url: str, json_data: Optional[Dict] = None) -> Dict:
        """Make authenticated POST request."""
        if self._use_oauth:
            self._ensure_oauth_authenticated()
        resp = self._session.post(url, json=json_data or {}, timeout=10)
        if resp.status_code == 401 and self._use_oauth:
            self._oauth_authenticate()
            resp = self._session.post(url, json=json_data or {}, timeout=10)
        resp.raise_for_status()
        result = resp.json()
        return result if isinstance(result, dict) else {}

    def _delete(self, url: str) -> bool:
        """Make authenticated DELETE request. Returns True on success."""
        if self._use_oauth:
            self._ensure_oauth_authenticated()
        resp = self._session.delete(url, timeout=10)
        if resp.status_code == 401 and self._use_oauth:
            self._oauth_authenticate()
            resp = self._session.delete(url, timeout=10)
        return resp.status_code in (200, 204)

    def place_order(
        self,
        symbol: str,
        qty: int,
        side: str,
        order_type: str = "market",
        time_in_force: str = "day",
        limit_price: Optional[float] = None,
        stop_price: Optional[float] = None,
        extended_hours: bool = False,
    ) -> Optional[Dict]:
        """Place a single-leg order.

        Args:
            symbol: Ticker or OCC option symbol.
            qty: Number of shares/contracts.
            side: "buy" or "sell".
            order_type: "market", "limit", "stop", "stop_limit".
            time_in_force: "day", "gtc", "ioc", "fok".
            limit_price: Required for limit/stop_limit orders.
            stop_price: Required for stop/stop_limit orders.
            extended_hours: Allow extended hours trading.

        Returns:
            Order dict on success, None on failure.
        """
        payload: Dict = {
            "symbol": symbol,
            "qty": str(qty),
            "side": side,
            "type": order_type,
            "time_in_force": time_in_force,
        }
        if limit_price is not None:
            payload["limit_price"] = str(limit_price)
        if stop_price is not None:
            payload["stop_price"] = str(stop_price)
        if extended_hours:
            payload["extended_hours"] = True

        url = f"{self.base_url}/v2/orders"
        try:
            return self._post(url, json_data=payload)
        except requests.HTTPError as exc:
            logger.error("Failed to place order: %s", exc)
            return None

    def place_multi_leg_order(
        self,
        legs: List[Dict],
        order_type: str = "limit",
        time_in_force: str = "day",
        limit_price: Optional[float] = None,
    ) -> Optional[Dict]:
        """Place a multi-leg (combo) options order.

        Args:
            legs: List of dicts, each with keys:
                  symbol (OCC symbol), qty, side ("buy"/"sell"),
                  position_effect ("open"/"close").
            order_type: "market" or "limit".
            time_in_force: "day", "gtc".
            limit_price: Net debit (positive) or credit (negative).

        Returns:
            Order dict on success, None on failure.
        """
        payload: Dict = {
            "order_class": "mleg",
            "type": order_type,
            "time_in_force": time_in_force,
            "legs": legs,
        }
        if limit_price is not None:
            payload["limit_price"] = str(limit_price)

        url = f"{self.base_url}/v2/orders"
        try:
            return self._post(url, json_data=payload)
        except requests.HTTPError as exc:
            logger.error("Failed to place multi-leg order: %s", exc)
            return None

    def cancel_order(self, order_id: str) -> bool:
        """Cancel an order by ID. Returns True on success."""
        url = f"{self.base_url}/v2/orders/{order_id}"
        try:
            return self._delete(url)
        except requests.HTTPError:
            return False

    def cancel_all_orders(self) -> bool:
        """Cancel all open orders. Returns True on success."""
        url = f"{self.base_url}/v2/orders"
        try:
            return self._delete(url)
        except requests.HTTPError:
            return False
