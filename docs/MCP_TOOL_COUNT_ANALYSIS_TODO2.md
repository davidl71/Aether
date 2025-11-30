# MCP Tool Count Analysis - With Todo2 Workflow (UPDATED)

**Date:** 2025-01-20
**Issue:** Exceeding 80-tool limit across all MCP servers
**Constraint:** Todo2 workflow is actively used - must keep `agentic-tools`
**Goal:** Identify which servers/tools can be removed while keeping Todo2 functionality

---

## Current Status After Desktop Commander Removal

### Global Servers (`~/.cursor/mcp.json`) - 2 servers

1. ✅ **tractatus_thinking** - Logical concept analysis (5-8 tools)
2. ✅ **sequential_thinking** - Structured problem-solving (5-8 tools)

### Project Servers (`.cursor/mcp.json`) - 6 servers

1. ✅ **filesystem** - File operations (10-15 tools) - **ESSENTIAL**
2. ✅ **git** - Version control (10-15 tools) - **ESSENTIAL**
3. ✅ **agentic-tools** - Task management (30-50 tools) - **REQUIRED FOR TODO2** ⚠️ **MUST KEEP**
4. ⚠️ **context7** - Documentation lookup (8-12 tools) - **CAN REMOVE**
5. ⚠️ **notebooklm** - Research & documentation (10-15 tools) - **EVALUATE**
6. ✅ **semgrep** - Security scanning (3-5 tools) - **REQUIRED BY RULES**

**Total: 8 MCP servers (2 global + 6 project)**

---

## Updated Tool Count Estimates (With Todo2 Constraint)

### ✅ Must Keep (63-101 tools)

| Server | Estimated Tools | Why Keep | Required By |
|--------|----------------|----------|-------------|
| **agentic-tools** | **30-50** | Todo2 workflow | ✅ **MANDATORY** - Todo2 is actively used |
| **filesystem** | 10-15 | Core file operations | ✅ Essential |
| **git** | 10-15 | Version control | ✅ Essential |
| **semgrep** | 3-5 | Security scanning | ✅ `.cursorrules` requirement |
| **tractatus_thinking** | 5-8 | Universal thinking tool | ✅ Important |
| **sequential_thinking** | 5-8 | Universal thinking tool | ✅ Important |

**Must Keep Total: 63-101 tools**

### ⚠️ Can Remove (18-27 tools)

| Server | Estimated Tools | Priority to Remove | Rationale |
|--------|----------------|-------------------|-----------|
| **context7** | **8-12** | 🔴 **HIGH** | Redundant with web search |
| **notebooklm** | **10-15** | 🟡 **MEDIUM** | Remove if not actively using TWS API notebook |

**Can Remove Total: 18-27 tools**

---

## Tool Count After Desktop Commander Removal

**Estimated Current Total: 81-128 tools** ⚠️ **STILL EXCEEDS 80 LIMIT**

**Breakdown:**

- Must keep: 63-101 tools
- Can remove: 18-27 tools
- **Total:** 81-128 tools

---

## Optimization Strategy (With Todo2 Constraint)

Since Todo2 is actively used, `agentic-tools` (30-50 tools) must be kept. This means we need to be more aggressive about removing other servers.

### Phase 1: Remove Low-Value Servers (Saves 18-27 tools)

#### Priority 1: Remove Context7 (Saves 8-12 tools)

**Rationale:**

- Redundant with web search
- Low value for C++ trading project
- Can use web search for library documentation

**Action:** Remove `context7` from project config

**Expected Result:** Tool count drops to ~73-116 tools

#### Priority 2: Evaluate NotebookLM (Saves 10-15 tools if removed)

**Decision Criteria:**

- ✅ **Keep if:** Actively using TWS API notebook regularly
- ❌ **Remove if:** Not actively querying notebook

**Action:** Remove `notebooklm` from project config if not actively using

**Expected Result (if removed):** Tool count drops to ~63-101 tools

**Recommendation:**

- If you actively query the TWS API notebook → **KEEP** (saves research time)
- If you rarely use the notebook → **REMOVE** (saves 10-15 tools, brings count under 80)

---

## Expected Tool Count After Optimizations

### Option A: Remove Context7 Only (Conservative)

**Keep:**

- ✅ All must-keep servers (63-101 tools)
- ✅ notebooklm (10-15 tools)

**Remove:**

- ❌ context7 (8-12 tools)

**Result: 73-116 tools** ⚠️ **MAY STILL EXCEED 80 IF AGENTIC-TOOLS HAS 50+ TOOLS**

### Option B: Remove Context7 + NotebookLM (Optimal - Recommended)

**Keep:**

- ✅ All must-keep servers (63-101 tools)

**Remove:**

- ❌ context7 (8-12 tools)
- ❌ notebooklm (10-15 tools)

**Result: 63-101 tools** ⚠️ **MAY STILL EXCEED 80 IF AGENTIC-TOOLS HAS 50+ TOOLS**

**Recommendation:** 🔴 **TRY THIS FIRST** - Most likely to get under 80

### Option C: Minimal Setup (If Still Over 80)

If Option B still exceeds 80 tools (if agentic-tools has 50+ tools), consider:

**Keep Only:**

- ✅ filesystem (10-15 tools)
- ✅ git (10-15 tools)
- ✅ agentic-tools (30-50 tools) - **REQUIRED FOR TODO2**
- ✅ semgrep (3-5 tools)

**Remove:**

- ❌ tractatus_thinking (5-8 tools)
- ❌ sequential_thinking (5-8 tools)
- ❌ context7 (8-12 tools)
- ❌ notebooklm (10-15 tools)

