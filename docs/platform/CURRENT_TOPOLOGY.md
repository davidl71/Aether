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
- `Python` owns selected specialist integration services; the active TUI runtime is Rust.

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
  -> selected Python integration services
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

- `python/tui/`
  - active terminal UI
- `python/integration/`
  - selected broker and bank integrations
  - benchmark/rate routes and active logic are now Rust-owned
  - IB public routes and active logic are now Rust-owned
Python is no longer the general frontend read-model backend or a collection/live-state ownership layer.

## Storage

| Store | Writer | Purpose |
|-------|--------|---------|
| `NATS KV LIVE_STATE` | Rust backend collector | live key-value state, full `NatsEnvelope` values |
| `QuestDB` | Rust backend collector | time-series archive |
| `SQLite ledger` | Rust ledger target owner | durable ledger state |
| C++ in-memory cache | C++ only | hot tick data |

## Known Simplification Gaps

- Rust and Python still overlap around some durable/local finance state, especially loans.
- TUI still mixes Rust read models with selected Python service calls.
- Go `api-gateway` and Go `heartbeat-aggregator` are retired. Rust now owns the client-facing health and heartbeat routes and consumes `system.health` directly.

## Default Deployment Direction

- browser -> shared origin
- shared origin -> Rust API and selected Python specialist services by path
- direct per-port service wiring remains a dev/debug override, not the preferred default
