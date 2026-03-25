---
description: First-time setup for OpenCode development
---

# OpenCode Setup

First-time setup for developing Aether with OpenCode.

## Prerequisites

Ensure you have installed:
- **Rust** 1.75+ (`rustc --version`)
- **CMake** 3.20+ (`cmake --version`)
- **Ninja** (usually with CMake)
- **exarp-go** (task management)

## Setup Steps

### 1. Verify exarp-go

```bash
# Check if exarp-go is available
which exarp-go

# Or use the wrapper
./scripts/run_exarp_go.sh --version
```

If not installed, see the exarp-go repository.

### 2. Run MCP Setup

```bash
./scripts/setup_mcp.sh
```

This configures MCP servers for your machine.

### 3. Prime Session

```bash
/prime
```

This loads:
- Current tasks from exarp-go
- Project hints and context
- Suggested next actions

### 4. Build the Project

```bash
# Fast Rust build
cd agents/backend && cargo build

# Or full build
make build
```

### 5. Verify Setup

```bash
# Run tests
cargo test

# Run linters
make lint

# Check health
/health
```

## OpenCode-Specific Features

### Plugin Tools (Instant)

| Tool | Use |
|------|-----|
| `exarp_tasks` | List tasks |
| `exarp_update_task` | Update task status |
| `exarp_prime` | Prime session |
| `exarp_config` | View config |

### Slash Commands

| Command | Runs |
|---------|------|
| `/welcome` | This help |
| `/setup` | Setup guide |
| `/prime` | Prime session |
| `/tasks` | List tasks |
| `/scorecard` | Project scorecard |
| `/health` | Health checks |

### MCP Tools

Full exarp-go capabilities via MCP:
- `task_workflow` - Task CRUD, batch operations
- `session` - Prime, handoff, context
- `report` - Scorecard, overview, briefing
- `health` - Docs, git, tools checks

## Skills

Available skills in `.cursor/skills/`:
- `exarp-go` - Task management
- `build-shortcuts` - Make/cargo commands
- `trading-safety` - Trading safety checks
- `before-commit` - Pre-commit checklist

## Daily Workflow

```bash
# Start of day
/prime                    # Load context
/tasks                    # See what's todo

# During work
<do work>
exarp_update_task(task_id="T-...", new_status="Done")
/followup                 # Get follow-up suggestions

# End of day
/scorecard                # Check project health
```

## Troubleshooting

**exarp-go not found**: Check `PROJECT_ROOT` env var or run `./scripts/run_exarp_go.sh`

**Build fails**: Ensure you're in `agents/backend/` for Rust builds

**MCP errors**: Run `./scripts/setup_mcp.sh` to reconfigure

## Learn More

- Project guide: `AGENTS.md`
- AI workflow: `docs/AI_WORKFLOW.md`
- Architecture: `ARCHITECTURE.md`
