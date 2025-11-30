# MCP Servers: GUI vs Headless Mode Compatibility

**Date**: 2025-11-23
**Purpose**: Identify which MCP servers require GUI Cursor vs work in headless/server mode

## Summary

| Server | GUI Required | Headless Compatible | Notes |
|--------|--------------|---------------------|-------|
| **exarp** | ❌ No | ✅ Yes | Pure CLI tool |
| **filesystem** | ❌ No | ✅ Yes | File operations work anywhere |
| **notebooklm** | ⚠️ **Yes** | ❌ No | Uses browser automation |
| **context7** | ❌ No | ✅ Yes | API-based, no GUI needed |
| **agentic-tools** | ❌ No | ✅ Yes | File-based storage |
| **tractatus_thinking** | ❌ No | ✅ Yes | Pure logic processing |
| **sequential_thinking** | ❌ No | ✅ Yes | Pure logic processing |
| **git** | ❌ No | ✅ Yes | CLI git operations |
| **terminal** | ⚠️ **Partial** | ⚠️ **Partial** | Works but less useful without GUI |
| **browser** | ⚠️ **Yes** | ❌ No | Requires browser/GUI |
| **iterm2** | ⚠️ **Yes** | ❌ No | macOS only, requires iTerm2 GUI |

## GUI-Required Servers

### 1. **iTerm2 Server** (macOS Only)

**Package**: `@rishabkoul/iterm-mcp-server`
**Status**: ⚠️ **Not in current config** (removed for Linux)

**Why GUI Required**:

- Requires iTerm2 application to be running
- Needs GUI terminal emulator with iTerm2-specific features
- Uses macOS AppleScript to interact with iTerm2 windows
- Cannot work in headless SSH sessions

**Use Cases**:

- Terminal context awareness
- Command execution in visible terminals
- Session management
- Output reading from GUI terminals

**Headless Alternative**: Use `terminal` server instead (works via stdio)

---

### 2. **Browser Automation Server**

**Package**: `@modelcontextprotocol/server-browser`
**Status**: ⚠️ **Not in current config**

**Why GUI Required**:

- Requires browser (Chrome/Chromium) with GUI
- Needs display server (X11/Wayland on Linux, Quartz on macOS)
- Cannot run headless without display (though headless Chrome exists, MCP server may not support it)
- Requires visual browser for interaction

**Use Cases**:

- Testing web UI
- Screenshot capture
- PWA verification
- Web interaction testing

**Headless Alternative**: Use API-based testing or headless browser automation tools

---

### 3. **NotebookLM Server**

**Package**: `notebooklm-mcp@latest` (via mcpower-proxy)
**Status**: ✅ **In current config**

**Why GUI Required**:

- Uses browser automation to interact with NotebookLM web interface
- Requires Chrome/Chromium with GUI
- Needs display server for browser
- Accesses NotebookLM via web UI (not API)

**Use Cases**:

- Summarizing YouTube videos
- Processing documentation links
- Creating knowledge bases
- Research synthesis

**Headless Alternative**: Use direct API access if NotebookLM provides one (currently not available)

---

## GUI-Optional Servers (Work Better with GUI)

### **Terminal/Shell Server**

**Package**: `@modelcontextprotocol/server-terminal`
**Status**: ⚠️ **Not in current config**

**Why GUI-Optional**:

- ✅ **Works in headless**: Can execute commands via stdio
- ⚠️ **Less useful**: No visual feedback, harder to debug
- ⚠️ **Limited interaction**: Can't see output in real-time GUI terminal

**Use Cases**:

- Running build commands
- Executing tests
- Running scripts
- Command execution

**Recommendation**: Works in both modes, but GUI provides better visibility

---

## Headless-Compatible Servers (Current Config)

All servers in your current `.cursor/mcp.json` work in headless mode:

### ✅ **exarp**

- Pure CLI tool
- No GUI dependencies
- Works via stdio

### ✅ **filesystem**

- File operations via stdio
- No GUI needed
- Works in SSH/headless environments

### ✅ **context7**

- API-based documentation lookup
- No GUI dependencies
- Pure HTTP/API communication

### ✅ **agentic-tools**

- File-based storage (`.agentic-tools-mcp/`)
- No GUI dependencies
- Works via stdio

### ✅ **tractatus_thinking**

- Pure logic processing
- No GUI dependencies
- Works via stdio

### ✅ **sequential_thinking**

- Pure logic processing
- No GUI dependencies
- Works via stdio

---

## Configuration Recommendations

### For GUI Cursor (Current Setup)

Your current config is **optimal for GUI Cursor**:

- ✅ All servers work in GUI mode
- ⚠️ **notebooklm** requires browser (works in GUI)
- ✅ No GUI-only servers that would fail

### For Headless Cursor

If running Cursor in headless/server mode, consider:

**Remove**:

- ❌ `notebooklm` (requires browser/GUI)

**Keep**:

- ✅ `exarp`
- ✅ `filesystem`
- ✅ `context7`
- ✅ `agentic-tools`
- ✅ `tractatus_thinking`
- ✅ `sequential_thinking`

**Optional Add**:

- ⚠️ `terminal` (works but less useful without GUI)

---

## Platform-Specific Considerations

### macOS

- ✅ **iTerm2 server** available (GUI only)
- ✅ **Browser automation** works (requires display)
- ✅ **NotebookLM** works (requires browser)

### Linux

- ❌ **iTerm2 server** not available (macOS only)
- ✅ **Browser automation** works (requires X11/Wayland)
- ✅ **NotebookLM** works (requires browser)

### Windows

- ❌ **iTerm2 server** not available (macOS only)
- ✅ **Browser automation** works (requires display)
- ✅ **NotebookLM** works (requires browser)

---

## Testing GUI Requirements

To test if a server requires GUI:

```bash

# Test in headless environment (no display)

unset DISPLAY
export DISPLAY=""

# Try running server

npx -y @modelcontextprotocol/server-browser --version

# If it fails with display errors → GUI required
# If it works → Headless compatible
```

---

## Recommendations

### Current Configuration (GUI Cursor)

✅ **Keep as-is** - All servers work in GUI Cursor:

- `notebooklm` works because GUI provides browser
- All other servers are GUI-agnostic

### If Moving to Headless

1. **Remove** `notebooklm` (requires browser/GUI)
2. **Keep** all other servers (headless-compatible)
3. **Consider** adding `terminal` server if command execution needed

### Hybrid Approach

- **GUI Cursor**: Use full config (including `notebooklm`)
- **Headless Cursor**: Use minimal config (remove `notebooklm`)

---

## See Also

- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - Detailed server documentation
- [MCP_TROUBLESHOOTING.md](MCP_TROUBLESHOOTING.md) - Troubleshooting guide
- [MCP_CONFIG_LINT_REPORT.md](MCP_CONFIG_LINT_REPORT.md) - Configuration validation
