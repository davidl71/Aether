# MCP Bridge Troubleshooting

## Error: "vscode mcp bridge: failed to start socket server"

This error indicates that the MCP bridge (which connects Cursor to MCP servers) cannot start its socket server.

### Common Causes

1. **Port Conflict**: Another process (including Cursor itself) is using the MCP bridge port (commonly port 9527)
2. **Missing Dependencies**: Node.js/npx not available or not in PATH
3. **Permission Issues**: Cannot bind to socket/port
4. **Too Many MCP Servers**: Starting too many servers simultaneously
5. **LSP MCP Conflict**: LSP MCP server trying to use port already in use by Cursor

### Port 9527 Conflict

**Issue**: Cursor is already listening on port 9527, and the MCP bridge is trying to use the same port.

**Check if port 9527 is in use**:
```bash
lsof -i :9527
```

**If Cursor is using port 9527**:
- This is normal - Cursor uses this port for internal communication
- The MCP bridge should use a different port automatically
- If it doesn't, try restarting Cursor

### Permission Denied Error (EACCES)

**Error**: `listen EACCES: permission denied /Users/davidlowes/Library/Application Support/YuTengjing.vscode-mcp/vscode-mcp-*.sock`

**Issue**: The MCP bridge cannot create a Unix socket file due to permission issues.

**Solution**:

1. **Create the directory with correct permissions**:
   ```bash
   mkdir -p "/Users/davidlowes/Library/Application Support/YuTengjing.vscode-mcp"
   chmod 755 "/Users/davidlowes/Library/Application Support/YuTengjing.vscode-mcp"
   ```

2. **Check parent directory permissions**:
   ```bash
   ls -ld "/Users/davidlowes/Library/Application Support"
   # Should show: drwx------ or drwxr-xr-x
   # If not, fix with: chmod 755 "/Users/davidlowes/Library/Application Support"
   ```

3. **Remove any existing socket files with wrong permissions**:
   ```bash
   rm -f "/Users/davidlowes/Library/Application Support/YuTengjing.vscode-mcp/"*.sock
   ```

4. **Restart Cursor** after fixing permissions

**Note**: The `YuTengjing.vscode-mcp` extension creates this directory. If the error persists, try:
- Uninstalling and reinstalling the MCP bridge extension
- Checking if antivirus/security software is blocking socket creation
- Running Cursor with elevated permissions (not recommended, but can test)

### Quick Fixes

#### 1. Check Node.js/npx Availability

```bash
# Verify Node.js is installed
node --version

# Verify npx is available
npx --version

# If missing, install Node.js
brew install node
```

#### 2. Restart Cursor

1. Close Cursor completely
2. Reopen the workspace
3. Check if error persists

#### 3. Simplify MCP Configuration (Temporary)

If the error persists, temporarily disable some MCP servers to isolate the issue:

Edit `.cursor/mcp.json` and comment out servers one by one:

```json
{
  "mcpServers": {
    "semgrep": {
      // ... keep this
    }
    // Temporarily disable others to test
  }
}
```

#### 4. Check for Port Conflicts

```bash
# Check if port 9527 is in use (LSP MCP bridge port)
lsof -i :9527

# Check if port 3000-3010 are in use (common MCP bridge ports)
lsof -i :3000-3010

# If Cursor is using port 9527, this is normal
# The MCP bridge should automatically use a different port
# If it doesn't, restart Cursor
```

#### 5. Check Cursor Logs

1. Open Cursor
2. `Cmd+Shift+P` → "Developer: Toggle Developer Tools"
3. Check Console tab for detailed error messages
4. Look for MCP-related errors

### Step-by-Step Resolution

#### Step 1: Verify Prerequisites

```bash
# Check Node.js
node --version  # Should be v16+ or v18+

# Check npx
npx --version

# Check if workspace folder is accessible
cd /Users/davidlowes/ib_box_spread_full_universal
pwd
```

#### Step 2: Test MCP Server Manually

Test if MCP servers can start manually:

```bash
# Test filesystem server
npx -y @modelcontextprotocol/server-filesystem /Users/davidlowes/ib_box_spread_full_universal

# Test git server
npx -y @modelcontextprotocol/server-git --repository /Users/davidlowes/ib_box_spread_full_universal
```

If these fail, the issue is with the MCP servers themselves, not the bridge.

#### Step 3: Minimal Configuration Test

Create a minimal `.cursor/mcp.json` to test:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "${workspaceFolder}"
      ]
    }
  }
}
```

If this works, add servers one by one to identify the problematic one.

#### Step 4: Check Cursor MCP Extension

1. Open Extensions view (`Cmd+Shift+X`)
2. Search for "MCP" or "Model Context Protocol"
3. Ensure any MCP-related extensions are installed and enabled
4. Try disabling/re-enabling them

### Advanced Troubleshooting

#### Check System Resources

```bash
# Check available ports
netstat -an | grep LISTEN | grep -E '300[0-9]|301[0-9]'

# Check system limits
ulimit -n  # Should be 1024+ for macOS
```

#### Use Absolute Paths

If `${workspaceFolder}` isn't resolving, use absolute paths:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "/Users/davidlowes/ib_box_spread_full_universal"
      ]
    }
  }
}
```

#### Disable Problematic Servers

Some servers might have issues:

1. **Browser MCP**: Requires Chrome/Chromium - might fail if not available
2. **Terminal MCP**: Might have permission issues
3. **NotebookLM**: Requires authentication - might fail if not set up

Temporarily disable these to test:

```json
{
  "mcpServers": {
    "semgrep": { /* ... */ },
    "filesystem": { /* ... */ },
    "git": { /* ... */ }
    // Comment out browser, terminal, notebooklm temporarily
  }
}
```

### Known Issues

#### Issue: Browser MCP Fails

**Solution**: Browser MCP requires Chrome/Chromium. If not installed:

```bash
# Install Chrome (if needed)
brew install --cask google-chrome

# Or disable browser MCP if not needed
```

#### Issue: NotebookLM Authentication

**Solution**: NotebookLM requires Google authentication. If not set up:

1. Disable NotebookLM MCP temporarily
2. Or set up authentication (see `docs/NOTEBOOKLM_USAGE.md`)

#### Issue: Too Many Servers

**Solution**: Reduce the number of active MCP servers. Start with core servers:

- Semgrep (security)
- Filesystem (file operations)
- Git (version control)

Add others (Browser, Terminal, NotebookLM) only when needed.

### Getting Help

If none of these solutions work:

1. **Check Cursor Version**: Ensure you're on the latest Cursor version
2. **Check MCP Documentation**: https://modelcontextprotocol.io/
3. **Cursor Support**: Check Cursor's support channels
4. **Logs**: Share Cursor developer console logs (remove sensitive info)

### Prevention

To avoid this issue:

1. **Start Minimal**: Begin with 1-2 MCP servers, add more gradually
2. **Test Each Server**: Verify each server works before adding the next
3. **Use Absolute Paths**: More reliable than `${workspaceFolder}` variable
4. **Keep Node.js Updated**: Use Node.js 18+ for best compatibility
