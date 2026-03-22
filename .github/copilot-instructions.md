# GitHub Copilot Instructions

This repository follows the comprehensive guidelines in [AGENTS.md](../AGENTS.md).

## Project

Comprehensive multi-asset synthetic financing platform. Manages financing across options, futures, bonds, loans, and pension funds with unified portfolio management across 21+ accounts and multiple brokers. Box spreads are one active strategy (spare cash, T-bill-equivalent yields). **Rust-first codebase**: all active development is in `agents/backend/`. C++ native build has been removed.

## Structure

| Directory | Contents |
|-----------|----------|
| `agents/backend/crates/` | Rust crates (api, broker_engine, ib_adapter, ledger, market_data, nats_adapter, quant, risk, strategy, etc.) |
| `agents/backend/services/` | backend_service (:8080), tui_service, tws_yield_curve_daemon |
| `agents/backend/bin/cli` | Rust CLI entry point |
| `config/` | Config templates |
| `docs/` | Documentation |
| `proto/` | Protocol Buffer definitions |

## Code Style

- **Rust** (primary), 4-space indentation
- Types: `PascalCase` (`Scenario`, `OrderManager`)
- Functions/variables: `snake_case` (`make_scenario`, `strike_price`)
- Constants: `k` prefix (`kMaxPositions`, `kDefaultPort`)
- Comment only non-obvious trading math

## Build & Test

All Rust, run from `agents/backend/`:

```bash
cd agents/backend
cargo build
cargo test
cargo fmt && cargo clippy
./scripts/run_linters.sh   # full project lint
```

## Key Files

| File | Purpose |
|------|---------|
| `agents/backend/services/backend_service` | REST+WS API, NATS collector, snapshot |
| `agents/backend/services/tui_service` | Ratatui TUI |
| `agents/backend/crates/broker_engine/` | Broker trait + domain types |
| `agents/backend/crates/ib_adapter/` | IBKR/TWS adapter |
| `agents/backend/crates/quant/` | Greeks, margin, amortization, yield curve |
| `agents/backend/crates/risk/` | Risk calculations |
| `ARCHITECTURE.md` | System architecture |

## Dependencies

- Rust crates managed via Cargo.toml
- TWS API sourced from sibling `tws-api` repo or `agents/backend/crates/ib_adapter/third_party/`

## Safety Rules

- Never commit credentials or API keys
- Paper trading port 7497 only
- Never modify vendor code in `ib_adapter/src/` — wrap IBKR TWS API calls there
- All trading/risk calculations must have Rust `#[test]` tests
- Imperative commit messages, 72-char subject lines

For complete guidelines, see [AGENTS.md](../AGENTS.md).
