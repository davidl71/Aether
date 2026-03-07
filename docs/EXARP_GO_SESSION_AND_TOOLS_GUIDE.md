# Using exarp-go: session handoff and other tools

Learn how to use **session handoff** and other exarp-go tools. Patterns are aligned with the **sibling exarp-go repository**; when you have it (e.g. `EXARP_GO_ROOT` or `../exarp-go`), use its docs and hooks as the source of truth.

---

## 1. Where to learn in the sibling exarp-go repo

When the exarp-go repo is available (sibling, or `EXARP_GO_ROOT`), use it for:

| Topic | exarp-go location | This repo reference |
|-------|-------------------|----------------------|
| Session prime | `.cursor/hooks/session-prime.sh` | [EXARP_GO_OPENCORE_AI_CONTEXT_PATTERNS.md](EXARP_GO_OPENCORE_AI_CONTEXT_PATTERNS.md) |
| Session handoff | MCP `session` tool, `action=handoff` | [SESSION_HANDOFF_EXPORT.md](SESSION_HANDOFF_EXPORT.md), `.opencode/commands/handoff.md` |
| Tool catalog / help | MCP `tool_catalog` (action=help, tool_name=…) or CLI `exarp-go -tool …` | This doc, [project-automation.mdc](../.cursor/rules/project-automation.mdc) |
| Git hooks | `scripts/git-hooks/`, `internal/tools/hooks_setup.go` | [EXARP_GO_GIT_HOOKS_LEARNINGS.md](EXARP_GO_GIT_HOOKS_LEARNINGS.md) |
| CLI usage | `exarp-go -tool <name> -args '{"key":"value"}'` | [EXARP_GO_SCRIPTS_AND_PATTERNS.md](EXARP_GO_SCRIPTS_AND_PATTERNS.md) |

**Working directory:** All exarp-go MCP tools expect **this project’s root** (Cursor workspace root). The sibling repo’s server uses that to find `.todo2`, config, etc.

---

## 2. Session handoff

### What it does

- **Prime** (`action=prime`): Loads context at session start — suggested next tasks, task summary, handoff alert from the previous session. Use at the beginning of a conversation or when switching projects.
- **Handoff** (`action=handoff`): Saves a note for the next developer or session — summary, next steps, optional git status and **point-in-time task snapshot** for syncing to another server.

### MCP: session handoff (with export for sync)

Call the **session** tool with:

| Parameter | Value | Purpose |
|-----------|--------|---------|
| `action` | `handoff` | Save handoff (use `prime` to load context instead). |
| `summary` | string | Short note (e.g. "Handoff for sync to other server"). |
| `include_tasks` | `true` | Include task list in handoff. |
| `include_git_status` | `true` | Include branch and changed files. |
| `include_point_in_time_snapshot` | `true` | **Export tasks**: full task list (e.g. gzip+base64) so the other server can restore Todo2. |
| `export_latest` | `true` | Use latest task state when building the snapshot. |

The response is the handoff JSON (including the snapshot). To get a **file** to copy to another server, use the CLI and redirect stdout (see [SESSION_HANDOFF_EXPORT.md](SESSION_HANDOFF_EXPORT.md)).

### MCP: session prime (start of session)

Call **session** with:

- `action`: `prime`
- `include_tasks`: `true`
- `include_hints`: `true` (optional)
- `compact`: `true` (optional, to reduce response size)

You get `suggested_next`, `suggested_next_action`, task counts, and `handoff_alert` if a handoff was left.

### CLI

```bash
# Prime (e.g. in a hook or script)
./scripts/run_exarp_go.sh -tool session -args '{"action":"prime","include_tasks":true}' -json -quiet

# Handoff with task export → write to file for sync
./scripts/run_exarp_go.sh -tool session -args '{"action":"handoff","summary":"Handoff for sync","include_tasks":true,"include_git_status":true,"include_point_in_time_snapshot":true,"export_latest":true}' -json -quiet > build/handoff-export/session-handoff.json
```

### Commands in this repo

