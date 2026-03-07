---
description: Create a session handoff note
argument-hint: "<summary of what was done>"
---

Call the exarp-go `session` MCP tool with `action=handoff`, `summary=$ARGUMENTS`, `include_tasks=true`, `include_git_status=true`.

If $ARGUMENTS is empty, ask the user: "What should I include in the handoff summary?"

Confirm the handoff was saved and show the key fields (summary, next steps, git status).

**Optional (export for sync to another server):** Add `include_point_in_time_snapshot=true` and `export_latest=true`. To get a file to copy, run: `./scripts/run_exarp_go.sh -tool session -args '{"action":"handoff",...}' -json -quiet > build/handoff-export/session-handoff.json`. See docs/SESSION_HANDOFF_EXPORT.md and docs/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md.
