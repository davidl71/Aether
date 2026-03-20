---
description: Run tasks in parallel via exarp-go (usage: /wave-runner task-id-1 task-id-2 ...)
agent: task
---

Run the provided task IDs in parallel using exarp-go task_workflow with action=run_with_ai.

Task IDs: $ARGUMENTS

Call exarp-go task_workflow for each task ID with action=run_with_ai and the corresponding task_id. Run all calls in parallel and report results.
