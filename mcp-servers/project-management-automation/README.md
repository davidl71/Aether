# Project Management Automation MCP Server

**Version:** 0.1.0
**Status:** Phase 1 Complete - Core Framework Ready

## Overview

MCP server exposing project management automation tools built on `IntelligentAutomationBase`. Provides AI assistants with access to documentation health checks, Todo2 analysis, duplicate detection, security scanning, and more.

## Phase 1 Status

✅ **Core Framework:** Complete
✅ **Package Configuration:** Complete
✅ **Error Handling:** Complete
⏳ **Tool Implementation:** Phase 2
⏳ **Resource Handlers:** Phase 3
⏳ **Testing:** Phase 4

## Installation

```bash
cd mcp-servers/project-management-automation
pip install -e .
```

Or install MCP dependency:
```bash
pip install mcp
```

## Structure

```
project-management-automation/
├── __init__.py
├── server.py              # Main MCP server (Phase 1 complete)
├── error_handler.py       # Error handling & logging (Phase 1 complete)
├── tools/                 # Tool implementations (Phase 2)
│   ├── __init__.py
│   ├── docs_health.py
│   ├── todo2_alignment.py
│   └── ...
├── resources/             # Resource handlers (Phase 3)
│   ├── __init__.py
│   ├── status.py
│   └── ...
└── pyproject.toml         # Package config (Phase 1 complete)
```

## Usage

### Running the Server

```bash
python -m project_management_automation.server
```

### MCP Configuration

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "project-management-automation": {
      "command": "python",
      "args": ["-m", "project_management_automation.server"]
    }
  }
}
```

## Tools (Phase 2)

Will expose:
- `check_documentation_health` - Analyze docs, find broken refs
- `analyze_todo2_alignment` - Check task alignment
- `detect_duplicate_tasks` - Find duplicate tasks
- `scan_dependency_security` - Security vulnerability scan
- `find_automation_opportunities` - Discover new automations
- `sync_todo_tasks` - Sync tasks across systems
- `review_pwa_config` - Validate PWA setup

## Resources (Phase 3)

Will expose:
- `automation://status` - Server status
- `automation://history` - Execution history
- `automation://list` - Available tools

## Error Handling

Centralized error handling via `error_handler.py`:
- Standard error codes
- Structured error responses
- Execution logging
- Graceful degradation

## Development

### Phase 1 Complete ✅
- Core server framework
- Package configuration
- Error handling infrastructure

### Next Steps (Phase 2)
- Implement tool wrappers
- Register tools with MCP server
- Test tool execution

## References

- [IntelligentAutomationBase Guide](../../docs/INTELLIGENT_AUTOMATION_GUIDE.md)
- [MCP Server Proposal](../../docs/MCP_PROJECT_MANAGEMENT_SERVER_PROPOSAL.md)
- [Implementation Plan](../../docs/MCP_SERVER_IMPLEMENTATION_PLAN.md)
