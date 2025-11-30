# MCP Tool Count Analysis - 80 Tool Limit Optimization

**Date:** 2025-01-20
**Issue:** Exceeding 80-tool limit across all MCP servers
**Goal:** Identify which individual tools within each MCP server can be disabled

---

## Current MCP Server Configuration

### Global Servers (`~/.cursor/mcp.json`) - 3 servers

1. **tractatus_thinking** - Logical concept analysis
2. **desktop-commander** - System-level operations ⚠️ **LIKELY HIGH TOOL COUNT**
3. **sequential_thinking** - Structured problem-solving

### Project Servers (`.cursor/mcp.json`) - 6 servers

1. **filesystem** - File operations (workspace-scoped)
2. **git** - Version control (repository-specific)
3. **agentic-tools** - Task management ⚠️ **LIKELY HIGH TOOL COUNT**
4. **context7** - Documentation lookup
5. **notebooklm** - Research & documentation
6. **semgrep** - Security scanning

**Total: 9 MCP servers**

---

## Estimated Tool Counts by Server

### 🔴 HIGH TOOL COUNT SERVERS (Likely 30+ tools each)

#### 1. **desktop-commander** ⚠️ **PRIMARY SUSPECT**

**Estimated Tools: 40-60+ tools**

**Tool Categories:**

- File operations (read, write, create, delete, move, copy, search)
- Directory operations (list, create, delete, tree)
- Terminal/process operations (start, interact, read output, kill)
- System commands (execute shell commands)
- File information (metadata, permissions, sizes)
- Search operations (file search, content search)
- Network operations (HTTP requests, downloads)
- System information (processes, environment)

**Why So Many Tools:**

- Comprehensive system operations server
- Replaces multiple specialized servers
- Designed to be a "swiss army knife" for system operations

**Recommendation:** 🔴 **HIGH PRIORITY TO DISABLE OR REPLACE**

- Consider using `filesystem` server instead for file operations
- Consider using regular terminal commands instead of process tools
- This server likely accounts for 40-60 tools (50-75% of limit!)

#### 2. **agentic-tools** ⚠️ **SECONDARY SUSPECT**

**Estimated Tools: 30-50+ tools**

**Tool Categories:**

- Project management (list, create, update, delete projects)
- Task management (create, update, delete tasks)
- Subtask management (unlimited nesting)
- Memory operations (create, search, update, delete memories)
- Task recommendations
- Complexity analysis
- Progress inference
- PRD parsing
- Research tools
- Dependency management

**Why So Many Tools:**

- Comprehensive task management system
- Supports complex workflows (Todo2)
- Multiple features (tasks, memories, research, analysis)

**Recommendation:** 🟡 **REVIEW & OPTIMIZE**

- Essential for Todo2 workflow (mandatory per rules)
- Could potentially disable some advanced features if not used
- If Todo2 workflow is not actively used, this server could be removed

### 🟡 MEDIUM TOOL COUNT SERVERS (10-20 tools each)

#### 3. **notebooklm**

**Estimated Tools: 10-15 tools**

**Tool Categories:**

- Notebook management (list, create, update, remove, search)
- Question asking (query notebooks)
- Session management
- Notebook selection
- Resource access

**Recommendation:** 🟢 **KEEP IF USING**

- Remove if not actively querying TWS API notebook
- Nice to have but not essential

#### 4. **context7**

**Estimated Tools: 8-12 tools**

**Tool Categories:**

- Library ID resolution
- Documentation retrieval
- Version-specific docs
- Code examples

**Recommendation:** 🔴 **REMOVE** (Redundant with web search)

- Low tool count but redundant functionality
- Web search provides similar documentation access
- Saves 8-12 tools

#### 5. **filesystem**

**Estimated Tools: 10-15 tools**

**Tool Categories:**

- File read/write operations
- Directory listing
- File search
- File metadata
- Directory tree
- File operations

**Recommendation:** ✅ **KEEP** (Essential core functionality)

- Required for AI file operations
- Not excessive tool count
- Core functionality

#### 6. **git**

**Estimated Tools: 10-15 tools**

**Tool Categories:**

- Git status, diff, log
- Branch operations
- Commit operations
- Repository management
- History operations

**Recommendation:** ✅ **KEEP** (Essential core functionality)

