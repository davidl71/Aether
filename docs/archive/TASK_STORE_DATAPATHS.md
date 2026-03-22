# Task Store Datapaths

This document maps the local task store files under `.todo2/`, the repo-side commands that
touch them, and the underlying `exarp-go` functions that read, write, or synchronize them.

## Task store files

All task-store state for this repo lives under [`/.todo2`](/Users/davidl/Projects/Trading/Aether/.todo2):

| Path | Role |
|------|------|
| [`.todo2/todo2.db`](/Users/davidl/Projects/Trading/Aether/.todo2/todo2.db) | Primary SQLite task store |
| [`.todo2/todo2.db-wal`](/Users/davidl/Projects/Trading/Aether/.todo2/todo2.db-wal) | SQLite WAL sidecar |
| [`.todo2/todo2.db-shm`](/Users/davidl/Projects/Trading/Aether/.todo2/todo2.db-shm) | SQLite shared-memory sidecar |
| [`.todo2/state.todo2.json`](/Users/davidl/Projects/Trading/Aether/.todo2/state.todo2.json) | JSON snapshot / fallback store |
| [`.todo2/handoffs.json`](/Users/davidl/Projects/Trading/Aether/.todo2/handoffs.json) | Session handoff store |

## Datapath overview

```text
repo command / editor command
  -> scripts/run_exarp_go.sh or scripts/run_exarp_go_tool.sh
  -> exarp-go CLI / MCP tool
  -> project root resolution (PROJECT_ROOT)
  -> database init / task store selection
  -> SQLite (.todo2/todo2.db) first
  -> JSON sync / fallback (.todo2/state.todo2.json)
  -> handoff JSON (.todo2/handoffs.json) for session notes
```

## Repo-side entrypoints

These are the commands in this repo that route into the task store:

| Command / file | What it does | Store effect |
|------|------|------|
| [scripts/run_exarp_go.sh](/Users/davidl/Projects/Trading/Aether/scripts/run_exarp_go.sh) | Resolves `exarp-go`, exports `PROJECT_ROOT`, optionally sets `EXARP_MIGRATIONS_DIR`, then `exec`s the binary | Indirect: routes all exarp-go access to the right repo |
| [scripts/run_exarp_go_tool.sh](/Users/davidl/Projects/Trading/Aether/scripts/run_exarp_go_tool.sh) | Convenience wrapper for `-tool <name> -args <json>` | Indirect |
| `exarp-go task sync` | Initializes DB if needed, then syncs SQLite and JSON | Reads/writes `todo2.db`, `state.todo2.json` |
| `exarp-go task list` | Lists tasks via task workflow / task store | Reads `todo2.db` or `state.todo2.json` fallback |
| `exarp-go task create|update|delete|show` | CRUD over tasks | Reads/writes `todo2.db`, then sync/fallback JSON depending on path |
| `exarp-go -tool task_workflow -args '{"action":"sync"}'` | Tool-mode task sync | Reads/writes `todo2.db`, `state.todo2.json` |
| `exarp-go -tool task_workflow -args '{"action":"sync","sub_action":"list"}'` | Tool-mode list after sync | Reads store, may sync first |
| `exarp-go -tool session -args '{"action":"prime",...}'` | Session prime | Reads tasks and handoff alert state |
| `exarp-go -tool session -args '{"action":"handoff",...}'` | Create/list/resume/export handoff | Reads/writes `handoffs.json`; may read tasks for snapshot/journal |
| (removed) `.claude/commands/tasks.md` | Previously Claude command for task_workflow | — |
| (removed) `.opencode/commands/tasks.md` | Previously OpenCode wrapper for task_workflow | — |

## Repo-side shell functions

These local functions do not mutate the task store themselves, but they decide which exarp-go
process gets the request and which repo root it operates on:

