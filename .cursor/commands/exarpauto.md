# Exarp Daily Automation

Run daily automation checks including documentation health, task alignment, duplicate detection, and security scanning.

## Usage

Run Exarp MCP tools in Cursor chat (exarp-go server must be configured in `.cursor/mcp.json`). For example:

- "Check documentation health and create Todo2 tasks for issues"
- "Analyze Todo2 alignment with project goals"
- "Find duplicate Todo2 tasks with 85% similarity"

**Manual fallback** (when exarp-go MCP isn't available and `uvx exarp` is installed):

```bash
cd /path/to/ib_box_spread_full_universal
uvx exarp check-documentation-health . --dry-run
uvx exarp analyze-todo2-alignment .
uvx exarp detect-duplicate-tasks .
```

Or use `scripts/exarp_daily_automation_wrapper.py` which runs all three via `uvx exarp`.

## What It Does

1. **Documentation Health Check** - Validates documentation structure, broken links, format issues
2. **Task Alignment Analysis** - Evaluates Todo2 task alignment with project goals
3. **Duplicate Task Detection** - Finds and reports duplicate tasks
4. **Security Scanning** - Scans dependencies for vulnerabilities (when supported by the server)

## Requirements

- Exarp MCP server (exarp-go) configured in `.cursor/mcp.json`, or
- Optional: `uvx exarp` for script fallback
