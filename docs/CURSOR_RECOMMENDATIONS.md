# Cursor IDE Recommendations

This document provides comprehensive recommendations for optimizing your Cursor IDE setup for this project.

## MCP Servers

MCP (Model Context Protocol) servers enhance Cursor's AI capabilities. See [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) for detailed configuration.

### Recommended MCP Servers

1. **Semgrep** - Security scanning (required per `.cursorrules`)
2. **Filesystem** - File operations
3. **Git** - Version control operations

## VS Code Extensions

### Essential Extensions (Already Recommended)

✅ **C++ Development**

- `ms-vscode.cpptools` - C++ IntelliSense, debugging
- `ms-vscode.cmake-tools` - CMake integration

✅ **Python Development**

- `ms-python.python` - Python language support
- `ms-python.vscode-pylance` - Fast Python language server
- `ms-python.black-formatter` - Code formatting

✅ **Multi-Language Support**

- `rust-lang.rust-analyzer` - Rust (for agents/backend)
- C++ extensions (TUI is now C++ with FTXUI)
- `sswg.swift-lang` - Swift (for ios/ and desktop/)

✅ **Code Quality**

- `timonwong.shellcheck` - Shell script linting
- `streetsidesoftware.code-spell-checker` - Spell checking
- `usernamehw.errorlens` - Inline error highlighting

### Optional But Useful Extensions

**C++ Static Analysis**:

- The project uses `cppcheck` (command-line tool) via `scripts/run_linters.sh`
- `cpptools` extension already includes some static analysis
- Consider installing `cppcheck` CLI: `brew install cppcheck`

**Python Security**:

- The project uses `bandit` (command-line tool) via `scripts/run_linters.sh`
- Consider installing `bandit` CLI: `pip install bandit`
- VS Code extension for Bandit may not be necessary if using CLI

**AI & Prompt Enhancement**:

- **Prompt Tower** (`prompttower.prompttower`) - Enhance and refine prompts for better AI assistance
  - One-click prompt enhancement
  - Works with @docs feature
  - Supports multiple formats (JSON, XML, YAML)
  - See [PROMPT_TOWER_USAGE.md](PROMPT_TOWER_USAGE.md) for detailed guide

**Additional Useful Extensions** (not in recommendations but useful):

- **ClangFormat** (built into cpptools) - Code formatting
- **Better Comments** - Colorize comments by type
- **Todo Tree** - Highlight TODO/FIXME comments
- **Bracket Pair Colorizer** (built into VS Code now)
- **Indent Rainbow** - Colorize indentation

## Command-Line Tools

These tools are used by `scripts/run_linters.sh` and should be installed:

```bash

# C++ Static Analysis

brew install cppcheck

# Python Security Scanning

pip install bandit

# Go Linting

brew install golangci-lint

# Swift Linting

brew install swiftlint

# Clang Static Analyzer (usually comes with Xcode)

xcode-select --install
```

## Workspace Settings

Key settings in `.vscode/settings.json`:

- **C++ IntelliSense**: Configured for C++20
- **Format on Save**: Enabled
- **100-character ruler**: Visual guide for line length
- **File exclusions**: Build artifacts hidden
- **CMake integration**: Auto-configuration

## MCP Server Benefits

### Semgrep MCP

- **Security scanning**: Automatically scans code for vulnerabilities
- **Bug detection**: Finds common programming errors
- **Code quality**: Suggests improvements
- **Trading software focus**: Critical for financial applications

### Filesystem MCP

- **Context awareness**: AI understands project structure
- **File operations**: Can read/write files intelligently
- **Better suggestions**: More accurate code recommendations

### Git MCP

- **Version control**: AI understands git history
- **Commit assistance**: Helps write better commit messages
- **Change tracking**: Understands what changed and why

## Performance Tips

1. **Use CMake presets**: Faster configuration
2. **Enable ccache**: 10-100x faster rebuilds (see `scripts/build_fast.sh`)
3. **Exclude build directories**: Reduces file indexing overhead
4. **Use Ninja generator**: Faster than Make

## Troubleshooting

### MCP Servers Not Working

- Ensure Node.js/npm is installed: `node --version`
- Check `.cursor/mcp.json` syntax
- Restart Cursor after configuration changes

### IntelliSense Not Working

- Run CMake configure: `cmake --preset macos-universal-debug`
- Check `compile_commands.json` exists in build directory
- Reload window: `Cmd+Shift+P` → "Developer: Reload Window"

### Extensions Not Installing

- Check internet connection
- Verify extension IDs are correct
- Try installing manually from Extensions view

## See Also

- [CURSOR_AI_TUTORIAL.md](CURSOR_AI_TUTORIAL.md) - Cursor AI tutorial and best practices
- [CURSOR_SETUP.md](research/integration/CURSOR_SETUP.md) - Complete Cursor setup guide
- [CURSOR_DOCS_USAGE.md](research/integration/CURSOR_DOCS_USAGE.md) - Using @docs in Cursor
- [PROMPT_TOWER_USAGE.md](PROMPT_TOWER_USAGE.md) - Prompt Tower extension usage guide
- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - MCP server configuration details
- [.cursorrules](../.cursorrules) - AI assistant guidelines
