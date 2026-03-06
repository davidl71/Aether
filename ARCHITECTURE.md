# Architecture Overview

This document provides a high-level overview of the IBKR Box Spread Generator system architecture.

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Applications                       │
├──────────┬──────────┬──────────┬──────────┬────────────────────┤
│  iPad    │   Web    │ Desktop  │   TUI    │        CLI         │
│ SwiftUI  │  React   │  Swift   │  FTXUI   │      C++/Python    │
└────┬─────┴────┬─────┴────┬─────┴────┬─────┴──────────┬─────────┘
     │          │          │          │                │
     └──────────┴──────────┴──────────┴────────────────┘
                           │
                    ┌──────┴──────┐
                    │  REST API   │
                    │   (Rust)    │
                    └──────┬──────┘
                           │
     ┌─────────────────────┼─────────────────────┐
     │                     │                     │
┌────┴─────┐        ┌──────┴──────┐       ┌─────┴─────┐
│  NATS    │        │   Backend   │       │  Market   │
│ Messaging│◄──────►│   Engine    │◄─────►│   Data    │
└──────────┘        │   (Rust)    │       │  Gateway  │
                    └──────┬──────┘       └─────┬─────┘
                           │                    │
                    ┌──────┴──────┐       ┌─────┴─────┐
                    │   Core      │       │   TWS     │
                    │ Calculations│       │   API     │
                    │   (C++)     │       │  (IBKR)   │
                    └─────────────┘       └───────────┘
```

## Components

### Client Applications

| Component | Technology | Purpose |
|-----------|------------|---------|
| iPad App | SwiftUI | Mobile trading interface |
| Web SPA | React/TypeScript | Browser-based dashboard |
| Desktop | Swift/AppKit | Native macOS application |
| TUI | FTXUI (C++) | Terminal user interface |
| CLI | C++/Python | Command-line tools |

### Backend Services

| Component | Technology | Purpose |
|-----------|------------|---------|
| REST API | Rust (Axum) | HTTP API for clients |
| Backend Engine | Rust | Strategy execution, order management |
| NATS | NATS JetStream | Async messaging, event streaming |
| Core Calculations | C++ | Options pricing, risk calculations |

### External Integrations

| Integration | Protocol | Purpose |
|-------------|----------|---------|
| TWS API | TCP Socket | Interactive Brokers connectivity |
| Market Data | WebSocket/REST | Real-time quotes, historical data |

## Data Flow

1. **Market Data Ingestion**
   - TWS API → Market Data Gateway → NATS → Backend Engine

2. **Strategy Execution**
   - Client → REST API → Backend Engine → Core Calculations → TWS API

3. **Real-time Updates**
   - NATS → WebSocket → Clients

## Technology Stack

| Layer | Technologies |
|-------|--------------|
| Frontend | SwiftUI, React, TypeScript, FTXUI |
| Backend | Rust, C++, Python |
| Messaging | NATS JetStream |
| Build | CMake, Cargo, npm |
| Testing | Catch2, pytest, Jest |

## Languages and components

A **language–component table** and rationale are maintained in **`docs/design/DIRECTORY_STRUCTURE_BY_LANGUAGE.md`**. Summary:

- **No broad language change is recommended.** C++ stays for the core engine and TWS (API is C++-only); Rust for the backend service; Python for integration and TUI; TypeScript for the web app; Node for the Israeli bank scrapers service.
- **Go** is used in `agents/go/` for agents/tools. Good candidates to **rewrite in Go** are small, standalone CLI/ops pieces that would benefit from a single binary and no runtime (e.g. config validator, small NATS bridge). The main Python broker services, the Rust backend, and the Node scrapers service are **not** recommended rewrites. See the same doc for details.

## Key Design Decisions

1. **Multi-language Architecture**: C++ for performance-critical calculations, Rust for safety-critical backend, Python for scripting/integration
2. **Message-driven**: NATS for decoupled, async communication
3. **Cross-platform**: Universal binaries for macOS, Linux support
4. **Independent providers**: Discount (bank), Alpaca, Tastytrade, TradeStation, and IB are independent—accounts and positions can exist in parallel. So are the Israeli providers in config (`broker.priorities`): Fibi, Meitav, IBI, and Discount; the Israeli bank scrapers service supports multiple company IDs (**fibi**, **max**, **visaCal**, discount, leumi, hapoalim, etc.), each independent. See `docs/platform/MULTI_ACCOUNT_AGGREGATION_DESIGN.md`.

## Directory Structure

See **`docs/design/DIRECTORY_STRUCTURE_BY_LANGUAGE.md`** for the language–component table, rationale (no broad language changes), where Go fits, and rewrite-in-Go guidance.

```
ib_box_spread_full_universal/
├── native/              # C++ core (calculations, TUI, CLI)
│   ├── src/
│   ├── include/
│   └── tests/
├── agents/
│   └── backend/         # Rust backend services
├── python/              # Python integration layer
├── web/                 # React web application
├── ios/                 # iOS/iPad application
├── desktop/             # macOS desktop application
└── docs/                # Documentation
```

## Detailed Documentation

- **What we track vs ignore**: `docs/planning/TRACKING_AND_GITIGNORE.md`
- **API Reference**: `docs/API_DOCUMENTATION_INDEX.md`
- **NATS Integration**: `docs/NATS_INTEGRATION_STATUS.md`
- **Backend Design**: `docs/research/architecture/`
- **Build System**: `CMakeLists.txt`, `CMakePresets.json`

For complete project guidelines, see [AGENTS.md](AGENTS.md).
