# MCP Tool-Level Cherry Picking Analysis

**Date:** 2025-01-20
**Question:** Can we disable individual tools within each MCP server instead of removing entire servers?
**Goal:** Identify which tools can be selectively disabled to optimize tool count while keeping useful servers

---

## Feasibility Summary

### ✅ Tool-Level Selection IS Possible (with limitations)

**Status:** Partially supported - requires server-specific configuration or middleware

**Options:**

1. **Server-Specific Configuration** - If server supports it (varies by server)
2. **MCP Middleware** - MCPJungle or similar to filter tools
3. **Standard Cursor Config** - ❌ NOT directly supported (server-level only)

---

## Tool-Level Analysis by Server

### 🎯 Priority Target: **agentic-tools** (30-50 tools)

**Potential Savings: 15-30 tools** by disabling advanced features

#### Essential Tools (Keep - ~15-20 tools)

- **Project CRUD:** `list_projects`, `create_project`, `get_project`, `update_project`, `delete_project`
- **Task CRUD:** `create_task`, `update_task`, `get_task`, `list_tasks`, `delete_task`
- **Subtask CRUD:** `create_subtask`, `update_subtask`, `get_subtask`, `list_subtasks`, `delete_subtask`
- **Memory CRUD:** `create_memory`, `get_memory`, `list_memories`, `search_memories`, `update_memory`, `delete_memory`

#### Advanced Tools (Can Disable - ~15-30 tools)

- `get_next_task_recommendation` - Can manually select tasks
- `analyze_task_complexity` - Nice to have, not essential
- `infer_task_progress` - Automated, may not be needed
- `parse_prd` - Only if not using PRDs
- `research_task` - Can use web search instead
- `generate_research_queries` - Can generate manually
- `migrate_subtasks` - One-time operation
- `move_task` - Can reorganize manually

**Potential Savings:** 15-30 tools

---

## Implementation Approaches

### Approach 1: Check Server Documentation

**First Step:** Check if `agentic-tools` supports tool-level configuration

```bash

# Check agentic-tools documentation

npm info @pimzino/agentic-tools-mcp

# Or check GitHub repository
# https://github.com/Pimzino/agentic-tools-mcp
```

**Look For:**

- Configuration file support
- Environment variables to disable tools
- Command-line flags for tool filtering
- Tool enable/disable settings

### Approach 2: MCPJungle Middleware (If Server Doesn't Support)

**What is MCPJungle:**

- MCP middleware that allows creating "Tool Groups"
- Exposes only specified tools from connected servers
- Acts as a proxy between Cursor and MCP servers

**Configuration Example:**

```json
{
  "name": "optimized-tools",
  "description": "Selected essential tools only",
  "included_tools": [
    "agentic-tools__create_task",
    "agentic-tools__update_task",
    "agentic-tools__get_task",
    "agentic-tools__list_tasks",
    "agentic-tools__create_project",
    "agentic-tools__create_memory",
    "agentic-tools__search_memories",
    "filesystem__read_file",
    "filesystem__write_file",
    "filesystem__list_directory",
    "git__get_status",
    "git__get_diff",
    "git__create_commit",
    "semgrep__scan_file"
  ],
  "excluded_tools": [
    "agentic-tools__get_next_task_recommendation",
    "agentic-tools__analyze_task_complexity",
    "agentic-tools__infer_task_progress",
    "agentic-tools__parse_prd",
    "agentic-tools__research_task"
  ]
}
```

### Approach 3: Server Configuration File (If Supported)

**For agentic-tools (if supported):**

```json
// .cursor/agentic-tools-config.json
{
  "disabled_tools": [
    "get_next_task_recommendation",
    "analyze_task_complexity",
    "infer_task_progress",
    "parse_prd",
    "research_task",
    "generate_research_queries"
  ]
}
```

**Update mcp.json:**

```json
{
  "agentic-tools": {
    "command": "npx",
    "args": [
      "-y",
      "@pimzino/agentic-tools-mcp",
      "--config",
      ".cursor/agentic-tools-config.json"
    ]
  }
}
```

---

## Other Servers (Low Priority)

### **filesystem** (10-15 tools) - All essential

- Keep all tools (low tool count)
- **Potential Savings:** 2-5 tools (minimal impact)

### **git** (10-15 tools) - All essential

- Keep all tools (low tool count)
- **Potential Savings:** 2-5 tools (minimal impact)

