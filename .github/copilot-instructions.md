# GitHub Copilot Instructions

This repository follows the comprehensive guidelines in [AGENTS.md](../AGENTS.md).

## Project

Comprehensive multi-asset synthetic financing platform. Manages financing across options, futures, bonds, loans, and pension funds with unified portfolio management across 21+ accounts and multiple brokers. Box spreads are one active strategy (spare cash, T-bill-equivalent yields). Multi-language: C++ core, pybind11-backed Python binding tests, Rust backend. Active runtime surfaces are the native CLI and Rust TUI; the web client is archived.

## Structure

| Directory | Contents |
|-----------|----------|
| `native/src/` | C++ source files |
| `native/include/` | C++ headers |
| `native/tests/` | Catch2 tests (mirror source names with `test_` prefix) |
| `native/CMakeLists.txt` | Build definition |
| `native/tests/python/` | Python binding tests for the native module |
| `agents/` | Rust backend crates and services |
| `config/` | Config templates |
| `docs/` | Documentation |

## Code Style

- **C++20**, 2-space indentation, Allman braces, 100-char lines
- Types: `PascalCase` (`Scenario`, `OrderManager`)
- Functions/variables: `snake_case` (`make_scenario`, `strike_price`)
- Constants: `k` prefix (`kMaxPositions`, `kDefaultPort`)
- Comment only non-obvious trading math

## Build & Test

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
ninja -C build
ctest --test-dir build --output-on-failure
./scripts/run_linters.sh
```

## Key Files

| File | Purpose |
|------|---------|
| `native/src/ib_box_spread.cpp` | CLI entry point |
| `native/src/risk_calculator.cpp` | Risk calculations |
| `native/src/greeks_calculator.cpp` | Options Greeks |
| `native/src/order_manager.cpp` | Order lifecycle |
| `native/src/tws_client.cpp` | TWS API wrapper |
| `native/include/types.h` | Core type definitions |
| `ARCHITECTURE.md` | System architecture |

## Dependencies

FetchContent: nlohmann/json (3.11.3), spdlog (1.13.0), CLI11 (2.4.1), Catch2 (3.5.2), Eigen3 (3.4.0), QuantLib (1.36), NLopt (2.9.1).
Vendored: TWS API, Intel Decimal. System: Boost (Homebrew).

## Safety Rules

- Never commit credentials or API keys
- Paper trading port 7497 only
- Never modify `native/third_party/` — use wrappers
- All trading/risk calculations must have Catch2 tests
- Imperative commit messages, 72-char subject lines

## Python

Prefer `uv` for package management. Fallback: `pip`.

For complete guidelines, see [AGENTS.md](../AGENTS.md).
