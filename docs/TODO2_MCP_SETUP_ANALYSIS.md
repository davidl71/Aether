# Todo2 MCP Setup Analysis

**Date**: 2025-12-24
**Purpose**: Analyze Todo2 MCP server installation across projects and identify missing configurations

## Current Status

### Projects with Todo2

| Project                                    | Todo2 Directory | MCP Config          | Status    |
| ------------------------------------------ | --------------- | ------------------- | --------- |
| **ib_box_spread_full_universal** (current) | ✅ `.todo2/`     | ✅ Configured        | ✅ Active  |
| **project-management-automation**          | ✅ `.todo2/`     | ❌ Not configured    | ⚠️ Missing |
| **devwisdom-go**                           | ✅ `.todo2/`     | ❌ No `.cursor/` dir | ⚠️ Missing |

## Current Project (ib_box_spread_full_universal)

**Status**: ✅ **Todo2 MCP Configured**

**Configuration:**
- Task T-206 completed: "Configure Todo2 MCP server in Cursor"
- MCP server configured in `.cursor/mcp.json`
- Server name: `Todo2`
- Command: `npx`
- Package: `todo2-extension-todo2`

**Verification:**
```bash
# Check if Todo2 MCP is configured
cat .cursor/mcp.json | python3 -m json.tool | grep -A 5 -i todo2
```

## Other Projects Status

### 1. project-management-automation

**Location**: `/Users/davidl/Projects/project-management-automation`

**Status:**
- ✅ Has `.todo2/` directory
- ✅ Has `.cursor/mcp.json` file
- ❌ **Todo2 MCP server NOT configured**

**Current MCP Servers:**
- `filesystem` (via mcpower-proxy)
- `interactive` (interactive-mcp)

**Action Needed:**
- Add Todo2 MCP server to `.cursor/mcp.json`

### 2. devwisdom-go

**Location**: `/Users/davidl/Projects/devwisdom-go`

**Status:**
- ✅ Has `.todo2/` directory
- ❌ No `.cursor/` directory found
- ❌ **Todo2 MCP server NOT configured**

**Action Needed:**
- Create `.cursor/` directory
- Create `.cursor/mcp.json` with Todo2 MCP server

## Ansible Playbook Status

### Current Ansible Setup

**Existing Playbooks:**
1. `ansible/playbooks/setup_devtools.yml` - Global developer tools
2. `ansible/playbooks/setup_distcc_macos.yml` - Distributed compilation
3. `ansible/playbooks/backend_setup.yml` - Backend environment
4. `ansible/playbooks/fetch_third_party.yml` - Third-party dependencies

**MCP Configuration Handling:**
- `setup_distcc_macos.yml` copies `.cursor/mcp.json` to worker machines
- **No dedicated Todo2 MCP setup playbook exists**

### Missing: Todo2 MCP Setup Playbook

**What's Needed:**
- Ansible playbook to configure Todo2 MCP server
- Should work across multiple projects
- Should check for existing `.todo2/` directory
- Should merge with existing MCP configuration (not overwrite)

## Recommended Solution

### Option 1: Create Dedicated Todo2 MCP Playbook

**File**: `ansible/playbooks/setup_todo2_mcp.yml`

**Features:**
- Check for `.todo2/` directory in project
- Create `.cursor/` directory if missing
- Merge Todo2 MCP config into existing `mcp.json`
- Support multiple projects

### Option 2: Add to Existing Playbook

**File**: `ansible/playbooks/setup_devtools.yml` or `ansible/roles/devtools/tasks/main.yml`

**Features:**
- Add Todo2 MCP configuration task
- Run as part of standard devtools setup

## Todo2 MCP Configuration Template

**Standard Configuration:**
```json
{
  "mcpServers": {
    "Todo2": {
      "command": "npx",
      "args": [
        "-y",
        "todo2-extension-todo2"
      ]
    }
  }
}
```

**Note:** This should be **merged** with existing MCP servers, not replace them.

## Implementation Plan

### Step 1: Create Ansible Playbook

**File**: `ansible/playbooks/setup_todo2_mcp.yml`

**Tasks:**
1. Check if project has `.todo2/` directory
2. Ensure `.cursor/` directory exists
3. Read existing `mcp.json` (if exists)
4. Remove duplicate MCP servers (same command/args)
5. Merge Todo2 MCP server configuration
6. Write updated `mcp.json`

### Step 2: Update Other Projects

**For each project with `.todo2/` directory:**
1. Run playbook to configure Todo2 MCP
2. Verify MCP server is active
3. Test Todo2 MCP tools

### Step 3: Document Usage

**Create documentation:**
- How to run the playbook
- How to verify Todo2 MCP is working
- Troubleshooting guide

## Quick Fix for Other Projects

### Manual Configuration

**For project-management-automation:**
```bash
cd ~/Projects/project-management-automation
# Edit .cursor/mcp.json and add Todo2 server
```

**For devwisdom-go:**
```bash
cd ~/Projects/devwisdom-go
mkdir -p .cursor
# Create .cursor/mcp.json with Todo2 server
```

## Next Steps

1. ✅ **Create Todo2 MCP setup playbook** (recommended)
2. ✅ **Run playbook on other projects**
3. ✅ **Verify Todo2 MCP works in all projects**
4. ✅ **Document the setup process**

---

**Last Updated**: 2025-12-24
**Status**: Analysis Complete - Playbook Needed
