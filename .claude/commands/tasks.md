---
description: List current tasks (default: Todo + In Progress)
argument-hint: "[status filter, e.g. 'Todo' or 'In Progress']"
---

Call the exarp-go `task_workflow` MCP tool with `action=sync`, `sub_action=list`.

If $ARGUMENTS is provided, filter by that status.

Display results as a readable table with ID, Priority, Status, and Name (truncated to 60 chars).
