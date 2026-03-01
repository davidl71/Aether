# MCP Server uvx Migration

**Date**: 2025-12-24  
**Status**: ✅ Migration Complete  

**Note:** Exarp is now typically provided by **exarp-go** (Go binary). The uvx exarp option below is legacy; see `docs/EXARP_GO_MIGRATION_LEFTOVERS.md`.

## Summary

Migrated Python-based MCP servers to use `uvx` wherever possible for faster, more reliable package execution.

## Changes Made

### ✅ exarp Server

**Before:**

```json
{
  "exarp": {
    "command": "python3",
    "args": ["-m", "exarp_automation_mcp.server"],
    "env": {
      "PROJECT_ROOT": "/Users/davidl/Projects/Trading/ib_box_spread_full_universal"
    }
  }
}
```

**After:**

```json
{
  "exarp": {
    "command": "uvx",
    "args": ["exarp", "--mcp"],
    "env": {
      "PROJECT_ROOT": "/Users/davidl/Projects/Trading/ib_box_spread_full_universal"
    }
  }
}
```

**Benefits:**

- ✅ Faster package resolution (uvx uses uv's fast resolver)
- ✅ Automatic environment management (no manual venv needed)
- ✅ Better dependency isolation
- ✅ Environment variables preserved

### ✅ ollama Server

**Status**: Already using `uvx` ✅

```json
{
  "ollama": {
    "command": "uvx",
    "args": ["mcp-ollama"],
    "env": {
      "OLLAMA_BASE_URL": "http://localhost:11434"
    }
  }
}
```

## Servers That Cannot Use uvx

The following servers use npm packages and cannot be migrated to `uvx` (uvx is for Python packages only):

1. **Todo2** - `npx todo2-extension-todo2`
2. **agentic-tools** - `npx @pimzino/agentic-tools-mcp`
3. **context7** - `npx @upstash/context7-mcp`
4. **filesystem** - `npx @modelcontextprotocol/server-filesystem`
5. **git** - `npx @modelcontextprotocol/server-git`
6. **semgrep** - `npx @semgrep/mcp-server-semgrep`
7. **sequential_thinking** - `npx @modelcontextprotocol/server-sequential-thinking`
8. **tractatus_thinking** - `npx tractatus-thinking-mcp`

## Final Configuration

### uvx Servers (2 total) ✅

- ✅ **exarp** - `uvx exarp --mcp` (migrated from `python3 -m`)
- ✅ **ollama** - `uvx mcp-ollama` (already using uvx)

### npm Servers (8 total)

- **Todo2** - `npx -y todo2-extension-todo2`
- **agentic-tools** - `npx -y @pimzino/agentic-tools-mcp`
- **context7** - `npx -y @upstash/context7-mcp`
- **filesystem** - `npx -y @modelcontextprotocol/server-filesystem`
- **git** - `npx -y @modelcontextprotocol/server-git`
- **semgrep** - `npx -y @semgrep/mcp-server-semgrep`
- **sequential_thinking** - `npx -y @modelcontextprotocol/server-sequential-thinking`
- **tractatus_thinking** - `npx -y tractatus-thinking-mcp`

## Benefits of uvx

1. **Faster**: 10-100x faster than pip for dependency resolution
2. **Automatic**: No manual virtual environment creation needed
3. **Isolated**: Each package runs in its own isolated environment
4. **Reliable**: Better dependency resolution with lockfile support
5. **Cross-platform**: Works on macOS, Linux, and Windows

## Verification

To verify the migration:

```bash
# Check exarp works with uvx
uvx exarp --help

# Check ollama works with uvx
uvx mcp-ollama --help
```

## Next Steps

1. ✅ Restart Cursor to load updated MCP configuration
2. ✅ Verify exarp MCP server works correctly
3. ✅ Test that all MCP tools are accessible

---

**Last Updated**: 2025-12-24
**Status**: Migration Complete, Ready for Testing
