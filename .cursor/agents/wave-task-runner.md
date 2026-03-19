# Wave Task Runner (subagent)

**Role:** Run one instance per task; implement or complete that task and return a short summary.

**Used by:** [.cursor/plans/parallel-execution-subagents.plan.md](../plans/parallel-execution-subagents.plan.md) for Wave 0, Wave 1, and Wave 3.

## Inputs

- **task_id** — Todo2 task ID (e.g. `T-1773513131667911000`).
- **Context** — Plan file [.cursor/plans/ib_box_spread_full_universal.plan.md](../plans/ib_box_spread_full_universal.plan.md) and the task description from Todo2 or the plan.

## Behavior

1. Load the task description for the given `task_id` (from Todo2 or the main plan).
2. Implement or complete the task (code, docs, config, or decision as specified).
3. Mark the task **Done** in Todo2 (via exarp-go `task_workflow` update or equivalent).
4. Return a short summary (what was done, any blockers or follow-ups).

## Invocation

In Cursor, run one subagent (e.g. `mcp_task` or equivalent) per task ID in a single message so they run in parallel. Pass `task_id` and plan context. No separate binary; this document describes the contract for “wave-task-runner” referenced in the parallel-execution plan.
