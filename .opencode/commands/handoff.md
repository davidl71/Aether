Create a session handoff note using the local exarp-go CLI wrapper.

If $ARGUMENTS is empty, ask: "What should I include in the handoff summary?"

RUN ./scripts/run_exarp_go.sh -tool session -args '{"action":"handoff","summary":"$ARGUMENTS","include_tasks":true,"include_git_status":true}' -json -quiet

Confirm the handoff was saved and show the key fields.

**Optional (export for sync to another server):** Add `include_point_in_time_snapshot=true`, `export_latest=true`. To write a file, run from repo root: `./scripts/run_exarp_go.sh -tool session -args '{"action":"handoff","summary":"...","include_tasks":true,"include_point_in_time_snapshot":true,"export_latest":true}' -json -quiet > build/handoff-export/session-handoff.json`. See docs/SESSION_HANDOFF_EXPORT.md and docs/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md.
