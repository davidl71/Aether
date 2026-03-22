# Session handoff export for syncing to another server

Use **exarp-go session handoff** with task export to create a portable bundle you can copy to another machine or server.

## Exarp tool

**Tool:** `session`  
**Action:** `handoff` with `include_point_in_time_snapshot=true` and `export_latest=true`.

The handoff payload includes suggested next tasks, git status, and a point-in-time task snapshot (e.g. gzip+base64) so the other server can restore or sync Todo2 state.

## Create export bundle (CLI)

`build/` is gitignored; export files live under `build/handoff-export/` and are not committed. To regenerate:

```bash
mkdir -p build/handoff-export

# Handoff with task snapshot → write to file
./scripts/run_exarp_go.sh -tool session -args '{"action":"handoff","summary":"Handoff for sync to other server","include_tasks":true,"include_git_status":true,"include_point_in_time_snapshot":true,"export_latest":true}' -json -quiet > build/handoff-export/session-handoff.json

# Optional: copy Todo2 state and plan for simple restore on other server
cp .todo2/state.todo2.json build/handoff-export/
cp .cursor/plans/Aether.plan.md build/handoff-export/
```

## Sync to the other server

1. Copy `build/handoff-export/` to the other server (e.g. `scp -r build/handoff-export user@host:path/to/repo/build/`).
2. **Restore Todo2:**  
   `cp build/handoff-export/state.todo2.json .todo2/state.todo2.json`  
   (If the other side uses exarp-go SQLite, use exarp-go sync/import as needed.)
3. **Restore plan:**  
   `cp build/handoff-export/Aether.plan.md .cursor/plans/`
4. On the other server, run **session prime** or open the handoff JSON for `suggested_next` and the task snapshot.

## MCP usage

From Cursor/OpenCode, call the exarp-go **session** MCP tool with:

- `action`: `handoff`
- `summary`: optional note (e.g. "Handoff for sync to other server")
- `include_tasks`: `true`
- `include_git_status`: `true`
- `include_point_in_time_snapshot`: `true`
- `export_latest`: `true`

The response contains the handoff JSON (including the snapshot). To get a file to sync, use the CLI and redirect stdout as above.

## See also

- [EXARP_GO_OPENCORE_AI_CONTEXT_PATTERNS.md](EXARP_GO_OPENCORE_AI_CONTEXT_PATTERNS.md) — Session prime and handoff patterns
- `.opencode/commands/handoff.md` — Handoff command (session handoff with summary)
