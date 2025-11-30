# Nightly Task Automation - Scheduling Guide

**Date:** 2025-01-20
**Status:** ✅ **GitHub Actions Workflow Created**

---

## Scheduling Options

### 1. GitHub Actions (Recommended) ✅

**Workflow:** `.github/workflows/nightly-task-automation.yml`

**Schedule:** Runs daily at 2 AM UTC

**Features:**

- Automatic daily execution
- Manual trigger via GitHub Actions UI
- Configurable parameters
- Results saved as artifacts
- Summary in GitHub Actions UI

---

## GitHub Actions Setup

### Automatic Daily Run

The workflow runs automatically at 2 AM UTC daily:

```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM UTC daily
```

### Manual Trigger

You can also trigger manually from GitHub Actions UI:

- Go to Actions → Nightly Task Automation
- Click "Run workflow"
- Configure parameters:
  - Max tasks per host (default: 5)
  - Max parallel tasks (default: 10)
  - Priority filter (optional)
  - Dry run mode (preview only)

---

## Configuration

### Default Settings

- **Max tasks per host:** 5
- **Max parallel tasks:** 10
- **Priority filter:** None (all priorities)
- **Dry run:** False

### Custom Settings

Override via workflow_dispatch inputs:

- `max_tasks_per_host`: Integer
- `max_parallel_tasks`: Integer
- `priority_filter`: "high", "medium", or "low"
- `dry_run`: true/false

---

## Results

### Artifacts

Results are saved as artifacts:

- **Name:** `nightly-automation-results`
- **File:** `nightly-automation-results.json`
- **Retention:** 30 days

### Summary

Summary appears in GitHub Actions UI:

- Background tasks found
- Tasks assigned
- Tasks moved to Review
- Hosts used

---

## Alternative: Cron (Local Machine)

If you prefer to run locally instead of GitHub Actions:

### Setup Cron Job

```bash

# Edit crontab

crontab -e

# Add line (runs at 2 AM local time daily)

0 2 * * * cd /path/to/project && python3 -m mcp-servers.project-management-automation.tools.nightly_task_automation >> /tmp/nightly-automation.log 2>&1
```

### With Custom Settings

```bash

# Create wrapper script

cat > scripts/run_nightly_automation.sh << 'EOF'

#!/bin/bash

cd /path/to/project
python3 << 'PYTHON'
import sys
sys.path.insert(0, 'mcp-servers/project-management-automation')
from tools.nightly_task_automation import run_nightly_task_automation

result = run_nightly_task_automation(
    max_tasks_per_host=5,
    max_parallel_tasks=10,
    priority_filter='high',
    dry_run=False
)
print(f"Assigned: {result['summary']['tasks_assigned']} tasks")
print(f"Moved to Review: {result['summary']['tasks_moved_to_review']} tasks")
PYTHON
EOF

chmod +x scripts/run_nightly_automation.sh

# Add to crontab

0 2 * * * /path/to/project/scripts/run_nightly_automation.sh
```

---

## Monitoring

### GitHub Actions

- View workflow runs: GitHub Actions → Nightly Task Automation
- Check artifacts for detailed results
- Review summary in workflow run

### Local Cron

- Check logs: `/tmp/nightly-automation.log`
- Monitor TODO2 state file changes
- Review task comments for audit trail

---

## Timezone Configuration

### GitHub Actions

Default is UTC. To change schedule:

```yaml

# Example: 2 AM EST (UTC-5) = 7 AM UTC

on:
  schedule:
    - cron: '0 7 * * *'  # 2 AM EST
```

### Local Cron

Cron uses local machine timezone automatically.

---

## Recommendations

### For Production Use

1. **Start with Manual Runs:**
   - Test workflow manually first
   - Verify results are correct
   - Adjust parameters as needed

2. **Use Dry Run First:**
   - Enable dry run for first scheduled run
   - Review what would happen
   - Then disable dry run

3. **Monitor Initial Runs:**
   - Check first few automatic runs
   - Review artifacts and summaries
   - Adjust limits if needed

4. **Gradual Scale-Up:**
   - Start with small limits (3-5 tasks)
   - Gradually increase as confidence grows
   - Monitor system load

---

## Troubleshooting

### Workflow Not Running

**Check:**

1. Workflow file is in `.github/workflows/` directory
2. File syntax is valid YAML
3. Repository has Actions enabled
4. Schedule cron syntax is correct

### Tasks Not Assigned

**Check:**

1. Tasks are in "Todo" status
2. Tasks are background-capable
3. Limits not too restrictive
4. Filters not excluding all tasks

---

## Related Documentation

- [Full Tool Documentation](./NIGHTLY_TASK_AUTOMATION.md)
- [Quick Start Guide](./NIGHTLY_AUTOMATION_QUICK_START.md)
- [Implementation Summary](./NIGHTLY_AUTOMATION_IMPLEMENTATION_SUMMARY.md)

---

**Status:** ✅ **GitHub Actions Workflow Ready**
