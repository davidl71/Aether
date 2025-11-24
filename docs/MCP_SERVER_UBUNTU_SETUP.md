# MCP Server Setup for Ubuntu Agent

**Issue:** ENOENT error when Cursor tries to start MCP server on Ubuntu agent

**Root Cause:** Virtual environment not set up or MCP package not installed

---

## Quick Fix

Run on Ubuntu agent:

```bash
cd ~/ib_box_spread_full_universal/mcp-servers/project-management-automation
bash setup.sh
```

Or manually:

```bash
cd ~/ib_box_spread_full_universal/mcp-servers/project-management-automation

# Create venv if missing
python3 -m venv venv

# Activate and install
source venv/bin/activate
pip install --upgrade pip
pip install mcp

# Verify
python3 -c "import mcp; print('✅ MCP installed')"
```

---

## Verification

After setup, verify the server can start:

```bash
cd ~/ib_box_spread_full_universal
bash mcp-servers/project-management-automation/run_server.sh --help
```

---

## Troubleshooting

### Error: ENOENT (File not found)

**Cause:** `run_server.sh` not found or not executable

**Fix:**
```bash
cd ~/ib_box_spread_full_universal
git pull  # Ensure latest changes
chmod +x mcp-servers/project-management-automation/run_server.sh
```

### Error: Virtual environment not found

**Cause:** `venv/` directory missing

**Fix:**
```bash
cd ~/ib_box_spread_full_universal/mcp-servers/project-management-automation
python3 -m venv venv
source venv/bin/activate
pip install mcp
```

### Error: MCP not installed

**Cause:** MCP package not in venv

**Fix:**
```bash
cd ~/ib_box_spread_full_universal/mcp-servers/project-management-automation
source venv/bin/activate
pip install mcp
```

### Error: Cannot import from server

**Cause:** Server structure changed or dependencies missing

**Fix:**
```bash
cd ~/ib_box_spread_full_universal
git pull  # Get latest server.py
cd mcp-servers/project-management-automation
source venv/bin/activate
pip install -r requirements.txt  # If exists
pip install mcp
```

---

## After Fix

1. **Restart Cursor** on Ubuntu agent to reload MCP configuration
2. **Verify MCP server** appears in Cursor's MCP panel
3. **Test a tool** like "List tasks awaiting clarification"

---

## Prevention

Add to Ansible setup or initial agent setup:

```bash
# In ansible/roles/devtools/tasks/main.yml or setup script
- name: Setup MCP server virtual environment
  command: bash setup.sh
  args:
    chdir: "{{ project_root }}/mcp-servers/project-management-automation"
  when: ansible_facts['os_family'] == 'Debian'
```

---

**See Also:**
- `mcp-servers/project-management-automation/setup.sh` - Setup script
- `mcp-servers/project-management-automation/run_server.sh` - Server wrapper
- `.cursor/mcp.json` - MCP configuration

