# MCP Post-Restart Review

**Date:** 2025-01-20
**Status:** After removing desktop-commander and notebooklm, then restarting Cursor
**Goal:** Review actual MCP configuration and identify current tool status

---

## Current Configuration Status

### Global Servers (`~/.cursor/mcp.json`) - 3 servers

1. ✅ `tractatus_thinking` - Universal logical analysis
   - **Type:** npx via uvx/mcpower-proxy
   - **Estimated Tools:** 5-8 tools

2. ✅ `sequential_thinking` - Universal structured problem-solving
   - **Type:** sequential-thinking-mcp (direct command)
   - **Estimated Tools:** 5-8 tools

3. ⚠️ **`openmemory`** - **NEW - NOT IN PREVIOUS ANALYSIS**
   - **Type:** URL-based MCP server (https://api.openmemory.dev/mcp-stream)
   - **Configuration:** Requires API key (Authorization: Token om-{YOUR OPENMEMORY API KEY})
   - **Estimated Tools:** Unknown ⚠️ **NEEDS INVESTIGATION**

**Total:** 3 global servers

### Project Servers (`.cursor/mcp.json`) - 5 servers

1. ✅ `filesystem` - File operations (workspace-scoped)
   - **Estimated Tools:** 10-15 tools
   - **Status:** Essential

2. ✅ `git` - Version control (repository-specific)
   - **Estimated Tools:** 10-15 tools
   - **Status:** Essential

3. ✅ `agentic-tools` - Task management (required for Todo2)
   - **Estimated Tools:** 30-50 tools ⚠️ **LARGEST CONTRIBUTOR**
   - **Status:** Required for Todo2 workflow

4. ⚠️ `context7` - Documentation lookup
   - **Estimated Tools:** 8-12 tools
   - **Status:** Can remove (redundant with web search)

5. ✅ `semgrep` - Security scanning (required by `.cursorrules`)
   - **Estimated Tools:** 3-5 tools
   - **Status:** Required by rules

**Total:** 5 project servers

**Combined Total:** 8 servers (3 global + 5 project)

---

## Changes Summary

### ✅ Servers Removed (From Previous Analysis)

1. ✅ `desktop-commander` - Removed from global config
   - **Tools Saved:** ~40-60 tools
   - **Reason:** Primary tool consumer

2. ✅ `notebooklm` - Removed from project config
   - **Tools Saved:** ~10-15 tools
   - **Reason:** Not actively used

**Total Tools Removed:** ~50-75 tools

### ⚠️ New Discovery: openmemory Server

**Found in Global Config:**
- **Server:** `openmemory`
- **Type:** URL-based MCP server (api.openmemory.dev)
- **Status:** Unknown - not in previous analysis
- **Tool Count:** Unknown ⚠️ **NEEDS INVESTIGATION**

**Questions:**
1. What is openmemory used for?
2. How many tools does it provide?
3. Is it essential or can it be disabled?
4. Why wasn't it in the previous analysis?

---

## Updated Tool Count Estimate

**With openmemory included (unknown count):**

| Server | Location | Estimated Tools | Status |
|--------|----------|----------------|--------|
| **tractatus_thinking** | Global | 5-8 | ✅ Known |
| **sequential_thinking** | Global | 5-8 | ✅ Known |
| **openmemory** | Global | **?** | ⚠️ **UNKNOWN** |
| **filesystem** | Project | 10-15 | ✅ Known |
| **git** | Project | 10-15 | ✅ Known |
| **agentic-tools** | Project | 30-50 | ✅ Known ⚠️ **LARGEST** |
| **context7** | Project | 8-12 | ✅ Known (can remove) |
| **semgrep** | Project | 3-5 | ✅ Known |

**Estimated Total: 61-113 + openmemory (unknown)**

**Without openmemory count:**
- Minimum: 61 tools (if openmemory has 0-5 tools)
- Maximum: 113+ tools (if openmemory has 10+ tools)

---

## Priority Actions

### 1. ⚠️ CRITICAL: Investigate openmemory

**Action Required:**
- Check what openmemory does
- Determine tool count
- Assess if it's essential
- Decide if it should be removed

**Questions to Answer:**
1. What is openmemory used for?
2. How many tools does it provide?
3. Is it essential for your workflow?
4. Can it be disabled to save tools?

**Investigation Steps:**
```bash
# Check if openmemory is in project documentation
grep -r "openmemory" docs/

# Check if there's any mention of openmemory in rules
grep -r "openmemory" .cursor/rules/
```

### 2. Verify Actual Tool Count in Cursor

**Action:**
1. Open Cursor Settings → MCP Servers
2. Check actual tool count displayed
3. Compare with estimated counts
4. Identify which server is contributing most tools

### 3. If Still Over 80 Tools

**Priority Removal Order:**
1. ⚠️ **openmemory** - If tool count is high and not essential (unknown impact)
2. 🔴 **context7** - Known 8-12 tools, redundant with web search
3. 🟡 **agentic-tools advanced tools** - If tool filtering supported (potential 15-30 tools)
4. 🟢 **thinking tools** - Last resort (saves 10-16 tools)

---

## Next Steps

### Immediate Actions

1. ✅ **Review this document** - Understand current state
2. ⚠️ **Investigate openmemory** - Determine purpose and tool count
3. ⚠️ **Check Cursor Settings** - Verify actual tool count in Cursor → Settings → MCP Servers
4. ⚠️ **Report back tool count** - Let me know the actual count shown in Cursor

### If Still Over 80 Tools

**Phase 1: Quick Wins**
- Remove `context7` (saves 8-12 tools)
- Investigate `openmemory` removal (unknown savings)

**Phase 2: Tool Filtering**
- Investigate `agentic-tools` tool filtering support
- Disable advanced tools if supported (potential 15-30 tool savings)

**Phase 3: Last Resort**
- Remove thinking tools if still over 80 (saves 10-16 tools)

---

## Summary

**Current Status:**
- ✅ Removed desktop-commander and notebooklm (saved ~50-75 tools)
- ⚠️ Discovered openmemory server (unknown impact - needs investigation)
- ⚠️ Estimated remaining: 61-113 + openmemory tools

**Key Finding:**
- ⚠️ **openmemory** was not in previous analysis - may be contributing significant tools
- Needs investigation to determine impact

**Immediate Action:**
1. Check Cursor Settings → MCP Servers for actual tool count
2. Investigate openmemory purpose and tool count
3. Report back with actual numbers

---

## Questions for You

1. **What tool count is shown in Cursor Settings → MCP Servers?**
   - This will tell us if we're still over 80

2. **Do you use openmemory? What is it for?**
   - Helps determine if it's essential or can be removed

3. **Which server shows the most tools in Cursor Settings?**
   - This will help prioritize optimization efforts
