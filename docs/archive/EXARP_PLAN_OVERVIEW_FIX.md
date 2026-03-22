# Exarp plan overview fix

## Why the plan shows "MCP Server"

When exarp-go runs as an MCP server (e.g. from Cursor), it infers the project type from its own context and sets the plan **overview** and **Purpose** to "MCP Server" instead of reading the repo (AGENTS.md, README). So after regenerating the plan, the overview is wrong until fixed.

**Project-specific plan override:** Exarp-go has a project-specific plan override feature so generated plans can use your project name and overview. If you enable it, you may not need this script. See [EXARP_PLAN_OVERRIDE.md](EXARP_PLAN_OVERRIDE.md).

## Making it stick

1. **Regenerate the plan** with a proper title (optional but recommended):
   ```bash
   exarp-go -tool report -args '{"action":"plan","plan_title":"Aether – Multi-asset financing platform"}'
   ```
   Or use the MCP report tool with `action=plan` and `plan_title=Aether – Multi-asset financing platform`.

2. **Run the overview fix script** from the repo root:
   ```bash
   ./scripts/fix_exarp_plan_overview.sh
   ```
   This patches all `.cursor/plans/*.plan.md` files: it replaces `overview: "MCP Server"` and the Scope **Purpose** / **Project type** with the real project description (Aether / multi-asset financing platform).

3. Optionally **wire the script** into your workflow (e.g. a Cursor command or a post-step after "regenerate plan") so you don’t have to remember to run it.

## Script location

- **Script:** `scripts/fix_exarp_plan_overview.sh`
- **Plan dir:** `.cursor/plans/`

The script is idempotent: safe to run multiple times; it only changes lines that still say "MCP Server".
