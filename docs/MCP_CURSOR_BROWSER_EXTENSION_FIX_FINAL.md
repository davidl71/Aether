# Cursor Browser Extension MCP Config Fix - Final Steps

**Date**: 2025-12-24
**Status**: Comprehensive Cache Clear Applied

## Error Message

```
Invalid config for cursor-browsew-extension: mcpServers must be an object
```

## Problem

The error persists even after initial cache clearing, suggesting the invalid configuration is being regenerated or exists in a location we haven't accessed yet.

## Actions Taken

### ✅ Cache Cleared (Round 2)

1. **Cleared workspaceStorage**
   ```bash
   rm -rf ~/Library/Application Support/Cursor/User/workspaceStorage
   ```
   - This removes workspace-specific settings for all projects
   - Cursor will regenerate this on restart

2. **Removed mcp-cache.json**
   ```bash
   rm ~/.cursor/projects/Users-davidl-Projects-Trading-ib-box-spread-full-universal/mcp-cache.json
   ```
   - Cache file will be regenerated with correct structure

3. **Previously cleared:**
   - Cursor MCP extension storage
   - Cursor general cache

## Next Steps

### 1. Restart Cursor Completely

**Critical**: Quit Cursor completely (Cmd+Q), then reopen.

### 2. If Error Persists - Get Exact File Path

1. **Open Cursor Developer Tools:**
   - Help → Toggle Developer Tools
   - Console tab

2. **Look for the error message:**
   - It should show the exact file path
   - Example: `Invalid config for cursor-browser-extension: mcpServers must be an object at /path/to/file.json`

3. **Share the file path** and we can fix it directly

### 3. Nuclear Option (If Nothing Else Works)

If the error still appears after restart:

```bash
# Clear ALL Cursor caches and state
rm -rf ~/Library/Application\ Support/Cursor/User/globalStorage/anysphere.cursor-mcp
rm -rf ~/Library/Application\ Support/Cursor/Cache
rm -rf ~/Library/Application\ Support/Cursor/User/workspaceStorage
rm -f ~/.cursor/projects/Users-davidl-Projects-Trading-ib-box-spread-full-universal/mcp-cache.json

# Then restart Cursor
```

**Warning**: This will reset workspace settings for ALL projects.

## Root Cause Analysis

The error suggests that somewhere in Cursor's configuration hierarchy, `mcpServers` is set to:
- `null`
- An array `[]`
- A string `""`
- `true` or `false`

Instead of a valid object `{}`.

## Prevention

**Best Practice**: Keep MCP configuration only in `.cursor/mcp.json`

- ✅ **Use**: `.cursor/mcp.json` (project-level, version controlled)
- ❌ **Avoid**: Extension caches (auto-generated)
- ❌ **Avoid**: Workspace settings (`.vscode/settings.json`)
- ❌ **Avoid**: User-level configs (unless truly global)

## Verification

After restarting Cursor:

1. ✅ Error should be gone
2. ✅ All MCP servers should work
3. ✅ No duplicate entries
4. ✅ Clean configuration

If the error persists, **check Cursor Developer Tools Console** for the exact file path causing the issue.

---

**Last Updated**: 2025-12-24
**Status**: Comprehensive Cache Clear Applied, Awaiting Cursor Restart
