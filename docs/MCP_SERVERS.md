# MCP Servers Configuration

This document describes the Model Context Protocol (MCP) servers configured for this project to enhance Cursor AI capabilities.

## Configured MCP Servers

### ✅ Currently Active

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

### 4. NotebookLM Server

**Purpose**: Zero-hallucination knowledge base for summarizing YouTube videos, documentation, and links

**Status**: ✅ Configured

### 5. Filesystem Server

**Purpose**: Enhanced file system operations for better context understanding

**Status**: ✅ Configured (recently added)

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

- Better context understanding of project structure
- Intelligent file operations
- Enhanced AI suggestions based on file structure

### 6. Git Server

**Purpose**: Enhanced Git version control operations

**Status**: ✅ Configured (recently added)

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

- AI understands git history and changes
- Better commit message assistance
- Understands branch structure and diffs
- Can help with merge conflicts

### 7. Browser Automation Server

**Purpose**: Test and interact with React web UI, verify PWA functionality

**Status**: ✅ Configured

**Configuration**: `.cursor/mcp.json`

```json
{
  "browser": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-browser"]
  }
}
```

**Benefits**:

- Test web UI after changes
- Verify PWA installation and offline mode
- Screenshot dashboard states
- Test box spread scenario explorer
- Validate responsive design

**Use Cases**:

- Navigate to local dev server (`http://localhost:5173`)
- Take screenshots of dashboard states
- Test box spread scenario explorer
- Verify PWA offline functionality
- Test responsive design on different viewport sizes

### 8. Terminal/Shell Server

**Purpose**: Safely execute build commands, tests, and trading scripts

**Status**: ✅ Configured

**Configuration**: `.cursor/mcp.json`

```json
{
  "terminal": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-terminal"]
  }
}
```

**Benefits**:

- Run CMake builds, tests, linters
- Execute integration tests safely
- Run trading scripts in dry-run mode
- Execute multi-language build commands (C++, Python, Rust, TypeScript)

**Use Cases**:

- Run `./scripts/build_universal.sh`
- Execute `ctest --output-on-failure`
- Run `./scripts/run_linters.sh`
- Test integration scripts safely
- Execute Python strategy runner in dry-run mode

**⚠️ Safety Note**: For trading software, always use `--dry-run` flag when testing via terminal MCP. Never execute live trading commands.

---

## Additional Interactive MCP Tools (Optional)

For more interactive tools like GitHub integration and database queries, see [MCP_INTERACTIVE_TOOLS.md](MCP_INTERACTIVE_TOOLS.md).

---

## VSCode LSP MCP Extension

**Extension**: `yutengjing.vscode-mcp` (recommended in `.vscode/extensions.json`)

**Purpose**: Bridges VS Code/Cursor's Language Server Protocol (LSP) with MCP, exposing LSP diagnostics, type information, and code navigation to AI assistants.

**Benefits**:

- AI can access real-time diagnostics from all your LSPs (C++, Python, Rust, TypeScript, Swift)
- Type information available to AI for better code suggestions
- Code navigation features accessible to AI assistants
- Better context understanding for multi-language projects

**Configuration**:

- Install the extension: `yutengjing.vscode-mcp`
- Extension automatically creates MCP server
- Socket file location: `/Users/davidlowes/Library/Application Support/YuTengjing.vscode-mcp/vscode-mcp-*.sock`
- Cursor's MCP bridge connects automatically

**Troubleshooting**:

- If you see permission errors, see [MCP_TROUBLESHOOTING.md](MCP_TROUBLESHOOTING.md) for permission fixes
- Extension must be installed and enabled
- Restart Cursor after installation

### 9. Agentic Tools MCP Server

**Purpose**: Advanced task management and agent memories with project-specific storage

**Status**: ✅ Configured

**Configuration**: `.cursor/mcp.json`

```json
{
  "agentic-tools": {
    "command": "npx",
    "args": ["-y", "@pimzino/agentic-tools-mcp"]
  }
}
```

**Benefits**:

- **Unlimited Task Hierarchy**: Projects → Tasks → Subtasks → infinite depth nesting
- **Agent Memories**: Persistent context storage across AI sessions
- **Project-Specific Storage**: Each project has isolated `.agentic-tools-mcp/` directory
- **Git-Trackable**: Task and memory data can be committed with code
- **Rich Task Metadata**: Priority, complexity, dependencies, tags, time tracking
- **Intelligent Features**: PRD parsing, task recommendations, complexity analysis

**Use Cases for Your Project**:

- Track TWS API integration tasks hierarchically
- Store agent memories about trading strategies, API patterns, decisions
- Manage box spread implementation tasks with dependencies
- Share task lists and memories via git with team
- Maintain context about trading software requirements across sessions

**Storage Location**: `.agentic-tools-mcp/` in project root

- `tasks.json` - Projects, tasks, and subtasks
- `memories/` - JSON files organized by category (preferences, technical, context)

**Key Features**:

- Unlimited nesting depth for tasks
- Parent-child relationships with automatic level tracking
- Dependency management across hierarchy
- Priority (1-10) and complexity (1-10) at every level
- Status workflow: pending → in-progress → blocked → done
- Tag-based organization and filtering
- Time tracking (estimated and actual hours)

---

## Additional MCP Servers (Optional)

**Configuration**: `.cursor/mcp.json`

```json
{
  "notebooklm": {
    "command": "npx",
    "args": ["-y", "notebooklm-mcp@latest"]
  }
}
```

**Benefits**:

- Summarize YouTube videos and convert them to markdown documentation
- Process documentation links and extract key information
- Zero-hallucination answers based on your uploaded sources
- Pre-processed by Gemini 2.5 for intelligent synthesis
- Citation-backed answers with source references
- Natural language Q&A that understands context across multiple documents

**Usage**: The AI assistant can use NotebookLM to research topics, summarize videos, and create documentation. See [NotebookLM Usage Guide](NOTEBOOKLM_USAGE.md) for detailed instructions.

**Key Features**:

- Upload PDFs, Google Docs, markdown files, websites, GitHub repos, and YouTube videos
- Ask questions and get synthesized answers from your sources
- Save notebooks in a library with tags for easy retrieval
- Autonomous research with follow-up questions
- Browser automation for seamless integration

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
- **NotebookLM**: Uses browser automation with local Chrome profile. Credentials never leave your machine. Consider using a dedicated Google account for automation.
- **Never commit**: API keys, tokens, or credentials in MCP configuration

## See Also

- [Cursor Setup Guide](CURSOR_SETUP.md) - General Cursor IDE configuration
- [NotebookLM Usage Guide](NOTEBOOKLM_USAGE.md) - How to use NotebookLM for summarizing videos and documentation
- [.cursorrules](../.cursorrules) - AI assistant guidelines mentioning Semgrep
- [MCP Documentation](https://modelcontextprotocol.io/) - Official MCP documentation
- [NotebookLM MCP Repository](https://github.com/PleasePrompto/notebooklm-mcp) - Source code and detailed documentation
