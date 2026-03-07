# CI, development, and AI patterns from sibling exarp-go repository

This doc summarizes **CI workflows**, **development patterns**, and **AI usage (skills and rules)** from the sibling [exarp-go](https://github.com/davidl71/exarp-go) repository. Use it when you don’t have the repo cloned or when aligning this project with exarp-go.

**Source:** GitHub `davidl71/exarp-go` (main). When you have the repo (e.g. `EXARP_GO_ROOT` or `../exarp-go`), use its files as the source of truth.

---

## 1. CI patterns (exarp-go)

### 1.1 Workflows

| Workflow | Purpose |
|----------|---------|
| **agentic-ci.yml** | Build + agent validation using exarp-go tools; handoff export in CI. |
| **go.yml** | Go build, test, lint (standard). |

### 1.2 Agentic CI (agentic-ci.yml) — main pattern

- **Trigger:** Push/PR to `main`, `develop`; `workflow_dispatch`.
- **Jobs:**
  1. **build** — Build `exarp-go` binary; upload artifact.
  2. **agent-validation** — Download binary; run exarp-go tools:
     - `./bin/exarp-go -test tool_workflow` (and `automation`, `testing`)
     - `./bin/exarp-go -tool testing -args '{"action":"validate",...}'`
     - `./bin/exarp-go -tool task_workflow -args '{"action":"sync"}'`
     - `./bin/exarp-go -tool task_workflow -args '{"action":"sanity_check"}'`
     - `./bin/exarp-go -tool report -args '{"action":"overview",...}'` → `agent_report.txt`
     - **Handoff export:**  
       `./bin/exarp-go -tool session -args '{"action":"handoff","sub_action":"end",...}'`  
       `./bin/exarp-go -tool session -args '{"action":"handoff","sub_action":"export","output_path":"docs/CI_HANDOFF.json",...}'`
     - Upload artifacts: `agent_report.txt`, `docs/CI_HANDOFF.json`.
  3. **combined-validation** — Combine reports (optional).

- **Takeaways for this repo:**
  - Run exarp-go **tool self-tests** (`-test <tool>`) in CI.
  - Run **task_workflow** `sanity_check` and **report** `overview` for agent validation.
  - Use **session handoff** with `sub_action=end` and `sub_action=export` and `output_path` to produce a CI handoff artifact.

### 1.3 Git hooks (exarp-go)

- **Pre-commit:** Build + health/docs check (no vulnerability scan).
- **Pre-push:** Alignment check only.
- **Before release:** `make pre-release` (build + govulncheck + security scan). See `docs/VULNERABILITY_CHECK_POLICY.md`.

---

## 2. Development patterns (exarp-go)

### 2.1 One-command dev (recommended)

```bash
make dev-full
```

- Auto-reload server on file changes.
- Auto-run tests on file changes.
- Auto-generate coverage.
- Uses `fswatch` (macOS) or `inotifywait` (Linux) or polling fallback.

### 2.2 Other make targets

| Command | Purpose |
|---------|---------|
| `make dev` | Watch only (auto-reload). |
| `make dev-test` | Watch + auto-test (no coverage). |
| `make test-watch` | Test-only watch. |
| `make test` | Run all tests once. |
| `make test-coverage` | Tests with coverage. |
| `make quick-test` | Fast import verification. |
| `make fmt` | Format code. |
| `make lint` | Lint. |
| `make lint-fix` | Lint and auto-fix. |

### 2.3 Automation (daily / sprint)

```bash
exarp-go -tool automation -args '{"action":"daily"}'
# or: action=sprint
```

Python `automate_daily` is removed; use the Go `automation` tool only.

### 2.4 References in exarp-go

- **Quick ref:** `docs/WORKFLOW_USAGE.md`
- **Full:** `docs/DEV_TEST_AUTOMATION.md`
- **Summary:** `docs/STREAMLINED_WORKFLOW_SUMMARY.md`

---

## 3. AI usage: Cursor skills (exarp-go)

Skills live under `.cursor/skills/<name>/SKILL.md`. Cursor uses them when the user’s request matches the skill description.

### 3.1 Skill index (from exarp-go)

| Skill | When to use |
|-------|-------------|
| **task-workflow** | List/update/create/show/delete Todo2 tasks; move statuses. Prefer exarp-go (locking); avoid editing `.todo2` directly. |
| **report-scorecard** | Project overview, scorecard, briefing; after big changes; before reviews. |
| **session-handoff** | End session (create handoff), list handoffs, resume, export handoff data. |
| **task-cleanup** | Bulk remove one-off/performance tasks; batch delete via `task_ids`. |
| **lint-docs** | Broken references, doc links, markdown lint; gomarklint link check. |
| **tractatus-decompose** | Tractatus Thinking MCP for logical decomposition (operation=start, add, export). |
| **thinking-workflow** | Chain tractatus + sequential + exarp-go for backlog/sprint/dependency analysis. |
| **use-exarp-tools** | When to use which exarp-go tool (tasks, reports, health, testing, automation). |
| **text-generate** | Local LLM text generation (fast, on-device). |

### 3.2 Session handoff (skill details)

From exarp-go `.cursor/skills/session-handoff/SKILL.md`:

- **End session:** `action=handoff`, `sub_action=end`, `summary` (required). Optional: `blockers`, `next_steps`, `include_tasks`, `include_git_status`, `unassign_my_tasks`, `include_point_in_time_snapshot`, `modified_task_ids` / `task_journal`.
- **List handoffs:** `sub_action=list`, optional `limit`.
- **Resume:** `sub_action=resume` (latest handoff).
- **Export:** `sub_action=export`, optional `output_path`, `export_latest`.
- **Storage:** `.todo2/handoffs.json` (last 20). Prime shows handoff alerts from other hosts.

### 3.3 Use-exarp-tools (skill summary)

- **Suggested next:** `session` with `action=prime`, `include_tasks=true`, `include_hints=true` → `suggested_next`.
- **Tasks:** `task_workflow` (or `exarp-go task` CLI).
- **Overview/scorecard/briefing:** `report` with `action=overview|scorecard|briefing`.
- **Docs/CI/repo:** `health` with `action=docs|git|cicd`.
- **Task branches, merge, history:** `git_tools` with `action=commits|branches|tasks|diff|graph|merge|set_branch`.
- **Task analysis:** `task_analysis` (dependencies, duplicates, conflicts, execution_plan, etc.).
- **Tool help:** `tool_catalog` with `action=help`, `tool_name=<name>`; or resources `stdio://tools`, `stdio://cursor/skills`.
- **Do not** run `exarp-go --help` to discover tools; use MCP resources or tool_catalog.

### 3.4 Multi-agent: locking and conflicts

- **Task locking:** Use exarp-go for Todo → In Progress so `ClaimTaskForAgent` is used; avoid Todo2 MCP `update_todos` for that in multi-agent setups.
- **Conflict detection:** `task_analysis` with `action=conflicts` for overlapping In Progress tasks.
- **Rule:** `.cursor/rules/agent-locking.mdc` (always-apply).

---

## 4. Cursor rules (exarp-go)

From `docs/CURSOR_RULES.md`:

| Rule | Description | When |
|------|-------------|------|
| **agent-locking.mdc** | Task locking, agent ID, parallel execution | Always |
| **agentic-ci.mdc** | Agentic CI workflow and validation | Always |
| **code-and-planning-tag-hints.mdc** | Tag hints in plans and Go code | Plans (`.plan.md`, `docs/*_PLAN*.md`), Go (`**/*.go`) |
| **plan-todos-required.mdc** | Todos with task IDs in plans | `**/*.plan.md`, `.cursor/plans/**` |
| **go-development.mdc** | Go style, Makefile, Todo2 DB, testing | Go files, key sections |
| **llm-tools.mdc** | LLM backend discovery and tool choice | Always |
| **mcp-configuration.mdc** | MCP config; Context7 vs web search | Always |
| **session-prime.mdc** | Session priming at conversation start | Always |
| **todo2.mdc** | Todo2 workflow, research, lifecycle | Always |
| **todo2-overview.mdc** | Auto-generated task overview | Always |
| **task-discovery.mdc** | task_discovery behavior; deprecated (strikethrough/removed) never create tasks | — |

### 4.1 Code and planning tag hints

- **Plans:** In top or frontmatter add tag hints, e.g. `**Tag hints:** \`#migration\` \`#cli\`` or `tag_hints: [migration, cli]`.
- **Go:** One file/package-level hint, e.g. `// exarp-tags: #migration #cli`.
- **Canonical tags (examples):** `#migration` `#refactor` `#cli` `#mcp` `#testing` `#docs` `#security` `#build` `#planning` …

---

## 5. Where to find what in exarp-go (sync checklist)

When you have the sibling repo:

| What | exarp-go path |
|------|----------------|
| CI workflows | `.github/workflows/agentic-ci.yml`, `.github/workflows/go.yml` |
| Dev workflow | `Makefile` (dev-full, test, lint), `dev.sh`, `docs/WORKFLOW_USAGE.md`, `docs/DEV_TEST_AUTOMATION.md` |
| Session prime | `.cursor/hooks/session-prime.sh` |
| Session handoff | MCP `session` tool; `.cursor/skills/session-handoff/SKILL.md` |
| Skills index | `.cursor/skills/README.md` |
| Skills (each) | `.cursor/skills/<name>/SKILL.md` |
| Cursor rules | `.cursor/rules/*.mdc` |
| Agent locking | `.cursor/rules/agent-locking.mdc` |
| Cursor docs | `docs/CURSOR_RULES.md`, `docs/CURSOR_SKILLS_GUIDE.md` |
| Config | `.exarp/config.pb`; `docs/CONFIGURATION_REFERENCE.md` |

---

## 5.1 Editor-agnostic usage (Cursor, Claude Code, OpenCode)

The same exarp-go behaviors (session prime, handoff, tasks, report) work in **Cursor**, **Claude Code**, and **OpenCode**. Each editor uses the same MCP tools or CLI; only the trigger (command name or natural language) differs. See [EXARP_GO_CURSOR_CLAUDE_OPENCODE.md](EXARP_GO_CURSOR_CLAUDE_OPENCODE.md) for command parity and handoff-with-export. Do not rely on `exarp-go --help` for discovery; use the doc and MCP tool names/args so behavior is identical across editors.

---

## 6. Applying to this repo (ib_box_spread_full_universal)

- **CI:** Consider adding an agent-validation job that runs exarp-go `-test` and `task_workflow sanity_check`, and optionally session handoff export (see [SESSION_HANDOFF_EXPORT.md](SESSION_HANDOFF_EXPORT.md)).
- **Skills:** This repo has `.cursor/skills/` (e.g. when-to-use-subagents, trading-safety); you can add or align skills from exarp-go (task-workflow, session-handoff, use-exarp-tools, report-scorecard, etc.) by copying or adapting `.cursor/skills/<name>/SKILL.md`.
- **Rules:** Copy or adapt exarp-go rules (e.g. session-prime, code-and-planning-tag-hints, agent-locking if you run multi-agent) into `.cursor/rules/` and reference them in a rules index.
- **Handoff:** Use `session` with `action=handoff`, `sub_action=end` or `sub_action=export`, and `output_path` for CI or sync; see [EXARP_GO_SESSION_AND_TOOLS_GUIDE.md](EXARP_GO_SESSION_AND_TOOLS_GUIDE.md) and [SESSION_HANDOFF_EXPORT.md](SESSION_HANDOFF_EXPORT.md).

---

## See also

- [EXARP_GO_CURSOR_CLAUDE_OPENCODE.md](EXARP_GO_CURSOR_CLAUDE_OPENCODE.md) — exarp-go usage across Cursor, Claude Code, OpenCode
- [EXARP_GO_SESSION_AND_TOOLS_GUIDE.md](EXARP_GO_SESSION_AND_TOOLS_GUIDE.md) — Session handoff and tools in this repo
- [SESSION_HANDOFF_EXPORT.md](SESSION_HANDOFF_EXPORT.md) — Handoff export for sync
- [EXARP_GO_OPENCORE_AI_CONTEXT_PATTERNS.md](EXARP_GO_OPENCORE_AI_CONTEXT_PATTERNS.md) — Prime, handoff, workingDirectory
- [.cursor/rules/project-automation.mdc](../.cursor/rules/project-automation.mdc) — Automation tools in this repo
- exarp-go: `docs/CURSOR_SKILLS_GUIDE.md`, `docs/CURSOR_RULES.md`, `docs/WORKFLOW_USAGE.md`, `README.md`