- Required for version control
- Not excessive tool count
- Core functionality

### 🟢 LOW TOOL COUNT SERVERS (5-10 tools each)

#### 7. **tractatus_thinking**

**Estimated Tools: 5-8 tools**

**Tool Categories:**

- Start analysis
- Add propositions
- Navigate structure
- Export results
- Revise understanding

**Recommendation:** ✅ **KEEP** (Low tool count, high value)

- Universal thinking tool
- Low tool count
- Important for complex analysis

#### 8. **sequential_thinking**

**Estimated Tools: 5-8 tools**

**Tool Categories:**

- Start sequential analysis
- Add steps
- Refine steps
- Navigate workflow
- Export workflow

**Recommendation:** ✅ **KEEP** (Low tool count, high value)

- Universal thinking tool
- Low tool count
- Complements tractatus_thinking

#### 9. **semgrep**

**Estimated Tools: 3-5 tools**

**Tool Categories:**

- Security scanning
- Code analysis
- Vulnerability detection

**Recommendation:** ✅ **KEEP** (Required by rules, low tool count)

- Required by `.cursorrules`
- Low tool count
- Critical for security

---

## Tool Count Summary

| Server | Estimated Tools | Priority | Action |
|--------|----------------|----------|--------|
| **desktop-commander** | **40-60** | 🔴 **HIGH** | ⚠️ **REMOVE OR REPLACE** |
| **agentic-tools** | **30-50** | 🟡 **MEDIUM** | ⚠️ **REVIEW** |
| **notebooklm** | **10-15** | 🟢 **LOW** | 🟢 **REMOVE IF NOT USING** |
| **context7** | **8-12** | 🔴 **HIGH** | 🔴 **REMOVE** (Redundant) |
| **filesystem** | **10-15** | ✅ **KEEP** | ✅ **ESSENTIAL** |
| **git** | **10-15** | ✅ **KEEP** | ✅ **ESSENTIAL** |
| **tractatus_thinking** | **5-8** | ✅ **KEEP** | ✅ **LOW COUNT** |
| **sequential_thinking** | **5-8** | ✅ **KEEP** | ✅ **LOW COUNT** |
| **semgrep** | **3-5** | ✅ **KEEP** | ✅ **REQUIRED** |

**Estimated Total: 121-188 tools** ⚠️ **EXCEEDS 80 TOOL LIMIT BY 41-108 TOOLS**

---

## Optimization Strategy

### Phase 1: Remove High-Tool-Count Servers (Saves 48-72 tools)

#### Option A: Remove Desktop Commander (Saves 40-60 tools)

**Rationale:**

- Desktop Commander is likely the biggest contributor (40-60 tools)
- Many operations can be done via filesystem server or terminal
- Removes largest tool consumer

**Impact:**

- ✅ Reduces tool count by 40-60 tools
- ❌ Loses system-level operations outside workspace
- ✅ Filesystem server covers workspace file operations

**Alternative Solutions:**

- Use `filesystem` server for workspace file operations
- Use regular terminal commands for system operations
- Desktop Commander mostly redundant with filesystem + terminal

**Recommendation:** 🔴 **REMOVE** - Desktop Commander is the primary tool consumer

#### Option B: Remove Context7 (Saves 8-12 tools)

**Rationale:**

- Redundant with web search
- Low value for C++ trading project
- Documentation can be found via web search

**Impact:**

- ✅ Reduces tool count by 8-12 tools
- ❌ Loses version-specific documentation lookup
- ✅ Web search provides similar functionality

**Recommendation:** 🔴 **REMOVE** - Low value, redundant

### Phase 2: Review Medium-Tool-Count Servers (Saves 10-15 tools)

#### Option C: Remove NotebookLM (Saves 10-15 tools)

**Rationale:**

- Only useful if actively querying TWS API notebook
- Can use web search for general research
- Medium tool count

**Impact:**

- ✅ Reduces tool count by 10-15 tools
- ❌ Loses TWS API knowledge base notebook
- ✅ Web search can provide similar research

**Decision:**

- **Remove if:** Not actively using TWS API notebook
- **Keep if:** Actively querying notebook regularly

**Recommendation:** 🟢 **REMOVE IF NOT USING** - Decision based on usage

