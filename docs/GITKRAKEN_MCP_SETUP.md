# GitKraken MCP Server Setup

## Overview

GitKraken MCP server provides enhanced Git workflow capabilities, including:
- **Git Operations**: Branch management, commit history, diffs
- **Issue Tracking**: GitHub, GitLab, Jira, Linear, Azure DevOps
- **PR Management**: Create, review, and manage pull requests
- **Project Context**: Access to repository information

## Installation Steps

### 1. Install GitKraken CLI

**macOS (Homebrew)**:
```bash
brew install gitkraken-cli
```

**Or download directly**:
- Visit: https://www.gitkraken.com/cli
- Download the latest release for your platform
- Follow installation instructions

### 2. Authenticate GitKraken CLI

After installation, authenticate with your GitKraken account:

```bash
gk auth login
```

This will:
- Open a browser for authentication
- Link your GitKraken account
- Enable access to your repositories and integrations

### 3. Verify Installation

Check that GitKraken CLI is installed and working:

```bash
gk --version
gk mcp --help
```

### 4. Configure MCP in Cursor

The GitKraken MCP server is already configured in `.cursor/mcp.json`:

```json
{
  "gitkraken": {
    "command": "gk",
    "args": ["mcp"],
    "description": "GitKraken MCP server for enhanced Git workflow, issue tracking (GitHub, GitLab, Jira, Linear, Azure DevOps), and PR management."
  }
}
```

### 5. Restart Cursor

After installing GitKraken CLI:
1. Restart Cursor IDE
2. The MCP server should automatically connect
3. You can verify in Cursor's MCP settings

## Usage Examples

Once enabled, you can use GitKraken MCP through Cursor's AI assistant:

### Git Operations
```
List all branches using GitKraken
Show me the git status using GitKraken
Create a new branch for feature X using GitKraken
```

### Pull Requests
```
Create a PR for the current branch using GitKraken
List all open PRs using GitKraken
Show me PR #123 details using GitKraken
```

### Issue Tracking
```
List all open issues using GitKraken
Create an issue for bug X using GitKraken
Show me issue #456 using GitKraken
```

## Troubleshooting

### GitKraken CLI Not Found

If you see errors about `gk` command not found:

1. **Check installation**:
   ```bash
   which gk
   ```

2. **Add to PATH** (if needed):
   - macOS: Usually installed to `/usr/local/bin/gk` or `~/.local/bin/gk`
   - Add to your shell profile if not in PATH

3. **Reinstall**:
   ```bash
   brew reinstall gitkraken-cli
   ```

### Authentication Issues

If authentication fails:

1. **Re-authenticate**:
   ```bash
   gk auth login
   ```

2. **Check authentication status**:
   ```bash
   gk auth status
   ```

3. **Logout and re-login**:
   ```bash
   gk auth logout
   gk auth login
   ```

### MCP Server Not Connecting

1. **Verify configuration**: Check `.cursor/mcp.json` syntax
2. **Restart Cursor**: MCP servers load on startup
3. **Check Cursor logs**: Look for MCP connection errors
4. **Test CLI directly**: `gk mcp` should work in terminal

## Integration with Other Services

GitKraken MCP supports integration with:

- **GitHub**: Full PR and issue management
- **GitLab**: Issue tracking and merge requests
- **Jira**: Issue tracking and project management
- **Linear**: Issue tracking and project management
- **Azure DevOps**: Work items and pull requests

Configure these integrations through your GitKraken account settings.

## References

- [GitKraken CLI Documentation](https://help.gitkraken.com/cli/)
- [GitKraken MCP Getting Started](https://help.gitkraken.com/mcp/mcp-getting-started/)
- [GitKraken CLI Download](https://www.gitkraken.com/cli)

