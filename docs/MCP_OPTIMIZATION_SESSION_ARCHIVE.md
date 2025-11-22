# MCP Optimization Session Archive

**Date:** 2025-01-20
**Session Type:** MCP Server Review & Tool Count Optimization
**Status:** Completed initial review and optimizations

---

## Session Summary

This session focused on reviewing installed MCP servers, optimizing configuration to stay under the 80-tool limit, and identifying which servers/tools can be disabled while maintaining core functionality (including Todo2 workflow).

---

## Actions Taken

### ✅ Removed Servers

1. **desktop-commander** - Removed from global config
   - **Reason:** Primary tool consumer (estimated 40-60 tools - 50-75% of limit)
   - **Saved:** ~40-60 tools
   - **Alternative:** Use `filesystem` server + regular terminal commands

2. **notebooklm** - Removed from project config
   - **Reason:** Not actively using, tool optimization
   - **Saved:** ~10-15 tools
   - **Status:** Can be re-enabled if needed for research

**Total Tools Saved:** ~50-75 tools

### ✅ Configuration Updates

1. **Added semgrep** - Added to project config (required by `.cursorrules`)
   - **Tools:** ~3-5 tools
   - **Status:** Required for security scanning

2. **Moved sequential_thinking** - Moved from project to global config
   - **Reason:** Universal thinking tool, should be global like tractatus_thinking
   - **Tools:** ~5-8 tools

3. **Kept agentic-tools** - Required for Todo2 workflow
   - **Tools:** ~30-50 tools (largest remaining contributor)
   - **Status:** Mandatory - Todo2 is actively used

---

## Current Configuration Status

### Global Servers (`~/.cursor/mcp.json`) - 3 servers

1. ✅ `tractatus_thinking` - Universal logical analysis (5-8 tools)
2. ✅ `sequential_thinking` - Universal structured problem-solving (5-8 tools)
3. ⚠️ `openmemory` - Unknown tool count (discovered during review)

### Project Servers (`.cursor/mcp.json`) - 5 servers

1. ✅ `filesystem` - File operations (10-15 tools)
2. ✅ `git` - Version control (10-15 tools)
3. ✅ `agentic-tools` - Task management (30-50 tools) - Required for Todo2
4. ⚠️ `context7` - Documentation lookup (8-12 tools) - Can remove
5. ✅ `semgrep` - Security scanning (3-5 tools) - Required by rules

**Total: 8 servers (3 global + 5 project)**

**Estimated Tool Count: 61-113 + openmemory (unknown)**

---

## Key Decisions

### ✅ Must Keep (Required)

- **agentic-tools** - Required for Todo2 workflow (30-50 tools)
- **filesystem** - Essential for file operations (10-15 tools)
- **git** - Essential for version control (10-15 tools)
- **semgrep** - Required by `.cursorrules` (3-5 tools)

### ⚠️ Keep (Low Tool Count, High Value)

- **tractatus_thinking** - Universal thinking tool (5-8 tools)
- **sequential_thinking** - Universal thinking tool (5-8 tools)

### ⚠️ Can Remove (Optimization Targets)

- **context7** - Redundant with web search (8-12 tools)
- **openmemory** - Unknown tool count (needs investigation)

---

## Documents Created

1. **`docs/MCP_SERVER_REVIEW.md`** - Comprehensive server review and recommendations
2. **`docs/MCP_GLOBAL_VS_PROJECT_ANALYSIS.md`** - Global vs project placement analysis
3. **`docs/MCP_CONFIGURATION_UPDATE_SUMMARY.md`** - Configuration changes summary
4. **`docs/MCP_TOOL_COUNT_ANALYSIS.md`** - Tool count analysis and optimization strategy
5. **`docs/MCP_TOOL_COUNT_ANALYSIS_TODO2.md`** - Updated analysis accounting for Todo2
6. **`docs/MCP_TOOL_LEVEL_CHERRY_PICKING.md`** - Tool-level filtering analysis
7. **`docs/MCP_DESKTOP_COMMANDER_REMOVAL.md`** - Desktop Commander removal details
8. **`docs/MCP_NOTEBOOKLM_DISABLED.md`** - NotebookLM disabling details
9. **`docs/MCP_POST_RESTART_REVIEW.md`** - Post-restart status review

---

## Outstanding Items

### ⚠️ Needs Investigation

1. **openmemory Server**
   - Discovered in global config but not in previous analysis
   - Unknown tool count
   - Unknown purpose/essentiality
   - **Action:** Investigate purpose and tool count

