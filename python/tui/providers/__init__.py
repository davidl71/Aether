"""
Data providers for TUI — package re-exports.

Each provider is in its own module:
  _base.py          — abstract Provider + factory utilities
  _health.py        — BackendHealthAggregator
  _mock.py          — MockProvider
  _rest.py          — RestProvider  (modify for api-gateway routing, task P1-B)
  _file.py          — FileProvider
  _nats.py          — NatsProvider

MIGRATION PLAN:
- RestProvider currently polls Python microservices directly on :8000-:8006 (1s interval).
- Web frontend reads from Rust backend :8080 — two different data pipelines, potential divergence.
- Target: Route RestProvider through the Go api-gateway (:8090).
  Task P1-B: exarp T-1772887221914991889 — docs/platform/IMPROVEMENT_PLAN.md
- Longer term: Replace RestProvider with NatsProvider using NATS KV watch.
  Epic E1: exarp T-1772887222509770969 — ConnectRPC streaming
"""
# Keep `import requests` at package level so existing test patches still work:
#   @patch('tui.providers.requests.Session')
import requests  # noqa: F401 — used by test patches

from ._base import (
    Provider,
    normalize_rest_endpoint,
    BACKEND_HEALTH_TIMEOUT_SEC,
    BACKEND_HEALTH_TIMEOUT_IB_SEC,
    BACKEND_HEALTH_INTERVAL_SEC,
)
from ._health import BackendHealthAggregator
from ._mock import MockProvider
from ._rest import RestProvider
from ._file import FileProvider
from ._nats import NatsProvider

__all__ = [
    "Provider",
    "normalize_rest_endpoint",
    "BACKEND_HEALTH_TIMEOUT_SEC",
    "BACKEND_HEALTH_TIMEOUT_IB_SEC",
    "BACKEND_HEALTH_INTERVAL_SEC",
    "BackendHealthAggregator",
    "MockProvider",
    "RestProvider",
    "FileProvider",
    "NatsProvider",
]
