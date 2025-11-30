# MCP Server Duplicate Fix

## Problem

MCP servers were appearing twice in Cursor's "Installed MCP Servers" list because Cursor merges configurations from two locations:

1. **User-level**: `~/.cursor/mcp.json` (global configuration)
2. **Workspace-level**: `./.cursor/mcp.json` (project-specific configuration)

## Solution

Removed duplicate servers from user-level configuration:

1. **`context7`** - Removed from user-level (workspace version is more up-to-date)
2. **`GitKraken`** - Removed from user-level (workspace `gitkraken` is the official GitKraken CLI MCP)

The workspace-level versions are kept as they're more up-to-date and project-specific.

## Configuration Locations

### User-Level (`~/.cursor/mcp.json`)

- Global MCP servers available in all projects
- Contains: `claude-scientific-skills`, `claude-skills-mcp`, `codacy`, `cycode`, `desktop-commander`, `docfork`, `GitKraken`, `openmemory`, `tractatus_thinking`

### Workspace-Level (`./.cursor/mcp.json`)

- Project-specific MCP servers
- Contains: `agentic-tools`, `context7`, `filesystem`, `git`, `gitkraken`, `iterm2`, `notebooklm`, `semgrep`

## How Cursor Merges Configurations

Cursor merges both configurations, which means:

- If a server exists in both, it appears twice (duplicate)
- User-level servers are available in all projects
- Workspace-level servers are only available in this project

## Best Practices

1. **Avoid duplicates**: Don't define the same server in both files
2. **User-level**: Use for global tools (like `openmemory`, `codacy`)
3. **Workspace-level**: Use for project-specific tools (like `filesystem`, `git` with project path)
4. **Restart Cursor**: After making changes, restart Cursor completely

## Restore Backup

If needed, restore the user-level config from backup:

```bash
cp ~/.cursor/mcp.json.backup ~/.cursor/mcp.json
```

## Verification

After fixing, restart Cursor and check:

1. Open Cursor Settings
2. Go to MCP Servers
3. Verify `context7` appears only once
4. Check that all servers are working (no errors)