| File | Function | Role |
|------|------|------|
| [scripts/run_exarp_go.sh](/Users/davidl/Projects/Trading/Aether/scripts/run_exarp_go.sh) | `sanitize_go_env()` | Clears stale `GOROOT`; no task-store IO |
| [scripts/run_exarp_go.sh](/Users/davidl/Projects/Trading/Aether/scripts/run_exarp_go.sh) | `resolve_exarp_go()` | Finds the exarp-go binary and sets migrations path |
| (removed) `scripts/oh-my-zsh-exarp-plugin/` | Previously shell plugin for exarp-go | Use `scripts/run_exarp_go.sh` or Cursor MCP |

## exarp-go functions behind the task store

The actual store logic lives in the sibling `exarp-go` repo. These are the key functions touching
the `.todo2/` datapaths:

### CLI / command dispatch

| exarp-go file | Function | Effect |
|------|------|------|
| `internal/cli/cli.go` | `initializeDatabase()` | Finds project root and initializes DB before CLI/tool execution |
| `internal/cli/cli.go` | `EnsureConfigAndDatabase(projectRoot)` | Loads config and initializes DB using centralized/legacy config |
| `internal/cli/task.go` | `handleTaskCommand(...)` | Dispatches `task list|create|update|delete|show|sync` |
| `internal/cli/task.go` | `handleTaskSync(...)` | Runs `task_workflow action=sync` |
| `internal/cli/cli.go` | `handleSessionCommand(...)` | Dispatches session subcommands such as handoff list |

### Task storage and sync

| exarp-go file | Function | Datapath touched |
|------|------|------|
| `internal/tools/todo2_utils.go` | `LoadTodo2Tasks(projectRoot)` | DB-first read, JSON fallback |
| `internal/tools/todo2_utils.go` | `loadTodo2TasksFromJSON(projectRoot)` | Reads `.todo2/state.todo2.json` |
| `internal/tools/todo2_utils.go` | `SaveTodo2Tasks(projectRoot, tasks)` | Saves DB first, then writes JSON copy |
| `internal/tools/todo2_utils.go` | `saveTodo2TasksToJSON(projectRoot, tasks)` | Writes `.todo2/state.todo2.json` |
| `internal/tools/todo2_utils.go` | `SyncTodo2Tasks(projectRoot)` | Merges DB and JSON, DB takes precedence, writes both |
| `internal/tools/task_store.go` | `NewDefaultTaskStore(projectRoot)` | Creates DB-first / JSON-fallback task store |
| `internal/tools/task_store.go` | `dbOrFileStore.GetTask/ListTasks/CreateTask/UpdateTask/DeleteTask` | CRUD over DB first, JSON fallback as needed |
| `internal/tools/task_workflow_common.go` | `getTaskStore(ctx)` | Resolves the store used by `task_workflow` handlers |

### Session / handoff storage

| exarp-go file | Function | Datapath touched |
|------|------|------|
| `internal/tools/session_handoff.go` | `handleSessionHandoff(...)` | Dispatches handoff actions |
| `internal/tools/session_handoff.go` | `handleSessionEnd(...)` | Builds handoff payload, may include tasks/snapshot, then saves |
| `internal/tools/session_helpers_handoff.go` | `saveHandoff(projectRoot, handoff)` | Appends to `.todo2/handoffs.json` |
| `internal/tools/session_helpers_handoff.go` | `updateHandoffStatus(...)` | Rewrites `.todo2/handoffs.json` statuses |
| `internal/tools/session_helpers_handoff.go` | `deleteHandoffs(...)` | Removes entries from `.todo2/handoffs.json` |
| `internal/tools/todo2_utils.go` | `GetSuggestedNextTasks(projectRoot, limit)` | Reads tasks to compute suggested next work |
| `internal/tools/session.go` | `handleSessionPrime(...)` | Reads tasks and handoff state for prime output |

## Command-to-function map

### Task sync and list