- **OpenCode / Claude:** `.opencode/commands/handoff.md`, `.claude/commands/handoff.md` — “Call exarp-go session with action=handoff, summary=…, include_tasks=true, include_git_status=true.”
- **Session prime:** `.cursor/hooks/session-prime.sh` runs prime on session start and injects `additional_context` into Cursor.

---

## 3. Other exarp tools (quick reference)

From the exarp-go workflow guide and tool catalog:

| User intent | Tool | Typical action / usage |
|-------------|------|-------------------------|
| **Tasks:** list, update, create, show, delete | `task_workflow` | `action=sync`, `sub_action=list`; filter by `status_filter`, `filter_tag`. Prefer CLI: `exarp-go task list`, `exarp-go task update T-xxx --new-status Done`. |
| **Suggested next task**, “what should I work on?” | `session` | `action=prime`, `include_tasks=true`. |
| **End session, handoff, list handoffs** | `session` | `action=handoff` (see above); sub_actions for list/resume if supported. |
| **Project overview, scorecard, briefing** | `report` | `action=overview` \| `scorecard` \| `briefing`. |
| **Plan / backlog execution order** | `report` | `action=plan` → generates/updates `.cursor/plans/<name>.plan.md`. |
| **Docs health, git, CI, tools** | `health` | `action=docs` \| `git` \| `cicd` \| `tools`. |
| **Todo2 alignment, duplicates** | `analyze_alignment`, `task_analysis` | Alignment: `action=todo2` \| `prd`. Duplicates: `task_analysis` with `action=duplicates`. |
| **Lint (markdown, shell, etc.)** | `lint` | `action=run`, `linter=auto` or specific (e.g. `markdownlint`, `shellcheck`). |
| **Security / dependencies** | `security` | `action=scan` \| `report`; dependency scanning. |
| **Help for one tool** | `tool_catalog` | `action=help`, `tool_name=session` (or `task_workflow`, `report`, …). |

### Getting help for a tool

Use the MCP **tool_catalog** tool:

- `action`: `help`
- `tool_name`: e.g. `session`, `task_workflow`, `report`, `health`, `task_analysis`

You get a short description, when to use it, and examples.

### Resources (read-only)

Exarp-go exposes resources (e.g. via MCP `read_resource` or `list_resources`):

- `stdio://cursor/skills` — Workflow guide (when to use which tool).
- `stdio://session/status` — Current session context (handoff, current task).
- `stdio://suggested-tasks` — Dependency-ordered tasks ready to start.
- `stdio://tasks/summary` — Task counts by status/priority.
- `stdio://scorecard` — Project scorecard.

---

## 4. Summary

- **Session handoff:** Use **session** with `action=handoff`, `include_tasks=true`, `include_point_in_time_snapshot=true`, and `export_latest=true` to export tasks for syncing to another server. Redirect CLI stdout to a file to get a portable handoff JSON.
- **Session prime:** Use **session** with `action=prime` at session start (or let `.cursor/hooks/session-prime.sh` do it) for suggested next and handoff alert.
- **Other tools:** Use **task_workflow** for Todo2, **report** for overview/scorecard/plan, **health** for docs/git/CI, **tool_catalog** for per-tool help.
- **Sibling repo:** When you have exarp-go cloned, use its `.cursor/hooks/`, docs, and CLI for the authoritative patterns; this doc and the references above stay aligned with it.

## See also

- [SESSION_HANDOFF_EXPORT.md](SESSION_HANDOFF_EXPORT.md) — Export bundle and sync steps
- [EXARP_GO_OPENCORE_AI_CONTEXT_PATTERNS.md](EXARP_GO_OPENCORE_AI_CONTEXT_PATTERNS.md) — Prime, handoff, workingDirectory
- [.cursor/rules/project-automation.mdc](../.cursor/rules/project-automation.mdc) — Full list of automation tools and when to use them
- [.opencode/commands/handoff.md](../.opencode/commands/handoff.md) — Handoff command (session handoff with summary)
