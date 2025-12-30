# MCP Configuration Locations

**Date**: 2025-12-24
**Status**: Investigation Guide

## Problem

Cursor may read MCP server configuration from multiple locations, which can cause duplicates to appear in the UI even when individual files are clean.

## Configuration Hierarchy

Cursor reads MCP configuration in this order (later ones override earlier ones):

1. **User-level (Global)** - Applies to all workspaces
2. **Workspace-level** - Project-specific settings
3. **Project-level** - `.cursor/mcp.json` in project root

## Possible Config Locations

### Project-Level (Current)

**Location**: `.cursor/mcp.json`
- ✅ This is the file we've been editing
- ✅ Should be the primary config for this project
- ✅ Currently has 10 servers, 1 context7 entry

### User-Level (Global)

**Possible locations:**
- `~/Library/Application Support/Cursor/User/settings.json` (macOS)
- `~/Library/Application Support/Cursor/User/globalStorage/mcp.json` (macOS)
- `~/.cursor/mcp.json` (Unix)

**Check for MCP settings:**
```bash
# macOS
cat ~/Library/Application\ Support/Cursor/User/settings.json | grep -i mcp
```

### Workspace-Level

**Possible locations:**
- `.vscode/settings.json` (workspace settings)
- `.cursor/settings.json` (Cursor-specific workspace settings)

**Check:**
```bash
cat .vscode/settings.json | grep -i mcp
cat .cursor/settings.json | grep -i mcp 2>/dev/null
```

## How to Find All Config Files

### Quick Check

```bash
# Find all MCP config files
find ~/Library/Application\ Support/Cursor -name "*mcp*.json" 2>/dev/null
find . -name "*mcp*.json" -o -name "settings.json" | xargs grep -l "mcpServers" 2>/dev/null
```

### Comprehensive Check Script

```python
import os
import json

# Check all possible locations
locations = [
    '.cursor/mcp.json',  # Project
    '.vscode/settings.json',  # Workspace
    os.path.expanduser('~/Library/Application Support/Cursor/User/settings.json'),  # User
    os.path.expanduser('~/Library/Application Support/Cursor/User/globalStorage/mcp.json'),  # User
]

for loc in locations:
    if os.path.exists(loc):
        with open(loc, 'r') as f:
            if 'mcpServers' in f.read():
                print(f"Found MCP config: {loc}")
```

## Resolution Strategy

### If Multiple Configs Found

1. **Check each file for context7 duplicates:**
   ```bash
   # Check project config
   cat .cursor/mcp.json | python3 -m json.tool | grep -i context7

   # Check user settings
   cat ~/Library/Application\ Support/Cursor/User/settings.json | python3 -m json.tool | grep -i context7
   ```

2. **Remove duplicates from all locations:**
   - Edit each file that contains MCP config
   - Remove duplicate `context7` entries
   - Keep only one entry per file

3. **Prefer project-level config:**
   - Remove MCP config from user/workspace settings if possible
   - Keep only `.cursor/mcp.json` for project-specific servers

### Recommended Approach

**Best Practice**: Use only project-level config (`.cursor/mcp.json`)

1. **Remove MCP config from user settings:**
   - Open `~/Library/Application Support/Cursor/User/settings.json`
   - Remove any `mcpServers` entries
   - Let project-level config handle it

2. **Remove MCP config from workspace settings:**
   - Check `.vscode/settings.json`
   - Remove any `mcpServers` entries if present

3. **Use only `.cursor/mcp.json`:**
   - This is the recommended location for project-specific MCP servers
   - Easier to version control
   - Project-specific configuration

## Verification

After cleaning all config files:

1. **Verify project config is clean:**
   ```bash
   python3 scripts/deduplicate_mcp_servers.py .cursor/mcp.json
   ```

2. **Check for other configs:**
   ```bash
   find ~/Library/Application\ Support/Cursor -name "*mcp*.json" 2>/dev/null
   ```

3. **Restart Cursor:**
   - Quit completely (Cmd+Q)
   - Reopen
   - Check MCP Servers in Settings

## Troubleshooting

### If Duplicates Persist

1. **Check Cursor's MCP server list:**
   - Settings → MCP Servers
   - Note which servers appear duplicated
   - Check their source locations

2. **Check Cursor logs:**
   - Help → Toggle Developer Tools
   - Check Console for MCP-related errors
   - Look for config loading messages

3. **Manual cleanup:**
   - Remove duplicate entries directly in Cursor Settings UI
   - Save and restart

---

**Last Updated**: 2025-12-24
**Status**: Investigation Guide
