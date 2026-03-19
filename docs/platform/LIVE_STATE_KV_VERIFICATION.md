# LIVE_STATE KV verification in practice

How to verify the NATS JetStream Key-Value bucket `LIVE_STATE` with a running NATS server and the Rust backend collector.

---

## Overview

- **Bucket**: `LIVE_STATE` (or value of `NATS_KV_BUCKET`, default `LIVE_STATE`).
- **Writer**: `backend_service` collection aggregator (`agents/backend/services/backend_service/src/collection_aggregation.rs`). It subscribes to NATS subjects (`market-data.tick.>`, `strategy.signal.>`, `strategy.decision.>`), decodes `NatsEnvelope`, and writes keys `{message_type}.{symbol}`.
- **Readers**: No REST endpoints; use NATS CLI or any JetStream KV client (see below).

See also: [NATS_KV_USAGE_AND_RECOMMENDATIONS.md](NATS_KV_USAGE_AND_RECOMMENDATIONS.md), [DATAFLOW_ARCHITECTURE.md](DATAFLOW_ARCHITECTURE.md), [NATS_API.md](NATS_API.md) (full subject list and api.* request/reply).

---

## Prerequisites

- **NATS server** 2.6.2+ with JetStream enabled (e.g. `nats-server -js` or config with `jetstream: {}`).
- **NATS CLI** (`nats`), e.g. `brew install nats-server nats` or [install](https://docs.nats.io/running-a-nats-service/introduction/installation).
- **backend_service** built and runnable: `cargo run -p backend_service` from `agents/backend`.

---

## 1. Start NATS with JetStream

```bash
# Default port 4222
nats-server -js
```

Or with a config file that enables JetStream. The bucket will be created by the backend on first use if it does not exist.

---

## 2. Start the backend collector

From repo root or `agents/backend`:

```bash
export NATS_URL=nats://localhost:4222
export NATS_KV_BUCKET=LIVE_STATE   # optional; default is LIVE_STATE
cargo run -p backend_service
```

The collector will:

- Connect to NATS.
- Create the `LIVE_STATE` KV bucket if missing (history 5, 24h TTL, 64 KiB max value, 10 MiB bucket).
- Subscribe to `market-data.tick.>`, `strategy.signal.>`, `strategy.decision.>`.
- For each message: decode `NatsEnvelope`, write key `{message_type}.{symbol}` with the raw envelope bytes.

Keys only appear after **at least one message** is published on those subjects. If you have no publishers, use the “Publish test message” step below.

---

## 3. Verify with NATS CLI

### List keys in the bucket

```bash
nats kv ls LIVE_STATE
```

Example output (after some traffic):

```
LIVE_STATE Key-Value Store

  MarketDataEvent.SPY
  MarketDataEvent.AAPL
  StrategyDecision.QQQ
```

### Get a single key (value is binary NatsEnvelope)

```bash
nats kv get LIVE_STATE "MarketDataEvent.SPY"
```

Output is the raw protobuf `NatsEnvelope` bytes (not human-readable). To inspect with a small script, decode the `NatsEnvelope` protobuf (see `proto/` or `nats_adapter` crate).

### Watch all keys (stream updates)

```bash
nats kv watch LIVE_STATE
```

Shows key names as they are created or updated. Useful to confirm writes when you publish test messages.

### Optional: bucket status

```bash
nats kv status LIVE_STATE
```

Shows bucket config (history, TTL, size, etc.).

---

## 4. Publish a test message (optional)

If no other service is publishing to `market-data.tick.>` (or strategy subjects), you can publish a minimal envelope to trigger a KV write. The payload must be a serialized `NatsEnvelope` (see `proto/messages.proto` and `nats_adapter`). Example using a one-off Rust/Go/script that encodes `NatsEnvelope` and publishes to e.g. `market-data.tick.SPY` will cause key `MarketDataEvent.SPY` (if `message_type` is `MarketDataEvent`) to appear.

After publishing, run:

```bash
nats kv ls LIVE_STATE
nats kv get LIVE_STATE "MarketDataEvent.SPY"
```

---

## 5. Health and troubleshooting

- **Backend logs**: Look for `LIVE_STATE bucket reachable` after startup when KV is configured. If you see `LIVE_STATE bucket status check failed`, the bucket may still work (writes often succeed); check NATS server JetStream is enabled.
- **No keys**: Ensure (1) NATS_URL is correct, (2) backend_service is running and connected, (3) at least one message has been published on the subscribed subjects so the collector can write a key.
- **NATS not running**: Backend will log connection failures and retry every 2 seconds; `nats_ok` in the shared snapshot/metrics will be false.

---

## 6. Key and value reference

| Item | Value |
|------|--------|
| Key format | `{message_type}.{symbol}` (e.g. `MarketDataEvent.SPY`, `StrategySignal.AAPL`) |
| Value | Binary protobuf `NatsEnvelope` (id, message_type, timestamp, source, payload) |
| Subject → key | Last token of subject = symbol; envelope.message_type = message_type |

See `collection_aggregation.rs` `kv_key()` and `handle_message()` for the exact mapping.
