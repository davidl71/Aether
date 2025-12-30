# MCP Server Source Analysis

**Date**: 2025-12-24
**Status**: Complete Analysis

## Summary

- **Total Servers**: 10
- **Enabled**: 10 (100%)
- **Disabled**: 0 (0%)

## Enabled Servers

| #   | Name                | Source Type  | Package                                            | Registry       |
| --- | ------------------- | ------------ | -------------------------------------------------- | -------------- |
| 1   | Todo2               | npm          | `todo2-extension-todo2`                            | npmjs.com      |
| 2   | agentic-tools       | npm          | `@pimzino/agentic-tools-mcp`                       | npmjs.com      |
| 3   | context7            | npm          | `@upstash/context7-mcp`                            | npmjs.com      |
| 4   | exarp               | python (uvx) | `exarp`                                            | PyPI (via uvx) |
| 5   | filesystem          | npm          | `@modelcontextprotocol/server-filesystem`          | npmjs.com      |
| 6   | git                 | npm          | `@modelcontextprotocol/server-git`                 | npmjs.com      |
| 7   | ollama              | python (uvx) | `mcp-ollama`                                       | PyPI (via uvx) |
| 8   | semgrep             | npm          | `@semgrep/mcp-server-semgrep`                      | npmjs.com      |
| 9   | sequential_thinking | npm          | `@modelcontextprotocol/server-sequential-thinking` | npmjs.com      |
| 10  | tractatus_thinking  | npm          | `tractatus-thinking-mcp`                           | npmjs.com      |

## Source Type Breakdown

| Source Type  | Enabled | Disabled | Total |
| ------------ | ------- | -------- | ----- |
| npm          | 8       | 0        | 8     |
| python (uvx) | 2       | 0        | 2     |

## Detailed Server Information

### 1. Todo2 ✅ ENABLED
- **Source Type**: npm
- **Package**: `todo2-extension-todo2`
- **Command**: `npx`
- **Args**: `-y`, `todo2-extension-todo2`
- **Registry**: npmjs.com
- **Install**: `npm install -g todo2-extension-todo2`

### 2. agentic-tools ✅ ENABLED
- **Source Type**: npm
- **Package**: `@pimzino/agentic-tools-mcp`
- **Command**: `npx`
- **Args**: `-y`, `@pimzino/agentic-tools-mcp`
- **Registry**: npmjs.com
- **Install**: `npm install -g @pimzino/agentic-tools-mcp`

### 3. context7 ✅ ENABLED
- **Source Type**: npm
- **Package**: `@upstash/context7-mcp`
- **Command**: `npx`
- **Args**: `-y`, `@upstash/context7-mcp`
- **Registry**: npmjs.com
- **Install**: `npm install -g @upstash/context7-mcp`

### 4. exarp ✅ ENABLED
- **Source Type**: python (uvx)
- **Package**: `exarp`
- **Command**: `uvx`
- **Args**: `exarp`, `--mcp`
- **Registry**: PyPI (via uvx)
- **Install**: `uvx exarp`
- **Environment Variables**: `PROJECT_ROOT`

### 5. filesystem ✅ ENABLED
- **Source Type**: npm
- **Package**: `@modelcontextprotocol/server-filesystem`
- **Command**: `npx`
- **Args**: `-y`, `@modelcontextprotocol/server-filesystem`, `<project_path>`
- **Registry**: npmjs.com
- **Install**: `npm install -g @modelcontextprotocol/server-filesystem`

### 6. git ✅ ENABLED
- **Source Type**: npm
- **Package**: `@modelcontextprotocol/server-git`
- **Command**: `npx`
- **Args**: `-y`, `@modelcontextprotocol/server-git`, `--repository`, `<project_path>`
- **Registry**: npmjs.com
- **Install**: `npm install -g @modelcontextprotocol/server-git`

### 7. ollama ✅ ENABLED
- **Source Type**: python (uvx)
- **Package**: `mcp-ollama`
- **Command**: `uvx`
- **Args**: `mcp-ollama`
- **Registry**: PyPI (via uvx)
- **Install**: `uvx mcp-ollama`
- **Environment Variables**: `OLLAMA_BASE_URL`

### 8. semgrep ✅ ENABLED
- **Source Type**: npm
- **Package**: `@semgrep/mcp-server-semgrep`
- **Command**: `npx`
- **Args**: `-y`, `@semgrep/mcp-server-semgrep`
- **Registry**: npmjs.com
- **Install**: `npm install -g @semgrep/mcp-server-semgrep`

### 9. sequential_thinking ✅ ENABLED
- **Source Type**: npm
- **Package**: `@modelcontextprotocol/server-sequential-thinking`
- **Command**: `npx`
- **Args**: `-y`, `@modelcontextprotocol/server-sequential-thinking`
- **Registry**: npmjs.com
- **Install**: `npm install -g @modelcontextprotocol/server-sequential-thinking`

### 10. tractatus_thinking ✅ ENABLED
- **Source Type**: npm
- **Package**: `tractatus-thinking-mcp`
- **Command**: `npx`
- **Args**: `-y`, `tractatus-thinking-mcp`
- **Registry**: npmjs.com
- **Install**: `npm install -g tractatus-thinking-mcp`

## Observations

### Distribution
- **80% npm packages** (8 servers)
- **20% Python via uvx** (2 servers)

### All Servers Enabled
- No disabled servers found
- All 10 servers are active and configured

### Environment Variables
- **exarp**: Uses `PROJECT_ROOT` environment variable
- **ollama**: Uses `OLLAMA_BASE_URL` environment variable

### Package Sources
- **npm packages**: All use `npx -y` for execution (no global install required)
- **Python packages**: All use `uvx` for execution (fast, isolated environments)
  - `exarp` uses `uvx exarp --mcp`
  - `ollama` uses `uvx mcp-ollama`

## Recommendations

1. **All servers are enabled** - No action needed for disabled servers
2. **Source diversity is good** - Mix of npm and Python packages
3. **Consider documentation** - Document why each server is needed
4. **Monitor performance** - 10 servers may impact startup time

---

**Last Updated**: 2025-12-24
**Status**: Complete Analysis
