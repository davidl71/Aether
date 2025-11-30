# Nightly Automation Monitoring Guide

**Date:** 2025-11-24
**Status:** ✅ Active and Scheduled

---

## Automation Schedule

### Daily Automation

- **Time:** 2:00 AM UTC daily
- **Cron:** `0 2 * * *`
- **Purpose:**
  - Batch approve tasks from Review → Todo
  - Assign background tasks to agents
  - Move interactive tasks to Review

### Weekly Automation

- **Time:** 3:00 AM UTC on Sundays
- **Cron:** `0 3 * * 0`
- **Purpose:**
  - Comprehensive batch approval
  - Weekly cleanup and coordination

### Manual Trigger

- **Available:** Yes (via GitHub Actions UI)
- **Location:** `.github/workflows/nightly-task-automation.yml`
- **Parameters:**
  - `max_tasks_per_host` (default: 5)
  - `max_parallel_tasks` (default: 10)
  - `priority_filter` (optional)
  - `dry_run` (default: false)

---

## Monitoring Methods

### 1. GitHub Actions UI

**View Runs:**

```
https://github.com/davidl71/ib_box_spread_full_universal/actions/workflows/nightly-task-automation.yml
```

**Check Status:**

- ✅ Green: Successful run
- ⚠️ Yellow: In progress
- ❌ Red: Failed run
- ⚪ Gray: Not run yet

### 2. Command Line (GitHub CLI)

```bash

# List workflow runs

gh workflow list

# View latest run

gh run list --workflow=nightly-task-automation.yml

# View run details

gh run view <run-id>
```

### 3. Check TODO2 State

```bash

# Check for automation activity

python3 scripts/batch_update_todos.py list --status Review

# Check assigned tasks

python3 -c "
import json
from pathlib import Path
data = json.load(open('.todo2/state.todo2.json'))
todos = data.get('todos', [])
assigned = [t for t in todos if 'assigned' in str(t.get('comments', []))]
print(f'Assigned tasks: {len(assigned)}')
"
```

### 4. Dry Run Test

```bash

# Test automation without executing

python3 -c "
import sys
sys.path.insert(0, 'mcp-servers/project-management-automation')
from tools.nightly_task_automation import NightlyTaskAutomation
automation = NightlyTaskAutomation()
result = automation.run_nightly_automation(dry_run=True)
import json
print(json.dumps(json.loads(result), indent=2))
"
```

---

## What the Automation Does

### Step 1: Batch Approval

- Checks tasks in Review status
- Approves tasks with "Clarification Required: None"
- Moves them from Review → Todo

### Step 2: Working Copy Health Check

- Checks all agents (local, ubuntu, macos)
- Reports uncommitted changes
- Reports sync status
- Includes warnings in results

### Step 3: Task Filtering

- Identifies background-capable tasks (research, automation, etc.)
- Identifies interactive tasks (need user input)
- Filters by priority and tags if specified

### Step 4: Task Assignment

- Assigns background tasks to available hosts
- Round-robin distribution across agents
- Updates task status and adds assignment comments

### Step 5: Interactive Task Handling

- Moves interactive tasks to Review status
- Adds comments explaining why
- Waits for user clarification

---

## Expected Results

### Successful Run Output

```json
{
  "timestamp": "2025-11-24T02:00:00Z",
  "dry_run": false,
  "working_copy_status": {
    "summary": {
      "total_agents": 3,
      "ok_agents": 2,
      "warning_agents": 1
    }
  },
  "summary": {
    "background_tasks_found": 15,
    "interactive_tasks_found": 5,
    "tasks_assigned": 10,
    "tasks_moved_to_review": 5,
    "tasks_batch_approved": 3,
    "hosts_used": 2,
    "working_copy_warnings": 1
  },
  "assigned_tasks": [...],
  "moved_to_review": [...]
}
```

---

## Troubleshooting

### Automation Not Running

**Check:**

1. GitHub Actions enabled for repository
2. Workflow file syntax is valid
3. Scheduled time has passed
4. No workflow errors in Actions tab

**Fix:**

- Manually trigger workflow to test
- Check workflow file for syntax errors
- Verify cron schedule is correct

### No Tasks Assigned

**Possible Reasons:**

1. No background-capable tasks available
2. All tasks need clarification
3. Task limits reached
4. No available hosts configured

**Check:**

```bash

# Count background tasks

python3 scripts/analyze_task_execution_modes.py
```

### Tasks Not Moving to Review

**Possible Reasons:**

1. Tasks are already background-capable
2. Automation logic needs adjustment
3. Task tags/descriptions don't match patterns

**Fix:**

- Review task execution mode analysis
- Update task tags/descriptions
- Adjust automation filtering logic

---

## Monitoring Checklist

### Daily

- [ ] Check GitHub Actions for last run status
- [ ] Review tasks moved to Review
- [ ] Check assigned tasks status
- [ ] Verify working copy health

### Weekly

- [ ] Review comprehensive automation results
- [ ] Check batch approval results
- [ ] Verify all agents are in sync
- [ ] Review task distribution across agents

### After Each Run

- [ ] Check automation summary
- [ ] Review assigned tasks
- [ ] Address tasks moved to Review
- [ ] Verify working copy warnings

---

## Current Status

**Last Check:** 2025-11-24

**Automation Status:**

- ✅ Workflow configured
- ✅ Scheduled (daily + weekly)
- ✅ Manual trigger available
- ⏳ Waiting for first scheduled run

**Ready Tasks:**

- Todo tasks: 79+ (includes 5 newly approved)
- Background-capable: ~15-20 estimated
- Interactive: ~5-10 estimated

**Agent Status:**

- ✅ All agents synced
- ⚠️ Some uncommitted files (non-blocking)
- ✅ Ready for task assignment

---

## Next Steps

1. **Wait for First Scheduled Run**
   - Daily: Tomorrow at 2 AM UTC
   - Weekly: Next Sunday at 3 AM UTC

2. **Monitor Results**
   - Check GitHub Actions after run
   - Review assigned tasks
   - Address Review tasks

3. **Adjust Limits if Needed**
   - Increase `max_tasks_per_host` if needed
   - Increase `max_parallel_tasks` if needed
   - Add priority/tag filters if needed

---

**See Also:**

- `docs/NIGHTLY_TASK_AUTOMATION.md` - Complete automation documentation
- `docs/NIGHTLY_AUTOMATION_SCHEDULING.md` - Scheduling details
- `.github/workflows/nightly-task-automation.yml` - Workflow file
