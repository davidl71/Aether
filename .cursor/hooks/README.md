# Cursor hooks

Hooks in this directory run at specific Cursor lifecycle events (e.g. when a new Composer conversation starts). Cursor invokes them with **cwd = project root** and expects a defined JSON output.

## session-prime.sh (sessionStart)

- **When:** New Composer conversation is created.
- **What:** Runs exarp-go **session prime** for this project via `scripts/run_exarp_go.sh`, then formats the result as `additional_context`.
- **Output:** JSON `{ "additional_context": "<string>", "continue": true }`. Cursor injects `additional_context` into the new conversation so the AI sees task summary, suggested next, and handoff hints without you asking.
- **Requires:** exarp-go on PATH or EXARP_GO_ROOT, `jq` (optional but recommended). If exarp-go is missing, the hook still returns valid JSON with a short message so the session is created.

**Note:** Cursor must be configured to use project hooks (see Cursor docs for session start / context hooks). The script name may need to match Cursor’s expected hook name (e.g. `sessionStart` or `session-prime` depending on Cursor version).

## Todo2 mirror (not a separate hook)

Session prime reads current tasks from exarp-go. This project does **not** use a `state.todo2.json` mirror or routine **`task sync`**; canonical tasks are in `.todo2/todo2.db`. Cheatsheet: `.cursor/skills/aether-todo2-exarp/SKILL.md`.

## Reference

- exarp-go (same pattern): `.cursor/hooks/session-prime.sh` in the exarp-go repo.
- This repo: `docs/archive/EXARP_GO_SCRIPTS_AND_PATTERNS.md`, `docs/archive/SUBAGENTS_REFERENCE.md`.