2. **Actual Tool Count**
   - Estimated: 61-113 + openmemory (unknown)
   - **Action:** Verify actual count in Cursor Settings → MCP Servers
   - **Question:** Are we under the 80-tool limit?

3. **Tool-Level Filtering**
   - **agentic-tools** may support tool-level configuration
   - Potential savings: 15-30 tools by disabling advanced features
   - **Action:** Investigate agentic-tools documentation for tool filtering support

### ⚠️ Recommended Next Steps

1. **Verify Tool Count** - Check Cursor Settings → MCP Servers
2. **Investigate openmemory** - Determine purpose and tool count
3. **Remove context7** - If still over 80 tools (saves 8-12 tools)
4. **Investigate agentic-tools tool filtering** - Potential 15-30 tool savings
5. **Remove thinking tools** - Last resort if still over 80 (saves 10-16 tools)

---

## Optimization Strategy

### Phase 1: Remove Servers (Completed ✅)

- ✅ Removed desktop-commander (saved 40-60 tools)
- ✅ Removed notebooklm (saved 10-15 tools)
- **Result:** ~50-75 tools saved

### Phase 2: Additional Optimizations (Pending ⚠️)

**If still over 80 tools:**

1. **Remove context7** - Saves 8-12 tools
   - **Expected:** ~63-101 tools remaining

2. **Investigate tool-level filtering** - For agentic-tools
   - **Potential savings:** 15-30 tools
   - **Expected:** ~48-86 tools remaining

3. **Remove thinking tools** (last resort) - Saves 10-16 tools
   - **Expected:** ~38-70 tools remaining ✅ **WELL UNDER 80**

---

## Configuration Files Modified

1. **`~/.cursor/mcp.json`** (Global)
   - Removed: `desktop-commander`
   - Kept: `tractatus_thinking`, `sequential_thinking`
   - Found: `openmemory` (was already present)

2. **`.cursor/mcp.json`** (Project)
   - Removed: `notebooklm`
   - Added: `semgrep`
   - Kept: `filesystem`, `git`, `agentic-tools`, `context7`

---

## Key Insights

### Tool Count Distribution

- **High tool count servers:**
  - agentic-tools: 30-50 tools (largest remaining)
  - filesystem: 10-15 tools
  - git: 10-15 tools

- **Medium tool count servers:**
  - context7: 8-12 tools (can remove)
  - openmemory: Unknown (needs investigation)

- **Low tool count servers:**
  - tractatus_thinking: 5-8 tools
  - sequential_thinking: 5-8 tools
  - semgrep: 3-5 tools

### Optimization Strategy

1. **Remove entire servers** - Simplest approach (current)
2. **Tool-level filtering** - More granular (if supported)
3. **Use middleware** - MCPJungle for tool filtering (if needed)

---

## Next Session Actions

When returning to this work:

1. **Check actual tool count** in Cursor Settings → MCP Servers
2. **Investigate openmemory:**
   - What is it for?
   - How many tools does it provide?
   - Can it be removed?
3. **If still over 80:**
   - Remove context7 (saves 8-12 tools)
   - Investigate agentic-tools tool filtering (potential 15-30 tool savings)
   - Remove thinking tools as last resort (saves 10-16 tools)

---

## Summary

**Work Completed:**
- ✅ Reviewed all installed MCP servers
- ✅ Analyzed tool counts and optimization opportunities
- ✅ Removed desktop-commander (saved 40-60 tools)
- ✅ Removed notebooklm (saved 10-15 tools)
- ✅ Added semgrep (required by rules)
- ✅ Organized global vs project server placement
- ✅ Created comprehensive documentation

**Current Status:**
- ⚠️ Estimated tool count: 61-113 + openmemory (unknown)
- ⚠️ Need to verify actual count in Cursor
- ⚠️ openmemory needs investigation

**Key Constraint:**
- Todo2 workflow is actively used → Must keep agentic-tools (30-50 tools)

---

## Related Documentation

- [MCP_SERVER_REVIEW.md](MCP_SERVER_REVIEW.md) - Initial comprehensive review
- [MCP_TOOL_COUNT_ANALYSIS_TODO2.md](MCP_TOOL_COUNT_ANALYSIS_TODO2.md) - Optimization strategy with Todo2
- [MCP_TOOL_LEVEL_CHERRY_PICKING.md](MCP_TOOL_LEVEL_CHERRY_PICKING.md) - Tool-level filtering options
- [MCP_POST_RESTART_REVIEW.md](MCP_POST_RESTART_REVIEW.md) - Post-restart status

---

**Session End:** 2025-01-20
**Status:** Archived - Ready for next session
