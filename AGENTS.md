# Aether - Repository Guidelines

## Project Overview

Aether (formerly ib_box_spread_full_universal) - Comprehensive multi-asset synthetic financing optimization platform. Manages financing across options, futures, bonds, bank loans, and pension funds with unified portfolio management, cash flow modeling, opportunity simulation, and multi-instrument relationship optimization across 21+ accounts and multiple brokers (IBKR).

Rust-first codebase: backend, TUI, CLI, and broker adapter in `agents/backend/`. C++ native build has been removed (see root `CMakeLists.txt`). See **`docs/MULTI_LANGUAGE_CODEBASE.md`** for language map and build/test/lint commands.

Current product direction is documented in
**`docs/DATA_EXPLORATION_MODE.md`**. AI/editor workflow defaults are in
**`docs/AI_WORKFLOW.md`**.

---

## ⚠️ CRITICAL: Cargo Workspace Location

**The Rust workspace is at `agents/backend/`, NOT the project root.**

All `cargo` commands must be run from that directory:

```bash
cd agents/backend && cargo build
cd agents/backend && cargo test
cd agents/backend && cargo run -p backend_service
```

Running `cargo` from the project root will show an error message directing you
to the correct location. See `.cursor/rules/cargo-workspace-path.mdc` for details.

---

Box spreads are one active strategy component (7-10% of portfolio, spare cash allocation for T-bill-equivalent yields). The platform supports multiple strategy types including futures-implied financing, bond ETFs, and secured lending.

### Product framing for AI agents

Treat Aether as a **relative-value financing and opportunity-discovery
console**, not as a single-strategy box spread app and not as a generic quote
tracker.

Primary operator use case:

- Explore and compare opportunities across **bonds, T-bills, synthetic boxes,
  bond ETFs, and bank loans**
- Express one instrument **in terms of another** (e.g. box spread yield vs.
  T-bill yield, bond ETF carry vs. direct bond ladder, bank loan cost vs.
  synthetic financing)
- Explore **specific synthetic instruments** as first-class objects, including
  instrument-specific views, spreads, and **OHLCV candle bars** where a time
  series is meaningful
- Inspect and manage **configuration and daemon/service health** directly from
  the TUI as part of the operator workflow
- Surface relative spread, carry, haircut, duration, convexity, liquidity, and
  funding trade-offs in a form that supports portfolio allocation decisions

Implications for AI behavior:

- Do not reduce the product to "box spread trading" when describing features,
  tasks, or UI priorities
- Do not reduce the TUI to a "ticker/watchlist" mental model; it is an
  operator console for comparing financing instruments and executing or managing
  those positions
- Prefer wording like **relative value**, **cross-instrument comparison**,
  **financing alternatives**, **opportunity exploration**, and **instrument
  expression/parity**
- Treat **synthetic instrument candles/OHLCV history** as part of the product
  surface for discovery and comparison, not as optional decoration
- Treat **full TUI configurability** and **daemon health visibility** as core
  product requirements; avoid assuming users should drop to config files or
  shell commands for routine operational workflows
- When proposing analytics, UI, or data models, bias toward workflows that let
  the user compare instruments on a common basis rather than inspect each
  instrument type in isolation
- Assume **read-only exploration mode** unless the repo docs or the user make a
  newer explicit decision to re-enable execution work

## Project Structure & Module Organization

```
Aether/
├── agents/backend/          # Rust workspace (primary codebase)
│   ├── crates/              # api, credential_store, broker_engine, ib_adapter, ledger, market_data, nats_adapter, quant, risk, strategy, discount_bank_parser, common, tws_yield_curve
│   ├── services/            # backend_service (:8080), tui_service, tws_yield_curve_daemon
│   └── bin/                 # cli (Rust CLI)
├── agents/nautilus/         # Optional Python Nautilus ↔ NATS agent (uv/pytest); see docs/PYTHON_INVENTORY.md
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
| Credential storage (env / keyring / file) | Rust | `agents/backend/crates/credential_store` (re-exported as `api::credentials`) |
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
| `agents/backend/crates/api/src/finance_rates/` | SOFR/Treasury/box-spread read model (`api::finance_rates`; `types`, `curve`, `benchmarks`, `comparison`) |
| `agents/backend/crates/broker_engine` | Broker trait + domain types (engine abstraction) |
| `agents/backend/crates/common` | Shared snapshot/event types across crates |
| `agents/backend/crates/ib_adapter` | IBKR/TWS adapter (implements BrokerEngine) |
| `agents/backend/crates/quant` | Greeks, margin, amortization, convexity, yield curve |
| `agents/backend/crates/risk` | Risk calculations |
| `agents/backend/crates/ledger` | Durable ledger |
| `agents/backend/crates/nats_adapter` | NATS protobuf and messaging |
| `agents/backend/crates/tws_yield_curve` | TWS SOFR/treasury yield curve |
| `agents/backend/bin/cli` | CLI entry point |

## Build, Test & Development Commands

**IMPORTANT:** The Rust workspace is in `agents/backend/`. Always `cd agents/backend` before running cargo commands.

```bash
# Rust (primary)
cd agents/backend && cargo build
./scripts/build_rust_ai_friendly.sh --json-only   # AI-friendly JSON output

