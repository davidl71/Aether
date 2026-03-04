# Task Clarification Resolution Automation

**Status:** **Removed.** The script `scripts/resolve_task_clarifications.py` was removed. This repo does not perform direct Todo2 edits. Use **exarp-go** for task updates and clarifications. See `docs/MCP_REQUIRED_SERVERS.md`.

---

**Historical reference below.**

**Purpose:** Automated resolution of task clarifications.

---

## Quick Reference

### Resolve Single Task

```bash
python3 scripts/resolve_task_clarifications.py \
  --task-id T-76 \
  --clarification "Storage format preference" \
  --decision "Use JSON config for simplicity, design for future database migration"
```

### Resolve Multiple Tasks from File

Create `decisions.json`:

```json
{
  "T-76": {
    "clarification": "Storage format preference",
    "decision": "Use JSON config for simplicity, design for future database migration"
  },
  "T-77": {
    "clarification": "Preferred interface",
    "decision": "Primary: TUI form (interactive), Secondary: Config file editing, Import: CSV format"
  }
}
```

Then run:

```bash
python3 scripts/resolve_task_clarifications.py --file decisions.json
```

### Dry Run (Preview)

```bash
python3 scripts/resolve_task_clarifications.py --file decisions.json --dry-run
```

---

## Current Tasks Awaiting Input

**Remaining: 6 tasks in Review**

1. **T-76:** Bank loan storage (JSON vs database?)
2. **T-77:** Loan entry interface (TUI vs CLI vs config?)
3. **T-111:** Config file format (multiple sources vs single?)
4. **T-112:** Config loader (Python, TypeScript, or both?)
5. **T-113:** PWA config UI (settings page vs modal?)
6. **T-114:** TUI config (watch changes vs startup only?)

---

## Usage Examples

### Example 1: Resolve All Remaining Tasks

Create `remaining_decisions.json`:

```json
{
  "T-76": {
    "clarification": "Storage format preference (JSON config vs database)",
    "decision": "Start with JSON config for simplicity, design for future database migration, REST API for updates"
  },
  "T-77": {
    "clarification": "Preferred interface (TUI form, CLI commands, config file editing)",
    "decision": "Primary: TUI form (interactive), Secondary: Config file editing (power users), Import: CSV format"
  },
  "T-111": {
    "clarification": "Should config support multiple active sources or single source selection?",
    "decision": "Support multiple active sources with priority/fallback order"
  },
  "T-112": {
    "clarification": "Should loader be in Python, TypeScript, or both?",
    "decision": "Both - Python for TUI/backend, TypeScript for PWA, shared JSON schema"
  },
  "T-113": {
    "clarification": "Should this be a settings page, modal, or both?",
    "decision": "Settings page (primary) with modal for quick edits"
  },
  "T-114": {
    "clarification": "Should TUI watch for config file changes or only read on startup?",
    "decision": "Watch for changes (better UX, allows hot-reload)"
  }
}
```

Run:

```bash
python3 scripts/resolve_task_clarifications.py --file remaining_decisions.json
```

### Example 2: Single Task with Custom Options

```bash

# Resolve but don't move to Todo

python3 scripts/resolve_task_clarifications.py \
  --task-id T-76 \
  --clarification "Storage format" \
  --decision "JSON config" \
  --no-move-to-todo
```

---

## What the Script Does

1. **Loads TODO2 state** from `.todo2/state.todo2.json`
2. **Finds the task** by ID
3. **Updates task description** with clarification and decision
4. **Adds note comment** documenting the resolution
5. **Moves to Todo status** (unless `--no-move-to-todo` is used)
6. **Saves updated state** back to file

---

## Benefits Over Python Heredocs

✅ **No code editing required** - Just JSON or command-line arguments
✅ **Reusable** - Create decision files for similar tasks
✅ **Dry run support** - Preview changes before applying
✅ **Batch processing** - Resolve multiple tasks at once
✅ **Version controlled** - Decision files can be committed
✅ **Clear audit trail** - Comments added automatically

---

## Integration with Other Tools

### With Batch Update Script

```bash

# First resolve clarifications

python3 scripts/resolve_task_clarifications.py --file decisions.json

# Then approve/move to Todo (if not already done)

python3 scripts/batch_update_todos.py update-status \
  --task-ids T-76,T-77,T-111,T-112,T-113,T-114 \
  --new-status Todo \
  --yes
```

### With Nightly Automation

The nightly automation will automatically:

1. Check for tasks in Review
2. Batch approve those with "Clarification Required: None"
3. Assign background tasks

After resolving clarifications, tasks will be picked up by nightly automation.

---

## See Also

- `scripts/batch_update_todos.py` - Batch task operations
- `docs/TASKS_AWAITING_INPUT.md` - Current tasks needing input
- `docs/BATCH_TODO_UPDATE_SCRIPT.md` - Batch script documentation
