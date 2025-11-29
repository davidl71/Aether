# MCP Server Troubleshooting Guide

## Required MCP Servers

The following MCP servers are **required** for full project functionality:

1. **automa** - Project management automation (self-hosted)
2. **filesystem** - File system operations
3. **agentic-tools** - Advanced task management
4. **context7** - Up-to-date documentation
5. **git** - Git version control
6. **semgrep** - Security scanning
7. **tractatus_thinking** - Logical concept analysis
8. **sequential_thinking** - Implementation workflows

**Total**: 8 required MCP servers

## Common Error Causes

### 1. **Package Name Errors (Tractatus/Sequential Thinking)**

**Problem**: `No module named sequential_thinking` or `tractatus-thinking` package not found.

**Root Cause**: Incorrect package names in configuration.

**Solution**: Use correct npm package names:

```json
{
  "mcpServers": {
    "tractatus_thinking": {
      "command": "npx",
      "args": ["-y", "tractatus_thinking"],
      "description": "Tractatus Thinking MCP server..."
    },
    "sequential_thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"],
      "description": "Sequential Thinking MCP server..."
    }
  }
}
```

**Important Notes**:
- `tractatus_thinking` uses underscore (not hyphen): `tractatus_thinking`
- `sequential_thinking` is an npm package (not Python): `@modelcontextprotocol/server-sequential-thinking`
- Both use `npx` for installation, not `python3 -m`

**Troubleshooting Steps**:
1. Verify package exists: `npm search tractatus_thinking`
2. Test manually: `npx -y tractatus_thinking --version`
3. Test manually: `npx -y @modelcontextprotocol/server-sequential-thinking --version`
4. Update `.cursor/mcp.json` with correct package names
5. Restart Cursor completely

### 2. **Variable Expansion Issues**

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

**⚠️ Platform Note**: iTerm2 is **macOS only**. On Linux/Windows, this server should be disabled in `.cursor/mcp.json`.

**Solution**:
- **macOS**: Install iTerm2: `brew install --cask iterm2`
- **Linux/Windows**: Remove or comment out the `iterm2` server entry in `.cursor/mcp.json` (already done in current config)

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

# iTerm2 (macOS only - skip on Linux/Windows)
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
- **Issue**: iTerm2 not installed (macOS only)
- **Fix**: 
  - **macOS**: `brew install --cask iterm2`
  - **Linux/Windows**: Remove `iterm2` entry from `.cursor/mcp.json` (already done)

### Context7 Server
- **Issue**: Package download fails
- **Fix**: Test manually: `npx -y @upstash/context7-mcp`

### Agentic-tools Server
- **Issue**: Package download fails
- **Fix**: Test manually: `npx -y @pimzino/agentic-tools-mcp`

### Tractatus Thinking Server
- **Issue**: `tractatus-thinking` package not found
- **Fix**: Use correct package name: `tractatus_thinking` (with underscore)
- **Test**: `npx -y tractatus_thinking --version`

### Sequential Thinking Server
- **Issue**: `No module named sequential_thinking` (Python error)
- **Root Cause**: Configured as Python module, but it's an npm package
- **Fix**: Use npm package: `@modelcontextprotocol/server-sequential-thinking`
- **Test**: `npx -y @modelcontextprotocol/server-sequential-thinking --version`

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
