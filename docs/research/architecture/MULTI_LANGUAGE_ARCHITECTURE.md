# Multi-Language Architecture Guide

**Date**: 2025-11-17
**Status**: Active Documentation
**Purpose**: Comprehensive guide to the multi-language architecture (C++, Python, Rust, Go, TypeScript) and how components interact

---

## Overview

The IBKR Box Spread Generator uses a multi-language architecture to leverage the strengths of each language:

- **C++**: Core trading engine, TWS API integration, performance-critical calculations
- **Python**: Strategy development, data analysis, integration layer, TUI
- **Rust**: Backend services, agent coordination, high-performance data processing
- **Go**: Market data ingestion, QuestDB integration, microservices
- **TypeScript**: Web frontend, PWA, real-time UI

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  C++ CLI/TUI │  │ Python TUI   │  │ TypeScript   │          │
│  │  (FTXUI)     │  │ (Textual)    │  │ Web App      │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
└─────────┼─────────────────┼─────────────────┼──────────────────┘
          │                 │                 │
┌─────────┼─────────────────┼─────────────────┼──────────────────┐
│         │    Core Trading Engine (C++)        │                   │
│  ┌──────▼─────────────────────────────────────▼──────┐          │
│  │  Box Spread Strategy  │  Order Manager  │  Risk   │          │
│  │  (C++)                │  (C++)          │  (C++)  │          │
│  └──────┬────────────────┴──────┬──────────┴──────┬─┘          │
│         │                       │                  │             │
│  ┌──────▼───────────────────────▼──────────────────▼──────┐    │
│  │              TWS Client (C++)                          │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │    │
│  │  │  Market Data │  │  Order Exec  │  │  Positions   │ │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘ │    │
│  └────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
          │
┌─────────┼─────────────────────────────────────────────────────────┐
│         │    Python Integration Layer                              │
│  ┌──────▼──────────────────────────────────────────────────────┐ │
│  │  Cython Bindings (C++ → Python)                            │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │ │
│  │  │  Box Spread  │  │  Option Chain│  │  Risk Calc   │    │ │
│  │  │  Calculations│  │  Manager     │  │  Functions   │    │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘    │ │
│  └──────┬──────────────────────────────────────────────────────┘ │
│         │                                                         │
│  ┌──────▼──────────────────────────────────────────────────────┐ │
│  │  Python Strategy Runner                                      │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │ │
│  │  │  Nautilus    │  │  Alpaca      │  │  ORATS       │    │ │
│  │  │  Integration │  │  Client      │  │  Fallback   │    │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘    │ │
│  └────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
          │
┌─────────┼─────────────────────────────────────────────────────────┐
│         │    Rust Backend Services                                 │
│  ┌──────▼──────────────────────────────────────────────────────┐ │
│  │  Backend Agent (Rust)                                       │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │ │
│  │  │  REST API    │  │  gRPC        │  │  Strategy    │    │ │
│  │  │  (Axum)      │  │  Service     │  │  Engine      │    │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘    │ │
│  └──────┬──────────────────────────────────────────────────────┘ │
│         │                                                         │
│  ┌──────▼──────────────────────────────────────────────────────┐ │
│  │  Market Data Service (Rust)                                 │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │ │
│  │  │  gRPC Stream │  │  Health      │  │  Data        │    │ │
│  │  │  Service     │  │  Checks      │  │  Processing  │    │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘    │ │
│  └────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
          │
┌─────────┼─────────────────────────────────────────────────────────┐
│         │    Go Services                                           │
│  ┌──────▼──────────────────────────────────────────────────────┐ │
│  │  Market Data Ingestion Gateway (Go)                         │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │ │
│  │  │  QuestDB     │  │  Data        │  │  Time-Series │    │ │
│  │  │  Integration │  │  Ingestion   │  │  Archive     │    │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘    │ │
│  └────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
          │
