# exarp-go usage: Cursor, Claude Code, and OpenCode

This doc describes how to use **exarp-go** (session prime, handoff, tasks, reports, export for sync) in a way that works across **Cursor**, **Claude Code**, and **OpenCode**. All three editors can call the same MCP tools or CLI; only the invocation method differs.

**Canonical context:** [AGENTS.md](../AGENTS.md), [CLAUDE.md](../CLAUDE.md). Exarp-go uses **PROJECT_ROOT** (workspace root) for `.todo2`, config, and plans.

---

## 1. Command parity (exarp-go)

| Action | Cursor | Claude Code | OpenCode | MCP / CLI |
|--------|--------|-------------|----------|-----------|
| **Prime session** | Session-prime hook or ask AI to call session | Command: `prime` | Command: `prime-context` or `ai-context` | `session` with `action=prime`, `include_tasks=true`, `include_hints=true` |
| **Handoff** | Ask AI or use handoff command | Command: `handoff` | Command: `handoff` | `session` with `action=handoff`, `summary=…`, `include_tasks=true`, `include_git_status=true` |
| **Handoff + export for sync** | Ask AI or run CLI (see below) | Same MCP args or CLI | Same MCP args or CLI | Add `include_point_in_time_snapshot=true`, `export_latest=true`; for file use CLI and redirect stdout |
| **List tasks** | Ask AI or exarp command | Command: `tasks` | Command: `tasks` | `task_workflow` with `action=sync`, `sub_action=list` |
| **Scorecard / overview** | Ask AI or command | Command: `scorecard` | Command: `scorecard` | `report` with `action=overview` or `action=scorecard` |
| **Build / test / lint** | See [AI_EDITOR_SETUP.md](AI_EDITOR_SETUP.md) | Same | Same | Shell: `ninja -C build`, `ctest`, `./scripts/run_linters.sh` |

---

## 2. Per-editor invocation

### 2.1 Cursor

- **MCP:** exarp-go is configured in `.cursor/mcp.json`. The AI can call tools by name (e.g. `mcp_exarp-go_session`, `mcp_exarp-go_task_workflow`). Pass **workingDirectory** = workspace root when the tool supports it (Cursor often injects this).
- **Commands:** `.cursor/commands.json` defines build, test, lint, exarp. Use command palette or slash commands.
- **Prime:** `.cursor/hooks/session-prime.sh` runs on session start and calls exarp-go session prime via `scripts/run_exarp_go.sh`.
- **Skills:** `.cursor/skills/` — use or @-mention skill files; exarp-go workflow guide is in resource `stdio://cursor/skills`.

**To prime:** Start a new Composer conversation (hook runs) or ask: “Prime my session with exarp” / “What should I work on next?”

**To handoff:** Ask: “Create a handoff with summary: …” or use handoff command if defined.

### 2.2 Claude Code

- **MCP:** Add exarp-go to Claude Code’s MCP config; set `PROJECT_ROOT` (or equivalent) to the workspace root.
- **Commands:** `.claude/commands/` — each `.md` describes what to do (e.g. call exarp-go session with action=handoff). Claude runs the instructions in the command file.

**Commands available:**

| Command | File | What it does |
|---------|------|---------------|
| `prime` | `.claude/commands/prime.md` | Call session with `action=prime`, `include_tasks=true`, `include_hints=true`; summarize suggested next and handoff alert. |
| `handoff` | `.claude/commands/handoff.md` | Call session with `action=handoff`, `summary=$ARGUMENTS`, `include_tasks=true`, `include_git_status=true`. |
| `tasks` | `.claude/commands/tasks.md` | Call task_workflow with `action=sync`, `sub_action=list`; optional status filter. |
| `scorecard` | `.claude/commands/scorecard.md` | Call report with `action=overview` (or scorecard); add local metrics. |

**To prime:** Run the `prime` command (e.g. from command palette or “Run command: prime”).

**To handoff:** Run the `handoff` command and provide a summary when prompted (or pass it as argument).

### 2.3 OpenCode

- **MCP:** OpenCode can load exarp-go, but keep it disabled for now in `opencode.json` if GPT-5 / Sonnet reports `custom.input_schema` JSON Schema errors from the exarp-go tool catalog.
- **Commands:** `.opencode/commands/` — markdown files with READ/RUN instructions. In this repo, OpenCode commands now call the local `./scripts/run_exarp_go.sh` wrapper instead of invoking exarp-go MCP tools directly.

**Commands available:**

| Command | File | What it does |
|---------|------|---------------|
| `prime-context` | `.opencode/commands/prime-context.md` | Load AGENTS.md, ARCHITECTURE.md, etc. |
| `ai-context` | `.opencode/commands/ai-context.md` | Load AGENTS.md, CLAUDE.md, ARCHITECTURE.md. |
| `handoff` | `.opencode/commands/handoff.md` | Run `./scripts/run_exarp_go.sh -tool session` with `action=handoff`, `summary=$ARGUMENTS`, `include_tasks=true`, `include_git_status=true`. |
| `tasks` | `.opencode/commands/tasks.md` | Run `./scripts/run_exarp_go.sh -tool task_workflow` with `action=sync`, `sub_action=list`; optional status filter. |
| `scorecard` | `.opencode/commands/scorecard.md` | Run `./scripts/run_exarp_go.sh -tool report` with `action=overview`; add local metrics. |

