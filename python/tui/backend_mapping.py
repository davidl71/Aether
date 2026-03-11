"""Shared mappings between TUI provider types and backend keys."""

from __future__ import annotations

from typing import Dict


BACKEND_KEY_TO_PROVIDER_TYPE: Dict[str, str] = {
    "ib": "rest_ib",
    "tws": "rest_tws_gateway",
    "alpaca": "rest_alpaca",
    "tastytrade": "rest_tastytrade",
    "mock": "mock",
    "nats": "nats",
    "file": "file",
}

PROVIDER_TYPE_TO_BACKEND_KEY: Dict[str, str] = {
    provider_type: backend_key
    for backend_key, provider_type in BACKEND_KEY_TO_PROVIDER_TYPE.items()
}
