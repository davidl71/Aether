# Aether Architecture Overview

Multi-asset synthetic financing platform. Box spreads are one strategy (7-10% allocation);
the platform manages financing across options, futures, bonds, bank loans, and pension funds
across 21+ accounts and multiple brokers.

**Last updated**: 2026-03-11 (backend topology simplification)

## System Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Client Applications                          │
├──────────────────────────────┬──────────────────────────────────────┤
│ Web (React, archived)        │ TUI (Rust/Ratatui)                 │
│ Historical only              │ Rust read models + NATS/REST       |
└───────────────┬──────────────┴──────────────────┬───────────────────┘
                │                                 │
      ┌─────────┴──────────────┐       ┌──────────┴──────────┐
      │ Rust REST+WS backend   │       │ Specialist Python    │
      │ Axum :8080             │       │ outputs / helpers    │
      │ /api/v1/snapshot       │       │ broker/bank/rate     │
      │ /api/v1/frontend/*     │       │ health dashboard     │
      │ /ws/snapshot           │       └──────────┬──────────┘
      └──────────┬─────────────┘                  │
                 │                                │
     ┌───────────┴────────────────────────────────┴──────────────┐
     │                           NATS                              │
     │   ┌─────────────────────┐  ┌─────────────────────────────┐  │
     │   │ Rust backend        │  │ C++ engine                  │  │
     │   │ snapshot/read path  │  │ tws_client / nats_client    │  │
     │   │ ledger owner        │  │ pricing / risk / orders     │  │
     │   │ UI/API surface      │  │ protobuf event publisher    │  │
     │   └─────────────────────┘  └──────────────┬──────────────┘  │
     └───────────────────────────────────────────┼─────────────────┘
                                                 │
                                          ┌──────┴──────┐
                                          │  TWS/IBKR   │
                                          │  port 7497  │
                                          └─────────────┘

Storage layers:
  InMemoryCache (C++)  →  hot tick data
  NATS KV              →  live key-value state (written by collection-daemon as full envelopes)
  SQLite               →  Rust ledger owner; Python overlap is legacy technical debt
  QuestDB              →  time-series archive
```

## Components

### Client Applications

| Component | Technology | Data Source | Update Mechanism |
|-----------|------------|-------------|------------------|
| Web app | React/TypeScript | Rust `:8080` WebSocket → REST fallback | Full snapshot on connect, then changed sections every ~2s |
| Terminal UI | Rust/Ratatui | Rust read-model endpoints and NATS | Event-driven updates with Rust-owned UI state |
| CLI | C++ | Direct TWS | Synchronous |

### Backend Services

| Component | Technology | Purpose |
|-----------|------------|---------|
| Rust REST+WS backend | Rust (Axum) :8080 | Shared frontend API owner for snapshot and frontend read models consumed by web and TUI |
| Python helper surface | Python (generated/test helpers only) | pybind11 binding tests and generated artifacts; not an active runtime tier |
| C++ engine | C++20 | TWS connectivity, strategy execution, risk/Greeks/pricing |
| NATS | NATS JetStream | Async messaging, market data events, heartbeats |

### Messaging Contract

All C++ NATS messages use `NatsEnvelope` (protobuf) with serialized inner messages.
Topics: `market-data.tick.<symbol>`, `strategy.signal.<symbol>`, `strategy.decision.<symbol>`

See `docs/message_schemas/README.md` and `proto/messages.proto` for the canonical schema.

### Serialization

Generated from `proto/messages.proto` via `./proto/generate.sh`:

| Language | Output | Status |
|----------|--------|--------|
| C++ | `native/generated/messages.pb.{h,cc}` | Active — `ENABLE_PROTO` flag |
| Python | `native/generated/python/` (betterproto) | Generated helper output |
| TypeScript | `web/src/proto/messages.ts` (ts-proto) | Generated, migration pending |
| Go | `agents/go/proto/v1/messages.pb.go` | Active |
| Rust | `nats_adapter` crate (prost via build.rs) | Active |

## Known Issues / Technical Debt

See `docs/platform/DATAFLOW_ARCHITECTURE.md` for full analysis. Key issues:

1. **Legacy Python references in docs/config**: parts of the repo still describe retired Python runtime surfaces and need ongoing cleanup.
2. **Split read paths remain**: Rust owns the active TUI/backend path, but some docs and planning artifacts still describe older Python integration flows.
3. **WebSocket sends full snapshot**: Rust WS sends full snapshot once on connect, then only changed sections (delta) every 2s — see IMPROVEMENT_PLAN P2-A (done). Remaining gap: scale if many clients.
4. **Collector durability gap**: `collection-daemon` now decodes `NatsEnvelope` and owns `LIVE_STATE`, but durable JetStream replay remains opt-in instead of the default collection mode.
5. **Hardcoded ETF duration table**: `greeks_calculator.cpp` uses a static lookup table for ETF duration/convexity instead of `QuantLib::BondFunctions`.
6. **No IV solver**: `greeks_calculator.cpp` takes implied volatility as external input; no Newton-Raphson solver over `BlackCalculator`.

## Technology Stack

| Layer | Technologies |
|-------|--------------|
| Core engine | C++20, QuantLib, Intel Decimal Library, NLopt, Eigen |
| Backend services | Rust (Axum, prost, sqlx), Go (stdlib, nats.go) |
| Integration layer | Python helpers (pybind11 binding tests, generated betterproto output) |
| Frontends | Rust Ratatui TUI, native CLI, archived React 18/TypeScript |
| Messaging | NATS JetStream, Protocol Buffers |
| Storage | SQLite, QuestDB, NATS KV |
| Build | CMake/Ninja, Cargo, uv, npm |
| Testing | Catch2, pytest, cargo test, Vitest |

## Language Rationale

- **C++**: stays for core engine and TWS (API is C++-only)
- **Rust**: stays for safety-critical backend and ledger
- **Python**: binding tests and generated helper artifacts where needed
- **TypeScript**: web app — not a rewrite candidate

## Directory Structure

```
ib_box_spread_full_universal/
├── native/              # C++ core (engine, CLI, tests)
│   ├── src/             # tws_client, nats_client, greeks, risk, order mgmt...
│   ├── include/
│   ├── tests/           # Catch2
│   └── third_party/     # TWS API, Intel Decimal, QuantLib (via FetchContent)
├── agents/
│   └── backend/         # Rust backend (Axum REST, ledger, nats_adapter, TUI)
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
