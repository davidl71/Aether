# Exarp: Run Tool

Run a specific exarp-go tool. Default: lint.

## Usage

From project root:

```bash
# Default: lint
./scripts/run_exarp_go_tool.sh

# Specific tool (e.g. testing, security, task_workflow)
./scripts/run_exarp_go_tool.sh testing
./scripts/run_exarp_go_tool.sh security
```

Or with Just: `just exarp lint`, `just exarp testing`, `just exarp-lint`

In Cursor chat use the exarp-go MCP tools (e.g. task_workflow, report, session, lint).
