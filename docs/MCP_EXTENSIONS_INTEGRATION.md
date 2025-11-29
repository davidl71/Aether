# MCP Servers, Extensions, and LSP Integration Guide

This document provides a comprehensive analysis of your current Cursor setup and recommendations for better integration of MCP servers, extensions, and LSPs.

## Current State Analysis

### ✅ Currently Configured MCP Servers

Your `.cursor/mcp.json` includes:

1. **Semgrep** - Security scanning (via uvx/mcpower-proxy)
2. **Filesystem** - File operations (via uvx/mcpower-proxy)
3. **Git** - Version control (via uvx/mcpower-proxy)
4. **NotebookLM** - Research and documentation (via uvx/mcpower-proxy)

### ✅ Currently Configured Extensions

Your `.vscode/extensions.json` includes comprehensive language support:

- **C++**: `ms-vscode.cpptools`, `ms-vscode.cmake-tools`
- **Python**: `ms-python.python`, `ms-python.vscode-pylance`, `ms-python.black-formatter`
- **Rust**: `rust-lang.rust-analyzer`
- **Go**: `golang.go`
- **TypeScript**: `dbaeumer.vscode-eslint`, `esbenp.prettier-vscode`
- **Swift**: `sswg.swift-lang`
- **General**: GitLens, EditorConfig, Markdown tools, ShellCheck, Error Lens, Prompt Tower
- **MCP Integration**: `yutengjing.vscode-mcp` - Bridges LSP features to MCP for AI assistants

### ✅ Currently Configured LSPs

LSPs are configured via extensions in `.vscode/settings.json`:

