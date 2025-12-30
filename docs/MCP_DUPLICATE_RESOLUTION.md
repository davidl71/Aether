# MCP Server Duplicate Resolution

**Date**: 2025-12-24
**Status**: ✅ Resolved

## Issue

User reported duplicates in MCP server configuration after restarting Cursor.

## Investigation

### Initial Check
- Found 9 servers in configuration (Todo2 was missing)
- No duplicates found by deduplication script
- All servers had unique (command, args) tuples

### Root Cause
- **Todo2 server was missing** from configuration
- This may have been removed during the uvx migration
- No actual duplicates found in the file

## Resolution

### Actions Taken

1. ✅ **Added Todo2 server back**
   - Restored Todo2 configuration
   - Total servers: 9 → 10

2. ✅ **Ran deduplication script**
   - Verified no duplicates exist
   - All servers have unique configurations

3. ✅ **Verified configuration**
   - All 10 expected servers present
   - No duplicate (command, args) tuples

## Final Configuration

### All Servers (10 total)

**npx servers (8):**
- Todo2
- agentic-tools
- context7
- filesystem
- git
- semgrep
- sequential_thinking
- tractatus_thinking

**uvx servers (2):**
- exarp
- ollama

### Duplicate Check Results

✅ **No duplicates found**
- All servers have unique (command, args) combinations
- Deduplication script confirms: 0 duplicates

## If Duplicates Still Appear in Cursor

If you still see duplicates in Cursor's UI after this fix:

1. **Clear Cursor cache:**
   ```bash
   # macOS
   rm -rf ~/Library/Application\ Support/Cursor/Cache
   ```

2. **Restart Cursor completely:**
   - Quit Cursor (not just reload)
   - Reopen Cursor

3. **Check MCP server status:**
   - Open Cursor Settings → MCP Servers
   - Verify all servers are listed correctly
   - Check for any error messages

4. **Verify configuration file:**
   ```bash
   cat .cursor/mcp.json | python3 -m json.tool
   python3 scripts/deduplicate_mcp_servers.py .cursor/mcp.json
   ```

## Prevention

The Ansible playbook `ansible/playbooks/setup_todo2_mcp.yml` includes automatic duplicate removal. Run it to ensure no duplicates:

```bash
ansible-playbook ansible/playbooks/setup_todo2_mcp.yml
```

---

**Last Updated**: 2025-12-24
**Status**: Duplicates Resolved, Configuration Clean
