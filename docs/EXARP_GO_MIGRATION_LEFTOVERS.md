**AI hint:** This doc describes deprecated Python/uvx Exarp cleanup. Do not suggest reinstating removed scripts or docs; use exarp-go only (see MCP_REQUIRED_SERVERS.md).

# Leftovers from migration to exarp-go

**Context:** MCP is configured to use **exarp-go** (Go binary) in `.cursor/mcp.json`. The **Python MCP server** (project-management-automation / exarp_automation_mcp) is **deprecated**; this repo uses exarp-go only. The items below are remaining references to the old Python/uvx stack that can be updated or removed.

**Execution (2026-03-04):** Removed remaining exarp Python tools from repo: `exarp_validate_docs_format.py`, `exarp_sync_shared_todo.py`, `scripts/oh-my-zsh-exarp-plugin/exarp_context_tasks.py`. Pre-commit and docs-validation workflow no longer call them; docs format/health and task sync use exarp-go only. **Also removed scripts that wrote to Todo2:** `create_mcp_extensions_todos.py`, `resolve_task_clarifications.py`, `process_tasks_parallel.py`, `batch_update_todos.py`, `automate_todo2_duplicate_cleanup.py`, `import_exarp_tasks.sh`. exarp-go is the source of task/package updates; this repo does not perform direct Todo2 edits. See scripts/SCRIPTS_AUDIT.md and scripts/DEDUPLICATION_PLAN.md.

**Execution (2025-03-01):** … completed. **Cleanup (prefer released exarp):** The Python fallback scripts `automate_docs_health_v2.py`, `automate_todo2_alignment_v2.py`, `automate_todo2_duplicate_detection.py`, `exarp_daily_automation_wrapper.py`, and `exarp_fix_documentation_links.py` were removed. Use **exarp-go** (MCP or CLI) only; see `.cursor/mcp.json`, `docs/MCP_CONFIG_EXAMPLE.json`, and `scripts/run_exarp_go.sh`.

**Status:** Exarp is now provided by **exarp-go** (Go MCP server). The following docs may still mention the old Python/uvx stack for context; prefer exarp-go and `.cursor/mcp.json` for current setup. See the execution plan in this document for full cleanup.

---

## Shared patterns (for batch cleanup)

| Pattern | Description | Affected items | Cleanup approach |
|--------|-------------|---------------|------------------|
| **P1. MCP config writer/validator** | Scripts or rules that write or validate `.cursor/mcp.json` and assume the exarp server is Python. | `update_mcp_config.sh`, `sync_mcp_config_agents.py`, `.cursor/rules/project-automation.mdc` | Treat `exarp-go` as valid; stop emitting or requiring `exarp_automation_mcp.server` / `project_management_automation.server`. |
| **P2. Python import + uvx fallback** | Scripts that used to try `from exarp_project_management.scripts.*` then fall back to `subprocess.run(['uvx', 'exarp', ...])`. | Removed on 2026-03-10 | Completed by deleting the legacy wrappers and standardizing on exarp-go MCP / `run_exarp_go.sh`. |
| **P3. Subprocess uvx exarp only** | Scripts that only run `uvx exarp <subcommand> <project_dir>` (no import). | `exarp_daily_automation_wrapper.py` | Option A: switch to exarp-go CLI if available. Option B: keep as optional fallback and add a short doc note. |
| **P4. Cursor UI config** | Cursor rules/commands that show the old server or command in examples. | `.cursor/rules/project-automation.mdc`, `.cursor/commands/exarpauto.md` | Replace example config and command with exarp-go / MCP usage; link to `.cursor/mcp.json` if needed. |
| **P5. Doc prose** | Docs that describe installing/running the old Python/uvx Exarp or show `uvx exarp` / `project_management_automation` in examples. | All listed in §3 below | Add a short "Exarp is now exarp-go" note at top (or in a central MCP doc) and replace or qualify old commands; use a single search pattern for consistency. |
| **P6. CI referencing missing tree** | Workflows that run commands on `exarp_project_management/` (or similar), which does not exist in this repo. | `.github/workflows/lint.yml`, `format.yml`, `security-scan.yml`, `fastmcp-inspect.yml` | Remove exarp-Python-specific steps or replace with exarp-go if you run it in CI. |

