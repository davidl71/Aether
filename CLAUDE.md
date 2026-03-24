# Claude Code Instructions

See [AGENTS.md](AGENTS.md) for complete project guidelines. For multi-editor setup
(OpenCode, Cursor, skills, subagents), see [docs/archive/AI_EDITOR_SETUP.md](docs/archive/AI_EDITOR_SETUP.md).
For exarp-go (session prime, handoff, tasks, scorecard) in Cursor, Claude Code, and OpenCode, see [docs/archive/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md](docs/archive/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md).
For current product direction and workflow defaults, see
[docs/DATA_EXPLORATION_MODE.md](docs/DATA_EXPLORATION_MODE.md) and
[docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md).

## Project at a Glance

Comprehensive multi-asset synthetic financing platform. Manages financing across options, futures, bonds, loans, and pension funds with unified portfolio management, cash flow modeling, and multi-instrument optimization across 21+ accounts and multiple brokers. Box spreads are one active strategy component (T-bill-equivalent yields on spare cash). **Rust-first codebase**: all active development is in `agents/backend/`. C++ native build has been removed (see root `CMakeLists.txt`).

Primary operator workflow:

- Explore relative-value opportunities across bonds, T-bills, synthetic boxes,
  bond ETFs, and bank loans
- Express one instrument in terms of another on a common financing basis
- Explore specific synthetic instruments with instrument-level views and OHLCV
  candle bars where useful
- Inspect configuration and daemon/service health from the TUI
- Treat Aether as an operator console rather than a ticker-style tracking app

When proposing features, UX, or analytics, bias toward cross-instrument
comparison, financing alternatives, and synthetic instrument time-series views
instead of single-instrument inspection alone. Assume routine configuration and
daemon-health workflows belong in the TUI.

Current default mode is **read-only exploration**. Keep real positions and data
surfaces visible, but do not add or restore execution paths unless the product
direction changes explicitly.

## Build & Test

All active development is Rust. Run from `agents/backend/`:

```bash
cd agents/backend

# Build
cargo build

# Test
cargo test

# Lint
cargo fmt && cargo clippy

# Full lint (all languages)
./scripts/run_linters.sh
```

AI-friendly JSON output (for parsing):
```bash
./scripts/build_rust_ai_friendly.sh --json-only
```

## Key Files to Know

| What | Where |
|------|-------|
| Backend service (REST+WS API) | `agents/backend/services/backend_service` |
| TUI (ratatui) | `agents/backend/services/tui_service` |
| CLI entry point | `agents/backend/bin/cli` |
| Broker trait + domain | `agents/backend/crates/broker_engine/` |
| IBKR adapter | `agents/backend/crates/ib_adapter/` |
| Market data | `agents/backend/crates/market_data/` |
| Quant / risk / pricing | `agents/backend/crates/quant/` |
| Ledger | `agents/backend/crates/ledger/` |
| NATS messaging | `agents/backend/crates/nats_adapter/` |
| Architecture | `ARCHITECTURE.md` |
| API docs index | `docs/API_DOCUMENTATION_INDEX.md` |

## Common Tasks

### Adding a new Rust crate

1. Add to `agents/backend/crates/` with `Cargo.toml`
2. Add to workspace members in `agents/backend/Cargo.toml`
3. Add tests under `crates/*/tests/`

### Adding a Rust source file

1. Create `crates/<name>/src/<file>.rs`
2. Add `mod <file>;` to `crates/<name>/src/lib.rs`
3. Run `cargo test -p <crate>` to verify

## Safety Rules

- **Never** commit credentials, API keys, or secrets
- **Always** use paper trading port `7497` for testing
- **Never** modify vendor code in `ib_adapter/src/` — wrap IBKR TWS API calls there
- **Always** add `#[test]` tests for trading logic and risk calculations
- Gate live trading behind explicit configuration flags

## exarp Workflow

For any non-trivial implementation or refactor spanning multiple steps, files, or commits:

1. **Track in exarp before coding** — update an existing task or create one first
2. **Compact context before long work** — summarize goal, constraints, worktree state, prior decisions
3. **Create follow-up tasks when finishing** — 1-2 tasks for verification, cleanup, docs, or deferred risk
4. **Update task status at completion** — add a result comment with verification commands

Skip for trivial one-shot edits, status checks, or isolated fixes with no realistic follow-up.

Use the prompt shape in [docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md):
`Goal / Context / Constraints / Done when / Verification`.
