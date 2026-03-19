# Market / position data persistence

**Last updated**: 2026-03-17  
**Purpose**: Where market data and position data are persisted and how they flow.

---

## Summary

| Data | Live (current) | Durable persistence | Optional |
|------|-----------------|----------------------|----------|
| **Market data** (ticks, quotes) | In-memory snapshot + NATS `snapshot.{backend_id}` | NATS KV `LIVE_STATE` (key `MarketDataEvent.{symbol}`) | QuestDB (time-series, when `QUESTDB_ILP_ADDR` set) |
| **Positions** (current view) | In-memory snapshot + NATS `snapshot.{backend_id}` | NATS KV `LIVE_STATE` (key `Positions.{backend_id}`) when `NATS_USE_JETSTREAM=1` | — |
| **Positions / orders** (audit trail) | — | SQLite ledger (`ledger.db`) | — |
| **Strategy signals / decisions** | In-memory + snapshot | NATS KV `LIVE_STATE` (`StrategySignal.*`, `StrategyDecision.*`) | QuestDB (when configured) |

---

## 1. Market data

### Ingest path

```
IBKR TWS (7497) or Polygon/mock
  → Rust ib_adapter / market_data
  → backend_service
       → NATS subjects: market-data.tick.<symbol>
       → collection_aggregation subscribes to market-data.tick.>
            → Decodes NatsEnvelope
            → LIVE_STATE KV: put(MessageType.MarketDataEvent.{symbol}, envelope)
            → Optional: QuestDB ILP sink (tick time-series)
       → In-memory state (SystemSnapshot) updated
       → snapshot_publisher publishes full snapshot to snapshot.{backend_id}
```

### Where it is stored

- **NATS KV bucket `LIVE_STATE`**: Key `MarketDataEvent.{symbol}`; value = protobuf `NatsEnvelope`. Last-write-wins; TTL/history from bucket config (see [NATS_KV_USAGE_AND_RECOMMENDATIONS.md](NATS_KV_USAGE_AND_RECOMMENDATIONS.md)).
- **QuestDB** (optional): InfluxDB Line Protocol; only when `QUESTDB_ILP_ADDR` is set. Used for historical ticks/analytics.
- **In-memory**: `SystemSnapshot` in backend_service; merged into the periodic snapshot published to `snapshot.{backend_id}`. TUI and other clients read current market data from that snapshot only (no REST).

### Not persisted

- No PostgreSQL/MySQL; the design in [BACKEND_DATA_STORAGE_ARCHITECTURE.md](../research/architecture/BACKEND_DATA_STORAGE_ARCHITECTURE.md) (e.g. positions table) is proposed, not implemented.

---

## 2. Position data

### Current positions (live view)

- **Source of truth for UI**: `SystemSnapshot.positions` built in backend_service and published on NATS as part of `snapshot.{backend_id}` (full `NatsEnvelope(SystemSnapshot)`).
- **Position persistence**: When JetStream is enabled (`NATS_USE_JETSTREAM=1`), the snapshot publisher writes a `PositionsSnapshot` (positions + generated_at) to NATS KV under key **`Positions.{backend_id}`** (bucket `NATS_KV_BUCKET`, default `LIVE_STATE`). This gives durable position state for late-joining clients and restarts; readers can load the key for last-known positions. After backend restart, positions are still repopulated from TWS/ib_adapter (and ledger where applicable); the KV copy is a cache, not the sole source.

### Durable position/order record

- **SQLite ledger** (`agents/backend/data/ledger.db`): Single writer = Rust `ledger` crate. Records orders, position-affecting events, and related transactional state. Read by Rust API and position logic.
- See [LEDGER_OWNERSHIP_AUDIT.md](LEDGER_OWNERSHIP_AUDIT.md) and [DATAFLOW_ARCHITECTURE.md](DATAFLOW_ARCHITECTURE.md) (§2).

### Flow

```
Orders / position changes
  → ib_adapter (TWS) callbacks
  → backend state + NATS strategy.decision.*
  → Rust ledger crate → SQLite ledger.db
```

---

## 3. Storage inventory (reference)

| Store | Written by | Data | Retention |
|-------|------------|------|-----------|
| **NATS KV LIVE_STATE** | backend_service (collection_aggregation) | MarketDataEvent.*, StrategySignal.*, StrategyDecision.* | Bucket TTL (e.g. 24h); configurable |
| **NATS KV LIVE_STATE** | backend_service (snapshot_publisher) | Positions.{backend_id} (PositionsSnapshot proto) | When NATS_USE_JETSTREAM=1; same bucket TTL |
| **QuestDB** | backend_service (collection_aggregation ILP sink) | Tick time-series | Configurable |
| **SQLite ledger** | Rust ledger crate | Orders, positions (audit), decisions | Permanent |
| **In-memory + snapshot** | backend_service | Full SystemSnapshot (positions, orders, market, alerts, etc.) | Ephemeral; clients subscribe to snapshot.{backend_id} |

---

## 4. References

- [DATAFLOW_ARCHITECTURE.md](DATAFLOW_ARCHITECTURE.md) — data flow and storage table  
- [NATS_KV_USAGE_AND_RECOMMENDATIONS.md](NATS_KV_USAGE_AND_RECOMMENDATIONS.md) — LIVE_STATE usage and bucket config  
- [LIVE_STATE_KV_VERIFICATION.md](LIVE_STATE_KV_VERIFICATION.md) — how to verify KV with NATS CLI  
- [PERSISTENCE_OPTIONS.md](../research/PERSISTENCE_OPTIONS.md) — SQLite, NATS KV, QuestDB summary  
- [CURRENT_TOPOLOGY.md](CURRENT_TOPOLOGY.md) — topology and storage writers  
- [NATS_API.md](NATS_API.md) — subjects and snapshot/source list  