# Run backend and TUI (from agents/backend/)
cargo run -p backend_service   # :8080
cargo run -p tui_service       # TUI
# Optional: ratatui-interact field↔list sub-focus (charts search, orders filter, palette, loan import); see docs/TUI_RATATUI_INTERACT.md
# cargo run -p tui_service --features tui-interact

cargo run -p cli               # CLI

# Test and lint (from agents/backend/)
cargo test
cargo clippy
./scripts/run_linters.sh       # includes Rust linters
```

**Common Error:** If you see `error: could not find Cargo.toml`, you're in the wrong directory. Run `cd agents/backend` first.

See `docs/MULTI_LANGUAGE_CODEBASE.md` for full language map and `docs/BUILD_PARALLELIZATION_AND_MODULARITY.md` for parallelization. C++ native build is removed; CMake targets at repo root are for lint/scripts only.

## Coding Style & Naming Conventions

Active development is **Rust-first**. Follow normal Rust crate/service
conventions in `agents/backend/` and keep comments short and focused on
non-obvious trading math or operational behavior.

Legacy C++ conventions below apply only when editing archived or reintroduced
native code paths:

- Target ISO C++20
- Prefer two-space indentation and Allman braces for multi-line scopes
- Use `PascalCase` for types, `snake_case` for functions/variables, and `k`
  prefix for constants
- Add short `//` comments only where the trading math is non-obvious

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

### Market Data Provider Factories

All market data providers must use the factory pattern for discoverability and consistent initialization.

**Quote Sources** - Use `MarketDataSourceFactory` or `SimpleMarketDataSourceFactory` trait:
```rust
// In market_data/src/{provider}.rs
pub struct YahooFinanceSourceFactory;
impl SimpleMarketDataSourceFactory for YahooFinanceSourceFactory {
    fn name(&self) -> &'static str { "yahoo" }
    fn create(&self, symbols: &[String], interval: Duration) -> anyhow::Result<Box<dyn MarketDataSource>> {
        Ok(Box::new(YahooFinanceSource::new(symbols.to_vec(), interval)?))
    }
}
```

Register in `crates/market_data/src/lib.rs`:
```rust
pub fn provider_registry() -> &'static HashMap<&'static str, DynFactory> {
    // Use create_provider("yahoo") to instantiate
}
```

**Options Sources** - Use `OptionsDataSource` trait with `options_registry()`:
```rust
pub fn create_options_provider(name: &str) -> anyhow::Result<Box<dyn OptionsDataSource>> {
    let registry = options_registry();
    registry.get(name).ok_or_else(|| /* */)?()
}
```

**Enforced by:** `docs/MARKET_DATA_PROVIDER_ARCHITECTURE.md`

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

**Gaps affecting AI tooling:** `docs/API_DOCUMENTATION_INDEX.md` is still cited from AGENTS.md, CLAUDE.md, and other guides but **is not in the repo**—search `docs/` and `agents/backend/` until the file returns or references are fixed. **`.cursor/commands.json`** still runs `python3 python/tools/*.py` for several palette commands; **`python/tools/` is missing** (`python/` is effectively `generated/` only), so those commands fail until scripts exist or paths change.

**Excluded from AI context:**
- `docs/archive/` — Historical C++/LEAN/WASM research; git-tracked but not indexed by AI tools. See `docs/ARCHIVE_CPP_KNOWLEDGE_SUMMARY.md` for extracted knowledge relevant to current development.
- `docs/research/integration/` — Cleared; docs moved to `docs/` or `docs/archive/`.

