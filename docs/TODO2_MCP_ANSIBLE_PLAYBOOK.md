# Todo2 MCP Ansible Playbook

**Date**: 2025-12-24
**Purpose**: Automated setup of Todo2 MCP server across projects

## Overview

The `setup_todo2_mcp.yml` playbook automatically configures Todo2 MCP server for any project that has a `.todo2/` directory.

## Features

- ✅ **Automatic Detection**: Checks for `.todo2/` directory
- ✅ **Safe Merging**: Merges with existing MCP configuration (doesn't overwrite)
- ✅ **Duplicate Removal**: Automatically removes duplicate MCP servers (same command/args)
- ✅ **Idempotent**: Safe to run multiple times
- ✅ **Multi-Project**: Works across all your projects

## Usage

### Run for Current Project

```bash
cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
ansible-playbook -i localhost, --connection=local ansible/playbooks/setup_todo2_mcp.yml
```

### Run for Specific Project

```bash
cd ~/Projects/project-management-automation
ansible-playbook -i localhost, --connection=local \
  /Users/davidl/Projects/Trading/ib_box_spread_full_universal/ansible/playbooks/setup_todo2_mcp.yml
```

### Run for All Projects

```bash
# For each project with .todo2 directory
for project in ~/Projects/*/; do
  if [ -d "$project/.todo2" ]; then
    echo "Setting up Todo2 MCP for $(basename $project)..."
    cd "$project"
    ansible-playbook -i localhost, --connection=local \
      /Users/davidl/Projects/Trading/ib_box_spread_full_universal/ansible/playbooks/setup_todo2_mcp.yml
  fi
done
```

## What It Does

1. **Checks for Todo2**: Verifies `.todo2/` directory exists
2. **Creates .cursor Directory**: If missing
3. **Reads Existing Config**: Preserves current MCP servers
4. **Removes Duplicates**: Detects and removes duplicate MCP servers (same command/args)
5. **Adds Todo2 Server**: Merges Todo2 MCP configuration
6. **Writes Updated Config**: Saves cleaned and merged configuration

## Configuration Added

```json
{
  "mcpServers": {
    "Todo2": {
      "command": "npx",
      "args": ["-y", "todo2-extension-todo2"]
    }
  }
}
```

**Note:** This is merged with existing servers, not replacing them.

## Duplicate Removal

The playbook automatically detects and removes duplicate MCP servers based on:
- **Command**: Same command (e.g., `npx`, `python3`)
- **Args**: Same arguments array

**Implementation:**
- Uses `scripts/deduplicate_mcp_servers.py` for duplicate detection
- Keeps first occurrence of each unique (command, args) combination
- Reports which duplicates were removed

**Example:**
If you have:
```json
{
  "mcpServers": {
    "filesystem": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem"]},
    "fs": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem"]}
  }
}
```

The playbook will remove `fs` (duplicate of `filesystem`) and keep `filesystem`.

**Manual Usage:**
You can also run the deduplication script directly:
```bash
cat .cursor/mcp.json | python3 scripts/deduplicate_mcp_servers.py
```

## Verification

After running the playbook:

1. **Check MCP Config:**
   ```bash
   cat .cursor/mcp.json | python3 -m json.tool | grep -A 5 Todo2
   ```

2. **Restart Cursor**: MCP servers load on Cursor startup

3. **Test Todo2 MCP**: Use Todo2 MCP tools in Cursor chat

## Projects Status

| Project                           | Todo2 Dir | MCP Config    | Status       |
| --------------------------------- | --------- | ------------- | ------------ |
| **ib_box_spread_full_universal**  | ✅         | ⚠️ Needs setup | Run playbook |
| **project-management-automation** | ✅         | ❌ Missing     | Run playbook |
| **devwisdom-go**                  | ✅         | ❌ Missing     | Run playbook |

## Troubleshooting

### Playbook Fails

**Error: "Todo2 directory not found"**
- ✅ This is expected if project doesn't use Todo2
- Playbook skips projects without `.todo2/` directory

**Error: "Permission denied"**
- Check file permissions on `.cursor/mcp.json`
- May need to run with appropriate user permissions

### Todo2 MCP Not Working After Setup

1. **Restart Cursor**: MCP servers load on startup
2. **Check npm package**: `npx -y todo2-extension-todo2 --version`
3. **Check Cursor logs**: Look for MCP server errors
4. **Verify config**: Ensure JSON is valid

## Integration with Other Playbooks

### Add to setup_devtools.yml

To include Todo2 MCP setup in standard devtools setup:

```yaml
---
- name: Configure global developer tools
  hosts: localhost
  become: false
  gather_facts: true
  vars:
    ansible_python_interpreter: auto

  roles:
    - devtools

  tasks:
    - name: Setup Todo2 MCP
      include: setup_todo2_mcp.yml
```

## Manual Setup (Alternative)

If you prefer manual setup:

1. **Check for Todo2:**
   ```bash
   test -d .todo2 && echo "Todo2 found" || echo "No Todo2"
   ```

2. **Create/Edit MCP Config:**
   ```bash
   mkdir -p .cursor
   # Edit .cursor/mcp.json and add Todo2 server
   ```

3. **Add Todo2 Server:**
   ```json
   {
     "mcpServers": {
       "Todo2": {
         "command": "npx",
         "args": ["-y", "todo2-extension-todo2"]
       }
     }
   }
   ```

## Next Steps

1. ✅ **Run playbook** on current project
2. ✅ **Run playbook** on other projects with Todo2
3. ✅ **Verify** Todo2 MCP works in all projects
4. ✅ **Restart Cursor** to load MCP servers

---

**Last Updated**: 2025-12-24
**Status**: Playbook Created and Ready
