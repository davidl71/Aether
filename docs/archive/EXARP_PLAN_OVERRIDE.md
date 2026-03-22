# Exarp project-specific plan override

Exarp-go supports a **project-specific plan override** so that generated plans use your project’s name and overview instead of the inferred default (e.g. "MCP Server" when run via MCP).

## Why use it

When the plan is generated via MCP (e.g. from Cursor), exarp infers project type from its context and may set **overview** and **Purpose** / **Project type** to "MCP Server". The override lets you pin the plan title and overview per project so regenerating the plan keeps the correct description.

## How to configure

Configure the override in your project so that `report(action=plan)` uses your values. Exact mechanism depends on your exarp-go version:

- **Config file:** Look for a project-level config (e.g. in repo root or `.todo2/`) that exarp-go reads for plan generation. Set the plan title and overview (or project type) there.
- **Report tool parameters:** When calling the report tool with `action=plan`, pass `plan_title` (and any new override parameters your version supports) so the generated plan frontmatter uses them.
- **Upstream docs:** Check the exarp-go repo or changelog for “plan override” or “project-specific plan” for the current API.

## This project’s values

When configuring the override, use:

| Field | Value |
|-------|--------|
| **Plan title** | Aether – Multi-asset financing platform |
| **Overview** | Multi-asset synthetic financing platform (Aether); Rust backend/TUI, box spreads, IBKR integration |
| **Purpose / Project type** | Trading/financing platform (Rust-first); multi-asset synthetic financing |

## Relation to the overview fix script

- **With override:** Once the project-specific plan override is configured and working, plan regeneration should produce the correct overview; `scripts/fix_exarp_plan_overview.sh` may no longer be needed.
- **Without override:** Continue using the script after each plan regeneration, as described in [EXARP_PLAN_OVERVIEW_FIX.md](EXARP_PLAN_OVERVIEW_FIX.md).

## References

- [EXARP_PLAN_OVERVIEW_FIX.md](EXARP_PLAN_OVERVIEW_FIX.md) — script-based fix when override is not in use
- exarp-go report tool: `action=plan`, optional `plan_title`, `plan_path`, `output_path`
