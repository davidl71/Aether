# Codex Instructions

See [AGENTS.md](AGENTS.md) for complete project guidelines. For multi-editor
setup and command parity across Codex, Cursor, Claude Code, OpenCode, skills,
and subagents, see [docs/archive/AI_EDITOR_SETUP.md](docs/archive/AI_EDITOR_SETUP.md). For
exarp-go session/task workflows used by the other editors, see
[docs/archive/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md](docs/archive/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md).
For current product direction and workflow defaults, see
[docs/DATA_EXPLORATION_MODE.md](docs/DATA_EXPLORATION_MODE.md) and
[docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md).

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
to the correct location.

---

## Project at a Glance

Comprehensive multi-asset synthetic financing optimization platform. Manages
financing across options, futures, bonds, bank loans, and pension funds with
unified portfolio management, cash flow modeling, opportunity simulation, and
multi-instrument relationship optimization across 21+ accounts and multiple
brokers. Box spreads are one active strategy component rather than the whole
system.

Primary operator workflow:

- Explore relative-value opportunities across bonds, T-bills, synthetic boxes,
  bond ETFs, and bank loans
- Express one instrument in terms of another on a common financing basis
- Explore specific synthetic instruments with instrument-level views and OHLCV
  candle bars where useful
- Inspect configuration and daemon/service health from the TUI
- Treat the TUI/CLI as an operator console, not as a simple ticker or watchlist

Current default mode is **read-only exploration**. Keep real positions and data
surfaces visible, but do not add or restore execution paths unless that
direction changes explicitly.

When reasoning about product behavior, prefer cross-instrument comparison,
carry/funding trade-offs, opportunity exploration, and synthetic instrument
time-series views over single-instrument static summaries. Assume user-facing
configuration and routine daemon-health workflows should be operable from the
TUI.

**Rust-first codebase**: all active development is in `agents/backend/`. C++ native build has been removed (see root `CMakeLists.txt`).

Primary implementation areas:

- `agents/backend/crates/` — Rust crates (api, broker_engine, ib_adapter, ledger, market_data, nats_adapter, quant, risk, strategy, etc.)
- `agents/backend/services/` — backend_service (:8080), tui_service, tws_yield_curve_daemon
- `agents/backend/bin/cli` — Rust CLI entry point
- `docs/` — architecture, build, API, AI-editor setup

## Build, Test, Lint

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

AI-friendly JSON output:
```bash
./scripts/build_rust_ai_friendly.sh --json-only
```

## Key Files

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
| AI/editor setup | `docs/archive/AI_EDITOR_SETUP.md` |

## Code Style

- Rust (primary): standard cargo conventions
- Types: `PascalCase`
- Functions and variables: `snake_case`
- Constants: `k` prefix
- Add short comments only when trading math or scaling is not obvious

## Working Rules

- Never commit credentials, API keys, or broker secrets
- Always use paper trading port `7497` for testing
- Never modify vendor code in `ib_adapter/src/` directly — wrap IBKR TWS API calls there
- All trading, pricing, and risk logic changes need matching `#[test]` tests
- Prefer existing scripts, presets, and repository conventions over ad hoc commands
- Use imperative commit messages with 72-character subject lines

## exarp Workflow

For non-trivial implementation work:

1. Create or update an exarp task before coding
2. Compact the active context before starting long-running work
3. Create 1-2 exarp follow-up tasks when finishing
4. Add a result comment and update task status after verification

Use the prompt shape in [docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md):
`Goal / Context / Constraints / Done when / Verification`.