| User command | Repo wrapper | exarp-go function path | Files touched |
|------|------|------|------|
| `exarp-go task sync` | none or [run_exarp_go.sh](/Users/davidl/Projects/Trading/Aether/scripts/run_exarp_go.sh) | `initializeDatabase()` -> `handleTaskCommand()` -> `handleTaskSync()` -> `task_workflow(action=sync)` -> `SyncTodo2Tasks()` | `todo2.db`, `state.todo2.json` |
| `exarp-go task list` | same | `initializeDatabase()` -> `handleTaskCommand()` -> `handleTaskListParsed()` -> `task_workflow(action=sync, sub_action=list)` -> `getTaskStore()` / `LoadTodo2Tasks()` | usually `todo2.db`, fallback JSON |
| `-tool task_workflow {"action":"sync","sub_action":"list"}` | [run_exarp_go_tool.sh](/Users/davidl/Projects/Trading/Aether/scripts/run_exarp_go_tool.sh) or MCP | `initializeDatabase()` -> `server.CallTool("task_workflow", ...)` -> `getTaskStore()` / `SyncTodo2Tasks()` | `todo2.db`, `state.todo2.json` |

### Task CRUD

| User command | exarp-go function path | Files touched |
|------|------|------|
| `exarp-go task create ...` | `handleTaskCommand()` -> `handleTaskCreateParsed()` -> `task_workflow(action=create)` -> `dbOrFileStore.CreateTask()` | DB first; JSON fallback and/or later sync |
| `exarp-go task update ...` | `handleTaskCommand()` -> `handleTaskUpdateParsed()` -> `task_workflow(action=update|approve)` -> `dbOrFileStore.UpdateTask()` -> `SyncTodo2Tasks()` | `todo2.db`, `state.todo2.json` |
| `exarp-go task delete <id>` | `handleTaskCommand()` -> `handleTaskDelete()` -> `task_workflow(action=delete)` -> `dbOrFileStore.DeleteTask()` | `todo2.db`, fallback JSON |
| `exarp-go task show <id>` | `handleTaskCommand()` -> `handleTaskShow()` -> `loadSingleTask()` -> store read | read-only |

### Session and handoff

| User command | exarp-go function path | Files touched |
|------|------|------|
| `-tool session {"action":"prime"}` | `handleSessionPrime()` -> `GetSuggestedNextTasks()` + handoff alert helpers | reads tasks, reads `handoffs.json` |
| `-tool session {"action":"handoff","summary":"..."}` | `handleSessionHandoff()` -> `handleSessionEnd()` -> `saveHandoff()` | writes `handoffs.json`; may read tasks for journal/snapshot |
| `exarp-go session handoffs` | `handleSessionCommand()` -> `runSessionHandoffList()` -> session handoff list handler | reads `handoffs.json` |
| `-tool session {"action":"handoff","sub_action":"export"}` | `handleSessionHandoff()` -> `handleSessionExport()` | reads `handoffs.json`, may emit snapshot/export |

## Important behavior notes

- `PROJECT_ROOT` determines which `.todo2/` directory exarp-go uses.
- CLI mode initializes the DB before task/tool execution.
- MCP server mode does not create the DB automatically; initialize once with CLI first.
- Task reads are DB-first with JSON fallback.
- Task writes prefer DB and then keep JSON synchronized.
- Handoffs are stored separately in `handoffs.json`, not in SQLite.
- Do not edit [state.todo2.json](/Users/davidl/Projects/Trading/Aether/.todo2/state.todo2.json) manually; use exarp-go.

## References

- [scripts/run_exarp_go.sh](/Users/davidl/Projects/Trading/Aether/scripts/run_exarp_go.sh)
- [scripts/run_exarp_go_tool.sh](/Users/davidl/Projects/Trading/Aether/scripts/run_exarp_go_tool.sh)
- [EXARP_TODO2_DB_INIT.md](/Users/davidl/Projects/Trading/Aether/docs/EXARP_TODO2_DB_INIT.md)
- [EXARP_GO_SIBLING_CI_AND_AI_PATTERNS.md](/Users/davidl/Projects/Trading/Aether/docs/EXARP_GO_SIBLING_CI_AND_AI_PATTERNS.md)
