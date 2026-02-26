"""
tastytrade_client.py - Client for Tastytrade API integration

Supports both OAuth (preferred) and session-based authentication.

OAuth credentials (environment variables or parameters):
- TASTYTRADE_CLIENT_SECRET
- TASTYTRADE_REFRESH_TOKEN

Session-based credentials (fallback):
- TASTYTRADE_USERNAME
- TASTYTRADE_PASSWORD

Optional:
- TASTYTRADE_BASE_URL (defaults to https://api.tastytrade.com)
"""
from __future__ import annotations

import os
import logging
from typing import Dict, List, Optional
from datetime import datetime, timedelta

import requests

logger = logging.getLogger(__name__)


class TastytradeError(RuntimeError):
    """Generic error raised for Tastytrade API failures."""


class TastytradeClient:
    """Client for Tastytrade Open API with OAuth and session-based authentication."""

    def __init__(
        self,
        client_secret: Optional[str] = None,
        refresh_token: Optional[str] = None,
        username: Optional[str] = None,
        password: Optional[str] = None,
        base_url: Optional[str] = None,
        sandbox: Optional[bool] = None,
        sandbox_base_url: Optional[str] = None,
        session: Optional[requests.Session] = None,
    ) -> None:
        # OAuth credentials (preferred)
        self.client_secret = client_secret or os.getenv("TASTYTRADE_CLIENT_SECRET", "")
        self.refresh_token = refresh_token or os.getenv("TASTYTRADE_REFRESH_TOKEN", "")
        self._use_oauth = bool(self.client_secret and self.refresh_token)

        # Session-based credentials (fallback)
        self.username = username or os.getenv("TASTYTRADE_USERNAME", "")
        self.password = password or os.getenv("TASTYTRADE_PASSWORD", "")

        # Require at least one authentication method
        if not self._use_oauth and (not self.username or not self.password):
            raise RuntimeError(
                "Missing Tastytrade credentials. Provide either:\n"
                "  OAuth: TASTYTRADE_CLIENT_SECRET and TASTYTRADE_REFRESH_TOKEN\n"
                "  Session: TASTYTRADE_USERNAME and TASTYTRADE_PASSWORD"
            )

        # Determine sandbox mode
        # Priority: parameter > environment variable > config (if available) > default False
        if sandbox is None:
            sandbox_env = os.getenv("TASTYTRADE_SANDBOX", "").lower()
            sandbox = sandbox_env in ("true", "1", "yes", "on")

        self.sandbox = sandbox

        # Select base URL based on sandbox mode
        if self.sandbox:
            # Sandbox mode: use sandbox URL
            default_sandbox_url = "https://api.cert.tastyworks.com"
            self.base_url = (sandbox_base_url or base_url or os.getenv("TASTYTRADE_SANDBOX_BASE_URL", default_sandbox_url)).rstrip("/")
            logger.info("Tastytrade client initialized in SANDBOX mode (resets every 24h, 15-min delayed quotes)")
        else:
            # Production mode: use production URL
            default_prod_url = "https://api.tastytrade.com"
            self.base_url = (base_url or os.getenv("TASTYTRADE_BASE_URL", default_prod_url)).rstrip("/")
            logger.info("Tastytrade client initialized in PRODUCTION mode")
        self._session = session or requests.Session()
        self._session.headers.update({
            "Content-Type": "application/json",
            "Accept": "application/json",
        })

        # OAuth token management
        self._access_token: Optional[str] = None
        self._token_expires_at: Optional[datetime] = None

        # Session-based token (fallback)
        self._session_token: Optional[str] = None
        self._user_data: Optional[Dict] = None

    def _ensure_authenticated(self) -> None:
        """Ensure we have a valid access token or session token."""
        if self._use_oauth:
            # Check if access token is valid (not expired or expiring soon)
            if self._access_token and self._token_expires_at:
                # Refresh if expiring within 1 minute
                if datetime.now() < self._token_expires_at - timedelta(minutes=1):
                    return
            # Get or refresh OAuth token
            self._oauth_authenticate()
        else:
            # Use session-based authentication
            if self._session_token:
                return
            self.login()

    def _oauth_authenticate(self) -> None:
        """
        Authenticate using OAuth refresh token and get access token.

        Raises:
            TastytradeError: If authentication fails
        """
        endpoint = f"{self.base_url}/oauth/token"
        payload = {
            "client_secret": self.client_secret,
            "refresh_token": self.refresh_token,
        }

        try:
            response = self._session.post(endpoint, json=payload, timeout=10)
            response.raise_for_status()
            data = response.json()

            # Extract access token and expiration
            # Tastytrade API response format may vary
            if "data" in data:
                token_data = data["data"]
            else:
                token_data = data

            access_token = token_data.get("access_token") or token_data.get("token")
            expires_in = token_data.get("expires_in", 900)  # Default 15 minutes

            if not access_token:
                raise TastytradeError("No access token in OAuth response")

            self._access_token = access_token
            # Set expiration time (subtract 1 minute for safety margin)
            self._token_expires_at = datetime.now() + timedelta(seconds=expires_in - 60)

            # Update session headers
            self._session.headers["Authorization"] = f"Bearer {self._access_token}"

            logger.info("Tastytrade OAuth authentication successful")
        except requests.exceptions.RequestException as e:
            # Clear tokens on failure
            self._access_token = None
            self._token_expires_at = None
            raise TastytradeError(f"OAuth authentication failed: {e}") from e

    def refresh_access_token(self) -> Dict:
        """
        Manually refresh the OAuth access token.

        Returns:
            Token data dictionary

        Raises:
            TastytradeError: If refresh fails or OAuth not configured
        """
        if not self._use_oauth:
            raise TastytradeError("OAuth not configured. Cannot refresh token.")

        self._oauth_authenticate()
        return {
            "access_token": self._access_token,
            "expires_at": self._token_expires_at.isoformat() if self._token_expires_at else None,
        }

    def login(self) -> Dict:
        """
        Authenticate with Tastytrade API and store session token.

        Returns:
            User data dictionary
        """
        endpoint = f"{self.base_url}/sessions"
        payload = {
            "login": self.username,
            "password": self.password,
        }

        try:
            response = self._session.post(endpoint, json=payload, timeout=10)
            response.raise_for_status()
            data = response.json()

            # Extract session token from response
            # Tastytrade API typically returns token in 'data' field or as 'session-token' header
            if "data" in data and "user" in data["data"]:
                self._user_data = data["data"]["user"]
                # Session token may be in response data or as a cookie
                if "session-token" in response.headers:
                    self._session_token = response.headers["session-token"]
                elif "data" in data and "session-token" in data["data"]:
                    self._session_token = data["data"]["session-token"]
                else:
                    # Some APIs return token in cookies
                    cookies = response.cookies
                    if "session-token" in cookies:
                        self._session_token = cookies["session-token"]
                    elif "session_token" in cookies:
                        self._session_token = cookies["session_token"]

                # Update session headers with token if found
                if self._session_token:
                    self._session.headers["Authorization"] = f"Bearer {self._session_token}"

                logger.info(f"Tastytrade authentication successful for user: {self._user_data.get('username', 'unknown')}")
                return data["data"]
            else:
                raise TastytradeError("Invalid response format from login endpoint")
        except requests.exceptions.RequestException as e:
            raise TastytradeError(f"Authentication failed: {e}") from e

    def _get(self, url: str, params: Optional[Dict] = None) -> Dict:
        """Make authenticated GET request with automatic token refresh on 401."""
        self._ensure_authenticated()
        resp = self._session.get(url, params=params or {}, timeout=10)

        # Handle 401 Unauthorized - try refreshing token once
        if resp.status_code == 401 and self._use_oauth:
            logger.warning("Received 401, attempting token refresh")
            self._oauth_authenticate()
            resp = self._session.get(url, params=params or {}, timeout=10)

        resp.raise_for_status()
        return resp.json()

    def _post(self, url: str, json_data: Optional[Dict] = None) -> Dict:
        """Make authenticated POST request with automatic token refresh on 401."""
        self._ensure_authenticated()
        resp = self._session.post(url, json=json_data or {}, timeout=10)

        # Handle 401 Unauthorized - try refreshing token once
        if resp.status_code == 401 and self._use_oauth:
            logger.warning("Received 401, attempting token refresh")
            self._oauth_authenticate()
            resp = self._session.post(url, json=json_data or {}, timeout=10)

        resp.raise_for_status()
        return resp.json()

    def get_accounts(self) -> List[Dict]:
        """
        Get list of available accounts.

        Returns:
            List of account dictionaries
        """
        try:
            # Tastytrade API endpoint for accounts
            endpoint = f"{self.base_url}/customers/me/accounts"
            data = self._get(endpoint)

            # Extract accounts from response
            if "data" in data and "items" in data["data"]:
                return data["data"]["items"]
            elif "data" in data and isinstance(data["data"], list):
                return data["data"]
            elif isinstance(data, list):
                return data
            else:
                logger.warning(f"Unexpected account response format: {data}")
                return []
        except requests.exceptions.RequestException as e:
            logger.error(f"Failed to get accounts: {e}")
            raise TastytradeError(f"Failed to get accounts: {e}") from e

    def get_account_summary(self, account_number: str) -> Dict:
        """
        Get account summary/balance information.

        Args:
            account_number: Account number/ID

        Returns:
            Account summary dictionary
        """
        try:
            # Tastytrade API endpoint for account summary
            endpoint = f"{self.base_url}/accounts/{account_number}/summary"
            data = self._get(endpoint)

            # Extract summary from response
            if "data" in data:
                return data["data"]
            return data
        except requests.exceptions.RequestException as e:
            logger.error(f"Failed to get account summary for {account_number}: {e}")
            raise TastytradeError(f"Failed to get account summary: {e}") from e

    def get_positions(self, account_number: str) -> List[Dict]:
        """
        Get account positions.

        Args:
            account_number: Account number/ID

        Returns:
            List of position dictionaries
        """
        try:
            # Tastytrade API endpoint for positions
            endpoint = f"{self.base_url}/accounts/{account_number}/positions"
            data = self._get(endpoint)

            # Extract positions from response
            if "data" in data and "items" in data["data"]:
                return data["data"]["items"]
            elif "data" in data and isinstance(data["data"], list):
                return data["data"]
            elif isinstance(data, list):
                return data
            else:
                logger.warning(f"Unexpected positions response format: {data}")
                return []
        except requests.exceptions.RequestException as e:
            logger.error(f"Failed to get positions for {account_number}: {e}")
            raise TastytradeError(f"Failed to get positions: {e}") from e

    def get_snapshot(self, symbol: str) -> Dict:
        """
        Get market data snapshot for a symbol.

        Args:
            symbol: Stock/option symbol (e.g., "SPY")

        Returns:
            Market data dictionary with bid, ask, last, etc.
        """
        try:
            endpoint = f"{self.base_url}/quotes/{symbol}"
            data = self._get(endpoint)

            if "data" in data:
                return data["data"]
            return data
        except requests.exceptions.RequestException as e:
            logger.warning(f"Failed to get snapshot for {symbol}: {e}")
            return {
                "last": 0.0,
                "bid": 0.0,
                "ask": 0.0,
                "volume": 0,
            }

    # ========================================================================
    # Option Chain
    # ========================================================================

    def get_option_chain(
        self,
        underlying_symbol: str,
        expiration_date: Optional[str] = None,
    ) -> Dict[str, List[Dict]]:
        """Fetch option chain for an underlying, grouped by expiration.

        Args:
            underlying_symbol: Underlying ticker (e.g. "SPY").
            expiration_date: Optional filter (YYYY-MM-DD).

        Returns:
            Dict mapping expiration dates to lists of option dicts.
        """
        endpoint = f"{self.base_url}/option-chains/{underlying_symbol}/nested"
        params: Dict = {}
        if expiration_date:
            params["expiration-date"] = expiration_date

        try:
            data = self._get(endpoint, params=params)
        except requests.exceptions.RequestException as exc:
            logger.warning("Failed to get option chain for %s: %s", underlying_symbol, exc)
            return {}

        items = []
        if "data" in data and "items" in data["data"]:
            items = data["data"]["items"]
        elif "data" in data and isinstance(data["data"], list):
            items = data["data"]
        elif isinstance(data, list):
            items = data

        chain: Dict[str, List[Dict]] = {}
        for item in items:
            if not isinstance(item, dict):
                continue
            expirations = item.get("expirations", [item])
            for exp in expirations:
                if not isinstance(exp, dict):
                    continue
                exp_date = exp.get("expiration-date", "unknown")
                strikes = exp.get("strikes", [exp])
                chain.setdefault(exp_date, []).extend(
                    s for s in strikes if isinstance(s, dict)
                )
        return chain

    def get_option_expirations(self, underlying_symbol: str) -> List[str]:
        """Return available expiration dates for an underlying."""
        chain = self.get_option_chain(underlying_symbol)
        return sorted(chain.keys())

    # ========================================================================
    # Order Placement & Management
    # ========================================================================

    def get_orders(
        self,
        account_number: str,
        status: Optional[str] = None,
        per_page: int = 50,
    ) -> List[Dict]:
        """Fetch orders for an account.

        Args:
            account_number: Account number.
            status: Optional filter (e.g. "Received", "Filled", "Cancelled").
            per_page: Page size.

        Returns:
            List of order dicts.
        """
        endpoint = f"{self.base_url}/accounts/{account_number}/orders"
        params: Dict = {"per-page": per_page}
        if status:
            params["status"] = status

        try:
            data = self._get(endpoint, params=params)
        except requests.exceptions.RequestException as exc:
            logger.error("Failed to get orders for %s: %s", account_number, exc)
            raise TastytradeError(f"Failed to get orders: {exc}") from exc

        if "data" in data and "items" in data["data"]:
            return data["data"]["items"]
        elif "data" in data and isinstance(data["data"], list):
            return data["data"]
        return []

    def place_order(
        self,
        account_number: str,
        order_type: str,
        time_in_force: str,
        legs: List[Dict],
        price: Optional[float] = None,
    ) -> Optional[Dict]:
        """Place an order (single or multi-leg).

        Args:
            account_number: Account to place order in.
            order_type: "Limit", "Market", "Stop", "Stop Limit".
            time_in_force: "Day", "GTC", "IOC".
            legs: List of leg dicts, each with:
                  instrument-type ("Equity Option"), action ("Buy to Open",
                  "Sell to Open", etc.), symbol, quantity.
            price: Net price for limit orders. Positive = debit,
                   negative = credit.

        Returns:
            Order dict on success, None on failure.
        """
        payload: Dict = {
            "order-type": order_type,
            "time-in-force": time_in_force,
            "legs": legs,
        }
        if price is not None:
            payload["price"] = str(price)

        endpoint = f"{self.base_url}/accounts/{account_number}/orders"
        try:
            data = self._post(endpoint, json_data=payload)
            if "data" in data:
                return data["data"]
            return data
        except requests.exceptions.RequestException as exc:
            logger.error("Failed to place order: %s", exc)
            return None

    def cancel_order(self, account_number: str, order_id: str) -> bool:
        """Cancel an order. Returns True on success."""
        endpoint = f"{self.base_url}/accounts/{account_number}/orders/{order_id}"
        self._ensure_authenticated()
        try:
            resp = self._session.delete(endpoint, timeout=10)
            if resp.status_code == 401 and self._use_oauth:
                self._oauth_authenticate()
                resp = self._session.delete(endpoint, timeout=10)
            return resp.status_code in (200, 204)
        except requests.exceptions.RequestException:
            return False
