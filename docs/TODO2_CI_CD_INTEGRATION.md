# TODO2 and CI/CD Integration

**Date:** 2025-01-20
**Purpose:** Integration between TODO2 task management and CI/CD workflows
**Status:** ✅ **Active Integration**

---

## Overview

This document describes how TODO2 tasks integrate with CI/CD workflows and parallel agent development. The integration ensures task tracking, coordination validation, and automatic synchronization.

---

## Integration Points

### 1. CI/CD Workflow Validation

**Location:** `.github/workflows/parallel-agents-ci.yml`

**Validation Steps:**

1. ✅ **TODO Table Validation** - Checks format and completeness
2. ✅ **TODO2 Sync Validation** - Verifies TODO2 tasks are in sync
3. ✅ **Coordination Checks** - Ensures both agents are coordinated

**What Gets Validated:**

- TODO table format (agents/shared/TODO_OVERVIEW.md)
- TODO2 file structure (.todo2/state.todo2.json)
- Task sync status between systems
- Agent coordination (no conflicts)

### 2. Shared TODO Table

**Location:** `agents/shared/TODO_OVERVIEW.md`

**Purpose:**

- Quick reference for all agents
- Status tracking (pending → in_progress → completed)
- Agent assignment (which agent owns which task)

**Format:**

```markdown
| TODO ID | Description | Owner Agent | Status |
|---------|-------------|-------------|--------|
| 4 | Add ANSI colorized output | backend | pending |
```

### 3. TODO2 System

**Location:** `.todo2/state.todo2.json`

**Purpose:**

- Detailed task tracking with rich metadata
- Dependencies, priorities, tags
- Research comments, result comments
- Full activity history

**Features:**

- Long descriptions with acceptance criteria
- Priority levels (high, medium, low)
- Tags for organization
- Dependencies between tasks
- Status workflow (Todo → In Progress → Review → Done)

---

## Synchronization

### Automated Sync

**Script:** `scripts/automate_todo_sync.py`

**Features:**

- Bidirectional sync between shared TODO and TODO2
- Status mapping (pending ↔ Todo, in_progress ↔ In Progress, completed ↔ Done)
- Conflict resolution (Todo2 is source of truth)
- Auto-creates TODO2 tasks for new shared TODOs

**Run Manually:**

```bash

# Dry run (check without changes)

python3 scripts/automate_todo_sync.py --dry-run

# Live sync

python3 scripts/automate_todo_sync.py
```

**Run in CI:**

