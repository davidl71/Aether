# Update MCP Configuration to Use Virtual Environment

## Issue
Cursor is running the MCP server with system Python, which doesn't have `fastmcp` installed. The server needs to use the virtual environment's Python.

## Solution
Update `.cursor/mcp.json` to use the wrapper script that activates the virtual environment.

## Current Configuration
```json
{
  "mcpServers": {
    "project-management-automation": {
      "command": "python3",
      "args": [
        "/home/david/ib_box_spread_full_universal/mcp-servers/project-management-automation/server.py"
      ],
      "description": "Project management automation tools. ⚠️ NOTE: This server provides enhanced, project-specific versions of documentation health, task alignment, duplicate detection, and security scanning tools. Prefer these tools over generic MCP server tools for this project."
    }
  }
}
```

## Updated Configuration
```json
{
  "mcpServers": {
    "project-management-automation": {
      "command": "/home/david/ib_box_spread_full_universal/mcp-servers/project-management-automation/run_server.sh",
      "args": [],
      "description": "Project management automation tools. ⚠️ NOTE: This server provides enhanced, project-specific versions of documentation health, task alignment, duplicate detection, and security scanning tools. Prefer these tools over generic MCP server tools for this project."
    }
  }
}
```

## Steps to Update

1. Open `.cursor/mcp.json` in your editor
2. Find the `project-management-automation` entry
3. Change:
   - `"command": "python3"` → `"command": "/home/david/ib_box_spread_full_universal/mcp-servers/project-management-automation/run_server.sh"`
   - `"args": ["/home/david/ib_box_spread_full_universal/mcp-servers/project-management-automation/server.py"]` → `"args": []`
4. Save the file
5. **Restart Cursor completely** (not just reload)

## Verification

After restarting, check the MCP server logs. You should see:
- ✅ "FastMCP available from fastmcp package - using FastMCP server"
- ✅ "FastMCP server initialized"
- ✅ "All tools loaded successfully"

Instead of:
- ❌ "MCP not available - Phase 2 tools complete, install MCP to enable server"

## Alternative: Direct Python Path

If the wrapper script doesn't work, you can also use the venv Python directly:

```json
{
  "mcpServers": {
    "project-management-automation": {
      "command": "/home/david/ib_box_spread_full_universal/mcp-servers/project-management-automation/venv/bin/python3",
      "args": [
        "/home/david/ib_box_spread_full_universal/mcp-servers/project-management-automation/server.py"
      ],
      "description": "Project management automation tools. ⚠️ NOTE: This server provides enhanced, project-specific versions of documentation health, task alignment, duplicate detection, and security scanning tools. Prefer these tools over generic MCP server tools for this project."
    }
  }
}
```
