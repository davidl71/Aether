# Multi-Programming-Language Codebase

This repository uses **multiple programming languages**. This doc maps each language to directories, build/test/lint commands, and cross-language boundaries.

## Language map

| Language | Directories | Build | Test | Lint |
|----------|-------------|--------|------|------|
| **C++** | `native/` (core engine, CLI, tests in `native/tests/`) | `cmake --build build` or presets; `./scripts/build_universal.sh` for macOS universal | `ctest --test-dir build --output-on-failure` | `./scripts/run_linters.sh` (cppcheck, clang-tidy, etc.) |
| **Python** | `python/` (integration, services, tests) | — (interpreted); Cython bindings built via CMake | `uv run --project python pytest python/tests/` | `uv run --project python ruff check python`; `just lint-shell` for scripts |
| **Rust** | `agents/backend/` (crates: api, ledger, market_data, nats_adapter, risk, strategy, discount_bank_parser) | `cargo build` in `agents/backend/` | `cargo test` in `agents/backend/` | `cargo clippy` |
| **TypeScript / React** | `web/` (archived Vite/React client) | Historical only | Historical only | Historical only |

## Shared and generated code

- **Protocol Buffers** (`proto/`): `proto/messages.proto` → C++ (generated at CMake build), Python/TypeScript via `./proto/generate.sh`. See `docs/message_schemas/README.md`.
- **Config**: JSON under `config/`; shared by the Rust TUI, CLI, and backend services.

## Cross-language boundaries

- **NATS**: C++ publishes market and strategy events; Rust backend consumes them; Rust TUI uses REST polling as fallback when NATS unavailable. See `docs/platform/DATAFLOW_ARCHITECTURE.md`.
- **REST / WebSocket**: the active client path is Rust TUI/CLI to the Rust backend, plus selected Python specialist services where still required.
- **Ledger**: Rust `agents/backend/crates/ledger` is the durable ledger owner; Python direct SQLite access is legacy and should move behind service/API boundaries. See P1-A in `docs/platform/IMPROVEMENT_PLAN.md`.

## Quick reference

- **Canonical project guidelines:** `AGENTS.md`, `CLAUDE.md`.
- **Build/lint/test shortcuts:** `.cursor/rules/just-cmake-shortcuts.mdc`, `docs/CURSOR_PROJECT_COMMANDS.md` (if present).
- **API and external docs:** `docs/API_DOCUMENTATION_INDEX.md`.
