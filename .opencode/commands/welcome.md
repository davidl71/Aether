---
description: Welcome to Aether - get oriented with the project
---

# Welcome to Aether

Get oriented with the project structure, key files, and how to start working.

## Project Overview

Aether is a **relative-value financing console** for comparing synthetic financing across bonds, T-bills, options (box spreads), bond ETFs, and bank loans.

**Current mode**: Read-only exploration. The TUI/CLI are operator consoles, not just tickers.

## Quick Start

```bash
# 1. Prime session (loads tasks, hints, suggestions)
/prime

# 2. Build the project
cd agents/backend && cargo build

# 3. Run backend (terminal 1)
cargo run -p backend_service

# 4. Run TUI (terminal 2)
cargo run -p tui_service
```

## Key Commands

| Command | Purpose |
|---------|---------|
| `/prime` | Prime session with exarp-go context |
| `/tasks` | List current tasks |
| `/scorecard` | Project health scorecard |
| `/health` | Run health checks |
| `/setup` | First-time setup |

## Project Structure

```
Aether/
├── agents/backend/       # Rust workspace (active)
│   ├── crates/          # api, broker_engine, ib_adapter, ledger, etc.
│   ├── services/        # backend_service, tui_service, tws_yield_curve_daemon
│   └── bin/cli          # CLI entry point
├── docs/                # Documentation
├── scripts/             # Helper scripts
└── config/              # Config examples
```

## Key Files

| File | Purpose |
|------|---------|
| `AGENTS.md` | Canonical project guidelines |
| `CLAUDE.md` | Claude Code quick reference |
| `docs/AI_WORKFLOW.md` | Workflow defaults |
| `docs/DATA_EXPLORATION_MODE.md` | Current product direction |

## Next Steps

1. Read `/setup` if this is your first time
2. Run `/prime` to load context
3. Check `/tasks` to see what's in progress
4. Start with a small task from the suggestions

Happy coding!
