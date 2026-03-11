# Codex Instructions

See [AGENTS.md](AGENTS.md) for complete project guidelines. For multi-editor
setup and command parity across Codex, Cursor, Claude Code, OpenCode, skills,
and subagents, see [docs/AI_EDITOR_SETUP.md](docs/AI_EDITOR_SETUP.md). For
exarp-go session/task workflows used by the other editors, see
[docs/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md](docs/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md).

## Project at a Glance

Comprehensive multi-asset synthetic financing optimization platform. Manages
financing across options, futures, bonds, bank loans, and pension funds with
unified portfolio management, cash flow modeling, opportunity simulation, and
multi-instrument relationship optimization across 21+ accounts and multiple
brokers. Box spreads are one active strategy component rather than the whole
system.

Primary implementation areas:

- `native/` — C++20 core engine, broker adapters, pricing/risk logic, Catch2 tests
- `python/` — Python integration layer, TUI, bindings, tests
- `agents/` — Rust backend agents
- `web/` — browser client
- `docs/` — architecture, build, API, AI-editor setup

## Build, Test, Lint

Prefer CMake presets so build paths match `CMakePresets.json`:

```bash
cmake --preset macos-arm64-debug
cmake --build --preset macos-arm64-debug
ctest --preset macos-arm64-debug --output-on-failure
./scripts/run_linters.sh
```

Useful alternatives:

```bash
./scripts/shortcuts/run_build.sh build
./scripts/build_fast.sh
./scripts/build_universal.sh
```

If configure fails because vendored dependencies are missing, run:

```bash
./scripts/fetch_third_party.sh
```

## Key Files

| What | Where |
|------|-------|
| CLI entry point | `native/src/ib_box_spread.cpp` |
| TWS API wrapper | `native/src/tws_client.cpp` |
| Order lifecycle | `native/src/order_manager.cpp` |
| Risk calculations | `native/src/risk_calculator.cpp` |
| Greeks | `native/src/greeks_calculator.cpp` |
| Convexity | `native/src/convexity_calculator.cpp` |
| Config loading | `native/src/config_manager.cpp` |
| Market hours | `native/src/market_hours.cpp` |
| Architecture | `ARCHITECTURE.md` |
| AI/editor setup | `docs/AI_EDITOR_SETUP.md` |

## Code Style

- C++20, 2-space indentation, Allman braces, 100-char soft wrap
- Types: `PascalCase`
- Functions and variables: `snake_case`
- Constants: `k` prefix
- Add short comments only when trading math or scaling is not obvious

## Working Rules

- Never commit credentials, API keys, or broker secrets
- Always use paper trading port `7497` for testing
- Never modify `native/third_party/` directly; wrap vendor code in `native/src/`
- All trading, pricing, and risk logic changes need matching tests in `native/tests/`
- Prefer existing scripts, presets, and repository conventions over ad hoc commands
- Use imperative commit messages with 72-character subject lines

## Python

Always use `uv` for dependency management and command execution in this repo:

```bash
uv sync --project python --extra dev
uv run --project python pytest python/tests/ -v
```

Avoid direct `pip` / bare `pytest` unless `uv` is unavailable and you are fixing bootstrap issues.

## Codex-Specific Notes

- Codex should treat `AGENTS.md` as the canonical rule source and `CODEX.md`
  as the quick reference for this environment.
- When aligning AI/editor configs, prefer fixing shared docs over creating a
  separate Codex-only workflow.
- For task/session/report workflows, mirror the same exarp-go usage documented
  for Cursor, Claude Code, and OpenCode.
- For non-trivial implementation work, use this workflow by default:
  - create or update an exarp task before coding
  - compact the active context before starting long-running work
  - create 1-2 exarp follow-up tasks when finishing
  - add a result comment and update task status after verification
