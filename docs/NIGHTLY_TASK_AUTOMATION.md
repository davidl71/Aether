# Nightly Task Automation Tool

**Date:** 2025-01-20
**Status:** ✅ **Implemented and Ready**

---

## Overview

The **Nightly Task Automation Tool** automatically executes background-capable TODO2 tasks in parallel across multiple remote hosts. Tasks requiring user input or decisions are automatically moved to Review status, allowing the automation to proceed with autonomous tasks.

---

## Features

### ✅ Automatic Task Filtering

- **Background-Capable Tasks:** Automatically identified and queued for execution
- **Interactive Tasks:** Automatically moved to Review status (requires user input)
- **Smart Detection:** Uses execution mode analysis to categorize tasks

### ✅ Parallel Execution

- **Multiple Hosts:** Distributes tasks across Ubuntu and macOS agents
- **Round-Robin Assignment:** Balances workload across available hosts
- **Configurable Limits:** Control max tasks per host and total parallel tasks

### ✅ Safe Operation

- **Dry Run Mode:** Preview what would happen without executing
- **State Backups:** Automatic backups before state changes
- **Audit Trail:** All changes logged in task comments

---

## Usage

### Via MCP Tool

```python
# Run nightly automation with defaults
run_nightly_task_automation_tool()

# Custom configuration
run_nightly_task_automation_tool(
    max_tasks_per_host=5,      # Max tasks per host (default: 5)
    max_parallel_tasks=10,     # Max total parallel tasks (default: 10)
    priority_filter="high",    # Filter by priority (optional)
    tag_filter=["mcp", "research"],  # Filter by tags (optional)
    dry_run=False              # Preview mode (default: False)
)
```

### Via Command Line (Future)

```bash
# Dry run to preview
python3 -m mcp-servers.project-management-automation.tools.nightly_task_automation \
    --dry-run

# Execute with custom limits
python3 -m mcp-servers.project-management-automation.tools.nightly_task_automation \
    --max-tasks-per-host 10 \
    --max-parallel-tasks 20 \
    --priority-filter high
```

---

## Task Categorization

### ✅ Background-Capable Tasks (Executed)

**Automatically identified by:**
- MCP extension tasks (MCP-EXT-*)
- Research tasks ("Research" in name)
- Implementation tasks ("Implement", "Create", "Add", "Update" in name)
- Testing tasks ("Test", "Testing", "Validate" in name)
- Documentation tasks ("Document", "Documentation" in name)
- Configuration tasks ("Config", "Configure", "Setup" in name)

**Excluded if:**
- Status is "Review" or "Done"
- Needs clarification (has "clarification required")
- Requires user input (has "user input" or "user interaction")
- Design decisions needed (Design + Framework/System/Strategy)

---

### ❌ Interactive Tasks (Moved to Review)

**Automatically moved to Review if:**
- Needs clarification
- Requires user input
- Design decisions needed
- Strategy/planning input required

**Result:** Task status changed to "Review" with explanatory comment

---

## Configuration

### Agent Hostnames

The tool automatically loads agent configuration from:
- **File:** `docs/AGENT_HOSTNAMES.md`
- **Default Configuration:** Built-in Ubuntu and macOS agent details

**Current Agents:**
- **Ubuntu:** `david@192.168.192.57` → `~/ib_box_spread_full_universal`
- **macOS M4:** `davidl@192.168.192.141` → `/Users/davidl/Projects/Trading/ib_box_spread_full_universal`

---

## Output

### Result Structure

```json
{
  "timestamp": "2025-01-20T12:00:00Z",
  "dry_run": false,
  "summary": {
    "background_tasks_found": 46,
    "interactive_tasks_found": 73,
    "tasks_assigned": 10,
    "tasks_moved_to_review": 5,
    "hosts_used": 2
  },
  "assigned_tasks": [
    {
      "task_id": "MCP-EXT-2",
      "task_name": "Implement validate_agent_coordination_tool",
      "host": "ubuntu",
      "hostname": "david@192.168.192.57"
    }
  ],
  "moved_to_review": [
    "T-60",
    "T-62"
  ],
  "background_tasks_remaining": 36
}
```

---

## Workflow

### 1. Task Discovery

1. Load TODO2 state file (`.todo2/state.todo2.json`)
2. Filter tasks by status (only "Todo" tasks)
3. Categorize tasks as background-capable or interactive

### 2. Interactive Task Handling

1. Identify interactive tasks (need user input/clarification)
2. Move to Review status
3. Add explanatory comment
4. Update task state

### 3. Background Task Assignment

1. Filter background-capable tasks
2. Apply priority/tag filters (if specified)
3. Assign to hosts using round-robin
4. Update task status to "In Progress"
5. Add assignment comment

### 4. State Persistence