- **C++**: IntelliSense via cpptools (clang-based)
- **Python**: Pylance (Microsoft's fast Python language server)
- **Rust**: rust-analyzer (via extension)
- **TypeScript**: Built-in TypeScript language server
- **Go**: gopls (via golang.go extension)

### ✅ VSCode LSP MCP Extension

**Extension**: `yutengjing.vscode-mcp`

**Purpose**: Bridges VS Code/Cursor's Language Server Protocol (LSP) features with MCP, allowing AI assistants to access:

- Real-time LSP diagnostics (errors, warnings, hints)
- Type information from language servers
- Code navigation features (go-to-definition, find references)
- IntelliSense data from all configured LSPs

**Benefits for Your Project**:

- **Multi-language Intelligence**: AI can access diagnostics from C++, Python, Rust, TypeScript, Swift
- **Better Code Understanding**: AI sees the same type information and errors as your editor
- **Enhanced Suggestions**: More accurate code recommendations with full LSP context
- **Trading Software Safety**: AI can see type errors and warnings before suggesting code changes

**How It Works**:

1. Extension creates an MCP server that exposes LSP data
2. Uses Unix socket at: `/Users/davidlowes/Library/Application Support/YuTengjing.vscode-mcp/vscode-mcp-*.sock`
3. Cursor's MCP bridge connects to this server automatically
4. AI assistants can query LSP diagnostics, types, and navigation data

**Status**: ✅ Recommended in `.vscode/extensions.json`

## Available but Not Configured

### 🔍 GitKraken MCP Server

**Status**: Available in your tool list but not configured in `.cursor/mcp.json`

**Benefits**:

- Enhanced Git operations (branches, commits, stashes, worktrees)
- Issue tracking integration (GitHub, GitLab, Jira, Linear, Azure DevOps)
- Pull request management (create, review, comment)
- Repository file content retrieval
- Better Git workflow automation

**Use Cases for Your Project**:

- Automate PR creation when implementing features
- Track issues related to TWS API integration
- Manage multiple worktrees (you already use worktrees)
- Retrieve file content from specific commits/branches
- Add comments to issues/PRs programmatically

### 🔍 Cursor IDE Browser MCP Server

**Status**: Available in your tool list but not configured

**Benefits**:

- Browser automation for testing web components
- Web scraping for market data (if needed)
- Automated testing of your React web app
- Documentation verification
- API endpoint testing

**Use Cases for Your Project**:

- Test WASM builds in the browser
- Verify web UI components
- Automate testing of your React frontend
- Scrape market data from public sources (if needed)

## Recommendations

### 1. Add GitKraken MCP Server

**Why**: Your project uses Git worktrees extensively, and GitKraken MCP provides better Git workflow automation.

**Configuration** (add to `.cursor/mcp.json`):

```json
{
  "mcpServers": {
    // ... existing servers ...
    "gitkraken": {
      "command": "uvx",
      "args": [
        "mcpower-proxy==0.0.87",
        "--wrapped-config",
        "{\n      \"command\": \"npx\",\n      \"args\": [\n        \"-y\",\n        \"@gitkraken/mcp-server-gitkraken\"\n      ],\n      \"description\": \"GitKraken MCP server for enhanced Git workflow, issue tracking, and PR management\"\n    }",
        "--name",
        "gitkraken"
      ]
    }
  }
}
```

**Integration in `.cursorrules`**:
Add guidance for when to use GitKraken MCP:

- When creating PRs or managing branches
- When tracking issues related to features
- When working with multiple worktrees
- When retrieving historical code from specific commits

### 2. Add Cursor IDE Browser MCP Server

**Why**: Your project has a React web frontend and WASM builds that need browser testing.

**Configuration** (add to `.cursor/mcp.json`):

```json
{
  "mcpServers": {
    // ... existing servers ...
    "cursor-ide-browser": {
      "command": "uvx",
      "args": [
        "mcpower-proxy==0.0.87",
        "--wrapped-config",
        "{\n      \"command\": \"npx\",\n      \"args\": [\n        \"-y\",\n        \"@cursor-ide/mcp-server-browser\"\n      ],\n      \"description\": \"Browser automation for testing web components, WASM builds, and React frontend\"\n    }",
        "--name",
        "cursor-ide-browser"
      ]
    }
  }
}
```

**Integration in `.cursorrules`**:
Add guidance for when to use browser MCP:

- When testing WASM builds
- When verifying React components
- When testing API endpoints
- When validating web UI functionality

### 3. Enhance `.cursorrules` with MCP-Specific Guidance

Add a new section to `.cursorrules`:

```markdown
## MCP Server Usage Guidelines

### Semgrep MCP
- **Always use** when writing security-sensitive code (credentials, API keys, trading logic)
- **Use before committing** to scan for vulnerabilities
- **Reference in prompts**: "Scan this code with Semgrep for security issues"

### GitKraken MCP
- **Use for**: Creating PRs, managing branches, tracking issues, retrieving historical code
- **When working with worktrees**: Use GitKraken to manage multiple worktrees
- **For issue tracking**: Link code changes to GitHub/GitLab issues

### NotebookLM MCP
- **Use for**: Researching TWS API topics, summarizing videos, creating documentation
- **Reference**: `docs/NOTEBOOKLM_USAGE.md` for detailed instructions
- **Always ask permission** before consulting notebooks for tasks

### Filesystem MCP
- **Automatically used** by AI for file operations
- **No explicit prompts needed** - AI will use it when reading/writing files

### Cursor IDE Browser MCP
- **Use for**: Testing WASM builds, verifying React components, testing web UI
- **When building web features**: Use browser MCP to test in real browser environment
```

### 4. Optimize LSP Configurations

#### C++ LSP Enhancements

Your current C++ configuration is good, but consider adding:

```json
{
  "C_Cpp.intelliSenseEngine": "default",
  "C_Cpp.intelliSenseEngineFallback": "enabled",
  "C_Cpp.default.compilerPath": "/usr/bin/clang++",
  "C_Cpp.default.cStandard": "c17",
  "C_Cpp.default.cppStandard": "c++20",
  "C_Cpp.default.intelliSenseMode": "macos-clang-arm64",
  "C_Cpp.codeAnalysis.clangTidy.path": "/usr/local/bin/clang-tidy",
  "C_Cpp.codeAnalysis.clangTidy.config": "${workspaceFolder}/.clang-tidy"
}
```

**Consider creating `.clang-tidy` file** for consistent static analysis:

```yaml
Checks: >
  clang-analyzer-*,
  readability-*,
  performance-*,
  modernize-*,
  -readability-magic-numbers,
  -modernize-use-trailing-return-type
WarningsAsErrors: ''
HeaderFilterRegex: '.*'
```

#### Python LSP Enhancements

Your Pylance configuration is good. Consider adding:

```json
{
  "python.analysis.typeCheckingMode": "basic",  // Consider "strict" for new code
  "python.analysis.diagnosticMode": "workspace",
  "python.analysis.stubPath": "${workspaceFolder}/python/stubs",
  "python.analysis.autoImportCompletions": true,
  "python.analysis.completeFunctionParens": true
}
```

#### Rust LSP Enhancements

Your rust-analyzer configuration is good. Consider adding:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-W", "clippy::all"],
  "rust-analyzer.cargo.buildScripts.enable": true,
  "rust-analyzer.cargo.allFeatures": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true
}
```

### 5. Additional Extension Recommendations

#### C++ Development

- **clangd** (alternative to cpptools): Faster, more accurate IntelliSense
  - Extension: `llvm-vs-code-extensions.vscode-clangd`
  - Note: Disable cpptools IntelliSense if using clangd

#### Code Quality

- **Todo Tree** (`gruntfuggly.todo-tree`): Highlight TODO/FIXME comments
- **Better Comments** (`aaron-bond.better-comments`): Colorize comments by type
- **Error Lens** (already recommended): Inline error highlighting

#### Testing

- **Test Explorer UI** (`hbenl.vscode-test-explorer`): Unified test interface
- **C++ TestMate** (`matepek.vscode-catch2-test-adapter`): Catch2 test integration

#### Documentation

- **Markdown Preview Enhanced** (`shd101wyy.markdown-preview-enhanced`): Better markdown preview
- **Doxygen Documentation Generator** (`cschlosser.doxdocgen`): Auto-generate Doxygen comments

### 6. Prompt Optimization for MCP Integration

Update your prompts to explicitly leverage MCP servers:

**Example prompts that benefit from MCP integration**:

1. **Security-focused prompts**:

   ```
   "Before committing, scan this code with Semgrep for security vulnerabilities"
   ```

2. **Git workflow prompts**:

   ```
   "Create a PR for this feature branch using GitKraken MCP"
   ```

3. **Research prompts**:

   ```
   "Use NotebookLM to research TWS API error handling patterns"
   ```

4. **Testing prompts**:

   ```
   "Use browser MCP to test the WASM build in Chrome"
   ```

## Implementation Steps

### Step 1: Add GitKraken MCP

1. Update `.cursor/mcp.json` with GitKraken configuration
2. Restart Cursor
3. Test with: "List all branches using GitKraken"

### Step 2: Add Browser MCP

1. Update `.cursor/mcp.json` with browser configuration
2. Restart Cursor
3. Test with: "Navigate to localhost:3000 and take a screenshot"

### Step 3: Update `.cursorrules`

1. Add MCP usage guidelines section
2. Add examples of when to use each MCP
3. Update AI assistant guidelines to reference MCPs

### Step 4: Optimize LSP Settings

1. Create `.clang-tidy` file for C++
2. Update `.vscode/settings.json` with enhanced LSP configs
3. Test IntelliSense improvements

### Step 5: Install Additional Extensions

1. Review recommendations above
2. Install extensions that match your workflow
3. Update `.vscode/extensions.json` if needed

## Testing Your Setup

### Test MCP Servers

```bash
# Test Semgrep
# In Cursor chat: "Scan native/src/box_spread_strategy.cpp with Semgrep"

