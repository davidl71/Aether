# Where NATS Is Used and Where Else We Could Use It

Summary of current NATS usage and opportunities to add NATS (including QuestDB). See also `docs/NATS_TOPICS_REGISTRY.md` for the topic schema.

---

## Current NATS usage

| Role | Components |
|------|------------|
| **Publishers** | Rust backend (market data, strategy signals/decisions), C++ TWS client (when `ENABLE_NATS`), Python strategy runner |
| **Subscribers** | Web PWA (WebSocket via `nats.ws`), optional Python consumers |
| **Server** | `config/nats-server.conf`, started via `scripts/service.sh` or `start_nats.sh` |
| **Bridge** | Go `nats-questdb-bridge` (JetStream → QuestDB ILP) – see QuestDB section below |

---

## Opportunities to use NATS

### 1. TUI snapshot (replace or augment REST/file polling)

**Today:** `RestProvider` and `FileProvider` poll REST or a file every 500ms–1s.

**With NATS:** Add a NATS provider that subscribes to e.g. `snapshot.ib` or `snapshot.aggregated`. Backends publish when they have a new snapshot; TUI updates on message instead of polling.

---

### 2. TUI backend health (replace health polling)

**Today:** `BackendHealthAggregator` polls each backend’s `/api/health` every 2.5s.

**With NATS:** Each backend publishes to `system.health` (or `health.{backend}`) on a timer; the aggregator subscribes to `health.>` and updates the status line from messages.

---

### 3. Backend services publishing snapshots

**Today:** IB/Alpaca/TradeStation only serve snapshot via REST (and optionally write a file for the TUI).

**With NATS:** When a service builds a new snapshot, also publish it on e.g. `snapshot.ib`, `snapshot.alpaca`, `snapshot.tradestation`. TUI/PWA can subscribe; an aggregator can merge.

---

### 4. Unified snapshot aggregator

**Today:** TUI shows one REST backend; multi-backend is via health aggregator + one snapshot source.

**With NATS:** A small service subscribes to `snapshot.ib`, `snapshot.alpaca`, `snapshot.tradestation`, merges into one payload, and publishes to `snapshot.aggregated` or serves a single REST `/api/snapshot`. TUI/PWA then need only one subscription or one REST endpoint.

---

### 5. Alerts

**Today:** Alerts are embedded in snapshot payloads and shown in `AlertsTab`.

**With NATS:** Backends and strategy publish to `system.alerts` when something happens (e.g. wide spread, margin warning). TUI and PWA subscribe to `system.alerts` for real-time alerts without waiting for the next snapshot poll.

---

### 6. Orders and positions

**Registry:** `orders.status.{id}`, `orders.fill.{id}`, `positions.update.{symbol}`, `positions.snapshot`.

**Today:** Order/position state is only in REST snapshot or internal state.

**With NATS:** When the order manager or TWS client updates an order or position, publish to the appropriate topic. TUI and PWA subscribe to `orders.status.>`, `orders.fill.>`, `positions.update.>` for live updates.

---

### 7. Strategy control and status

**Registry:** `strategy.control`, `strategy.status`, `rpc.strategy.status`.

**With NATS:** Frontends publish start/stop/pause to `strategy.control` and subscribe to `strategy.status`; request/reply for current status via `rpc.strategy.status`.

---

### 8. RPC snapshot (on-demand)

**Registry:** `rpc.system.snapshot`.

**With NATS:** When the UI needs a snapshot “now”, it can do a NATS request to `rpc.system.snapshot` and get a one-off reply instead of or in addition to GET `/api/snapshot`.

---

### 9. Risk and config

**Registry:** `risk.limit.{type}`, `risk.violation`, `system.config`.

**With NATS:** Risk engine publishes limit/violation events; UIs and logging subscribe. Config changes can be broadcast on `system.config` so all processes react without polling.

---

### 10. QuestDB (evaluation)

**Today:**

- **Python path:** `questdb_client.py` writes quotes and trades to QuestDB via ILP (TCP 9009). `strategy_runner.py` and `market_data_handler.py` call the client directly when `questdb.enabled`; data does **not** flow through NATS.
- **NATS → QuestDB (implemented):** `python/integration/questdb_nats_writer.py` subscribes to **Core NATS** `market-data.tick.>`, parses **JSON** ticks (envelope or flat), and writes each message to QuestDB via ILP (table `market_data`). Run via `./scripts/run_questdb_nats_writer.sh` or `./scripts/service.sh start questdb_nats`. Supports JSON payloads; Rust backend currently publishes **protobuf** (NatsEnvelope + MarketDataEvent) on the same subject — add Python proto decode or a small adapter to republish proto→JSON if you need to ingest from Rust.
- **Go bridge:** `agents/go/cmd/nats-questdb-bridge` subscribes to **NATS JetStream** (stream `MARKET_DATA`, subject `market.data.>`), parses JSON ticks, and writes each message to QuestDB via ILP. So NATS → QuestDB is already implemented for the JetStream path.
- **Mismatch:** The rest of the system uses **Core NATS** topics `market-data.tick.{symbol}` (and protobuf or JSON envelope from Rust). The Go bridge expects JetStream and subject pattern `market.data.>` with a specific JSON shape (`symbol`, `bid`, `ask`, `last`, `volume`, `timestamp`). So today the bridge is only useful if something publishes into that JetStream stream/subject.

**Opportunities:**

