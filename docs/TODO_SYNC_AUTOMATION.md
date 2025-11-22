# Shared TODO Table Synchronization Automation

**Date**: 2025-11-20
**Purpose**: Automated bidirectional synchronization between `agents/shared/TODO_OVERVIEW.md` and Todo2

---

## Overview

This automation keeps the shared TODO table and Todo2 tasks in sync, ensuring all agents have consistent task status information.

---

## How It Works

### Bidirectional Sync

1. **Reads Both Systems**:
   - Shared TODO: `agents/shared/TODO_OVERVIEW.md` (markdown table)
   - Todo2: `.todo2/state.todo2.json` (JSON)

2. **Matches Tasks**:
   - By description similarity (>70% match)
   - By explicit TODO ID reference in Todo2 description

3. **Detects Conflicts**:
   - Status mismatches between systems
   - Resolves by preferring Todo2 status (source of truth)

4. **Creates Missing Tasks**:
   - New shared TODOs → Creates Todo2 tasks
   - New Todo2 tasks → Logged (not auto-created in shared TODO)

5. **Updates Status**:
   - Syncs status changes bidirectionally
   - Handles status mapping (pending↔Todo, in_progress↔In Progress, completed↔Done)

---

## Status Mapping

| Shared TODO | Todo2 | Notes |
|-------------|-------|-------|
| `pending` | `Todo` | Initial state |
| `in_progress` | `In Progress` | Active work |
| `completed` | `Done` | Finished |
| - | `Review` | Maps to `in_progress` in shared TODO |
| - | `Cancelled` | Maps to `completed` in shared TODO |

---

## Usage

### Manual Run

```bash
# Dry run (simulate without changes)
python3 scripts/automate_todo_sync.py --dry-run

# Live sync
python3 scripts/automate_todo_sync.py

# Custom output
python3 scripts/automate_todo_sync.py --output docs/MY_SYNC_REPORT.md
```

### Automated (Cron)

```bash
# Setup hourly cron job
./scripts/setup_todo_sync_cron.sh

# Test cron script manually
./scripts/run_todo_sync_cron.sh
```

**Cron Schedule**: Every hour at :00 (`0 * * * *`)

---

## Configuration

Edit `scripts/todo_sync_config.json`:

```json
{
  "project_root": ".",
  "todo_file": ".todo2/state.todo2.json",
  "dry_run": false,
  "output_file": "docs/TODO_SYNC_REPORT.md",
  "sync_frequency": "hourly"
}
```

---

## Output

### Report File

Generated at: `docs/TODO_SYNC_REPORT.md`

**Contents**:
- Executive summary
- Matches found
- Conflicts resolved
- New tasks created
- Updates performed

### Logs

- **Success**: `scripts/todo_sync.log`
- **Cron**: `scripts/todo_sync_cron.log`
- **Errors**: `scripts/todo_sync_cron_error.log`

---

## Conflict Resolution

### Strategy

**Todo2 is source of truth** for status conflicts:
- If Todo2 says "In Progress" and shared TODO says "pending"
- → Shared TODO updated to "in_progress"

### Rationale

- Todo2 is more actively updated
- Todo2 has richer status tracking
- Shared TODO is simpler (3 states)

---

## Task Creation

### New Shared TODOs

When a shared TODO has no Todo2 match:
- Creates new Todo2 task
- ID format: `SHARED-{TODO_ID}`
- Tags: `shared-todo`, `synced`, `{owner}`
- Description includes shared TODO ID reference

### New Todo2 Tasks

When a Todo2 task has no shared TODO match:
- Logged in report
- **Not** automatically created in shared TODO (manual review needed)
- Can be reviewed and manually added if needed

---

## Matching Algorithm

### Description Similarity

Uses word overlap (Jaccard similarity):
- Tokenizes both descriptions
- Calculates intersection / union
- Match if similarity > 0.7 (70%)

### Explicit ID Reference

Matches if Todo2 description contains:
- `TODO {ID}` (e.g., "TODO 5")
- `#{ID}` (e.g., "#5")

---

## Safety Features

### Dry Run Mode

Always test with `--dry-run` first:
- Simulates all operations
- Shows what would be changed
- No actual file modifications

### Backup

Before first live run:
```bash
# Backup shared TODO
cp agents/shared/TODO_OVERVIEW.md agents/shared/TODO_OVERVIEW.md.backup

# Backup Todo2
cp .todo2/state.todo2.json .todo2/state.todo2.json.backup
```

### Logging

All operations logged:
- What was matched
- What was updated
- What was created
- Any errors

---

## Integration with Intelligent Automation

Uses `IntelligentAutomationBase`:
- **Tractatus Thinking**: Understands sync structure
- **Sequential Thinking**: Plans sync workflow
- **Todo2 Integration**: Creates tracking tasks
- **Follow-up Tasks**: Creates review tasks for conflicts

---

## Troubleshooting

### No Matches Found

**Cause**: Descriptions too different or no ID references

**Solution**:
- Add explicit TODO ID to Todo2 description
- Improve description similarity
- Manually link tasks

### Too Many Conflicts

**Cause**: Systems out of sync

**Solution**:
- Run sync more frequently
- Review conflict resolution strategy
- Manually align high-priority tasks

### Sync Fails

**Check**:
1. File permissions
2. JSON validity (Todo2)
3. Markdown table format (shared TODO)
4. Logs for specific errors

---

## Best Practices

1. **Run Frequently**: Hourly sync keeps systems aligned
2. **Review Reports**: Check sync reports for issues
3. **Use Dry Run**: Test before live sync
4. **Manual Override**: For critical tasks, sync manually
5. **Monitor Logs**: Watch for sync errors

---

## Future Enhancements

- [ ] Two-way task creation (Todo2 → shared TODO)
- [ ] Custom conflict resolution rules
- [ ] Webhook notifications on conflicts
- [ ] Sync history tracking
- [ ] Visual diff of changes

---

*This automation ensures all agents stay aligned with consistent task status across systems.*
