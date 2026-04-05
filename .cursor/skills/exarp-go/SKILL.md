# exarp-go Project Automation

Complete guide for task management, project health, and automation with the **exarp-go binary / MCP server**.

## PROJECT_ROOT — which repo? (critical)

exarp-go is **one tool**; **`PROJECT_ROOT` is whichever project’s `.todo2/` you mean**:

| You are working in… | Set `PROJECT_ROOT` to… | Tasks / docs come from… |
|---------------------|---------------------------|-------------------------|
| **This app repo (Aether)** | **Aether repository root** | `.todo2/` under Aether |
| **exarp-go source repo** | **exarp-go repository root** | `.todo2/` under exarp-go |

- Cursor MCP for **Aether** should point `run_exarp_go.sh` at **Aether** root so Todo2 matches this codebase.
- When **developing exarp-go itself**, `PROJECT_ROOT` must be the **exarp-go** clone (contains its `Makefile`, `cmd/server`, migrations).
- This skill file may live under Aether for editor discovery; **always match `PROJECT_ROOT` to the backlog you intend to use.**

**Aether-only quick ref:** `.cursor/skills/aether-todo2-exarp/SKILL.md` (bulk Review→Done, JSON `task_workflow`, Cargo.lock, TUI layout notes; no `task sync` / JSON mirror).

If `PROJECT_ROOT` is wrong, exarp-go reads the wrong `.todo2/` and wrong `docs/` for health/scorecard.

## When to Use

- **Starting work**: Prime session, view tasks, get suggestions
- **Task lifecycle**: Create, update, complete tasks with proper workflow
- **Project health**: Run scorecards, health checks, documentation validation
- **Session management**: Handoffs, context preservation

## Quick Start

```bash
# Prime session (get tasks, hints, suggestions)
/prime

# List current tasks
/tasks

# Check project health
/health

# View scorecard
/scorecard
```

## Plugin Tools (Fastest - No MCP Round-Trip)

These tools are built into the OpenCode plugin and execute instantly:

| Tool | Use When | Example |
|------|----------|---------|
| `exarp_tasks` | Quick task list | `exarp_tasks(status="Todo")` |
| `exarp_update_task` | Mark task done | `exarp_update_task(task_id="T-123", new_status="Done")` |
| `exarp_prime` | Start session | `exarp_prime()` |
| `exarp_config` | View/change config | `exarp_config(action="show")` |
| `exarp_followup` | Get next steps | `exarp_followup(action="suggest")` |

## MCP Tools (Full Feature Set)

Use these for advanced operations:

### Task Management (`task_workflow`)

```json
// List tasks
{"action": "list", "status": "Todo", "order": "execution"}

// Create task
{"action": "create", "name": "Fix bug", "priority": "high", "tags": "bug,rust"}

// Update status
{"action": "update", "task_id": "T-123", "new_status": "In Progress"}

// Add result comment
{"action": "add_comment", "task_id": "T-123", "comment_type": "result", "content": "Fixed in commit abc123"}

// Batch operations
{"action": "approve", "status": "Review", "new_status": "Done", "task_ids": "T-1,T-2,T-3"}
```

### Session Management (`session`)

```json
// Prime session
{"action": "prime", "include_hints": true, "include_tasks": true}

// Create handoff
{"action": "handoff", "sub_action": "end", "summary": "Completed T-123, blocked on API response"}

// Get suggested next tasks
{"action": "prime", "include_tasks": true}
```

### Reporting (`report`)

```json
// Project overview
{"action": "overview", "include_tasks": true}

// Scorecard
{"action": "scorecard"}

// Briefing
{"action": "briefing"}
```

### Health Checks (`health`)

```json
// Documentation health
{"action": "docs"}

// Git status
{"action": "git"}

// Full check
{"action": "tools"}
```

## Task Lifecycle Workflow

### Starting Work

1. **Prime session** to get context:
   ```
   /prime
   ```

2. **Review suggested tasks** from prime output

3. **Claim task** (for multi-agent):
   ```json
   {"action": "claim", "task_id": "T-123"}
   ```

4. **Start run**:
   ```json
   {"action": "start_run", "task_id": "T-123", "summary": "Implementing feature X"}
   ```

### During Work

- **Add progress**:
  ```json
  {"action": "add_progress", "run_id": "R-...", "summary": "Completed module Y", "files": "src/file.rs"}
  ```

- **Update task** if scope changes

### Completing Work

1. **Add result comment**:
   ```json
   {"action": "add_comment", "task_id": "T-123", "comment_type": "result", "content": "Implemented... Verified with..."}
   ```

2. **End run**:
   ```json
   {"action": "end_run", "run_id": "R-...", "summary": "Task complete"}
   ```

3. **Update status**:
   ```
   exarp_update_task(task_id="T-123", new_status="Done")
   ```

4. **Check follow-ups**:
   ```
   /followup
   ```

5. **Create follow-up tasks** as needed

## Slash Commands

| Command | Runs | Purpose |
|---------|------|---------|
| `/tasks` | `exarp_tasks` | List all tasks |
| `/prime` | `exarp_prime` | Prime session |
| `/scorecard` | `report scorecard` | Project health score |
| `/health` | `health tools` + `health docs` | Full health check |
| `/config` | `exarp_config show` | Show configuration |
| `/followup` | `exarp_followup suggest` | Get follow-up suggestions |

## Configuration

View current config:
```
exarp_config(action="show")
```

Common settings:
```
# Get value
exarp_config(action="get", key="timeouts.task_lock_lease")

# Set value
exarp_config(action="set", key="timeouts.task_lock_lease", value="30m")

# Reset to defaults
exarp_config(action="reset", key="all")

# View history
exarp_config(action="history")
```

## Multi-Agent Workflow

When multiple agents may work in parallel:

1. **Claim before starting**:
   ```json
   {"action": "claim", "task_id": "T-123", "lease_minutes": 60}
   ```

2. **Release when done**:
   ```json
   {"action": "release", "task_id": "T-123"}
   ```

3. **Check agent status**:
   ```json
   {"action": "agent_status"}
   ```

## Resources

- `stdio://tools` — Full tool catalog
- `stdio://tasks` — All tasks
- `stdio://suggested-tasks` — Dependency-ready suggestions
- `stdio://cursor/skills` — Available skills

## Best Practices

1. **Always prime first** — Get context before starting work
2. **Track everything** — Create tasks for non-trivial work
3. **Add results** — Document completion with verification
4. **Follow up** — Create follow-up tasks to maintain momentum
5. **Use plugin tools** — Faster for common operations
6. **Use MCP tools** — For advanced operations (batch, sync, etc.)

## Troubleshooting

**Task not found**: Check task ID format (T- followed by 16+ digits)

**MCP not responding**: Verify exarp-go is in PATH or `run_exarp_go.sh` works

**Config not persisting / wrong tasks**: Confirm `PROJECT_ROOT` is the **intended project root** (Aether vs exarp-go clone) for the backlog you are editing (see [PROJECT_ROOT — which repo?](#project_root--which-repo-critical))

**Session handoff missing**: Use `session` tool with `action=handoff` and `sub_action=end`
