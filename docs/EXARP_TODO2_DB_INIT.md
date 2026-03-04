# Exarp-go Todo2 database initialization

exarp-go uses SQLite (`.todo2/todo2.db`) for task_workflow and related tools. If you see **"no such table: tasks"**, the DB has not been created or migrated yet.

## Built-in initialization (recommended)

exarp-go **does** initialize the database when you run the **CLI** from the project root. Any `exarp-go task` subcommand (and `-tool` mode) calls `initializeDatabase()` first, which creates the DB file and runs migrations. Then **`exarp-go task sync`** syncs SQLite ↔ JSON, so the first run will **create the DB and populate it from `.todo2/state.todo2.json`**.

From this project root, with **exarp-go** on your PATH and **`EXARP_MIGRATIONS_DIR`** pointing at the exarp-go repo’s migrations:

```bash
export PROJECT_ROOT="$(pwd)"
export EXARP_MIGRATIONS_DIR="/path/to/exarp-go/migrations"   # or e.g. ../exarp-go/migrations
exarp-go task sync
```

If you use the wrapper script (see below), it sets these for you. After that, **restart Cursor** (or reload MCP) so the exarp-go server uses the new DB.

**Note:** When exarp-go runs as an **MCP server** (e.g. from Cursor), it does *not* call `initializeDatabase()`, so the DB must already exist (created by a prior CLI run or migrate).

## What was done in this repo

1. **`.cursor/mcp.json`** – `EXARP_MIGRATIONS_DIR` was set to your exarp-go repo’s `migrations` folder so that when the DB is created (via CLI or migrate), the correct schema is used.
2. **`scripts/init_exarp_todo2_db.sh`** – Runs **`exarp-go task sync`** with `PROJECT_ROOT` and `EXARP_MIGRATIONS_DIR` set so one command creates the DB and populates it from JSON. Use this if you prefer a script over running the CLI directly.

## Initialize the DB (one-time)

**Option A – CLI (built-in)**
From this project root:

```bash
export PROJECT_ROOT="$(pwd)"
export EXARP_MIGRATIONS_DIR="/home/david/exarp-go/migrations"   # adjust if your path differs
exarp-go task sync
```

**Option B – Script**
If exarp-go is on PATH or at `EXARP_GO_ROOT` / `../exarp-go`, the script uses the same resolution as the MCP runner (global install, then working-dir fallback):

```bash
./scripts/init_exarp_todo2_db.sh
```

Then **restart Cursor** (or reload MCP). After that, `task_workflow` (create/list/update) and other task tools should work.

**Portable runner**: `scripts/run_exarp_go.sh` (used by MCP and by the init script) prefers the exarp-go working-dir build when run from inside the exarp-go repo, otherwise uses the global `exarp-go` and falls back to `EXARP_GO_ROOT` or `../exarp-go`. See [PORTABLE_BUILD_AND_RUNNER.md](PORTABLE_BUILD_AND_RUNNER.md).

## Separate migrate command (optional)

The **`cmd/migrate`** binary in exarp-go does the same thing (init DB + load JSON) and is used in CI or when you want an explicit “migrate” step. For local one-time setup, **`exarp-go task sync`** is enough.

## References

- exarp-go: `internal/cli/cli.go` — `initializeDatabase()` is called before `task` / `-tool` / etc.
- exarp-go: `internal/tools/todo2_utils.go` — `SyncTodo2Tasks` (SQLite ↔ JSON).
- exarp-go: `docs/archive/migration-planning/SQLITE_MIGRATION_PLAN.md`
- exarp-go: `cmd/migrate/main.go` (alternative: init + load JSON)
