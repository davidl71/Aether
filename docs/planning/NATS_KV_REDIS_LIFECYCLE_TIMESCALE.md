# NATS KV → Redis → Background-Task Lifecycle → TimescaleDB

Prioritized plan for shared state, caching, task management, and analytics storage.

**Order:** NATS KV first → Redis later → background-task lifecycle → TimescaleDB (optional).

---

## 1. NATS KV first (current focus)

**Goal:** Use NATS JetStream Key-Value as the primary shared state store so all services (IB, Alpaca, Tastytrade, TUI, PWA) see the same “current account” and “current mode” without adding another process.

**Why first:**
- NATS is already in the stack (messaging, health dashboard, QuestDB bridge).
- One less service than Redis; same NATS URL for pub/sub and state.
- Key history (revisions) useful for debugging and audit.
- Fits “one bus + one state store” for the platform.

**Implementation:**
- **Bucket:** `ib_box_spread_state` (or `NATS_KV_STATE_BUCKET`). Keys: `current_account:ib`, `current_account:alpaca`, `current_account:tastytrade`, etc.
- **Module:** `python/integration/nats_kv_state.py` – async get/set/delete with JSON values; bucket created on first connect if missing.
- **Helpers:** `get_nats_kv_state()`, `get_current_account(backend)`, `set_current_account(backend, account_id)`.
- **Wiring:** In backend services (ib_service, alpaca_service, tastytrade_service), when setting or reading current account, call `get_nats_kv_state()` and use `state.get("current_account:ib")` / `state.set(...)` (or the helpers) so all processes see the same value. Fallback remains in-memory until wired.
- **JetStream:** Must be enabled in NATS server (see `config/nats-server.conf` and `docs/NATS_SETUP.md`).

**References:** `python/integration/redis_cache.py` (NATS KV as fallback – flip to NATS KV primary), `python/integration/cache_client.py` (CacheClient protocol).

---

## 2. Redis later

**Goal:** Add Redis only when needed for lower latency or richer structures.

**When to add:**
- Conid/symbol cache with very high read volume and need for minimal latency.
- Rich structures (hashes, lists, sorted sets) without encoding everything in JSON in NATS KV.
- Heavy cache eviction (e.g. LRU) or rate limiting.

**Integration:** Keep `cache_client.py` and `redis_cache.py`. Offer a **unified state factory** that returns NATS KV when `NATS_URL` is set and Redis when `REDIS_URL` (or similar) is set and NATS KV is unavailable or not desired for that use case.

---

## 3. Background-task lifecycle

**Goal:** Structured concurrency for Python services so background tasks (polling, health, snapshot refresh) are started and stopped in one place and shutdown is clean.

**Scope:**
- TUI: RestProvider, BackendHealthAggregator, FileProvider worker threads; single place to stop all on exit.
- Backend services (IB, Alpaca, etc.): health/snapshot loops; ensure one coordinator (e.g. FastAPI lifespan or an anyio task group) starts/stops them.
- Optional: use `anyio` or `asyncio` TaskGroup so all tasks are children of one scope and cancel together.

**Deliverables:**
- Document pattern (e.g. “start all background work in lifespan, cancel on shutdown”).
- Refactor one service (e.g. IB or health dashboard) as reference; then apply to others.

**See:** **`docs/planning/BACKGROUND_TASK_LIFECYCLE.md`** for current state (TUI, FastAPI, scripts), gaps, and suggested improvements (task registry, lifespan migration, strategy-runner task cancel, signal handlers).

---

## 4. TimescaleDB (optional)

**Goal:** Single SQL database for both relational data (accounts, config, ledger) and time-series (ticks, PnL, decisions) if we want to consolidate analytics.

**When to consider:**
- Need to run SQL over both “current state” and “time-series” in one place.
- Prefer one operational DB (TimescaleDB) over QuestDB + PostgreSQL (or SQLite) for reporting.

**Alternative:** Keep current split: QuestDB for time-series, SQLite/PostgreSQL for ledger/relational; add TimescaleDB only if consolidation wins.

---

## Summary table

| Priority | Component              | Purpose                          | Status   |
|----------|------------------------|----------------------------------|----------|
| 1        | NATS KV                | Shared state (account, mode)     | Implemented (`nats_kv_state.py`); wiring in backends optional |
| 2        | Redis                  | Optional low-latency / rich cache| Later    |
| 3        | Background-task lifecycle | Clean start/stop, one coordinator | After KV |
| 4        | TimescaleDB            | Optional unified SQL + time-series | Optional |

---

## Logic Unify (single message bus + cache order)

**Message bus:** NATS (Core + JetStream) is the single message bus for the platform. Use one topic registry (`docs/NATS_TOPICS_REGISTRY.md`) and one wire format (protobuf for NATS payloads). Do not introduce a second bus (e.g. RabbitMQ, Kafka) for the same use cases unless there is a clear reason.

**Cache/state order:** Use a unified client (e.g. `CacheClient` protocol / state factory in `python/integration/cache_client.py`) so callers do not branch on "NATS vs Redis vs memcached" in business logic. Canonical order: (1) **NATS KV** when NATS is available – one URL for pub/sub and state. (2) **Redis** when richer structures (hashes, lists, TTL, sorted sets) or a dedicated cache server are needed. (3) **Memcached** only where already mandated or for pure key-value cache; implement behind the same abstraction so switching is config-driven. C++ engine: when `ENABLE_NATS` is on, any future market-data or strategy cache should go through a small adapter that can be swapped (NATS KV, Redis, or memcached).

**See:** `docs/design/LOGIC_WE_COULD_UNIFY.md` §9.

---

## See also

- `docs/NATS_SETUP.md` – Current NATS setup and QuestDB bridge wiring
- `docs/NATS_SETUP.md` – JetStream and KV bucket creation
- `python/integration/nats_client.py` – NATS connection and publish
- `python/integration/redis_cache.py` – Redis state cache (NATS KV fallback)
- `python/integration/cache_client.py` – CacheClient protocol (Redis/Memcached)
