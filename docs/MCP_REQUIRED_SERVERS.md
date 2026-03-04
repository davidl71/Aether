# Required MCP Servers

This document lists all **required** MCP servers for the project and provides installation/configuration guidance.

**Status:** Use **exarp-go** only. The Python MCP server (project-management-automation / exarp_automation_mcp) is **deprecated** and no longer used in this repo. See `docs/EXARP_GO_MIGRATION_LEFTOVERS.md` for context.

## Required Servers (8 total)

All of these servers must be configured in `.cursor/mcp.json` for full project functionality:

### 1. exarp-go (Project Management Automation)

**Type**: exarp-go (Go binary, installed or on PATH)
**Purpose**: Project management automation (docs health, task alignment, duplicate detection, security scanning).

**Setup:** Configure in `.cursor/mcp.json` using the project's runner script so exarp-go sees the correct project root. Example (see `docs/MCP_CONFIG_EXAMPLE.json`):

```json
{
  "exarp-go": {
    "command": "{{PROJECT_ROOT}}/scripts/run_exarp_go.sh",
    "args": [],
    "env": { "PROJECT_ROOT": "{{PROJECT_ROOT}}", "EXARP_WATCH": "0" }
  }
}
```

Ensure **exarp-go** is installed (on PATH or set `EXARP_GO_ROOT`). See `docs/PORTABLE_BUILD_AND_RUNNER.md`.

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

Test exarp-go:

```bash
# exarp-go (required)
exarp-go --help
# or via project script:
./scripts/run_exarp_go.sh --help
```

**npm packages:**

```bash
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

### Issue 3: exarp-go not found or not starting

**Cause**: exarp-go not on PATH or wrapper script not used.

**Fix**:

1. Install exarp-go (e.g. `go install` or build from source) and ensure it is on PATH, or set `EXARP_GO_ROOT` to the exarp-go repo and use `scripts/run_exarp_go.sh`.
2. In `.cursor/mcp.json` use `"exarp-go"` with command `{{PROJECT_ROOT}}/scripts/run_exarp_go.sh` and env `PROJECT_ROOT`. See `docs/MCP_CONFIG_EXAMPLE.json` and `docs/PORTABLE_BUILD_AND_RUNNER.md`.

## Workflow Integration

These servers work together in a recommended workflow:

1. **tractatus_thinking** → Understand WHAT (structure/logic)
2. **exarp-go** → Analyze and automate (project management)
3. **sequential_thinking** → Plan HOW (implementation)

See `docs/PORTABLE_BUILD_AND_RUNNER.md` and `docs/MCP_CONFIG_EXAMPLE.json` for exarp-go setup.

## See Also

- [MCP_TROUBLESHOOTING.md](MCP_TROUBLESHOOTING.md) - Detailed troubleshooting guide
- [MCP_QUICK_REFERENCE.md](MCP_QUICK_REFERENCE.md) - Quick usage reference
- [MCP_SERVERS.md](research/integration/MCP_SERVERS.md) - Complete MCP server documentation
