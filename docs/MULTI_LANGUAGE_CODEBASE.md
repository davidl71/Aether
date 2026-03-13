# Multi-Programming-Language Codebase

This repository uses **multiple programming languages**. This doc maps each language to directories, build/test/lint commands, and cross-language boundaries.

## Language map

| Language | Directories | Build | Test | Lint |
|----------|-------------|--------|------|------|
| **C++** | `native/` (core engine, CLI, tests in `native/tests/`) | `cmake --build build` or presets; `./scripts/build_universal.sh` for macOS universal | `ctest --test-dir build --output-on-failure` | `./scripts/run_linters.sh` (cppcheck, clang-tidy, etc.) |
| **Rust** | `agents/backend/` (crates: api, ledger, market_data, nats_adapter, risk, strategy, discount_bank_parser) | `cargo build` in `agents/backend/` | `cargo test` in `agents/backend/` | `cargo clippy` |
| **Python** | `scripts/` (utilities only) | — (interpreted) | — | `uv run ruff check scripts/` |
| **TypeScript / React** | `web/` (archived, not active runtime) | Historical only | Historical only | Historical only |

## C++ Module Inventory

The C++ codebase (`native/`) owns these functional areas:

### Core Data Types
| Module | Files | Purpose |
|--------|-------|---------|
| **Types** | `types.h`, `types.cpp` | Core structs: `OptionContract`, `BoxSpreadLeg`, `MarketData`, `Order`, `Position`, `AccountValue` |
| **Enums** | `types.h` | `OptionType`, `OrderAction`, `OrderStatus`, `TimeInForce`, `OptionStyle` |

### Trading Strategy
| Module | Files | Purpose |
|--------|-------|---------|
| **Box Spread** | `strategies/box_spread/` | `BoxSpreadCalculator`: profit, ROI, implied rate calculations; `BoxSpreadValidator`; `BoxSpreadBag` multi-leg |
| **Option Chain** | `option_chain.h/cpp` | `OptionChainScanner`, `OptionChainEntry` for scanning IV, liquidity |
| **Hedge Manager** | `hedge_manager.h/cpp` | Delta/gamma hedging position management |

### Risk & Greeks
| Module | Files | Purpose |
|--------|-------|---------|
| **Risk Calculator** | `risk_calculator.h/cpp`, `risk_calculator_*.cpp` | VaR, position sizing, correlation, stress testing |
| **Greeks Calculator** | `greeks_calculator.h/cpp` | Delta, gamma, theta, vega, rho calculations (QuantLib) |
| **Margin Calculator** | `margin_calculator.h/cpp` | RegT, portfolio margin calculations |
| **Collateral Valuator** | `collateral_valuator.h/cpp` | Bond/loan collateral valuation |

### IBKR Integration
| Module | Files | Purpose |
|--------|-------|---------|
| **TWS Client** | `tws_client.h/cpp`, `tws_client_impl.h/cpp` | EWrapper callbacks, connection management, market data (BROKEN) |
| **TWS Connection** | `tws_connection.h/cpp` | Socket connection, reader thread |
| **TWS Market Data** | `tws_market_data.h/cpp` | `reqMarketData`, tick handling |
| **TWS Orders** | `tws_orders.h/cpp` | Order placement, combo orders |
| **TWS Positions** | `tws_positions.h/cpp` | Position queries, updates |
| **TWS Contracts** | `tws_contracts.h/cpp` | Contract details, scanning |
| **TWS Conversions** | `tws_conversions.h/cpp` | `calculate_dte`, date/expiry helpers |

### Order Management
| Module | Files | Purpose |
|--------|-------|---------|
| **Order Manager** | `order_manager.h/cpp` | Order lifecycle, state machine, fills |

### Infrastructure
| Module | Files | Purpose |
|--------|-------|---------|
| **Config Manager** | `config_manager.h/cpp` | JSON config loading/validation |
| **Rate Limiter** | `rate_limiter.h/cpp` | IB API message rate limiting |
| **Market Hours** | `market_hours.h/cpp` | Exchange hours, trading days |
| **NATS Client** | `nats_client.h/cpp` | Publish market data to NATS (optional) |
| **Cache Client** | `cache_client.h/cpp` | In-memory cache with TTL |
| **PCAP Capture** | `pcap_capture.h/cpp` | Network packet capture (debug) |
| **Proto Adapter** | `proto_adapter.h/cpp` | Protobuf ↔ C++ type conversion |

### Calculations
| Module | Files | Purpose |
|--------|-------|---------|
| **Financing Optimizer** | `financing_optimizer.h/cpp` | Portfolio-level financing optimization |
| **Financing Instrument Registry** | `financing_instrument_registry.h/cpp` | Bonds, loans, pension fund instruments |
| **Convexity Calculator** | `convexity_calculator.h/cpp` | Bond convexity adjustments |
| **Asset Relationship** | `asset_relationship.h/cpp` | Multi-asset correlation/relationship graph |
| **Path Validator** | `path_validator.h/cpp` | File path validation, existence checks |

### CLI (Currently Disabled)
| Module | Files | Purpose |
|--------|-------|---------|
| **IB Box Spread CLI** | `ib_box_spread.cpp` | Main CLI entry point (TUI/table rendering) |

## Build Status

| Component | Status | Notes |
|-----------|--------|-------|
| C++ CLI | **Disabled** | TWS API vtable issues; see T-1773409358177120000 |
| C++ Tests | **Disabled** | Depends on CLI fix |
| Python Bindings | **Removed** | No active consumers |
| NATS Publishing | **Works** | Optional, compiles when `ENABLE_NATS=ON` |
| QuantLib/Eigen | **Works** | Third-party math libs |

## Shared and generated code

- **Protocol Buffers** (`proto/`): `proto/messages.proto` → C++ (generated at CMake build). See `docs/message_schemas/README.md`.
- **Config**: JSON under `config/`; shared by the Rust TUI, CLI, and backend services.

## Cross-language boundaries

- **NATS**: C++ publishes market and strategy events; Rust backend consumes them; Rust TUI uses REST polling as fallback when NATS unavailable. See `docs/platform/DATAFLOW_ARCHITECTURE.md`.
- **REST / WebSocket**: The active client path is Rust TUI/CLI to the Rust backend.
- **Ledger**: Rust `agents/backend/crates/ledger` is the durable ledger owner.

## Rust Quant Finance

See `docs/RUST_FINANCE_LIBRARIES.md` for libraries that could replace C++ QuantLib code.

## Quick reference

- **Canonical project guidelines:** `AGENTS.md`, `CLAUDE.md`.
- **Build/lint/test shortcuts:** `.cursor/rules/just-cmake-shortcuts.mdc`, `docs/CURSOR_PROJECT_COMMANDS.md` (if present).
- **API and external docs:** `docs/API_DOCUMENTATION_INDEX.md`.
