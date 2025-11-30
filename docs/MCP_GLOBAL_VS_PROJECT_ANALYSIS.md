# MCP Server Global vs Project Configuration Analysis

**Date:** 2025-01-20
**Analysis Type:** Global (`~/.cursor/mcp.json`) vs Project (`.cursor/mcp.json`) placement
**Goal:** Optimize server placement for best performance and minimal duplication

---

## Current Configuration Status

### ✅ Global MCP Configuration (`~/.cursor/mcp.json`)

**Currently Installed (2 servers):**

| Server | Status | Tools | Purpose |
|--------|--------|-------|---------|
| **tractatus_thinking** | ✅ Active | Multiple | Logical concept analysis and structured thinking |
| **desktop-commander** | ✅ Active | Multiple | System-level operations, file access, terminal commands |

**Total:** 2 servers in global config

### ✅ Project MCP Configuration (`.cursor/mcp.json`)

**Currently Installed (6 servers):**

| Server | Status | Tools | Purpose |
|--------|--------|-------|---------|
| **filesystem** | ✅ Active | Core | File system operations (workspace-scoped) |
| **git** | ✅ Active | Core | Git version control (repository-specific) |
| **agentic-tools** | ✅ Active | Multiple | Task management with project storage |
| **context7** | ✅ Active | Multiple | Documentation lookup |
| **notebooklm** | ✅ Active | Multiple | Research & documentation |
| **sequential_thinking** | ✅ Active | Multiple | Structured problem-solving |

**Total:** 6 servers in project config

**Combined Total:** 8 servers (2 global + 6 project)

---

## Placement Criteria Analysis

### 🔴 Should Be GLOBAL (Available in All Projects)

**Criteria:**

