# Aether Architecture Overview

Multi-asset synthetic financing platform. Box spreads are one strategy (7-10% allocation);
the platform manages financing across options, futures, bonds, bank loans, and pension funds
across 21+ accounts and multiple brokers.

**Last updated**: 2026-03-14 (architecture docs and ownership)

## Current build and runtime settings

- **Active runtime**: Rust backend (`backend_service` :8080) and Rust TUI (`tui_service`). C++ native build is **removed** (see root `CMakeLists.txt`).
- **Broker connectivity**: Rust-owned; IBKR routes and adapter in `agents/backend/crates/ib_adapter` and API in `crates/api`.
- **Daemons**: `nats` (4222) and `rust_backend` (8080) only. See `docs/BACKEND_SERVICES_DAEMONIZED.md`.

## System Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Client Applications                          │
├──────────────────────────────┬──────────────────────────────────────┤
│ Web (React, archived)        │ TUI (Rust/Ratatui) + CLI (Rust bin)   │
│ Historical only              │ Rust read models + NATS/REST          │
└───────────────┬──────────────┴──────────────────┬───────────────────┘
                │                                 │
                └──────────────┬──────────────────┘
                               │
      ┌────────────────────────┴────────────────────────┐
      │ Rust REST+WS backend (Axum :8080)                │
      │ /api/v1/snapshot, /api/v1/frontend/*, /ws/snapshot
      │ Owns: snapshot, read models, IBKR routes        │
      └────────────────────────┬────────────────────────┘
                               │
     ┌─────────────────────────┴─────────────────────────┐
     │ NATS (JetStream) — Rust backend: collector,       │
     │ LIVE_STATE KV, QuestDB fanout                      │
     └─────────────────────────┬─────────────────────────┘
                               │
     ┌─────────────────────────┴─────────────────────────┐
     │ TWS/IBKR (port 7497) — via Rust ib_adapter          │
     └─────────────────────────────────────────────────────┘

Storage:
  NATS KV   → live key-value state (Rust backend collector)
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
