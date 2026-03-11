"""
Data providers for TUI — package re-exports.

Each provider is in its own module:
  _base.py          — abstract Provider + factory utilities
  _health.py        — BackendHealthAggregator
  _mock.py          — MockProvider
  _rest.py          — RestProvider  (shared Rust origin by default; gateway for optional specialist routes)
  _file.py          — FileProvider
  _nats.py          — NatsProvider

CURRENT SHAPE:
- RestProvider defaults to the shared Rust origin for frontend read models.
- Rust owns frontend read models.
- Go gateway remains only for operational aggregation and selected specialist-service routing.
- Longer term, some rest-backed reads may move to NATS KV watch or direct Rust ownership.
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
    "requests",
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
