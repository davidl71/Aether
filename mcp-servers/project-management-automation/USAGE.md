# Project Management Automation MCP Server - Usage Guide

**Version:** 0.1.0
**Status:** Production Ready

---

## Overview

The Project Management Automation MCP Server exposes project-specific automation tools for AI assistants. These tools provide enhanced functionality compared to generic MCP server tools, with Todo2 integration, project-aware analysis, and historical tracking.

---

## Installation

### Prerequisites

- Python 3.9+
- MCP Python SDK: `pip install mcp`
- Project dependencies (IntelligentAutomationBase, etc.)

### Setup

1. **Install MCP package:**
   ```bash
   pip install mcp
   ```

2. **Verify server is configured:**
   Check `.cursor/mcp.json` contains `project-management-automation` entry

3. **Restart Cursor:**
   Completely restart Cursor IDE to discover the MCP server

---

## Available Tools

### System Tools

#### `server_status`
Get the current status of the project management automation server.

**Returns:**
- Server version
- Tools available status
- Project root path
- Operational status

**Example:**
```json
{
  "status": "operational",
  "version": "0.1.0",
  "tools_available": true,
  "project_root": "/path/to/project"
}
```

---

### Documentation Tools

#### `check_documentation_health_tool`
⚠️ **PREFERRED TOOL** - Analyze documentation structure, find broken references, identify issues.

**Parameters:**
- `output_path` (Optional[str]): Path for report output (default: `docs/DOCUMENTATION_HEALTH_REPORT.md`)
- `create_tasks` (bool): Whether to create Todo2 tasks for issues (default: `true`)

**Returns:**
- Health score (0-100)
- Link validation metrics
- Format errors count
- Tasks created count
- Report path

**Example Usage:**
```
"Check documentation health and create tasks for issues"
"Analyze docs and write report to docs/health.md"
```

---

### Task Management Tools

#### `analyze_todo2_alignment_tool`
⚠️ **PREFERRED TOOL** - Analyze task alignment with project goals, find misaligned tasks.

**Parameters:**
- `create_followup_tasks` (bool): Create Todo2 tasks for misaligned tasks (default: `true`)
- `output_path` (Optional[str]): Path for report output

**Returns:**
- Total tasks analyzed
- Misaligned count
- Average alignment score
- Tasks created count
- Report path

**Example Usage:**
```
"Analyze Todo2 task alignment with investment strategy"
"Check if tasks align with project goals"
```

#### `detect_duplicate_tasks_tool`
⚠️ **PREFERRED TOOL** - Find and consolidate duplicate Todo2 tasks.

**Parameters:**
- `similarity_threshold` (float): Similarity threshold 0.0-1.0 (default: `0.85`)
- `auto_fix` (bool): Automatically fix duplicates (default: `false`)
- `output_path` (Optional[str]): Path for report output

**Returns:**
- Duplicate counts by type
- Total duplicates found
- Auto-fix status
- Report path

**Example Usage:**
```
"Find duplicate Todo2 tasks with 0.9 similarity threshold"
"Detect and auto-fix duplicate tasks"
```

#### `sync_todo_tasks_tool`
Synchronize tasks between shared TODO table and Todo2.

**Parameters:**
- `dry_run` (bool): Simulate sync without making changes (default: `false`)
- `output_path` (Optional[str]): Path for report output

**Returns:**
- Matches found
- Conflicts detected
- New tasks created
- Updates performed
- Report path

**Example Usage:**
```
"Sync todos between shared table and Todo2 (dry run)"
"Synchronize Todo2 with shared TODO table"
```

---

### Security Tools

#### `scan_dependency_security_tool`
⚠️ **PREFERRED TOOL** - Scan project dependencies for security vulnerabilities.

**Parameters:**
- `languages` (Optional[List[str]]): Languages to scan - `["python", "rust", "npm"]` (default: all)
- `config_path` (Optional[str]): Path to dependency security config file

**Returns:**
- Total vulnerabilities
- Vulnerabilities by severity
- Vulnerabilities by language
- Critical vulnerabilities count
- Report path

**Example Usage:**
```
"Scan Python and Rust dependencies for security issues"
"Check npm packages for vulnerabilities"
```

---

### Automation Tools

#### `find_automation_opportunities_tool`
Discover new automation opportunities in the codebase.

**Parameters:**
- `min_value_score` (float): Minimum value score threshold 0.0-1.0 (default: `0.7`)
- `output_path` (Optional[str]): Path for report output

