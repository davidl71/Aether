# MCP Servers Configuration

This document describes the Model Context Protocol (MCP) servers configured for this project to
enhance Cursor AI capabilities.

## Configured MCP Servers

### ✅ Currently Active

### 1. automa (Project Management Automation)

**Purpose**: Self-hosted project management automation tools for documentation health, task alignment, duplicate detection, security scanning, and automation opportunities

**Status**: ✅ Configured and Active

**Configuration**: `.cursor/mcp.json`

```json
{
  "automa": {
    "command": "/Users/davidl/Projects/Trading/ib_box_spread_full_universal/mcp-servers/project-management-automation/run_server.sh",
    "args": [],
    "description": "Project management automation tools - documentation health, task alignment, duplicate detection, security scanning, and automation opportunities"
  }
}
```

**Server Location**: `mcp-servers/project-management-automation/`

**Benefits**:

- Documentation health checks with Todo2 integration
- Todo2 task alignment analysis with project goals
- Duplicate task detection and consolidation
- Multi-language dependency security scanning (Python, Rust, npm)
- Automation opportunity discovery
- PWA configuration review
- Task synchronization between systems
- Project-specific analysis (not generic operations)

**Usage**: The AI assistant will automatically use automa tools when you ask for:
- Documentation health checks
- Task alignment analysis
- Duplicate task detection
- Security scanning
- Automation opportunities

**See Also**: `.cursor/rules/project-automation.mdc` for detailed usage guide

### 2. Semgrep Security Scanner

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

**Usage**: The AI assistant will automatically use Semgrep when analyzing code for security
issues, as mentioned in `.cursorrules`.

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

**Configuration**: `.cursor/mcp.json`

```json
{
  "notebooklm": {
    "command": "uvx",
    "args": [
      "mcpower-proxy==0.0.87",
      "--wrapped-config",
      "{\"command\": \"npx\", \"args\": [\"-y\", \"notebooklm-mcp@latest\"]}"
    ]
  }
}
```

**Benefits**:

- Summarize YouTube videos and documentation
- Create zero-hallucination knowledge bases
- Extract key information from links

### 5. Context7 Server

**Purpose**: Up-to-date, version-specific documentation and code examples

**Status**: ✅ Configured

**Configuration**: `.cursor/mcp.json`

```json
{
  "context7": {
    "command": "npx",
    "args": ["-y", "@upstash/context7-mcp"]
  }
}
```

**Benefits**:

- Access current, version-specific documentation
- Get accurate code examples for latest API versions
- Ensure code generation uses up-to-date references
- No need to leave coding environment for docs

**Usage**: Append `use context7` to your prompts to access current documentation. For example:

- "How do I use FastAPI async endpoints? use context7"
- "Show me React hooks examples use context7"
- "CMake best practices 2025 use context7"

### 6. Filesystem Server

**Purpose**: Enhanced file system operations for better context understanding

**Status**: ✅ Configured

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

### 7. Git Server

**Purpose**: Enhanced Git version control operations

**Status**: ✅ Configured

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

### 8. Browser Automation Server

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

### 9. Terminal/Shell Server

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

**⚠️ Safety Note**: For trading software, always use `--dry-run` flag when testing via terminal
MCP. Never execute live trading commands.

---

## Additional Interactive MCP Tools (Optional)

For more interactive tools like GitHub integration and database queries, see [MCP_INTERACTIVE_TOOLS.md](MCP_INTERACTIVE_TOOLS.md).

---

## VSCode LSP MCP Extension

**Extension**: `yutengjing.vscode-mcp` (recommended in `.vscode/extensions.json`)

**Purpose**: Bridges VS Code/Cursor's Language Server Protocol (LSP) with MCP, exposing LSP
diagnostics, type information, and code navigation to AI assistants.

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

### 10. iTerm2 MCP Server

**Purpose**: Provide terminal context to Cursor agents by integrating with iTerm2

**Status**: ✅ Configured

**Configuration**: `.cursor/mcp.json`

```json
{
  "iterm2": {
    "command": "npx",
    "args": ["-y", "@rishabkoul/iterm-mcp-server"],
    "description": "iTerm2 MCP server - provides terminal context to Cursor agents"
  }
}
```

**Benefits**:

- **Terminal Context**: AI can see your active iTerm2 sessions, current directories, and command history
- **Service Status**: AI understands which services are running in your tmux sessions
- **Command Execution**: AI can execute commands in iTerm2 terminals (with your permission)
- **Output Reading**: AI can read terminal output to understand errors, logs, and service status
- **Session Management**: AI can create new terminal sessions or interact with existing ones
- **Real-time Awareness**: AI knows what's happening in your terminal environment

**Use Cases for Your Project**:

- **Service Monitoring**: AI can check if PWA services are running, see port status, view logs
- **Debugging**: AI can read error messages from terminal output and suggest fixes
- **Build Verification**: AI can execute build commands and see results
- **Integration Testing**: AI can run tests and understand test output
- **Context Sharing**: When you ask about services, AI can see actual terminal state

**How It Works**:

1. **Terminal State**: AI can see current directory, shell, recent commands, exit status
2. **Command History**: AI can view command history for context
3. **Active Sessions**: AI can list and interact with active iTerm2/tmux sessions
4. **Output Capture**: AI can read terminal output to understand what's happening
5. **Command Execution**: With permissions, AI can run commands to diagnose issues

**Integration with Launch Script**:

The launch script now includes `[AI:ANALYZE]` markers that make it easy for AI to understand:

- Which services are running
- Port assignments
- Service dependencies
- Gateway status

**Permissions** (configured in iTerm2):

When using iTerm2's AI Chat feature, you can grant permissions for:

- **Check Terminal State**: See directory, shell, commands, exit status
- **Run Commands**: Execute diagnostic commands
- **View History**: Access command history
- **View Manpages**: Reference documentation

**⚠️ Security Note**: The MCP server respects iTerm2's permission system. You control what AI can access.

### 11. Tractatus Thinking MCP Server

**Purpose**: Logical concept analysis and structured thinking for breaking down complex problems

**Status**: ✅ Configured and Active

**Configuration**: `.cursor/mcp.json`

```json
{
  "tractatus_thinking": {
    "command": "npx",
    "args": ["-y", "tractatus_thinking"],
    "description": "Tractatus Thinking MCP server for logical concept analysis and structured thinking - breaks down complex concepts into atomic truths, reveals multiplicative relationships, and finds missing elements"
  }
}
```

**⚠️ Important**: Package name is `tractatus_thinking` (with underscore), not `tractatus-thinking` (with hyphen).

**Benefits**:

- **Break Down Complex Concepts**: Decompose complex ideas into atomic truths
- **Reveal Multiplicative Relationships**: Understand why ALL factors must align (A × B × C)
- **Find Missing Elements**: Identify the ONE critical missing piece preventing success
- **Separate Essential from Accidental**: Focus on what must be true vs what happens to be true
- **Eliminate Confusion**: Create precise definitions that eliminate team confusion
- **Understand WHY vs HOW**: Provides structural understanding beyond surface-level implementation

**Use Cases for Your Project**:

- **Trading Logic Analysis**: Break down box spread arbitrage into atomic components
- **Architecture Decisions**: Understand multiplicative dependencies for successful deployments
- **Debugging**: Find the ONE missing element causing systematic failures
- **Requirements Analysis**: Separate essential features from nice-to-haves
- **Concept Clarification**: Define fuzzy trading/finance terms precisely

**Strategic Sequencing**:

1. **Start with Tractatus Thinking** to understand WHAT (structure/logic)
2. **Switch to sequential thinking** for HOW (process/steps)
3. **Return to Tractatus Thinking** to formalize and verify solution structure

**Example Usage**:

```
Problem: "Why do all box spread calculations fail?"
Tractatus reveals: Success = Valid Market Data × Correct Strikes × Proper Pricing × API Connection
Missing element: API Connection always fails
Solution: Fix API connection, now all factors align
```

**Operations**:

- `start`: Begin analysis of a concept or problem
- `add`: Add propositions and decompose into components
- `navigate`: Explore the logical structure
- `export`: Export analysis in markdown, JSON, or Graphviz
- `revise`: Update understanding as you learn more

**Integration**:

- Use BEFORE implementation to understand structure
- Combine with codebase search for existing patterns
- Use with research to determine what needs investigation
- Store structural insights in OpenMemory for future reference