### Phase 3: Review Agentic Tools (Saves 30-50 tools if removed)

#### Option D: Review Agentic Tools Usage

**Rationale:**

- Second-largest tool consumer (30-50 tools)
- Required by Todo2 workflow (mandatory per rules)
- Can be kept if Todo2 workflow is actively used

**Impact:**

- ✅ Reduces tool count by 30-50 tools if removed
- ❌ Loses Todo2 workflow (violates rules)
- ❌ Loses task management and memories

**Decision:**

- **Keep if:** Todo2 workflow is mandatory and actively used
- **Remove if:** Todo2 workflow is not being used

**Recommendation:** 🟡 **KEEP IF USING TODO2** - Required by workflow rules

---

## Recommended Removal Order

### 🔴 Priority 1: Remove Desktop Commander (40-60 tools)

**Action:** Remove `desktop-commander` from global config

**Expected Savings:** 40-60 tools

**Alternative:**

- Use `filesystem` server for workspace file operations
- Use regular terminal commands for system operations

**After removal:** ~41-128 tools remaining (still may exceed limit)

### 🔴 Priority 2: Remove Context7 (8-12 tools)

**Action:** Remove `context7` from project config

**Expected Savings:** 8-12 tools

**Alternative:**

- Use web search for library documentation
- Use official documentation websites

**After removal:** ~33-116 tools remaining

### 🟡 Priority 3: Evaluate NotebookLM (10-15 tools)

**Decision:**

- **Remove if:** Not actively using TWS API notebook
- **Keep if:** Actively querying notebook regularly

**Action:** Remove `notebooklm` from project config if not using

**Expected Savings:** 10-15 tools (if removed)

**Alternative:**

- Use web search for general research
- Create documentation from research manually

**After removal:** ~23-101 tools remaining (should be under 80 if all removed)

### 🟢 Priority 4: Review Agentic Tools (30-50 tools)

**Decision:**

- **Keep if:** Todo2 workflow is mandatory and actively used
- **Remove if:** Todo2 workflow is not being used

**Action:** Review Todo2 usage - remove if not needed

**Expected Savings:** 30-50 tools (if removed)

**Alternative:**

- Use simpler task management (GitHub issues, linear tasks)
- Use regular TODO comments in code

**After removal:** ~0-51 tools remaining (well under 80 if removed)

---

## Quick Win Recommendations

### ✅ Immediate Actions (Saves 48-72 tools)

1. **Remove desktop-commander** (40-60 tools) - Primary tool consumer
2. **Remove context7** (8-12 tools) - Redundant with web search

**Total Savings: 48-72 tools**

**Result:** ~33-82 tools remaining (may still exceed limit)

### ✅ If Still Over Limit (Saves additional 10-15 tools)

3. **Remove notebooklm** (10-15 tools) - If not actively using

**Total Savings: 58-87 tools**

**Result:** ~23-72 tools remaining (should be under 80)

### ✅ Last Resort (Saves additional 30-50 tools)

4. **Remove agentic-tools** (30-50 tools) - Only if Todo2 not used

**Total Savings: 88-137 tools**

**Result:** ~0-51 tools remaining (well under 80)

---

## Final Configuration Options

### Option 1: Minimal Setup (Well Under 80 Tools)

**Servers:**

- ✅ filesystem (10-15 tools)
- ✅ git (10-15 tools)
- ✅ semgrep (3-5 tools)
- ✅ tractatus_thinking (5-8 tools)
- ✅ sequential_thinking (5-8 tools)

**Total: 33-51 tools** ✅ **WELL UNDER 80**

**Removed:**

- ❌ desktop-commander (40-60 tools)
- ❌ agentic-tools (30-50 tools)
- ❌ context7 (8-12 tools)
- ❌ notebooklm (10-15 tools)

### Option 2: Optimal Setup (Under 80 Tools)

**Servers:**

- ✅ filesystem (10-15 tools)
- ✅ git (10-15 tools)
- ✅ semgrep (3-5 tools)
- ✅ tractatus_thinking (5-8 tools)
- ✅ sequential_thinking (5-8 tools)
- ✅ agentic-tools (30-50 tools) - Keep if using Todo2

**Total: 63-101 tools** ⚠️ **MAY EXCEED 80 IF AGENTIC-TOOLS HAS 50+ TOOLS**

