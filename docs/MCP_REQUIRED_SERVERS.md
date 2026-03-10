# Required MCP Servers

This document lists all **required** MCP servers for the project and provides installation/configuration guidance.

**Status:** Exarp is now typically provided by **exarp-go** (Go MCP server). The Python/uvx options below are legacy; see `docs/EXARP_GO_MIGRATION_LEFTOVERS.md` for migration details.

## Required Servers (8 total)

All of these servers must be configured in `.cursor/mcp.json` for full project functionality:

### 1. exarp-go (Project Management Automation)

**Type**: exarp-go (Go binary)
**Purpose**: Project management automation tools (docs health, task alignment, duplicate detection, security scanning).

**Current setup:** Configure in `.cursor/mcp.json` with the path to your exarp-go binary or this repo's wrapper script. Example:

```json
{
  "exarp-go": {
    "command": "/absolute/path/to/ib_box_spread_full_universal/scripts/run_exarp_go.sh",
    "args": [],
    "env": {
      "PROJECT_ROOT": "/absolute/path/to/this/repo"
    }
  }
}
```

**Legacy note:** Older Python-based Exarp docs remain in the repo for migration history only. They are not the recommended setup for this project.

---

### 2. filesystem

**Type**: npm package
**Purpose**: File system operations

**Configuration**:

```json
{
  "filesystem": {
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-filesystem",
      "/absolute/path/to/project"
    ],
    "description": "File system operations for reading, writing, and managing project files"
  }
}
```

---

### 3. agentic-tools

**Type**: npm package
**Purpose**: Advanced task management and agent memories

**Configuration**:

```json
{
  "agentic-tools": {
    "command": "npx",
    "args": ["-y", "@pimzino/agentic-tools-mcp"],
    "description": "Advanced task management and agent memories with project-specific storage"
  }
}
```

---

### 4. context7

**Type**: npm package
**Purpose**: Up-to-date documentation lookup

**Configuration**:

```json
{
  "context7": {
    "command": "npx",
    "args": ["-y", "@upstash/context7-mcp"],
    "description": "Context7 MCP server - provides up-to-date, version-specific documentation and code examples"
  }
}
```

---

### 5. git

**Type**: npm package
**Purpose**: Git version control operations

**Configuration**:

```json
{
  "git": {
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-git",
      "--repository",
      "/absolute/path/to/project"
    ],
    "description": "Git version control operations"
  }
}
```

---

### 6. semgrep

**Type**: npm package
**Purpose**: Security scanning

**Configuration**:

```json
{
  "semgrep": {
    "command": "npx",
    "args": ["-y", "@semgrep/mcp-server-semgrep"],
    "description": "Security scanning for code analysis - checks for security vulnerabilities, bugs, and code quality issues"
  }
}
```

---

### 7. tractatus_thinking

**Type**: npm package
**Purpose**: Logical concept analysis and structured thinking

**Configuration**:

```json
{
  "tractatus_thinking": {
    "command": "npx",
    "args": ["-y", "tractatus_thinking"],
    "description": "Tractatus Thinking MCP server for logical concept analysis and structured thinking - breaks down complex concepts into atomic truths, reveals multiplicative relationships, and finds missing elements"
  }
}
```

**⚠️ Important**:

- Package name is `tractatus_thinking` (with **underscore**), not `tractatus-thinking` (with hyphen)
- Use `npx` for installation, not Python

**Troubleshooting**: If you see "package not found", verify the package name:

```bash
npm search tractatus_thinking
npx -y tractatus_thinking --version
```

---

### 8. sequential_thinking

**Type**: npm package
**Purpose**: Implementation workflows and structured problem-solving

**Configuration**:

```json
{
  "sequential_thinking": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"],
    "description": "Sequential Thinking MCP server for structured problem-solving and implementation workflow - converts structural understanding from Tractatus Thinking into actionable implementation steps"
  }
}
```

**⚠️ Important**:

- This is an **npm package**, not a Python module
- Package name: `@modelcontextprotocol/server-sequential-thinking`
- Use `npx` for installation, not `python3 -m sequential_thinking`

**Troubleshooting**: If you see "No module named sequential_thinking":

1. The error indicates it's trying to run as a Python module
2. Update configuration to use npm package: `@modelcontextprotocol/server-sequential-thinking`
3. Test manually: `npx -y @modelcontextprotocol/server-sequential-thinking --version`

---

## Installation Verification

Test all servers manually:

```bash
# exarp-go
./scripts/run_exarp_go.sh -list

# npm packages

npx -y @modelcontextprotocol/server-filesystem --version
npx -y @pimzino/agentic-tools-mcp --version
npx -y @upstash/context7-mcp --version
npx -y @modelcontextprotocol/server-git --version
npx -y @semgrep/mcp-server-semgrep --version
npx -y tractatus_thinking --version
npx -y @modelcontextprotocol/server-sequential-thinking --version
```

## Common Issues

### Issue 1: "No module named sequential_thinking"

**Cause**: Configuration uses Python module syntax instead of npm package.

**Fix**: Update `.cursor/mcp.json`:

```json
{
  "sequential_thinking": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"]
  }
}
```

### Issue 2: "tractatus-thinking package not found"

**Cause**: Wrong package name (hyphen instead of underscore).

**Fix**: Use `tractatus_thinking` (with underscore):

```json
{
  "tractatus_thinking": {
    "command": "npx",
    "args": ["-y", "tractatus_thinking"]
  }
}
```

### Issue 3: exarp server import errors

**Cause**: Package not installed or wrong Python environment.

**Fix**:

1. Install `exarp-go` or use the sibling `../mcp/exarp-go` checkout
2. Verify installation: `./scripts/run_exarp_go.sh -list -quiet`
3. Check `.cursor/mcp.json` or local editor config points at `scripts/run_exarp_go.sh` or a native `exarp-go` binary

## Workflow Integration

These servers work together in a recommended workflow:

1. **tractatus_thinking** → Understand WHAT (structure/logic)
2. **exarp-go** → Analyze and automate (project management)
3. **sequential_thinking** → Plan HOW (implementation)

See [EXARP_MCP_TOOLS_USAGE.md](EXARP_MCP_TOOLS_USAGE.md) for current workflow examples.

## See Also

- [MCP_TROUBLESHOOTING.md](MCP_TROUBLESHOOTING.md) - Detailed troubleshooting guide
- [MCP_QUICK_REFERENCE.md](MCP_QUICK_REFERENCE.md) - Quick usage reference
- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - Complete MCP server documentation
