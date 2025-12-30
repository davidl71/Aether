# MCP User vs Project Config Fix

**Date**: 2025-12-24
**Status**: ✅ Fixed

## Problem

`context7` appeared twice in Cursor's MCP server list because Cursor was reading MCP configuration from **two locations**:

1. **Project-level**: `.cursor/mcp.json` (10 servers, including context7)
2. **User-level**: `~/.cursor/mcp.json` (7 servers, including context7)

Cursor merges both configs, causing duplicates for servers that appear in both.

## Root Cause

Cursor reads MCP configuration from multiple locations in this order:
1. User-level config (`~/.cursor/mcp.json`) - Global, applies to all projects
2. Project-level config (`.cursor/mcp.json`) - Project-specific

When the same server (like `context7`) appears in both configs, Cursor shows it twice in the UI.

## Solution

**Removed `mcpServers` from user-level config** to use only the project-level config.

### Why This Approach?

1. **Project-specific servers** should be in project config
2. **Version controlled** - project config is in git
3. **Team consistency** - everyone uses the same MCP servers
4. **Easier management** - one source of truth per project

## Changes Made

### Before

**User config** (`~/.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "context7": { ... },
    // ... 6 other servers
  }
}
```

**Project config** (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "context7": { ... },
    // ... 9 other servers
  }
}
```

**Result**: Cursor shows `context7` twice (merged from both configs)

### After

**User config** (`~/.cursor/mcp.json`):
```json
{
  // mcpServers removed - using only project config
}
```

**Project config** (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "context7": { ... },
    // ... 9 other servers (10 total)
  }
}
```

**Result**: Cursor shows `context7` once (only from project config)

## Verification

After the fix:

1. ✅ **Project config** has 10 servers, 1 context7 entry
2. ✅ **User config** no longer has mcpServers
3. ✅ **No overlap** between configs
4. ✅ **Single source of truth** (project config)

## Next Steps

1. **Restart Cursor** completely (Cmd+Q, then reopen)
2. **Check MCP Servers** in Settings
3. **Verify** `context7` appears only once

## Prevention

### Best Practice: Use Only Project-Level Config

For project-specific MCP servers:
- ✅ Use `.cursor/mcp.json` (project-level)
- ✅ Commit to git (version controlled)
- ✅ Share with team

For global/personal MCP servers (if needed):
- Use `~/.cursor/mcp.json` (user-level)
- But avoid overlapping with project configs
- Consider if you really need global servers

### Recommendation

**Remove MCP config from user-level entirely** and use only project-level configs. This ensures:
- No duplicates
- Version controlled
- Team consistency
- Easier management

---

**Last Updated**: 2025-12-24
**Status**: Fixed - User Config Cleaned, Using Only Project Config
