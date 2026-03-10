# Repository Guidelines

## Project Overview

Comprehensive multi-asset synthetic financing optimization platform. Manages financing across options, futures, bonds, bank loans, and pension funds with unified portfolio management, cash flow modeling, opportunity simulation, and multi-instrument relationship optimization across 21+ accounts and multiple brokers (IBKR, Alpaca, Tradier, Tastytrade).

Box spreads are one active strategy component (7-10% of portfolio, spare cash allocation for T-bill-equivalent yields). The platform supports multiple strategy types including futures-implied financing, bond ETFs, and secured lending.

Multi-language codebase: C++ core engine, Python integration layer (TUI, bindings, NautilusTrader), Rust backend agents, Go agents (api-gateway, collectors), TypeScript/React web. See **`docs/MULTI_LANGUAGE_CODEBASE.md`** for a map of languages to directories and build/test/lint commands.

## Project Structure & Module Organization

```
ib_box_spread_full_universal/
├── native/                  # C++ core (the main codebase)
│   ├── src/                 # Implementation files (.cpp)
│   │   ├── brokers/         # Broker adapter implementations
│   │   └── strategies/      # Strategy implementations (box spread, etc.)
│   ├── include/             # Public headers (.h)
│   │   ├── brokers/         # Broker adapter interfaces
│   │   └── strategies/      # Strategy interfaces
│   ├── tests/               # Catch2 test suite
│   ├── third_party/         # Vendored dependencies (TWS API, Intel Decimal)
│   ├── ibapi_cmake/         # CMake glue for TWS API build
│   └── CMakeLists.txt       # Main build definition
├── python/                  # Python integration layer (TUI, bindings, tests)
├── agents/                  # Multi-language agents (Rust backend)
├── web/                     # React web application
├── proto/                   # Protocol Buffer definitions
├── config/                  # Configuration files (example configs only in repo)
├── scripts/                 # Helper scripts (build, lint, deploy)
├── docs/                    # Documentation
├── notebooks/               # Jupyter notebooks for analysis
└── build/                   # CMake build output (disposable, not committed)
```

### Key Source Files

| File | Purpose |
|------|---------|
| `native/src/ib_box_spread.cpp` | CLI entry point, renders XSP box-spread table |
| `native/src/tws_client.cpp` | Interactive Brokers TWS API client wrapper |
| `native/src/order_manager.cpp` | Order lifecycle management |
| `native/src/risk_calculator.cpp` | Position risk calculations |
| `native/src/greeks_calculator.cpp` | Options Greeks (delta, gamma, theta, vega) |
| `native/src/convexity_calculator.cpp` | Bond convexity calculations |
| `native/src/option_chain.cpp` | Option chain data structures and queries |
| `native/src/config_manager.cpp` | JSON configuration loading and validation |
| `native/src/loan_manager.cpp` | Synthetic loan position management |
| `native/src/market_hours.cpp` | Exchange trading hours logic |
| `native/src/rate_limiter.cpp` | API rate limiting |

## Build, Test & Development Commands

```bash
# Configure (one-time) — use a preset so build dir matches CMakePresets.json
cmake --preset macos-arm64-debug   # or macos-x86_64-debug on Intel, linux-x64-debug, etc.

# Build (Ninja uses CMAKE_BUILD_PARALLEL_LEVEL if set; scripts set it when unset)
cmake --build --preset macos-arm64-debug

# Or configure + build in one go (scripts set parallelism)
./scripts/shortcuts/run_build.sh build
./scripts/build_fast.sh

# Run CLI (path depends on preset: build/<preset>/bin/ib_box_spread)
./build/macos-arm64-debug/bin/ib_box_spread   # or build/macos-x86_64-debug on Intel

# Run tests
ctest --preset macos-arm64-debug --output-on-failure

# Build universal binary (macOS arm64+x86_64)
./scripts/build_universal.sh

# Lint
./scripts/run_linters.sh
```

If configure fails with missing **TWS API** or **Intel decimal** dependencies, run from repo root: `./scripts/fetch_third_party.sh`, then reconfigure. Or run `cmake --build <build-dir> --target fetch_third_party` then reconfigure. Set `CMAKE_BUILD_PARALLEL_LEVEL` (e.g. `$(nproc)` or `sysctl -n hw.ncpu`) for parallel builds when not using the wrapper scripts. See `docs/BUILD_PARALLELIZATION_AND_MODULARITY.md` for parallelization and modularity across C++, Rust, and lint.

**Optional (macOS):** To reduce disk reads for third-party trees, create a read-only compressed DMG and use it in builds: run `./scripts/third_party_dmg.sh create` after fetch, then set `USE_THIRD_PARTY_DMG=1` so build scripts mount the DMG automatically. See `docs/RAM_OPTIMIZATION_GUIDE.md` (§ Third-party on read-only compressed DMG).