**Skills & subagents:** Cursor/plugin skills and subagents (e.g. mcp_task,
exarp-go, Claude agents) should use AGENTS.md as canonical context. `CLAUDE.md`
and `CODEX.md` are tool-specific quick references. See
[docs/archive/AI_EDITOR_SETUP.md](docs/archive/AI_EDITOR_SETUP.md) for setup and command parity
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
   - Add 1-2 follow-up exarp tasks for verification, cleanup, docs,
     or deferred risk uncovered by the implementation.
4. **Update task status at completion**
   - Add a result comment with verification commands and move the completed task
     to the correct status.

Default scope for this rule:

- Apply it to architecture work, data-path changes, refactors, clean cuts, or
  any task likely to create follow-up work.
- Do not require it for trivial one-shot answers, status checks, or tiny
  isolated edits with no realistic follow-up.

**CLI note:** When you call exarp-go from the terminal (outside the MCP/chat
workflow) prefer the repo wrapper `scripts/run_exarp_go.sh` (aka `exarp_go`) so
`PROJECT_ROOT`, `EXARP_MIGRATIONS_DIR`, and related env vars track this project.
For tool-specific commands you can also use `scripts/run_exarp_go_tool.sh -tool
<name> -args '<json>'` to keep the wrapper’s sanitization logic. This keeps task
updates, docs health, scorecard, and other automation aligned with the repo
context referenced by the agents and skills that follow.

**Cheatsheet (Aether Todo2 + exarp):** `.cursor/skills/aether-todo2-exarp/SKILL.md` and
`.cursor/rules/aether-todo2-exarp-cheatsheet.mdc` — bulk `task update --status Review --new-status Done`,
`task sync`, JSON `task_workflow` for comments/dependencies, `agents/backend` Cargo.lock discipline,
TUI workspace layout module (`ui/workspace_layout.rs`) vs chart hint naming.

Use **`docs/AI_WORKFLOW.md`** for the preferred prompt structure, backlog
hygiene, and thread-splitting defaults used in this repo.

## Learned User Preferences

- After changing task status in the DB or via exarp-go, run **`./scripts/run_exarp_go.sh task sync`** so `.todo2/state.todo2.json` and JSON-backed views stay aligned with SQLite.

## Learned Workspace Facts

- `scripts/run_exarp_go.sh` with **no** arguments starts stdio MCP mode; in a normal terminal it looks hung. For one-shot CLI use, always pass a subcommand (for example `task sync`, `task list`).
- Todo2: treat **`.todo2/todo2.db` as canonical** for task status; **`.todo2/state.todo2.json`** mirrors the DB. After DB changes, run `./scripts/run_exarp_go.sh task sync`. If a JSON-backed UI still disagrees with SQLite, prefer the DB and treat it as a sync or tooling gap.
- **RAM disk:** `workspace_ram_disk_manager.sh` is not in this repo; folder-open / Cursor RAM-disk tasks that called it were removed. For disk-caching workflows use **`scripts/setup_disk_caching.sh`** and CMake presets with **`*-ramdisk`** / `build-ramdisk`, not a missing workspace startup script.
- **`api` → `market_data` dependency:** do not make `market_data` depend on `api` (Cargo cycle). Shared credential resolution lives in **`credential_store`**; `market_data` and similar low crates should use that crate only; callers that already use `api` keep **`api::credentials`** as the stable path.
- **NATS subjects:** canonical `api.*` and system command subject strings are **`nats_adapter::topics`** (`topics::api`, `topics::system::all_commands()`). Operator-oriented listing: **`docs/NATS_TOPICS_REGISTRY.md`** (keep in sync when adding subjects).
- **`docs/EXECUTION_COCKPIT.md`** is an **exarp-generated snapshot** (automation / execution cockpit), not the source of truth; canonical task state is **`.todo2/todo2.db`** and **`task list` / `task_workflow`**. Refresh the file after bulk task edits so it does not reference stale or removed IDs.
- **exarp-go work** (MCP/CLI/tooling in the exarp-go repo) should be tracked in **that repo’s Todo2** with **`PROJECT_ROOT`** set to the **exarp-go tree**, not as parallel **Todo** rows in Aether—keeps migrations, binaries, and backlog aligned with the code that implements them.
