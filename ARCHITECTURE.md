# Architecture Overview

Multi-asset synthetic financing platform. Box spreads are one strategy (7-10% allocation);
the platform manages financing across options, futures, bonds, bank loans, and pension funds
across 21+ accounts and multiple brokers.

**Last updated**: 2026-03-10 (frontend and language usage cleanup)

## System Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Client Applications                          │
├──────────────────────────────┬──────────────────────────────────────┤
│ Web (React)                  │ TUI (Python/Textual)                │
│ WebSocket + REST from :8080  │ Polling from :8000-:8006 or NATS    │
└───────────────┬──────────────┴──────────────────┬───────────────────┘
                │                                 │
      ┌─────────┴──────────────┐       ┌──────────┴──────────┐
      │ Rust REST+WS backend   │       │ Python microservices │
      │ Axum :8080             │       │ :8000-:8006          │
      │ /api/snapshot          │       │ positions/rates/risk │
      │ /ws/snapshot           │       │ lending/blotter/etc. │
      └──────────┬─────────────┘       └──────────┬──────────┘
                 │                                 │
     ┌───────────┴─────────────────────────────────┴──────────────┐
     │                           NATS                              │
     │   ┌─────────────────────┐  ┌─────────────────────────────┐  │
     │   │ Go agents           │  │ C++ engine                  │  │
     │   │ collection-daemon   │  │ tws_client / nats_client    │  │
     │   │ heartbeat-agg       │  │ pricing / risk / orders     │  │
     │   │ nats-qdb-bridge     │  │ protobuf event publisher    │  │
     │   │ supervisor/config   │  └──────────────┬──────────────┘  │
     │   └─────────────────────┘                 │                 │
     └───────────────────────────────────────────┼─────────────────┘
                                                 │
                                          ┌──────┴──────┐
                                          │  TWS/IBKR   │
                                          │  port 7497  │
                                          └─────────────┘

Storage layers:
  InMemoryCache (C++)  →  hot tick data
  Redis (Python)       →  inter-service cache
  NATS KV              →  live key-value state (written by collection-daemon as full envelopes)
  SQLite               →  Rust ledger + Python (SHARED — see known issues)
  QuestDB              →  time-series archive
  MongoDB              →  trade blotter
```

## Components

### Client Applications

| Component | Technology | Data Source | Update Mechanism |
|-----------|------------|-------------|------------------|
| Web app | React/TypeScript | Rust `:8080` WebSocket → REST fallback | Full snapshot on connect, then changed sections every ~2s |
| Terminal UI | Python/Textual | Python microservices `:8000-:8006` or NATS | 1s polling (`RestProvider`) or event-driven (`NatsProvider`) |
| CLI | C++ | Direct TWS | Synchronous |

### Backend Services

| Component | Technology | Purpose |
|-----------|------------|---------|
| Rust REST+WS backend | Rust (Axum) :8080 | HTTP API + snapshot/delta WebSocket for the web client |
| Python microservices | Python (FastAPI) :8000-:8006 | Positions, rates, risk, lending, blotter, alerts, health for the Textual TUI |
| C++ engine | C++20 | TWS connectivity, strategy execution, risk/Greeks/pricing |
| NATS | NATS JetStream | Async messaging, market data events, heartbeats |
| Go agents | Go (stdlib+nats.go) | Collection, health aggregation, QuestDB bridge, supervisor, config validation |

### Messaging Contract

All C++ NATS messages use `NatsEnvelope` (protobuf) with serialized inner messages.
Topics: `market-data.tick.<symbol>`, `strategy.signal.<symbol>`, `strategy.decision.<symbol>`

See `docs/message_schemas/README.md` and `proto/messages.proto` for the canonical schema.

### Serialization

Generated from `proto/messages.proto` via `./proto/generate.sh`:

| Language | Output | Status |
|----------|--------|--------|
| C++ | `native/generated/messages.pb.{h,cc}` | Active — `ENABLE_PROTO` flag |
| Python | `python/generated/` (betterproto) | Generated, NATS migration pending |
| TypeScript | `web/src/proto/messages.ts` (ts-proto) | Generated, migration pending |
| Go | `agents/go/proto/v1/messages.pb.go` | Active |
| Rust | `nats_adapter` crate (prost via build.rs) | Active |

## Known Issues / Technical Debt

See `docs/platform/DATAFLOW_ARCHITECTURE.md` for full analysis. Key issues:

1. **Dual SQLite writers**: Rust ledger and Python both write to the same SQLite DB — risk of contention/corruption under load.
2. **Split data backends**: TUI reads from Python :8000-:8006; Web reads from Rust :8080 — different data, potential inconsistency.
3. **WebSocket sends full snapshot**: Rust WS sends full snapshot once on connect, then only changed sections (delta) every 2s — see IMPROVEMENT_PLAN P2-A (done). Remaining gap: scale if many clients.
4. **Collector durability gap**: `collection-daemon` now decodes `NatsEnvelope` and owns `LIVE_STATE`, but durable JetStream replay remains opt-in instead of the default collection mode.
5. **Hardcoded ETF duration table**: `greeks_calculator.cpp` uses a static lookup table for ETF duration/convexity instead of `QuantLib::BondFunctions`.
6. **No IV solver**: `greeks_calculator.cpp` takes implied volatility as external input; no Newton-Raphson solver over `BlackCalculator`.

## Technology Stack

| Layer | Technologies |
|-------|--------------|
| Core engine | C++20, QuantLib, Intel Decimal Library, NLopt, Eigen |
| Backend services | Rust (Axum, prost, sqlx), Go (stdlib, nats.go) |
| Integration layer | Python 3.12 (FastAPI, Textual, betterproto, redis) |
| Frontends | React 18, TypeScript, Textual |
| Messaging | NATS JetStream, Protocol Buffers |
| Storage | SQLite, Redis, QuestDB, MongoDB, NATS KV |
| Build | CMake/Ninja, Cargo, uv, npm |
| Testing | Catch2, pytest, cargo test, Vitest |

## Language Rationale

- **C++**: stays for core engine and TWS (API is C++-only)
- **Rust**: stays for safety-critical backend and ledger
- **Python**: integration layer, Textual TUI, bindings — not a rewrite candidate
- **Go**: ops agents — good for single-binary CLI/bridge tools
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
│   ├── backend/         # Rust backend (Axum REST, ledger, nats_adapter)
│   └── go/              # Go agents (api-gateway, collection-daemon, heartbeat-agg, nats-qdb-bridge...)
├── python/              # FastAPI microservices, Textual TUI, integration
├── web/                 # React web client
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