### **context7** (8-12 tools) - Better to remove entire server

- **Recommendation:** Remove entire server (simpler)
- **Potential Savings:** 8-12 tools (same as cherry-picking)

### **semgrep** (3-5 tools) - All essential

- Keep all tools (required by rules)
- **Potential Savings:** 0 tools

### **tractatus_thinking** (5-8 tools) - All essential

- Keep all tools (low tool count)
- **Potential Savings:** 0 tools

### **sequential_thinking** (5-8 tools) - All essential

- Keep all tools (low tool count)
- **Potential Savings:** 0 tools

---

## Recommendation

### ✅ Hybrid Strategy (Best Approach)

**Phase 1: Remove Low-Value Servers (Current Approach)**

1. ✅ Remove `desktop-commander` (saved 40-60 tools) - Already done
2. ✅ Remove `notebooklm` (saved 10-15 tools) - Already done
3. ⚠️ Remove `context7` (save 8-12 tools) - Recommended next

**Phase 2: Investigate Tool-Level Filtering for agentic-tools**

**Action Plan:**

1. **Check agentic-tools documentation:**
   - Does it support tool-level configuration?
   - Does it support environment variables?
   - Does it support command-line flags?

2. **If supported:**
   - Create config file to disable advanced tools
   - Update `mcp.json` to point to config
   - Test and verify tool count reduction
   - **Potential Savings:** 15-30 tools

3. **If not supported:**
   - Consider MCPJungle middleware
   - Or remove entire server if Todo2 not critical
   - **Fallback:** Remove thinking tools as last resort

**Phase 3: Verify Tool Count**

**Expected Results:**

- After Phase 1: ~71-113 tools (may still exceed 80)
- After Phase 2 (if agentic-tools supports filtering): ~56-83 tools ✅ **SHOULD BE UNDER 80**
- If still over 80: Remove thinking tools (saves 10-16 tools) → ~46-73 tools ✅ **WELL UNDER 80**

---

## Expected Tool Count Scenarios

### Scenario A: Remove Servers Only (Current Approach)

- Removed: desktop-commander (40-60), notebooklm (10-15), context7 (8-12)
- **Total: 63-101 tools** ⚠️ **MAY STILL EXCEED 80**

### Scenario B: Remove Servers + Tool Filtering (Optimal)

- Removed: desktop-commander (40-60), notebooklm (10-15), context7 (8-12)
- Disabled advanced tools in agentic-tools (15-30)
- **Total: 48-86 tools** ⚠️ **MAY STILL EXCEED IF AGENTIC-TOOLS HAS 50 TOOLS**

### Scenario C: Remove Servers + Tool Filtering + Thinking Tools (Minimal)

- Removed: desktop-commander (40-60), notebooklm (10-15), context7 (8-12), thinking tools (10-16)
- Disabled advanced tools in agentic-tools (15-30)
- **Total: 38-70 tools** ✅ **SHOULD BE UNDER 80**

---

## Next Steps

### Immediate Action

1. **Remove context7** (simple, saves 8-12 tools)
   - Edit `.cursor/mcp.json`
   - Remove `context7` entry
   - Restart Cursor

2. **Check agentic-tools documentation** for tool filtering support
   - Review GitHub repository
   - Check for config file examples
   - Look for environment variable documentation

3. **If tool filtering supported:**
   - Create config file to disable advanced tools
   - Update configuration
   - Test and verify

4. **If tool filtering not supported:**
   - Consider MCPJungle middleware
   - Or proceed with server removal only

---

## Summary

**Cherry-Picking Tools:**

- ✅ **Possible** but requires server-specific config or middleware
- ⚠️ **Not directly supported** in Cursor's standard `mcp.json`
- 🎯 **Best Target:** `agentic-tools` (potential 15-30 tool savings)

**Recommendation:**

1. **First:** Remove `context7` (saves 8-12 tools) - Simple, immediate
2. **Then:** Investigate `agentic-tools` tool filtering (potential 15-30 tool savings)
3. **Last Resort:** Remove thinking tools if still over 80 (saves 10-16 tools)

**Potential Final Tool Count:**

- **With server removal only:** 63-101 tools (may still exceed)
- **With server removal + tool filtering:** 48-86 tools (should be under 80)
- **With server removal + tool filtering + thinking tools removed:** 38-70 tools ✅ **WELL UNDER 80**
