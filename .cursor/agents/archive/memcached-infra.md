# Agent C: Memcached & Infrastructure

## Role

Add memcached to the project infrastructure (Ansible provisioning) and create a CacheClient abstraction layer in Python and C++ that supports both Redis and Memcached backends.

## Tasks

1. **Add memcached to Ansible provisioning** (`T-1772136087097427000`)
   - Update `ansible/roles/devtools/tasks/main.yml`:
     - macOS (Homebrew): add `memcached`, `libmemcached`
     - Debian/Ubuntu (apt): add `memcached`, `libmemcached-dev`
   - Update `playbooks/global.yml`:
     - Add `memcached` and `libmemcached` to `brew_packages` list
   - Add `pymemcache` to Python dependencies (requirements file or uv)

2. **Create CacheClient abstraction with memcached backend** (`T-1772136087069653000`)
   - **Python** -- Create `python/integration/cache_client.py`:
     - Define `CacheClient` protocol/ABC with methods: `get(key) -> Optional[dict]`, `set(key, value, ttl)`, `delete(key)`, `is_healthy() -> bool`
     - Implement `MemcachedStateCache` using `pymemcache` (HashClient for multi-server)
     - Refactor existing `RedisStateCache` in `python/integration/redis_cache.py` to implement the same `CacheClient` protocol
     - Factory function: `create_cache_client(backend="memcached"|"redis", **kwargs) -> CacheClient`
     - Fall back gracefully if memcached/redis unavailable
   - **C++** -- Create `native/include/cache_client.h` and `native/src/cache_client.cpp`:
     - Abstract `CacheClient` class with virtual methods: `get(key)`, `set(key, value, ttl)`, `remove(key)`, `is_connected()`
     - `MemcachedClient` implementation using libmemcached
     - `NullCacheClient` no-op fallback when no cache is available
     - Key format: `ib:<type>:<identifier>`, TTL from `config_manager.h` `cache_duration_seconds`

## Reference: Existing Redis Cache

```python
# python/integration/redis_cache.py (65 lines)
class RedisStateCache:
    def __init__(self, host, port, db, prefix="ib:", default_ttl=300)
    def get(self, key) -> Optional[Dict]
    def set(self, key, value, ttl=None)
    def delete(self, key)
    def is_healthy(self) -> bool
```

## Client Libraries

| Language | Library | Install |
|----------|---------|---------|
| Python | `pymemcache` | `uv pip install pymemcache` |
| C++ | `libmemcached` | `brew install libmemcached` / `apt install libmemcached-dev` |
| Go | `gomemcache` | `go get github.com/bradfitz/gomemcache/memcache` (future task) |

## Files You Own (exclusive)

- `ansible/roles/devtools/tasks/main.yml`
- `playbooks/global.yml`
- `python/integration/cache_client.py` (new file)
- `python/integration/redis_cache.py` (refactor to implement CacheClient)
- `native/include/cache_client.h` (new file)
- `native/src/cache_client.cpp` (new file)

## Files You Must NOT Touch

- `native/src/tws_client.cpp` (owned by Agent E)
- `native/CMakeLists.txt` (owned by Agent E)
- `scripts/` (owned by Agent B)
- `Justfile` (owned by Agents A, B, D)
- `proto/` (owned by Agent D)

## Completion Criteria

- [ ] `ansible/roles/devtools/tasks/main.yml` includes memcached packages for both macOS and Debian
- [ ] `python/integration/cache_client.py` exists with `CacheClient` protocol, `MemcachedStateCache`, and factory function
- [ ] `native/include/cache_client.h` exists with abstract `CacheClient` and `MemcachedClient`
- [ ] `native/src/cache_client.cpp` exists with implementation
- [ ] Both exarp tasks marked Done
