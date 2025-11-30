# Desktop Commander MCP Server Removal

**Date:** 2025-01-20
**Action:** Removed `desktop-commander` from global MCP configuration
**Reason:** Exceeding 80-tool limit - Desktop Commander estimated to contribute 40-60 tools (50-75% of limit)

---

## Changes Made

### ✅ Global Configuration (`~/.cursor/mcp.json`)

**Removed:**

- ❌ `desktop-commander` - System-level operations server

**Kept:**

- ✅ `tractatus_thinking` - Universal logical analysis
- ✅ `sequential_thinking` - Universal structured problem-solving

**Before:** 3 servers (tractatus_thinking, desktop-commander, sequential_thinking)
**After:** 2 servers (tractatus_thinking, sequential_thinking)

---

## Impact Assessment

### ✅ Expected Savings

**Tools Removed:**

- Estimated **40-60 tools** removed (Desktop Commander tool count)

**Expected Tool Count After Removal:**

- **Before:** 121-188 tools (exceeded 80 limit by 41-108 tools)
- **After:** ~81-128 tools (may still exceed limit, but significantly reduced)

### ✅ Functionality Impact

**What You Lose:**

- ❌ System-level file operations outside workspace
- ❌ Terminal/process management operations
- ❌ System information tools (processes, environment)
- ❌ Network operations via Desktop Commander
- ❌ Advanced file search outside workspace

**What You Keep:**

- ✅ Workspace file operations via `filesystem` server
- ✅ Regular terminal commands (via terminal/command line)
- ✅ All project-specific operations via `filesystem` and `git` servers
- ✅ All thinking tools (tractatus_thinking, sequential_thinking)

### ✅ Alternative Solutions

**For Workspace File Operations:**

- ✅ Use `filesystem` server (already configured in project config)
- ✅ Workspace-scoped file operations are still available

**For System Operations:**

- ✅ Use regular terminal commands
- ✅ Use macOS system commands directly
- ✅ Use shell scripts for automation

**For Terminal/Process Operations:**

- ✅ Use regular terminal commands
- ✅ Use system terminal applications (iTerm2, Terminal.app)
- ✅ Use shell scripts for automation

---

## Rationale

### Why Remove Desktop Commander?

1. **Primary Tool Consumer:**
   - Desktop Commander estimated to contribute **40-60 tools** (50-75% of 80-tool limit)
   - Largest single contributor to tool count

2. **Redundant Functionality:**
   - Many operations can be done via `filesystem` server (workspace-scoped)
   - System operations can be done via regular terminal commands
   - Advanced features rarely used for C++ trading project

3. **Project Focus:**
   - Trading application primarily needs:
     - File operations (workspace-scoped) ✅ `filesystem` server
     - Version control ✅ `git` server
     - Security scanning ✅ `semgrep` server
     - Thinking tools ✅ `tractatus_thinking`, `sequential_thinking`
   - System-level operations less critical

4. **Tool Limit Priority:**
   - Must stay under 80-tool limit
   - Desktop Commander is the easiest removal with largest impact
   - Core functionality preserved via `filesystem` server

---

## Next Steps

### 1. Restart Cursor

**⚠️ IMPORTANT:** Restart Cursor completely after configuration changes:

1. Close all Cursor windows
2. Quit Cursor completely (Cmd+Q on macOS)
3. Reopen Cursor
4. Verify MCP servers are loaded correctly

### 2. Verify Tool Count

After restarting, verify:

1. **Check Tool Count:**
   - Open Cursor Settings
   - Go to MCP Servers
   - Verify tool count is now under 80
   - Expected: ~81-128 tools (may still exceed if agentic-tools is large)

2. **Test Functionality:**
   - Test `filesystem` server: File operations should still work
   - Test `git` server: Version control should still work
   - Test thinking tools: Tractatus and Sequential should still work
   - Verify workspace file operations work as expected

### 3. If Still Over 80 Tools

If tool count is still over 80 after removing Desktop Commander:

1. **Remove Context7** (saves 8-12 tools)
   - Edit `.cursor/mcp.json` (project config)
   - Remove `context7` entry
   - Restart Cursor

2. **Evaluate NotebookLM** (saves 10-15 tools if removed)
   - Remove if not actively using TWS API notebook
   - Keep if actively querying notebook regularly

3. **Review Agentic Tools** (saves 30-50 tools if removed)
   - Keep if Todo2 workflow is mandatory and used
   - Remove if Todo2 workflow is not being used

---

## Configuration Before and After

### Before Removal

**Global Config (`~/.cursor/mcp.json`):**

```json
{
  "mcpServers": {
    "tractatus_thinking": { ... },
    "desktop-commander": { ... },  // REMOVED
    "sequential_thinking": { ... }
  }
}
```

**Servers:** 3 (tractatus_thinking, desktop-commander, sequential_thinking)

### After Removal

**Global Config (`~/.cursor/mcp.json`):**

```json
{
  "mcpServers": {
    "tractatus_thinking": { ... },
    "sequential_thinking": { ... }
  }
}
```

**Servers:** 2 (tractatus_thinking, sequential_thinking)

---

## Verification Checklist

- [x] Desktop Commander removed from global config
- [x] Global config validated as valid JSON
- [x] Tractatus Thinking still configured
- [x] Sequential Thinking still configured
- [x] Configuration documented

**Next Steps:**

- [ ] Restart Cursor completely
- [ ] Verify tool count is reduced
- [ ] Test file operations via filesystem server
- [ ] Verify thinking tools still work
- [ ] Check if still over 80 tools (if yes, remove context7)

---

## Related Documents

- [MCP_TOOL_COUNT_ANALYSIS.md](MCP_TOOL_COUNT_ANALYSIS.md) - Comprehensive tool count analysis
- [MCP_CONFIGURATION_UPDATE_SUMMARY.md](MCP_CONFIGURATION_UPDATE_SUMMARY.md) - Previous configuration updates
- [MCP_GLOBAL_VS_PROJECT_ANALYSIS.md](MCP_GLOBAL_VS_PROJECT_ANALYSIS.md) - Global vs project placement analysis
