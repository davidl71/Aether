"""RestProvider: polls an HTTP endpoint for snapshots."""
from __future__ import annotations

import logging
import threading
import time
from pathlib import Path
from typing import Optional
from urllib.parse import urlparse

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from ..models import SnapshotPayload
from ._base import Provider, normalize_rest_endpoint, _connection_error_hint

logger = logging.getLogger(__name__)


class RestProvider(Provider):
    """
    REST API provider that polls an HTTP endpoint for snapshots.
    When the endpoint is an IB service (e.g. .../api/snapshot), also polls
    .../api/health and exposes get_health() for UI status (e.g. IB connected).

    P1-B: Presets and default endpoint use api-gateway (config.py); gateway proxies to Rust or Python backends.
    """

    def __init__(
        self,
        endpoint: str,
        update_interval_ms: int = 1000,
        timeout_ms: int = 15000,
        verify_ssl: bool = True,
        backend_id: Optional[str] = None,
        snapshot_cache_path: Optional[str] = None,
        out_of_market_interval_ms: int = 0,
    ):
        super().__init__()
        self.endpoint = normalize_rest_endpoint(endpoint)
        self.update_interval_ms = update_interval_ms
        self._out_of_market_interval_ms = max(0, out_of_market_interval_ms)
        self.timeout_sec = timeout_ms / 1000.0
        self._verify_ssl = verify_ssl
        self._backend_id = backend_id
        self._snapshot_cache_path = Path(snapshot_cache_path) if snapshot_cache_path else None
        self._worker_thread: Optional[threading.Thread] = None
        self._health_url = self._derive_health_url(endpoint)
        self._last_health: Optional[dict] = None
        self._health_lock = threading.Lock()

        self._session = requests.Session()
        retry_strategy = Retry(
            total=3,
            backoff_factor=0.1,
            status_forcelist=[429, 500, 502, 503, 504]
        )
        adapter = HTTPAdapter(max_retries=retry_strategy)
        self._session.mount("http://", adapter)
        self._session.mount("https://", adapter)

    def start(self) -> None:
        if self._running:
            return
        self._running = True
        if self._backend_id:
            try:
                from ...integration.snapshot_store import get_latest
                db_path = self._snapshot_cache_path if self._snapshot_cache_path else None
                data = get_latest(self._backend_id, db_path=db_path)
                if data:
                    with self._lock:
                        self._latest_snapshot = SnapshotPayload.from_dict(data)
                    logger.debug("RestProvider loaded cached snapshot for %s", self._backend_id)
            except Exception as e:
                logger.debug("RestProvider cache load skipped: %s", e)
        self._worker_thread = threading.Thread(target=self._poll_loop, daemon=True)
        self._worker_thread.start()
        logger.info(f"RestProvider started: {self.endpoint}")

    def stop(self) -> None:
        self._running = False
        if self._worker_thread:
            self._worker_thread.join(timeout=2.0)
        logger.info("RestProvider stopped")

    def get_snapshot(self) -> SnapshotPayload:
        with self._lock:
            if self._latest_snapshot is not None:
                return self._latest_snapshot
            return SnapshotPayload()

    def is_running(self) -> bool:
        return self._running

    @staticmethod
    def _derive_health_url(snapshot_endpoint: str) -> Optional[str]:
        """Derive /api/health URL from snapshot endpoint."""
        if not snapshot_endpoint or not snapshot_endpoint.strip():
            return None
        parsed = urlparse(snapshot_endpoint.strip().rstrip("/"))
        if not parsed.scheme or not parsed.netloc:
            return None
        origin = f"{parsed.scheme}://{parsed.netloc}"
        return f"{origin}/api/health"

    def get_health(self) -> Optional[dict]:
        """Return last health response from backend, if available."""
        with self._health_lock:
            return dict(self._last_health) if self._last_health else None

    def _fetch_health(self) -> None:
        if not self._health_url:
            return
        try:
            response = self._session.get(
                self._health_url,
                timeout=self.timeout_sec,
                headers={"Accept": "application/json"},
                verify=self._verify_ssl,
            )
            if response.ok:
                data = response.json()
            else:
                try:
                    data = response.json()
                except Exception:
                    data = {}
                data["status"] = "error"
                data["ib_connected"] = False
                data["error"] = data.get("error") or f"HTTP {response.status_code}"
            with self._health_lock:
                self._last_health = data
        except Exception as e:
            logger.debug("Health check failed (backend may be restarting): %s", e)
            hint = _connection_error_hint(e, self._health_url or "")
            if not hint:
                hint = "Retrying…"
            with self._health_lock:
                self._last_health = {
                    "status": "error",
                    "ib_connected": False,
                    "error": str(e),
                    "hint": hint,
                }

    def _poll_loop(self) -> None:
        while self._running:
            try:
                snapshot = self._fetch()
                if snapshot is not None:
                    with self._lock:
                        self._latest_snapshot = snapshot
                    if self._backend_id:
                        try:
                            from ...integration.snapshot_store import save_latest
                            db_path = self._snapshot_cache_path if self._snapshot_cache_path else None
                            save_latest(self._backend_id, snapshot.to_dict(), db_path=db_path)
                        except Exception as e:
                            logger.debug("RestProvider cache save skipped: %s", e)
            except Exception as e:
                logger.debug("RestProvider fetch error (will retry): %s", e)
            try:
                self._fetch_health()
            except Exception as e:
                logger.debug("RestProvider health check error: %s", e)
            if self._out_of_market_interval_ms > 0:
                try:
                    from ...integration.market_hours import effective_refresh_interval_ms
                    interval_ms = effective_refresh_interval_ms(
                        self.update_interval_ms,
                        self._out_of_market_interval_ms,
                    )
                except Exception:
                    interval_ms = self.update_interval_ms
            else:
                interval_ms = self.update_interval_ms
            time.sleep(interval_ms / 1000.0)

    def _fetch(self) -> Optional[SnapshotPayload]:
        try:
            response = self._session.get(
                self.endpoint,
                timeout=self.timeout_sec,
                headers={"Accept": "application/json"},
                verify=self._verify_ssl,
            )
            response.raise_for_status()
            data = response.json()
            return SnapshotPayload.from_dict(data)
        except Exception as e:
            logger.debug("REST fetch error (backend may be restarting): %s", e)
            return None
