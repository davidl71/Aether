"""
ibkr_portal_client.py - Lightweight wrapper around IBKR Client Portal Web API.

The Client Portal Gateway must be running locally. Authentication typically
requires establishing a browser session once; this client focuses on
maintaining the session (validate / reauthenticate) and exposes helpers for
account and portfolio data retrieval.

Environment:
- REAUTH_SLEEP_SECONDS: seconds to sleep after triggering reauth (default 0.5; clamp 0.1-2.0).
"""
from __future__ import annotations

import logging
import os
import time
from concurrent.futures import ThreadPoolExecutor
from typing import Dict, List, Optional

import requests


logger = logging.getLogger(__name__)

# Short TTL for in-memory accounts list to avoid repeated gateway round-trips during snapshot build
ACCOUNTS_CACHE_TTL_SECONDS = 2.0


def _reauth_sleep_seconds() -> float:
    """Seconds to sleep after triggering reauth (configurable via REAUTH_SLEEP_SECONDS)."""
    try:
        val = float(os.getenv("REAUTH_SLEEP_SECONDS", "0.5"))
        return max(0.1, min(2.0, val))
    except (ValueError, TypeError):
        return 0.5


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
        self._accounts_cached_at: float = 0.0
        self._conid_cache: Dict[str, int] = {}  # symbol -> conid cache
        # Request-scoped: when True, ensure_session() is a no-op (caller ensured once).
        self._session_ensured_for_request: bool = False

    def set_session_ensured_for_request(self, value: bool) -> None:
        """Set whether session has already been ensured for this request (skip ensure_session in workers)."""
        self._session_ensured_for_request = value

    # ------------------------------------------------------------------
    # Session Helpers
    # ------------------------------------------------------------------

    def ensure_session(self, timeout: Optional[int] = None) -> None:
        """
        Validate the current session, requesting re-auth only if necessary.

        The IB Client Portal Gateway uses browser-based authentication. When you log in
        via the browser, the gateway session is active. API clients can use this session,
        but may need to trigger re-authentication to establish their own session token.

        This method tries to use the gateway first. Only if that fails do we trigger
        re-authentication, and only if we're confident the gateway is running.

        When _session_ensured_for_request is True (set by caller after ensuring once),
        returns immediately without making requests.
        """
        if self._session_ensured_for_request:
            return

        # First, try to access a protected endpoint to see if we can use the gateway
        # If gateway is authenticated via browser, this should work
        accounts_resp = self._call(
            "GET", "/iserver/accounts", raise_for_status=False, timeout=timeout
        )
        if accounts_resp.status_code == 200:
            logger.debug("IBKR gateway is authenticated and accessible")
            return

        # If accounts endpoint returns 401, we need authentication
        if accounts_resp.status_code == 401:
            try:
                auth_status_resp = self._call(
                    "GET", "/iserver/auth/status", raise_for_status=False, timeout=timeout
                )
                if auth_status_resp.status_code in (200, 401):
                    logger.info("IBKR gateway requires API client authentication")
                    logger.info("Triggering re-authentication - if already logged in via browser, you may need to approve")
                    reauth_resp = self._call(
                        "POST", "/iserver/reauthenticate", raise_for_status=False, timeout=timeout
                    )
                    if reauth_resp.status_code in (200, 202):
                        logger.info("Re-authentication triggered successfully")
                        time.sleep(_reauth_sleep_seconds())
                        verify_resp = self._call(
                            "GET", "/iserver/accounts", raise_for_status=False, timeout=timeout
                        )
                        if verify_resp.status_code == 200:
                            logger.info("Re-authentication successful, gateway accessible")
                            return
                        logger.warning("Re-authentication triggered but accounts endpoint still returns %d", verify_resp.status_code)
                        return
                    raise IBKRPortalError(
                        f"Reauthentication failed (status={reauth_resp.status_code}): {reauth_resp.text[:200]}"
                    )
            except IBKRPortalError:
                raise
            except Exception as e:
                logger.debug(f"Gateway connectivity check failed: {e}")
                raise IBKRPortalError(
                    f"Unable to connect to IB Client Portal Gateway: {e}. "
                    "Ensure the gateway is running at https://localhost:5000"
                ) from e

        raise IBKRPortalError(
            f"Unexpected response from IB Client Portal Gateway "
            f"(status={accounts_resp.status_code}): {accounts_resp.text[:200]}"
        )

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def get_accounts(self, timeout: Optional[int] = None) -> List[str]:
        """Return list of tradeable account IDs. Uses short TTL cache to avoid repeated gateway round-trips."""
        now = time.monotonic()
        if self._cached_accounts and (now - self._accounts_cached_at) < ACCOUNTS_CACHE_TTL_SECONDS:
            return self._cached_accounts

        # Single request: if 200 we're done; if 401 run ensure_session and retry once
        resp = self._call("GET", "/iserver/accounts", raise_for_status=False, timeout=timeout)
        if resp.status_code == 200:
            data = resp.json()
            accounts = data.get("accounts", []) if isinstance(data, dict) else []
            if accounts:
                self._cached_accounts = accounts
                self._accounts_cached_at = time.monotonic()
            return accounts

        if resp.status_code == 401:
            self.ensure_session(timeout=timeout)
            resp = self._call("GET", "/iserver/accounts", timeout=timeout)
            data = resp.json()
            accounts = data.get("accounts", []) if isinstance(data, dict) else []
            self._cached_accounts = accounts
            self._accounts_cached_at = time.monotonic()
            return accounts

        raise IBKRPortalError(
            f"Client Portal responded with status {resp.status_code}: {resp.text[:200]}"
        )

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
            fields = [31, 55, 84, 86, 7309]

        self.ensure_session()
        fields_str = ",".join(str(f) for f in fields)
        endpoint = "/iserver/marketdata/snapshot"
        params = {"conids": str(conid), "fields": fields_str}

        try:
            url = f"{self.base_url}{endpoint}"
            response = self.session.get(url, params=params, timeout=self.timeout)
            if not response.ok:
                raise IBKRPortalError(
                    f"Client Portal responded with status {response.status_code}: {response.text[:200]}"
                )
            data = response.json()

            if isinstance(data, list) and len(data) > 0:
                return data[0]

            return {}
        except IBKRPortalError:
            logger.warning(f"Market data snapshot failed for conid {conid}")
            return {}

    def get_market_data_snapshots_batch(
        self, conids: List[int], fields: Optional[List[int]] = None
    ) -> List[Dict]:
        """
        Get market data snapshots for multiple conids in one request.

        Returns:
            List of snapshot dicts in the same order as conids.
        """
        if not conids:
            return []

        if fields is None:
            fields = [31, 55, 84, 86, 7309]

        self.ensure_session()
        conids_str = ",".join(str(c) for c in conids)
        fields_str = ",".join(str(f) for f in fields)
        endpoint = "/iserver/marketdata/snapshot"
        params = {"conids": conids_str, "fields": fields_str}

        try:
            url = f"{self.base_url}{endpoint}"
            response = self.session.get(url, params=params, timeout=self.timeout)
            if not response.ok:
                raise IBKRPortalError(
                    f"Client Portal responded with status {response.status_code}: {response.text[:200]}"
                )
            data = response.json()

            if isinstance(data, list):
                # IB returns one object per conid in order; pad if shorter
                result: List[Dict] = []
                for i, conid in enumerate(conids):
                    if i < len(data) and isinstance(data[i], dict):
                        result.append(data[i])
                    else:
                        result.append({})
                return result

            return [{}] * len(conids)
        except IBKRPortalError:
            logger.warning("Market data snapshot batch failed for %s conids", len(conids))
            return [{}] * len(conids)

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

    def get_snapshots_batch(self, symbols: List[str]) -> List[Dict[str, float]]:
        """
        Get market data snapshots for multiple symbols with one batched request when possible.
        Resolves conids in parallel for uncached symbols, then one batch snapshot call.

        Returns:
            List of dicts with keys bid, ask, last, close, volume, in same order as symbols.
        """
        if not symbols:
            return []

        def conid_for(sym: str) -> tuple:
            conid = self.get_conid(sym)
            return (sym, conid)

        with ThreadPoolExecutor(max_workers=min(8, len(symbols))) as pool:
            conids_by_symbol = list(pool.map(conid_for, symbols))

        conids_ordered = [c for _, c in conids_by_symbol]
        valid_conids = [c for c in conids_ordered if c is not None]
        if not valid_conids:
            return [
                {"bid": 0.0, "ask": 0.0, "last": 0.0, "close": 0.0, "volume": 0.0}
                for _ in symbols
            ]

        batch = self.get_market_data_snapshots_batch(valid_conids)
        conid_to_snapshot = {valid_conids[i]: batch[i] for i in range(len(valid_conids))}

        empty = {"bid": 0.0, "ask": 0.0, "last": 0.0, "close": 0.0, "volume": 0.0}

        def to_result(snap: Dict) -> Dict[str, float]:
            if not isinstance(snap, dict):
                return dict(empty)

            def f(k: str) -> float:
                v = snap.get(k, 0)
                try:
                    return float(v) if v else 0.0
                except (ValueError, TypeError):
                    return 0.0

            return {
                "bid": f("31"),
                "ask": f("55"),
                "last": f("84"),
                "close": f("86"),
                "volume": f("7309"),
            }

        results: List[Dict[str, float]] = []
        for sym, conid in conids_by_symbol:
            if conid is not None and conid in conid_to_snapshot:
                results.append(to_result(conid_to_snapshot[conid]))
            else:
                results.append(dict(empty))

        return results

    def prewarm_conids(self, symbols: List[str]) -> None:
        """
        Resolve conids for the given symbols so subsequent get_snapshots_batch calls
        avoid per-symbol search_contracts round-trips. Populates in-memory _conid_cache.
        """
        if not symbols:
            return
        for sym in symbols:
            try:
                self.get_conid(sym)
            except IBKRPortalError:
                logger.debug("Prewarm conid failed for %s", sym)

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _choose_account(self, explicit: Optional[str]) -> List[str]:
        # Fast path: caller already resolved account (e.g. from get_accounts() in same request).
        if explicit is not None and str(explicit).strip():
            return [str(explicit).strip()]
        accounts = self.get_accounts()
        for preferred in self.preferred_accounts:
            if preferred in accounts:
                return [preferred]
        return accounts[:1] if accounts else []

    def _call(
        self,
        method: str,
        endpoint: str,
        raise_for_status: bool = True,
        timeout: Optional[int] = None,
    ) -> requests.Response:
        url = f"{self.base_url}{endpoint}"
        to = timeout if timeout is not None else self.timeout
        try:
            response = self.session.request(method, url, timeout=to)
        except requests.RequestException as exc:  # pragma: no cover - network error
            raise IBKRPortalError(f"Client Portal request to {endpoint} failed: {exc}") from exc

        if raise_for_status and not response.ok:
            raise IBKRPortalError(
                f"Client Portal responded with status {response.status_code}: {response.text[:200]}"
            )
        return response
