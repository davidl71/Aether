# Nightly Automation with Batch Approval

**Location**: `.github/workflows/nightly-task-automation.yml`

**Purpose**: Automated nightly and weekly task processing with batch approval of research tasks.

---

## Schedule

### Daily (Nightly)
- **Time**: 2 AM UTC
- **Purpose**: Process background tasks, assign to agents, move interactive tasks to Review

### Weekly
- **Time**: 3 AM UTC (Sundays)
- **Purpose**: Batch approval of research tasks, cleanup, and weekly maintenance

---

## Workflow Steps

### 1. Batch Approval (Review → Todo)

**Runs First**: Before task assignment

```bash
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --new-status Todo --yes
```

**What It Does:**
- Finds all Review tasks with "Clarification Required: None"
- Moves them to Todo status
- Makes them available for automation

**Why First:**
- Clears the Review queue of tasks that don't need user input
- Makes more tasks available for background execution
- Reduces manual review burden

### 2. Nightly Task Automation

**Runs After**: Batch approval

**What It Does:**
- Identifies background-capable tasks
- Assigns them to available hosts (Ubuntu, macOS)
- Moves interactive tasks to Review
- Updates task statuses

**Integration:**
- The nightly automation tool now includes batch approval internally
- It runs batch approval before processing tasks
- Results include `tasks_batch_approved` count

---

## Results Tracking

### GitHub Actions Summary

The workflow creates a summary with:
- Tasks batch approved (Review → Todo)
- Background tasks found
- Tasks assigned to agents
- Tasks moved to Review
- Hosts used

### Example Output

```
## Nightly Task Automation Results

- Tasks batch approved (Review → Todo): success
- Background tasks found: 22
- Tasks assigned: 10
- Tasks moved to Review: 10
- Tasks batch approved: 5
- Hosts used: 2
```

---

## Manual Triggering

You can manually trigger the workflow:

```bash
# Via GitHub Actions UI or API
gh workflow run nightly-task-automation.yml
```

**With Custom Parameters:**
- `max_tasks_per_host`: Max tasks per host (default: 5)
- `max_parallel_tasks`: Max total parallel tasks (default: 10)
- `priority_filter`: Filter by priority (high/medium/low)
- `dry_run`: Preview only (true/false)

---

## Integration with Batch Script

The nightly automation uses `scripts/batch_update_todos.py` for:
- ✅ Batch approving Review tasks with no clarification needed
- ✅ Automatically processing research tasks
- ✅ Reducing manual review workload

**Benefits:**
- Automated approval of research tasks
- More tasks available for background execution
- Reduced manual intervention needed

---

## Weekly Maintenance

The weekly run (Sundays at 3 AM UTC) performs:
1. **Batch Approval**: Approve all Review tasks that don't need clarification
2. **Task Assignment**: Assign background tasks to agents
3. **Cleanup**: Move interactive tasks to Review for user input

**Purpose:**
- Weekly cleanup of the task queue
- Ensure research tasks are approved and available
- Maintain task flow

---

## Configuration

### Adjust Schedule

Edit `.github/workflows/nightly-task-automation.yml`:

```yaml
schedule:
  # Daily at 2 AM UTC
  - cron: '0 2 * * *'
  # Weekly on Sundays at 3 AM UTC
  - cron: '0 3 * * 0'
```

### Adjust Batch Approval Criteria

The batch approval uses:
- Status: `Review`
- Clarification: `None` (no clarification needed)
- New Status: `Todo`

To change criteria, modify the batch approval step in the workflow.

---

## Troubleshooting

### Batch Approval Fails

**Check:**
1. Script exists: `scripts/batch_update_todos.py`
2. Python version: Python 3.11+
3. Permissions: Script is executable

**Logs:**
- Check GitHub Actions logs for error messages
- Batch approval errors don't fail the workflow (non-blocking)

### No Tasks Approved

**Possible Reasons:**
- No Review tasks with "Clarification Required: None"
- All Review tasks need clarification
- Tasks already processed

**Verify:**
```bash
python3 scripts/batch_update_todos.py list --status Review --clarification-none
```

---

## See Also

- `docs/BATCH_TODO_UPDATE_SCRIPT.md` - Batch script documentation
- `docs/NIGHTLY_TASK_AUTOMATION.md` - Nightly automation details
- `scripts/batch_update_todos.py` - Batch update script
