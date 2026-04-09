# exarp-go Todo2 backlog (Aether)

How exarp-go lists and orders Todo2 tasks for this repo, and how `PROJECT_ROOT` selects which `.todo2` store is used.

## PROJECT_ROOT

exarp-go resolves tasks from **`.todo2/todo2.db`** under the project root (`PROJECT_ROOT` env, or walk-up for `.todo2` / `.exarp`).

- **Work on Aether’s backlog:** `PROJECT_ROOT` must be the **Aether repository root** (the directory that contains this `docs/` folder and `.todo2/`).
- **Work on exarp-go’s own tasks:** point `PROJECT_ROOT` at your exarp-go clone (see [`.cursor/skills/exarp-go/SKILL.md`](../.cursor/skills/exarp-go/SKILL.md)).

This repo’s Cursor MCP entry sets `PROJECT_ROOT` to Aether in [`.cursor/mcp.json`](../.cursor/mcp.json). The portable runner [`scripts/run_exarp_go.sh`](../scripts/run_exarp_go.sh) defaults `PROJECT_ROOT` to the repo root when run from Aether.

Wrong root ⇒ wrong task list.

## What counts as “backlog”

Definitions differ by tool:

| Surface | Meaning |
|--------|--------|
| **`task_workflow` `list`** | You choose explicitly, e.g. `status=Todo`, `status=In Progress`, or omit `status` for a broader list. |
| **`task_analysis` `execution_plan`** | Open execution statuses: **Todo**, **In Progress**, **Blocked** (`IsOpenStatus` in exarp-go `internal/models/constants.go`). **Review** is not included. Implementation: `BacklogExecutionOrder` + `IsBacklogStatus` in exarp-go `internal/tools/graph_helpers.go`. |
| **`task_analysis` `discover_tags` + `backlog_only`** | **Todo** or **In Progress** only (per tool schema). |
| **`session` `prime` → `suggested_next`** | Dependency waves / ready tasks (same mental model as execution ordering). |

**Note:** `BacklogExecutionOrder` in exarp-go includes **Blocked** because `IsBacklogStatus` uses `IsOpenStatus`; some comments say only “Todo + In Progress” but **Blocked** is included.

## Commands (CLI)

From Aether, with exarp-go on `PATH` or via `scripts/run_exarp_go.sh`:

```bash
./scripts/run_exarp_go.sh task list --status Todo
./scripts/run_exarp_go.sh task list --status "In Progress"
./scripts/run_exarp_go.sh task list --status Blocked
```

In the exarp-go repo, Makefile shortcuts exist (e.g. `make task-list-todo`); see exarp-go `CLAUDE.md`.

## MCP (Cursor / automation)

Prefer JSON + `compact` where supported (see exarp-go `.cursor/rules/exarp-mcp-output.mdc`).

**`task_workflow`**

- `action=list`, `status=Todo`, optional `order=dependency` or `order=execution` for backlog dependency order (`sub_action=list` where required by your client).
- Optional `ready_only` if your tool schema exposes it (tasks not blocked by open dependencies).
- Example JSON args: `{"action":"list","sub_action":"list","status":"Todo","output_format":"json","compact":true,"limit":20}`.

**`task_analysis`**

- `action=execution_plan`, optional `filter_tag` / `filter_tags` to restrict the backlog slice.
- Response includes `waves`, `backlog_count`, and ordered IDs.

**`session`**

- `action=prime`, `include_tasks=true`, `compact=true` for `suggested_next` and context hints.

## Optional product hardening (exarp-go)

If you need a **single canonical “backlog”** export (e.g. always Todo+In Progress, or include Review), that would be a small exarp-go change: align `IsBacklogStatus` / list filters / docs and possibly add a `task_workflow` filter such as `backlog_preset=strict|open|execution`. Not required for normal use.

## Backlog by component

Component-style tags keep filtered views repeatable. When creating tasks, prefer normalized tags such as `tui`, `cli`, `backend`, `api`, `market-data`, `docs`, `build`, `database` (see [`CLOUD_AGENT_BACKLOG_CLERK.md`](./CLOUD_AGENT_BACKLOG_CLERK.md)). **Todo2 may still store hashtags** (for example `#tui`); tools treat tags differently:

| Goal | What to use |
|------|-------------|
| Dependency-ordered slice + tag filter | **`task_analysis`** `action=execution_plan` with **`filter_tag`** or comma-separated **`filter_tags`**. Match is **exact** on the stored string (pass `#tui` if tasks are tagged `#tui`). |
| Raw lists by status | **`task_workflow`** `action=list` with `status`; optional `order=dependency` / `order=execution`. |
| Wave-aligned batch, prefer a component | From repo root: `python3 scripts/parallel_wave_remaining.py <wave> [batch_size] --tag <name>` with `wave` in `0`, `1`, or `2`. **`--tag` ignores a leading `#`**, so `--tag tui` matches `#tui`. Repeat `--tag`; add **`--strict-tag`** to only take matching tasks (batch may be smaller). Waves live under **`waves`** in [`.cursor/plans/parallel-execution-waves.json`](../.cursor/plans/parallel-execution-waves.json) (keys `"0"`, `"1"`, `"2"`). Full runbook: [`WAVE_RESUME_RUNBOOK.md`](./WAVE_RESUME_RUNBOOK.md). |
| Unblocked “suggested next” | **`session`** `action=prime` with tasks included for **`suggested_next`**. |

**Verify `PROJECT_ROOT` before any exarp-go call** (wrong root ⇒ wrong backlog):

```bash
./scripts/run_exarp_go.sh doctor
```

**Examples (CLI tool mode from Aether root):**

```bash
./scripts/run_exarp_go.sh -tool task_analysis -args '{"action":"execution_plan","filter_tag":"#tui"}'
./scripts/run_exarp_go.sh -tool task_analysis -args '{"action":"execution_plan","filter_tags":"#tui,#cli"}'
python3 scripts/parallel_wave_remaining.py 0 15 --tag tui
```

After direct edits to **`.todo2/todo2.db`**, refresh mirrors and IDE views:

```bash
./scripts/run_exarp_go.sh task sync
```

## References

- exarp-go: `internal/models/constants.go` (`IsOpenStatus`), `internal/tools/graph_helpers.go` (`BacklogExecutionOrder`), `internal/tools/task_analysis_deps.go` (`execution_plan`).
- Aether automation rule: `.cursor/rules/project-automation.mdc` (always pass project root for exarp-go tools).
