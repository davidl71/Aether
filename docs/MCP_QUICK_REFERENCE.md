# MCP Server Quick Reference

Quick reference for using MCP servers in your Cursor prompts.

## Available MCP Servers

### ✅ Required MCP Servers (8 total)

1. **exarp-go** - Project management automation for this repo
2. **Semgrep** - Security scanning
3. **Filesystem** - File operations
4. **Git** - Version control
5. **agentic-tools** - Advanced task management
6. **context7** - Documentation lookup
7. **tractatus_thinking** - Logical concept analysis (structural thinking)
8. **sequential_thinking** - Implementation workflows (process thinking)

**Note**: All 8 servers are required for full project functionality. See [MCP_TROUBLESHOOTING.md](MCP_TROUBLESHOOTING.md) for installation and troubleshooting.

### 🔍 Available but Not Configured

1. **GitKraken** - Enhanced Git workflow, PRs, issues
2. **Cursor IDE Browser** - Browser automation

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

### exarp-go

- ✅ Documentation health checks with Todo2 integration
- ✅ Todo2 task alignment analysis
- ✅ Duplicate task detection
- ✅ Multi-language dependency security scanning
- ✅ Automation opportunity discovery
- ✅ PWA configuration review
- ✅ Task synchronization
- ⚠️ **PREFERRED TOOL** for project-specific analysis (see `.cursor/rules/project-automation.mdc`)
- 💡 **Works with**: tractatus_thinking (structure) → exarp-go (analysis) → sequential_thinking (implementation)

### tractatus_thinking

- ✅ Break down complex concepts into atomic components
- ✅ Reveal multiplicative dependencies (A × B × C must ALL be true)
- ✅ Find missing elements preventing success
- ✅ Use BEFORE exarp-go tools to understand problem structure
- 💡 **Workflow**: tractatus → exarp-go → sequential

### sequential_thinking

- ✅ Convert structural analysis into implementation steps
- ✅ Create step-by-step workflows
- ✅ Use AFTER exarp-go analysis to plan fixes
- 💡 **Workflow**: tractatus → exarp-go → sequential

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
- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - Detailed MCP server configuration
- **NotebookLM:** `.cursor/rules/notebooklm.mdc` - when to use NotebookLM MCP