1. Create backup of state file
2. Save updated state
3. Return execution results

---

## Examples

### Example 1: Default Nightly Run

```python
result = run_nightly_task_automation_tool()
# Assigns up to 10 tasks (5 per host) across 2 hosts
# Moves interactive tasks to Review
```

**Expected Output:**
- 10 background tasks assigned (5 to Ubuntu, 5 to macOS)
- 5-10 interactive tasks moved to Review
- Summary of actions taken

---

### Example 2: High Priority Only

```python
result = run_nightly_task_automation_tool(
    priority_filter="high",
    max_parallel_tasks=20
)
# Only processes high-priority tasks
# Up to 20 tasks in parallel
```

**Expected Output:**
- 20 high-priority background tasks assigned
- Interactive high-priority tasks moved to Review
- Summary focused on high-priority work

---

### Example 3: MCP Extensions Only

```python
result = run_nightly_task_automation_tool(
    tag_filter=["mcp"],
    max_parallel_tasks=10
)
# Only processes MCP extension tasks
```

**Expected Output:**
- 10 MCP-EXT tasks assigned
- Other tasks skipped
- Summary of MCP extension progress

---

### Example 4: Dry Run (Preview)

```python
result = run_nightly_task_automation_tool(
    dry_run=True,
    max_parallel_tasks=10
)
# Preview what would happen without executing
```

**Expected Output:**
- List of tasks that would be assigned
- List of tasks that would be moved to Review
- No actual state changes made

---

## Integration

### With GitHub Actions (Future)

```yaml
name: Nightly Task Automation

on:
  schedule:
    - cron: '0 2 * * *'  # Run at 2 AM daily

jobs:
  automate-tasks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Nightly Automation
        run: |
          python3 -m mcp-servers.project-management-automation.tools.nightly_task_automation \
            --max-tasks-per-host 10 \
            --max-parallel-tasks 20
```

### With Cron (Local)

```bash
# Add to crontab: Run nightly at 2 AM
0 2 * * * cd /path/to/project && python3 -m mcp-servers.project-management-automation.tools.nightly_task_automation
```

---

## Safety Features

### ✅ Dry Run Mode

Always test with `dry_run=True` first to preview actions.

### ✅ Automatic Backups

State file is automatically backed up before changes.

### ✅ Audit Trail

All changes logged in:
- Task comments (why moved to Review)
- Task status changes (In Progress, Review)
- TODO2 state file changes array

### ✅ Idempotent Operation

Can be run multiple times safely:
- Already "In Progress" tasks are skipped
- Already "Review" tasks are skipped
- Already "Done" tasks are skipped

---

## Limitations

### Current Limitations

1. **Task Execution:** Currently only assigns tasks and updates status
   - Future: Actual task execution via SSH/Cursor agents
2. **Host Communication:** Uses basic SSH connectivity
   - Future: Full Cursor agent integration
3. **Progress Tracking:** Limited to status updates
   - Future: Real-time progress tracking and reporting

### Future Enhancements

1. **Real Task Execution:** Execute tasks via Cursor agents on remote hosts
2. **Progress Monitoring:** Track task progress in real-time
3. **Failure Handling:** Retry failed tasks, notify on errors
4. **Result Aggregation:** Collect and summarize results from all hosts
5. **Scheduling:** Built-in scheduling for nightly/weekly runs

---

## Troubleshooting

### Tasks Not Being Assigned

**Check:**
1. Tasks are in "Todo" status (not "In Progress" or "Done")
2. Tasks are background-capable (not interactive)
3. Host limits not exceeded (`max_tasks_per_host`)
4. Total limit not exceeded (`max_parallel_tasks`)

---

### Too Many Tasks Moved to Review

**Expected:** Interactive tasks are intentionally moved to Review

**If unexpected:**
1. Review task descriptions for "clarification required"
2. Check for "user input" or "user interaction" mentions
3. Verify tasks are actually interactive vs. background-capable

---

### State File Issues

**Backup Location:** `.todo2/state.todo2.json.bak`

**If state corrupted:**
1. Restore from backup: `.todo2/state.todo2.json.bak`
2. Review task changes in comments
3. Manually fix any issues

---

## Related Tools

- **`analyze_task_execution_modes`:** Categorizes tasks by execution mode
- **`validate_agent_coordination`:** Validates agent coordination
- **`run_daily_automation`:** Daily maintenance automation

---

## References

- [TODO2 Execution Mode Analysis](./TODO2_EXECUTION_MODE_ANALYSIS.md)
- [Agent Hostnames](./AGENT_HOSTNAMES.md)
- [TODO2 Background vs Interactive Summary](./TODO2_BACKGROUND_VS_INTERACTIVE_SUMMARY.md)

---

**Status:** ✅ **Ready for Use**
