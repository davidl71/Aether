Load canonical AI context (AGENTS.md, CLAUDE.md). Use before starting a task so the
assistant follows project guidelines.

READ AGENTS.md
READ CLAUDE.md
READ ARCHITECTURE.md

If the user wants suggested next tasks or handoff alert from exarp-go, prefer the local CLI wrapper and run `./scripts/run_exarp_go.sh -tool session -args '{"action":"prime","include_tasks":true,"include_hints":true}' -json -quiet`. See docs/EXARP_GO_CURSOR_CLAUDE_OPENCODE.md.
