"""
Data providers for TUI — package re-exports.

Each provider is in its own module:
  _base.py          — abstract Provider + factory utilities
  _health.py        — BackendHealthAggregator
  _mock.py          — MockProvider
  _rest.py          — RestProvider  (P1-B: routes via api-gateway :9000)
  _file.py          — FileProvider
  _nats.py          — NatsProvider

MIGRATION PLAN:
- RestProvider presets and default endpoint go through Go api-gateway (:9000); gateway proxies to Rust or Python backends.
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
from ._nats import NatsProvider, NATS_PY_AVAILABLE

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
    "NATS_PY_AVAILABLE",
]
