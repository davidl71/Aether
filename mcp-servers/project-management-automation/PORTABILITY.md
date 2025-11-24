# Automa MCP Server Portability

## Overview

The automa MCP server is designed to work across different agents and servers with different paths. This document explains how portability is achieved.

## Portable Components

### ✅ Python Code (`server.py` and resources)

All Python code uses **relative path detection** and is fully portable:

```python
# Uses __file__ to find project root dynamically
project_root = _find_project_root(Path(__file__))
```

- **No hardcoded paths** in Python code
- Uses `Path(__file__)` to detect location
- Finds project root by looking for `.git`, `.todo2`, or `CMakeLists.txt`
- Works from any installation path

### ✅ Shell Script (`run_server.sh`)

The wrapper script uses **script-relative paths**:

```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
```

- Uses `${BASH_SOURCE[0]}` to find script location
- Works regardless of where it's called from
- Automatically finds virtual environment in same directory

## Configuration Path Issue

### ⚠️ `.cursor/mcp.json` Requires Absolute Paths

Cursor's MCP configuration **requires absolute paths** for the `command` field. This means:

- Each agent/server needs its own path in `.cursor/mcp.json`
- The path will be different on different machines:
  - macOS: `/Users/davidl/Projects/Trading/ib_box_spread_full_universal/...`
  - Ubuntu: `/home/david/ib_box_spread_full_universal/...`
  - Other servers: Different paths

### Solution: Sync Script

**Use the sync script** to automatically configure paths for each agent:

```bash
# Run from project root on each agent/server
python3 scripts/sync_mcp_config_agents.py
```

The sync script:
1. Detects the current project root
2. Finds the correct absolute path to `run_server.sh`
3. Updates `.cursor/mcp.json` with the correct path
4. Updates all agent `.cursor/mcp.json` files
5. Generates an example configuration

## Setup Instructions

### For Each Agent/Server

1. **Clone/checkout the repository** on the agent
2. **Run the sync script**:
   ```bash
   cd /path/to/project/on/this/server
   python3 scripts/sync_mcp_config_agents.py
   ```
3. **Verify the path** in `.cursor/mcp.json` is correct for this server
4. **Restart Cursor** completely

### Manual Configuration (if needed)

If you need to configure manually, use this format:

```json
{
  "mcpServers": {
    "automa": {
      "command": "/absolute/path/to/project/mcp-servers/project-management-automation/run_server.sh",
      "args": [],
      "description": "Project management automation tools - documentation health, task alignment, duplicate detection, security scanning, and automation opportunities"
    }
  }
}
```

**Important**: Replace `/absolute/path/to/project` with the actual absolute path on that server.

## Path Detection Logic

The server uses this logic to find the project root:

1. **Environment variables**: Checks `PROJECT_ROOT` or `WORKSPACE_PATH`
2. **Marker files**: Looks for `.git`, `.todo2`, or `CMakeLists.txt`
3. **Relative fallback**: Assumes standard directory structure

This means the server will work correctly as long as:
- The repository structure is maintained
- The server is run from within the project (or path is set correctly)

## Verification

After setup, verify portability:

1. **Check Python code**:
   ```bash
   python3 -c "from pathlib import Path; print(Path(__file__).parent)"
   # Should show relative path, not hardcoded
   ```

2. **Check shell script**:
   ```bash
   cd /any/directory
   /path/to/project/mcp-servers/project-management-automation/run_server.sh --help
   # Should work from any directory
   ```

3. **Check MCP config**:
   ```bash
   cat .cursor/mcp.json | grep automa
   # Should show absolute path for this server
   ```

## Troubleshooting

### Server Not Starting

**Problem**: MCP server fails to start with path errors

**Solution**:
1. Run the sync script: `python3 scripts/sync_mcp_config_agents.py`
2. Verify the path in `.cursor/mcp.json` is absolute and correct
3. Check that `run_server.sh` is executable: `chmod +x mcp-servers/project-management-automation/run_server.sh`

### Wrong Project Root Detected

**Problem**: Server detects wrong project root

**Solution**:
1. Set environment variable: `export PROJECT_ROOT=/correct/path`
2. Or ensure marker files (`.git`, `.todo2`, `CMakeLists.txt`) exist in project root
3. Check that repository structure is intact

### Virtual Environment Not Found

**Problem**: `run_server.sh` can't find venv

**Solution**:
1. Create virtual environment: `cd mcp-servers/project-management-automation && python3 -m venv venv`
2. Install dependencies: `venv/bin/pip install -e .`
3. Verify venv exists: `ls mcp-servers/project-management-automation/venv/bin/python3`

## Summary

- ✅ **Python code**: Fully portable, no hardcoded paths
- ✅ **Shell script**: Portable, uses script-relative paths
- ⚠️ **MCP config**: Requires absolute paths (use sync script)
- ✅ **Solution**: Run `scripts/sync_mcp_config_agents.py` on each agent

---

**Last Updated**: 2025-11-24
