# Aether Documentation Index

**Last updated:** 2026-03-24

## Quick Links

| Doc | Purpose |
|-----|---------|
| [AGENTS.md](../AGENTS.md) | Canonical project guidelines |
| [ARCHITECTURE.md](../ARCHITECTURE.md) | System overview |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | How to contribute |
| [QUICKSTART_RUST.md](./QUICKSTART_RUST.md) | Rust dev quickstart |
| [DATA_EXPLORATION_MODE.md](./DATA_EXPLORATION_MODE.md) | Current read-only product direction and architectural boundaries |
| [AI_WORKFLOW.md](./AI_WORKFLOW.md) | Repo workflow defaults for AI-assisted implementation |

## Architecture & Design

| Doc | Purpose |
|-----|---------|
| [ARCHITECTURE.md](../ARCHITECTURE.md) | System overview, component ownership |
| [TUI_ARCHITECTURE.md](./TUI_ARCHITECTURE.md) | Ratatui TUI design, main loop, planned improvements |
| [TUI_UX_BENCHMARKS.md](./TUI_UX_BENCHMARKS.md) | UX comparison against benchmark trading/terminal apps |
| [ARCHITECTURE_BROKER_ENGINE.md](./ARCHITECTURE_BROKER_ENGINE.md) | Broker engine architecture and `ib_adapter` integration |
| [ALPACA_SOURCE_ARCHITECTURE.md](./ALPACA_SOURCE_ARCHITECTURE.md) | Alpaca as a market-data source only, not an execution engine |
| [CRATE_BOUNDARIES.md](./CRATE_BOUNDARIES.md) | Logic for Rust crate ownership and segmentation |
| [COMMAND_DB_TRADING_ENGINE_GUIDANCE.md](./COMMAND_DB_TRADING_ENGINE_GUIDANCE.md) | Guidance for command bus, database roles, and trading-engine scope |
| [TWS_BACKEND_PROVIDER_DECISION.md](./TWS_BACKEND_PROVIDER_DECISION.md) | Why `ib_adapter` is the active TWS backend path |
| [BACKLOG_EXECUTION_PLAN_2026_03_24.md](./BACKLOG_EXECUTION_PLAN_2026_03_24.md) | Manual backlog cleanup and execution waves after the current architecture decisions |

## Development Guides

| Doc | Purpose |
|-----|---------|
| [QUICKSTART_RUST.md](./QUICKSTART_RUST.md) | Rust build/run quickstart |
| [ADDING_A_BROKER_ADAPTER.md](./ADDING_A_BROKER_ADAPTER.md) | How to implement BrokerEngine trait for new broker |
| [MARKDOWN_STYLE_GUIDE.md](./MARKDOWN_STYLE_GUIDE.md) | Doc linting rules (gomarklint) |

## Type & Data Reference

| Doc | Purpose |
|-----|---------|
| [BACKEND_TYPE_COMPARISON.md](./BACKEND_TYPE_COMPARISON.md) | Position/Order/MarketData type map across crates |
| [HEALTH_DASHBOARD.md](./HEALTH_DASHBOARD.md) | Rust backend health endpoints |

## Planning (Active)

| Doc | Purpose |
|-----|---------|
| [BACKGROUND_TASK_LIFECYCLE.md](./BACKGROUND_TASK_LIFECYCLE.md) | Background task lifecycle in Rust backend |
| [NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md](./NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md) | NATS KV vs Redis timeline |
| [PRIMARY_CURRENCY_AND_TASE_HEDGING.md](./PRIMARY_CURRENCY_AND_TASE_HEDGING.md) | Multi-currency handling |
| [REAL_TASK_DEPENDENCIES_EVALUATION.md](./REAL_TASK_DEPENDENCIES_EVALUATION.md) | Task dependency analysis |
| [WAVE_RESUME_RUNBOOK.md](./WAVE_RESUME_RUNBOOK.md) | Wave-based task execution |
| [FUTURE_IMPROVEMENTS.md](./FUTURE_IMPROVEMENTS.md) | Future enhancement ideas |
| [LINT_LOG_FIX_SUGGESTIONS.md](./LINT_LOG_FIX_SUGGESTIONS.md) | Linting/logging improvements |
| [TRACKING_AND_GITIGNORE.md](./TRACKING_AND_GITIGNORE.md) | Build artifact tracking |

## Pattern References

| Doc | Purpose |
|-----|---------|
| [ARCHIVED_INTEGRATION_PATTERNS.md](./ARCHIVED_INTEGRATION_PATTERNS.md) | Patterns extracted from archived integration plans |

## Archived Docs

**All historical docs are in `docs/archive/`** (~416 files).

Key archived categories:
- `archive/box-spread/` — Box spread research and implementation docs
- `archive/planning/` — Historical planning docs  
- `archive/research/` — Framework evaluations, learnings
- `archive/platform/` — Backend, NATS, TWS integration docs
