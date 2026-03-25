# Aether Architecture Overview

Multi-asset synthetic financing platform. Box spreads are one strategy (7-10% allocation);
the platform manages financing across options, futures, bonds, bank loans, and pension funds
across 21+ accounts and multiple brokers.

**Last updated**: 2026-03-25 (crate layer boundaries added)

## Current build and runtime settings

- **Active runtime**: Rust backend (`backend_service` :8080) and Rust TUI (`tui_service`). C++ native build is **removed** (see root `CMakeLists.txt`).
- **Broker connectivity**: Rust-owned; IBKR routes and adapter in `agents/backend/crates/ib_adapter` and API in `crates/api`.
- **Credential/config modeling**: provider credential helpers in `crates/api` now distinguish Alpaca paper vs live identities and separate trading vs data endpoints, even though Alpaca is not part of the active runtime path.
- **Daemons / long-lived Rust services**: `backend_service`, `tui_service`, and `tws_yield_curve_daemon`, plus `nats` (4222). Each active Rust service now publishes a `system.health` heartbeat with service identity and PID metadata.

## System Overview

```
External data sources (outbound connections from backend_service):
  Yahoo Finance / FMP / Polygon  ← polled HTTP; priority 50/60/70
  TWS/IBKR :7497                 ← push subscription; priority 100

              ↓ MarketDataAggregator (best-quote, priority-based)
              ↓ handle_market_event()
              ↓   ├─ apply_market_event() → SystemSnapshot (in-memory, RwLock)
              ↓   └─ NATS publish: market-data tick, candle, alert

┌──────────────────────────────────────────────────────────────────────┐
│  backend_service (Axum :8080)                                        │
│  Owns: SystemSnapshot, market data loops, strategy fanout,           │
│        snapshot_publisher, health_publisher, api_handlers (NATS RPC) │
└────────────────┬────────────────────────────────────┬───────────────┘
                 │ REST+WS                             │ NATS publish
                 │ /api/v1/snapshot                    │ snapshot.<backend_id>
                 │ /ws/snapshot                        │ market-data.tick.<sym>
                 │                                     │ system.health
                 ▼                                     ▼
┌────────────────────────────────────────────────────────────────────┐
│  NATS JetStream :4222                                               │
└─────────────────┬──────────────────────────────────────────────────┘
                  │ subscribe
                  ▼
┌──────────────────────────────────────┐
│  tui_service (NATS-only, read-only)  │
│  tws_yield_curve_daemon              │
└──────────────────────────────────────┘

Storage:
  NATS KV   → live key-value state
  SQLite    → Rust ledger owner (crates/ledger)
  QuestDB   → time-series archive (when configured)
```

## Component and backend ownership

| Area | Owner | Location | Notes |
|------|--------|----------|--------|
| **Shared frontend API** | Rust backend | `agents/backend/crates/api`, `services/backend_service` | Snapshot, read models, health, heartbeat |
| **Broker adapters** | Rust | `agents/backend/crates/ib_adapter` | IBKR; no separate broker daemons |
| **Ledger** | Rust | `agents/backend/crates/ledger` | Durable ledger state |
| **Quant / risk / pricing** | Rust | `crates/quant`, `crates/risk` | Greeks, margin, amortization, convexity, yield curve |
| **Market data & strategy** | Rust | `crates/market_data`, `crates/strategy` | Read path and strategy logic |
| **NATS ingestion** | Rust | `crates/nats_adapter`, backend_service | NatsEnvelope decode, LIVE_STATE, QuestDB fanout |
| **TUI** | Rust | `services/tui_service` | Ratatui UI, REST + optional NATS |
| **CLI** | Rust | `agents/backend/bin/cli` | Rust CLI binary |
| **Discount Bank parsing** | Rust | `crates/discount_bank_parser` | Bank statement / reconciliation |
| **NATS broker** | Infrastructure | Port 4222 | JetStream; see BACKEND_SERVICES_DAEMONIZED.md |
| **C++ native** | — | Removed | No longer in build; see root CMakeLists.txt |

## Components

### Client Applications

| Component | Technology | Data Source | Update Mechanism |
|-----------|------------|-------------|------------------|
| Web app | React/TypeScript (archived) | Rust `:8080` WebSocket → REST fallback | Historical only |
| Terminal UI | Rust/Ratatui | Rust read-model endpoints and NATS | Event-driven; Rust-owned UI state |
| CLI | Rust (`bin/cli`) | Rust backend / REST | Synchronous |

