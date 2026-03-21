# Aether - Repository Guidelines

## Project Overview

Aether (formerly ib_box_spread_full_universal) - Comprehensive multi-asset synthetic financing optimization platform. Manages financing across options, futures, bonds, bank loans, and pension funds with unified portfolio management, cash flow modeling, opportunity simulation, and multi-instrument relationship optimization across 21+ accounts and multiple brokers (IBKR).

Rust-first codebase: backend, TUI, CLI, and broker adapter in `agents/backend/`. C++ native build has been removed (see root `CMakeLists.txt`). See **`docs/MULTI_LANGUAGE_CODEBASE.md`** for language map and build/test/lint commands.

Box spreads are one active strategy component (7-10% of portfolio, spare cash allocation for T-bill-equivalent yields). The platform supports multiple strategy types including futures-implied financing, bond ETFs, and secured lending.

## Project Structure & Module Organization

```
Aether/
├── agents/backend/          # Rust workspace (primary codebase)
│   ├── crates/              # api, broker_engine, ib_adapter, ledger, market_data, nats_adapter, quant, risk, strategy, discount_bank_parser, common, tws_yield_curve, yatws_adapter
│   ├── services/            # backend_service (:8080), tui_service, tws_yield_curve_daemon
│   └── bin/                 # cli (Rust CLI)
├── native/                  # C++ removed from build (see root CMakeLists.txt)
├── web/                     # Archived React web application (not active runtime)
├── proto/                   # Protocol Buffer definitions
├── config/                  # Configuration files (example configs only in repo)
├── scripts/                 # Helper scripts (build, lint, deploy)
└── docs/                    # Documentation
```

### Component and backend ownership

| Area | Owner | Location |
|------|--------|----------|
| Frontend API, snapshot, health | Rust | `agents/backend/crates/api`, `services/backend_service` |
| Broker abstraction (traits + domain) | Rust | `agents/backend/crates/broker_engine` |
| Broker adapters (IBKR) | Rust | `agents/backend/crates/ib_adapter` |
| Ledger | Rust | `agents/backend/crates/ledger` |
| Quant / risk / pricing | Rust | `crates/quant`, `crates/risk` |
| Market data, strategy | Rust | `crates/market_data`, `crates/strategy` |
| NATS ingestion, LIVE_STATE | Rust | `crates/nats_adapter`, backend_service |
| TUI | Rust | `services/tui_service` |
| CLI | Rust | `agents/backend/bin/cli` |

See **ARCHITECTURE.md** for full ownership and current build settings.

### Key Source Files (Rust)

| Path | Purpose |
|------|---------|
| `agents/backend/services/backend_service` | REST+WS API, NATS collector, snapshot |
| `agents/backend/services/tui_service` | Ratatui TUI |
| `agents/backend/services/tws_yield_curve_daemon` | TWS yield curve fetcher daemon |
| `agents/backend/crates/api` | REST routes, snapshot, frontend read models |
| `agents/backend/crates/broker_engine` | Broker trait + domain types (engine abstraction) |
| `agents/backend/crates/common` | Shared snapshot/event types across crates |
| `agents/backend/crates/ib_adapter` | IBKR/TWS adapter (implements BrokerEngine) |
| `agents/backend/crates/yatws_adapter` | yatws TWS adapter (implements BrokerEngine) |
| `agents/backend/crates/quant` | Greeks, margin, amortization, convexity, yield curve |
| `agents/backend/crates/risk` | Risk calculations |
| `agents/backend/crates/ledger` | Durable ledger |
| `agents/backend/crates/nats_adapter` | NATS protobuf and messaging |
| `agents/backend/crates/tws_yield_curve` | TWS SOFR/treasury yield curve |
| `agents/backend/bin/cli` | CLI entry point |

## Build, Test & Development Commands

```bash
# Rust (primary)
cd agents/backend && cargo build
./scripts/build_rust_ai_friendly.sh --json-only   # AI-friendly JSON output

# Run backend and TUI
cargo run -p backend_service   # :8080
cargo run -p tui_service       # TUI
cargo run -p cli               # CLI

# Test and lint
cargo test
cargo clippy
./scripts/run_linters.sh       # includes Rust linters
```

See `docs/MULTI_LANGUAGE_CODEBASE.md` for full language map and `docs/BUILD_PARALLELIZATION_AND_MODULARITY.md` for parallelization. C++ native build is removed; CMake targets at repo root are for lint/scripts only.

## Coding Style & Naming Conventions

Target ISO C++20. Prefer two-space indentation, Allman braces for multi-line scopes, and 100-character soft wraps.

| Element | Convention | Example |
|---------|-----------|---------|
| Types | `PascalCase` | `Scenario`, `OrderManager` |
| Functions | `snake_case` | `make_scenario`, `calculate_profit` |
| Variables | `snake_case` | `strike_price`, `expiry_date` |
| Constants | `k` prefix | `kMaxPositions`, `kDefaultPort` |

Add short `//` comments only where the trading math is non-obvious (e.g., APR scaling by the contract multiplier).

## Code Patterns

### derive_builder for Event/Data Structs

