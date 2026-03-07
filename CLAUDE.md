# Claude Code Instructions

See [AGENTS.md](AGENTS.md) for complete project guidelines. For multi-editor setup
(OpenCode, Cursor, skills, subagents), see [docs/AI_EDITOR_SETUP.md](docs/AI_EDITOR_SETUP.md).
For exarp-go (session prime, handoff, tasks, scorecard) in Cursor, Claude Code, and OpenCode, see [docs/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md](docs/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md).

## Project at a Glance

Comprehensive multi-asset synthetic financing platform. Manages financing across options, futures, bonds, loans, and pension funds with unified portfolio management across 21+ accounts and multiple brokers. Box spreads are one active strategy (spare cash, T-bill-equivalent yields). C++ core in `native/src/` and `native/include/`, tests in `native/tests/` (Catch2), Python layer in `python/`.

## Build & Test

```bash
ninja -C build                                    # build
ctest --test-dir build --output-on-failure         # test
./scripts/run_linters.sh                           # lint
./scripts/build_universal.sh                       # universal binary
```

If the build directory doesn't exist yet:

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
```

## Code Style

- C++20, 2-space indentation, Allman braces, 100-char soft wrap
- `snake_case` functions/variables, `PascalCase` types, `k` prefix constants
- Only comment non-obvious trading math

## Key Files to Know

| What | Where |
|------|-------|
| CLI entry point | `native/src/ib_box_spread.cpp` |
| Build config | `native/CMakeLists.txt` |
| Risk calculations | `native/src/risk_calculator.cpp` |
| Greeks | `native/src/greeks_calculator.cpp` |
| Order management | `native/src/order_manager.cpp` |
| TWS API wrapper | `native/src/tws_client.cpp` |
| Config loading | `native/src/config_manager.cpp` |
| Type definitions | `native/include/types.h` |
| Architecture | `ARCHITECTURE.md` |
| API docs index | `docs/API_DOCUMENTATION_INDEX.md` |

## Common Tasks

### Adding a new source file

1. Create `native/src/foo.cpp` and `native/include/foo.h`
2. Add to the `SOURCES` / `HEADERS` lists in `native/CMakeLists.txt`
3. Create `native/tests/test_foo.cpp` and register in `native/tests/CMakeLists.txt`

### Adding a dependency

Most deps use CMake FetchContent — see existing patterns in `native/CMakeLists.txt`.

### Running a single test

```bash
ctest --test-dir build -R test_name --output-on-failure
```

## Safety Rules

- **Never** commit credentials or API keys
- **Always** use paper trading port `7497` for testing
- **Never** modify code under `native/third_party/` — write wrappers instead
- **Always** add tests for trading logic and risk calculations
- All pricing/risk calculations must have Catch2 tests before merge

## Python

Use `uv` for package management when available. Fall back to `pip` if not.

```bash
uv sync                          # install deps
uv run pytest python/tests/ -v   # run tests
```
