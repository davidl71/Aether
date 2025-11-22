# MCP Server Troubleshooting Guide

## Common Error Causes

### 1. **Variable Expansion Issues**

**Problem**: `${workspaceFolder}` may not be expanded correctly in Cursor.

**Solution**: Replace with absolute path or relative path.

**Files affected**:
- `filesystem` server uses `${workspaceFolder}`
- `git` server uses `${workspaceFolder}`

**Fix**: Update `.cursor/mcp.json`:
```json
{
  "filesystem": {
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-filesystem",
      "/Users/davidlowes/ib_box_spread_full_universal"
    ]
  },
  "git": {
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-git",
      "--repository",
      "/Users/davidlowes/ib_box_spread_full_universal"
    ]
  }
}
```

### 2. **Missing Dependencies**

**GitKraken CLI**:
- **Error**: `gk: command not found`
- **Solution**: Install GitKraken CLI:
  ```bash
  brew install gitkraken-cli
  gk auth login
  ```

**uvx/mcpower-proxy**:
- **Error**: `uvx: command not found` or proxy errors
- **Solution**: Install uvx:
  ```bash
  pip install uv
  # or
  brew install uv
  ```

### 3. **Network/Download Issues**

**Problem**: npm packages fail to download on first run.

**Solution**: Pre-download packages:
```bash
# Test each package manually
npx -y @modelcontextprotocol/server-filesystem --help
npx -y @modelcontextprotocol/server-git --help
npx -y @semgrep/mcp-server-semgrep --help
npx -y notebooklm-mcp@latest --help
npx -y @pimzino/agentic-tools-mcp --help
npx -y @rishabkoul/iterm-mcp-server --help
npx -y @upstash/context7-mcp --help
```

### 4. **Permission Issues**

**Problem**: Commands can't execute due to permissions.

**Solution**: Check permissions:
```bash
ls -la $(which npx)
ls -la $(which uvx)
chmod +x $(which npx)  # if needed
```

### 5. **JSON Syntax Errors**

**Problem**: Invalid JSON in `.cursor/mcp.json`.

**Solution**: Validate JSON:
```bash
python3 -m json.tool .cursor/mcp.json > /dev/null && echo "Valid" || echo "Invalid"
```

### 6. **iTerm2 Server Issues**

**Problem**: iTerm2 MCP server may require iTerm2 to be installed.

**Solution**:
- Install iTerm2: `brew install --cask iterm2`
- Or disable the server if not using iTerm2

## Step-by-Step Troubleshooting

### Step 1: Check Prerequisites

```bash
# Check Node.js
node --version  # Should be v18+ or v20+

# Check npm
npm --version  # Should be 9+

# Check uvx
uvx --version  # Should be available

# Check GitKraken CLI
gk --version  # Should be available (or install it)
```

### Step 2: Test Each Server Manually

```bash
# Filesystem server
npx -y @modelcontextprotocol/server-filesystem /Users/davidlowes/ib_box_spread_full_universal

# Git server
npx -y @modelcontextprotocol/server-git --repository /Users/davidlowes/ib_box_spread_full_universal

# Semgrep (via uvx)
uvx mcpower-proxy==0.0.87 --wrapped-config '{"command":"npx","args":["-y","@semgrep/mcp-server-semgrep"]}' --name semgrep

# NotebookLM (via uvx)
uvx mcpower-proxy==0.0.87 --wrapped-config '{"command":"npx","args":["-y","notebooklm-mcp@latest"]}' --name notebooklm

# GitKraken
gk mcp

# iTerm2
npx -y @rishabkoul/iterm-mcp-server

# Context7
npx -y @upstash/context7-mcp

# Agentic-tools
npx -y @pimzino/agentic-tools-mcp
```

### Step 3: Check Cursor Logs

1. Open Cursor
2. Go to: **Help → Toggle Developer Tools**
3. Check **Console** tab for MCP errors
4. Look for specific error messages

### Step 4: Fix Configuration

Based on errors found:

1. **Replace `${workspaceFolder}`** with absolute path
2. **Install missing dependencies** (GitKraken CLI, uvx)
3. **Fix JSON syntax** if invalid
4. **Disable problematic servers** temporarily

### Step 5: Restart Cursor

After fixing configuration:
1. **Quit Cursor completely** (Cmd+Q on macOS)
2. **Restart Cursor**
3. **Check MCP server status** in settings

## Quick Fixes by Server

### Filesystem Server
- **Issue**: `${workspaceFolder}` not expanded
- **Fix**: Use absolute path

### Git Server
- **Issue**: `${workspaceFolder}` not expanded
- **Fix**: Use absolute path

### GitKraken Server
- **Issue**: `gk` command not found
- **Fix**: `brew install gitkraken-cli && gk auth login`

### Semgrep Server
- **Issue**: uvx/mcpower-proxy errors
- **Fix**: Ensure `uvx` is installed and working

### NotebookLM Server
- **Issue**: uvx/mcpower-proxy errors
- **Fix**: Ensure `uvx` is installed and working

### iTerm2 Server
- **Issue**: iTerm2 not installed
- **Fix**: `brew install --cask iterm2` or disable server

### Context7 Server
- **Issue**: Package download fails
- **Fix**: Test manually: `npx -y @upstash/context7-mcp`

### Agentic-tools Server
- **Issue**: Package download fails
- **Fix**: Test manually: `npx -y @pimzino/agentic-tools-mcp`

## Minimal Working Configuration

If many servers are failing, start with a minimal set:

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
    },
    "git": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-git",
        "--repository",
        "/Users/davidlowes/ib_box_spread_full_universal"
      ]
    }
  }
}
```

Then add servers one by one, testing each.

## Getting Help

1. **Check Cursor logs** (Developer Tools → Console)
2. **Test commands manually** in terminal
3. **Check MCP server documentation**:
   - [Model Context Protocol](https://modelcontextprotocol.io/)
   - [GitKraken MCP](https://help.gitkraken.com/mcp/)
4. **Verify Node.js/npm versions** are compatible
