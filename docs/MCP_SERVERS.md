# MCP Servers Configuration

This document describes the Model Context Protocol (MCP) servers configured for this project to enhance Cursor AI capabilities.

## Configured MCP Servers

### 1. Semgrep Security Scanner
**Purpose**: Automated security vulnerability scanning and code quality analysis

**Configuration**: `.cursor/mcp.json`
```json
{
  "semgrep": {
    "command": "npx",
    "args": ["-y", "@semgrep/mcp-server-semgrep"]
  }
}
```

**Benefits**:
- Scans code for security vulnerabilities
- Detects common bugs and anti-patterns
- Provides code quality suggestions
- Especially important for trading software where security is critical

**Usage**: The AI assistant will automatically use Semgrep when analyzing code for security issues, as mentioned in `.cursorrules`.

### 2. Filesystem Server
**Purpose**: File system operations for reading, writing, and managing project files

**Configuration**: `.cursor/mcp.json`
```json
{
  "filesystem": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-filesystem", "${workspaceFolder}"]
  }
}
```

**Benefits**:
- Allows AI to read and write files intelligently
- Better context understanding of project structure
- Can create/modify files as needed

### 3. Git Server
**Purpose**: Git version control operations

**Configuration**: `.cursor/mcp.json`
```json
{
  "git": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-git", "--repository", "${workspaceFolder}"]
  }
}
```

**Benefits**:
- AI can understand git history and changes
- Can help with commit messages
- Understands branch structure and diffs

## Setup Instructions

### Prerequisites
- Node.js and npm installed (for `npx`)
- Cursor IDE with MCP support

### Installation

The MCP servers are configured in `.cursor/mcp.json`. Cursor will automatically:
1. Detect the configuration file
2. Install MCP servers via `npx` when first used
3. Connect to the servers as needed

### Manual Setup (if needed)

If automatic setup doesn't work, you can manually install:

```bash
# Install Semgrep MCP server globally (optional)
npm install -g @semgrep/mcp-server-semgrep

# Or use npx (recommended - no global install needed)
npx -y @semgrep/mcp-server-semgrep
```

## Configuration File Location

The MCP configuration is stored at: `.cursor/mcp.json`

**Note**: This file should be committed to the repository so all team members have the same MCP server setup.

## Troubleshooting

### MCP Servers Not Connecting

1. **Check Node.js installation**:
   ```bash
   node --version
   npm --version
   ```

2. **Verify npx is available**:
   ```bash
   which npx
   ```

3. **Test MCP server manually**:
   ```bash
   npx -y @semgrep/mcp-server-semgrep --help
   ```

4. **Check Cursor logs**:
   - Open Cursor
   - Check Developer Tools → Console for MCP connection errors

### Workspace Folder Variable

If `${workspaceFolder}` doesn't resolve correctly, you may need to:
1. Use absolute path in `.cursor/mcp.json`
2. Or set environment variable: `export WORKSPACE_FOLDER=/path/to/project`

## Additional MCP Servers (Optional)

### GitHub MCP Server
For GitHub integration:
```json
{
  "github": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-github"],
    "env": {
      "GITHUB_PERSONAL_ACCESS_TOKEN": "your-token"
    }
  }
}
```

### PostgreSQL MCP Server
If you add database support:
```json
{
  "postgres": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-postgres"],
    "env": {
      "POSTGRES_CONNECTION_STRING": "postgresql://..."
    }
  }
}
```

## Security Considerations

- **Semgrep**: Scans code but doesn't send data externally (runs locally)
- **Filesystem**: Only has access to `${workspaceFolder}` directory
- **Git**: Only has access to the configured repository
- **Never commit**: API keys, tokens, or credentials in MCP configuration

## See Also

- [Cursor Setup Guide](CURSOR_SETUP.md) - General Cursor IDE configuration
- [.cursorrules](../.cursorrules) - AI assistant guidelines mentioning Semgrep
- [MCP Documentation](https://modelcontextprotocol.io/) - Official MCP documentation
