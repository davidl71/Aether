---
description: Create a session handoff note
argument-hint: "<summary of what was done>"
---

Call the exarp-go `session` MCP tool with `action=handoff`, `summary=$ARGUMENTS`, `include_tasks=true`, `include_git_status=true`.

If $ARGUMENTS is empty, ask the user: "What should I include in the handoff summary?"

Confirm the handoff was saved and show the key fields (summary, next steps, git status).
