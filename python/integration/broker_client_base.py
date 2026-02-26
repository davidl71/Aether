"""
broker_client_base.py - Base class for REST-based broker/data clients.

Provides common session setup, HTTP helpers with retry/timeout,
environment variable credential loading, and structured logging.
"""
from __future__ import annotations

import logging
import os
import time
from typing import Any, Dict, Optional

import requests


class BrokerClientError(RuntimeError):
    """Base error for all broker client failures."""


class BrokerClientBase:
    """Shared foundation for REST-based broker and data clients.

    Subclasses override ``_configure`` to set up headers and auth.
    Use ``_get`` / ``_post`` / ``_put`` / ``_delete`` for API calls.
    """

    _default_timeout: int = 10
    _max_retries: int = 1
    _retry_backoff: float = 0.5

    def __init__(
        self,
        base_url: str,
        *,
        session: Optional[requests.Session] = None,
        timeout: Optional[int] = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self._timeout = timeout or self._default_timeout
        self._session = session or requests.Session()
        self._session.headers.setdefault("Accept", "application/json")
        self.logger = logging.getLogger(type(self).__module__)

    # ------------------------------------------------------------------
    # Credential helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _env(name: str, default: str = "") -> str:
        return os.getenv(name, default)

    @staticmethod
    def _require_env(name: str) -> str:
        val = os.getenv(name, "")
        if not val:
            raise BrokerClientError(f"Missing required environment variable: {name}")
        return val

    @staticmethod
    def _is_truthy(value: str) -> bool:
        return value.lower() in ("1", "true", "yes", "on")

    # ------------------------------------------------------------------
    # HTTP helpers
    # ------------------------------------------------------------------

    def _request(
        self,
        method: str,
        path: str,
        *,
        params: Optional[Dict] = None,
        json: Optional[Any] = None,
        data: Optional[Any] = None,
        headers: Optional[Dict] = None,
        timeout: Optional[int] = None,
        raise_for_status: bool = True,
    ) -> requests.Response:
        url = f"{self.base_url}{path}"
        last_exc: Optional[Exception] = None
        for attempt in range(self._max_retries + 1):
            try:
                resp = self._session.request(
                    method,
                    url,
                    params=params,
                    json=json,
                    data=data,
                    headers=headers,
                    timeout=timeout or self._timeout,
                )
                if raise_for_status:
                    resp.raise_for_status()
                return resp
            except requests.RequestException as exc:
                last_exc = exc
                if attempt < self._max_retries:
                    time.sleep(self._retry_backoff * (attempt + 1))
        raise last_exc  # type: ignore[misc]

    def _get(self, path: str, params: Optional[Dict] = None, **kw: Any) -> Dict:
        resp = self._request("GET", path, params=params, **kw)
        result = resp.json()
        return result if isinstance(result, dict) else {}

    def _get_list(self, path: str, params: Optional[Dict] = None, **kw: Any) -> Any:
        resp = self._request("GET", path, params=params, **kw)
        return resp.json()

    def _post(
        self, path: str, json: Optional[Any] = None, **kw: Any
    ) -> Dict:
        resp = self._request("POST", path, json=json, **kw)
        result = resp.json()
        return result if isinstance(result, dict) else {}

    def _put(self, path: str, json: Optional[Any] = None, **kw: Any) -> Dict:
        resp = self._request("PUT", path, json=json, **kw)
        result = resp.json()
        return result if isinstance(result, dict) else {}

    def _delete(self, path: str, **kw: Any) -> Dict:
        resp = self._request("DELETE", path, **kw)
        try:
            result = resp.json()
            return result if isinstance(result, dict) else {}
        except ValueError:
            return {}
