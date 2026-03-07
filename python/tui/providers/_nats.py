"""NatsProvider: subscribes to NATS snapshot topics for event-driven UI updates."""
from __future__ import annotations

import asyncio
import json
import logging
import threading
from typing import Optional

from ..models import SnapshotPayload
from ._base import Provider

logger = logging.getLogger(__name__)

try:
    import nats
    NATS_PY_AVAILABLE = True
except ImportError:
    nats = None  # type: ignore
    NATS_PY_AVAILABLE = False

# NOTE: Tests that patch NATS availability should use:
#   @patch("tui.providers._nats.NATS_PY_AVAILABLE", False)
# instead of the old @patch("tui.providers.NATS_PY_AVAILABLE", False)


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
