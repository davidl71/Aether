"""
nats_kv_state.py - NATS JetStream Key-Value as primary shared state store.

Use for current account, current mode, and other small shared state so IB, Alpaca,
Tastytrade, TUI, and PWA see the same values without running Redis.

Bucket: ib_box_spread_state (configurable). Keys are JSON values; optional TTL via bucket config.
Requires: NATS server with JetStream enabled (see docs/NATS_SETUP.md).

Usage:
  from integration.nats_kv_state import get_nats_kv_state, NATS_KV_STATE_BUCKET

  state = await get_nats_kv_state()
  if state:
      await state.set("current_account:ib", {"account_id": "DU123"})
      data = await state.get("current_account:ib")
"""

from __future__ import annotations

import json
import logging
import os
from typing import Any, Dict, Optional

logger = logging.getLogger(__name__)

try:
    import nats
    from nats.js.errors import BucketNotFoundError
    NATS_AVAILABLE = True
except ImportError:
    nats = None
    BucketNotFoundError = Exception  # type: ignore
    NATS_AVAILABLE = False

# Default bucket name for shared state (current account, mode, etc.)
NATS_KV_STATE_BUCKET = os.environ.get("NATS_KV_STATE_BUCKET", "ib_box_spread_state")
DEFAULT_HISTORY = 10  # Keep last 10 revisions per key for debugging
_global_kv: Optional["NATSKVState"] = None


class NATSKVState:
    """
    Async NATS KV state store: get/set/delete with JSON values.
    Uses JetStream Key-Value bucket; creates bucket if missing (when create_if_missing=True).
    """

    def __init__(
        self,
        bucket: str = NATS_KV_STATE_BUCKET,
        url: str = "",
        create_if_missing: bool = True,
        history: int = DEFAULT_HISTORY,
    ):
        self._bucket_name = bucket
        self._url = (url or os.environ.get("NATS_URL", "nats://localhost:4222")).strip()
        self._create_if_missing = create_if_missing
        self._history = history
        self._nc: Any = None
        self._kv: Any = None

    async def connect(self) -> bool:
        """Connect to NATS and bind to KV bucket. Creates bucket if missing and create_if_missing."""
        if not NATS_AVAILABLE:
            logger.warning("nats-py not available - NATS KV state disabled")
            return False
        try:
            self._nc = await nats.connect(
                servers=[self._url],
                reconnect_time_wait=2,
                max_reconnect_attempts=-1,
            )
            js = self._nc.jetstream()
            try:
                self._kv = await js.key_value(self._bucket_name)
            except BucketNotFoundError:
                if self._create_if_missing:
                    self._kv = await js.create_key_value(
                        bucket=self._bucket_name,
                        history=self._history,
                    )
                    logger.info("Created NATS KV bucket %s", self._bucket_name)
                else:
                    logger.warning("NATS KV bucket %s not found", self._bucket_name)
                    await self._nc.close()
                    return False
            logger.info("NATS KV state connected (bucket=%s)", self._bucket_name)
            return True
        except Exception as e:
            logger.warning("NATS KV connect failed: %s", e)
            if self._nc:
                try:
                    await self._nc.close()
                except Exception:
                    pass
            return False

    async def close(self) -> None:
        """Close NATS connection."""
        if self._nc:
            try:
                await self._nc.drain()
                await self._nc.close()
            except Exception as e:
                logger.debug("NATS KV close: %s", e)
            self._nc = None
            self._kv = None

    def is_healthy(self) -> bool:
        """True if connected and KV bucket is bound."""
        return self._nc is not None and self._kv is not None

    async def get(self, key: str) -> Optional[Dict[str, Any]]:
        """Get key value as JSON dict. Returns None if key missing or not JSON."""
        if not self._kv:
            return None
        try:
            entry = await self._kv.get(key)
            if entry is None or entry.value is None:
                return None
            return json.loads(entry.value.decode("utf-8"))
        except (KeyError, json.JSONDecodeError, Exception) as e:
            logger.debug("NATS KV get %s: %s", key, e)
            return None

    async def set(self, key: str, value: Dict[str, Any]) -> bool:
        """Set key to JSON-serialized value. Returns True on success."""
        if not self._kv:
            return False
        try:
            body = json.dumps(value).encode("utf-8")
            await self._kv.put(key, body)
            return True
        except Exception as e:
            logger.warning("NATS KV set %s: %s", key, e)
            return False

    async def delete(self, key: str) -> bool:
        """Delete key (soft delete with tombstone). Returns True on success."""
        if not self._kv:
            return False
        try:
            await self._kv.delete(key)
            return True
        except Exception as e:
            logger.debug("NATS KV delete %s: %s", key, e)
            return False


async def get_nats_kv_state(
    bucket: str = NATS_KV_STATE_BUCKET,
    url: str = "",
    create_if_missing: bool = True,
) -> Optional[NATSKVState]:
    """
    Return a connected NATSKVState instance, or None if NATS_URL unset or connection fails.
    Uses a module-level singleton so multiple callers share the same connection.
    """
    global _global_kv
    if not url and not os.environ.get("NATS_URL", "").strip():
        return None
    if _global_kv is not None and _global_kv.is_healthy():
        return _global_kv
    if _global_kv is not None:
        try:
            await _global_kv.close()
        except Exception:
            pass
        _global_kv = None
    state = NATSKVState(bucket=bucket, url=url or None, create_if_missing=create_if_missing)
    if await state.connect():
        _global_kv = state
        return state
    return None


def _state_key(backend: str, name: str) -> str:
    """Canonical key for backend state (e.g. current_account:ib, current_mode:alpaca)."""
    return f"{name}:{backend}"


async def get_current_account(backend: str) -> Optional[str]:
    """Get current account id for backend (ib, alpaca, tastytrade) from NATS KV."""
    state = await get_nats_kv_state()
    if not state:
        return None
    data = await state.get(_state_key(backend, "current_account"))
    if isinstance(data, dict) and "account_id" in data:
        return data["account_id"]
    return None


async def set_current_account(backend: str, account_id: str) -> bool:
    """Set current account id for backend in NATS KV."""
    state = await get_nats_kv_state()
    if not state:
        return False
    return await state.set(_state_key(backend, "current_account"), {"account_id": account_id})