| Option | Description |
|--------|-------------|
| **A. Unify ingestion via NATS** | Have all market data producers (Rust backend, C++ TWS when NATS enabled, Python strategy) publish to a **single** topic pattern. Either (i) use Core NATS `market-data.tick.{symbol}` and add a **NATS → QuestDB writer** (Rust or Python) that subscribes to that pattern and writes ILP to QuestDB, or (ii) have producers also (or only) publish to JetStream `market.data.{symbol}` in the JSON shape the Go bridge expects, and run the existing bridge. Then QuestDB is fed from one pipeline instead of separate Python direct writes. |
| **B. Use existing Go bridge** | Enable JetStream in `config/nats-server.conf` (already present). Add a small adapter that subscribes to Core NATS `market-data.tick.>` (or the protobuf topic), converts to the bridge’s JSON format, and republishes to JetStream `market.data.{symbol}`. Run `nats-questdb-bridge` so QuestDB gets all ticks. |
| **C. Strategy/order/position events to QuestDB** | Publish strategy decisions, order fills, and position updates on NATS (see sections 6–7). A single subscriber (e.g. Rust or Go) subscribes to those topics and writes to QuestDB (e.g. `strategy_decisions`, `order_fills`, `position_updates` tables). Gives one audit trail and one place for backtests/analytics. |
| **D. Snapshot / health to QuestDB** | If snapshots or health events are published on NATS (sections 1–4), a subscriber could periodically or on-change write aggregated snapshots or health state to QuestDB for historical dashboards or debugging. |

**Recommendation:** Use **A** or **B** so that all market ticks flow through NATS and QuestDB is populated from one NATS-driven pipeline. Optionally add **C** (and **D**) so QuestDB holds not only ticks but decisions, fills, and positions for analysis. That keeps QuestDB in the evaluation as the time-series store fed by NATS rather than by direct Python calls from each producer.

---

### 11. Memcached (evaluation)

**Today:**

- **C++:** Optional market data cache backend when `ENABLE_MEMCACHED` is ON and libmemcached is available (`native/src/cache_client_memcached.cpp`, `native/include/cache_client.h`). Uses get/set/del with TTL; typically keyed by symbol or composite keys for market data. Started via `scripts/service.sh start memcached` (port 11211).
- **Python:** `python/integration/cache_client.py` provides a `CacheClient` protocol with Redis and Memcached backends (`MemcachedStateCache` via pymemcache). Same get/set/delete + TTL semantics; used for state/snapshot-style caching where a backend is configured.
- **No NATS link:** Memcached is used as a dumb key-value store; nothing today publishes cache invalidation or cache-warm events over NATS.

**Opportunities:**

| Option | Description |
|--------|-------------|
| **A. Cache invalidation over NATS** | When data that is cached in memcached changes (e.g. config update, account switch, snapshot rebuild), the writer publishes to a topic such as `cache.invalidate.{scope}` or `cache.invalidate.snapshot.ib`. Any process (Python services, C++ engine, or a dedicated cache layer) that shares memcached subscribes and deletes or refreshes the relevant keys. Keeps caches consistent across instances without polling. |
| **B. Snapshot cache shared via memcached + NATS** | IB (and other) services use in-memory snapshot cache today. If snapshot were written to memcached (shared across restarts and multiple consumers), NATS could carry `snapshot.updated.{backend}` when a new snapshot is written. TUI or aggregator subscribes and then reads from memcached by key, or invalidates local view so the next read hits memcached. Reduces duplicate work and keeps UI in sync. |
| **C. Market data cache consistency** | C++ engine (or any producer) that writes market data into memcached could publish a message on NATS (e.g. `market-data.tick.{symbol}` or `cache.warm.market.{symbol}`) when it updates the cache. Subscribers that also use memcached for market data know to refresh or can rely on the same keys; alternatively, a single NATS subscriber updates memcached from the NATS stream so that both NATS-driven and cache-only readers see the same data. |
| **D. Health and presence** | If memcached is a shared dependency, services could publish `system.health` or `cache.backend.memcached` with status (e.g. healthy/unhealthy). Monitoring or a cache-router could subscribe and switch to a fallback or alert when memcached is down. |

**Recommendation:** **A** is the highest leverage: use NATS to broadcast cache invalidation (and optionally warm) events so that all consumers of memcached (C++, Python, any future Go/Rust) stay in sync without tight coupling. **B** is useful once snapshot is published on NATS (see sections 3–4) and you want a shared snapshot cache in memcached. **C** fits if market data flows through NATS and you want one writer to memcached for both NATS subscribers and cache-only readers.

---

## Summary table

| Area | Today | With NATS |
|------|--------|-----------|
| TUI snapshot | Poll REST/file | Subscribe to `snapshot.*` or `snapshot.aggregated` |
| TUI health | Poll each /api/health | Subscribe to `system.health` or `health.>` |
| Backend snapshot | REST only | Publish to `snapshot.{backend}` when updated |
| Aggregated view | N/A or ad hoc | Service subscribes to all `snapshot.*`, merges, publishes/serves one |
| Alerts | Inside snapshot | Publish `system.alerts`; UIs subscribe |
| Orders/positions | In snapshot only | Publish `orders.*`, `positions.*`; UIs subscribe |
| Strategy control | REST? | Publish/subscribe `strategy.control`, `strategy.status` |
| On-demand snapshot | GET /api/snapshot | Request/reply `rpc.system.snapshot` |
| **QuestDB** | Python direct ILP; Go bridge (JetStream) unused with current subjects | Unify ticks on NATS; one NATS→QuestDB pipeline (adapter + existing bridge or new writer); optional events (decisions, fills, positions) to QuestDB |
| **Memcached** | C++/Python use as dumb K/V cache; no cross-process coordination | NATS for cache invalidation / warm (`cache.invalidate.*`, `snapshot.updated.*`); optional shared snapshot or market-data cache with NATS-driven updates |
