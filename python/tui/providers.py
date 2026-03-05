"""
Data providers for TUI

These providers fetch snapshot data from various sources (REST API, file, mock, etc.)
and can be shared between Python TUI and PWA (via REST API).

MIGRATION NOTES FOR FUTURE C++ MIGRATION (pybind11):
- Provider interface can be exposed as abstract C++ class via pybind11
- Python providers can call C++ implementations through pybind11 bindings
- Consider keeping Python providers as thin wrappers around C++ core logic
- REST/file providers can remain in Python (or use C++ HTTP libraries)
"""

from __future__ import annotations

import json
import logging
import time
import threading
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Optional, Dict, Any
from datetime import datetime, timezone

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from .models import SnapshotPayload

logger = logging.getLogger(__name__)


# Backend services that expose GET /api/health (same shape: status, ib_connected?, error?)
BACKEND_HEALTH_TIMEOUT_SEC = 2.0
BACKEND_HEALTH_INTERVAL_SEC = 2.5


class BackendHealthAggregator:
    """
    Polls /api/health for multiple backend services and exposes aggregated health.
    Used by the TUI to display all backend provider status in the status line.
    """

    def __init__(self, backend_ports: Dict[str, int], interval_sec: float = BACKEND_HEALTH_INTERVAL_SEC):
        self._backend_ports = dict(backend_ports)
        self._interval_sec = interval_sec
        self._healths: Dict[str, Dict[str, Any]] = {}
        self._lock = threading.Lock()
        self._thread: Optional[threading.Thread] = None
        self._running = False
        self._session = requests.Session()

    def start(self) -> None:
        if self._running or not self._backend_ports:
            return
        self._running = True
        self._thread = threading.Thread(target=self._poll_loop, daemon=True)
        self._thread.start()
        logger.info(f"BackendHealthAggregator started for backends: {list(self._backend_ports.keys())}")

    def stop(self) -> None:
        self._running = False
        if self._thread:
            self._thread.join(timeout=self._interval_sec + 1.0)
        logger.info("BackendHealthAggregator stopped")

    def get_all_health(self) -> Dict[str, Dict[str, Any]]:
        """Return current health dict for all backends (name -> health payload)."""
        with self._lock:
            return dict(self._healths)

    def _poll_loop(self) -> None:
        while self._running:
            for name, port in list(self._backend_ports.items()):
                if not self._running:
                    break
                url = f"http://127.0.0.1:{port}/api/health"
                try:
                    r = self._session.get(url, timeout=BACKEND_HEALTH_TIMEOUT_SEC)
                    if r.ok:
                        data = r.json()
                    else:
                        try:
                            data = r.json()
                        except Exception:
                            data = {}
                        data["status"] = "error"
                        data["ib_connected"] = False
                        data["error"] = data.get("error") or f"HTTP {r.status_code}"
                        logger.debug("Backend %s unreachable: %s", name, data["error"])
                    with self._lock:
                        self._healths[name] = data
                except Exception as e:
                    logger.debug("Backend %s unreachable: %s", name, e)
                    with self._lock:
                        self._healths[name] = {
                            "status": "error",
                            "ib_connected": False,
                            "error": str(e),
                        }
            time.sleep(self._interval_sec)


def normalize_rest_endpoint(url: str) -> str:
    """Ensure REST URL points at a snapshot endpoint (e.g. .../api/snapshot or .../api/v1/snapshot)."""
    if not url or not url.strip():
        return "http://127.0.0.1:8002/api/snapshot"
    url = url.strip().rstrip("/")
    if "/api/" in url and ("/snapshot" in url or "/v1/snapshot" in url):
        return url
    if url.endswith("/api") or url.endswith("/api/v1"):
        return f"{url}/snapshot"
    if "/api" not in url:
        return f"{url}/api/snapshot"
    return f"{url}/snapshot"


