# Aether Todo2 + exarp-go cheatsheet

Use this repo’s **Todo2** (`.todo2/todo2.db`) and **exarp-go** via the portable wrapper. This is **not** the same as editing the exarp-go *source* repo: here **`PROJECT_ROOT` is this project (Aether root)**.

## PROJECT_ROOT and canonical store

- **MCP / portable runner:** command `scripts/run_exarp_go.sh` with `env.PROJECT_ROOT` = workspace root (see `.cursor/mcp.json`).
- **Canonical tasks:** `.todo2/todo2.db` (SQLite). `.todo2/state.todo2.json` is a mirror.
- After any task change that might skip sync (bulk CLI, raw DB, or tooling quirks), run:

```bash
cd "$(git rev-parse --show-toplevel)"
./scripts/run_exarp_go.sh task sync
```

## One-shot CLI (from repo root)

Always pass a subcommand — running the wrapper with **no** args starts stdio MCP mode (terminal looks “hung”).

```bash
./scripts/run_exarp_go.sh task list --status Todo
./scripts/run_exarp_go.sh task list --status Review
./scripts/run_exarp_go.sh task show T-…
./scripts/run_exarp_go.sh task update T-… --new-status Done
```

### Bulk: move every Review task to Done

```bash
./scripts/run_exarp_go.sh task update --status Review --new-status Done
```

Then run `task sync` so JSON-backed views match.

### Task comments and dependencies (JSON tool)

When the convenience CLI rejects `--dependencies` or complex args, use the wrapper’s tool mode:

```bash
./scripts/run_exarp_go.sh -tool task_workflow -args '<json>'
```

Examples:

```json
{"action":"add_comment","task_id":"T-…","comment_type":"result","content":"…"}
```

```json
{"action":"update","task_id":"T-…","dependencies":["T-…"]}
```

Comment types: `result`, `note`, `research_with_links`, `manualsetup`.

Prefer **`output_format":"json"`** (and **`compact":true`** when supported) on MCP calls for parsing.

## Cargo.lock discipline (agents/backend)

For parallel or multi-crate manifest edits:

1. Serialize substantive `Cargo.toml` / dependency work where possible.
2. Run **`cd agents/backend && cargo check`** or **`cargo test`** once after the batch (not after every tiny edit).
3. Commit **one** reconciled **`Cargo.lock`**; on conflict regenerate with cargo, do not hand-merge.

## TUI helpers (layout / hints)

- **Workspace chrome:** shared helpers live in `agents/backend/services/tui_service/src/ui/workspace_layout.rs` (`workspace_outer_rows`, `workspace_banner`, `split_workspace_banner`, `paint_workspace_top_banner`). Prefer this module over duplicating banner rows in `ui/mod.rs`.
- **Naming:** the global footer hint strip is `render_hint_bar` in `ui/mod.rs` (uses `discoverability::context_hints_for`). Chart symbol context lines in `ui/charts.rs` should **not** reuse that name long term—rename to something like `render_chart_symbol_hint` to avoid confusion.

## Related rules and docs

- `.cursor/rules/aether-todo2-exarp-cheatsheet.mdc` — short AI-facing summary.
- `.cursor/rules/project-automation.mdc`, `automation-tool-suggestions.mdc` — MCP patterns, `workingDirectory`.
- `AGENTS.md` — exarp workflow; **Learned User Preferences** (task sync).
- `scripts/run_exarp_go_tool.sh` — sanitized `-tool` / `-args` invocations.
