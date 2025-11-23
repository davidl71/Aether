# Intentional Duplicate Functionality - Design Documentation

## Overview

The automa MCP server intentionally provides some functionality through both **tools** and **resources**. This is not a bug - it's a design decision that follows MCP best practices and provides flexibility for different use cases.

---

## Tool vs Resource: Server Status

### Two Access Patterns for Server Status

#### 1. Tool: `server_status`

**Access:** `call_tool("server_status")`

**Purpose:** Quick, on-demand status checks

**Returns:**
```json
{
  "status": "operational",
  "version": "0.1.0",
  "tools_available": true,
  "project_root": "/path/to/project"
}
```

**Use Cases:**
- Quick health checks
- Simple status queries
- When you need basic information fast

**Code Location:** `server.py:256-272` (FastMCP) and `server.py:375-381` (stdio)

---

#### 2. Resource: `automation://status`

**Access:** `read_resource("automation://status")`

**Purpose:** Comprehensive server health and status information

**Returns:**
```json
{
  "server": "automa",
  "version": "0.1.0",
  "status": "operational",
  "mcp_available": true,
  "tools_available": true,
  "error_handling_available": true,
  "timestamp": "2025-11-23T17:20:00",
  "tools": {
    "total": 8,
    "high_priority": 4,
    "medium_priority": 3,
    "available": ["server_status", "check_documentation_health_tool", ...]
  }
}
```

**Use Cases:**
- Detailed health monitoring
- Resource discovery
- Cached status information
- Comprehensive server diagnostics

**Code Location:** `server.py:666-669` (FastMCP) and `server.py:724-725` (stdio)

---

## Why Both Exist

### 1. **MCP Best Practices**

According to MCP design principles:
- **Resources** are preferred for read-only data that can be cached
- **Tools** are for operations that may have side effects or need fresh data

Server status is read-only, so it fits the resource pattern. However, we also provide it as a tool for:
- Familiar function-call interface
- Clients that prefer tools over resources
- Quick checks without resource discovery overhead

### 2. **Different Data Granularity**

- **Tool** provides minimal, essential information (4 fields)
- **Resource** provides comprehensive information (10+ fields including tool list)

This allows clients to choose based on their needs:
- Need quick check? Use tool
- Need detailed info? Use resource

### 3. **Access Pattern Flexibility**

Different MCP clients may prefer different patterns:

**Tool Pattern:**
```python
# Function-call style
result = mcp_client.call_tool("server_status")
```

**Resource Pattern:**
```python
# URI-based style
result = mcp_client.read_resource("automation://status")
```

### 4. **Caching Benefits**

Resources can be cached by MCP clients, tools cannot:
- **Resource:** Client can cache `automation://status` and refresh periodically
- **Tool:** Always executes fresh query

For monitoring/health checks, caching is beneficial.

---

## Comparison Table

| Aspect | Tool (`server_status`) | Resource (`automation://status`) |
|--------|------------------------|----------------------------------|
| **Access Method** | `call_tool("server_status")` | `read_resource("automation://status")` |
| **Data Fields** | 4 (status, version, tools_available, project_root) | 10+ (includes health, tool list, timestamps) |
| **Data Detail** | Basic | Comprehensive |
| **Caching** | ❌ Not cacheable | ✅ Can be cached |
| **Discovery** | Listed in `list_tools()` | Listed in `list_resources()` |
| **Use Case** | Quick status check | Detailed health monitoring |
| **Performance** | Fast (minimal data) | Slightly slower (more data) |
| **Best For** | On-demand queries | Periodic health checks |

---

## When to Use Which

### Use Tool (`server_status`) When:
- ✅ You need a quick status check
- ✅ You only need basic information (operational/not operational)
- ✅ You're making a one-off query
- ✅ You prefer function-call style API

### Use Resource (`automation://status`) When:
- ✅ You need comprehensive server information
- ✅ You want to cache status information
- ✅ You're building monitoring/health dashboards
- ✅ You need the full tool list and health metrics
- ✅ You prefer URI-based resource access

---

## Implementation Details

### Tool Implementation

**FastMCP:**
```python
@mcp.tool()
def server_status() -> str:
    """Get the current status of the project management automation server."""
    return json.dumps({
        "status": "operational",
        "version": "0.1.0",
        "tools_available": TOOLS_AVAILABLE,
        "project_root": str(project_root),
    }, indent=2)
```

**Stdio Server:**
```python
Tool(
    name="server_status",
    description="Get the current status of the project management automation server.",
    inputSchema={"type": "object", "properties": {}},
)
```

### Resource Implementation

**FastMCP:**
```python
@mcp.resource("automation://status")
def get_automation_status() -> str:
    """Get automation server status and health information."""
    return get_status_resource()  # Calls resources/status.py
```

**Stdio Server:**
```python
@stdio_server_instance.read_resource()
async def read_resource(uri: str) -> str:
    if uri == "automation://status":
        return get_status_resource()
```

---

## Related Resources

- **`automation://history`** - Tool execution history (resource only)
- **`automation://tools`** - Tool list with descriptions (resource only)

These are resources-only because they're read-only data that benefits from caching.

---

## Future Considerations

### Potential Consolidation

If MCP clients consistently prefer one pattern over the other, we could:
1. **Keep both** (current) - Maximum flexibility
2. **Deprecate tool** - If resources become standard
3. **Deprecate resource** - If tools are preferred (unlikely)

### Monitoring

Track usage patterns:
- Which is used more: tool or resource?
- Are clients caching the resource?
- Do users need both, or just one?

---

## Conclusion

The duplicate functionality between `server_status` (tool) and `automation://status` (resource) is **intentional design**, not a bug. It provides:

1. ✅ Flexibility for different client preferences
2. ✅ Different data granularity for different use cases
3. ✅ Caching benefits for resources
4. ✅ Familiar patterns for both tool and resource users

**Status:** ✅ Intentional design - no changes needed

---

**Last Updated:** 2025-11-23
