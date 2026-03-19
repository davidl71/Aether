# Claude Code Instructions

See [AGENTS.md](AGENTS.md) for complete project guidelines. For multi-editor setup
(OpenCode, Cursor, skills, subagents), see [docs/AI_EDITOR_SETUP.md](docs/AI_EDITOR_SETUP.md).
For exarp-go (session prime, handoff, tasks, scorecard) in Cursor, Claude Code, and OpenCode, see [docs/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md](docs/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md).

## Project at a Glance

Comprehensive multi-asset synthetic financing platform. Manages financing across options, futures, bonds, loans, and pension funds with unified portfolio management, cash flow modeling, and multi-instrument optimization across 21+ accounts and multiple brokers. Box spreads are one active strategy component (T-bill-equivalent yields on spare cash). Multi-language codebase: C++ core engine, pybind11-backed Python binding tests under `native/tests/python/`, Rust backend agents, and an archived TypeScript/React web client. Active frontend focus is the Rust TUI plus the native CLI.

## Build & Test

Prefer `just` commands (see `just --list`):

```bash
just configure          # one-time CMake setup (debug)
just build              # ninja -C build
just test               # ctest --test-dir build --output-on-failure
just lint               # run all linters
just build-universal    # universal binary (arm64 + x86_64)
just build-portable     # auto-detect platform and preset
```

**AI-friendly builds** (JSON output for easy parsing):

```bash
# C++ (AI-friendly: quiet, JSON result, error extraction)
./scripts/build_ai_friendly.sh --json-only

# Rust
./scripts/build_rust_ai_friendly.sh --json-only
```

Direct CMake/Ninja (when `just` isn't available):

```bash
# Configure (one-time)
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
# Or use a preset (matches CMakePresets.json)
cmake --preset macos-arm64-debug   # linux-x64-debug, macos-x86_64-debug

# Build
ninja -C build

# Test
ctest --test-dir build --output-on-failure
ctest --test-dir build -R test_name --output-on-failure   # single test

# Lint
./scripts/run_linters.sh
```

If configure fails with missing TWS API or Intel Decimal deps, run `./scripts/fetch_third_party.sh` first.

### CMake Options

| Option | Default | Description |
|--------|---------|-------------|
| `BUILD_TESTING` | ON | Build Catch2 test suite |
| `ENABLE_NATIVE_CLI` | ON | Build the CLI binary |
| `ENABLE_PYTHON_BINDINGS` | ON | Build pybind11 Python bindings |
| `ENABLE_ASAN` | OFF | AddressSanitizer |
| `ENABLE_TSAN` | OFF | ThreadSanitizer |
| `ENABLE_LTO` | ON | Link-Time Optimization |
| `ENABLE_NATS` | OFF | NATS message queue |

## Code Style

- C++20, 2-space indentation, Allman braces, 100-char soft wrap
- `snake_case` functions/variables, `PascalCase` types, `k` prefix constants
- Only comment non-obvious trading math

## Key Files to Know

| What | Where |
|------|-------|
| CLI entry point | `native/src/ib_box_spread.cpp` |
| Build config | `native/CMakeLists.txt` |
| Type definitions | `native/include/types.h` |
| TWS API wrapper | `native/src/tws_client.cpp` |
| Order management | `native/src/order_manager.cpp` |
| Risk calculations | `native/src/risk_calculator.cpp` |
| Greeks | `native/src/greeks_calculator.cpp` |
| Margin calculator | `native/src/margin_calculator.cpp` |
| Financing optimizer | `native/src/financing_optimizer.cpp` |
| Hedge manager | `native/src/hedge_manager.cpp` |
| Config loading | `native/src/config_manager.cpp` |
| Broker adapters | `native/src/brokers/` |
| Strategy impls | `native/src/strategies/` |
| Architecture | `ARCHITECTURE.md` |
| API docs index | `docs/API_DOCUMENTATION_INDEX.md` |

## Common Tasks

### Adding a new source file

1. Create `native/src/foo.cpp` and `native/include/foo.h`
2. Add to the `SOURCES` / `HEADERS` lists in `native/CMakeLists.txt`
3. Create `native/tests/test_foo.cpp` and register in `native/tests/CMakeLists.txt`

### Adding a dependency

Most deps use CMake FetchContent — see existing patterns in `native/CMakeLists.txt`.

## Safety Rules

- **Never** commit credentials, API keys, or secrets
- **Always** use paper trading port `7497` for testing
- **Never** modify code under `native/third_party/` (legacy C++ deps); wrap vendor code in `agents/backend/crates/ib_adapter/src/`
- **Always** add tests for trading logic and risk calculations
- All pricing/risk calculations must have Rust `#[test]` tests before merge
- Gate live trading behind explicit configuration flags

## Python

Use `uv` for package management when available. Fall back to `pip` if not.

```bash
cd native
uv run --project . pytest tests/python/ -v
```

## exarp Workflow

For any non-trivial implementation or refactor spanning multiple steps, files, or commits:

1. **Track in exarp before coding** — update an existing task or create one first
2. **Compact context before long work** — summarize goal, constraints, worktree state, prior decisions
3. **Create follow-up tasks when finishing** — 1-2 tasks for verification, cleanup, docs, or deferred risk
4. **Update task status at completion** — add a result comment with verification commands

Skip for trivial one-shot edits, status checks, or isolated fixes with no realistic follow-up.
