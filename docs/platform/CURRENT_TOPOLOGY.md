# Current Topology

**Last updated**: 2026-03-11

Short-form reference for the active runtime topology.

For deeper detail, see:

- [ARCHITECTURE.md](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/ARCHITECTURE.md)
- [DATAFLOW_ARCHITECTURE.md](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/platform/DATAFLOW_ARCHITECTURE.md)
- [API_GATEWAY_AND_ROUTING_REVIEW.md](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/platform/API_GATEWAY_AND_ROUTING_REVIEW.md)

## Summary

- `C++` produces market and strategy events.
- `Go` collects, fans out, and exposes operational aggregation.
- `Rust` owns shared frontend read APIs.
- `Python` owns the Textual TUI and specialist integration services.

## Runtime Shape

```text
IBKR/TWS
  -> C++ engine / tws_client
    -> NATS (NatsEnvelope protobuf)
      -> Go collection-daemon
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

- `agents/go/cmd/collection-daemon`
  - decodes `NatsEnvelope`
  - writes `LIVE_STATE` KV
  - writes QuestDB when configured
- `agents/go/cmd/api-gateway`
  - aggregated health
  - `LIVE_STATE` reads
  - operational routing convenience
- `agents/go/cmd/heartbeat-aggregator`
  - service liveness aggregation

### Shared frontend API

- `agents/backend/`
  - owns snapshot API
  - owns shared frontend read models
  - is the primary browser-facing backend

### Python scope

- `python/tui/`
  - active terminal UI
- `python/integration/`
  - broker and bank integrations
  - risk-free-rate service
- `python/services/health_dashboard.py`
  - health dashboard service

Python is no longer the general frontend read-model backend.

## Storage

| Store | Writer | Purpose |
|-------|--------|---------|
| `NATS KV LIVE_STATE` | Go `collection-daemon` | live key-value state, full `NatsEnvelope` values |
| `QuestDB` | Go `collection-daemon` | time-series archive |
| `SQLite ledger` | Rust ledger target owner | durable ledger state |
| C++ in-memory cache | C++ only | hot tick data |

## Known Simplification Gaps

- Rust and Python still overlap around some durable/local finance state, especially loans.
- TUI still mixes Rust read models with selected Python service calls.
- `api-gateway` still exists for operational aggregation; not all routing is single-origin by default in every dev path.

## Default Deployment Direction

- browser -> shared origin
- shared origin -> Rust API, Go gateway, selected Python specialist services by path
- direct per-port service wiring remains a dev/debug override, not the preferred default
