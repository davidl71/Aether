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

    # ------------------------------------------------------------------
    # Session Helpers
    # ------------------------------------------------------------------

    def ensure_session(self) -> None:
        """Validate the current session, requesting re-auth if required."""
        if self._call("GET", "/sso/validate", raise_for_status=False).status_code == 200:
            return

        logger.info("IBKR portal session invalid, attempting re-authentication")
        response = self._call("POST", "/iserver/reauthenticate", raise_for_status=False)
        if response.status_code not in (200, 202):
            raise IBKRPortalError(
                f"Reauthentication failed (status={response.status_code}): {response.text[:200]}"
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

