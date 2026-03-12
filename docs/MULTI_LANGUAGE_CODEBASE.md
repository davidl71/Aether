# Multi-Programming-Language Codebase

This repository uses **multiple programming languages**. This doc maps each language to directories, build/test/lint commands, and cross-language boundaries.

## Language map

| Language | Directories | Build | Test | Lint |
|----------|-------------|--------|------|------|
| **C++** | `native/` (core engine, CLI, tests in `native/tests/`) | `cmake --build build` or presets; `./scripts/build_universal.sh` for macOS universal | `ctest --test-dir build --output-on-failure` | `./scripts/run_linters.sh` (cppcheck, clang-tidy, etc.) |
| **Python** | `native/tests/python/` (binding tests), `scripts/` (utilities) | — (interpreted); pybind11 bindings built via CMake | `cd native && uv run --project . pytest tests/python/` | `uv run ruff check scripts/` |
| **Rust** | `agents/backend/` (crates: api, ledger, market_data, nats_adapter, risk, strategy, discount_bank_parser) | `cargo build` in `agents/backend/` | `cargo test` in `agents/backend/` | `cargo clippy` |
| **TypeScript / React** | `web/` (archived, not active runtime) | Historical only | Historical only | Historical only |

## Shared and generated code

- **Protocol Buffers** (`proto/`): `proto/messages.proto` → C++ (generated at CMake build). See `docs/message_schemas/README.md`.
- **Config**: JSON under `config/`; shared by the Rust TUI, CLI, and backend services.

## Cross-language boundaries

- **NATS**: C++ publishes market and strategy events; Rust backend consumes them; Rust TUI uses REST polling as fallback when NATS unavailable. See `docs/platform/DATAFLOW_ARCHITECTURE.md`.
- **REST / WebSocket**: The active client path is Rust TUI/CLI to the Rust backend.
- **Ledger**: Rust `agents/backend/crates/ledger` is the durable ledger owner.

## Quick reference

- **Canonical project guidelines:** `AGENTS.md`, `CLAUDE.md`.
- **Build/lint/test shortcuts:** `.cursor/rules/just-cmake-shortcuts.mdc`, `docs/CURSOR_PROJECT_COMMANDS.md` (if present).
- **API and external docs:** `docs/API_DOCUMENTATION_INDEX.md`.
