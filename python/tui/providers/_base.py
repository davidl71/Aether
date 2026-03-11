"""Abstract Provider base class, connection hint helper, and REST URL utilities."""
from __future__ import annotations

import logging
import threading
from abc import ABC, abstractmethod
from typing import Optional

from ..models import SnapshotPayload

logger = logging.getLogger(__name__)

# Backend health poll timeouts
BACKEND_HEALTH_TIMEOUT_SEC = 2.0
BACKEND_HEALTH_TIMEOUT_IB_SEC = 5.0  # IB can be slow (gateway round-trip)
BACKEND_HEALTH_INTERVAL_SEC = 2.5


def _connection_error_hint(exc: Exception, url: str) -> Optional[str]:
    """Return a short hint when the error is a connection failure (e.g. service not running)."""
    if "8002" in url or ":8002" in url:
        return "Use the Rust backend IB routes on :8080; the standalone IB service is retired"
    return None


def normalize_rest_endpoint(url: str) -> str:
    """Ensure REST URL points at a snapshot endpoint (e.g. .../api/snapshot or .../api/v1/snapshot)."""
    if not url or not url.strip():
        return "http://127.0.0.1:8080/api/v1/snapshot"
    url = url.strip().rstrip("/")
    if "/api/" in url and ("/snapshot" in url or "/v1/snapshot" in url):
        return url
    if url.endswith("/api") or url.endswith("/api/v1"):
        return f"{url}/snapshot"
    if "/api" not in url:
        return f"{url}/api/v1/snapshot"
    return f"{url}/snapshot"


class Provider(ABC):
    """
    Abstract base class for data providers.

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

    def get_health(self) -> Optional[dict]:
        """Optional provider health payload for providers that expose it."""
        return None