# Test GitKraken
# In Cursor chat: "List all git branches using GitKraken"

# Test NotebookLM
# In Cursor chat: "What notebooks are available in NotebookLM?"

# Test Browser
# In Cursor chat: "Navigate to https://example.com and take a screenshot"
```

### Test LSPs

1. **C++**: Open a `.cpp` file, check IntelliSense works
2. **Python**: Open a `.py` file, check type hints work
3. **Rust**: Open a `.rs` file, check rust-analyzer diagnostics
4. **TypeScript**: Open a `.ts` file, check ESLint integration

## Troubleshooting

### MCP Servers Not Connecting

1. Check Node.js: `node --version` (should be v18+)
2. Check uvx: `uvx --version` (should be installed)
3. Check Cursor logs: Developer Tools → Console
4. Restart Cursor after configuration changes

### LSP Not Working

1. **C++**: Run `cmake --preset macos-universal-debug` to generate `compile_commands.json`
2. **Python**: Check Python interpreter path in settings
3. **Rust**: Run `rustup update` to ensure rust-analyzer is current
4. **TypeScript**: Run `npm install` in `web/` directory

### Extensions Not Installing

1. Check internet connection
2. Verify extension IDs are correct
3. Try installing manually from Extensions view
4. Check Cursor/VS Code version compatibility

## Benefits Summary

### With Enhanced MCP Integration

✅ **Better Git Workflow**: Automated PR creation, issue tracking, worktree management
✅ **Enhanced Security**: Automated Semgrep scanning before commits
✅ **Better Research**: NotebookLM for TWS API documentation and tutorials
✅ **Automated Testing**: Browser MCP for web component testing
✅ **Improved Context**: AI understands project structure better

### With Optimized LSPs

✅ **Faster IntelliSense**: Better code completion and error detection
✅ **Better Type Checking**: Catch errors before compilation
✅ **Improved Navigation**: Go-to-definition, find references
✅ **Inline Diagnostics**: See errors as you type

### With Additional Extensions

✅ **Better Code Quality**: Todo tracking, comment highlighting
✅ **Improved Testing**: Unified test interface
✅ **Better Documentation**: Auto-generated comments, enhanced markdown

## Next Steps

1. **Review this document** and decide which recommendations to implement
2. **Start with GitKraken MCP** - most immediately useful for your workflow
3. **Add Browser MCP** if you're actively developing web features
4. **Update `.cursorrules`** to guide AI on when to use each MCP
5. **Test thoroughly** before committing changes

## See Also

- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - Detailed MCP server configuration
- [CURSOR_RECOMMENDATIONS.md](CURSOR_RECOMMENDATIONS.md) - Extension recommendations
- [CURSOR_SETUP.md](research/integration/CURSOR_SETUP.md) - General Cursor setup
- [NOTEBOOKLM_USAGE.md](research/integration/NOTEBOOKLM_USAGE.md) - NotebookLM usage guide
- [.cursorrules](../.cursorrules) - AI assistant guidelines
