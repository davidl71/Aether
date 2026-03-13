# Architecture Overview

Multi-asset synthetic financing platform. Box spreads are one strategy (7-10% allocation);
the platform manages financing across options, futures, bonds, bank loans, and pension funds
across 21+ accounts and multiple brokers.

**Last updated**: 2026-03-13 (NautilusTrader IB adapter integration)

## System Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Client Applications                          │
├──────────────────────────────┬──────────────────────────────────────┤
│ Web (React)                  │ TUI (Python/Textual)                │
│ WebSocket + REST from :8080  │ Rust read models + Python integrations |
└───────────────┬──────────────┴──────────────────┬───────────────────┘
                │                                 │
      ┌─────────┴──────────────┐       ┌──────────┴──────────┐
      │ Rust REST+WS backend   │       │ Python integration   │
      │ Axum :8080             │       │ services             │
      │ /api/v1/snapshot       │       │ broker/bank/rate     │
      │ /api/v1/frontend/*     │       │ health dashboard     │
      │ /ws/snapshot           │       └──────────┬──────────┘
      └──────────┬─────────────┘                  │
                 │                                │
     ┌───────────┴────────────────────────────────┴──────────────┐
     │                           NATS                              │
     │   ┌──────────────────────────┐  ┌───────────────────────┐ │
     │   │ NautilusTrader agent     │  │ C++ engine            │ │
     │   │ agents/nautilus/         │  │ (calculations only    │ │
     │   │ BoxSpreadStrategy        │  │  when NAUTILUS_BROKER │ │
     │   │ NatsBridge               │  │  _ENABLED is set;     │ │
     │   │ NatsEnvelope publisher   │  │  no TWS connection)   │ │
     │   └────────────┬─────────────┘  └───────────────────────┘ │
     └────────────────┼──────────────────────────────────────────┘
                      │ owns IB connection
                ┌─────┴──────┐
                │  TWS/IBKR  │
                │  port 7497 │  ← paper | 7496 live
                └────────────┘

Storage layers:
  InMemoryCache (C++)  →  hot tick data
  NATS KV              →  live key-value state (written by collection-daemon as full envelopes)
  SQLite               →  Rust ledger owner; Python overlap is legacy technical debt
  QuestDB              →  time-series archive

NautilusTrader mode (ENABLE_NAUTILUS_BROKER=ON):
  NT Python agent (agents/nautilus/) owns the IBKR TWS connection.
  C++ TWSClient is suppressed at compile time; C++ engine runs in mock/calc-only mode.
  NT NatsBridge publishes NatsEnvelope protos to the same NATS topics as before:
    market-data.tick.{symbol}, strategy.decision.{symbol}, orders.fill.{id}, positions.update.{symbol}
  Rust backend and TUI are unchanged — they continue consuming from NATS normally.
```

## Components

### Client Applications

| Component | Technology | Data Source | Update Mechanism |
|-----------|------------|-------------|------------------|
| Web app | React/TypeScript | Rust `:8080` WebSocket → REST fallback | Full snapshot on connect, then changed sections every ~2s |
| Terminal UI | Python/Textual | Rust read-model endpoints, selected Python integration services, or NATS | Worker-driven fetches and event-driven updates |
| CLI | C++ | Direct TWS | Synchronous |

### Backend Services

| Component | Technology | Purpose |
|-----------|------------|---------|
| Rust REST+WS backend | Rust (Axum) :8080 | Shared frontend API owner for snapshot and frontend read models consumed by web and TUI |
| Python integration services | Python (FastAPI) | Explicit specialist services only: broker/bank integrations, risk-free-rate service, and health dashboard |
| **NautilusTrader agent** | **Python + NT 1.224.0** | **Primary IBKR connection (replaces C++ TWSClient when `ENABLE_NAUTILUS_BROKER=ON`). BoxSpreadStrategy, NatsBridge.** |
| C++ engine | C++20 | Risk/Greeks/pricing/margin calculations. TWS connectivity disabled when `NAUTILUS_BROKER_ENABLED` is defined. |
| NATS | NATS JetStream | Async messaging, market data events, heartbeats |

### Messaging Contract

All NATS messages use `NatsEnvelope` (protobuf) with serialized inner messages.
Published by the C++ engine (default) or the NautilusTrader agent (`ENABLE_NAUTILUS_BROKER=ON`).
Topics: `market-data.tick.<symbol>`, `strategy.signal.<symbol>`, `strategy.decision.<symbol>`,
        `orders.fill.<order_id>`, `positions.update.<symbol>`

See `docs/message_schemas/README.md` and `proto/messages.proto` for the canonical schema.

### Serialization

Generated from `proto/messages.proto` via `./proto/generate.sh`:

| Language | Output | Status |
|----------|--------|--------|
| C++ | `native/generated/messages.pb.{h,cc}` | Active — `ENABLE_PROTO` flag |
| Python (legacy) | `python/generated/` (betterproto) | Generated, NATS migration pending |
| Python (NT agent) | `agents/nautilus/src/nautilus_agent/generated/` (google-protobuf) | Run `just proto-gen-nautilus` to generate |
| TypeScript | `web/src/proto/messages.ts` (ts-proto) | Generated, migration pending |
| Rust | `nats_adapter` crate (prost via build.rs) | Active |

## Known Issues / Technical Debt

See `docs/platform/DATAFLOW_ARCHITECTURE.md` for full analysis. Key issues:

1. **Dual SQLite writers**: Rust ledger owns SQLite, but some legacy Python overlap remains — risk of contention/corruption until fully removed.
2. **Split read paths remain**: TUI still mixes Rust read models, Python integration endpoints, and NATS, while the web is primarily Rust-backed.
3. **WebSocket sends full snapshot**: Rust WS sends full snapshot once on connect, then only changed sections (delta) every 2s — see IMPROVEMENT_PLAN P2-A (done). Remaining gap: scale if many clients.
4. **Collector durability gap**: `collection-daemon` now decodes `NatsEnvelope` and owns `LIVE_STATE`, but durable JetStream replay remains opt-in instead of the default collection mode.
5. **Hardcoded ETF duration table**: `greeks_calculator.cpp` uses a static lookup table for ETF duration/convexity instead of `QuantLib::BondFunctions`.
6. **No IV solver**: `greeks_calculator.cpp` takes implied volatility as external input; no Newton-Raphson solver over `BlackCalculator`.

## Technology Stack

| Layer | Technologies |
|-------|--------------|
| Core engine | C++20, QuantLib, Intel Decimal Library, NLopt, Eigen |
| Backend services | Rust (Axum, prost, sqlx) |
| Integration layer | Python 3.12 (specialist services, bindings, betterproto) |
| Frontends | Rust Ratatui TUI, native CLI, archived React 18/TypeScript |
| Messaging | NATS JetStream, Protocol Buffers |
| Storage | SQLite, QuestDB, NATS KV |
| Build | CMake/Ninja, Cargo, uv, npm |
| Testing | Catch2, pytest, cargo test, Vitest |

## Language Rationale

- **C++**: stays for core engine and TWS (API is C++-only)
- **Rust**: stays for safety-critical backend and ledger
- **Python**: specialist broker/bank integrations, bindings, and remaining finance helpers
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
│   └── backend/         # Rust backend (Axum REST, ledger, nats_adapter)
├── python/              # Python integration services, bindings, and research helpers
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