- Automatic validation in coordination-validation job
- Warnings only (doesn't block CI)
- Reports sync status

### Status Mapping

| Shared TODO | TODO2 | Notes |
|-------------|-------|-------|
| `pending` | `Todo` | Initial state |
| `in_progress` | `In Progress` | Active work |
| `completed` | `Done` | Finished |
| - | `Review` | Maps to `in_progress` in shared TODO |
| - | `Cancelled` | Maps to `completed` |

---

## CI/CD Workflow Integration

### Coordination Validation Job

**Workflow:** `.github/workflows/parallel-agents-ci.yml`

**Job:** `coordination-validation`

**Validations:**

1. **TODO Table Format** - Checks markdown table structure
2. **TODO2 Sync** - Validates TODO2 sync status
3. **API Contract** - Validates API contract changes
4. **Merge Conflicts** - Checks for merge conflicts

**Validation Scripts:**

```bash

# TODO Table validation

bash scripts/validate_todo_table.sh

# TODO2 sync validation

bash scripts/validate_todo2_sync.sh

# API contract validation

bash scripts/validate_api_contract.sh
```

---

## Task Creation for CI/CD Setup

### TODO2 Tasks for CI/CD

The following TODO2 tasks should be created for the CI/CD setup work:

1. **Setup GitHub Actions Runners** (Ubuntu + macOS)
2. **Configure Parallel Agent CI Workflow**
3. **Create Validation Scripts**
4. **Document Agent Hostnames and Paths**
5. **Test CI/CD Workflows**

### Shared TODO Table Entries

**Location:** `agents/shared/TODO_OVERVIEW.md`

Add entries for CI/CD setup tasks (if using shared TODO table for tracking).

---

## Workflow

### When Creating New Tasks

**For TODO2:**

1. Create task with detailed description
2. Add tags, priority, dependencies
3. Add research comments if needed

**For Shared TODO Table:**

1. Add entry to `agents/shared/TODO_OVERVIEW.md`
2. Set status: `pending`, `in_progress`, or `completed`
3. Assign owner agent

**Sync:**

- Run `python3 scripts/automate_todo_sync.py` to sync
- Or let CI/CD validation detect sync issues

### When Updating Task Status

**TODO2 First (Recommended):**

1. Update TODO2 task status
2. Run sync script to update shared TODO table

**Or Shared TODO First:**

1. Update shared TODO table
2. Run sync script to update TODO2

**CI/CD Validates:**

- Sync status checked automatically
- Warnings if sync issues detected

---

## Validation in CI/CD

### Pre-Merge Checks

**On Every PR:**

1. TODO Table format validation
2. TODO2 sync status check
3. Coordination validation
4. API contract validation

**Blocking vs Non-Blocking:**

- TODO Table format: **Blocking** (must pass)
- TODO2 sync: **Non-blocking** (warnings only)
- API contract: **Blocking** (must pass)
- Coordination: **Blocking** (must pass)

### Post-Merge Actions

**After Merge:**

1. Run sync script to ensure both systems updated
2. Update TODO2 task status to "Done"
3. Update shared TODO table status to "completed"
4. Document completion in TODO2 result comment

---

## Best Practices

### For Parallel Agent Development

**Starting Work:**

1. Create TODO2 task (or use existing)
2. Update shared TODO table status: `pending` → `in_progress`
3. Update TODO2 status: `Todo` → `In Progress`
4. Run sync script

**During Work:**

1. Update TODO2 with progress notes
2. Add research comments if needed
3. Update dependencies as discovered

**Completing Work:**

1. Add TODO2 result comment
2. Update TODO2 status: `In Progress` → `Review`
3. Update shared TODO table: `in_progress` → (wait for review)
4. After approval: Update both to `Done`/`completed`

**Coordination:**

1. Both agents check shared TODO table
2. Update TODO2 for detailed tracking
3. Run sync before pushing

---

## Troubleshooting

### Sync Issues

**Problem:** TODO2 and shared TODO table out of sync

**Solution:**

```bash

# Check sync status

python3 scripts/automate_todo_sync.py --dry-run

# Run sync

python3 scripts/automate_todo_sync.py
```

### CI/CD Validation Failures

**Problem:** TODO2 sync validation fails in CI

**Solution:**

1. Check sync status locally
2. Run sync script to fix issues
3. Commit sync results
4. Re-run CI

### Missing Tasks

**Problem:** Task in TODO2 but not in shared TODO (or vice versa)

**Solution:**

- Run sync script (auto-creates missing tasks)
- Or manually add to missing system
- Then run sync to keep in sync

---

## Automation

### Scheduled Sync

**Cron Job:**

```bash

# Run hourly sync

0 * * * * cd /path/to/project && python3 scripts/automate_todo_sync.py
```

**Or use automation script:**

```bash
./scripts/setup_todo_sync_cron.sh
```

### CI/CD Integration

**In Workflows:**

- Validation runs on every PR
- Sync can run automatically on merge
- Status updates can trigger notifications

---

## Task Creation Examples

### TODO2 Task for CI/CD Setup

**Example Task Structure:**

```json
{
  "id": "T-XXX",
  "name": "Setup GitHub Actions runners on Ubuntu and macOS agents",
  "long_description": "🎯 **Objective:** Set up self-hosted GitHub Actions runners on both remote agents...",
  "status": "Todo",
  "priority": "high",
  "tags": ["ci-cd", "infrastructure", "parallel-agents"],
  "dependencies": []
}
```

### Shared TODO Entry

**Example Entry:**

```markdown
| CI-1 | Setup GitHub Actions runners on Ubuntu and macOS | ubuntu/macos | pending |
```

---

## Next Steps

### Immediate Actions

1. **Create TODO2 Tasks:**
   - Setup GitHub Actions runners (Ubuntu)
   - Setup GitHub Actions runners (macOS)
   - Test parallel agent CI workflow
   - Collect system information from both agents

2. **Update Shared TODO Table:**
   - Add CI/CD setup tasks
   - Assign to appropriate agents
   - Update status as work progresses

3. **Run Sync:**

   ```bash
   python3 scripts/automate_todo_sync.py
   ```

4. **Test Validation:**
   - Create test PR
   - Verify CI/CD validation runs
   - Check sync status reports

---

## References

- [TODO Sync Automation](./TODO_SYNC_AUTOMATION.md) - Detailed sync documentation
- [CI/CD Enhancement Plan](./CI_CD_ENHANCEMENT_PLAN.md) - CI/CD setup guide
- [Parallel Agents Workflow](./PARALLEL_CURSOR_AGENTS_WORKFLOW.md) - Parallel development guide
- [Coordination Guidelines](../agents/shared/COORDINATION.md) - Agent coordination

---

**Status:** ✅ Integration documented and ready for use

**Next Action:** Create TODO2 tasks for CI/CD setup and run sync to ensure both systems are aligned.
