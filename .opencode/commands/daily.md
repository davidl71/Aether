---
description: Daily workflow - prime, check tasks, get suggestions
---

# Daily Workflow

Standard daily workflow for Aether development.

## Morning Routine

```bash
# 1. Prime session - get context, tasks, hints
/prime

# 2. List tasks - see what's in progress and todo
/tasks

# 3. Check scorecard - project health overview
/scorecard
```

## Picking Work

After `/prime`, you'll see:
- **Active runs** - What's currently being worked on
- **Suggested next** - Priority tasks ready to work on
- **Hints** - Context about the project

To start a task:
```bash
# Option 1: Use plugin (fast)
exarp_update_task(task_id="T-...", new_status="In Progress")

# Option 2: Use MCP (full features)
{"action": "claim", "task_id": "T-..."}
{"action": "start_run", "task_id": "T-...", "summary": "Starting work on..."}
```

## During Development

### Build & Test Loop

```bash
# Rust (in agents/backend/)
cargo build -p <crate>    # Build specific crate
cargo test -p <crate>     # Test specific crate
cargo fmt && cargo clippy # Lint

# Or use Make
make build                # Full build
make test                 # Run tests
make lint                 # Run linters
```

### Progress Tracking

```bash
# Add progress
{"action": "add_progress", "run_id": "R-...", "summary": "Completed...", "files": "src/file.rs"}

# Add note
{"action": "add_comment", "task_id": "T-...", "comment_type": "note", "content": "Blocked on..."}
```

## Completing Work

```bash
# 1. Add result comment
{"action": "add_comment", "task_id": "T-...", "comment_type": "result", "content": "Implemented... Verified with..."}

# 2. End run
{"action": "end_run", "run_id": "R-...", "summary": "Task complete"}

# 3. Mark done
exarp_update_task(task_id="T-...", new_status="Done")

# 4. Check for follow-ups
/followup

# 5. Create follow-up tasks if needed
{"action": "create", "name": "Verify...", "priority": "medium", "dependencies": "T-..."}
```

## End of Day

```bash
# Review what was done
/tasks                    # See completed work

# Check project health
/scorecard
/health

# Create handoff if needed (for long-running work)
{"action": "handoff", "sub_action": "end", "summary": "Progress on T-...: completed X, blocked on Y, next step Z"}
```

## Quick Reference

### Task Status Flow
```
Todo → In Progress → Review → Done
```

### Essential Commands
- `/prime` - Start of day/session
- `/tasks` - View tasks
- `/scorecard` - Project health
- `/health` - Run checks
- `/followup` - Get suggestions after completing work

### Essential Tools
- `exarp_tasks` - Quick task list
- `exarp_update_task` - Update status
- `exarp_prime` - Prime session
- `task_workflow` - Full task management
- `session` - Context management
- `report` - Scorecards and overviews

## Tips

1. **Always prime first** - Gets you context and suggestions
2. **Track everything** - Use tasks for all non-trivial work
3. **Add results** - Document what you did and how you verified it
4. **Follow up** - Check for follow-up tasks to maintain momentum
5. **Use handoffs** - For work spanning multiple sessions