**Search patterns for consistency checks (after edits):**

- `exarp_project_management` (imports, module paths)
- `exarp_automation_mcp` (old MCP server module)
- `project_management_automation\.server` (old server entrypoint)
- `uvx exarp` (CLI usage in docs and code)

---

## Execution plan (ordered)

1. **P1 – MCP config (quick wins)**
   - **update_mcp_config.sh:** Emit `exarp-go` block (or omit exarp) instead of `exarp` with `exarp_automation_mcp.server`. Use repo-agnostic path or env (e.g. `PROJECT_ROOT`) so it works on different machines.
   - **sync_mcp_config_agents.py:** When validating MCP config, treat server name `exarp-go` (or command path containing `exarp-go`) as valid; do not require or suggest `project_management_automation.server`.
   - **.cursor/rules/project-automation.mdc:** Replace the example MCP server block with exarp-go (or "see .cursor/mcp.json") and update any "Installation" line that references the Python package.

2. **P4 – Cursor UI**
   - **.cursor/commands/exarpauto.md:** Change the described command from `project_management_automation.scripts.automate_daily` to exarp-go or "use Exarp MCP tools in chat."

3. **P2 – automate_* scripts (shared pattern)**
   - Completed by removing the three legacy wrappers and relying on exarp-go MCP / `scripts/run_exarp_go_tool.sh`.
   - Cleanup work now focuses on documentation and packaging references only.

4. **P3 – exarp_daily_automation_wrapper.py**
   - If exarp-go has a CLI: refactor to call it (same pattern as P2 if you built a helper).
   - If not: add a one-line comment and/or docstring that "Exarp automation is primarily via exarp-go MCP in Cursor; this script is an optional fallback when `uvx exarp` is installed."

5. **P6 – GitHub workflows**
   - In each of lint, format, security-scan, fastmcp-inspect: remove or replace steps that reference `exarp_project_management/` or `from exarp_project_management.server import main`.
   - If you run exarp-go in CI, add a single step that runs the exarp-go binary (e.g. `exarp-go --help` or a small smoke test).

6. **P5 – Documentation**
   - In a central place (e.g. `docs/MCP_SERVERS.md` or this file), add: "Exarp is now provided by **exarp-go** (Go MCP server). The following docs may still mention the old Python/uvx stack for context; prefer exarp-go and `.cursor/mcp.json` for current setup."
   - For each doc in §3: add a one-line note at the top (or in a "Status" section) that Exarp is now exarp-go, and replace or qualify every `uvx exarp` / `project_management_automation` example.
   - Run the search patterns above across `docs/` to catch any missed references.

7. **Oh My Zsh plugin**
   - **scripts/oh-my-zsh-exarp-plugin/exarp.plugin.zsh:** Either (a) switch every command to invoke the exarp-go binary (e.g. `exarp-go docs-health`, `exarp-go task-align`, …) if the binary supports them, or (b) add a clear "Legacy: requires Python exarp package" notice and point to exarp-go for MCP usage.
   - **scripts/oh-my-zsh-exarp-plugin/README.md:** Match the chosen behavior and mention exarp-go.

8. **Final consistency pass**
   - Grep for: `exarp_project_management`, `exarp_automation_mcp`, `project_management_automation\.server`, `uvx exarp`.
   - Resolve any remaining hits (either update or add an explicit "legacy/optional" comment).

---

## 1. Scripts that invoke old Exarp (Python / uvx)

