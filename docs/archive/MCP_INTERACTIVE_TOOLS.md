# Interactive MCP Tools Recommendations

## Current Status

✅ **Currently Configured:**

- Semgrep (security scanning)
- NotebookLM (research/documentation)

❌ **Missing from Docs (Should Add):**

- Filesystem MCP (mentioned in docs but not configured)
- Git MCP (mentioned in docs but not configured)

## Recommended Interactive MCP Tools

### 1. **Browser Automation** ⭐ Highly Recommended

**Purpose**: Test and interact with your React web UI, verify PWA functionality

**Why You Need It:**

- You have a React web app (`web/`) with PWA support
- Need to verify UI functionality, test scenarios
- Can automate testing of box spread dashboard
- Verify offline PWA capabilities

**Configuration:**

```json
{
  "browser": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-browser"]
  }
}
```

**Use Cases:**

- Test web UI after changes
- Verify PWA installation and offline mode
- Screenshot dashboard states
- Test box spread scenario explorer
- Validate responsive design

### 2. **Terminal/Shell Execution** ⭐ Highly Recommended

**Purpose**: Safely execute build commands, tests, and trading scripts

**Why You Need It:**

- Run CMake builds, tests, linters
- Execute integration tests safely
- Run trading scripts in dry-run mode
- Execute multi-language build commands (C++, Python, Rust, TypeScript)

**Configuration:**

```json
{
  "terminal": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-terminal"]
  }
}
```

**Use Cases:**

- Run `./scripts/build_universal.sh`
- Execute `ctest --output-on-failure`
- Run `./scripts/run_linters.sh`
- Test integration scripts safely
- Execute Python strategy runner in dry-run mode

**Safety Note**: For trading software, always use dry-run mode when testing via terminal MCP.

### 3. **GitHub Integration** ⭐ Recommended

**Purpose**: Manage PRs, issues, code reviews directly from Cursor

**Why You Need It:**

- You have CI/CD workflows (see `agents/shared/CI.md`)
- Manage issues and PRs
- Code review assistance
- Release management

**Configuration:**

```json
{
  "github": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-github"],
    "env": {
      "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_TOKEN}"
    }
  }
}
```

**Use Cases:**

- Create/update issues
- Review PRs
- Check CI status
- Manage releases
- Code review assistance

**Security**: Store token in environment variable, never commit to repo.

### 4. **QuestDB/PostgreSQL MCP** ⭐ Recommended

**Purpose**: Query time-series data, verify data ingestion

**Why You Need It:**

- You use QuestDB for time-series archiving (`python/integration/questdb_client.py`)
- Need to verify quote/trade data ingestion
- Query historical PnL, exposure metrics
- Debug data pipeline issues

**Configuration:**

```json
{
  "questdb": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-postgres"],
    "env": {
      "POSTGRES_CONNECTION_STRING": "postgresql://127.0.0.1:8812/qdb"
    }
  }
}
```

**Note**: QuestDB uses PostgreSQL wire protocol on port 8812 (default).

**Use Cases:**

- Query recent quotes: `SELECT * FROM quotes ORDER BY timestamp DESC LIMIT 100`
- Verify trade ingestion: `SELECT COUNT(*) FROM trades WHERE timestamp > NOW() - INTERVAL '1 hour'`
- Check data quality: `SELECT symbol, COUNT(*), MIN(timestamp), MAX(timestamp) FROM quotes GROUP BY symbol`
- Debug missing data issues

### 5. **Filesystem MCP** ✅ Should Add

**Purpose**: Enhanced file operations (already mentioned in docs but missing)

**Why You Need It:**

- Better context understanding
- Intelligent file operations
- Project structure awareness

**Configuration:**

```json
{
  "filesystem": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-filesystem", "${workspaceFolder}"]
  }
}
```

### 6. **Git MCP** ✅ Should Add

**Purpose**: Enhanced Git operations (already mentioned in docs but missing)

**Why You Need It:**

- Better commit message assistance
- Understand git history
- Branch management

**Configuration:**

```json
{
  "git": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-git", "--repository", "${workspaceFolder}"]
  }
}
```

## Priority Recommendations

### High Priority (Add Now)

1. **Filesystem MCP** - ✅ **DONE** - Added to `.cursor/mcp.json`
2. **Git MCP** - ✅ **DONE** - Added to `.cursor/mcp.json`
3. **Browser Automation** - ✅ **DONE** - Added to `.cursor/mcp.json`
4. **Terminal/Shell** - ✅ **DONE** - Added to `.cursor/mcp.json`

### Medium Priority (Add When Needed)

1. **GitHub Integration** - Useful for PR/issue management
2. **QuestDB MCP** - Useful when debugging data ingestion

### ✅ Recently Added

1. **Agentic Tools MCP** - ✅ **DONE** - Advanced task management and agent memories

## Security Considerations

### Trading Software Safety

- **Terminal MCP**: Always use `--dry-run` flag for trading scripts
- **Never execute live trading commands** via MCP
- **QuestDB MCP**: Read-only queries recommended for production data
- **GitHub MCP**: Use fine-grained tokens with minimal permissions

### General Security

- Store all tokens/credentials in environment variables
- Never commit secrets to repository
- Use separate tokens for different services
- Review MCP server permissions regularly

## Implementation Steps

1. **Add Missing Core Servers** (Filesystem, Git) - ✅ Done
2. **Add Browser Automation** (for web UI testing) - ✅ Done
3. **Add Terminal MCP** (for safe command execution) - ✅ Done
4. **Add GitHub** (when you need PR/issue management) - Optional
5. **Add QuestDB** (when debugging data pipeline) - Optional

## Example Workflow with Interactive Tools

### Testing Web UI Changes

1. Make changes to React components
2. Use **Browser MCP** to navigate to local dev server
3. Take screenshots of dashboard states
4. Verify PWA functionality
5. Test box spread scenario explorer

### Running Integration Tests

1. Use **Terminal MCP** to run `./scripts/integration_test.sh`
2. Monitor output for failures
3. Use **Git MCP** to check what changed
4. Use **Semgrep MCP** to scan for security issues
5. Use **GitHub MCP** to create PR with results

### Debugging Data Pipeline

1. Use **QuestDB MCP** to query recent quotes
2. Verify data ingestion timestamps
3. Check for missing symbols
4. Use **Terminal MCP** to restart strategy runner
5. Monitor logs via terminal

## See Also

- [MCP_SERVERS.md](./MCP_SERVERS.md) - Current MCP server documentation
- [CURSOR_SETUP.md](./CURSOR_SETUP.md) - General Cursor configuration
- [.cursorrules](../.cursorrules) - AI assistant guidelines
