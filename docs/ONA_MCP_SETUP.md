# Ona MCP Configuration Guide

This document explains how to configure Model Context Protocol (MCP) servers for use with **Ona** (formerly Gitpod), a cloud-based development environment platform with AI agent capabilities.

## What is Ona?

[Ona](https://ona.com) is a software development platform that provides:

- **Ona Environments**: Secure, ephemeral, pre-configured development environments
- **Ona Agents**: Professional software engineering agents that execute tasks autonomously
- **Ona Guardrails**: Security, compliance, and governance controls

Ona supports MCP to extend AI agent capabilities beyond built-in features, similar to how Cursor uses MCP.

## Ona vs Cursor MCP Configuration

### Key Differences

| Feature | Cursor (`.cursor/mcp.json`) | Ona (`.ona/mcp-config.json`) |
|---------|---------------------------|------------------------------|
| **Location** | `.cursor/mcp.json` | `.ona/mcp-config.json` |
| **Format** | Simplified JSON | Extended JSON with additional fields |
| **Environment Variables** | Direct values | `${exec:...}`, `${file:...}` patterns |
| **Docker Support** | Limited | Full Docker support |
| **Security Controls** | Basic | Advanced (toolDenyList, timeouts, isolation) |
| **Global Settings** | Not supported | `globalTimeout`, `logLevel` |

### Configuration File Structure

**Cursor Format** (`.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "semgrep": {
      "command": "npx",
      "args": ["-y", "@semgrep/mcp-server-semgrep"]
    }
  }
}
```

**Ona Format** (`.ona/mcp-config.json`):

```json
{
  "mcpServers": {
    "semgrep": {
      "name": "semgrep",
      "command": "npx",
      "args": ["-y", "@semgrep/mcp-server-semgrep"],
      "disabled": false,
      "timeout": 30,
      "description": "Security scanning"
    }
  },
  "globalTimeout": 45,
  "logLevel": "info"
}
```

## Current Configuration

The project includes both configurations:

- **Cursor**: `.cursor/mcp.json` - For local Cursor IDE development
- **Ona**: `.ona/mcp-config.json` - For Ona cloud environments

### Configured MCP Servers

Both configurations include:

1. **Semgrep** - Security vulnerability scanning
2. **Filesystem** - File operations
3. **Git** - Version control operations
4. **Context7** - Up-to-date documentation access
5. **NotebookLM** - Research and knowledge base creation
6. **Tractatus Thinking** - Logical concept analysis
7. **Sequential Thinking** - Implementation workflow planning

## Ona-Specific Features

### Environment Variable Resolution

Ona supports dynamic environment variable resolution:

```json
{
  "env": {
    "GITHUB_TOKEN": "${exec:printenv GITHUB_TOKEN}",
    "API_KEY": "${file:/path/to/secret/file}"
  }
}
```

**Patterns**:

- `${exec:...}` - Execute command and use output
- `${file:/path}` - Read from file (for secrets)
- `${workspaceFolder}` - Resolves to workspace root

### Docker-Based Servers

Ona supports running MCP servers in Docker containers:

```json
{
  "github": {
    "name": "github",
    "command": "docker",
    "args": [
      "run",
      "-i",
      "--rm",
      "-e",
      "GITHUB_PERSONAL_ACCESS_TOKEN",
      "ghcr.io/github/github-mcp-server"
    ],
    "env": {
      "GITHUB_PERSONAL_ACCESS_TOKEN": "${exec:printf 'protocol=https\nhost=github.com\n' | git credential fill 2>/dev/null | awk -F= '/password/ {print $2}' 2>/dev/null}"
    }
  }
}
```

### Security Controls

**Tool Deny List**:

```json
{
  "toolDenyList": ["dangerous*", "rm*", "delete_*"]
}
```

**Timeouts**:

```json
{
  "timeout": 30,  // Per-server timeout
  "globalTimeout": 45  // Global timeout
}
```

**Process Isolation**:

- Each MCP server runs as a separate process
- Docker containers provide additional isolation
- Environment variables isolated per server

## Setup Instructions

### 1. Prerequisites

- Ona account (sign up at [app.gitpod.io](https://app.gitpod.io))
- Project repository accessible to Ona

### 2. Configuration File

The `.ona/mcp-config.json` file is already configured in this repository. Ona will automatically detect and use it when you:

1. Open the project in Ona
2. Start an Ona Agent session
3. The MCP servers will be initialized automatically

### 3. Credential Management

**Recommended**: Use Ona Secrets for sensitive credentials:

1. Navigate to Ona Settings → Secrets
2. Create environment variable secrets for tokens
3. Reference in MCP config: `${exec:printenv YOUR_VAR}`

**Example**:

```json
{
  "env": {
    "GITHUB_TOKEN": "${exec:printenv GITHUB_TOKEN}",
    "LINEAR_API_KEY": "${exec:printenv LINEAR_API_KEY}"
  }
}
```

### 4. Organization Controls (Enterprise)

Organization owners can control MCP usage:

1. Navigate to **Settings → Agents**
2. Locate **MCP controls**
3. Toggle MCP on/off for the organization

When disabled:

- `.ona/mcp-config.json` files are ignored
- Ona Agent operates with built-in tools only
- External MCP server connections are blocked

## Usage

### Starting Ona Agent

1. Open project in Ona environment
2. Access Ona Agent interface
3. MCP servers will be automatically available
4. Use AI agents with extended MCP capabilities

### Example Workflows

**Security Scanning**:

```
Agent: "Scan this code for security vulnerabilities"
→ Uses Semgrep MCP server automatically
```

**Research**:

```
Agent: "Summarize this YouTube video about TWS API"
→ Uses NotebookLM MCP server
```

**Documentation**:

```
Agent: "Show me FastAPI async patterns use context7"
→ Uses Context7 MCP server for current docs
```

**Problem Solving**:

```
Agent: "Analyze why box spread calculations fail"
→ Uses Tractatus Thinking to break down the problem
→ Uses Sequential Thinking to create implementation steps
```

## Troubleshooting

### MCP Servers Not Connecting

1. **Check Ona Agent logs**:
   - Open Ona Agent interface
   - Check connection status for each server

2. **Verify Node.js/Python**:

   ```bash
   node --version
   python3 --version
   ```

3. **Test MCP server manually**:

   ```bash
   npx -y @semgrep/mcp-server-semgrep --help
   ```

4. **Check timeouts**:
   - Increase `timeout` for slow servers
   - Check `globalTimeout` setting

### Permission Errors

- Ensure MCP servers have necessary permissions
- Check Docker container permissions (if using Docker)
- Verify file system access for filesystem server

### Environment Variable Issues

- Verify secrets are set in Ona Settings
- Check `${exec:...}` command syntax
- Test command manually: `printenv GITHUB_TOKEN`

## Additional Resources

- [Ona MCP Documentation](https://ona.com/docs/ona/mcp)
- [Ona Getting Started](https://ona.com/docs/ona/getting-started)
- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [Project MCP Servers Documentation](../docs/ MCP_SERVERS.md)

## Migration from Cursor

If you're migrating from Cursor to Ona:

1. **Copy configuration**: `.cursor/mcp.json` → `.ona/mcp-config.json`
2. **Add Ona-specific fields**: `name`, `disabled`, `timeout`, `description`
3. **Update environment variables**: Use `${exec:...}` patterns
4. **Add global settings**: `globalTimeout`, `logLevel`
5. **Test in Ona environment**: Verify all servers connect

## Security Best Practices

1. **Never commit secrets** to `.ona/mcp-config.json`
2. **Use Ona Secrets** for sensitive credentials
3. **Enable tool deny lists** for dangerous operations
4. **Set appropriate timeouts** to prevent hanging
5. **Use Docker isolation** for untrusted servers
6. **Review organization controls** for enterprise deployments

---

**Note**: This configuration is optimized for the IBKR Box Spread Generator project. Adjust server configurations based on your specific needs and security requirements.
