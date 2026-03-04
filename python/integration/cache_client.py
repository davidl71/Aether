"""
cache_client.py - Abstract cache protocol with Redis and Memcached backends.

Provides a CacheClient protocol and factory for switching between
Redis and Memcached with identical JSON get/set + TTL semantics.
"""
from __future__ import annotations

import json
import logging
from typing import Any, Dict, Optional, Protocol, runtime_checkable

logger = logging.getLogger(__name__)


@runtime_checkable
class CacheClient(Protocol):
    """Protocol for JSON key-value caches with TTL."""

    def get(self, key: str) -> Optional[Dict[str, Any]]: ...
    def set(self, key: str, value: Dict[str, Any], ttl: Optional[int] = None) -> None: ...
    def delete(self, key: str) -> None: ...
    def is_healthy(self) -> bool: ...


class RedisStateCache:
    """Redis backend implementing CacheClient."""

    def __init__(
        self,
        host: str = "127.0.0.1",
        port: int = 6379,
        db: int = 0,
        prefix: str = "ib:",
        default_ttl: int = 300,
    ) -> None:
        try:
            import redis
        except ImportError as exc:
            raise RuntimeError("redis package not installed: pip install redis") from exc
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
        self._client.set(self._key(key), json.dumps(value), ex=ttl or self._ttl)

    def delete(self, key: str) -> None:
        self._client.delete(self._key(key))

    def is_healthy(self) -> bool:
        try:
            return self._client.ping()
        except Exception:
            return False


class MemcachedStateCache:
    """Memcached backend implementing CacheClient."""

    def __init__(
        self,
        host: str = "127.0.0.1",
        port: int = 11211,
        prefix: str = "ib:",
        default_ttl: int = 300,
    ) -> None:
        try:
            from pymemcache.client.base import Client
        except ImportError as exc:
            raise RuntimeError(
                "pymemcache not installed: pip install pymemcache"
            ) from exc
        self._client = Client((host, port), default_noreply=False)
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
        except (json.JSONDecodeError, UnicodeDecodeError):
            return None

    def set(self, key: str, value: Dict[str, Any], ttl: Optional[int] = None) -> None:
        self._client.set(
            self._key(key),
            json.dumps(value),
            expire=ttl or self._ttl,
        )

    def delete(self, key: str) -> None:
        self._client.delete(self._key(key))

    def is_healthy(self) -> bool:
        try:
            self._client.stats()
            return True
        except Exception:
            return False


def create_cache(
    backend: str = "redis",
    host: str = "127.0.0.1",
    prefix: str = "ib:",
    default_ttl: int = 300,
    **kwargs: Any,
) -> CacheClient:
    """Factory: create a cache client by backend name.

    Args:
        backend: "redis" or "memcached"
        host: Server hostname
        prefix: Key prefix for namespacing
        default_ttl: Default TTL in seconds
        **kwargs: Passed to the backend constructor (port, db, etc.)
    """
    if backend == "memcached":
        return MemcachedStateCache(
            host=host,
            port=kwargs.get("port", 11211),
            prefix=prefix,
            default_ttl=default_ttl,
        )
    return RedisStateCache(
        host=host,
        port=kwargs.get("port", 6379),
        db=kwargs.get("db", 0),
        prefix=prefix,
        default_ttl=default_ttl,
    )