**Result: 53-85 tools** ✅ **SHOULD BE UNDER 80** (if agentic-tools is closer to 30 tools)

**Trade-off:** Lose thinking tools (can use web search for similar functionality)

---

## Recommended Action Plan (With Todo2)

### Step 1: Remove Context7 (Immediate - Saves 8-12 tools)

**Rationale:**

- Redundant with web search
- Low value for trading project
- Easy removal

**Action:**

1. Edit `.cursor/mcp.json` (project config)
2. Remove `context7` entry
3. Save and restart Cursor

**Expected Result:** Tool count drops by 8-12 tools

### Step 2: Evaluate NotebookLM Usage

**Question:** How often do you query the TWS API notebook?

**If frequently (> once per day):**

- ✅ **KEEP** notebooklm
- Tool count: ~73-116 tools
- May still exceed 80 if agentic-tools is large

**If rarely (< once per week):**

- ❌ **REMOVE** notebooklm
- Tool count: ~63-101 tools
- Should be under 80 if agentic-tools is closer to 30 tools

### Step 3: Verify Tool Count

After removing context7 (+ notebooklm if removed):

1. Restart Cursor
2. Check tool count in Cursor Settings → MCP Servers
3. If under 80 → ✅ Done!
4. If still over 80 → Proceed to Step 4

### Step 4: Last Resort - Review Thinking Tools (Only if still over 80)

**If still over 80 after removing context7 + notebooklm:**

Consider removing thinking tools (only as last resort):

**Remove:**

- ❌ tractatus_thinking (5-8 tools)
- ❌ sequential_thinking (5-8 tools)

**Impact:**

- ✅ Saves 10-16 tools
- ❌ Lose structured problem-solving tools
- ✅ Can use web search for similar thinking patterns

**Final Result:** 53-85 tools ✅ **SHOULD BE UNDER 80**

---

## Tool Count Summary Table

| Configuration | Estimated Tools | Status | Recommendation |
|--------------|----------------|--------|----------------|
| **After removing desktop-commander** | **81-128** | ⚠️ May exceed | Remove context7 |
| **After removing desktop-commander + context7** | **73-116** | ⚠️ May exceed | Remove notebooklm if not using |
| **After removing desktop-commander + context7 + notebooklm** | **63-101** | ⚠️ May exceed | Remove thinking tools if still over |
| **After removing desktop-commander + context7 + notebooklm + thinking tools** | **53-85** | ✅ Should be under 80 | Last resort option |

---

## Final Recommendation (With Todo2 Active)

### ✅ Recommended Configuration

**Keep (6 servers - 63-101 tools):**

1. ✅ **agentic-tools** - **REQUIRED** - Todo2 workflow (30-50 tools)
2. ✅ **filesystem** - Essential file operations (10-15 tools)
3. ✅ **git** - Essential version control (10-15 tools)
4. ✅ **semgrep** - Required by rules (3-5 tools)
5. ✅ **tractatus_thinking** - Universal thinking tool (5-8 tools)
6. ✅ **sequential_thinking** - Universal thinking tool (5-8 tools)

**Remove (2 servers - 18-27 tools):**

1. ❌ **context7** - Redundant with web search (8-12 tools)
2. ❌ **notebooklm** - Remove if not actively using TWS API notebook (10-15 tools)

**Total: 63-101 tools**

**Note:** If agentic-tools has 50+ tools, this may still exceed 80. In that case, remove thinking tools as last resort.

---

## Decision Tree

```
Start: 81-128 tools (after removing desktop-commander)
│
├─> Remove context7 (saves 8-12 tools)
│   └─> Result: 73-116 tools
│       │
│       ├─> If actively using NotebookLM TWS API notebook
│       │   └─> Keep notebooklm
│       │       └─> Result: 73-116 tools ⚠️ May still exceed
│       │           └─> If still over 80: Remove thinking tools (saves 10-16 tools)
│       │               └─> Result: 63-100 tools ✅ Should be under 80
│       │
│       └─> If NOT actively using NotebookLM
│           └─> Remove notebooklm (saves 10-15 tools)
│               └─> Result: 63-101 tools ⚠️ May still exceed
│                   └─> If still over 80: Remove thinking tools (saves 10-16 tools)
│                       └─> Result: 53-85 tools ✅ Should be under 80
```

---

## Implementation Steps

### Immediate Action: Remove Context7

1. Edit `.cursor/mcp.json` (project config)
2. Remove the `context7` entry
3. Save file
4. Restart Cursor
5. Verify tool count

### Next: Evaluate NotebookLM

**Decision:** Are you actively using the TWS API notebook?

- **Yes (frequently):** Keep notebooklm
- **No (rarely):** Remove notebooklm

### If Still Over 80: Remove Thinking Tools (Last Resort)

1. Remove `tractatus_thinking` from global config
2. Remove `sequential_thinking` from global config
3. Save files
4. Restart Cursor
5. Verify tool count is now under 80

---

## Summary

**Key Constraint:** Todo2 is actively used → Must keep `agentic-tools` (30-50 tools)

**Recommendation:**

1. ✅ **Remove context7** (saves 8-12 tools) - Immediate action
2. ⚠️ **Evaluate notebooklm** (saves 10-15 tools if removed) - Decision based on usage
3. 🔴 **Remove thinking tools** (saves 10-16 tools) - Last resort if still over 80

**Expected Final Tool Count:** 53-101 tools (should be under 80 after optimizations)

**Priority Order:**

1. Remove context7 (immediate)
2. Remove notebooklm if not using (if needed)
3. Remove thinking tools if still over 80 (last resort)