┌─────────┼─────────────────────────────────────────────────────────┐
│         │    TypeScript Web Frontend                                │
│  ┌──────▼──────────────────────────────────────────────────────┐ │
│  │  React PWA                                                   │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │ │
│  │  │  Dashboard   │  │  Positions   │  │  Orders      │    │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘    │ │
│  └──────┬──────────────────────────────────────────────────────┘ │
│         │                                                         │
│  ┌──────▼──────────────────────────────────────────────────────┐ │
│  │  REST API Client (TypeScript)                                │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │ │
│  │  │  Alpaca      │  │  Backend     │  │  WebSocket   │    │ │
│  │  │  Service     │  │  API         │  │  (Future)    │    │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘    │ │
│  └────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

---

## Language-Specific Components

### C++ Core (`native/`)

**Purpose**: Performance-critical trading engine and TWS API integration

**Key Components**:

- **TWS Client** (`native/src/tws_client.cpp`): Interactive Brokers API integration
- **Box Spread Strategy** (`native/src/box_spread_strategy.cpp`): Core arbitrage detection
- **Order Manager** (`native/src/order_manager.cpp`): Multi-leg order execution
- **Risk Calculator** (`native/src/risk_calculator.cpp`): Position risk assessment
- **Option Chain** (`native/src/option_chain.cpp`): Option data management
- **CLI/TUI** (`native/src/ib_box_spread.cpp`, `native/src/tui_app.cpp`): User interfaces

**Build System**: CMake with universal binary support (Intel + Apple Silicon)

**Dependencies**:

- TWS API (libtwsapi.dylib)
- Protocol Buffers
- Intel Decimal Math Library
- Abseil, spdlog, nlohmann/json, CLI11, Catch2

**Interfaces**:

- Cython bindings for Python integration
- Header-only interfaces for C++ clients

---

### Python Integration (`python/`)

**Purpose**: Strategy development, data analysis, integration layer

**Key Components**:

- **Cython Bindings** (`python/bindings/`): C++ → Python bridge
- **Strategy Runner** (`python/integration/strategy_runner.py`): Strategy execution
- **Nautilus Integration** (`python/integration/nautilus_strategy.py`): NautilusTrader bridge
- **Alpaca Client** (`python/integration/alpaca_client.py`): Alpaca API integration
- **TUI** (`python/tui/`): Textual-based terminal UI
- **Data Tools** (`python/tools/`): Analysis and backtesting tools

**Build System**: setuptools with Cython compilation

**Dependencies**:

- Cython 3.0+
- NautilusTrader (optional)
- Alpaca SDK (optional)
- Textual (for TUI)
- FastAPI (for services)

**Interfaces**:

- Cython bindings expose C++ functions
- REST API services (FastAPI)
- Python modules for strategy development

---

### Rust Backend Services (`agents/backend/`)

**Purpose**: High-performance backend services, agent coordination

**Key Components**:

- **Backend Agent** (`agents/backend/services/backend_service/`): Main backend service
- **REST API** (`agents/backend/crates/api/src/rest.rs`): HTTP endpoints
- **gRPC Service** (`agents/backend/crates/api/src/grpc.rs`): gRPC streaming
- **Strategy Engine** (`agents/backend/crates/strategy/`): Strategy decision logic
- **Risk Checks** (`agents/backend/crates/risk/`): Risk validation
- **State Management** (`agents/backend/crates/api/src/state.rs`): Shared state

**Build System**: Cargo (Rust package manager)

**Dependencies**:

- Axum (REST framework)
- Tonic (gRPC framework)
- Tokio (async runtime)
- Tracing (logging)

**Interfaces**:

- REST API (HTTP/JSON)
- gRPC (Protocol Buffers)
- Health check endpoints

---

### Go Services (`agents/backend-market-data/`, future)

**Purpose**: Market data ingestion, QuestDB integration, microservices

**Key Components**:

- **Market Data Gateway** (planned): High-throughput data ingestion
- **QuestDB Service** (planned): Time-series data archiving
- **Build Coordinator** (planned): Distributed build coordination

**Build System**: Go modules

**Dependencies**:

