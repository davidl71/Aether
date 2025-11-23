# MCP Server Name Change: `project-management-automation` → `automa`

## Summary

The MCP server name has been shortened from `project-management-automation` to `automa` for brevity and ease of use.

**New Name:** `automa` (short for "automation")

---

## What Changed

### Server Display Name
- **Before:** `"Project Management Automation"`
- **After:** `"automa"`

This affects:
- Server initialization in `server.py`
- FastMCP/stdio server name
- Log messages showing server name

### MCP Configuration Key
- **Before:** `"project-management-automation"`
- **After:** `"automa"`

This affects:
- `.cursor/mcp.json` configuration key
- Server identification in Cursor
- MCP client references

---

## Required Updates

### 1. Update `.cursor/mcp.json`

Change the server key from `project-management-automation` to `automa`:

**Before:**
```json
{
  "mcpServers": {
    "project-management-automation": {
      "command": "/path/to/run_server.sh",
      "args": []
    }
  }
}
```

**After:**
```json
{
  "mcpServers": {
    "automa": {
      "command": "/path/to/run_server.sh",
      "args": []
    }
  }
}
```

### 2. Restart Cursor

After updating the configuration:
1. Save `.cursor/mcp.json`
2. **Restart Cursor completely** (not just reload)
3. Verify the server appears as "automa" in MCP logs

---

## Files Updated

### ✅ Already Updated
- `server.py` - Server display name changed to "automa"
- `.cursor/rules/project-automation.mdc` - Configuration example updated

### ⚠️ Manual Update Required
- `.cursor/mcp.json` - Change the server key from `project-management-automation` to `automa`

### 📝 Documentation (Optional)
- Documentation files still reference the old name for historical context
- No functional impact, but can be updated for consistency

---

## Why "automa"?

- **Short & Memorable:** Easy to type and remember
- **Clear Meaning:** Obviously related to automation
- **Professional:** Sounds like a tool name, not a description
- **Unique:** Unlikely to conflict with other MCP servers

---

## Verification

After updating `.cursor/mcp.json` and restarting Cursor, you should see:

1. **MCP Logs:** Server name shows as "automa"
2. **Server Status:** `server_status` tool returns server name as "automa"
3. **Tool Availability:** All 7 tools still work normally

---

**Status:** ✅ Server code updated, ⚠️ Manual config update required
