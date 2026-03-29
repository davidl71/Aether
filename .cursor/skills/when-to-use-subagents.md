# Skill: When to use subagents

**When:** User asks for a code review, refactor, test generation, trading/financial audit, task/report/docs automation, or broad codebase exploration.

**Do:**

1. **Code review (general)** → Suggest **mcp_task** with **code-reviewer**. Attach the file(s) to review. In the prompt include: project root path, “Canonical guidelines: AGENTS.md and CLAUDE.md in repo root.”
2. **Trading/risk/order code review** → Suggest **mcp_task** with **trading-reviewer**. Same context (project root + AGENTS.md/CLAUDE.md). Attach the relevant Rust/Python files.
3. **Safe refactor with test verification** → Suggest **mcp_task** with **refactor**. Describe the refactor; verification is **after coherent batches** of edits (not necessarily after every tiny change).
4. **Generate tests for an active file** → Suggest **mcp_task** with **test-writer**. Attach the active Rust or Python source file. Mention project root and canonical guidelines.
5. **Search / explore / multi-step investigation** → Suggest **mcp_task** with **generalPurpose**. Give a clear task and project root + canonical guidelines.
6. **Todo2 tasks, docs health, scorecard, security scan** → Use **exarp-go** MCP tools (e.g. `task_workflow`, `report`, `health`, `security`). Prefer the repo wrapper `scripts/run_exarp_go.sh` (aka `exarp_go`) or `scripts/run_exarp_go_tool.sh -tool <name> -args '<json>'` so the CLI always sees this project’s `PROJECT_ROOT` and migration configuration. When calling the MCP tools directly, still pass **workingDirectory** = this project’s root (workspace root in Cursor).

**Reference:** docs/archive/SUBAGENTS_REFERENCE.md, .cursor/rules/ai-editors-skills-subagents.mdc.
