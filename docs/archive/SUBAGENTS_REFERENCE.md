# Subagents you can use

Subagents are specialized agents you can launch from Cursor (or Claude Code) for focused tasks. **Always pass project context** when launching: project root path and “Canonical guidelines: AGENTS.md and CLAUDE.md in repo root.”

---

## 1. Cursor `mcp_task` (built-in subagents)

Invoke via the **Task** tool in Cursor. Use when you want a dedicated agent to run in a separate process with a clear prompt and optional file list.

| Subagent | When to use | Example prompt |
|----------|-------------|----------------|
| **code-reviewer** | Review recently written or modified code for quality, performance, maintainability, best practices | “Review native/src/order_manager.cpp for safety and style. Project root: <path>. Canonical guidelines: AGENTS.md and CLAUDE.md in repo root.” |
| **trading-reviewer** | Audit trading/financial code: pricing, risk, order execution, safety, regulatory | “Review native/src/risk_calculator.cpp for pricing correctness and risk edge cases. Project root: <path>. Canonical guidelines: AGENTS.md and CLAUDE.md.” |
| **refactor** | Safe, incremental refactors (C++/Python) with test verification each step | “Refactor config_manager to use std::expected; verify tests after each change. Project root: <path>. Canonical guidelines: AGENTS.md and CLAUDE.md.” |
| **test-writer** | Generate Catch2 tests for C++ modules | “Write Catch2 tests for native/src/margin_calculator.cpp. Project root: <path>. Canonical guidelines: AGENTS.md and CLAUDE.md.” |
| **generalPurpose** | Broad exploration, search, multi-step tasks | “Find all call sites of place_order and list validation paths. Project root: <path>. Canonical guidelines: AGENTS.md and CLAUDE.md.” |

**Tip:** Attach the specific files you care about (e.g. `attachments: ["native/src/order_manager.cpp"]`) so the subagent has the right context.

---

## 2. exarp-go (MCP)

exarp-go runs as an MCP server (see `.cursor/mcp.json`). It doesn’t run as a separate “subagent” process but provides **tools** you can call for tasks, reports, and health. Use when you want task lists, docs health, alignment, or security scans for this project.

| Need | Tool / usage |
|------|----------------|
| Task list / create / update / show | `task_workflow` (or exarp-go CLI: `exarp-go task list`, etc.) |
| Project overview, scorecard, briefing | `report` with `action=overview`, `action=scorecard`, or `action=briefing` |
| Docs health, git, CI status | `health` with appropriate `action` |
| Session context at conversation start | `session` with `action=prime`, `include_hints=true`, `include_tasks=true` |
| Security scan (multi-language) | `security` with `action=scan` |
| Duplicate task detection | `task_analysis` with `action=duplicates` |
| Task alignment | `analyze_alignment` with `action=todo2` |

**Required:** Set `workingDirectory` (or equivalent) to **this project’s root** for all exarp-go tool calls. See `.cursor/rules/project-automation.mdc` and `.cursor/commands/exarpauto.md`.

---

## 3. Claude custom agents (Claude Code)

Used in **Claude Code** (not Cursor). Same roles as the Cursor `mcp_task` specialists; pick the tool that matches your editor.

| Agent | File | Purpose |
|-------|------|---------|
| code-reviewer | `.claude/agents/code-reviewer.md` | Code quality, performance, maintainability |
| test-writer | `.claude/agents/test-writer.md` | Catch2 tests for C++ |
| trading-reviewer | `.claude/agents/trading-reviewer.md` | Financial correctness, safety, order/risk logic |
| refactor | `.claude/agents/refactor.md` | Incremental refactoring with test verification |
| build-investigator | `.claude/agents/build-investigator.md` | Diagnose and fix build/compile/linker/test failures |
| docs-writer | `.claude/agents/docs-writer.md` | Write or update module docs, API docs, architecture notes |
| exploration | `.claude/agents/exploration.md` | Map call graphs, trace data flow, answer architectural questions |

---

## 4. .cursor/agents (task briefs, not runnable subagents)

`.cursor/agents/` holds **task briefs** for parallel work (e.g. scripts-dx, protobuf-justfile, build-toolchain). They describe scope and ownership for a human or AI; they are not invoked as subagents. Use them when planning or delegating work, not as a “launch agent” target.

---

## Quick decision guide

- **“Review this C++/trading code”** → `mcp_task` **code-reviewer** or **trading-reviewer** (or Claude agent in Claude Code).
- **“Refactor X with tests passing each step”** → `mcp_task` **refactor** (or Claude refactor agent).
- **“Write Catch2 tests for X”** → `mcp_task` **test-writer** (or Claude test-writer agent).
- **”Search/explore the codebase or trace data flow”** → Claude **exploration** agent or `mcp_task` **generalPurpose**.
- **”Build or tests are broken”** → Claude **build-investigator** agent.
- **”Write or update docs for a module”** → Claude **docs-writer** agent.
- **”List/update Todo2 tasks, docs health, scorecard”** → **exarp-go** MCP tools (with workingDirectory = project root).

See also: `docs/archive/AI_EDITOR_SETUP.md`, `.cursor/rules/ai-editors-skills-subagents.mdc`, `.cursor/rules/ai-context-standards.mdc`.
