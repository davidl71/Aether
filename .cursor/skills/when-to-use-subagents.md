# Skill: When to use subagents

**When:** User asks for a code review, refactor, tests for C++, trading/financial audit, task/report/docs automation, or broad codebase exploration.

**Do:**

1. **Code review (general)** → Suggest **mcp_task** with **code-reviewer**. Attach the file(s) to review. In the prompt include: project root path, “Canonical guidelines: AGENTS.md and CLAUDE.md in repo root.”
2. **Trading/risk/order code review** → Suggest **mcp_task** with **trading-reviewer**. Same context (project root + AGENTS.md/CLAUDE.md). Attach the relevant C++/Python files.
3. **Safe refactor with test verification** → Suggest **mcp_task** with **refactor**. Describe the refactor and that tests must pass after each step.
4. **Generate Catch2 tests for a C++ file** → Suggest **mcp_task** with **test-writer**. Attach the source file (and optionally its header). Mention project root and canonical guidelines.
5. **Search / explore / multi-step investigation** → Suggest **mcp_task** with **generalPurpose**. Give a clear task and project root + canonical guidelines.
6. **Todo2 tasks, docs health, scorecard, security scan** → Use **exarp-go** MCP tools (e.g. `task_workflow`, `report`, `health`, `security`). Always pass **workingDirectory** = this project’s root (workspace root in Cursor).

**Reference:** docs/SUBAGENTS_REFERENCE.md, .cursor/rules/ai-editors-skills-subagents.mdc.