- ✅ Not project-specific (doesn't need workspace path)
- ✅ Useful across all projects (not tied to single project)
- ✅ System-level tools (OS operations, universal utilities)
- ✅ Thinking/problem-solving tools (universal)
- ✅ No project-specific storage or configuration

**Current Global Servers:**

1. ✅ **tractatus_thinking** - **CORRECT** - Universal thinking tool
2. ✅ **desktop-commander** - **CORRECT** - System-level operations

**Missing from Global (Should Add):**

- ⚠️ None identified - current global setup is appropriate

### 🟡 Should Be PROJECT (Project-Specific)

**Criteria:**

- ✅ Needs workspace path (filesystem with `${workspaceFolder}`)
- ✅ Needs repository path (git with `--repository`)
- ✅ Project-specific storage (agentic-tools stores in `.agentic-tools-mcp/`)
- ✅ Project-specific notebooks (notebooklm notebooks per project)
- ✅ Project-specific documentation (context7 queries per project)

**Current Project Servers:**

1. ✅ **filesystem** - **CORRECT** - Needs workspace path
2. ✅ **git** - **CORRECT** - Needs repository path
3. ✅ **agentic-tools** - **CORRECT** - Stores project-specific tasks
4. ✅ **context7** - **CORRECT** - Could be global, but project-specific is fine
5. ✅ **notebooklm** - **CORRECT** - Project-specific notebooks
6. ✅ **sequential_thinking** - **CONSIDER MOVING** - Universal tool, no project dependency

### 🟢 Should Be ADDED to Project (Missing)

**Missing Servers:**

1. ⚠️ **semgrep** - **ADD TO PROJECT** - Security scanning (required by `.cursorrules`)
2. ✅ **tractatus_thinking** - **ALREADY IN GLOBAL** - No need to duplicate in project

**Note:** `tractatus_thinking` is already in global config, so we don't need to add it to project config (that would create a duplicate).

---

## Detailed Server Analysis

### 1. **tractatus_thinking** - ✅ CORRECT in Global

**Current Location:** Global (`~/.cursor/mcp.json`)

**Analysis:**

- ✅ Universal thinking tool - no project dependencies
- ✅ Works across all projects - not project-specific
- ✅ No workspace path needed
- ✅ Complements sequential_thinking (which is in project)

**Recommendation:** ✅ **KEEP IN GLOBAL** - Correct placement

**Rationale:** Tractatus Thinking is a universal problem-solving tool that works the same way across all projects. It doesn't need project-specific configuration.

### 2. **desktop-commander** - ✅ CORRECT in Global

**Current Location:** Global (`~/.cursor/mcp.json`)

**Analysis:**

- ✅ System-level operations (file access, terminal commands)
- ✅ OS-level tools (not project-specific)
- ✅ Useful across all projects
- ✅ No workspace path needed

**Recommendation:** ✅ **KEEP IN GLOBAL** - Correct placement

**Rationale:** Desktop Commander provides system-level operations that are useful across all projects, not tied to a specific workspace.

### 3. **filesystem** - ✅ CORRECT in Project

**Current Location:** Project (`.cursor/mcp.json`)

**Analysis:**

- ✅ Needs workspace path (`${workspaceFolder}`)
- ✅ Scoped to project directory
- ✅ Project-specific file operations

**Recommendation:** ✅ **KEEP IN PROJECT** - Correct placement

**Rationale:** Filesystem server is scoped to the workspace folder, making it project-specific by design.

### 4. **git** - ✅ CORRECT in Project

**Current Location:** Project (`.cursor/mcp.json`)

**Analysis:**

- ✅ Needs repository path (`--repository`)
- ✅ Scoped to specific git repository
- ✅ Project-specific git operations

**Recommendation:** ✅ **KEEP IN PROJECT** - Correct placement

**Rationale:** Git server is tied to a specific repository, making it inherently project-specific.

### 5. **agentic-tools** - ✅ CORRECT in Project

**Current Location:** Project (`.cursor/mcp.json`)

**Analysis:**

- ✅ Stores data in project (`.agentic-tools-mcp/`)
- ✅ Project-specific tasks and memories
- ✅ Git-trackable project data

**Recommendation:** ✅ **KEEP IN PROJECT** - Correct placement

**Rationale:** Agentic Tools stores project-specific task lists and memories in the project directory, making it project-specific.

### 6. **context7** - ✅ CORRECT in Project (Could be Global)

**Current Location:** Project (`.cursor/mcp.json`)

**Analysis:**

- ✅ No project dependency (just documentation lookup)
- ⚠️ Could work globally (same docs across projects)
- ✅ Project-specific placement is fine (allows project-specific doc preferences)

**Recommendation:** ✅ **KEEP IN PROJECT** - Current placement is acceptable

**Alternative:** Could move to global if you want same documentation lookup across all projects.

**Rationale:** Context7 is universal documentation lookup, but keeping it in project allows project-specific documentation preferences if needed.

### 7. **notebooklm** - ✅ CORRECT in Project

**Current Location:** Project (`.cursor/mcp.json`)

**Analysis:**

- ✅ Project-specific notebooks (TWS API notebook for this project)
- ✅ Different notebooks per project make sense
- ✅ Project-specific research context

**Recommendation:** ✅ **KEEP IN PROJECT** - Correct placement

**Rationale:** NotebookLM notebooks are project-specific (you have a TWS API notebook for this trading project), making project-level placement appropriate.

### 8. **sequential_thinking** - ⚠️ CONSIDER MOVING to Global

**Current Location:** Project (`.cursor/mcp.json`)

**Analysis:**

- ✅ Universal thinking tool (same as tractatus_thinking)
- ✅ No project dependency
- ✅ Works across all projects
- ⚠️ Could be global like tractatus_thinking

**Recommendation:** ⚠️ **CONSIDER MOVING TO GLOBAL** - Universal tool

**Rationale:** Sequential Thinking is a universal problem-solving tool that complements Tractatus Thinking. It doesn't need project-specific configuration and would be useful across all projects.

**Decision:**

- **Keep in Project** - If you want project-specific thinking workflows
- **Move to Global** - If you want same thinking tools across all projects (recommended)

### 9. **semgrep** - ⚠️ MISSING - Should Add to Project

**Current Location:** Not configured

**Analysis:**

- ✅ Required by `.cursorrules` (security scanning mandate)
- ✅ Project-specific security scanning makes sense
- ✅ Can have project-specific security rules

**Recommendation:** ✅ **ADD TO PROJECT** - Required by rules

**Rationale:** Semgrep is required by project rules for security scanning. Project-specific placement allows project-specific security rules.

---

## Duplication Check

### ❌ Current Duplicates

**None Found** ✅

**Previous Duplicates (Fixed):**

- `context7` - Was in both global and project (fixed per `MCP_DUPLICATE_FIX.md`)
- `GitKraken` - Was in both global and project (fixed per `MCP_DUPLICATE_FIX.md`)

**Potential Duplicate Risk:**

- ⚠️ `tractatus_thinking` - In global, but project review says it's "missing" (it's not - it's in global!)

### ✅ Correct Separation

**Global (2 servers):**

- tractatus_thinking ✅
- desktop-commander ✅

**Project (6 servers):**

- filesystem ✅
- git ✅
- agentic-tools ✅
- context7 ✅
- notebooklm ✅
- sequential_thinking ✅

**No Overlap** ✅

---

## Optimization Recommendations

### ✅ Immediate Actions

#### 1. Add Missing Server to Project

**Add `semgrep` to project config:**

```json
{
  "semgrep": {
    "command": "npx",
    "args": ["-y", "@semgrep/mcp-server-semgrep"],
    "description": "Security scanning for code analysis - checks for security vulnerabilities, bugs, and code quality issues"
  }
}
```

#### 2. Optional: Move sequential_thinking to Global

**Decision:** Should `sequential_thinking` be global like `tractatus_thinking`?

**Recommendation:** ✅ **YES** - Move to global for consistency

**Rationale:**

- Both are universal thinking tools
- Complement each other (Tractatus → Sequential workflow)
- No project-specific configuration needed
- Useful across all projects

**Action:** Move `sequential_thinking` from project to global config

---

## Final Configuration Recommendations

### ✅ Global Configuration (`~/.cursor/mcp.json`)

**Keep (2 servers):**

1. ✅ tractatus_thinking - Universal logical analysis
2. ✅ desktop-commander - System-level operations

**Add (1 server):**
3. ✅ sequential_thinking - Universal structured problem-solving (move from project)

**Total: 3 global servers**

### ✅ Project Configuration (`.cursor/mcp.json`)

**Keep (6 servers):**

1. ✅ filesystem - Workspace-scoped file operations
2. ✅ git - Repository-specific version control
3. ✅ agentic-tools - Project-specific task management
4. ✅ context7 - Documentation lookup (project-specific preference)
5. ✅ notebooklm - Project-specific notebooks (TWS API notebook)
6. ⚠️ sequential_thinking - **REMOVE** (move to global)

**Add (1 server):**
7. ✅ semgrep - Security scanning (required by rules)

**Total: 6 project servers (after moving sequential_thinking to global)**

---

## Summary Table

| Server | Current | Recommended | Rationale |
|--------|---------|-------------|-----------|
| **tractatus_thinking** | Global ✅ | Global ✅ | Universal thinking tool |
| **desktop-commander** | Global ✅ | Global ✅ | System-level operations |
| **sequential_thinking** | Project ⚠️ | **Move to Global** | Universal thinking tool (like tractatus) |
| **filesystem** | Project ✅ | Project ✅ | Needs workspace path |
| **git** | Project ✅ | Project ✅ | Needs repository path |
| **agentic-tools** | Project ✅ | Project ✅ | Project-specific storage |
| **context7** | Project ✅ | Project ✅ | Project-specific preferences OK |
| **notebooklm** | Project ✅ | Project ✅ | Project-specific notebooks |
| **semgrep** | Missing ❌ | **Add to Project** | Required by rules |

---

## Implementation Steps

### Step 1: Add Missing Server (Semgrep)

1. Open `.cursor/mcp.json` (project config)
2. Add `semgrep` configuration
3. Save and restart Cursor

### Step 2: Move Sequential Thinking to Global (Optional)

1. Remove `sequential_thinking` from `.cursor/mcp.json` (project)
2. Add `sequential_thinking` to `~/.cursor/mcp.json` (global)
3. Save both files
4. Restart Cursor

### Step 3: Verify Configuration

1. Check Cursor Settings → MCP Servers
2. Verify no duplicates
3. Verify `semgrep` appears (if added)
4. Verify `sequential_thinking` is in global (if moved)

---

## Best Practices Summary

### ✅ Global Placement Guidelines

**Use Global For:**

- Universal thinking/problem-solving tools
- System-level operations (OS, terminal, file system access outside workspace)
- Tools that work the same across all projects
- Tools without project-specific configuration

**Examples:**

- tractatus_thinking ✅
- sequential_thinking ✅ (should move)
- desktop-commander ✅

### ✅ Project Placement Guidelines

**Use Project For:**

- Tools that need workspace path (`${workspaceFolder}`)
- Tools that need repository path (`--repository`)
- Tools with project-specific storage
- Tools with project-specific notebooks/resources
- Project-specific security/configuration rules

**Examples:**

- filesystem ✅ (needs workspace path)
- git ✅ (needs repository path)
- agentic-tools ✅ (stores in project directory)
- notebooklm ✅ (project-specific notebooks)
- semgrep ✅ (project-specific security rules)

---

## Impact Assessment

### After Adding Semgrep (Project)

**What you gain:**

- ✅ Security scanning (required by `.cursorrules`)
- ✅ Code quality analysis
- ✅ Compliance with project rules

**Total Servers:** 7 (2 global + 5 project) → 7 (2 global + 6 project)

### After Moving Sequential Thinking (Project → Global)

**What changes:**

- ✅ Consistent placement with tractatus_thinking
- ✅ Available across all projects
- ✅ No project-specific dependency

**What stays the same:**

- ✅ Same functionality
- ✅ Same workflow (Tractatus → Sequential)

**Total Servers:** 7 (2 global + 6 project) → 7 (3 global + 5 project)

**Note:** Server count stays the same (7), just reorganized for better separation.

---

## Decision Points

### 1. Sequential Thinking Placement

**Question:** Should `sequential_thinking` be moved to global?

**Recommendation:** ✅ **YES** - Move to global for consistency with `tractatus_thinking`

**Rationale:**

- Both are universal thinking tools
- Both work across all projects
- No project-specific configuration
- Better organized together in global config

**Action:** Move from project to global config

### 2. Context7 Placement

**Question:** Should `context7` stay in project or move to global?

**Current:** Project

**Recommendation:** ✅ **KEEP IN PROJECT** - Current placement is fine

**Rationale:**

- Can have project-specific documentation preferences
- Not critical either way
- Current placement works well

**Action:** No change needed

### 3. Semgrep Addition

**Question:** Should `semgrep` be added to project config?

**Recommendation:** ✅ **YES** - Add immediately (required by rules)

**Action:** Add to project config

---

## Final Recommendations

### ✅ Immediate Actions

1. **Add `semgrep` to project config** - Required by `.cursorrules`
2. **Move `sequential_thinking` to global config** - For consistency with `tractatus_thinking`

### ✅ Current Configuration Status

**Global (3 servers after changes):**

- tractatus_thinking ✅
- desktop-commander ✅
- sequential_thinking ✅ (move from project)

**Project (6 servers after changes):**

- filesystem ✅
- git ✅
- agentic-tools ✅
- context7 ✅
- notebooklm ✅
- semgrep ✅ (add)

**Total:** 9 servers (3 global + 6 project)

---

## See Also

- [MCP_SERVER_REVIEW.md](MCP_SERVER_REVIEW.md) - Comprehensive server review
- [MCP_DUPLICATE_FIX.md](MCP_DUPLICATE_FIX.md) - Previous duplicate resolution
- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - Detailed server documentation
- [.cursorrules](../.cursorrules) - Project rules mentioning MCP servers
