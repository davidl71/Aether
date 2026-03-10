"""BackendHealthAggregator: polls /api/health for HTTP backends and TCP for socket-only backends."""
from __future__ import annotations

import logging
import socket
import threading
import time
from datetime import datetime, timezone
from typing import Optional, Dict, Any

import requests

from ._base import (
    BACKEND_HEALTH_TIMEOUT_SEC,
    BACKEND_HEALTH_TIMEOUT_IB_SEC,
    BACKEND_HEALTH_INTERVAL_SEC,
)

logger = logging.getLogger(__name__)


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
                            # Supplement: poll HTTP backends not in the dashboard
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
                            # TCP backends (not in dashboard)
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
