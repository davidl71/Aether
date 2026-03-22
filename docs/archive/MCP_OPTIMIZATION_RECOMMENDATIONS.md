# MCP Server Optimization Recommendations

**Date:** 2025-01-20
**Issue:** Exceeding total tools limit
**Current Servers:** 8 configured

## Current MCP Servers

1. **semgrep** - Security scanning (required)
2. **notebooklm** - YouTube/documentation summarization
3. **filesystem** - File operations (core)
4. **git** - Version control (core)
5. **agentic-tools** - Task management (Todo2 workflow)
6. **gitkraken** - Git workflow enhancement
7. **iterm2** - Terminal context
8. **context7** - Documentation lookup

## Priority Classification

### 🔴 ESSENTIAL (Must Keep - 3 servers)

These are core to AI functionality and project requirements:

1. **filesystem** - Core AI file operations
   - **Why essential:** AI needs to read/write files
   - **Tools:** File read/write, directory listing, search
   - **Impact if removed:** AI cannot modify files

2. **git** - Version control operations
   - **Why essential:** AI needs git context for code changes
   - **Tools:** Git status, diff, commit, branch operations
   - **Impact if removed:** AI cannot understand git history or help with commits

3. **semgrep** - Security scanning
   - **Why essential:** Required by `.cursorrules` (security scanning mandate)
   - **Tools:** Security vulnerability scanning
   - **Impact if removed:** Violates project security requirements

### 🟡 IMPORTANT (Keep if possible - 1 server)

4. **agentic-tools** - Task management
   - **Why important:** Used by Todo2 workflow (mandatory per rules)
   - **Tools:** Task creation, management, memories
   - **Impact if removed:** Todo2 workflow breaks (violates rules)
   - **Note:** If you're not using Todo2, this can be removed

### 🟢 OPTIONAL (Can Remove - 4 servers)

5. **gitkraken** - Git workflow enhancement
   - **Why optional:** Overlaps with `git` server
   - **Tools:** Enhanced git operations, issue tracking, PR management
   - **Impact if removed:** Lose GitKraken-specific features, but `git` server covers basics
   - **Recommendation:** ⚠️ **REMOVE FIRST** - Redundant with `git` server

6. **notebooklm** - Documentation summarization
   - **Why optional:** Can use web search instead
   - **Tools:** YouTube video summarization, documentation processing
   - **Impact if removed:** Lose NotebookLM-specific research features
   - **Recommendation:** ⚠️ **REMOVE** - Nice to have, not essential

7. **context7** - Documentation lookup
   - **Why optional:** Overlaps with web search and NotebookLM
   - **Tools:** Up-to-date documentation access
   - **Impact if removed:** Lose version-specific doc lookup
   - **Recommendation:** ⚠️ **REMOVE** - Redundant with web search

8. **iterm2** - Terminal context
   - **Why optional:** Nice to have, not essential
   - **Tools:** Terminal session management, command execution
   - **Impact if removed:** Lose terminal context awareness
   - **Recommendation:** ⚠️ **REMOVE** - Can use regular terminal commands

## Recommended Removal Order

### Phase 1: Remove Redundant Servers (Saves 3 slots)

1. **gitkraken** - Overlaps with `git` server
2. **context7** - Overlaps with web search
3. **notebooklm** - Optional research tool

**Result:** 8 → 5 servers

### Phase 2: Remove Optional Servers (Saves 1 slot)

4. **iterm2** - Terminal context (nice but not essential)

**Result:** 5 → 4 servers (Essential + Important)

### Phase 3: If Still Over Limit

5. **agentic-tools** - Only if Todo2 workflow is not being used

**Result:** 4 → 3 servers (Essential only)

## Quick Removal Guide

### Remove gitkraken, notebooklm, context7, iterm2

Edit `.cursor/mcp.json` and remove these sections:

```json
{
  "mcpServers": {
    "semgrep": { ... },
    "filesystem": { ... },
    "git": { ... },
    "agentic-tools": { ... }
    // Remove: gitkraken, notebooklm, context7, iterm2
  }
}
```

## Impact Assessment

### After Removing 4 Optional Servers (8 → 4 servers)

**What you lose:**

- GitKraken-specific git features (but `git` server still works)
- NotebookLM video/documentation summarization
- Context7 version-specific documentation
- iTerm2 terminal context awareness

**What you keep:**

- ✅ Core file operations (filesystem)
- ✅ Git version control (git)
- ✅ Security scanning (semgrep)
- ✅ Task management (agentic-tools)

**Alternative solutions:**

- Use web search instead of NotebookLM/Context7
- Use regular terminal instead of iTerm2 MCP
- Use `git` CLI instead of GitKraken MCP

## Final Recommendation

**Remove these 4 servers:**

1. gitkraken (redundant)
2. notebooklm (optional)
3. context7 (redundant)
4. iterm2 (optional)

**Keep these 4 servers:**

1. filesystem (essential)
2. git (essential)
3. semgrep (required)
4. agentic-tools (important for Todo2)

This reduces from 8 → 4 servers while maintaining core functionality.