### CMake Options

| Option | Default | Description |
|--------|---------|-------------|
| `BUILD_TESTING` | ON | Build Catch2 test suite |
| `ENABLE_NATIVE_CLI` | ON | Build the CLI binary |
| `ENABLE_PYTHON_BINDINGS` | ON | Build Cython Python bindings |
| `ENABLE_ASAN` | OFF | AddressSanitizer |
| `ENABLE_TSAN` | OFF | ThreadSanitizer |
| `ENABLE_LTO` | ON | Link-Time Optimization |
| `ENABLE_NATS` | OFF | NATS message queue |

## Coding Style & Naming Conventions

Target ISO C++20. Prefer two-space indentation, Allman braces for multi-line scopes, and 100-character soft wraps.

| Element | Convention | Example |
|---------|-----------|---------|
| Types | `PascalCase` | `Scenario`, `OrderManager` |
| Functions | `snake_case` | `make_scenario`, `calculate_profit` |
| Variables | `snake_case` | `strike_price`, `expiry_date` |
| Constants | `k` prefix | `kMaxPositions`, `kDefaultPort` |

Add short `//` comments only where the trading math is non-obvious (e.g., APR scaling by the contract multiplier).

## Dependencies

| Dependency | Location | Purpose |
|------------|----------|---------|
| TWS API | `native/third_party/tws-api/` | IBKR connectivity |
| Intel Decimal | `native/third_party/IntelRDFPMathLib20U4/` | Exact decimal arithmetic |
| nlohmann/json | FetchContent (v3.11.3) | JSON parsing |
| spdlog | FetchContent (v1.13.0) | Logging |
| CLI11 | FetchContent (v2.4.1) | CLI argument parsing |
| Catch2 | FetchContent (v3.5.2) | Unit testing |
| Eigen3 | FetchContent (v3.4.0) | Linear algebra |
| QuantLib | FetchContent (v1.36) | Quantitative finance |
| NLopt | FetchContent (v2.9.1) | Optimization |
| Boost | System (Homebrew) | Date/time, filesystem |

## IB API Integration Notes

The TWS API is vendored under `native/third_party/tws-api/`. The `native/ibapi_cmake/` directory contains CMake presets to build `libtwsapi.dylib` and the Intel decimal math dependency. Never commit IB credentials, logs, or downloaded vendor artifacts — treat everything under `build/` as ephemeral. The CLI currently prints synthetic market data; gate any future live requests behind configuration flags.

## Testing Guidelines

Tests live in `native/tests/` and use the Catch2 framework. They mirror source file names (e.g., `test_risk_calculator.cpp` tests `risk_calculator.cpp`). Expand coverage alongside new features. Run `ctest --test-dir build --output-on-failure` locally before pushes.

## Commit & Pull Request Guidelines

Follow imperative, 72-character subject lines ("Add TSV formatter for CLI"). In the body, summarize option scenarios touched, list the commands run (build, tests, sample CLI output), and note IB API version bumps. PRs must call out configuration changes (e.g., new env vars or IB gateway ports).

## Security

- Never commit credentials, API keys, or secrets
- Always use paper trading port (7497) for testing
- Gate live trading behind explicit configuration flags
- Never modify third-party code directly — use wrappers in `native/src/`

## AI Configuration Files

| File | AI Tool |
|------|---------|
| `AGENTS.md` | All AI agents (canonical source) |
| `CLAUDE.md` | Claude Code |
| `CODEX.md` | OpenAI Codex |
| `opencode.json` | OpenCode (config, MCP) |
| `.opencode/commands/` | OpenCode (custom commands) |
| `.cursorrules` | Cursor IDE |
| `.cursor/rules/*.mdc` | Cursor IDE (glob-based rules) |
| `.cursor/commands.json` | Cursor (slash commands) |
| `.cursor/mcp.json` | Cursor (MCP servers) |
| `.windsurfrules` | Windsurf IDE |
| `.clinerules` | Cline |
| `.github/copilot-instructions.md` | GitHub Copilot |
| `.claude/settings.json` | Claude Code permissions |
| `.claude/agents/` | Custom Claude agents |

**Skills & subagents:** Cursor/plugin skills and subagents (e.g. mcp_task,
exarp-go, Claude agents) should use AGENTS.md as canonical context. `CLAUDE.md`
and `CODEX.md` are tool-specific quick references. See
[docs/AI_EDITOR_SETUP.md](docs/AI_EDITOR_SETUP.md) for setup and command parity
across Codex, OpenCode, Claude, Cursor, skills, and subagents.
