# NotebookLM MCP Server Disabled

**Date:** 2025-01-20
**Action:** Disabled `notebooklm` in project MCP configuration
**Reason:** Optimizing tool count to stay under 80-tool limit
**Status:** Disabled (can be re-enabled later if needed)

---

## Changes Made

### ✅ Project Configuration (`.cursor/mcp.json`)

**Removed:**
- ❌ `notebooklm` - Research & documentation server

**Kept:**
- ✅ `filesystem` - File operations (workspace-scoped)
- ✅ `git` - Version control (repository-specific)
- ✅ `agentic-tools` - Task management (required for Todo2)
- ✅ `context7` - Documentation lookup
- ✅ `semgrep` - Security scanning (required by `.cursorrules`)

**Before:** 6 servers
**After:** 5 servers

---

## Impact Assessment

### ✅ Expected Savings

**Tools Removed:**
- Estimated **10-15 tools** removed (NotebookLM tool count)

**Expected Tool Count After Removal:**
- **Before:** ~81-128 tools (after removing desktop-commander)
- **After:** ~71-113 tools (still may exceed 80, but reduced)

### ✅ Functionality Impact

**What You Lose:**
- ❌ Access to TWS API knowledge base notebook
- ❌ YouTube video summarization via NotebookLM
- ❌ Documentation link processing via NotebookLM
- ❌ Zero-hallucination knowledge base queries

**What You Keep:**
- ✅ All other research capabilities (web search)
- ✅ All core development tools
- ✅ All Todo2 workflow functionality
- ✅ All security scanning capabilities

### ✅ Alternative Solutions

**For Research:**
- ✅ Use web search for general research
- ✅ Use official documentation websites
- ✅ Use Context7 for library documentation (still configured)
- ✅ Create documentation from research manually

**For TWS API Knowledge:**
- ✅ Use official TWS API documentation
- ✅ Use project documentation in `docs/` directory
- ✅ Re-enable NotebookLM temporarily when needed
- ✅ Use web search for specific TWS API questions

---

## Rationale

### Why Disable NotebookLM?

1. **Tool Count Optimization:**
   - NotebookLM estimated to contribute **10-15 tools**
   - Removing it reduces tool count while keeping core functionality

2. **Usage-Based Decision:**
   - Disabled temporarily to optimize tool count
   - Can be re-enabled when needed for specific research tasks
   - Not essential for day-to-day development

3. **Redundancy:**
   - Research can be done via web search
   - Documentation can be accessed via Context7 or web search
   - Project documentation exists in `docs/` directory

4. **Tool Limit Priority:**
   - Must stay under 80-tool limit
   - Core development tools take priority
   - Research tools can be re-enabled when needed

---

## Current Configuration Status

### Global Servers (`~/.cursor/mcp.json`) - 2 servers

1. ✅ `tractatus_thinking` - Universal logical analysis (5-8 tools)
2. ✅ `sequential_thinking` - Universal structured problem-solving (5-8 tools)

### Project Servers (`.cursor/mcp.json`) - 5 servers

1. ✅ `filesystem` - File operations (10-15 tools)
2. ✅ `git` - Version control (10-15 tools)
3. ✅ `agentic-tools` - Task management (30-50 tools) - **REQUIRED FOR TODO2**
4. ✅ `context7` - Documentation lookup (8-12 tools)
5. ✅ `semgrep` - Security scanning (3-5 tools)

**Total: 7 servers (2 global + 5 project)**

**Estimated Total Tools: 71-113 tools** ⚠️ **MAY STILL EXCEED 80 IF AGENTIC-TOOLS IS LARGE**

---

## Re-Enabling NotebookLM (If Needed)

If you need to use NotebookLM for specific research tasks:

### Temporary Re-Enable

1. Edit `.cursor/mcp.json` (project config)
2. Add back `notebooklm` entry:
   ```json
   {
     "notebooklm": {
       "command": "uvx",
       "args": [
         "mcpower-proxy==0.0.87",
         "--wrapped-config",
         "{\n      \"command\": \"npx\",\n      \"args\": [\n        \"-y\",\n        \"notebooklm-mcp@latest\"\n      ],\n      \"description\": \"NotebookLM MCP server for summarizing YouTube videos, documentation links, and creating zero-hallucination knowledge base\"\n    }",
         "--name",
         "notebooklm"
       ]
     }
   }
   ```
3. Save and restart Cursor
4. Use NotebookLM for research
5. Disable again after research is complete

### Permanent Re-Enable

If you find NotebookLM is essential and frequently used:
1. Re-add NotebookLM configuration
2. Remove `context7` instead (redundant with web search)
3. Or remove thinking tools if still over 80-tool limit

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
   - Verify tool count is reduced
   - Expected: ~71-113 tools (may still exceed 80)

2. **Test Functionality:**
   - Test `filesystem` server: File operations should work
   - Test `git` server: Version control should work
   - Test `agentic-tools`: Todo2 workflow should work
   - Verify core development tools work as expected

### 3. If Still Over 80 Tools

If tool count is still over 80 after disabling NotebookLM:

1. **Remove Context7** (saves 8-12 tools)
   - Edit `.cursor/mcp.json` (project config)
   - Remove `context7` entry
   - Restart Cursor
   - Expected: ~63-101 tools

2. **Remove Thinking Tools** (saves 10-16 tools) - Last resort
   - Remove `tractatus_thinking` from global config
   - Remove `sequential_thinking` from global config
   - Restart Cursor
   - Expected: ~53-85 tools (should be under 80)

---

## Summary

**Action Taken:** Disabled NotebookLM (removed 10-15 tools)

**Expected Tool Count:** ~71-113 tools (down from ~81-128)

**Remaining Servers:**
- ✅ Core development: filesystem, git, semgrep
- ✅ Todo2 workflow: agentic-tools
- ✅ Documentation: context7
- ✅ Thinking tools: tractatus_thinking, sequential_thinking

**If Still Over 80:**
- Next step: Remove context7 (saves 8-12 tools)
- Last resort: Remove thinking tools (saves 10-16 tools)

---

## Related Documents

- [MCP_TOOL_COUNT_ANALYSIS_TODO2.md](MCP_TOOL_COUNT_ANALYSIS_TODO2.md) - Comprehensive analysis with Todo2 constraint
- [MCP_DESKTOP_COMMANDER_REMOVAL.md](MCP_DESKTOP_COMMANDER_REMOVAL.md) - Desktop Commander removal details
- [MCP_CONFIGURATION_UPDATE_SUMMARY.md](MCP_CONFIGURATION_UPDATE_SUMMARY.md) - Previous configuration updates
