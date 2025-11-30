# MCP Cursor Chat Troubleshooting

## `/mcp list` Command Errors

If you're getting errors when running `/mcp list` in Cursor chat terminal, follow these troubleshooting steps.

## Common Issues

### 1. MCP Server Configuration Errors

**Symptom**: `/mcp list` fails with errors about specific servers

**Solution**: Check each server configuration in `.cursor/mcp.json`:

```bash

# Validate JSON syntax

python3 -m json.tool .cursor/mcp.json > /dev/null && echo "✅ Valid" || echo "❌ Invalid"

# List configured servers

cat .cursor/mcp.json | python3 -c "import json, sys; print('\n'.join(json.load(sys.stdin)['mcpServers'].keys()))"
```

### 2. Server Dependency Issues

**Symptom**: Specific servers fail to load (e.g., semgrep)

**Known Issues**:

- `mcp-server-semgrep` has dependency issues with `@modelcontextprotocol/sdk`
- Some servers require specific Node.js/npm versions

**Solution**: Test each server manually:

```bash

# Test exarp

uvx exarp --mcp --help

# Test filesystem

npx -y @modelcontextprotocol/server-filesystem --help

# Test tractatus_thinking

npx -y tractatus_thinking --version

# Test sequential_thinking

npx -y @modelcontextprotocol/server-sequential-thinking --version

# Test context7

npx -y @upstash/context7-mcp --version

# Test agentic-tools

npx -y @pimzino/agentic-tools-mcp --version
```

### 3. Cursor Developer Tools Check

**Symptom**: `/mcp list` fails silently or with unclear errors

**Solution**: Check Cursor logs:

1. Open Cursor
2. Go to: **Help → Toggle Developer Tools**
3. Check **Console** tab for MCP errors
4. Look for specific error messages about:
   - Server startup failures
   - Missing dependencies
   - Configuration errors

### 4. Restart Cursor

**Symptom**: Changes to `.cursor/mcp.json` not taking effect

**Solution**:

1. **Quit Cursor completely** (Cmd+Q on macOS, Alt+F4 on Linux/Windows)
2. **Restart Cursor**
3. Try `/mcp list` again

## Current Configuration Status

Your `.cursor/mcp.json` has 8 servers configured:

1. ✅ **exarp** - Project management automation
2. ⚠️ **semgrep** - Security scanning (known dependency issues)
3. ✅ **filesystem** - File operations
4. ✅ **notebooklm** - Research and knowledge base
5. ✅ **context7** - Documentation lookup
6. ✅ **agentic-tools** - Task management
7. ✅ **sequential_thinking** - Implementation workflows
8. ✅ **tractatus_thinking** - Logical concept analysis

## Quick Fixes

### Fix 1: Temporarily Disable Problematic Servers

If a specific server is causing issues, temporarily comment it out in `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    // "semgrep": {
    //   "command": "npx",
    //   "args": ["-y", "mcp-server-semgrep"]
    // }
  }
}
```

### Fix 2: Verify Prerequisites

```bash

# Check Node.js version (should be v18+ or v20+)

node --version

# Check npm version (should be 9+)

npm --version

# Check uvx availability

which uvx || echo "uvx not found - install with: pip install uv"
```

### Fix 3: Clear npm Cache

If packages are corrupted:

```bash
npm cache clean --force
```

## Alternative: Use Natural Language

Instead of `/mcp list`, you can ask in Cursor chat:

- "List all MCP servers"
- "Show me configured MCP servers"
- "What MCP tools are available?"

The AI assistant can use the `list_mcp_resources` tool to show available resources.

## Getting Help

1. **Check Cursor logs** (Developer Tools → Console)
2. **Test commands manually** in terminal
3. **Check MCP server documentation**:
   - [MCP_TROUBLESHOOTING.md](MCP_TROUBLESHOOTING.md) - General troubleshooting
   - [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - Server documentation
4. **Verify Node.js/npm versions** are compatible

## See Also

- [MCP_TROUBLESHOOTING.md](MCP_TROUBLESHOOTING.md) - Comprehensive troubleshooting guide
- [MCP_QUICK_REFERENCE.md](MCP_QUICK_REFERENCE.md) - Quick reference for MCP usage
- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - Detailed server documentation
