# Current Topology

**Last updated**: 2026-03-14

Short-form reference for the active runtime topology and component/broker/backend ownership.

For deeper detail, see:

- [ARCHITECTURE.md](../../ARCHITECTURE.md)
- [DATAFLOW_ARCHITECTURE.md](DATAFLOW_ARCHITECTURE.md)
- [BACKEND_SERVICES_DAEMONIZED.md](../BACKEND_SERVICES_DAEMONIZED.md)

## Summary

- **Rust** owns backend, TUI, CLI, broker adapter (IBKR), quant/risk, ledger, and NATS producer/consumer. C++ native build is **removed**.
- **Daemons**: `nats` (4222) and `rust_backend` (8080) only.

## Runtime shape

```text
IBKR/TWS (port 7497)
  -> Rust ib_adapter (agents/backend/crates/ib_adapter)
    -> Rust backend_service
         -> NATS (NatsEnvelope protobuf)
         -> LIVE_STATE KV, QuestDB (when configured)

Rust API (:8080)
  - /api/v1/snapshot, /api/v1/frontend/*, /ws/snapshot
  - health, heartbeat

TUI / CLI
  -> Rust read-model endpoints (REST)
  -> optional NATS/event-driven path
```

## Component and backend ownership

| Area | Owner | Location |
|------|--------|----------|
| **Shared frontend API** | Rust | `agents/backend/crates/api`, `services/backend_service` |
| **Broker adapters (IBKR)** | Rust | `agents/backend/crates/ib_adapter` |
| **Ledger** | Rust | `crates/ledger` |
| **Quant / risk / pricing** | Rust | `crates/quant`, `crates/risk` |
| **Market data, strategy** | Rust | `crates/market_data`, `crates/strategy` |
| **NATS ingestion, LIVE_STATE** | Rust | `crates/nats_adapter`, backend_service |
| **TUI, CLI** | Rust | `services/tui_service`, `bin/cli` |

### Collection and backend service

- `agents/backend/services/backend_service`
  - decodes `NatsEnvelope`, writes `LIVE_STATE` KV and QuestDB when configured
  - owns snapshot API, frontend read models, health/heartbeat
  - primary producer/consumer for NATS (C++ engine removed)

## Storage

| Store | Writer | Purpose |
|-------|--------|---------|
| `NATS KV LIVE_STATE` | Rust backend | live key-value state, full `NatsEnvelope` values |
| `QuestDB` | Rust backend | time-series archive (when configured) |
| `SQLite ledger` | Rust `crates/ledger` | durable ledger state |

## Broker and data provider support

| Broker / provider | Role | Status |
|-------------------|------|--------|
| **IBKR** | Trading, market data, positions | Active (Rust `ib_adapter`) |
| **Discount Bank** | Banking, reconciliation | Active (Rust routes + file import) |
| **Alpaca / Tastytrade** | Trading | Retired (not in active runtime) |
| **Fibi, Meitav, IBI** | Israeli bank/broker | Config / file-based |

See [DATAFLOW_ARCHITECTURE.md](DATAFLOW_ARCHITECTURE.md) § 2.2–2.4 for feature readiness, broker support, and data provider tables.

## Notes

- Go `api-gateway` and Go `heartbeat-aggregator` are retired; Rust owns health and heartbeat routes.
- Python is not an active runtime tier; Rust is the only active backend/TUI/CLI surface.

## Default Deployment Direction

- browser -> shared origin
- shared origin -> Rust API
- direct per-port service wiring remains a dev/debug override, not the preferred default
