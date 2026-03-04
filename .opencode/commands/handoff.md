Create a session handoff note using the exarp-go session MCP tool.

Call `exarp-go-session` with `action=handoff`, `summary=$ARGUMENTS`, `include_tasks=true`, `include_git_status=true`.

If $ARGUMENTS is empty, ask: "What should I include in the handoff summary?"

Confirm the handoff was saved and show the key fields.
