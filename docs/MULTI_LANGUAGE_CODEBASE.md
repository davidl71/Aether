# Multi-Programming-Language Codebase

This repository uses **multiple programming languages**. This doc maps each language to directories, build/test/lint commands, and cross-language boundaries.

## Language map

| Language | Directories | Build | Test | Lint |
|----------|-------------|--------|------|------|
| **C++** | `native/` (core engine, CLI, tests in `native/tests/`) | `cmake --build build` or presets; `./scripts/build_universal.sh` for macOS universal | `ctest --test-dir build --output-on-failure` | `./scripts/run_linters.sh` (cppcheck, clang-tidy, etc.) |
| **Python** | `python/` (TUI, integration, services, tests) | — (interpreted); Cython bindings built via CMake | `uv run pytest python/tests/` or `pytest python/tests/` | `ruff`, `pylint`; `just lint-shell` for scripts |
| **Rust** | `agents/backend/` (crates: api, ledger, market_data, nats_adapter, risk, strategy, discount_bank_parser) | `cargo build` in `agents/backend/` | `cargo test` in `agents/backend/` | `cargo clippy` |
| **Go** | `agents/go/` (api-gateway, collection-daemon, config-validator, heartbeat-aggregator, nats-questdb-bridge, supervisor) | `go build ./...` in `agents/go/` | `go test ./...` | `golangci-lint`; `just exarp-lint` |
| **TypeScript / React** | `web/` (Vite, React, PWA) | `npm run build` in `web/` | `npm run test` (Vitest), `npm run e2e` (Playwright) | `npm run lint`, `npm run type-check` |
| **Swift** | `ios/`, `desktop/` (SwiftUI, AppKit) | Xcode / `xcodebuild` or `swift build` per project | Project-specific | Xcode / SwiftLint if configured |

## Shared and generated code

- **Protocol Buffers** (`proto/`): `proto/messages.proto` → C++ (generated at CMake build), Python/Go/TypeScript via `./proto/generate.sh`. See `docs/message_schemas/README.md`.
- **Config**: JSON under `config/`; shared by TUI, PWA, and backend. See `docs/platform/IMPROVEMENT_PLAN.md` (P1-B) and shared config design.

## Cross-language boundaries

- **NATS**: Rust backend, Go agents (api-gateway, collection-daemon, nats-questdb-bridge), and optionally Python/Web clients. See `docs/platform/DATAFLOW_ARCHITECTURE.md`.
- **REST**: api-gateway (Go) proxies to Rust backend and Python services; TUI and Web consume gateway. Default gateway base URL: `http://localhost:9000`.
- **Ledger**: Rust `agents/backend/crates/ledger` is the single writer (SQLite WAL); Python reads read-only or via REST. See P1-A in `docs/platform/IMPROVEMENT_PLAN.md`.

## Quick reference

- **Canonical project guidelines:** `AGENTS.md`, `CLAUDE.md`.
- **Build/lint/test shortcuts:** `.cursor/rules/just-cmake-shortcuts.mdc`, `docs/CURSOR_PROJECT_COMMANDS.md` (if present).
- **API and external docs:** `docs/API_DOCUMENTATION_INDEX.md`.
