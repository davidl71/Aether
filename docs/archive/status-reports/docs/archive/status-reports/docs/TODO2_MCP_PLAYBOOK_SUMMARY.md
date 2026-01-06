# Todo2 MCP Ansible Playbook - Summary

**Date**: 2025-12-24
**Status**: ✅ Created with Duplicate Removal

## What Was Created

### 1. Ansible Playbook

**File**: `ansible/playbooks/setup_todo2_mcp.yml`

**Features:**

- ✅ Detects `.todo2/` directory
- ✅ Creates `.cursor/` directory if missing
- ✅ **Removes duplicate MCP servers** (same command/args)
- ✅ Adds Todo2 MCP server configuration
- ✅ Merges with existing MCP servers (doesn't overwrite)
- ✅ Idempotent (safe to run multiple times)

### 2. Deduplication Script

**File**: `scripts/deduplicate_mcp_servers.py`

**Purpose:**

- Standalone script for removing duplicate MCP servers
- Can be used independently or by the playbook
- Detects duplicates based on (command, args) tuple

### 2. Documentation

- `docs/TODO2_MCP_SETUP_ANALYSIS.md` - Analysis of current status
- `docs/TODO2_MCP_ANSIBLE_PLAYBOOK.md` - Usage guide
- `docs/TODO2_MCP_PLAYBOOK_SUMMARY.md` - This file

## Duplicate Removal Logic

**How it works:**

1. Reads existing MCP configuration
2. Groups servers by `(command, args)` tuple
3. Keeps first occurrence of each unique config
4. Removes subsequent duplicates
5. Reports which duplicates were removed

**Example:**

```json
// Before
{
  "mcpServers": {
    "filesystem": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem"]},
    "fs": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem"]},
    "git": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-git"]}
  }
}

// After
{
  "mcpServers": {
    "filesystem": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem"]},
    "git": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-git"]}
  }
}
// Removed: "fs" (duplicate of "filesystem")
```

## Usage

### Run for Current Project

```bash
ansible-playbook -i localhost, --connection=local ansible/playbooks/setup_todo2_mcp.yml
```

### Run for Other Projects

```bash
cd ~/Projects/project-management-automation
ansible-playbook -i localhost, --connection=local \
  /Users/davidl/Projects/Trading/ib_box_spread_full_universal/ansible/playbooks/setup_todo2_mcp.yml
```

## Projects Status

| Project                           | Todo2 Dir | MCP Config    | Action       |
| --------------------------------- | --------- | ------------- | ------------ |
| **ib_box_spread_full_universal**  | ✅         | ⚠️ Needs setup | Run playbook |
| **project-management-automation** | ✅         | ❌ Missing     | Run playbook |
| **devwisdom-go**                  | ✅         | ❌ Missing     | Run playbook |

## Next Steps

1. ✅ **Run playbook** on current project
2. ✅ **Run playbook** on other projects
3. ✅ **Verify** Todo2 MCP works
4. ✅ **Restart Cursor** to load MCP servers

---

**Last Updated**: 2025-12-24
**Status**: Ready to Use
