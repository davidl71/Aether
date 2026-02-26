"""
redis_cache.py - Optional Redis state cache for shared service state.

Uses NATS KV as fallback if Redis is unavailable.
"""
from __future__ import annotations

import json
import logging
from typing import Any, Dict, Optional

logger = logging.getLogger(__name__)

try:
    import redis
    REDIS_AVAILABLE = True
except ImportError:
    REDIS_AVAILABLE = False


class RedisStateCache:
    """Thin wrapper around Redis for JSON get/set with TTL."""

    def __init__(
        self,
        host: str = "127.0.0.1",
        port: int = 6379,
        db: int = 0,
        prefix: str = "ib:",
        default_ttl: int = 300,
    ) -> None:
        if not REDIS_AVAILABLE:
            raise RuntimeError("redis package not installed: pip install redis")
        self._client = redis.Redis(host=host, port=port, db=db, decode_responses=True)
        self._prefix = prefix
        self._ttl = default_ttl

    def _key(self, key: str) -> str:
        return f"{self._prefix}{key}"

    def get(self, key: str) -> Optional[Dict[str, Any]]:
        raw = self._client.get(self._key(key))
        if raw is None:
            return None
        try:
            return json.loads(raw)
        except json.JSONDecodeError:
            return None

    def set(self, key: str, value: Dict[str, Any], ttl: Optional[int] = None) -> None:
        self._client.set(
            self._key(key),
            json.dumps(value),
            ex=ttl or self._ttl,
        )

    def delete(self, key: str) -> None:
        self._client.delete(self._key(key))

    def is_healthy(self) -> bool:
        try:
            return self._client.ping()
        except Exception:
            return False
