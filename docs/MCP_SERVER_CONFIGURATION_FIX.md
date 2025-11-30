# MCP Server Configuration Fix

**Date:** 2025-11-20
**Update Type:** Removed duplicate MCP servers from project config
**Status:** ✅ **COMPLETED**

---

## Issues Found

### 1. Duplicate Tractatus Thinking Server

- **Problem**: `tractatus_thinking` was configured in both:
  - Global config (`~/.cursor/mcp.json`) ✅ Correct
  - Project config (`.cursor/mcp.json`) ❌ Duplicate

- **Impact**: Unnecessary duplication, potential conflicts
- **Solution**: Removed from project config (already correctly in global)

### 2. Incorrect Sequential Thinking Configuration

- **Problem**: `sequential_thinking` was configured incorrectly in project config:
  - Project config: `python3 -m sequential_thinking` ❌ Module doesn't exist
  - Global config: `sequential-thinking-mcp` (via pipx) ✅ Correct

- **Impact**: Project config would fail to start the server
- **Solution**: Removed from project config (already correctly in global)

---

## Changes Made

### ✅ Project Configuration (`.cursor/mcp.json`)

**Removed:**

- ❌ `tractatus_thinking` - Duplicate (already in global config)
- ❌ `sequential_thinking` - Incorrect config (already in global config with correct setup)

**Current Project Servers (5):**

1. ✅ `filesystem` - Workspace-scoped file operations
2. ✅ `git` - Repository-specific version control
3. ✅ `agentic-tools` - Project-specific task management
4. ✅ `context7` - Documentation lookup
5. ✅ `semgrep` - Security scanning

### ✅ Global Configuration (`~/.cursor/mcp.json`)

**Current Global Servers (3):**

1. ✅ `tractatus_thinking` - Universal logical analysis (via `uvx mcpower-proxy`)
2. ✅ `sequential_thinking` - Universal structured problem-solving (via `pipx sequential-thinking-mcp`)
3. ✅ `openmemory` - Agent memory storage

---

## Verification

### ✅ Configuration Validation

- Project config JSON is valid
- Global config JSON is valid
- No duplicate servers between global and project
- All required servers present

### ✅ Server Installation Status

- `sequential-thinking-mcp` installed via pipx: ✅ (version 0.10.1)
- `tractatus_thinking` available via npx: ✅
- All project servers available via npx: ✅

---

## Final Configuration Summary

### Global Servers (3) - Available in All Projects

| Server | Purpose | Configuration |
|--------|---------|---------------|
| `tractatus_thinking` | Logical concept analysis | `uvx mcpower-proxy` wrapper |
| `sequential_thinking` | Structured problem-solving | `pipx sequential-thinking-mcp` |
| `openmemory` | Agent memory storage | API-based |

### Project Servers (5) - Project-Specific

| Server | Purpose | Why Project |
|--------|---------|-------------|
| `filesystem` | File operations | Needs workspace path |
| `git` | Version control | Needs repository path |
| `agentic-tools` | Task management | Stores in project (`.agentic-tools-mcp/`) |
| `context7` | Documentation lookup | Project-specific documentation preferences |
| `semgrep` | Security scanning | Required by `.cursorrules`, project-specific rules |

**Total: 8 servers (3 global + 5 project)**

---

## Next Steps

### 1. Restart Cursor

**⚠️ IMPORTANT:** Restart Cursor completely after configuration changes:

1. Close all Cursor windows
2. Quit Cursor completely (Cmd+Q on macOS)
3. Reopen Cursor
4. Verify MCP servers are loaded correctly

### 2. Verify Configuration

After restarting, verify:

1. **Check MCP Servers in Cursor:**
   - Open Cursor Settings
   - Go to MCP Servers
   - Verify all 8 servers are listed
   - Verify no duplicates
   - Verify `tractatus_thinking` and `sequential_thinking` are from global config

2. **Test Servers:**
   - Test Tractatus: "Analyze this problem with Tractatus Thinking"
   - Test Sequential: "Create workflow for this implementation"
   - Test Semgrep: "Scan this file with Semgrep"

---

## Related Documentation

- [MCP_CONFIGURATION_UPDATE_SUMMARY.md](MCP_CONFIGURATION_UPDATE_SUMMARY.md) - Previous configuration updates
- [MCP_TRACTATUS_TROUBLESHOOTING.md](MCP_TRACTATUS_TROUBLESHOOTING.md) - Tractatus Thinking troubleshooting
- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - Detailed server documentation
- [.cursorrules](../.cursorrules) - Project rules mentioning MCP servers

---

## Task Status

- ✅ **T-191**: Add Tractatus Thinking MCP server to configuration - **COMPLETE** (was already in global, removed duplicate from project)
- ✅ **T-197**: Install and configure Sequential MCP server - **COMPLETE** (was already in global with correct pipx setup, removed incorrect project config)
- ✅ **T-196**: Research Sequential MCP server installation and usage - **COMPLETE** (already done)
