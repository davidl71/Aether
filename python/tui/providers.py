"""
Data providers for TUI

These providers fetch snapshot data from various sources (REST API, file, mock, NATS, etc.)
and can be shared between Python TUI and PWA (via REST API).
"""
from __future__ import annotations

import asyncio
import json
import logging
import socket
import time
import threading
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Optional, Dict, Any, List
from datetime import datetime, timezone

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from .models import SnapshotPayload

logger = logging.getLogger(__name__)

try:
    import nats
    NATS_PY_AVAILABLE = True
except ImportError:
    nats = None  # type: ignore
    NATS_PY_AVAILABLE = False


def _connection_error_hint(exc: Exception, url: str) -> Optional[str]:
    """Return a short hint when the error is a connection failure (e.g. service not running)."""
    if "8002" in url or ":8002" in url:
        return "Start IB service: ./scripts/service.sh start ib"
    if "8000" in url or ":8000" in url:
        return "Start Alpaca service: ./scripts/service.sh start alpaca"
    return None


# Backend services that expose GET /api/health (same shape: status, ib_connected?, error?)
BACKEND_HEALTH_TIMEOUT_SEC = 2.0
BACKEND_HEALTH_TIMEOUT_IB_SEC = 5.0  # IB can be slow (gateway round-trip)
BACKEND_HEALTH_INTERVAL_SEC = 2.5