**Documentation**: See `.cursor/rules/tractatus-thinking.mdc` for detailed usage guidelines.

### 12. Sequential Thinking MCP Server

**Purpose**: Structured problem-solving and implementation workflow

**Status**: ✅ Configured and Active

**Configuration**: `.cursor/mcp.json`

```json
{
  "sequential_thinking": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"],
    "description": "Sequential Thinking MCP server for structured problem-solving and implementation workflow - converts structural understanding from Tractatus Thinking into actionable implementation steps"
  }
}
```

**⚠️ Important**: This is an npm package, not a Python module. Use `npx` with `@modelcontextprotocol/server-sequential-thinking`.

**Benefits**:

- **Structured Problem-Solving**: Dynamic and reflective problem-solving through structured thought sequences
- **Implementation Workflow**: Converts structural understanding into actionable implementation steps
- **Step-by-Step Reasoning**: Breaks down complex problems into sequential, manageable steps
- **Reflective Analysis**: Enables dynamic reflection and refinement of problem-solving approaches
- **Process-Oriented Thinking**: Focuses on HOW to implement solutions (complements Tractatus Thinking's WHAT analysis)

**Use Cases for Your Project**:

- **Implementation Planning**: Convert Tractatus Thinking structural analysis into concrete implementation steps
- **Workflow Design**: Create step-by-step workflows for complex features
- **Problem Decomposition**: Break down implementation tasks into sequential, manageable components
- **Reflective Problem-Solving**: Iteratively refine implementation approaches based on feedback
- **Process Documentation**: Document implementation processes and workflows

**Strategic Sequencing with Tractatus Thinking**:

1. **Start with Tractatus Thinking** to understand WHAT (structure/logic) - breaks down concepts into atomic components
2. **Switch to Sequential Thinking** for HOW (process/steps) - converts structural understanding into actionable implementation steps
3. **Return to Tractatus Thinking** to formalize and verify solution structure - ensures all components align

**Example Workflow**:

```
Step 1 (Tractatus): "What is the structure of box spread arbitrage?"
→ Reveals: Success = Valid Market Data × Correct Strikes × Proper Pricing × API Connection

Step 2 (Sequential): "How do we implement box spread arbitrage?"
→ Creates:
  1. Connect to TWS API
  2. Fetch market data for strikes
  3. Calculate pricing for all legs
  4. Validate arbitrage opportunity
  5. Execute trade if valid

Step 3 (Tractatus): "Verify implementation structure"
→ Confirms: All multiplicative dependencies are addressed in implementation
```

**Installation**:

The Sequential Thinking MCP server is an npm package. Installation is automatic via `npx`:

```bash
# Test installation
npx -y @modelcontextprotocol/server-sequential-thinking --version
```

**Note**: No manual installation required. `npx` automatically downloads and runs the package.

**Integration**:

- Use AFTER Tractatus Thinking analysis to convert structural understanding into steps
- Combine with codebase search for existing implementation patterns
- Use with research to determine specific implementation approaches
- Store implementation workflows in OpenMemory for future reference

**Documentation**: See `.cursor/rules/sequential-thinking.mdc` for detailed usage guidelines (to be created).

---

## Optional MCP Server Examples

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

**Usage**: The AI assistant can use NotebookLM to research topics, summarize videos, and create
documentation. See [NotebookLM Usage Guide](NOTEBOOKLM_USAGE.md) for detailed instructions.

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

**Note**: This file should be committed to the repository so all team members have the same MCP
server setup.

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
- **NotebookLM**: Uses browser automation with local Chrome profile. Credentials never leave your
  machine. Consider using a dedicated Google account for automation.
- **Never commit**: API keys, tokens, or credentials in MCP configuration

## See Also

- [Cursor Setup Guide](CURSOR_SETUP.md) - General Cursor IDE configuration
- [NotebookLM Usage Guide](NOTEBOOKLM_USAGE.md) - How to use NotebookLM for summarizing videos and documentation
- [.cursorrules](../.cursorrules) - AI assistant guidelines mentioning Semgrep
- [MCP Documentation](https://modelcontextprotocol.io/) - Official MCP documentation
- [NotebookLM MCP Repository](https://github.com/PleasePrompto/notebooklm-mcp) - Source code
  and detailed documentation
