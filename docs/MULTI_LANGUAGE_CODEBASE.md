# Multi-Programming-Language Codebase

This repository is **Rust-first**; C++ native build has been removed. This doc maps each language to directories, build/test/lint commands, and ownership.

## Language map

| Language | Directories | Build | Test | Lint |
|----------|-------------|--------|------|------|
| **Rust** | `agents/backend/` (crates: api, ib_adapter, ledger, market_data, nats_adapter, quant, risk, strategy, discount_bank_parser; services: backend_service, tui_service; bin: cli) | `cargo build` in `agents/backend/` | `cargo test` in `agents/backend/` | `cargo clippy` |
| **Python** | `scripts/` (utilities only) | — (interpreted) | — | `uv run ruff check scripts/` |
| **TypeScript / React** | `web/` (archived, not active runtime) | Historical only | Historical only | Historical only |
| **C++** | `native/` removed from build | — | — | — |

## Component and backend ownership

| Area | Owner | Location |
|------|--------|----------|
| Frontend API, snapshot, health | Rust | `crates/api`, `services/backend_service` |
| Broker adapters (IBKR) | Rust | `crates/ib_adapter` |
| Ledger | Rust | `crates/ledger` |
| Quant / risk / pricing | Rust | `crates/quant`, `crates/risk` |
| Market data, strategy | Rust | `crates/market_data`, `crates/strategy` |
| NATS ingestion | Rust | `crates/nats_adapter`, backend_service |
| TUI, CLI | Rust | `services/tui_service`, `bin/cli` |

See **ARCHITECTURE.md** and **AGENTS.md** for full ownership tables.

## Rust crate inventory (agents/backend)

| Crate | Purpose |
|-------|---------|
| **api** | REST routes, snapshot, frontend read models, health |
| **ib_adapter** | IBKR/TWS adapter (broker connectivity) |
| **ledger** | Durable ledger state |
| **market_data** | Market data read path |
| **nats_adapter** | NATS protobuf and messaging |
| **quant** | Greeks, margin, amortization, convexity, yield curve, option chain |
| **risk** | Risk calculations |
| **strategy** | Strategy logic |
| **discount_bank_parser** | Bank statement / reconciliation |
| **backend_service** | Binary: REST+WS API, NATS collector, LIVE_STATE, QuestDB |
| **tui_service** | Binary: Ratatui TUI |
| **cli** | Binary: CLI entry point |

## Build status

| Component | Status | Notes |
|-----------|--------|-------|
| Rust backend | **Active** | Primary runtime; `cargo build` in `agents/backend/` |
| Rust TUI / CLI | **Active** | `tui_service`, `bin/cli` |
| C++ native | **Removed** | No longer in build (root CMakeLists.txt) |
| NATS | **Active** | Rust backend is producer/consumer |

## Shared and generated code

- **Protocol Buffers** (`proto/`): `proto/messages.proto` → Rust (prost in `nats_adapter`). See `docs/message_schemas/README.md`.
- **Config**: JSON under `config/` shared by Rust TUI and backend; CLI uses TOML (`config/config.toml`). See [TUI/CLI feature parity](platform/TUI_CLI_FEATURE_PARITY.md).

## Boundaries

- **NATS**: Rust backend is the active producer/consumer; TUI uses REST (and optional NATS). See `docs/platform/DATAFLOW_ARCHITECTURE.md`.
- **REST / WebSocket**: Rust TUI and CLI talk to Rust backend (:8080).
- **Ledger**: Rust `crates/ledger` is the durable ledger owner.
- **Brokers**: Rust `crates/ib_adapter` owns IBKR connectivity; no separate broker daemons.

## Rust Quant Finance

See `docs/RUST_FINANCE_LIBRARIES.md` for libraries that could replace C++ QuantLib code.

## Quick reference

- **Canonical project guidelines:** `AGENTS.md`, `CLAUDE.md`.
- **Build/lint/test shortcuts:** `.cursor/rules/just-cmake-shortcuts.mdc`, `docs/CURSOR_PROJECT_COMMANDS.md` (if present).
- **API and external docs:** `docs/API_DOCUMENTATION_INDEX.md`.