class BackendHealthAggregator:
    """
    Polls /api/health for HTTP backends and TCP connect for socket-only backends (e.g. TWS/Gateway).
    When unified_health_url is set, fetches that URL first (expected shape: { "backends": { name: health } })
    and uses it for HTTP backends; falls back to per-backend polling on failure.
    """

    def __init__(
        self,
        backend_ports: Dict[str, int],
        tcp_backend_ports: Optional[Dict[str, int]] = None,
        interval_sec: float = BACKEND_HEALTH_INTERVAL_SEC,
        unified_health_url: Optional[str] = None,
    ):
        # Keep references so in-place updates (e.g. TWS port 7497 -> 7496 from Setup) are seen on next poll
        self._backend_ports = backend_ports if backend_ports is not None else {}
        self._tcp_backend_ports = tcp_backend_ports if tcp_backend_ports is not None else {}
        self._interval_sec = interval_sec
        self._unified_health_url = (unified_health_url or "").strip() or None
        self._healths: Dict[str, Dict[str, Any]] = {}
        self._lock = threading.Lock()
        self._thread: Optional[threading.Thread] = None
        self._running = False
        self._session = requests.Session()

    def start(self) -> None:
        if self._running or (not self._unified_health_url and not self._backend_ports and not self._tcp_backend_ports):
            return
        self._running = True
        self._thread = threading.Thread(target=self._poll_loop, daemon=True)
        self._thread.start()
        all_backends = list(self._backend_ports.keys()) + list(self._tcp_backend_ports.keys())
        if self._unified_health_url:
            all_backends = list(set(all_backends + ["dashboard"]))
        logger.info(f"BackendHealthAggregator started for backends: {all_backends}")

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
            # Prefer unified health dashboard when configured
            if self._unified_health_url:
                try:
                    r = self._session.get(
                        self._unified_health_url,
                        timeout=BACKEND_HEALTH_TIMEOUT_SEC * 2,
                    )
                    if r.ok:
                        data = r.json()
                        backends = data.get("backends")
                        if isinstance(backends, dict):
                            now_iso = datetime.now(timezone.utc).isoformat()
                            with self._lock:
                                for name, payload in backends.items():
                                    if isinstance(payload, dict):
                                        self._healths[name] = {**dict(payload), "updated_at": payload.get("updated_at") or now_iso}
                            # Supplement: poll any HTTP backends that are not in the dashboard
                            # (e.g. IB when dashboard gets updates only via NATS and IB hasn't published yet)
                            for name, port in list(self._backend_ports.items()):
                                if name in self._healths:
                                    continue
                                if not self._running:
                                    break
                                url = f"http://127.0.0.1:{port}/api/health"
                                timeout = BACKEND_HEALTH_TIMEOUT_IB_SEC if name == "ib" else BACKEND_HEALTH_TIMEOUT_SEC
                                try:
                                    r2 = self._session.get(url, timeout=timeout)
                                    if r2.ok:
                                        try:
                                            data2 = r2.json()
                                        except Exception:
                                            data2 = {}
                                        if not isinstance(data2, dict):
                                            data2 = {}
                                    else:
                                        try:
                                            data2 = r2.json()
                                        except Exception:
                                            data2 = {}
                                        data2 = data2 if isinstance(data2, dict) else {}
                                        data2["status"] = "error"
                                        data2["ib_connected"] = False
                                        data2["error"] = data2.get("error") or f"HTTP {r2.status_code}"
                                    data2.setdefault("updated_at", datetime.now(timezone.utc).isoformat())
                                    with self._lock:
                                        self._healths[name] = data2
                                except Exception as e:
                                    logger.debug("Backend %s (supplement) unreachable: %s", name, e)
                                    with self._lock:
                                        self._healths[name] = {
                                            "status": "error",
                                            "ib_connected": False,
                                            "error": str(e),
                                            "hint": "Retrying…",
                                            "updated_at": datetime.now(timezone.utc).isoformat(),
                                        }
                            # Still run TCP backends (not in dashboard)
                            for name, port in list(self._tcp_backend_ports.items()):
                                if not self._running:
                                    break
                                try:
                                    with socket.create_connection(("127.0.0.1", port), timeout=2.0):
                                        pass
                                    with self._lock:
                                        self._healths[name] = {"status": "ok", "updated_at": datetime.now(timezone.utc).isoformat()}
                                except Exception as e:
                                    logger.debug("TCP backend %s unreachable: %s", name, e)
                                    with self._lock:
                                        self._healths[name] = {"status": "error", "error": str(e), "hint": "Retrying…", "updated_at": datetime.now(timezone.utc).isoformat()}
                            time.sleep(self._interval_sec)
                            continue
                except Exception as e:
                    logger.debug("Unified health fetch failed: %s; falling back to per-backend poll", e)
            # HTTP backends: GET /api/health
            for name, port in list(self._backend_ports.items()):
                if not self._running:
                    break
                url = f"http://127.0.0.1:{port}/api/health"
                timeout = BACKEND_HEALTH_TIMEOUT_IB_SEC if name == "ib" else BACKEND_HEALTH_TIMEOUT_SEC
                try:
                    r = self._session.get(url, timeout=timeout)
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
                    data["updated_at"] = data.get("updated_at") or datetime.now(timezone.utc).isoformat()
                    with self._lock:
                        self._healths[name] = data
                except Exception as e:
                    logger.debug("Backend %s unreachable: %s", name, e)
                    with self._lock:
                        self._healths[name] = {
                            "status": "error",
                            "ib_connected": False,
                            "error": str(e),
                            "hint": "Retrying…",
                            "updated_at": datetime.now(timezone.utc).isoformat(),
                        }
            # TCP-only backends (e.g. TWS/Gateway on 7497): socket connect
            for name, port in list(self._tcp_backend_ports.items()):
                if not self._running:
                    break
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=2.0):
                        pass
                    with self._lock:
                        self._healths[name] = {"status": "ok", "updated_at": datetime.now(timezone.utc).isoformat()}
                except Exception as e:
                    logger.debug("TCP backend %s unreachable: %s", name, e)
                    with self._lock:
                        self._healths[name] = {"status": "error", "error": str(e), "hint": "Retrying…", "updated_at": datetime.now(timezone.utc).isoformat()}
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
    Mock provider that generates synthetic snapshots for testing.

    When symbols is provided (e.g. from config.watchlist), generates data for those
    symbols so the dashboard watchlist and snapshot stay in sync.
    """

    def __init__(
        self,
        update_interval_ms: int = 1000,
        symbols: Optional[List[str]] = None,
    ):
        super().__init__()
        self.update_interval_ms = update_interval_ms
        self._worker_thread: Optional[threading.Thread] = None
        self._symbols = (
            list(symbols)
            if symbols
            else ["SPX", "XSP", "NDX"]
        )

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
    Optional snapshot_cache_path and backend_id enable persisting the latest
    snapshot to SQLite so the UI has something to display when the backend is
    down or on next launch.
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
        # Load last persisted snapshot so we have something to display immediately (or when backend is down)
        if self._backend_id:
            try:
                from ..integration.snapshot_store import get_latest
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
            # Prefer cached: return immediately so UI stays responsive (never block on network)
            if self._latest_snapshot is not None:
                return self._latest_snapshot
            return SnapshotPayload()

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
        """Background thread that polls the REST endpoint and health. Never raises; retries on connection loss."""
        while self._running:
            try:
                snapshot = self._fetch()
                if snapshot is not None:
                    with self._lock:
                        self._latest_snapshot = snapshot
                    # Persist so we have something to display when backend is down or on next launch
                    if self._backend_id:
                        try:
                            from ..integration.snapshot_store import save_latest
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
            # Refresh less often outside US regular market hours (9:30–16:00 ET)
            if self._out_of_market_interval_ms > 0:
                try:
                    from ..integration.market_hours import effective_refresh_interval_ms
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
        """Fetch snapshot from REST endpoint. Returns None on connection/error so caller keeps last snapshot."""
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


class NatsProvider(Provider):
    """
    NATS pub/sub provider: subscribes to snapshot.{backend_id} and optionally system.health.
    Updates UI on each message (no polling). Requires NATS_URL and a backend (e.g. IB) publishing snapshots.
    """

    def __init__(
        self,
        nats_url: str = "nats://localhost:4222",
        snapshot_backend: str = "ib",
    ):
        super().__init__()
        self.nats_url = nats_url.strip() or "nats://localhost:4222"
        self.snapshot_backend = (snapshot_backend or "ib").strip().lower()
        self._worker_thread: Optional[threading.Thread] = None
        self._loop: Optional[asyncio.AbstractEventLoop] = None
        self._stop_event: Optional[asyncio.Event] = None
        self._last_health: Optional[dict] = None
        self._health_lock = threading.Lock()

    def start(self) -> None:
        if self._running:
            return
        if not NATS_PY_AVAILABLE:
            logger.warning("nats-py not installed - NatsProvider disabled. pip install nats-py")
            return
        self._running = True
        self._worker_thread = threading.Thread(target=self._run_loop, daemon=True)
        self._worker_thread.start()
        logger.info(f"NatsProvider started: {self.nats_url} snapshot.{self.snapshot_backend}")

    def stop(self) -> None:
        self._running = False
        if self._loop and self._stop_event:
            self._loop.call_soon_threadsafe(self._stop_event.set)
        if self._worker_thread:
            self._worker_thread.join(timeout=3.0)
        self._loop = None
        self._stop_event = None
        logger.info("NatsProvider stopped")

    def get_snapshot(self) -> SnapshotPayload:
        with self._lock:
            if self._latest_snapshot is not None:
                return self._latest_snapshot
            return SnapshotPayload()

    def get_health(self) -> Optional[dict]:
        with self._health_lock:
            return dict(self._last_health) if self._last_health else None

    def is_running(self) -> bool:
        return self._running

    def _run_loop(self) -> None:
        """Background thread: run asyncio loop, connect to NATS, subscribe until stop."""
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        self._loop = loop
        stop_ev = asyncio.Event()
        self._stop_event = stop_ev
        try:
            loop.run_until_complete(self._subscribe_until_stop(stop_ev))
        except Exception as e:
            logger.error("NatsProvider loop error: %s", e)
        finally:
            self._loop = None
            self._stop_event = None
            loop.close()

    async def _subscribe_until_stop(self, stop_ev: asyncio.Event) -> None:
        nc = None
        if nats is None:
            with self._health_lock:
                self._last_health = {"status": "error", "error": "nats-py not installed"}
            await stop_ev.wait()
            return
        try:
            nc = await nats.connect(
                servers=[self.nats_url],
                reconnect_time_wait=2,
                max_reconnect_attempts=-1,
            )
        except Exception as e:
            logger.error("NatsProvider connect failed: %s", e)
            with self._health_lock:
                self._last_health = {"status": "error", "error": str(e)}
            await stop_ev.wait()
            return

        with self._health_lock:
            self._last_health = {"status": "ok", "backend": self.snapshot_backend}

        snapshot_subject = f"snapshot.{self.snapshot_backend}"

        async def on_snapshot(msg):
            try:
                data = json.loads(msg.data.decode("utf-8"))
                payload = SnapshotPayload.from_dict(data)
                with self._lock:
                    self._latest_snapshot = payload
            except Exception as e:
                logger.debug("NatsProvider snapshot decode: %s", e)

        async def on_health(msg):
            try:
                data = json.loads(msg.data.decode("utf-8"))
                if data.get("backend") == self.snapshot_backend:
                    with self._health_lock:
                        self._last_health = data
            except Exception as e:
                logger.debug("NatsProvider health decode: %s", e)

        await nc.subscribe(snapshot_subject, cb=on_snapshot)
        await nc.subscribe("system.health", cb=on_health)
        logger.info("Subscribed to %s and system.health", snapshot_subject)
        await stop_ev.wait()
        await nc.drain()
