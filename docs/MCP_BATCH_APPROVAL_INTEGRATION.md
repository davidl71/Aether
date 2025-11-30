# MCP Batch Approval Integration

**Status:** ✅ Fully Integrated

**Purpose:** Batch approval functionality is now available both as a standalone MCP tool and integrated into nightly automation.

---

## MCP Tools Available

### 1. `batch_approve_tasks_tool` (Standalone)

**Purpose:** Batch approve TODO2 tasks via MCP interface

**Usage via Cursor Chat:**

```
"Batch approve all Review tasks that don't need clarification"
"Approve research tasks in Review status"
"Preview what tasks would be approved (dry run)"
```

**Parameters:**

- `status`: Current status to filter (default: "Review")
- `new_status`: New status after approval (default: "Todo")
- `clarification_none`: Only approve tasks with no clarification (default: true)
- `filter_tag`: Filter by tag (e.g., "research")
- `task_ids`: List of specific task IDs to approve
- `dry_run`: Preview mode (default: false)

**Returns:**

```json
{
  "success": true,
  "approved_count": 31,
  "task_ids": ["T-156", "T-157", ...],
  "status_from": "Review",
  "status_to": "Todo",
  "dry_run": false,
  "output": "..."
}
```

**File:** `mcp-servers/project-management-automation/tools/batch_task_approval.py`

---

### 2. `run_nightly_task_automation_tool` (Integrated)

**Purpose:** Nightly automation with automatic batch approval

**What It Does:**

1. **Automatically runs batch approval** before task assignment
2. Approves Review tasks with no clarification needed
3. Assigns background tasks to agents
4. Moves interactive tasks to Review

**Usage via Cursor Chat:**

```
"Run nightly automation"
"Process background tasks with batch approval"
"Preview nightly automation (dry run)"
```

**Returns:**

```json
{
  "summary": {
    "background_tasks_found": 22,
    "interactive_tasks_found": 45,
    "tasks_assigned": 10,
    "tasks_moved_to_review": 10,
    "tasks_batch_approved": 5,
    "hosts_used": 2
  },
  ...
}
```

**File:** `mcp-servers/project-management-automation/tools/nightly_task_automation.py`

**Integration:** Automatically calls batch approval internally

---

## Integration Points

### 1. MCP Server Registration

**File:** `mcp-servers/project-management-automation/server.py`

Both tools are registered as MCP tools:

- `batch_approve_tasks_tool` - Standalone batch approval
- `run_nightly_task_automation_tool` - Integrated automation

### 2. Nightly Automation Integration

**File:** `mcp-servers/project-management-automation/tools/nightly_task_automation.py`

The nightly automation:

- Calls `scripts/batch_update_todos.py` internally
- Runs batch approval before task assignment
- Tracks `tasks_batch_approved` in results
- Non-blocking (errors don't fail automation)

### 3. GitHub Actions Integration

**File:** `.github/workflows/nightly-task-automation.yml`

The workflow:

- Runs batch approval as a separate step
- Then runs nightly automation
- Reports batch approval results in summary

---

## Usage Examples

### Via Cursor Chat (MCP)

**Standalone Batch Approval:**

```
"Batch approve all Review tasks with no clarification needed"
"Approve research tasks in Review status"
"Preview what would be approved (dry run)"
```

**Nightly Automation (Includes Batch Approval):**

```
"Run nightly automation"
"Process background tasks"
"Preview nightly automation"
```

### Via Command Line

**Standalone:**

```bash
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

**Via MCP Tool (Python):**

```python
from mcp_servers.project_management_automation.tools.batch_task_approval import batch_approve_tasks

result = batch_approve_tasks(
    status="Review",
    clarification_none=True,
    dry_run=False
)
```

---

## Benefits

### ✅ Available via MCP

- Can be called from Cursor chat
- Integrated into AI assistant workflows
- Easy to use interactively

### ✅ Integrated into Automation

- Nightly automation includes batch approval
- GitHub Actions workflow includes batch approval
- Automatic processing of research tasks

### ✅ Flexible Usage

- Standalone tool for manual approval
- Integrated into automation workflows
- Command-line script for scripting

---

## Documentation

- **MCP Tools:** `mcp-servers/project-management-automation/TOOLS_STATUS.md`
- **Batch Script:** `docs/BATCH_TODO_UPDATE_SCRIPT.md`
- **Nightly Automation:** `docs/NIGHTLY_TASK_AUTOMATION.md`
- **Integration:** `docs/NIGHTLY_AUTOMATION_BATCH_APPROVAL.md`

---

## Testing

### Test MCP Tool Import

```bash
python3 -c "import sys; sys.path.insert(0, 'mcp-servers/project-management-automation'); from tools.batch_task_approval import batch_approve_tasks; print('✅ Import successful')"
```

### Test Batch Approval (Dry Run)

```bash
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --dry-run
```

### Test via MCP (Cursor Chat)

```
"Preview batch approval of Review tasks (dry run)"
```

---

## Summary

✅ **MCP Integration Complete:**

- Standalone `batch_approve_tasks_tool` available via MCP
- Integrated into `run_nightly_task_automation_tool`
- Available in GitHub Actions workflow
- Documented in TOOLS_STATUS.md

✅ **Usage Options:**

- Via Cursor chat (MCP tools)
- Via command line (batch script)
- Via automation (nightly automation)
- Via CI/CD (GitHub Actions)

✅ **Benefits:**

- Automated approval of research tasks
- Reduced manual review workload
- Better task flow and automation
- Flexible usage patterns
