# Project Automation MCP Server - Tools Status

**Last Updated:** 2025-11-23
**Status:** All Tools Fixed and Ready

---

## Tools Overview

The Project Automation MCP Server exports **14 tools** for project management automation:

### ✅ All Tools Fixed

All tools have been updated to fix import scoping issues. Error handler imports are now at module level to avoid "cannot access local variable" errors.

---

## Available Tools

### 1. `check_documentation_health_tool` ✅
**Status:** Fixed and Working
**Purpose:** Analyze documentation structure, find broken references, identify issues

**Parameters:**
- `output_path` (Optional[str]): Path for report (default: `docs/DOCUMENTATION_HEALTH_REPORT.md`)
- `create_tasks` (bool): Create Todo2 tasks for issues (default: `true`)

**Returns:**
- Health score (0-100)
- Link validation metrics
- Format errors count
- Tasks created count
- Report path

**File:** `tools/docs_health.py`
**Fix Applied:** ✅ Error handler imports moved to module level

---

### 2. `analyze_todo2_alignment_tool` ✅
**Status:** Fixed and Ready
**Purpose:** Analyze task alignment with project goals, find misaligned tasks

**Parameters:**
- `create_followup_tasks` (bool): Create Todo2 tasks for misaligned tasks (default: `true`)
- `output_path` (Optional[str]): Path for report

**Returns:**
- Total tasks analyzed
- Misaligned count
- Average alignment score
- Tasks created count
- Report path

**File:** `tools/todo2_alignment.py`
**Fix Applied:** ✅ Error handler imports moved to module level

---

### 3. `detect_duplicate_tasks_tool` ✅
**Status:** Fixed and Ready
**Purpose:** Find and consolidate duplicate Todo2 tasks

**Parameters:**
- `similarity_threshold` (float): 0.0-1.0 (default: `0.85`)
- `auto_fix` (bool): Automatically fix duplicates (default: `false`)
- `output_path` (Optional[str]): Path for report

**Returns:**
- Duplicate counts by type
- Total duplicates found
- Auto-fix status
- Report path

**File:** `tools/duplicate_detection.py`
**Fix Applied:** ✅ Error handler imports moved to module level

---

### 4. `scan_dependency_security_tool` ✅
**Status:** Fixed and Ready
**Purpose:** Scan project dependencies for security vulnerabilities

**Parameters:**
- `languages` (Optional[List[str]]): `["python", "rust", "npm"]` (default: all)
- `config_path` (Optional[str]): Path to security config file

**Returns:**
- Total vulnerabilities
- Vulnerabilities by severity
- Vulnerabilities by language
- Critical vulnerabilities count
- Report path

**File:** `tools/dependency_security.py`
**Fix Applied:** ✅ Error handler imports moved to module level

---

### 5. `find_automation_opportunities_tool` ✅
**Status:** Fixed and Ready
**Purpose:** Discover new automation opportunities in the codebase

**Parameters:**
- `min_value_score` (float): 0.0-1.0 (default: `0.7`)
- `output_path` (Optional[str]): Path for report

**Returns:**
- Total opportunities found
- Filtered opportunities (by score)
- High/medium/low priority counts
- Top opportunities list
- Report path

**File:** `tools/automation_opportunities.py`
**Fix Applied:** ✅ Error handler imports moved to module level

---

### 6. `sync_todo_tasks_tool` ✅
**Status:** Fixed and Ready
**Purpose:** Synchronize tasks between shared TODO table and Todo2

**Parameters:**
- `dry_run` (bool): Simulate sync without changes (default: `false`)
- `output_path` (Optional[str]): Path for report

**Returns:**
- Matches found
- Conflicts detected
- New tasks created
- Updates performed
- Report path

**File:** `tools/todo_sync.py`
**Fix Applied:** ✅ Error handler imports moved to module level

---

### 7. `review_pwa_config_tool` ✅
**Status:** Fixed and Ready
**Purpose:** Review PWA configuration and generate improvement recommendations

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

**File:** `tools/pwa_review.py`
**Fix Applied:** ✅ Error handler imports moved to module level

---

### 8. `validate_ci_cd_workflow_tool` ✅
**Status:** ✅ Implemented and Ready
**Purpose:** Validate CI/CD workflows and runner configurations

**Parameters:**
- `workflow_path` (Optional[str]): Path to workflow file (default: `.github/workflows/parallel-agents-ci.yml`)
- `check_runners` (bool): Validate runner configurations (default: `true`)
- `output_path` (Optional[str]): Path for validation report

**Returns:**
- Workflow validation status
- Runner configuration status
- Job dependency validation
- Matrix build validation
- Trigger validation
- Artifact validation
- Overall status
- Issues list
- Report path

**File:** `tools/ci_cd_validation.py`
**Fix Applied:** ✅ Error handler imports at module level