### Backend Services

| Component | Technology | Purpose |
|-----------|------------|---------|
| Rust REST+WS backend | Rust (Axum) :8080 | Shared frontend API, snapshot, read models, IBKR routes; collector for NATS → LIVE_STATE/QuestDB |
| NATS | NATS JetStream | Async messaging, market data events, heartbeats |
| C++ engine | Removed | No longer built; superseded by Rust backend and quant/risk/ib_adapter |

### Messaging Contract

NATS messages use `NatsEnvelope` (protobuf) with serialized inner messages. Rust backend is the active producer/consumer.
Topics: `market-data.tick.<symbol>`, `strategy.signal.<symbol>`, `strategy.decision.<symbol>`

See `docs/message_schemas/README.md` and `proto/messages.proto` for the canonical schema.

### Serialization

Generated from `proto/messages.proto` via `./proto/generate.sh`:

| Language | Output | Status |
|----------|--------|--------|
| Rust | `nats_adapter` crate (prost via build.rs) | Active |
| TypeScript | `web/src/proto/messages.ts` (ts-proto) | Archived web only |
| C++ | `native/generated/` | Inactive — native build removed |
| Python | — | No active proto consumer in repo |

## Crate Layer Boundaries

All active code lives under `agents/backend/`. Crates are organized in strict layers; each layer
may only depend on the layers below it.

```
┌─────────────────────────────────────────────────────────────────────┐
│  Services / Binaries (entry points)                                  │
│  backend_service  tui_service  cli  tws_yield_curve_daemon           │
└────────────────┬────────────────────────────────────────────────────┘
                 │ depends on
┌────────────────▼────────────────────────────────────────────────────┐
│  Domain API layer                                                    │
│  api  ← aggregates DTOs, fixtures, credentials, health              │
│       ← depends on: common, discount_bank_parser, market_data,      │
│                     nats_adapter, quant, risk, strategy              │
└────────────────┬────────────────────────────────────────────────────┘
                 │ depends on
┌────────────────▼────────────────────────────────────────────────────┐
│  Feature crates                                                      │
│  market_data  ← Yahoo, FMP, Polygon, TASE providers                 │
│  strategy     ← signal/decision models                              │
│  quant        ← Greeks, IV, yield curve, spread pricing             │
│  risk         ← position limits, risk checks (depends on quant)     │
│  broker_engine← trait abstraction for broker adapters               │
│  ib_adapter   ← IBKR/TWS adapter (depends on broker_engine)        │
│  tws_yield_curve← yield curve via TWS (depends on broker_engine)   │
│  discount_bank_parser← bank statement parsing (depends on ledger)   │
│  ledger       ← durable SQLite state (no domain deps)               │
│  nats_adapter ← NATS client + protobuf envelope encoding            │
└────────────────┬────────────────────────────────────────────────────┘
                 │ depends on
┌────────────────▼────────────────────────────────────────────────────┐
│  Foundation                                                          │
│  common  ← shared types, utilities, no external domain deps         │
└─────────────────────────────────────────────────────────────────────┘
```

### Dependency rules

| Crate | May depend on | Must NOT depend on |
|-------|--------------|-------------------|
| `common` | std, third-party only | any other internal crate |
| `ledger`, `nats_adapter`, `quant`, `strategy`, `market_data` | `common` | each other, `api`, services |
| `risk` | `common`, `quant` | `api`, broker, services |
| `broker_engine` | `common` | `api`, adapters, services |
| `ib_adapter`, `tws_yield_curve` | `common`, `broker_engine` | `api`, services |
| `discount_bank_parser` | `common`, `ledger` | `api`, services |
| `api` | all feature crates | services |
| services (`backend_service`, `tui_service`, `cli`) | any crate | — |

### Market data providers run inside backend_service

`market_data` providers (Yahoo, FMP, Polygon, TWS) are **spawned as tokio tasks inside
`backend_service`**. They do not write to NATS or the snapshot directly — all writes go through
`MarketDataAggregator` (priority-based best-quote selection) → `handle_market_event()`, which
updates the in-memory `SystemSnapshot` and publishes ticks + candles to NATS. The snapshot is
then periodically re-published to `snapshot.<backend_id>` by `snapshot_publisher`.

The TUI reads from `snapshot.<backend_id>` (full state) and NATS tick subjects. The
`live_market_data_source` field in the TUI status bar reflects which provider's tick arrived last
with sufficient priority.

### TUI is NATS-only (read-only)