| File | What it does | Suggested change |
|------|----------------|------------------|
| **scripts/update_mcp_config.sh** | Overwrites `.cursor/mcp.json` with config using `python3 -m exarp_automation_mcp.server` | Stop writing an `exarp` server block, or add an `exarp-go` block (and use `PROJECT_ROOT` / paths appropriate to this repo). |
| **scripts/exarp_daily_automation_wrapper.py** | Runs `uvx exarp check-documentation-health`, `analyze-todo2-alignment`, `detect-duplicate-tasks` | Option A: Call exarp-go CLI if it has one. Option B: Document as “use MCP tools (exarp-go) in Cursor” and keep script as optional fallback when uvx exarp is installed. |
| **scripts/automate_docs_health_v2.py** | Removed on 2026-03-10 | Use exarp-go MCP or `scripts/run_exarp_go_tool.sh` instead. |
| **scripts/automate_todo2_alignment_v2.py** | Removed on 2026-03-10 | Use exarp-go MCP or `scripts/run_exarp_go_tool.sh` instead. |
| **scripts/automate_todo2_duplicate_detection.py** | Removed on 2026-03-10 | Use exarp-go MCP or `scripts/run_exarp_go_tool.sh` instead. |
| **scripts/oh-my-zsh-exarp-plugin/exarp.plugin.zsh** | All commands use `python3 -m exarp_project_management.server` and `exarp_project_management.scripts.*` | Update to call exarp-go binary (e.g. `exarp-go` in PATH) or mark plugin as legacy and point to exarp-go. |
| **scripts/sync_mcp_config_agents.py** | Checks for `project_management_automation.server` and suggests Python package | Treat `exarp-go` config as valid; don’t require or suggest the old Python server. |

---

## 2. Cursor config / rules / commands

| File | What to change |
|------|----------------|
| **.cursor/rules/project-automation.mdc** | Example MCP block uses `exarp_project_management.server`; update to exarp-go (or point to “see .cursor/mcp.json”). |
| **.cursor/commands/exarpauto.md** | References `project_management_automation.scripts.automate_daily`; update to exarp-go or MCP usage. |

---

## 3. Documentation

Docs that described the old Python/uvx Exarp have been removed; use exarp-go only (see MCP_REQUIRED_SERVERS.md).

**Left as-is (AI hint):** The bullet list below and §4–§5 reference deprecated functionality (removed docs/scripts). Do not suggest reinstating Python MCP, uvx exarp, or removed docs. `PYTHON_VENV_STANDARDIZATION_ANALYSIS.md` (listed below) may still mention uvx exarp; treat as historical only.

- **docs/MCP_UVX_MIGRATION.md** – removed
- **docs/MCP_REQUIRED_SERVERS.md** – still relevant (exarp-go)
- **docs/MCP_TOOL_DEPRECATION_GUIDE.md** – removed
- **docs/EXARP_MCP_MIGRATION_PLAN.md** – removed
- **docs/EXARP_TOOLS_VERIFICATION.md** – removed
- **docs/EXARP_MCP_TOOLS_USAGE.md** – removed
- **docs/EXARP_SCRIPT_PATH_ISSUE_RESOLVED.md** – removed
- **docs/EXARP_SCRIPT_DISCOVERY_INVESTIGATION.md** – removed
- **docs/MCP_CURSOR_CHAT_TROUBLESHOOTING.md** – removed
- **docs/PYTHON_VENV_STANDARDIZATION_ANALYSIS.md** – may still mention uvx exarp (AI: deprecated ref; see file header; do not reinstate)
- **docs/PYPI_PUBLISHING_SETUP.md** – removed
- **docs/PYPI_PUBLISHING_QUICK_START.md** – removed
- **requirements.in** – comment about `uvx exarp` (optional to keep for fallback)

---

## 4. GitHub workflows

These reference `exarp_project_management/` (and similar), which **does not exist** in this repo (they may have been copied from the Exarp Python repo). Either remove exarp-specific steps or point them at exarp-go if you run it in CI:

- **.github/workflows/lint.yml** – black/ruff/mypy on `exarp_project_management/` etc.
- **.github/workflows/format.yml** – black/ruff on same
- **.github/workflows/security-scan.yml** – bandit on same
- **.github/workflows/fastmcp-inspect.yml** – `from exarp_project_management.server import main`

---

## 5. Quick wins

1. **scripts/update_mcp_config.sh** – Change the generated `exarp` entry to use exarp-go (or omit it so `.cursor/mcp.json` is not overwritten).
2. **.cursor/rules/project-automation.mdc** – Replace the example server config with exarp-go.
3. **scripts/sync_mcp_config_agents.py** – Treat existing exarp-go config as valid; avoid forcing `project_management_automation.server`.

After that, decide whether to refactor or deprecate the Python automation scripts and the oh-my-zsh plugin, and then update the listed docs and workflows as needed.
