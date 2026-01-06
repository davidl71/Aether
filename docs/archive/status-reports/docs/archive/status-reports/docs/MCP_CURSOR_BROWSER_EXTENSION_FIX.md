# Cursor Browser Extension MCP Config Fix

**Date**: 2025-12-24
**Status**: Investigation Guide

## Error Message

```
Invalid config for cursor-browsew-extension: mcpServers must be an object
```

## Problem

The `cursor-browser-extension` (or `cursor-browsew-extension` - likely a typo in the error message) is reporting that `mcpServers` is not an object in some configuration file.

## Possible Causes

1. **Workspace Settings** - `.vscode/settings.json` might have `mcpServers` set to null, array, or string
2. **Extension Storage** - Cursor's extension storage might have invalid config
3. **Workspace Storage** - Project-specific storage might have malformed config
4. **User Settings** - User-level settings might have invalid `mcpServers`

## Investigation Steps

### 1. Check Workspace Settings

```bash
# Check for mcpServers in workspace settings
grep -n "mcpServers" .vscode/settings.json
```

**If found and invalid:**

- Remove `mcpServers` from `.vscode/settings.json`
- MCP servers should only be in `.cursor/mcp.json`

### 2. Check Extension Storage

```bash
# Check Cursor MCP extension storage
ls -la ~/Library/Application\ Support/Cursor/User/globalStorage/anysphere.cursor-mcp/
```

**If invalid config found:**

- Remove or fix `mcpServers` in extension storage
- Set to empty object `{}` or remove entirely

### 3. Check Workspace Storage

```bash
# Check project-specific storage
ls -la ~/.cursor/projects/Users-davidl-Projects-Trading-ib-box-spread-full-universal/
```

**If invalid config found:**

- Remove or fix `mcpServers` in workspace storage files

## Resolution

### Option 1: Remove mcpServers from Workspace Settings

If `.vscode/settings.json` contains `mcpServers`:

1. **Edit `.vscode/settings.json`**
2. **Remove any `mcpServers` entries**
3. **Keep only `.cursor/mcp.json` for MCP configuration**

### Option 2: Fix Extension Storage

If extension storage has invalid config:

1. **Locate the config file** in extension storage
2. **Remove or fix `mcpServers`** (set to `{}` or remove)
3. **Restart Cursor**

### Option 3: Clear Extension Cache

If the issue persists:

```bash
# Clear Cursor extension cache
rm -rf ~/Library/Application\ Support/Cursor/User/globalStorage/anysphere.cursor-mcp
rm -rf ~/Library/Application\ Support/Cursor/Cache
```

Then restart Cursor.

## Prevention

**Best Practice**: Keep MCP configuration only in `.cursor/mcp.json`

- ✅ **Use**: `.cursor/mcp.json` (project-level)
- ❌ **Avoid**: `.vscode/settings.json` (workspace settings)
- ❌ **Avoid**: Extension storage configs
- ❌ **Avoid**: User-level MCP configs (unless truly global)

## Verification

After fixing:

1. **Restart Cursor** completely
2. **Check for error** - should be gone
3. **Verify MCP servers** - should all work correctly

---

**Last Updated**: 2025-12-24
**Status**: Investigation Guide
