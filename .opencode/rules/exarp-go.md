# exarp-go Task Tracking for OpenCode

## Rule

You MUST use exarp-go to track all work tasks. This is non-negotiable.

## Workflow

**For EVERY user request, you MUST:**

1. **Create task(s) FIRST** using `exarp-go_task_workflow` with `action=create` before doing any work
2. **Update task status** as work progresses: `exarp_update_task` → `In Progress`, `Review`, `Done`
3. **Add result comment** before marking `Done`: use `exarp-go_task_workflow` `action=add_comment` with `comment_type=result`
4. **Refine remaining tasks** after completing any task: assess, create, update, or delete as needed

## Quick Reference: Plugin Tools (Fast, No MCP Round-Trip)

| Tool | Purpose | Example |
|------|---------|---------|
| `exarp_tasks` | Quick task list with status filter | `exarp_tasks(status="Todo")` |
| `exarp_update_task` | Update task status | `exarp_update_task(task_id="T-123", new_status="Done")` |
| `exarp_prime` | Full session prime with tasks, hints, handoffs | `exarp_prime()` |
| `exarp_config` | Get/set config values | `exarp_config(action="show")` |
| `exarp_followup` | Get/create follow-up tasks | `exarp_followup(action="suggest")` |

## Slash Commands

| Command | Action |
|---------|--------|
| `/tasks` | List current tasks |
| `/prime` | Prime session with full context |
| `/scorecard` | Generate project scorecard |
| `/health` | Run project health checks |
| `/config` | Show exarp-go configuration |
| `/followup` | Get follow-up task suggestions |

## MCP Tools (Advanced Operations)

| Tool | Purpose |
|------|---------|
| `task_workflow` | Full task management (create, update, delete, sync, batch) |
| `session` | Prime, handoff, prompts, assignee |
| `report` | Overview, scorecard, briefing, plan |
| `health` | Docs, git, cicd, tools, database |
| `git_tools` | Commits, branches, tasks, diff, merge |
| `automation` | Daily, nightly, sprint, discover |

## Task Lifecycle

```
[Todo] → [In Progress] → [Review] → [Done]
              ↓
         (add result comment before Review)
```

## Complexity Assessment

- **Multiple tasks (2-5)**: Research needed, multi-component, parallel opportunities, 2+ hours
- **Single task**: Routine change, obvious solution, <1 hour, atomic

## Mandatory Refinement

After marking a task `Done`, you MUST immediately:
1. Review remaining incomplete tasks
2. Create new tasks for follow-up work discovered
3. Update tasks whose scope changed
4. Delete obsolete tasks

## Scope Control

Stay within defined task scope. Ask for clarification if ambiguous. Do not expand work beyond acceptance criteria.

## No Exceptions

- Every user message → create tasks first
- Every code change → track via exarp-go
- Every completion → add result comment → refine remaining tasks

## Best Practices

### Starting Work
```
# Prime session first
exarp_prime()

# Or use slash command
/prime
```

### Creating Tasks
```
# Single task
exarp-go_task_workflow(action="create", name="Fix bug", priority="high")

# Batch create
exarp-go_task_workflow(action="create", tasks='[{"name":"Task A","priority":"high"},...]')
```

### Updating Status
```
# Use plugin tool (fast)
exarp_update_task(task_id="T-123", new_status="In Progress")

# Or MCP tool
exarp-go_task_workflow(action="update", task_id="T-123", new_status="Done")
```

### Follow-up Tasks
```
# After completing work
exarp_followup(action="suggest")

# Then create them
exarp-go_task_workflow(action="create", tasks='[...]')
```

## Configuration

```
# View config
exarp_config(action="show")

# Get specific value
exarp_config(action="get", key="timeouts.task_lock_lease")

# Set value
exarp_config(action="set", key="timeouts.task_lock_lease", value="30m")
```

## Resources

- **stdio://tools** — Full tool catalog
- **stdio://tasks** — Task list
- **stdio://suggested-tasks** — Dependency-ready suggestions
- **stdio://cursor/skills** — Available skills

## Skills

| Skill | Location | Purpose |
|-------|----------|---------|
| task-workflow | `~/.claude/skills/task-workflow/SKILL.md` | Todo2 task management |
| use-exarp-tools | `~/.claude/skills/use-exarp-tools/SKILL.md` | When/how to use exarp-go |
| report-scorecard | `~/.claude/skills/report-scorecard/SKILL.md` | Project overview/scorecard |
| session-handoff | `~/.claude/skills/session-handoff/SKILL.md` | Session handoff workflows |
| task-cleanup | `~/.claude/skills/task-cleanup/SKILL.md` | Bulk remove tasks |

See `.cursor/skills/README.md` for project-specific skills.
