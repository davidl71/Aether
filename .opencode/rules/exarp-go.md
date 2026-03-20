# exarp-go Task Tracking for OpenCode

## Rule

You MUST use exarp-go to track all work tasks. This is non-negotiable.

## Workflow

**For EVERY user request, you MUST:**

1. **Create task(s) FIRST** using `exarp-go_task_workflow` with `action=create` before doing any work
2. **Update task status** as work progresses: `exarp-update-task` → `In Progress`, `Review`, `Done`
3. **Add result comment** before marking `Done`: use `task_workflow` `action=add_comment` with `comment_type=result`
4. **Refine remaining tasks** after completing any task: assess, create, update, or delete as needed

## exarp-go Tools Available

| Tool | Purpose |
|------|---------|
| `exarp-go_task_workflow` | Create, list, update, delete tasks |
| `exarp-go_tasks` | Quick task list (plugin tool, faster) |
| `exarp-go_update_task` | Update task status |
| `exarp-go_session` | Get suggested next tasks |
| `exarp-go_followup` | Get follow-up suggestions |
| `task_workflow` (MCP) | Full task management via exarp-go MCP |

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
