# Current Topology

**Last updated**: 2026-03-11

Short-form reference for the active runtime topology.

For deeper detail, see:

- [ARCHITECTURE.md](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/ARCHITECTURE.md)
- [DATAFLOW_ARCHITECTURE.md](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/platform/DATAFLOW_ARCHITECTURE.md)
- [API_GATEWAY_AND_ROUTING_REVIEW.md](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/platform/API_GATEWAY_AND_ROUTING_REVIEW.md)

## Summary

- `C++` produces market and strategy events.
- `Rust` owns shared frontend read APIs and active collection fanout.
- `Go` is narrowed to operational tooling that remains separate by role.
- `Python` is limited to selected legacy/helper surfaces; the active TUI runtime is Rust.

## Runtime Shape

```text
IBKR/TWS
  -> C++ engine / tws_client
    -> NATS (NatsEnvelope protobuf)
      -> Rust backend collector
           -> QuestDB
           -> NATS KV LIVE_STATE
      -> Rust nats_adapter
           -> Rust backend in-memory state

Web
  -> Rust API (:8080)
     - /api/v1/snapshot
     - /api/v1/frontend/*
     - /ws/snapshot

TUI
  -> Rust read-model endpoints
  -> optional NATS/event-driven path
```

## Ownership

### Producers

- `native/` C++ publishes market and strategy events to NATS.

### Collection and operational services

- `agents/backend/services/backend_service`
  - decodes `NatsEnvelope`
  - expects concrete symbol-scoped producer subjects for market and strategy events
  - writes `LIVE_STATE` KV
  - writes QuestDB when configured

### Shared frontend API

- `agents/backend/`
  - owns snapshot API
  - owns shared frontend read models
  - owns `LIVE_STATE` read/watch endpoints
  - owns `/api/heartbeat/*`, `/api/health-aggregated`, and `/gateway/health`
  - consumes `system.health` directly for aggregated health
  - is the primary browser-facing backend

### Python scope

- `native/tests/python/`
  - pybind11 binding tests
- `native/generated/python/`
  - generated helper output from `proto/messages.proto`
Python is no longer the terminal UI runtime, the general frontend read-model backend, or a collection/live-state ownership layer.

## Storage

| Store | Writer | Purpose |
|-------|--------|---------|
| `NATS KV LIVE_STATE` | Rust backend collector | live key-value state, full `NatsEnvelope` values |
| `QuestDB` | Rust backend collector | time-series archive |
| `SQLite ledger` | Rust ledger target owner | durable ledger state |
| C++ in-memory cache | C++ only | hot tick data |

## Known Simplification Gaps

- Some docs and plans still describe older Python overlap around finance state.
- Some docs still describe TUI Python service calls even though the active runtime path is Rust-first.
- Go `api-gateway` and Go `heartbeat-aggregator` are retired. Rust now owns the client-facing health and heartbeat routes and consumes `system.health` directly.

## Default Deployment Direction

- browser -> shared origin
- shared origin -> Rust API
- direct per-port service wiring remains a dev/debug override, not the preferred default