---

### 9. `batch_approve_tasks_tool` ✅
**Status:** ✅ Implemented and Ready
**Purpose:** Batch approve TODO2 tasks that don't need clarification, moving them from Review to Todo status

**Parameters:**
- `status` (str): Current status to filter (default: `"Review"`)
- `new_status` (str): New status after approval (default: `"Todo"`)
- `clarification_none` (bool): Only approve tasks with no clarification needed (default: `true`)
- `filter_tag` (Optional[str]): Filter by tag (e.g., "research") (optional)
- `task_ids` (Optional[List[str]]): List of specific task IDs to approve (optional)
- `dry_run` (bool): Preview mode without executing (default: `false`)

**Returns:**
- `success` (bool): Whether approval succeeded
- `approved_count` (int): Number of tasks approved
- `task_ids` (List[str]): List of approved task IDs
- `status_from` (str): Original status
- `status_to` (str): New status
- `dry_run` (bool): Whether this was a dry run
- `output` (str): Script output

**File:** `tools/batch_task_approval.py`
**Integration:** Uses `scripts/batch_update_todos.py` for batch operations

**Usage:**
- Use before nightly automation to clear Review queue
- Approve research tasks that don't need clarification
- Preview what would be approved with `dry_run=true`

---

### 10. `run_nightly_task_automation_tool` ✅
**Status:** ✅ Implemented and Ready
**Purpose:** Automatically execute background-capable TODO2 tasks in parallel across multiple hosts. Includes automatic batch approval step.

**Parameters:**
- `max_tasks_per_host` (int): Maximum tasks to assign per host (default: `5`)
- `max_parallel_tasks` (int): Maximum total parallel tasks (default: `10`)
- `priority_filter` (Optional[str]): Filter by priority - 'high', 'medium', or 'low' (optional)
- `tag_filter` (Optional[List[str]]): Filter by tags - list of tag strings (optional)
- `dry_run` (bool): Preview mode without executing (default: `false`)

**Returns:**
- `timestamp` (str): Execution timestamp
- `dry_run` (bool): Whether this was a dry run
- `summary` (dict): Summary with:
  - `background_tasks_found` (int): Number of background-capable tasks
  - `interactive_tasks_found` (int): Number of interactive tasks
  - `tasks_assigned` (int): Number of tasks assigned to hosts
  - `tasks_moved_to_review` (int): Number of tasks moved to Review
  - `tasks_batch_approved` (int): Number of tasks batch approved
  - `hosts_used` (int): Number of hosts used
- `assigned_tasks` (List[dict]): List of assigned tasks with host info
- `moved_to_review` (List[str]): List of task IDs moved to Review
- `background_tasks_remaining` (int): Number of background tasks not yet assigned

**File:** `tools/nightly_task_automation.py`
**Integration:** Includes automatic batch approval using `batch_approve_tasks_tool`

**Returns:**
- Summary (background tasks found, interactive tasks found, tasks assigned, tasks moved to review, hosts used)
- Assigned tasks (task_id, task_name, host, hostname)
- Moved to review (list of task IDs)
- Background tasks remaining
- Timestamp
- Dry run status

**Features:**
- Automatically identifies background-capable tasks (MCP extensions, research, implementation, testing, documentation, configuration)
- Moves interactive tasks requiring user input to Review status
- Distributes tasks across multiple hosts using round-robin assignment
- Safe operation with dry run mode and automatic backups
- Comprehensive audit trail with task comments

**File:** `tools/nightly_task_automation.py`
**Documentation:** `docs/NIGHTLY_TASK_AUTOMATION.md`
**Integration:** Includes automatic working copy health check before execution

---

### 11. `check_working_copy_health_tool` ✅
**Status:** ✅ Implemented and Ready
**Purpose:** Check git working copy status across all agents and runners

**Parameters:**
- `agent_name` (Optional[str]): Specific agent to check (local/ubuntu/macos)
- `check_remote` (bool): Whether to check remote agents (default: true)

**Returns:**
- `summary` (dict): Summary with total agents, ok/warning/error counts
- `agents` (dict): Detailed status for each agent:
  - `status` (str): ok/warning/error
  - `has_uncommitted_changes` (bool): Whether agent has uncommitted changes
  - `uncommitted_files` (List[str]): List of uncommitted files
  - `branch` (str): Current branch
  - `latest_commit` (str): Latest commit message
  - `behind_remote` (int): Commits behind origin/main
  - `ahead_remote` (int): Commits ahead of origin/main
  - `in_sync` (bool): Whether agent is in sync with remote
- `recommendations` (List[str]): Actionable recommendations

**File:** `tools/working_copy_health.py`
**Documentation:** `docs/WORKING_COPY_HEALTH_AUTOMATION.md`
**Integration:** Used by nightly automation before task execution