`tui_service` depends only on `api`, `market_data`, and `nats_adapter`. It has **no direct
dependency on `ib_adapter`, `broker_engine`, or REST client code**. All state arrives via NATS
subjects (`snapshot.<backend_id>`, `system.health`, `api.*`). The `rest_fallback` config field
exists in `tui_service/src/config.rs` but is unused in the runtime.

Provider-level `mock` market data remains a normal `market_data` source with priority `0`, and
the TUI treats it like any other source label. Service-local demo/bootstrap seeding now lives in
`backend_service`, separate from provider selection. `api::mock_data` is limited to legacy/domain
fixture helpers and is no longer the owner of backend startup seeding.

### Health ownership

`system.health` is a transport-level heartbeat bus, not a backend_service-only concern.

- `nats_adapter` owns the shared heartbeat publisher helper used by all active Rust services
- each long-lived service publishes its own `BackendHealth` message with a stable service id
- `api` owns the health DTO / aggregation model exposed by REST
- `backend_service` owns the aggregated REST health endpoints by subscribing to `system.health`
- `tui_service` is a consumer of the aggregated map and should render the full component set, not a hardcoded subset

Current service ids published on `system.health`:

- `backend_service`
- `tui_service`
- `tws_yield_curve_daemon`

### Legacy execution paths

`broker_execution_legacy` and `ib_execution_legacy` exist as isolated crates. They are **not
imported by any service or the `api` crate**. They are preserved for reference only and will be
removed once audit is complete.

---

## Known Issues / Technical Debt

See `docs/platform/DATAFLOW_ARCHITECTURE.md` for full analysis. Key issues:

1. **Legacy Python references in docs/config**: parts of the repo still describe retired Python runtime surfaces and need ongoing cleanup.
2. **Split read paths remain**: Rust owns the active TUI/backend path, but some docs and planning artifacts still describe older Python integration flows.
3. **WebSocket sends full snapshot**: Rust WS sends full snapshot once on connect, then only changed sections (delta) every 2s — see IMPROVEMENT_PLAN P2-A (done). Remaining gap: scale if many clients.
4. **Collector durability gap**: `collection-daemon` now decodes `NatsEnvelope` and owns `LIVE_STATE`, but durable JetStream replay remains opt-in instead of the default collection mode.
5. **Hardcoded ETF duration / IV**: Quant logic in Rust may still use static lookups or external IV; future work could add full bond/IV solvers.

## Technology Stack

| Layer | Technologies |
|-------|--------------|
| Core engine / quant / risk | Rust (quant, risk, ib_adapter crates) |
| Backend services | Rust (Axum, prost, sqlx, ibapi) |
| Frontends | Rust Ratatui TUI, Rust CLI (bin/cli), archived React 18/TypeScript |
| Messaging | NATS JetStream, Protocol Buffers |
| Storage | SQLite, QuestDB, NATS KV |
| Build | Cargo, uv, npm (C++/CMake removed) |
| Testing | cargo test, pytest (scripts), Vitest (archived web) |

## Language Rationale

- **Rust**: backend, ledger, broker adapter (ib_adapter), quant/risk, TUI, CLI
- **TypeScript/React**: archived web app only
- **Python**: scripts and utilities only; no active runtime tier

## Directory Structure

```
ib_box_spread_full_universal/
├── agents/backend/      # Rust workspace (primary codebase)
│   ├── crates/          # api, ib_adapter, ledger, market_data, nats_adapter, quant, risk, strategy, discount_bank_parser
│   ├── services/        # backend_service, tui_service
│   └── bin/             # cli
├── native/              # Removed from build (see CMakeLists.txt)
├── web/                 # Archived React web client
├── proto/               # Canonical protobuf schema (messages.proto)
├── scripts/             # Build, lint, deploy helpers
└── docs/                # Documentation
    └── platform/        # Architecture, dataflow, improvement plan
```

## Detailed Documentation

- **Full dataflow analysis**: `docs/platform/DATAFLOW_ARCHITECTURE.md`
- **Improvement plan**: `docs/platform/IMPROVEMENT_PLAN.md`
- **Message schemas**: `docs/message_schemas/README.md`
- **Synthetic financing architecture**: `docs/platform/SYNTHETIC_FINANCING_ARCHITECTURE.md`
- **Backend design research**: `docs/research/architecture/`
- **Build system**: `native/CMakeLists.txt`, `CMakePresets.json`

For complete project guidelines, see [AGENTS.md](AGENTS.md).
