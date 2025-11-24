# Agent MCP Configuration Setup

This guide explains how to ensure all agents have the correct project-management-automation MCP server configuration.

## Quick Setup

Run the sync script from the project root:

```bash
python3 scripts/sync_mcp_config_agents.py
```

This will:
1. Check all agent `.cursor/mcp.json` files
2. Update them with the correct configuration
3. Generate an example configuration file

## Configuration Format

Each agent's `.cursor/mcp.json` should include:

```json
{
  "mcpServers": {
    "project-management-automation": {
      "command": "/absolute/path/to/project/mcp-servers/project-management-automation/run_server.sh",
      "args": [],
      "description": "Project management automation tools. ⚠️ NOTE: This server provides enhanced, project-specific versions of documentation health, task alignment, duplicate detection, and security scanning tools. Prefer these tools over generic MCP server tools for this project."
    }
  }
}
```

## Important Notes

### Path Configuration

**⚠️ Critical**: The `command` path must be **absolute** and **specific to each server**. The sync script automatically detects the correct path for the current server.

### Why Absolute Paths?

Cursor MCP configuration requires absolute paths. The `run_server.sh` wrapper script handles:
- Virtual environment activation
- Project root detection (via `_find_project_root()`)
- Relative path resolution

### Multi-Server Setup

When setting up on different servers:

1. **Run the sync script on each server**:
   ```bash
   cd /path/to/project/on/this/server
   python3 scripts/sync_mcp_config_agents.py
   ```

2. **Verify the path is correct** for that server:
   - Ubuntu: `/home/david/ib_box_spread_full_universal/...`
   - macOS: `/Users/davidl/Projects/Trading/ib_box_spread_full_universal/...`
   - Local: `/Users/davidl/Projects/Trading/ib_box_spread_full_universal/...`

3. **Restart Cursor completely** after updating config

## Verification

After setup, verify the MCP server is working:

1. Open Cursor Settings → MCP Servers
2. Check that `project-management-automation` appears
3. Verify it shows as "Connected" or "Running"
4. Test with a tool like "Check documentation health"

## Troubleshooting

### Server Not Starting

If the MCP server fails to start:

1. **Check the path is correct**:
   ```bash
   ls -la /path/to/project/mcp-servers/project-management-automation/run_server.sh
   ```

2. **Verify virtual environment exists**:
   ```bash
   ls -la /path/to/project/mcp-servers/project-management-automation/venv/bin/python3
   ```

3. **Check Python dependencies**:
   ```bash
   cd /path/to/project/mcp-servers/project-management-automation
   ./venv/bin/python3 -c "import mcp; print('MCP installed')"
   ```

4. **Test the server directly**:
   ```bash
   cd /path/to/project/mcp-servers/project-management-automation
   ./run_server.sh
   ```

### Path Detection Issues

If the server can't find the project root:

1. **Set environment variable** (optional):
   ```bash
   export PROJECT_ROOT=/path/to/project
   ```

2. **Or set in Cursor config**:
   ```json
   {
     "mcpServers": {
       "project-management-automation": {
         "command": "...",
         "args": [],
         "env": {
           "PROJECT_ROOT": "/path/to/project"
         }
       }
     }
   }
   ```

## Agent-Specific Configs

Each agent has its own `.cursor/mcp.json` in:
- `agents/backend/.cursor/mcp.json`
- `agents/web/.cursor/mcp.json`
- `agents/tui/.cursor/mcp.json`
- etc.

The sync script updates all of these automatically.

## Remote Agent Setup

For remote agents (Ubuntu, macOS M4):

1. **SSH to the agent**:
   ```bash
   ssh david@192.168.192.57  # Ubuntu
   ssh davidl@192.168.192.141  # macOS M4
   ```

2. **Navigate to project**:
   ```bash
   cd ~/ib_box_spread_full_universal  # Ubuntu
   cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal  # macOS
   ```

3. **Run sync script**:
   ```bash
   python3 scripts/sync_mcp_config_agents.py
   ```

4. **Restart Cursor** on that agent

## Automated Sync (Future)

A future enhancement could:
- Use Ansible to sync configs across all agents
- Use environment variables for path detection
- Create a centralized config that gets distributed

For now, run the sync script on each server after cloning/updating the repository.
