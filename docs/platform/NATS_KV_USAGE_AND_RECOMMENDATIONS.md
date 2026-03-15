# NATS KV usage and recommendations

Summary of how this repo uses NATS JetStream Key-Value and recommended practices.

---

## Current usage

### 1. Bucket: `LIVE_STATE`

**Writers**

- **`backend_service`** (`agents/backend/services/backend_service/src/collection_aggregation.rs`):
  - Connects when `NATS_KV_BUCKET` is set (default: `LIVE_STATE`).
  - On each message from `market-data.tick.>`, `strategy.signal.>`, `strategy.decision.>`:
    - Decodes `NatsEnvelope` from payload.
    - Writes to KV with key `{message_type}.{symbol}` (e.g. `MarketDataEvent.SPY`).
  - Uses `get_key_value(bucket)` or `create_key_value(Config { bucket, .. })` if the bucket does not exist.
  - Key format: `kv_key(subject, message_type)` → `"{message_type}.{symbol}"`.

**Readers**

- **`api`** (`agents/backend/crates/api/src/rest.rs`):
  - **GET `/api/live/state`**: `live_state_store()` → `get_key_value("LIVE_STATE")`. Optional `?key=` returns single `store.entry(key)`; otherwise returns `store.keys()` as a key list.
  - **GET `/api/live/state/watch`**: Same store, `store.watch_all()` for SSE stream of key/value updates (key, revision, base64 value, envelope metadata).
  - Store obtained per request via `live_state_client()` → `jetstream.get_key_value("LIVE_STATE")` (no caching of store handle).

### 2. Configuration

| Env / source | Purpose |
|--------------|--------|
| `NATS_URL` | NATS server (default `nats://localhost:4222`) |
| `NATS_KV_BUCKET` | Bucket name for collection aggregator (default `LIVE_STATE`) |

### 3. Key semantics

- **Key format**: `{message_type}.{symbol}` (e.g. `MarketDataEvent.SPY`, `StrategySignal.AAPL`).
- **Value**: Protobuf binary of `NatsEnvelope` (id, message_type, payload).
- **Updates**: Each new message on the subject overwrites the key (last-write-wins). No explicit TTL or history configured in code (bucket uses server/default config if created by `create_key_value`).

---

## Recommendations

### 1. Bucket configuration (when creating the bucket)

When creating the bucket (e.g. in `collection_aggregation` or via server config), set explicit limits and behavior:

- **History**: Set `history` (e.g. 1–10) if you want revision history for debugging or audit. Default is 1 (no history). Max 64 per key; for more, use a Stream.
- **TTL**: Set `ttl` (e.g. 24h) so stale keys (e.g. symbols no longer updated) expire.
- **Max value size**: Set `max_value_size` to match your largest envelope (e.g. 64KB or 256KB).
- **Max bucket size / max bytes**: Use `max_bytes` / `max_bucket_size` so the bucket cannot grow unbounded.

Example (Rust `async_nats::jetstream::kv::Config`):

```rust
create_key_value(Config {
    bucket: bucket.clone(),
    history: 5,           // keep last 5 revisions per key
    ttl: Duration::from_secs(86400),  // 24h
    max_value_size: 65_536,
    ..Default::default()
}).await
```

Apply the same ideas in `config/nats-server.conf` or via `nats` CLI if you manage the bucket outside the app.

### 2. Single bucket vs multiple buckets

- **Current**: One bucket `LIVE_STATE` for all live state (market data, signals, decisions).
- **Recommendation**: Keep one bucket as long as key space is clear (`{message_type}.{symbol}`) and TTL/history are sufficient. If you add unrelated state (e.g. “current account”, “mode”) with different retention or access patterns, use a second bucket (e.g. `ib_box_spread_state` as in `docs/planning/NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md`) rather than overloading `LIVE_STATE`.

### 3. Store handle reuse

- **Current**: When `backend_service` starts with NATS, it passes a shared `live_state_kv` store into `RestState`; API handlers use it when present and otherwise connect per-request (`live_state_store()`).
- **Recommendation**: Keep this pattern. Ensure any other server that mounts the API (e.g. a standalone API binary) can also pass an optional shared store so production avoids per-request connections.