**Removed:**

- ❌ desktop-commander (40-60 tools)
- ❌ context7 (8-12 tools)
- ❌ notebooklm (10-15 tools)

### Option 3: Balanced Setup (Target 70-75 Tools)

**Servers:**

- ✅ filesystem (10-15 tools)
- ✅ git (10-15 tools)
- ✅ semgrep (3-5 tools)
- ✅ tractatus_thinking (5-8 tools)
- ✅ sequential_thinking (5-8 tools)
- ✅ notebooklm (10-15 tools) - Keep if using TWS API notebook

**Total: 43-66 tools** ✅ **UNDER 80**

**Removed:**

- ❌ desktop-commander (40-60 tools)
- ❌ agentic-tools (30-50 tools) - Remove if Todo2 not used
- ❌ context7 (8-12 tools)

---

## Implementation Steps

### Step 1: Remove Desktop Commander (Immediate - Saves 40-60 tools)

1. Edit `~/.cursor/mcp.json` (global config)
2. Remove `desktop-commander` entry
3. Save and restart Cursor

**Expected Result:** Tool count drops by 40-60 tools

### Step 2: Remove Context7 (Immediate - Saves 8-12 tools)

1. Edit `.cursor/mcp.json` (project config)
2. Remove `context7` entry
3. Save and restart Cursor

**Expected Result:** Tool count drops by additional 8-12 tools

### Step 3: Verify Tool Count

1. Restart Cursor
2. Check Cursor Settings → MCP Servers
3. Verify tool count is under 80
4. If still over, proceed to Step 4

### Step 4: Evaluate NotebookLM Usage

**Decision Point:**

- Actively using TWS API notebook? → Keep
- Not using notebook? → Remove

**If removing:**

1. Edit `.cursor/mcp.json` (project config)
2. Remove `notebooklm` entry
3. Save and restart Cursor

**Expected Result:** Tool count drops by additional 10-15 tools

### Step 5: Last Resort - Review Agentic Tools

**Decision Point:**

- Todo2 workflow mandatory and used? → Keep
- Todo2 workflow not used? → Remove

**If removing:**

1. Edit `.cursor/mcp.json` (project config)
2. Remove `agentic-tools` entry
3. Update `.cursorrules` to remove Todo2 requirements
4. Save and restart Cursor

**Expected Result:** Tool count drops by additional 30-50 tools

---

## Expected Tool Count After Optimizations

| Configuration | Estimated Tools | Status |
|--------------|----------------|--------|
| **Current** | **121-188** | ❌ **EXCEEDS LIMIT** |
| **After removing desktop-commander** | **81-128** | ⚠️ **MAY EXCEED** |
| **After removing desktop-commander + context7** | **73-116** | ⚠️ **MAY EXCEED** |
| **After removing desktop-commander + context7 + notebooklm** | **63-101** | ⚠️ **MAY EXCEED** |
| **After removing desktop-commander + context7 + notebooklm + agentic-tools** | **33-51** | ✅ **WELL UNDER 80** |

---

## Recommendations Summary

### 🔴 Must Remove (Saves 48-72 tools)

1. **desktop-commander** - Primary tool consumer (40-60 tools)
2. **context7** - Redundant with web search (8-12 tools)

### 🟡 Evaluate & Remove if Not Using (Saves 10-15 tools)

3. **notebooklm** - Remove if not actively using TWS API notebook

### 🟢 Keep If Using (Required by rules)

4. **agentic-tools** - Keep if Todo2 workflow is mandatory and used

### ✅ Keep (Essential Core Functionality)

5. **filesystem** - Essential for file operations
6. **git** - Essential for version control
7. **semgrep** - Required by `.cursorrules`
8. **tractatus_thinking** - Universal thinking tool
9. **sequential_thinking** - Universal thinking tool

---

## Next Steps

1. **Remove desktop-commander** from global config (immediate - saves 40-60 tools)
2. **Remove context7** from project config (immediate - saves 8-12 tools)
3. **Restart Cursor** and verify tool count
4. **If still over 80:** Remove notebooklm (saves 10-15 tools)
5. **If still over 80:** Review agentic-tools usage (saves 30-50 tools if removed)

**Target:** Get under 80 tools while maintaining core functionality