- QuestDB client library
- gRPC (for inter-service communication)

**Interfaces**:

- gRPC services
- REST APIs (if needed)

---

### TypeScript Web Frontend (`web/`)

**Purpose**: Modern web interface, PWA, real-time updates

**Key Components**:

- **React App** (`web/src/`): Main application
- **Components** (`web/src/components/`): UI components
- **Services** (`web/src/services/`): API clients
- **State Management** (`web/src/state/`): Application state

**Build System**: Vite (modern build tool)

**Dependencies**:

- React 18+
- TypeScript 5+
- Vite
- React Router
- State management library

**Interfaces**:

- REST API client
- WebSocket (future)
- Alpaca Service API

---

## Component Interactions

### 1. C++ → Python (Cython Bindings)

**Location**: `python/bindings/box_spread_bindings.pyx`

**Purpose**: Expose C++ calculations to Python

**Example**:

```python
from box_spread_bindings import calculate_arbitrage_profit, calculate_roi

# Call C++ functions from Python
profit = calculate_arbitrage_profit(spread)
roi = calculate_roi(spread)
```

**Data Flow**:

- Python calls Cython function
- Cython marshals Python objects to C++ types
- C++ function executes
- Results marshaled back to Python

---

### 2. Python → Rust (REST/gRPC)

**Location**: `agents/backend/crates/api/src/rest.rs`, `agents/backend/crates/api/src/grpc.rs`

**Purpose**: Python services communicate with Rust backend

**Example**:

```python
import requests

# Call Rust REST API
response = requests.get("http://localhost:8080/api/v1/snapshot")
snapshot = response.json()
```

**Data Flow**:

- Python makes HTTP request to Rust service
- Rust processes request
- Returns JSON response
- Python parses and uses data

---

### 3. Rust → Go (gRPC)

**Location**: `agents/backend-market-data/` (planned)

**Purpose**: Rust backend sends market data to Go ingestion service

**Example**:

```rust
// Rust client calls Go service
let mut client = MarketDataClient::connect("http://localhost:50061").await?;
client.stream_market_data(request).await?;
```

**Data Flow**:

- Rust gRPC client calls Go service
- Go service processes and stores in QuestDB
- Returns acknowledgment

---

### 4. TypeScript → Python/Rust (REST API)

**Location**: `web/src/services/`, `python/integration/alpaca_service.py`

**Purpose**: Web frontend fetches data from backend services

**Example**:

```typescript
// TypeScript client calls Python service
const response = await fetch('http://localhost:8000/api/v1/snapshot');
const snapshot = await response.json();
```

**Data Flow**:

- TypeScript makes HTTP request
- Python/Rust service processes
- Returns JSON response
- TypeScript updates UI

---

## Data Structures & Contracts

### Shared Data Types

**Protocol Buffers** (`agents/backend/proto/`, `agents/backend-market-data/proto/`):

- Used for gRPC communication
- Language-agnostic data serialization
- Ensures type safety across languages

**JSON** (`python/integration/alpaca_service.py`):

- Used for REST API communication
- Human-readable format
- Easy to debug and inspect

**C++ Types** (`native/include/types.h`):

- Core data structures
- Exposed via Cython to Python
- Used throughout C++ codebase

---

## Build & Integration

### Building C++ Components

```bash
cmake --preset macos-universal-debug
cmake --build --preset macos-universal-debug
```

### Building Python Bindings

```bash
cd python/bindings
pip install -e .
```

### Building Rust Services

```bash
cd agents/backend
cargo build --release
```

### Building TypeScript Web App

```bash
cd web
npm install
npm run build
```

---

## Communication Patterns

### 1. Synchronous Calls (C++ → Python)

- **Use Case**: Immediate calculation results
- **Pattern**: Direct function calls via Cython
- **Latency**: < 1ms (in-process)

### 2. Asynchronous REST (TypeScript → Python/Rust)

- **Use Case**: UI data fetching
- **Pattern**: HTTP requests with async/await
- **Latency**: 10-100ms (network dependent)

