# Exarp Daily Automation

Run project automation via **exarp-go** (released/installed). Prefer the exarp-go MCP server in Cursor.

## Usage

1. **In Cursor** – exarp-go is configured in `.cursor/mcp.json` to use the sibling repo’s runner (`../../mcp/exarp-go/scripts/run_exarp_go.sh`). See `docs/MCP_CONFIG_EXAMPLE.json`. In chat you can ask:
   - "Check documentation health and create Todo2 tasks for issues"
   - "Analyze Todo2 alignment with project goals"
   - "Find duplicate Todo2 tasks"

2. **CLI** – With exarp-go on PATH (e.g. `go install` or system install):
   ```bash
   cd /path/to/ib_box_spread_full_universal
   exarp-go task sync
   ./scripts/run_exarp_go_tool.sh lint
   ```
   The tool script uses this project’s `scripts/run_exarp_go.sh` (or set `EXARP_GO_ROOT` / PATH so exarp-go is found).

## What It Does

- **Documentation health** – Validate docs, broken links, format
- **Task alignment** – Todo2 vs project goals
- **Duplicate detection** – Find duplicate tasks
- **Security scanning** – Dependency vulnerabilities (when supported)

## Requirements

- exarp-go installed and on PATH, or `EXARP_GO_ROOT` set, or a runner script (this project’s `scripts/run_exarp_go.sh` or the sibling `../../mcp/exarp-go/scripts/run_exarp_go.sh`) that finds the binary. See `docs/PORTABLE_BUILD_AND_RUNNER.md` and `docs/MCP_REQUIRED_SERVERS.md`.
