# Todo2 shared table review

How to identify Todo2 tasks that are **not** in the shared TODO table and how to align or triage them.

## What is the “shared TODO table”?

The **shared TODO table** is the canonical list of tasks that your team or process treats as the single source of truth—for example a specific Todo2 view, a spreadsheet, or a doc that is kept in sync with Todo2. Tasks that exist only in Todo2 (or in a different view) and not in that table are “not in shared table”; the exarp backlog report may call out a count (e.g. “191 Todo2 tasks that are not in shared TODO table”).

## How to list or export Todo2 tasks

1. **Exarp report (backlog):** From the repo root run:
   ```bash
   just exarp-backlog
   ```
   or `./scripts/run_exarp_go_tool.sh report`
   This prints task counts, completion %, and next actions; it also writes `docs/PROJECT_OVERVIEW.md`.

2. **Todo2 MCP / CLI:** If you use exarp-go MCP or a Todo2 CLI, use the tool that lists or exports tasks (e.g. by status, source, or “not in shared table”). The exact command depends on your Todo2 integration.

3. **Manual export:** From your Todo2 app or API, export the full task list (e.g. CSV or JSON) so you can filter by “shared table” or equivalent flag.

## How to compare against the shared table

- If the shared table is a **document or spreadsheet**, diff or join by task ID (or title) to find tasks that appear in Todo2 but not in the table.
- If exarp or Todo2 exposes a **“in_shared_table”** (or similar) field, filter on that to list tasks not in the table.
- Use the **exarp report** next-actions list and task counts as a starting point; then in Todo2, filter or tag tasks that you consider “shared” vs “local/unsynced” and reconcile.

## Recommended actions

- **Move to shared table:** For tasks that should be canonical, add them to the shared table (or set the sync flag so they are included in the next export).
- **Close as duplicate:** If a Todo2 task duplicates an item already in the shared table, close the Todo2 task and point to the shared entry.
- **Tag as out-of-scope:** If a task is obsolete or out of scope, tag it (e.g. “out-of-scope” or “wont-do”) and optionally close it so it no longer counts as pending.
- **Batch review:** For large counts (e.g. 191), run the report periodically and process in batches (e.g. by priority or creation date).

There is no requirement to change all such tasks in one go; this doc defines a repeatable process for ongoing alignment.
