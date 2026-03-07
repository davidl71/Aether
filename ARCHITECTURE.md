# Architecture Overview

Multi-asset synthetic financing platform. Box spreads are one strategy (7-10% allocation);
the platform manages financing across options, futures, bonds, bank loans, and pension funds
across 21+ accounts and multiple brokers.

**Last updated**: 2026-03-07 (post proto-consolidation sprint)

## System Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                         Client Applications                           │
├──────────────┬──────────────┬──────────────┬──────────────────────────┤
│  iPad/iOS    │  Web (React) │  Desktop     │  TUI (Python/Textual)    │
│  SwiftUI     │  :5173       │  Swift/AppKit│  polls :8000-:8006       │
└──────┬───────┴──────┬───────┴──────┬───────┴──────────┬───────────────┘
       │              │              │                   │
       │    ┌─────────┴──────────────┴──┐               │
       │    │  Rust REST+WS backend     │               │
       │    │  Axum :8080               │               │
       │    │  WebSocket /ws/snapshot   │               │
       └────┤  (full snapshot every 2s) │               │
            └──────────┬────────────────┘               │
                       │                                 │
     ┌─────────────────┼─────────────────────────────────┼─────────────┐
     │                 │              NATS               │             │
     │   ┌─────────────┴──────┐  ┌────────────────┐  ┌──┴──────────┐  │
     │   │ Python microservices│  │ Go agents      │  │ C++ engine  │  │
     │   │ :8000-:8006         │  │ api-gateway    │  │ tws_client  │  │
     │   │ (positions,rates,   │  │ heartbeat-agg  │  │ nats_client │  │
     │   │  risk,lending,etc.) │  │ nats-qdb-bridge│  │ ENABLE_PROTO│  │
     │   │ Redis cache         │  │ supervisor     │  └──────┬──────┘  │
     │   └─────────────────────┘  │ config-validator│         │         │
     │                            └────────────────┘         │         │
     └──────────────────────────────────────────────────────┼─────────┘
                                                             │
                                                      ┌──────┴──────┐
                                                      │  TWS/IBKR   │
                                                      │  port 7497  │
                                                      └─────────────┘

Storage layers:
  InMemoryCache (C++)  →  hot tick data
  Redis (Python)       →  inter-service cache
  NATS KV              →  live key-value state
  SQLite               →  Rust ledger + Python (SHARED — see known issues)
  QuestDB              →  time-series archive
  MongoDB              →  trade blotter
```

## Components

### Client Applications

| Component | Technology | Data Source | Update Mechanism |
|-----------|------------|-------------|------------------|
| iPad App | SwiftUI | Rust :8080 REST | Polling |
| Web SPA | React/TypeScript | Rust :8080 WebSocket → REST fallback | Full snapshot every 2s |
| Desktop | Swift/AppKit | Rust :8080 REST | Polling |
| TUI | Python/Textual | Python microservices :8000-:8006 | 1s polling (RestProvider) or NATS (NatsProvider) |
| CLI | C++ | Direct TWS | Synchronous |

### Backend Services

| Component | Technology | Purpose |
|-----------|------------|---------|
| Rust REST+WS | Rust (Axum) :8080 | HTTP API + WebSocket snapshot for web/desktop/iOS |
| Python microservices | Python (FastAPI) :8000-:8006 | Position, risk, rate, lending data for TUI |
| C++ engine | C++20 | TWS connectivity, strategy execution, risk/Greeks/pricing |
| NATS | NATS JetStream | Async messaging, market data events, heartbeats |
| Go agents | Go (stdlib+nats.go) | api-gateway, heartbeat-aggregator, nats-questdb-bridge, supervisor, config-validator |

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
3. **WebSocket sends full snapshot**: Rust WS backend sends complete `SystemSnapshot` every 2s regardless of what changed — inefficient at scale.
4. **NATS Go agents**: `nats-questdb-bridge` and `heartbeat-aggregator` parse raw NATS bytes as strings rather than deserializing `NatsEnvelope` protobuf.
5. **Hardcoded ETF duration table**: `greeks_calculator.cpp` uses a static lookup table for ETF duration/convexity instead of `QuantLib::BondFunctions`.
6. **No IV solver**: `greeks_calculator.cpp` takes implied volatility as external input; no Newton-Raphson solver over `BlackCalculator`.

## Technology Stack

| Layer | Technologies |
|-------|--------------|
| Core engine | C++20, QuantLib, Intel Decimal Library, NLopt, Eigen |
| Backend agents | Rust (Axum, prost, sqlx), Go (stdlib, nats.go) |
| Integration layer | Python 3.12 (FastAPI, Textual, betterproto, redis) |
| Frontend | React 18, TypeScript, ts-proto |
| Mobile/Desktop | SwiftUI (iOS), Swift/AppKit (macOS) |
| Messaging | NATS JetStream, Protocol Buffers |
| Storage | SQLite, Redis, QuestDB, MongoDB, NATS KV |
| Build | CMake/Ninja, Cargo, uv/pip, npm |
| Testing | Catch2, pytest, Jest, cargo test |

## Language Rationale

- **C++**: stays for core engine and TWS (API is C++-only)
- **Rust**: stays for safety-critical backend and ledger
- **Python**: integration layer, TUI, bindings — not a rewrite candidate
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
│   └── go/              # Go agents (api-gateway, heartbeat-agg, nats-qdb-bridge...)
├── python/              # FastAPI microservices, Textual TUI, integration
├── web/                 # React SPA
├── proto/               # Canonical protobuf schema (messages.proto)
├── ios/                 # SwiftUI iPad app
├── desktop/             # Swift/AppKit macOS app
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