**To prime:** Run `prime-context` or `ai-context`; for exarp suggested-next, ask the AI to run `./scripts/run_exarp_go.sh -tool session -args '{"action":"prime","include_tasks":true,"include_hints":true}' -json -quiet`.

**To handoff:** Run the `handoff` command; provide summary as argument or when asked.

---

## 3. Handoff with export for sync (all editors)

To create a handoff that includes a **task snapshot for syncing to another server**:

**MCP (any editor):** Call `session` with:

- `action`: `handoff`
- `summary`: your note
- `include_tasks`: `true`
- `include_git_status`: `true`
- `include_point_in_time_snapshot`: `true`
- `export_latest`: `true`

Optional for “end session” semantics: `sub_action`: `end`. Optional for file export: use CLI and redirect (see below).

**CLI (editor-agnostic):** Run from project root:

```bash
./scripts/run_exarp_go.sh -tool session -args '{"action":"handoff","summary":"Handoff for sync","include_tasks":true,"include_git_status":true,"include_point_in_time_snapshot":true,"export_latest":true}' -json -quiet > build/handoff-export/session-handoff.json
```

Then copy `build/handoff-export/` to the other server (see [SESSION_HANDOFF_EXPORT.md](SESSION_HANDOFF_EXPORT.md)).

**In Claude Code / OpenCode:** You can add an optional “handoff-export” command that uses the same MCP args with `include_point_in_time_snapshot=true` and `export_latest=true`, and optionally tells the user to run the CLI to write a file.

---

## 4. Suggested-next and task list (all editors)

- **Suggested next tasks:** Call `session` with `action=prime`, `include_tasks=true`, `include_hints=true`. The response includes `suggested_next` (dependency-ordered) and `suggested_next_action`.
- **Task list:** Call `task_workflow` with `action=sync`, `sub_action=list`. Optional: `status_filter=Todo` (or `In Progress`), `limit=20`, `order=execution`.

---

## 5. Report and scorecard (all editors)

- **Overview / scorecard:** Call `report` with `action=overview` or `action=scorecard`. Add local metrics (e.g. file counts, test counts) if the command mentions them.
- **Briefing:** `action=briefing` for standup-style summary.
- **Plan:** `action=plan` to generate or update `.cursor/plans/<name>.plan.md`.

---

## 6. Do not depend on editor-specific discovery

- **Do not** run `exarp-go --help` or `exarp-go -list` in the chat to “discover” tools; that spawns the server and may not be configured for your project. Prefer MCP tool calls or the [EXARP_GO_SESSION_AND_TOOLS_GUIDE.md](EXARP_GO_SESSION_AND_TOOLS_GUIDE.md) and [EXARP_GO_SIBLING_CI_AND_AI_PATTERNS.md](EXARP_GO_SIBLING_CI_AND_AI_PATTERNS.md).
- **Do** use the same tool names and JSON args in every editor; only the way the user triggers the call (command name, slash command, or natural language) differs.

---

## 7. Summary

| Goal | Cursor | Claude Code | OpenCode |
|------|--------|-------------|----------|
| Get suggested next + handoff alert | Session-prime hook or ask AI | Run `prime` command | Run `prime-context`; ask AI to call session prime if needed |
| Save handoff for next session | Ask AI or handoff command | Run `handoff` command | Run `handoff` command |
| Handoff + task export for sync | Ask AI or CLI redirect | Same MCP args or CLI | Same MCP args or CLI |
| List tasks | Ask AI or tasks command | Run `tasks` command | Run `tasks` command |
| Project scorecard | Ask AI or scorecard command | Run `scorecard` command | Run `scorecard` command |

All three editors rely on **AGENTS.md** and **CLAUDE.md** for project rules; exarp-go provides task and session state. Configure MCP with **PROJECT_ROOT** (or equivalent) set to the workspace root so exarp-go finds `.todo2` and plans.

## See also

- [SESSION_HANDOFF_EXPORT.md](SESSION_HANDOFF_EXPORT.md) — Export bundle and sync to another server
- [EXARP_GO_SESSION_AND_TOOLS_GUIDE.md](EXARP_GO_SESSION_AND_TOOLS_GUIDE.md) — Session handoff and tools in detail
- [EXARP_GO_SIBLING_CI_AND_AI_PATTERNS.md](EXARP_GO_SIBLING_CI_AND_AI_PATTERNS.md) — CI, dev, and AI patterns from exarp-go
- [AI_EDITOR_SETUP.md](AI_EDITOR_SETUP.md) — Canonical context and command parity (build, test, lint)