class Provider(ABC):
    """
    Abstract base class for data providers

    MIGRATION NOTE: This interface can be exposed to C++ via pybind11 as:
    class Provider {
    public:
        virtual ~Provider() = default;
        virtual void Start() = 0;
        virtual void Stop() = 0;
        virtual SnapshotPayload GetSnapshot() = 0;
        virtual bool IsRunning() const = 0;
    };
    """

    def __init__(self):
        self._running = False
        self._lock = threading.Lock()
        self._latest_snapshot: Optional[SnapshotPayload] = None

    @abstractmethod
    def start(self) -> None:
        """Start the provider (begin emitting snapshots)"""
        pass

    @abstractmethod
    def stop(self) -> None:
        """Stop the provider"""
        pass

    @abstractmethod
    def get_snapshot(self) -> SnapshotPayload:
        """
        Get the latest snapshot (non-blocking, returns empty snapshot if none available)

        MIGRATION NOTE: In C++, this would return by value or const reference
        """
        pass

    @abstractmethod
    def is_running(self) -> bool:
        """Check if provider is running"""
        pass


class MockProvider(Provider):
    """
    Mock provider that generates synthetic snapshots for testing

    MIGRATION NOTE: Can be replaced with C++ MockProvider class via pybind11
    """

    def __init__(self, update_interval_ms: int = 1000):
        super().__init__()
        self.update_interval_ms = update_interval_ms
        self._worker_thread: Optional[threading.Thread] = None
        self._symbols = ["SPX", "XSP", "NDX"]

    def start(self) -> None:
        if self._running:
            return
        self._running = True
        self._worker_thread = threading.Thread(target=self._generate_loop, daemon=True)
        self._worker_thread.start()
        logger.info("MockProvider started")

    def stop(self) -> None:
        self._running = False
        if self._worker_thread:
            self._worker_thread.join(timeout=2.0)
        logger.info("MockProvider stopped")

    def get_snapshot(self) -> SnapshotPayload:
        with self._lock:
            if self._latest_snapshot is None:
                return self._generate_snapshot()
            return self._latest_snapshot

    def is_running(self) -> bool:
        return self._running

    def add_symbol(self, symbol: str) -> None:
        """Add a symbol to the mock provider's rotation"""
        if symbol not in self._symbols:
            self._symbols.append(symbol)

    def _generate_loop(self) -> None:
        """Background thread that generates snapshots"""
        while self._running:
            snapshot = self._generate_snapshot()
            with self._lock:
                self._latest_snapshot = snapshot
            time.sleep(self.update_interval_ms / 1000.0)

    def _generate_snapshot(self) -> SnapshotPayload:
        """Generate a synthetic snapshot"""
        now = datetime.now(timezone.utc).isoformat()

        # Generate mock symbols
        symbols = []
        for i, symbol in enumerate(self._symbols):
            base_price = 4000.0 + (i * 100.0)
            symbols.append({
                "symbol": symbol,
                "last": base_price + (i * 0.5),
                "bid": base_price + (i * 0.3),
                "ask": base_price + (i * 0.7),
                "spread": 0.4,
                "roi": 2.5 + (i * 0.5),
                "maker_count": i + 1,
                "taker_count": i,
                "volume": 1000 + (i * 100),
                "candle": {
                    "open": base_price,
                    "high": base_price + 1.0,
                    "low": base_price - 1.0,
                    "close": base_price + 0.5,
                    "volume": 1000,
                    "entry": base_price,
                    "updated": now
                },
                "option_chains": []
            })

        return SnapshotPayload.from_dict({
            "generated_at": now,
            "mode": "DRY-RUN",
            "strategy": "RUNNING",
            "account_id": "DU123456",
            "metrics": {
                "net_liq": 100000.0,
                "buying_power": 50000.0,
                "excess_liquidity": 45000.0,
                "margin_requirement": 5000.0,
                "commissions": 0.0,
                "portal_ok": True,
                "tws_ok": True,
                "orats_ok": True,
                "questdb_ok": True
            },
            "symbols": symbols,
            "positions": [],
            "historic": [],
            "orders": [],
            "alerts": []
        })


