# Trade-Frame Repository Learnings

## Overview

This document captures key learnings from the [trade-frame repository](https://github.com/rburkholder/trade-frame), a C++17-based trading library framework developed by Raymond P. Burkholder. Trade-frame provides a comprehensive foundation for testing automated trading strategies across equities, futures, currencies, ETFs, and options.

**Repository**: <https://github.com/rburkholder/trade-frame>
**Language**: C++17
**License**: See repository
**Stars**: 611+ (as of 2025)

## Key Architectural Patterns

### 1. Modular Library Organization

Trade-frame organizes code into distinct, reusable libraries:

#### Core Libraries (`lib/` directory)

- **TFTimeSeries**: Manages trades, quotes, greeks, and Level II order book data
- **TFSimulation**: Simulation engine for backtesting
- **TFIQFeed**: Engine for DTN IQFeed Level 1 & Level 2 data integration
- **TFInteractiveBrokers**: Engine for IB TWS API integration
- **TFIndicators**: Technical indicators library
- **TFHDF5TimeSeries**: Wraps HDF5 library for storing time series data
- **TFOptions**: Options calculations and pricing
- **TFTrading**: Manages orders, executions, portfolios, positions, and accounts
- **TFVuTrading**: UI elements, forms, and panels (wxWidgets-based)
- **OUCharting**: Wrapper around ChartDirector for plots and charts
- **OUSQL**: ORM wrapper around SQLite for maintaining trading records

**Key Insight**: Clear separation of concerns with each library having a single, well-defined responsibility. This makes the codebase maintainable and allows applications to link only the libraries they need.

### 2. Application Layer Pattern

Trade-frame separates library code from application code:

- **Libraries** (`lib/`): Reusable, framework-level code
- **Applications** (top-level directories): Sample applications demonstrating library usage
- **Examples**:
  - `AutoTrade`: Template for automated trading with ML-based work
  - `BarChart`: Tag instruments by interest and review last 200 daily bars
  - `Collector`: Stream real-time bid/ask/tick data to disk for backtesting
  - `ComboTrading`: Basics of trading multiple securities (options strategies)
  - `DepthOfMarket`: Level II ladder for trading futures
  - `LiveChart`: Real-time instrument viewing

**Key Insight**: Applications serve as both working examples and templates for new trading strategies. This pattern helps developers understand how to use the libraries effectively.

### 3. Multi-Provider Architecture

Trade-frame supports multiple market data providers and execution venues:

**Market Data Providers**:

- **IQFeed**: Real-time and historical data (primary)
- **Interactive Brokers**: Real-time market data via TWS API
- **Alpaca**: Real-time data and order execution
- **Phemex**: Real-time data and order execution (work in progress)

**Execution Venues**:

- **Interactive Brokers**: Via TWS API
- **Alpaca**: Via Alpaca API
- **Phemex**: Via Phemex API

**Key Insight**: Abstracting provider-specific code behind common interfaces allows the same trading logic to work with multiple data sources and brokers. This is critical for:

- Redundancy (fallback providers)
- Testing (paper trading vs. live)
- Multi-broker strategies

### 4. Data Storage Strategy

Trade-frame uses multiple storage backends:

- **HDF5**: For high-performance time series storage (via `TFHDF5TimeSeries`)
- **SQLite**: For trading records and metadata (via `OUSQL` ORM)
- **Excel**: For export and reporting (via included `exelformat`)

**Key Insight**: Different data types benefit from different storage formats:

- HDF5: High-frequency tick data, large time series
- SQLite: Relational data (trades, positions, account info)
- Excel: Human-readable reports and analysis

## Build System Patterns

### CMake Organization

Trade-frame uses CMake with a modular structure:

```cmake

# Main CMakeLists.txt includes subdirectories

add_subdirectory(lib)
add_subdirectory(ApplicationName)
```

**Key Patterns**:

1. Each library has its own `CMakeLists.txt`
2. Applications link against required libraries
3. Dependencies are managed at the library level
4. Build artifacts go to disposable directories (`build/`, etc.)

### Dependency Management

Trade-frame uses a separate repository (`libs-build`) for building dependencies:

**Dependencies**:

- wxWidgets (UI framework)
- boost (C++ utilities)
- curl (HTTP client)
- zlib (compression)
- hdf5 (data storage)
- sqlite (embedded database)
- exelformat (Excel I/O)
- rdaf/ROOT (CERN toolset, optional)
- libtorch (PyTorch C++ API, optional)

**Key Insight**: Separating dependency builds from main project builds:

- Simplifies main project CMakeLists.txt
- Allows dependency versioning independent of main project
- Makes it easier to update dependencies

## Development Workflow

### IDE Setup

Trade-frame is optimized for Visual Studio Code with:

- **C/C++ Extension** (Microsoft)
- **clangd Extension** (LLVM)
- **CMake Extension** (twxs)
- **CMake Tools** (Microsoft)

**Key Insight**: Modern C++ development benefits from:

- Language server (clangd) for symbol lookup and cross-referencing
- CMake integration for build management
- Consistent tooling across team members

### Testing Strategy

Trade-frame includes:

- **IQFeed Testing**: Uses `TST$Y` symbol for 24/7 looped test data
- **Paper Trading**: Always use IB paper trading account (port 7497) for testing
- **Simulation**: `TFSimulation` library for backtesting

**Key Insight**: Multiple testing layers:

1. Unit tests (library-level)
2. Integration tests (with mock data)
3. Paper trading (with real broker API)
4. Simulation/backtesting (historical data)

## Key Best Practices from Trade-Frame

### 1. Thread Safety for TWS API

Trade-frame demonstrates proper threading for IB TWS API:

- Dedicated `EReader` thread for receiving messages
- Mutex protection for shared data structures
- Callback-based architecture (EWrapper pattern)

**Relevance to Our Project**: Our `TWSClient` should follow similar patterns:

- Separate thread for EReader
- Thread-safe data structures for market data
- Proper synchronization for order state

### 2. Real-Time Data Streaming

Trade-frame's `Collector` application shows how to:

- Stream real-time tick data to disk
- Use HDF5 for efficient storage
- Enable backtesting with real market data

**Relevance to Our Project**: We could implement similar data collection for:

- Historical analysis of box spread opportunities
- Training ML models (if we add ML features)
- Performance analysis

### 3. Multi-Leg Order Management

Trade-frame's `ComboTrading` demonstrates:

- Managing complex multi-leg orders
- Synchronizing order placement
- Handling partial fills

**Relevance to Our Project**: Box spreads are 4-leg orders, so we should:

- Use IBKR Combo Orders for atomic execution
- Implement rollback logic for failed legs
- Track order state across all legs

### 4. Configuration Management

Trade-frame uses configuration files for:

- Market data provider settings
- Broker connection parameters
- Strategy parameters

**Relevance to Our Project**: Our `ConfigManager` should:

- Support multiple configuration sources (file, env vars, CLI)
- Validate configuration before use
- Provide sensible defaults

## Library Integration Patterns

### 1. Time Series Management

**TFTimeSeries** provides:

- Efficient storage of tick data
- Query interface for historical data
- Integration with indicators

**Pattern**: Use specialized data structures for time series rather than generic containers.

### 2. Options Calculations

**TFOptions** handles:

- Greeks calculations
- Option pricing models
- Volatility surfaces

**Pattern**: Separate options math into dedicated library for reuse across applications.

### 3. Trading Record Management

**OUSQL** (ORM wrapper) provides:

- Type-safe database access
- Schema management
- Query building

**Pattern**: Use ORM for trading records to:

- Reduce boilerplate
- Ensure type safety
- Simplify schema changes

## Application Examples

### AutoTrade

Template for automated trading with:

- Strategy framework
- ML integration points (libtorch)
- Risk management hooks

**Key Features**:

- Event-driven architecture
- Strategy state management
- Performance tracking

### Collector

Real-time data collection:

- Streams tick data to HDF5
- Supports multiple symbols
- Enables backtesting with real data

**Key Features**:

- High-performance I/O
- Configurable data retention
- Time-series storage

### ComboTrading

Multi-leg order management:

- Options strategies
- Synchronized execution
- Position tracking

**Key Features**:

- Combo order support
- Leg synchronization
- Risk checks

## Comparison with Our Project

### Similarities

1. **C++17 Standard**: Both use modern C++
2. **IB TWS API Integration**: Both integrate with Interactive Brokers
3. **Options Trading Focus**: Both handle options strategies
4. **CMake Build System**: Both use CMake
5. **Modular Architecture**: Both separate libraries from applications

### Differences

1. **Scope**: Trade-frame is a general-purpose framework; our project focuses on box spreads
2. **UI Framework**: Trade-frame uses wxWidgets; we use CLI/TUI
3. **Data Storage**: Trade-frame uses HDF5; we use QuestDB
4. **Language Bindings**: Trade-frame is C++ only; we have Python bindings
5. **Testing**: Trade-frame has extensive sample apps; we focus on unit tests

### What We Can Learn

1. **Library Organization**: Clear separation of concerns
2. **Multi-Provider Support**: Abstract interfaces for multiple brokers
3. **Data Collection**: Real-time data streaming for analysis
4. **Application Templates**: Sample code as learning resources
5. **Build System**: Modular CMake structure

## Recommendations for Our Project

### 1. Library Structure

Consider organizing our code into libraries:

- `libBoxSpread`: Core box spread calculations
- `libTWS`: TWS API wrapper (already exists as `TWSClient`)
- `libRisk`: Risk calculations
- `libData`: Data storage and retrieval

### 2. Sample Applications

Create sample applications demonstrating:

- Basic box spread scanning
- Automated execution
- Performance analysis
- Risk monitoring

### 3. Data Collection

Implement data collection similar to trade-frame's `Collector`:

- Stream real-time option chain data
- Store in QuestDB for historical analysis
- Enable backtesting and strategy refinement

### 4. Multi-Provider Support

Consider abstracting broker interfaces:

- Common interface for market data
- Common interface for order execution
- Provider-specific implementations

### 5. Testing Infrastructure

Adopt trade-frame's testing approach:

- Unit tests for core libraries
- Integration tests with mock data
- Paper trading tests with real API
- Simulation/backtesting framework

## Resources

- **Repository**: <https://github.com/rburkholder/trade-frame>
- **Blog**: <http://blog.raymond.burkholder.net/index.php?/categories/23-Trading>
- **Dependencies**: <https://github.com/rburkholder/libs-build>
- **Documentation**: See repository README.md

## Related Documentation

- `docs/CODEBASE_ARCHITECTURE.md`: Our project architecture
- `docs/TWS_INTEGRATION_STATUS.md`: TWS API integration details
- `docs/COMMON_PATTERNS.md`: Coding patterns and conventions
- `docs/IBKRBOX_LEARNINGS.md`: Learnings from ibkrbox project
- `docs/ICLI_LEARNINGS.md`: Learnings from icli project

## Summary

Trade-frame demonstrates several best practices for C++ trading applications:

1. **Modular Design**: Clear library boundaries and responsibilities
2. **Multi-Provider Support**: Abstract interfaces for flexibility
3. **Comprehensive Testing**: Multiple testing layers
4. **Real-Time Data**: Efficient streaming and storage
5. **Application Templates**: Reusable code examples

These patterns can inform our project's evolution, particularly as we add more features and support additional data sources or brokers.
