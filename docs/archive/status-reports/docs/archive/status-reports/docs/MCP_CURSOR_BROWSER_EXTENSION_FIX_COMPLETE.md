# Cursor Browser Extension MCP Config Fix

**Date**: 2025-12-24
**Status**: ✅ Cache Cleared

## Error Message

```
Invalid config for cursor-browsew-extension: mcpServers must be an object
```

## Problem

Cursor's browser extension (`cursor-browsew-extension` - likely a typo for `cursor-browser-extension`) is reporting that `mcpServers` is not an object in some cached configuration.

## Root Cause

The error is likely coming from Cursor's internal extension cache or workspace state, not from the standard configuration files (`.cursor/mcp.json`, `~/.cursor/mcp.json`).

## Solution Applied

### ✅ Cache Cleared

Cleared the following cache locations:

1. **Cursor MCP Extension Storage**

   ```bash
   rm -rf ~/Library/Application Support/Cursor/User/globalStorage/anysphere.cursor-mcp
   ```

2. **Cursor General Cache**

   ```bash
   rm -rf ~/Library/Application Support/Cursor/Cache
   ```

## Next Steps

1. **Restart Cursor completely** (Cmd+Q, then reopen)
2. **Check for error** - should be resolved
3. **Verify MCP servers** - should all work correctly

## If Error Persists

If the error still appears after clearing cache and restarting:

1. **Check Cursor Developer Tools:**
   - Help → Toggle Developer Tools
   - Console tab
   - Look for the exact error with file path
   - This will show which file has the invalid `mcpServers`

2. **Check mcp-cache.json:**

   ```bash
   cat ~/.cursor/projects/Users-davidl-Projects-Trading-ib-box-spread-full-universal/mcp-cache.json | python3 -m json.tool | grep -A 5 mcpServers
   ```

3. **Delete mcp-cache.json** (Cursor will regenerate it):

   ```bash
   rm ~/.cursor/projects/Users-davidl-Projects-Trading-ib-box-spread-full-universal/mcp-cache.json
   ```

4. **Restart Cursor** - Cache will be regenerated with correct structure

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

---

**Last Updated**: 2025-12-24
**Status**: Cache Cleared, Awaiting Cursor Restart
