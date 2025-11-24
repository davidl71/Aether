# automa MCP Server

**Quick Reference**: The **automa** MCP server is the configured name for the Project Management Automation MCP server in this repository.

## What is automa?

**automa** is the MCP server identifier configured in `.cursor/mcp.json` for the self-hosted project management automation tools located in `mcp-servers/project-management-automation/`.

## Why "automa"?

The server is configured with the shorter identifier "automa" for:
- **Easier reference** in Cursor prompts and documentation
- **Shorter name** than "project-management-automation"
- **Clear identity** as the automation server

## Server Location

- **Directory**: `mcp-servers/project-management-automation/`
- **MCP Identifier**: `automa` (in `.cursor/mcp.json`)
- **Entry Point**: `mcp-servers/project-management-automation/run_server.sh`

## Configuration

The server is configured in `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "automa": {
      "command": "/Users/davidl/Projects/Trading/ib_box_spread_full_universal/mcp-servers/project-management-automation/run_server.sh",
      "args": [],
      "description": "Project management automation tools - documentation health, task alignment, duplicate detection, security scanning, and automation opportunities"
    }
  }
}
```

## Available Resources

The automa server exposes these resources for quick access to cached data:

1. **`automation://status`** - Server status and health
2. **`automation://history`** - Execution history
3. **`automation://tools`** - Available tools list
4. **`automation://tasks`** - Cached Todo2 task list (filterable by agent/status)
5. **`automation://tasks/agent/{agent_name}`** - Tasks for specific agent
6. **`automation://tasks/status/{status}`** - Tasks by status
7. **`automation://agents`** - List of agents with configurations and task counts
8. **`automation://cache`** - Cache status and metadata

## Available Tools

The automa server provides:

1. **Documentation Health** - `check_documentation_health_tool`
2. **Task Alignment** - `analyze_todo2_alignment_tool`
3. **Duplicate Detection** - `detect_duplicate_tasks_tool`
4. **Security Scanning** - `scan_dependency_security_tool`
5. **Automation Discovery** - `find_automation_opportunities_tool`
6. **Todo Sync** - `sync_todo_tasks_tool`
7. **PWA Review** - `review_pwa_config_tool`
8. **Daily Automation** - `run_daily_automation_tool`
9. **Nightly Automation** - `run_nightly_task_automation_tool`
10. **Working Copy Health** - `check_working_copy_health_tool`
11. **CI/CD Validation** - `validate_ci_cd_workflow_tool`
12. **Batch Approval** - `batch_approve_tasks_tool`
13. **External Tool Hints** - `add_external_tool_hints_tool`

## Usage

When working with Cursor AI assistants, you can reference "automa" tools:

- "Check documentation health using automa"
- "Find duplicate tasks with automa"
- "Scan dependencies for security issues using automa"
- "Run daily automation tasks via automa"

## Documentation

- **Detailed Usage**: `.cursor/rules/project-automation.mdc`
- **Server README**: `mcp-servers/project-management-automation/README.md`
- **MCP Servers Guide**: `docs/research/integration/MCP_SERVERS.md`
- **Quick Reference**: `docs/MCP_QUICK_REFERENCE.md`

## For New Cursor Agents

When setting up new Cursor agents or worktrees:

1. The **automa** server is already configured in `.cursor/mcp.json`
2. The server code is in `mcp-servers/project-management-automation/`
3. Reference it as **"automa"** in prompts and documentation
4. See `.cursor/rules/project-automation.mdc` for detailed tool usage

## Directory Naming

**No renaming needed**: The directory `mcp-servers/project-management-automation/` is descriptive and appropriate. The "automa" identifier is just the MCP server name in the configuration, which is a common pattern (shorter identifier for easier reference).

---

**Last Updated**: 2025-11-24
**Status**: Active and Configured
