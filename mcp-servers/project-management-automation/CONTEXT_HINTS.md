# Context Summarization & Condensation Hints

## Overview

All MCP tools now include condensed hints at the start of their docstrings to help AI assistants quickly understand tool outputs and compress context efficiently.

---

## Hint Format

Each tool description starts with a `[HINT: ...]` line that provides:
1. **Tool purpose** - What the tool does
2. **Key outputs** - What data the tool returns (for context compression)

**Format:**
```
[HINT: Brief description. Returns key output fields.]
```

---

## Available Hints

### Documentation
- **`check_documentation_health_tool`**
  - `[HINT: Docs health check. Returns score 0-100, broken links count, tasks created.]`

### Task Management
- **`analyze_todo2_alignment_tool`**
  - `[HINT: Task alignment analysis. Returns misaligned count, avg score, follow-up tasks.]`
- **`detect_duplicate_tasks_tool`**
  - `[HINT: Duplicate detection. Returns duplicate count, groups, recommendations.]`
- **`sync_todo_tasks_tool`**
  - `[HINT: Task sync. Returns matches found, conflicts, new tasks created.]`

### Security
- **`scan_dependency_security_tool`**
  - `[HINT: Security scan. Returns vuln count by severity, language breakdown, remediation.]`

### Automation
- **`find_automation_opportunities_tool`**
  - `[HINT: Automation discovery. Returns opportunity count, value scores, recommendations.]`

### PWA
- **`review_pwa_config_tool`**
  - `[HINT: PWA review. Returns config status, missing features, recommendations.]`

### System
- **`server_status`**
  - `[HINT: Server status. Returns operational status, version, tools available.]`

---

## Benefits for Context Compression

### 1. **Quick Understanding**
Hints allow AI to quickly understand tool purpose without reading full descriptions.

### 2. **Output Awareness**
Hints specify what data tools return, helping AI:
- Know what to extract from results
- Compress tool outputs effectively
- Summarize multiple tool results

### 3. **Token Efficiency**
Hints are concise (20-40 chars) but informative, reducing token usage while maintaining clarity.

### 4. **Summarization Guidance**
Hints guide AI on what to prioritize when summarizing:
- Key metrics (counts, scores)
- Important outputs (tasks created, vulnerabilities found)
- Actionable data (recommendations, conflicts)

---

## Usage in Context Compression

When AI needs to compress context:

1. **Read hints first** - Understand tool outputs quickly
2. **Extract key metrics** - Focus on counts, scores, status
3. **Prioritize actionable data** - Recommendations, tasks, conflicts
4. **Summarize efficiently** - Use hint structure to guide compression

**Example Compression:**
```
Before: Full tool description + full result JSON
After: [HINT: Docs health check. Returns score 0-100...] → Score: 85, 3 broken links, 2 tasks created
```

---

## Implementation

Hints are embedded in tool docstrings using FastMCP's `@mcp.tool()` decorator. The hint appears first in the docstring, followed by the full description.

**Pattern:**
```python
@mcp.tool()
def tool_name(...) -> str:
    """
    [HINT: Brief description. Returns key outputs.]

    Full description here...
    """
```

---

## Future Enhancements

Potential additions:
- **Output schema hints** - JSON structure preview
- **Compression ratios** - Suggested summarization levels
- **Priority flags** - Mark critical vs. optional outputs
- **Example outputs** - Sample condensed results

---

**Status:** ✅ All tools include context compression hints