class RestProvider(Provider):
    """
    REST API provider that polls an HTTP endpoint for snapshots.
    When the endpoint is an IB service (e.g. .../api/snapshot), also polls
    .../api/health and exposes get_health() for UI status (e.g. IB connected).
    """

    def __init__(
        self,
        endpoint: str,
        update_interval_ms: int = 1000,
        timeout_ms: int = 15000,
        verify_ssl: bool = True,
    ):
        super().__init__()
        self.endpoint = normalize_rest_endpoint(endpoint)
        self.update_interval_ms = update_interval_ms
        self.timeout_sec = timeout_ms / 1000.0
        self._verify_ssl = verify_ssl
        self._worker_thread: Optional[threading.Thread] = None
        self._health_url = self._derive_health_url(endpoint)
        self._last_health: Optional[dict] = None
        self._health_lock = threading.Lock()

        # Configure requests session with retry strategy
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
            if self._latest_snapshot is None:
                # Try to fetch immediately if no snapshot available
                return self._fetch()
            return self._latest_snapshot

    def is_running(self) -> bool:
        return self._running

    @staticmethod
    def _derive_health_url(snapshot_endpoint: str) -> Optional[str]:
        """Derive /api/health URL from snapshot endpoint (e.g. .../api/snapshot -> origin/api/health)."""
        if not snapshot_endpoint or not snapshot_endpoint.strip():
            return None
        from urllib.parse import urlparse
        parsed = urlparse(snapshot_endpoint.strip().rstrip("/"))
        if not parsed.scheme or not parsed.netloc:
            return None
        origin = f"{parsed.scheme}://{parsed.netloc}"
        return f"{origin}/api/health"

    def get_health(self) -> Optional[dict]:
        """Return last health response from backend, if available (RestProvider only)."""
        with self._health_lock:
            return dict(self._last_health) if self._last_health else None

    def _fetch_health(self) -> None:
        """Fetch health from backend and store in _last_health."""
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
            logger.debug(f"Health check failed: {e}")
            with self._health_lock:
                self._last_health = {"status": "error", "ib_connected": False, "error": str(e)}

    def _poll_loop(self) -> None:
        """Background thread that polls the REST endpoint and health."""
        while self._running:
            try:
                snapshot = self._fetch()
                with self._lock:
                    self._latest_snapshot = snapshot
            except Exception as e:
                logger.error(f"Failed to fetch snapshot: {e}")
            self._fetch_health()
            time.sleep(self.update_interval_ms / 1000.0)

    def _fetch(self) -> SnapshotPayload:
        """Fetch snapshot from REST endpoint"""
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
            logger.error(f"REST fetch error: {e}")
            # Return empty snapshot on error
            return SnapshotPayload()


class FileProvider(Provider):
    """
    File provider that reads snapshots from a JSON file on disk

    MIGRATION NOTE: File I/O can be done in C++ using std::filesystem and nlohmann/json
    """

    def __init__(self, file_path: str, update_interval_ms: int = 1000):
        super().__init__()
        self.file_path = Path(file_path)
        self.update_interval_ms = update_interval_ms
        self._worker_thread: Optional[threading.Thread] = None
        self._last_mtime: float = 0.0

    def start(self) -> None:
        if self._running:
            return
        self._running = True
        self._worker_thread = threading.Thread(target=self._poll_loop, daemon=True)
        self._worker_thread.start()
        logger.info(f"FileProvider started: {self.file_path}")

    def stop(self) -> None:
        self._running = False
        if self._worker_thread:
            self._worker_thread.join(timeout=2.0)
        logger.info("FileProvider stopped")

    def get_snapshot(self) -> SnapshotPayload:
        with self._lock:
            if self._latest_snapshot is None:
                return self._load_from_file()
            return self._latest_snapshot

    def is_running(self) -> bool:
        return self._running

    def _poll_loop(self) -> None:
        """Background thread that polls the file"""
        while self._running:
            try:
                if self.file_path.exists():
                    current_mtime = self.file_path.stat().st_mtime
                    if current_mtime > self._last_mtime:
                        snapshot = self._load_from_file()
                        with self._lock:
                            self._latest_snapshot = snapshot
                        self._last_mtime = current_mtime
            except Exception as e:
                logger.error(f"File poll error: {e}")
            time.sleep(self.update_interval_ms / 1000.0)

    def _load_from_file(self) -> SnapshotPayload:
        """Load snapshot from JSON file"""
        try:
            if not self.file_path.exists():
                logger.warning(f"Snapshot file not found: {self.file_path}")
                return SnapshotPayload()

            with open(self.file_path, 'r') as f:
                data = json.load(f)
            return SnapshotPayload.from_dict(data)
        except Exception as e:
            logger.error(f"File load error: {e}")
            return SnapshotPayload()