New or refactored event, data, and snapshot structs with 5+ fields **must** use `#[derive(derive_builder::Builder)]` with field-level `#[builder(default)]` defaults. This ensures all construction sites remain valid when fields are added.

```rust
#[derive(Clone, Debug, Default, derive_builder::Builder)]
#[builder(setter(into), default)]
pub struct MarketDataEvent {
    #[builder(default = "0")]
    pub contract_id: i64,
    #[builder(setter(into))]
    pub symbol: String,
    #[builder(default = "0.0")]
    pub bid: f64,
}

// Construction: only set what matters
MarketDataEventBuilder::default()
    .symbol("SPY")
    .bid(500.0)
    .ask(502.0)
    .build()
```

**Enforced by:** `.opencode/rules/derive-builder-pattern.md`

### Shared Types in `crates/common/`

Types used by **multiple backend crates** (`api`, `nats_adapter`, `broker_engine`, `market_data`, `strategy`) must live in `crates/common/`. Do **not** copy structs across crates.

```
crates/common/snapshot.rs   ← shared types (PositionSnapshot, HistoricPosition, etc.)
crates/api/src/state.rs     ← re-exports from common + api-only types
crates/nats_adapter/       ← imports from common (NOT api, avoids cycle)
```

**Enforced by:** `.opencode/rules/shared-types-pattern.md`

### Proto Generation

- Prost proto types are generated at build time via `nats_adapter/build.rs` from `proto/messages.proto`
- `From<pb::X> for common::X` and `From<common::X> for pb::X` go in `nats_adapter/src/conversions.rs`
- Never hand-edit generated proto stubs

## Dependencies

| Dependency | Location | Purpose |
|------------|----------|---------|
| TWS API | `../tws-api/` (sibling repo) | IBKR connectivity |
| nlohmann/json | FetchContent (v3.11.3) | JSON parsing |
| spdlog | FetchContent (v1.13.0) | Logging |
| CLI11 | FetchContent (v2.4.1) | CLI argument parsing |
| Catch2 | FetchContent (v3.5.2) | Unit testing |
| Eigen3 | FetchContent (v3.4.0) | Linear algebra |
| QuantLib | FetchContent (v1.36) | Quantitative finance |
| NLopt | FetchContent (v2.9.1) | Optimization |
| Boost | System (Homebrew) | Date/time, filesystem |

## IB API Integration Notes

The TWS API is sourced from the sibling `tws-api` repo (clone to `../tws-api/` next to this project) or extracted to `native/third_party/tws-api/` if using the IBKR zip. IBKR connectivity is via Rust `ib_adapter`; no C++ client is used. Never commit IB credentials, logs, or downloaded vendor artifacts — treat everything under `build/` as ephemeral. The CLI currently prints synthetic market data; gate any future live requests behind configuration flags.

## Testing Guidelines

Rust tests live in `agents/backend/crates/*/tests/` (or `#[test]` modules). Legacy C++ tests were in `native/tests/` (Catch2). Run `cargo test` in `agents/backend/` for Rust tests.

## Commit & Pull Request Guidelines

Follow imperative, 72-character subject lines ("Add TSV formatter for CLI"). In the body, summarize option scenarios touched, list the commands run (build, tests, sample CLI output), and note IB API version bumps. PRs must call out configuration changes (e.g., new env vars or IB gateway ports).

## Security

- Never commit credentials, API keys, or secrets
- Always use paper trading port (7497) for testing
- Gate live trading behind explicit configuration flags
- Never modify third-party code directly — use wrappers in `agents/backend/crates/ib_adapter/src/`

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

**Excluded from AI context:**
- `docs/archive/` — Historical C++/LEAN/WASM research; git-tracked but not indexed by AI tools. See `docs/ARCHIVE_CPP_KNOWLEDGE_SUMMARY.md` for extracted knowledge relevant to current development.
- `docs/research/integration/` — Cleared; docs moved to `docs/` or `docs/archive/`.

**Skills & subagents:** Cursor/plugin skills and subagents (e.g. mcp_task,
exarp-go, Claude agents) should use AGENTS.md as canonical context. `CLAUDE.md`
and `CODEX.md` are tool-specific quick references. See
[docs/AI_EDITOR_SETUP.md](docs/AI_EDITOR_SETUP.md) for setup and command parity
across Codex, OpenCode, Claude, Cursor, skills, and subagents.

## exarp Workflow Rule

For any non-trivial implementation or refactor expected to span multiple steps,
files, or commits:

1. **Track in exarp before coding**
   - If there is an existing task, update or comment on it before starting.
   - If there is no suitable task, create one first.
2. **Compact context before starting long work**
   - Summarize the active goal, constraints, current worktree state, and prior
     decisions so the next execution step starts from a clean context.
3. **Create follow-up tasks when finishing**
   - Add 1-2 follow-up exarp tasks for verification, cleanup, migration, docs,
     or deferred risk uncovered by the implementation.
4. **Update task status at completion**
   - Add a result comment with verification commands and move the completed task
     to the correct status.

Default scope for this rule:

- Apply it to architecture work, data-path changes, refactors, migrations,
  compatibility cuts, or any task likely to create follow-up work.
- Do not require it for trivial one-shot answers, status checks, or tiny
  isolated edits with no realistic follow-up.
