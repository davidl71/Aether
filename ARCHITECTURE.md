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

## Key Design Decisions

1. **Multi-language Architecture**: C++ for performance-critical calculations, Rust for safety-critical backend, Python for scripting/integration
2. **Message-driven**: NATS for decoupled, async communication
3. **Cross-platform**: Universal binaries for macOS, Linux support

## Directory Structure

See **`docs/design/DIRECTORY_STRUCTURE_BY_LANGUAGE.md`** for a language-to-directory mapping and discussion of by-component vs by-language layout.

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

- **API Reference**: `docs/API_DOCUMENTATION_INDEX.md`
- **NATS Integration**: `docs/NATS_INTEGRATION_STATUS.md`
- **Backend Design**: `docs/research/architecture/`
- **Build System**: `CMakeLists.txt`, `CMakePresets.json`

For complete project guidelines, see [AGENTS.md](AGENTS.md).