### 3. Streaming gRPC (Rust → Go)

- **Use Case**: Real-time market data
- **Pattern**: Bidirectional streaming
- **Latency**: < 10ms (local network)

### 4. Event-Driven (Python Strategy Runner)

- **Use Case**: Strategy execution
- **Pattern**: Callbacks and event handlers
- **Latency**: Event-driven (variable)

---

## Code Consistency & Drift Prevention

### Current Challenges

1. **Box Spread Calculations**: C++ and Python implementations may diverge
2. **Broker API Integration**: Different patterns (TWS vs REST)
3. **Strategy Logic**: C++, Rust, Python implementations
4. **Data Transformation**: Multiple providers constructing same structures

### Solutions

1. **Pseudocode Documentation**: See `docs/PSEUDOCODE_IMPLEMENTATION_STRATEGY.md`
2. **Shared Test Suites**: Validate behavior across languages
3. **Protocol Buffers**: Type-safe data contracts
4. **API Contracts**: Shared interface definitions

---

## Development Workflow

### Adding a New Feature

1. **Design**: Document in pseudocode (language-agnostic)
2. **Implement**: Start with C++ core (if performance-critical)
3. **Expose**: Add Cython bindings (if Python access needed)
4. **Integrate**: Add REST/gRPC endpoints (if service access needed)
5. **Test**: Validate across all languages
6. **Document**: Update architecture docs

### Cross-Language Testing

1. **Unit Tests**: Language-specific test suites
2. **Integration Tests**: Test language boundaries
3. **End-to-End Tests**: Full workflow validation
4. **Paper Trading**: Real-world validation

---

## Performance Considerations

### C++ (Core Engine)

- **Latency**: < 1ms for calculations
- **Throughput**: 1000+ opportunities/second
- **Memory**: Minimal allocations, RAII patterns

### Python (Strategy Development)

- **Latency**: 1-10ms (Cython bindings)
- **Throughput**: Limited by GIL (use multiprocessing)
- **Memory**: Higher overhead, garbage collected

### Rust (Backend Services)

- **Latency**: < 1ms (zero-cost abstractions)
- **Throughput**: 10,000+ requests/second
- **Memory**: Safe, no garbage collection

### Go (Data Ingestion)

- **Latency**: < 5ms (goroutines)
- **Throughput**: 100,000+ events/second
- **Memory**: Efficient GC, low overhead

### TypeScript (Web Frontend)

- **Latency**: Network-dependent (10-100ms)
- **Throughput**: UI-limited (60 FPS)
- **Memory**: Browser-managed

---

## Deployment Architecture

### Development

- All services run locally
- Mock TWS client for testing
- Direct file system access

### Production

- **C++ Binary**: Runs on trading server
- **Python Services**: FastAPI on application server
- **Rust Services**: Backend services on API server
- **Go Services**: Data ingestion on data server
- **TypeScript Web**: Static files on CDN/web server

---

## Troubleshooting

### Common Issues

1. **Cython Import Errors**: Rebuild bindings after C++ changes
2. **gRPC Connection Failures**: Check service addresses and ports
3. **Type Mismatches**: Verify Protocol Buffer definitions match
4. **Performance Issues**: Profile language boundaries

### Debugging Tools

- **C++**: GDB, Valgrind, AddressSanitizer
- **Python**: pdb, cProfile, memory_profiler
- **Rust**: cargo test, cargo clippy, cargo fmt
- **TypeScript**: Chrome DevTools, React DevTools

---

## References

- `docs/CODEBASE_ARCHITECTURE.md` - Detailed C++ architecture
- `docs/PSEUDOCODE_IMPLEMENTATION_STRATEGY.md` - Code consistency strategy
- `agents/shared/API_CONTRACT.md` - API contracts
- `python/README.md` - Python integration guide
- `agents/backend/README.md` - Rust backend guide
- `web/README.md` - Web frontend guide

---

**Document Status**: ✅ Complete - Comprehensive multi-language architecture guide