**Usage:**
- Check all agents before starting work
- Verify agents are in sync
- Identify uncommitted changes across agents

---

### 12. `resolve_task_clarification_tool` ✅
**Status:** ✅ Implemented and Ready
**Purpose:** Resolve a single task clarification by updating task description with decision

**Parameters:**
- `task_id` (str): Task ID to resolve (e.g., "T-76")
- `clarification` (str): Clarification text/question
- `decision` (str): Your decision/answer to the clarification
- `move_to_todo` (bool): Whether to move task to Todo status (default: true)
- `dry_run` (bool): Preview mode without making changes (default: false)

**Returns:**
- `status` (str): success/error
- `task_id` (str): Task ID that was resolved
- `moved_to_todo` (bool): Whether task was moved to Todo
- `output` (str): Command output

**File:** `tools/task_clarification_resolution.py`
**Documentation:** `docs/TASK_CLARIFICATION_RESOLUTION.md`
**Integration:** Replaces Python heredocs for clarification resolution

**Usage:**
- Resolve single task clarification
- Update task with decision
- Automatically add comments and update status

---

### 13. `resolve_multiple_clarifications_tool` ✅
**Status:** ✅ Implemented and Ready
**Purpose:** Resolve multiple task clarifications from JSON decisions

**Parameters:**
- `decisions` (str): JSON string mapping task IDs to decision data
- `move_to_todo` (bool): Whether to move tasks to Todo status (default: true)
- `dry_run` (bool): Preview mode without making changes (default: false)

**Returns:**
- `status` (str): success/error
- `tasks_processed` (int): Number of tasks processed
- `tasks_updated` (int): Number of tasks successfully updated
- `output` (str): Command output

**File:** `tools/task_clarification_resolution.py`
**Documentation:** `docs/TASK_CLARIFICATION_RESOLUTION.md`
**Integration:** Batch clarification resolution

**Usage:**
- Resolve multiple tasks at once
- Process decisions from JSON
- Batch update task descriptions

---

### 14. `list_tasks_awaiting_clarification_tool` ✅
**Status:** ✅ Implemented and Ready
**Purpose:** List all tasks in Review status that need clarification

**Parameters:** None

**Returns:**
- `status` (str): success/error
- `total_tasks` (int): Number of tasks awaiting clarification
- `tasks` (List[dict]): List of tasks with:
  - `task_id` (str): Task ID
  - `name` (str): Task name
  - `priority` (str): Task priority
  - `clarification` (str): Clarification question

**File:** `tools/task_clarification_resolution.py`
**Documentation:** `docs/TASK_CLARIFICATION_RESOLUTION.md`
**Integration:** Quick overview of tasks needing input

**Usage:**
- See what tasks need your input
- Review clarification questions
- Plan batch resolution

---

## Resources

### `automation://status`
Get automation server status and health information.

### `automation://history`
Get automation tool execution history.

### `automation://tools`
Get list of available automation tools.

---

## Fixes Applied

### Issue
All tools had the same import scoping issue: error handler functions were imported inside `try` blocks, making them unavailable in `except` blocks, causing "cannot access local variable" errors.

### Solution
Moved error handler imports to module level (top of each tool file) with proper fallback handling:

1. Try relative import first (`from ..error_handler import ...`)
2. Fallback to absolute import (`from error_handler import ...`)
3. Final fallback: define minimal versions if imports fail

### Files Fixed
- ✅ `tools/docs_health.py`
- ✅ `tools/todo2_alignment.py`
- ✅ `tools/duplicate_detection.py`
- ✅ `tools/dependency_security.py`
- ✅ `tools/automation_opportunities.py`
- ✅ `tools/todo_sync.py`
- ✅ `tools/pwa_review.py`

---

## Testing

All tools have been verified to:
- ✅ Import successfully
- ✅ Have error handler functions available at module level
- ✅ Handle exceptions properly
- ✅ Return structured JSON responses

---

## Next Steps

1. **Restart Cursor** to reload MCP server with fixed code
2. **Test tools** via MCP interface
3. **Use tools** for project automation tasks

---

## Usage Examples

### Documentation Health
```
"Check documentation health and create Todo2 tasks for issues"
```

### Task Alignment
```
"Analyze Todo2 task alignment with project goals"
```

### Duplicate Detection
```
"Find duplicate Todo2 tasks with 85% similarity"
```

### Security Scan
```
"Scan all dependencies for security vulnerabilities"
```

### Automation Opportunities
```
"Find automation opportunities with value score >= 0.8"
```

### Todo Sync
```
"Sync todos between shared table and Todo2 (dry run)"
```

### PWA Review
```
"Review PWA configuration and suggest improvements"
```

---

**All tools are ready for use after Cursor restart!**
