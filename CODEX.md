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
| AI/editor setup | `docs/AI_EDITOR_SETUP.md` |

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