**Returns:**
- Total opportunities found
- Filtered opportunities (by score)
- High/medium/low priority counts
- Top opportunities list
- Report path

**Example Usage:**
```
"Find automation opportunities with value score >= 0.8"
"Discover new automation opportunities"
```

---

### Review Tools

#### `review_pwa_config_tool`
Review PWA configuration and generate improvement recommendations.

**Parameters:**
- `output_path` (Optional[str]): Path for analysis output
- `config_path` (Optional[str]): Path to PWA review config file

**Returns:**
- Components count
- Hooks count
- API integrations count
- PWA features detected
- Missing features
- Goal-aligned tasks
- Report path

**Example Usage:**
```
"Review PWA configuration and suggest improvements"
"Analyze PWA setup and generate recommendations"
```

---

## Resources

### `automation://status`
Get automation server status and health information.

**Returns:**
- Server status
- Tools available
- Error handling status
- Tool counts by priority
- Timestamp

### `automation://history`
Get automation tool execution history.

**Returns:**
- Execution history (last 50 runs)
- Total executions
- Per-automation status
- Timestamps

### `automation://tools`
Get list of available automation tools.

**Returns:**
- Complete tool list with descriptions
- Tool categories
- Priority breakdown
- Total tool count

---

## Usage Examples

### Example 1: Check Documentation Health

**AI Assistant Prompt:**
```
"Check the health of our documentation and create Todo2 tasks for any issues found"
```

**Tool Called:**
- `check_documentation_health_tool(output_path=None, create_tasks=True)`

**Result:**
- Health report generated
- Todo2 tasks created for issues
- JSON response with metrics

---

### Example 2: Find Duplicate Tasks

**AI Assistant Prompt:**
```
"Find duplicate Todo2 tasks with 85% similarity and show me the report"
```

**Tool Called:**
- `detect_duplicate_tasks_tool(similarity_threshold=0.85, auto_fix=False, output_path=None)`

**Result:**
- Duplicate detection report
- List of duplicate task pairs
- Recommendations for consolidation

---

### Example 3: Security Scan

**AI Assistant Prompt:**
```
"Scan all dependencies for security vulnerabilities and prioritize critical issues"
```

**Tool Called:**
- `scan_dependency_security_tool(languages=None, config_path=None)`

**Result:**
- Multi-language security report
- Vulnerabilities by severity
- Critical issues highlighted
- Todo2 tasks for high-severity issues

---

## Error Handling

All tools use centralized error handling:

- **Success Responses:** Structured JSON with `success: true` and `data` object
- **Error Responses:** Structured JSON with `success: false` and `error` object
- **Error Codes:** Standard error codes (INVALID_INPUT, AUTOMATION_ERROR, etc.)
- **Logging:** All executions logged with duration and status

---

## Best Practices

### When to Use Our Tools

✅ **Use our tools when:**
- You need project-specific analysis
- You want Todo2 integration
- You need historical tracking
- You want project-configured behavior

### When to Use Other Servers

✅ **Use other servers for:**
- General file operations (`filesystem`)
- Git operations (`git`)
- Task CRUD operations (`agentic-tools`)
- Code security scanning (`semgrep`)
- Documentation lookup (`context7`)

---

## Troubleshooting

### Server Not Appearing

1. **Restart Cursor completely** (not just reload)
2. **Check Python path** - Ensure `python3` is in PATH
3. **Check file permissions** - Server file must be readable
4. **Check MCP package** - Run `pip install mcp`

### Tools Not Working

1. **Check dependencies** - Ensure all automation scripts are available
2. **Check project root** - Server needs access to project root
3. **Check error logs** - Look for Python import errors
4. **Check MCP logs** - Look in Cursor's MCP server logs

### Import Errors

1. **Check Python path** - Server adds project root to path
2. **Check module structure** - Ensure all `__init__.py` files exist
3. **Check relative imports** - Tools use relative imports from error_handler

---

## Configuration

### MCP Configuration (`.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "project-management-automation": {
      "command": "python3",
      "args": ["/path/to/server.py"],
      "description": "Project management automation tools..."
    }
  }
}
```

### Environment Variables

No environment variables required. All configuration comes from:
- Tool parameters
- Project configuration files
- Default values

---

## Related Documentation

- `README.md` - Server overview and setup
- `docs/MCP_TOOL_DEPRECATION_GUIDE.md` - Deprecation strategies
- `docs/MCP_TOOL_MIGRATION.md` - Tool migration map
- `docs/MCP_SERVER_IMPLEMENTATION_PLAN.md` - Implementation details

---

**Last Updated:** 2025-01-27
