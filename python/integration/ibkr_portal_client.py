"""
ibkr_portal_client.py - Lightweight wrapper around IBKR Client Portal Web API.

The Client Portal Gateway must be running locally. Authentication typically
requires establishing a browser session once; this client focuses on
maintaining the session (validate / reauthenticate) and exposes helpers for
account and portfolio data retrieval.
"""
from __future__ import annotations

import logging
from typing import Dict, List, Optional

import requests


logger = logging.getLogger(__name__)


class IBKRPortalError(RuntimeError):
    """Generic error raised for Client Portal failures."""


class IBKRPortalClient:
    """Minimal client for high-value IBKR Client Portal endpoints."""

    def __init__(
        self,
        base_url: str = "https://localhost:5000/v1/portal",
        verify_ssl: bool = False,
        timeout_seconds: int = 5,
        preferred_accounts: Optional[List[str]] = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout_seconds
        self.preferred_accounts = preferred_accounts or []

        self.session = requests.Session()
        self.session.verify = verify_ssl
        self.session.headers.update({"Content-Type": "application/json"})

        self._cached_accounts: List[str] = []
        self._conid_cache: Dict[str, int] = {}  # symbol -> conid cache

    # ------------------------------------------------------------------
    # Session Helpers
    # ------------------------------------------------------------------

    def ensure_session(self) -> None:
        """
        Validate the current session, requesting re-auth only if necessary.

        The IB Client Portal Gateway uses browser-based authentication. When you log in
        via the browser, the gateway session is active. API clients can use this session,
        but may need to trigger re-authentication to establish their own session token.

        This method tries to use the gateway first. Only if that fails do we trigger
        re-authentication, and only if we're confident the gateway is running.
        """
        # First, try to access a protected endpoint to see if we can use the gateway
        # If gateway is authenticated via browser, this should work
        accounts_resp = self._call("GET", "/iserver/accounts", raise_for_status=False)
        if accounts_resp.status_code == 200:
            logger.debug("IBKR gateway is authenticated and accessible")
            return

        # If accounts endpoint returns 401, we need authentication
        # However, if the gateway is already authenticated via browser, calling
        # /iserver/reauthenticate will prompt the user unnecessarily.
        #
        # The IB Client Portal Gateway should allow API clients to use an authenticated
        # gateway session. If accounts returns 401 even when gateway is authenticated,
        # it might be a timing issue or the gateway needs the API client to establish
        # its own session token.
        #
        # We'll only trigger re-authentication if:
        # 1. The accounts endpoint returns 401 (needs auth)
        # 2. AND we can verify the gateway is running (not a connection error)

        if accounts_resp.status_code == 401:
            # Gateway is running but needs authentication
            # Check if gateway is at least responding (not a connection error)
            try:
                # Try auth status endpoint to verify gateway is running
                auth_status_resp = self._call("GET", "/iserver/auth/status", raise_for_status=False)
                # If we get any response (even 401), gateway is running
                if auth_status_resp.status_code in (200, 401):
                    logger.info("IBKR gateway requires API client authentication")
                    logger.info("Triggering re-authentication - if already logged in via browser, you may need to approve")
                    reauth_resp = self._call("POST", "/iserver/reauthenticate", raise_for_status=False)
                    if reauth_resp.status_code in (200, 202):
                        logger.info("Re-authentication triggered successfully")
                        # Wait a moment for re-auth to complete, then verify
                        import time
                        time.sleep(1)
                        # Try accounts again to verify re-auth worked
                        verify_resp = self._call("GET", "/iserver/accounts", raise_for_status=False)
                        if verify_resp.status_code == 200:
                            logger.info("Re-authentication successful, gateway accessible")
                            return
                        else:
                            logger.warning("Re-authentication triggered but accounts endpoint still returns %d", verify_resp.status_code)
                            # Don't raise error - let the calling code handle it
                            return
                    else:
                        raise IBKRPortalError(
                            f"Reauthentication failed (status={reauth_resp.status_code}): {reauth_resp.text[:200]}"
                        )
            except IBKRPortalError:
                # Re-raise IBKRPortalError as-is
                raise
            except Exception as e:
                # Connection error or other issue
                logger.debug(f"Gateway connectivity check failed: {e}")
                raise IBKRPortalError(
                    f"Unable to connect to IB Client Portal Gateway: {e}. "
                    "Ensure the gateway is running at https://localhost:5000"
                ) from e

        # If we get here with a non-401 status, it's an unexpected error
        raise IBKRPortalError(
            f"Unexpected response from IB Client Portal Gateway "
            f"(status={accounts_resp.status_code}): {accounts_resp.text[:200]}"
        )

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def get_accounts(self) -> List[str]:
        """Return list of tradeable account IDs."""
        if self._cached_accounts:
            return self._cached_accounts

        self.ensure_session()
        data = self._call("GET", "/iserver/accounts").json()
        accounts = data.get("accounts", []) if isinstance(data, dict) else []

        if not accounts:
            logger.warning("Client Portal returned no accounts; ensure gateway session is active")
        else:
            logger.info("Client Portal accounts discovered: %s", ", ".join(accounts))

        self._cached_accounts = accounts
        return accounts

    def get_account_summary(self, account_id: Optional[str] = None) -> Dict:
        """Fetch account summary for the provided account (or preferred/default)."""
        accounts = self._choose_account(account_id)
        if not accounts:
            raise IBKRPortalError("No accounts available for summary request")

        self.ensure_session()
        endpoint = f"/iserver/account/{accounts[0]}/summary"
        response = self._call("GET", endpoint)
        data = response.json()
        logger.debug("Client Portal summary for %s: %s", accounts[0], data)
        return data

    def get_portfolio_positions(self, account_id: Optional[str] = None) -> List[Dict]:
        """Return positions for a given account."""
        accounts = self._choose_account(account_id)
        if not accounts:
            return []

        self.ensure_session()
        endpoint = f"/iserver/account/{accounts[0]}/positions"
        response = self._call("GET", endpoint)
        positions = response.json()
        if isinstance(positions, list):
            return positions
        return []

    def search_contracts(
        self, symbol: str, sec_type: str = "STK", exchange: str = "SMART", currency: str = "USD"
    ) -> List[Dict]:
        """
        Search for contracts by symbol to get conid (contract ID).

        Args:
            symbol: Stock symbol (e.g., "SPY")
            sec_type: Security type (STK, OPT, FUT, etc.) - default: "STK"
            exchange: Exchange (SMART, NASDAQ, NYSE, etc.) - default: "SMART"
            currency: Currency - default: "USD"

        Returns:
            List of contract dictionaries with conid, symbol, etc.
        """
        self.ensure_session()
        # IB Client Portal contract search endpoint
        endpoint = "/iserver/secdef/search"
        payload = {
            "symbol": symbol,
            "name": symbol,
            "secType": sec_type,
        }

        try:
            # POST with JSON body
            url = f"{self.base_url}{endpoint}"
            response = self.session.post(url, json=payload, timeout=self.timeout)
            if not response.ok:
                raise IBKRPortalError(
                    f"Client Portal responded with status {response.status_code}: {response.text[:200]}"
                )
            data = response.json()

            # IB returns a list of matches
            if isinstance(data, list):
                # Filter by exchange/currency if specified
                matches = []
                for contract in data:
                    # Prefer SMART or exact exchange match
                    contract_exch = contract.get("exchange", "")
                    contract_curr = contract.get("currency", "")
                    if exchange == "SMART" or contract_exch == exchange:
                        if currency == contract_curr or not currency:
                            matches.append(contract)

                # If no filtered matches, return first result
                if matches:
                    return matches
                elif data:
                    return [data[0]]

            return []
        except IBKRPortalError:
            logger.warning(f"Contract search failed for {symbol}")
            return []

    def get_conid(self, symbol: str, sec_type: str = "STK", exchange: str = "SMART", currency: str = "USD") -> Optional[int]:
        """
        Get conid (contract ID) for a symbol, using cache when available.

        Returns:
            Conid as integer, or None if not found
        """
        cache_key = f"{symbol}:{sec_type}:{exchange}:{currency}"
        if cache_key in self._conid_cache:
            return self._conid_cache[cache_key]

        contracts = self.search_contracts(symbol, sec_type, exchange, currency)
        if contracts:
            conid = contracts[0].get("conid")
            if conid:
                self._conid_cache[cache_key] = int(conid)
                return int(conid)

        return None

    def get_market_data_snapshot(self, conid: int, fields: Optional[List[int]] = None) -> Dict:
        """
        Get market data snapshot for a contract ID (conid).

        Args:
            conid: Contract ID from search_contracts()
            fields: List of field IDs to request (default: bid, ask, last, close, volume)
                   Common fields: 31=bid, 55=ask, 84=last, 86=close, 7309=volume

        Returns:
            Dictionary with field values keyed by field ID
        """
        if fields is None:
            # Default fields: bid (31), ask (55), last (84), close (86), volume (7309)
            fields = [31, 55, 84, 86, 7309]

        self.ensure_session()
        # Convert conid to string and fields to comma-separated string
        fields_str = ",".join(str(f) for f in fields)
        endpoint = f"/iserver/marketdata/snapshot"
        params = {
            "conids": str(conid),
            "fields": fields_str,
        }

        try:
            # GET with query params
            url = f"{self.base_url}{endpoint}"
            response = self.session.get(url, params=params, timeout=self.timeout)
            if not response.ok:
                raise IBKRPortalError(
                    f"Client Portal responded with status {response.status_code}: {response.text[:200]}"
                )
            data = response.json()

            # IB returns a list, take first result
            if isinstance(data, list) and len(data) > 0:
                return data[0]

            return {}
        except IBKRPortalError:
            logger.warning(f"Market data snapshot failed for conid {conid}")
            return {}

    def get_snapshot(self, symbol: str) -> Dict[str, float]:
        """
        Get market data snapshot for a symbol (convenience method).
        Returns dict with bid, ask, last, close, volume.

        Returns:
            Dictionary with keys: bid, ask, last, close, volume
        """
        conid = self.get_conid(symbol)
        if not conid:
            return {
                "bid": 0.0,
                "ask": 0.0,
                "last": 0.0,
                "close": 0.0,
                "volume": 0.0,
            }

        # Field IDs: 31=bid, 55=ask, 84=last, 86=close, 7309=volume
        snapshot = self.get_market_data_snapshot(conid, fields=[31, 55, 84, 86, 7309])

        # Extract field values (IB returns fields as dictionary with numeric keys as strings)
        result = {
            "bid": 0.0,
            "ask": 0.0,
            "last": 0.0,
            "close": 0.0,
            "volume": 0.0,
        }

        # IB snapshot format: {"31": "509.15", "55": "509.18", ...}
        if isinstance(snapshot, dict):
            fields_map = snapshot.get("31", {})  # Sometimes nested
            if isinstance(fields_map, dict):
                result["bid"] = float(fields_map.get("31", 0) or 0)
                result["ask"] = float(fields_map.get("55", 0) or 0)
                result["last"] = float(fields_map.get("84", 0) or 0)
                result["close"] = float(fields_map.get("86", 0) or 0)
                result["volume"] = float(fields_map.get("7309", 0) or 0)
            else:
                # Flat format: {"31": "509.15", "55": "509.18", ...}
                result["bid"] = float(snapshot.get("31", 0) or 0)
                result["ask"] = float(snapshot.get("55", 0) or 0)
                result["last"] = float(snapshot.get("84", 0) or 0)
                result["close"] = float(snapshot.get("86", 0) or 0)
                result["volume"] = float(snapshot.get("7309", 0) or 0)

        return result

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _choose_account(self, explicit: Optional[str]) -> List[str]:
        accounts = self.get_accounts()
        if explicit:
            return [explicit]
        for preferred in self.preferred_accounts:
            if preferred in accounts:
                return [preferred]
        return accounts[:1]

    def _call(self, method: str, endpoint: str, raise_for_status: bool = True) -> requests.Response:
        url = f"{self.base_url}{endpoint}"
        try:
            response = self.session.request(method, url, timeout=self.timeout)
        except requests.RequestException as exc:  # pragma: no cover - network error
            raise IBKRPortalError(f"Client Portal request to {endpoint} failed: {exc}") from exc

        if raise_for_status and not response.ok:
            raise IBKRPortalError(
                f"Client Portal responded with status {response.status_code}: {response.text[:200]}"
            )
        return response
