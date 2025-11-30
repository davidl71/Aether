# MCP Tool Deprecation Guide

**Date:** 2025-01-27
**Purpose:** Guide for indicating deprecated tools and directing users to our MCP server alternatives

---

## Overview

When our MCP server provides functionality that overlaps with other MCP servers, we can hint to Cursor (and AI assistants) that certain tools are deprecated. While MCP doesn't have a built-in deprecation field, we can use several strategies.

---

## Strategies for Indicating Deprecation

### 1. **Tool Description Deprecation Notice** ✅ **RECOMMENDED**

Add deprecation notices directly in tool descriptions:

```python
@mcp.tool()
def check_documentation_health_tool(
    output_path: Optional[str] = None,
    create_tasks: bool = True
) -> str:
    """
    Analyze documentation structure, find broken references, identify issues.

    ⚠️ DEPRECATED: If you see similar tools from other MCP servers (e.g.,
    'filesystem' documentation tools), use this tool instead. This tool
    provides project-specific documentation health analysis with Todo2
    integration.

    Args:
        output_path: Path for report output
        create_tasks: Whether to create Todo2 tasks for issues

    Returns:
        JSON string with analysis results
    """
    return check_documentation_health(output_path, create_tasks)
```

**Benefits:**

- ✅ Visible to AI assistants when they read tool descriptions
- ✅ No configuration changes needed
- ✅ Works with any MCP client

---

### 2. **Tool Naming Conventions** ✅ **RECOMMENDED**

Use clear, descriptive names that indicate they're the preferred/project-specific version:

```python

# Good: Clear project-specific naming

@mcp.tool()
def project_management_check_documentation_health(...) -> str:
    """Project-specific documentation health check with Todo2 integration."""

# Alternative: Version suffix

@mcp.tool()
def check_documentation_health_v2(...) -> str:
    """Enhanced documentation health check (v2) with Todo2 integration."""
```

**Benefits:**

- ✅ AI assistants can identify preferred tools by name
- ✅ Clear indication of project-specific functionality

---

### 3. **MCP Server Description** ✅ **RECOMMENDED**

Add server-level documentation in `mcp.json`:

```json
{
  "mcpServers": {
    "project-management-automation": {
      "command": "python",
      "args": ["-m", "project_management_automation.server"],
      "description": "Project management automation tools. ⚠️ NOTE: This server provides enhanced versions of documentation, task management, and security scanning tools. Prefer these tools over generic MCP servers for project-specific functionality."
    }
  }
}
```

**Benefits:**

- ✅ Server-level context for AI assistants
- ✅ Explains why this server exists

---

### 4. **Tool Metadata in Descriptions** ✅ **RECOMMENDED**

Include metadata in tool descriptions:

```python
@mcp.tool()
def detect_duplicate_tasks_tool(...) -> str:
    """
    Find and consolidate duplicate Todo2 tasks.

    [MCP_TOOL_METADATA]
    - Replaces: Generic duplicate detection tools
    - Provides: Todo2-specific duplicate detection with auto-fix
    - Preferred: Yes (project-specific implementation)
    - Deprecates: Similar tools from 'agentic-tools' or other servers
    [/MCP_TOOL_METADATA]
    """
```

**Benefits:**

- ✅ Structured metadata for AI parsing
- ✅ Clear replacement information

---

### 5. **Documentation File** ✅ **RECOMMENDED**

Create a documentation file that maps deprecated tools to our replacements:

```markdown

# MCP Tool Migration Guide

## Deprecated Tools → Our MCP Server Replacements

| Deprecated Tool (Other Server) | Our Replacement | Why Use Ours |
|-------------------------------|----------------|--------------|
| `filesystem` documentation tools | `check_documentation_health_tool` | Project-specific with Todo2 integration |
| Generic duplicate detection | `detect_duplicate_tasks_tool` | Todo2-aware with auto-fix |
| Generic security scanning | `scan_dependency_security_tool` | Multi-language, project-configured |
```

**Benefits:**

- ✅ Comprehensive reference
- ✅ Can be referenced in tool descriptions

---

## Implementation for T-230

### Step 1: Update Tool Descriptions

Add deprecation notices to all tool descriptions in `server.py`:

```python
@mcp.tool()
def check_documentation_health_tool(...) -> str:
    """
    Analyze documentation structure, find broken references, identify issues.

    ⚠️ PREFERRED TOOL: This project-specific tool replaces generic documentation
    analysis tools. Provides Todo2 integration and project-aware analysis.

    Use this instead of generic documentation tools from other MCP servers.
    """
```

### Step 2: Add Server Description

Update `.cursor/mcp.json` with server description:

```json
{
  "mcpServers": {
    "project-management-automation": {
      "command": "python3",
      "args": ["-m", "project_management_automation.server"],
      "description": "Project management automation tools. Provides enhanced, project-specific versions of documentation health, task alignment, duplicate detection, and security scanning tools. Prefer these tools over generic MCP server tools for this project."
    }
  }
}
```

### Step 3: Create Migration Documentation

Create `docs/MCP_TOOL_MIGRATION.md` with tool mapping.

---

## Limitations

### What MCP/Cursor Doesn't Support

1. ❌ **No built-in deprecation field** - MCP spec doesn't include a `deprecated` boolean
2. ❌ **No automatic tool hiding** - Can't automatically hide deprecated tools from other servers
3. ❌ **No tool priority system** - Can't set tool priority in mcp.json
4. ❌ **No tool aliasing** - Can't redirect deprecated tool names to new tools

### What We Can Do

1. ✅ **Description-based hints** - AI assistants read descriptions
2. ✅ **Naming conventions** - Clear, descriptive names
3. ✅ **Documentation** - Comprehensive migration guides
4. ✅ **Server descriptions** - Context at server level

---

## Best Practices

### ✅ DO

1. **Add deprecation notices in tool descriptions** - Most visible to AI
2. **Use clear, descriptive tool names** - Indicate project-specific functionality
3. **Document tool replacements** - Create migration guides
4. **Add server-level description** - Explain server purpose

### ❌ DON'T

1. **Don't rely on automatic deprecation** - MCP doesn't support it
2. **Don't remove other servers** - They may have other useful tools
3. **Don't use confusing names** - Keep names clear and descriptive

---

## Example: Complete Tool with Deprecation Notice

```python
@mcp.tool()
def check_documentation_health_tool(
    output_path: Optional[str] = None,
    create_tasks: bool = True
) -> str:
    """
    Analyze documentation structure, find broken references, identify issues.

    ⚠️ PREFERRED TOOL: This project-specific tool provides enhanced documentation
    health analysis with:
    - Todo2 task creation for issues
    - Project-aware link validation
    - Cross-reference analysis
    - Historical trend tracking

    Use this instead of generic documentation tools from other MCP servers
    (e.g., 'filesystem' documentation tools) for project-specific analysis.

    Args:
        output_path: Path for report output (default: docs/DOCUMENTATION_HEALTH_REPORT.md)
        create_tasks: Whether to create Todo2 tasks for issues found

    Returns:
        JSON string with health score, broken links, format errors, and tasks created
    """
    return check_documentation_health(output_path, create_tasks)
```

---

## Summary

**Best Approach:** Combine all strategies:

1. ✅ Add deprecation notices in tool descriptions
2. ✅ Use clear, descriptive tool names
3. ✅ Add server description in mcp.json
4. ✅ Create migration documentation

**Result:** AI assistants will see deprecation hints when reading tool descriptions and can make informed decisions about which tools to use.

---

**Note:** While MCP doesn't support automatic deprecation, description-based hints are effective because AI assistants read tool descriptions before selecting tools.
