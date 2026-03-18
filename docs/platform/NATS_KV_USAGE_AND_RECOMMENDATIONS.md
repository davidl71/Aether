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

- **REST (removed):** The backend is NATS-only; there is no HTTP server. Former REST handlers for `GET /api/live/state` and `GET /api/live/state/watch` were removed (see `docs/platform/REMOVE_REST_OPTIONS.md`). To read LIVE_STATE, use the NATS CLI or a JetStream client; see `docs/platform/LIVE_STATE_KV_VERIFICATION.md`.
- **Real box yield curve:** When a client calls `api.finance_rates.build_curve` with **empty** `opportunities` and a `symbol`, the backend tries to load opportunities from the same bucket (default `LIVE_STATE`) under key **`yield_curve.{symbol}`** (e.g. `yield_curve.SPX`). Value must be a **JSON array** of objects; each object must have a `spread` key that deserializes to `BoxSpreadInput` (symbol, expiry, days_to_expiry, strike_width, buy_implied_rate, sell_implied_rate, net_debit, net_credit, liquidity_score). If the key is missing or invalid, the curve is built from the (empty) request and the TUI Yield tab shows no points. See [BOX_SPREAD_YIELD_CURVE_TWS.md](BOX_SPREAD_YIELD_CURVE_TWS.md) for TWS integration and fallback.
- **Writer (pre-populate + interval):** `backend_service` runs a **yield curve writer** (`yield_curve_writer.rs`) that writes to `yield_curve.{symbol}` **once immediately** on startup (so the Yield tab has data without waiting), then on an interval (`YIELD_CURVE_WRITER_INTERVAL_SECS`, default 60). Data source: if `YIELD_CURVE_SOURCE_URL` is set, the backend fetches that URL (JSON array of curve points; see BOXTRADES_REFERENCE.md); otherwise it uses **synthetic** points. To use live data instead, set `YIELD_CURVE_SOURCE_URL` to your feed or add a writer that puts the same key from TWS/strategy. TWS integration: [BOX_SPREAD_YIELD_CURVE_TWS.md](BOX_SPREAD_YIELD_CURVE_TWS.md).

**Optional watch endpoint contract (deferred)** — If a REST or gateway layer re-exposes `GET /api/live/state/watch` for UIs that prefer SSE over direct NATS:

- **No query params**: `store.watch_all()` — stream all bucket updates (same as current recommendation for “show everything”).
- **`?key=`** (e.g. `key=MarketDataEvent.SPY`): Stream updates for a single key. Use `store.watch(key)`.
- **`?prefix=`** (e.g. `prefix=MarketDataEvent.`): Stream updates for keys matching a subject prefix (all keys for one message type). Use `store.watch(prefix + ">")` (NATS subject wildcard `>` = one or more tokens).

Implementation is deferred; when adding the handler, branch on presence of `key` or `prefix` and call the appropriate `store` method.

### 2. Configuration

| Env / source | Purpose |
|--------------|--------|
| `NATS_URL` | NATS server (default `nats://localhost:4222`) |
| `NATS_KV_BUCKET` | Bucket name for collection aggregator and yield curve keys (default `LIVE_STATE`) |
| `YIELD_CURVE_WRITER_INTERVAL_SECS` | Interval in seconds for yield curve KV writes (default `60`). Backend only. |
| `YIELD_CURVE_SOURCE_URL` | Optional. URL returning a JSON array of curve points to pre-populate the yield curve (see BOXTRADES_REFERENCE.md § Pre-populating). If unset, backend uses synthetic data. |

### 3. Key semantics

- **Key format (collection_aggregation):** `{message_type}.{symbol}` (e.g. `MarketDataEvent.SPY`, `StrategySignal.AAPL`).
- **Key format (yield curve):** `yield_curve.{symbol}` (e.g. `yield_curve.SPX`). Not written by `collection_aggregation`; written by a separate process (strategy, TWS job, or script) that publishes box spread opportunities for the Yield tab.
- **Value (event keys):** Protobuf binary of `NatsEnvelope` (id, message_type, payload).
- **Value (yield_curve.{symbol}):** JSON array of objects with a `spread` key (see `api::finance_rates::BoxSpreadInput`). Used by `api.finance_rates.build_curve` when the request has empty opportunities.
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

- **Current**: `backend_service` does not run an HTTP server; it only writes to LIVE_STATE from `collection_aggregation`. No shared store is passed to any REST layer (REST was removed).
- **Recommendation**: Keep this pattern. Ensure any other server that mounts the API (e.g. a standalone API binary) can also pass an optional shared store so production avoids per-request connections.

### 4. Watch vs poll

- **Current**: No REST watch endpoint; to stream KV updates use a JetStream client and `store.watch_all()` or `store.watch(key)`.
- **Recommendation**: Keep `watch_all()` for “show everything” UIs. For “per-symbol” or “per-type” views, use `watch()` with a key/prefix filter to reduce traffic and decoding cost. If a REST `GET /api/live/state/watch` is re-introduced, support optional query params: `key=` → `store.watch(key)`; `prefix=` → `store.watch(prefix + ">")`. See “Optional watch endpoint contract (deferred)” in §1 Readers.

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
- **Fallback read path**: None. TUI is NATS-only; REST snapshot fallback has been removed.
- **Rule**: NATS is the only live path for TUI snapshot. Do not treat REST as a source for TUI.

### LIVE_STATE KV: derived view, not a second source

- **Source of truth for aggregated UI state**: Still the in-memory snapshot (positions, orders, strategy status, etc.).
- **LIVE_STATE KV**: Holds the **last message per key** (`{message_type}.{symbol}`). It is filled by `collection_aggregation`, which subscribes to the same NATS subjects (`market-data.tick.>`, `strategy.signal.>`, `strategy.decision.>`) that the backend publishes. So KV is a "last value" view of that stream, not an independent store.
- **Same event stream** drives both (1) in-memory snapshot updates (in backend) and (2) KV puts (in collection_aggregation). No merge of two sources; KV is derived from the same NATS traffic.

### Snapshot writers (single store)

The in-memory snapshot has multiple writers: strategy fanout, market data provider, REST handlers (config/mode/account/strategy), swiftness. They all write into the same `SharedSnapshot`. There is no second snapshot store to keep in sync.

---

## References

- **NATS JetStream KV**: [Key Value Store](https://docs.nats.io/nats-concepts/jetstream/key-value-store) (concepts); [KV store operations](https://docs.nats.io/using-nats/developing-with-nats/js/kv) (history, watch, TTL, limits).
- **This repo**: `agents/backend/services/backend_service/src/collection_aggregation.rs` (KV writes), `docs/platform/LIVE_STATE_KV_VERIFICATION.md` (verification and read path), `docs/planning/NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md`.