### 4. Watch vs poll

- **Current**: `/api/live/state/watch` uses `store.watch_all()` and streams all updates over SSE.
- **Recommendation**: Keep `watch_all()` for “show everything” UIs. For “per-symbol” or “per-type” views, use `watch()` with a key/prefix filter to reduce traffic and decoding cost.

### 5. Key naming and consistency

- **Current**: Keys are `{message_type}.{symbol}`; symbol is taken from the last token of the subject.
- **Recommendation**: Keep this convention and document it (e.g. in `docs/platform/NATS_TOPICS_REGISTRY.md` or this file). Use a single function (e.g. `kv_key(subject, message_type)`) everywhere so writers and readers stay in sync.

### 6. Error handling and health

- **Current**: Missing bucket or watch failure returns 503 or an error event on the SSE stream.
- **Recommendation**: Expose NATS/KV health in `/health` (you already surface `nats_ok`). Optionally add a dedicated “LIVE_STATE bucket reachable” check (e.g. `store.keys()` or a single `entry("_ping")`) so the UI can distinguish “NATS up, bucket missing” from “NATS down”.

### 7. History and retention

- NATS KV keeps up to 64 revisions per key (configurable at bucket create). For “current state” only, `history: 1` is enough and saves space.
- For audit/debug, set `history: 5` or similar and optionally add TTL so old keys and revisions expire. For long-term event history, use a JetStream Stream (consumers, replay) instead of KV.

### 8. Separate “state” bucket (future)

Planning doc `docs/planning/NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md` describes a separate bucket (e.g. `ib_box_spread_state`) for “current account” and “current mode” with keys like `current_account:ib`. That stays independent of `LIVE_STATE` (event-style, per-symbol). Recommendations there (single bus, NATS KV first, Redis only if needed) still apply.

---

## Read/write paths and single source of truth

There are no dual stores for the same logical data; the following is the intended model.

### TUI snapshot: one source, two delivery paths

- **Source of truth**: In-memory `SystemSnapshot` in `backend_service` (shared `state.snapshot`).
- **Primary read path**: TUI subscribes to NATS `snapshot.{backend_id}`. The snapshot publisher periodically serializes the same in-memory snapshot and publishes it.
- **Fallback read path**: When NATS is unavailable or the NATS snapshot is stale, TUI may poll `GET /api/v1/snapshot`, which returns the same in-memory snapshot over REST.
- **Rule**: NATS is primary; REST is fallback only. Config (`provider_type`, `rest_fallback`) enforces this. Do not treat REST as a second source of truth.

### LIVE_STATE KV: derived view, not a second source

- **Source of truth for aggregated UI state**: Still the in-memory snapshot (positions, orders, strategy status, etc.).
- **LIVE_STATE KV**: Holds the **last message per key** (`{message_type}.{symbol}`). It is filled by `collection_aggregation`, which subscribes to the same NATS subjects (`market-data.tick.>`, `strategy.signal.>`, `strategy.decision.>`) that the backend publishes. So KV is a "last value" view of that stream, not an independent store.
- **Same event stream** drives both (1) in-memory snapshot updates (in backend) and (2) KV puts (in collection_aggregation). No merge of two sources; KV is derived from the same NATS traffic.

### Snapshot writers (single store)

The in-memory snapshot has multiple writers: strategy fanout, market data provider, REST handlers (config/mode/account/strategy), swiftness. They all write into the same `SharedSnapshot`. There is no second snapshot store to keep in sync.

---

## References

- **NATS JetStream KV**: [Key Value Store](https://docs.nats.io/nats-concepts/jetstream/key-value-store) (concepts); [KV store operations](https://docs.nats.io/using-nats/developing-with-nats/js/kv) (history, watch, TTL, limits).
- **This repo**: `agents/backend/crates/api/src/rest.rs` (live state API), `agents/backend/services/backend_service/src/collection_aggregation.rs` (KV writes), `docs/planning/NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md`, `docs/NATS_TESTING_GUIDE.md`.
