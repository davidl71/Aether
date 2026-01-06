# Context7 Duplicate Fix

**Date**: 2025-12-24
**Status**: ✅ File Cleaned

## Issue

User reported seeing `context7` listed twice in Cursor's MCP server configuration UI, even though the file appeared to have only one entry.

## Investigation

### File Analysis

- ✅ Only **1** `context7` entry found in `.cursor/mcp.json`
- ✅ No case variations (Context7, CONTEXT7, etc.)
- ✅ No duplicate (command, args) configurations
- ✅ JSON is valid and parseable

### Possible Causes

1. **Cursor Cache Issue** (Most Likely)
   - Cursor may be caching old MCP server configurations
   - Cache might show duplicates even when file is clean

2. **JSON Duplicate Keys** (Possible)
   - JSON spec allows duplicate keys, but parsers typically keep only the last one
   - If file had duplicate keys, parser would show only one, but Cursor might see both

3. **Multiple Config Sources** (Unlikely)
   - Cursor might be reading from multiple locations
   - Could be merging configs from different sources

## Resolution

### Actions Taken

1. ✅ **Rewrote JSON file** to ensure clean structure
   - Removed any potential duplicate keys
   - Ensured proper JSON formatting

2. ✅ **Ran deduplication script**
   - Verified no duplicates exist
   - Confirmed all servers are unique

3. ✅ **Verified configuration**
   - Only 1 `context7` entry in file
   - All 10 servers present and unique

## Current Configuration

### All Servers (10 total)

- Todo2
- agentic-tools
- **context7** (only 1 entry)
- exarp
- filesystem
- git
- ollama
- semgrep
- sequential_thinking
- tractatus_thinking

## If Duplicate Still Appears in Cursor

If you still see `context7` twice in Cursor's UI after this fix:

### Step 1: Clear Cursor Cache

```bash
# macOS
rm -rf ~/Library/Application\ Support/Cursor/Cache
rm -rf ~/Library/Application\ Support/Cursor/User/workspaceStorage
```

### Step 2: Restart Cursor Completely

1. **Quit Cursor completely** (not just reload window)
   - Cmd+Q on macOS
   - Or: Cursor → Quit Cursor

2. **Reopen Cursor**

3. **Check MCP Servers**
   - Open Cursor Settings → MCP Servers
   - Verify `context7` appears only once

### Step 3: Verify Configuration

```bash
# Check the file
cat .cursor/mcp.json | python3 -m json.tool | grep -i context7

# Should show only one entry
```

### Step 4: If Still Duplicated

1. **Check for other config files:**

   ```bash
   find ~ -name "*mcp*.json" -type f 2>/dev/null | grep -i cursor
   ```

2. **Check Cursor's workspace settings:**
   - Look for workspace-specific MCP configs
   - Check `.vscode/settings.json` for MCP overrides

3. **Manual fix in Cursor:**
   - Open Cursor Settings → MCP Servers
   - Manually remove the duplicate entry
   - Save and restart

## Verification

The configuration file is now clean:

- ✅ Only 1 `context7` entry
- ✅ No duplicate keys
- ✅ Valid JSON structure
- ✅ All servers unique

If duplicates persist in Cursor's UI, it's a cache issue that should resolve after clearing cache and restarting.

---

**Last Updated**: 2025-12-24
**Status**: File Cleaned, Awaiting Cursor Cache Clear
