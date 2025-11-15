# MCP Server Quick Reference

Quick reference for using MCP servers in your Cursor prompts.

## Available MCP Servers

### ✅ Currently Active

1. **Semgrep** - Security scanning
2. **Filesystem** - File operations
3. **Git** - Version control
4. **NotebookLM** - Research and documentation

### 🔍 Available but Not Configured

5. **GitKraken** - Enhanced Git workflow, PRs, issues
6. **Cursor IDE Browser** - Browser automation

## Quick Prompt Examples

### Security Scanning

```
Scan this code with Semgrep for security vulnerabilities:
[code block]
```

```
Before I commit, check this file for security issues using Semgrep
```

### Git Operations

```
List all branches using GitKraken MCP
```

```
Create a PR for the current branch using GitKraken
```

```
Show me the git status using GitKraken
```

### Research & Documentation

```
Use NotebookLM to research TWS API error handling patterns
```

```
What notebooks are available in my NotebookLM library?
```

```
Summarize this YouTube video about options trading using NotebookLM: [URL]
```

### Browser Testing

```
Navigate to localhost:3000 and test the WASM build using browser MCP
```

```
Take a screenshot of the React app running in the browser
```

### File Operations

```
Read the contents of native/src/box_spread_strategy.cpp
```

```
Create a new file at docs/NEW_FEATURE.md with this content: [content]
```

## When to Use Each MCP

### Semgrep
- ✅ Before committing security-sensitive code
- ✅ When writing trading logic
- ✅ When handling credentials or API keys
- ✅ As part of code review process

### GitKraken
- ✅ Creating pull requests
- ✅ Managing branches and worktrees
- ✅ Tracking issues related to features
- ✅ Retrieving code from specific commits
- ✅ Adding comments to PRs/issues

### NotebookLM
- ✅ Researching TWS API topics
- ✅ Summarizing YouTube tutorials
- ✅ Creating documentation from links
- ✅ Getting answers from your knowledge base
- ⚠️ Always ask permission first if task isn't explicitly about these topics

### Filesystem
- ✅ Reading project files
- ✅ Creating new files
- ✅ Modifying existing files
- ✅ Understanding project structure
- ℹ️ Automatically used by AI - no explicit prompt needed

### Browser
- ✅ Testing WASM builds
- ✅ Verifying React components
- ✅ Testing web UI functionality
- ✅ Validating API endpoints
- ✅ Taking screenshots for documentation

## Integration with .cursorrules

The AI assistant will automatically:
- Use Semgrep when you mention security scanning
- Use Filesystem for file operations
- Use Git/GitKraken for version control tasks
- Use NotebookLM when you ask about research or documentation
- Use Browser when you mention testing web components

## Best Practices

1. **Be explicit**: "Use Semgrep to scan this code" is better than "check for security issues"
2. **Ask permission**: For NotebookLM, always ask if the task isn't explicitly about research
3. **Combine MCPs**: "Use GitKraken to create a PR, then use Semgrep to scan the changed files"
4. **Test first**: Test MCP functionality before relying on it for critical tasks

## Troubleshooting

### MCP Not Responding
- Restart Cursor
- Check `.cursor/mcp.json` syntax
- Verify Node.js/npm/uvx is installed
- Check Cursor Developer Tools → Console for errors

### MCP Not Available
- Ensure MCP server is configured in `.cursor/mcp.json`
- Check that the command/args are correct
- Verify the MCP server package is available via npx/uvx

## See Also

- [MCP_EXTENSIONS_INTEGRATION.md](MCP_EXTENSIONS_INTEGRATION.md) - Comprehensive integration guide
- [MCP_SERVERS.md](MCP_SERVERS.md) - Detailed MCP server configuration
- [NOTEBOOKLM_USAGE.md](NOTEBOOKLM_USAGE.md) - NotebookLM usage guide
