# TUI / CLI / Config Patterns from exarp-go (Sibling Repo)

Learnings from the **exarp-go** repository (`/Users/davidl/Projects/mcp/exarp-go` or `EXARP_GO_ROOT`) for TUI, CLI, and configuration design. Use these patterns when evolving this repo’s Rust TUI, CLI, and backend config.

---

## 1. Configuration

### Runtime vs editing

- **Runtime:** Only one format is loaded (exarp-go: `.exarp/config.pb` protobuf binary). No YAML at runtime.
- **Editing:** Export to a human-friendly format (YAML/JSON), edit, then convert back. CLI: `config export yaml`, edit file, `config convert yaml protobuf`.

### Override layering

- **Base:** `.exarp/config.pb` (committed).
- **Local:** `.exarp/config.local.pb` (gitignored) merges on top; later file wins.
- **Merge:** Top-level keys replaced, nested keys deep-merged; repeated fields (e.g. tags) replaced not appended.

### CLI surface

- `config init` — create default config file.
- `config validate` — validate current config.
- `config show [yaml|json]` — display current config.
- `config export [yaml|json|protobuf]` — export for editing or backup.
- `config convert <from> <to>` — e.g. `yaml protobuf` after editing YAML.

### Parameter categories (exarp-go)

Grouped in config schema: `timeouts`, `thresholds`, `tasks`, `database`, `cloud`, etc. Each has documented defaults and types (durations, floats, ints). Single loader returns a struct; tools read from it instead of hard-coded constants.

**Apply here:** Prefer one runtime config format (e.g. TOML or a single JSON). Support override file (e.g. `config.local.toml` gitignored). Document defaults and use a single load path (e.g. `config::load()` merging base + local). Optional: `cli config show` / `config export` for inspection and editing workflow.

---

## 2. CLI

### Single binary, multiple modes

- **MCP server** (default when no CLI flags, stdin not a TTY): stdio JSON-RPC.
- **CLI:** Explicit subcommands and flags: `task`, `config`, `tui`, `session`, `-tool`, `-list`, `-args`, `-help`, etc.
- **Special modes:** `-serve :8080`, `-acp`, `-mcp-http :8081` parsed first and exit early.

### Mode detection

- **Explicit flags win:** If any `CLIFlags` (e.g. `-tool`, `task`, `config`, `tui`) appear in `os.Args`, run CLI.
- **TTY fallback:** If no flags, `DetectMode()`: stdin is a TTY → `ModeCLI`, else → MCP server. So scripts/pipes run MCP, interactive terminal can run CLI.
- **Git hooks:** When `GIT_HOOK=1` and no CLI flags, exit with message: use `exarp-go -tool <name> -args '<json>'` so stdin (refs) isn’t parsed as JSON-RPC.

### Tool invocation

- **Canonical:** `exarp-go -tool <tool_name> -args '<json>'`.
- **Convenience:** `exarp-go task list`, `exarp-go task create "Title" --priority high`, etc. map to `task_workflow` (or other tools) with structured args.
- **NormalizeToolArgs:** Rewrite `exarp-go tool_name key=value ...` → `-tool tool_name -args '{"key":"value"}'` so hooks/scripts can use short form without JSON; reserved subcommands (task, config, tui, session, …) are not rewritten.

### Reserved subcommands

List is explicit: `task`, `config`, `tui`, `tui3270`, `lock`, `session`, `cursor`, `queue`, `worker`. First arg matching one of these → CLI. Anything else that looks like `tool_name key=value` can be normalized to `-tool` / `-args`.

**Apply here:** If the Rust CLI grows (e.g. `cli task list`, `cli config show`), keep a single binary; use subcommands and env (e.g. `GIT_HOOK=1`) or TTY to choose between “interactive CLI” and “NATS/stdio server”. Document one canonical invocation form for scripts (e.g. `cli -tool X -args '{}'` or `cli task list --json`).

---

## 3. TUI

### Data path: through one capability layer

- **Rule:** TUI does not call DB or business logic directly. All task/config data goes through the same layer the CLI uses (exarp-go: MCP tools via `server.CallTool()`).
- **Adapter:** `tui_mcp_adapter.go` provides typed helpers (e.g. `listTasksViaMCP(ctx, server, status)`), which call `task_workflow` with `action=sync`, `sub_action=list`, `output_format=json`, and map JSON → in-memory model. Same for handoffs, scorecard, etc.
- **Benefit:** One place to fix bugs, add features, and enforce permissions; TUI and CLI stay in sync.

### Structure

- **Per-view files:** `tui_config.go`, `tui_handoffs.go`, `tui_detail.go`, `tui_analysis.go`, `tui_jobs.go`, `tui_keybindings.go`, `tui_commands.go`, `tui_mcp_adapter.go`, etc. One (or a few) views per file.
- **Mode constants:** Named constants for view/mode (e.g. `tasks`, `config`, `scorecard`) instead of magic strings; transitions documented (e.g. `tui_transitions.go`).
- **Messages:** Shared message types in a single file (e.g. `tui_messages.go`) for key events, resize, refresh, etc.

### Feature creep prevention

- New business behavior (e.g. “reorder waves”) is implemented in the tool layer first; TUI only calls the tool and displays the result.
- Presentation (sort, filter, layout, keybindings) stays in TUI. No duplicate business logic in TUI.

**Apply here:** Rust TUI already uses NATS (e.g. `api.finance_rates.build_curve`, `api.finance_rates.benchmarks`) as the single data path; keep that. Avoid adding backend/DB calls only from the TUI. Split UI by tab/view (e.g. `ui.rs` per tab or module) and use enums for tab/mode. Document “TUI only calls NATS (or backend API); no direct DB.”

---

## 4. Entrypoint flow (exarp-go main)

1. **Special modes:** If `-serve`, `-acp`, or `-mcp-http` → run that mode and return.
2. **Normalize:** `NormalizeToolArgs(os.Args)` so `exarp-go tool_name k=v` becomes `-tool tool_name -args '{"k":"v"}'`.
3. **Git hook guard:** If `GIT_HOOK=1` and no CLI flags → print usage and exit 1.
4. **CLI vs MCP:** If `HasCLIFlags(os.Args) || DetectMode() == ModeCLI` → `cli.Run()` and return.
5. **MCP server:** Find project root, ensure config and DB, `config.Load()`, create server, register tools/prompts/resources, `server.Run(ctx, nil)`.

So: one binary, explicit flags and reserved subcommands, TTY-based fallback only when no flags, and a single MCP path with config + DB initialized once.

**Apply here:** Backend and TUI are separate binaries here; the pattern that still applies is “single config load, single data path (NATS/API), clear CLI vs server behavior.” If we add a combined CLI (e.g. `backend_service --cli task list`), mirror this flow: flags/subcommands → CLI, else run server.

---

## 5. References (exarp-go docs)

| Topic | Path in exarp-go |
|-------|-------------------|
| Config reference | `docs/CONFIGURATION_REFERENCE.md` |
| Config implementation | `docs/CONFIGURATION_IMPLEMENTATION_PLAN.md` |
| Config params | `docs/CONFIGURABLE_PARAMETERS_RECOMMENDATIONS.md` |
| TUI/CLI/MCP review | `docs/TUI_REVIEW.md` |
| CLI from Make/CI | `docs/CLI_MAKE_CI_USAGE.md` |
| TUI MCP adapter | `internal/cli/tui_mcp_adapter.go` |
| Mode / flags | `internal/cli/mode.go`, `internal/cli/cli.go` |
| Entrypoint | `cmd/server/main.go` |

---

## 6. Summary table

| Area | exarp-go pattern | Suggested for this repo |
|------|-------------------|--------------------------|
| **Config format** | One runtime format (protobuf); YAML for export/convert | One format (e.g. TOML); optional export for editing |
| **Overrides** | Base + `.local` (gitignored), merge | `config/default.toml` + `config.local.toml` (gitignored) |
| **CLI vs server** | Flags + reserved subcommands; TTY only if no flags | Keep backend vs TUI binaries; any new CLI: subcommands + env/TTY |
| **TUI data** | All via MCP tools (adapter layer) | Keep all via NATS (and optional REST); no direct DB in TUI |
| **TUI structure** | Per-view files, mode constants, shared messages | Per-tab modules, `Tab` enum, shared events |
| **Scripts/hooks** | `-tool <name> -args '<json>'`; NormalizeToolArgs for short form | Document one script-friendly form; e.g. `cli -tool X` or REST |

This doc is a living reference; update it when adopting more patterns from exarp-go or when this repo’s TUI/CLI/config design changes.

---

## 7. Patterns We Can Adopt (This Project)

Concrete patterns from exarp-go that fit our Rust backend/TUI and how to apply them.

### Already aligned

| Pattern | Our current state |
|--------|--------------------|
| **Single data path for TUI** | TUI gets all live data via NATS (`api.*` subjects); no direct DB. Yield tab uses `api.finance_rates.build_curve` and `api.finance_rates.benchmarks`. |
| **Tab/mode as enum** | `Tab` enum (`Dashboard`, `Positions`, `Orders`, `Alerts`, `Yield`, `Logs`) with `Tab::ALL`, `next()`/`prev()`, index; no magic strings. |
| **Config defaults + overrides** | `TuiConfig::default()` then `load_shared_config()` then `apply_env_overrides()`; single load path in `TuiConfig::load()`. |
| **One runtime config format** | TUI: shared JSON config (candidates from `shared_config_candidate_paths`). Backend: single TOML via `BACKEND_CONFIG`. CLI: same shared JSON discovery as TUI (prefers shared JSON; `--config` optional override; TOML supported as CLI-only for legacy/scripts). |

### TUI/CLI config unification (done)

CLI and TUI use the **same shared JSON config discovery**: `IB_BOX_SPREAD_CONFIG` env, then home/config paths, then workspace `config/config.json` (see `api::project_paths::shared_config_candidate_paths`). The CLI loads shared JSON first via `api::load_shared_config()`; if none is found, it falls back to the `--config` path (default `config/config.json`). **TOML is supported only as a CLI-only override** when you pass `--config path/to/file.toml` (e.g. for legacy or script-specific config). Prefer a single shared JSON file for both TUI and CLI.

### High value, low effort

| Pattern | From exarp-go | Apply here |
|--------|----------------|------------|
| **Local override file (gitignored)** | `.exarp/config.local.pb` merges over base; never committed. | Add optional `config.local.toml` (or `.local.json`) for TUI/backend: load after base, merge, gitignore. Document in BACKEND_CONFIG_ENV_OVERLAY.md when we add overlay. |
| **Config validate** | `config validate` checks schema and required fields. | Add a `validate` step in `TuiConfig::load()` (e.g. NATS_URL non-empty when not in non-interactive mode); optional `backend_service --validate-config` that loads and exits 0/1. |
| **Document “TUI only calls NATS”** | Explicit rule: no direct DB in TUI; all via MCP. | Add one sentence to `docs/platform/TUI_LEGACY_DESIGN_LEARNINGS.md` or AGENTS.md: “TUI data path: NATS only (api.*); no direct database or backend HTTP.” |

### Medium value, medium effort

| Pattern | From exarp-go | Apply here |
|--------|----------------|------------|
| **Per-view / per-tab modules** | `tui_config.go`, `tui_handoffs.go`, `tui_detail.go`, etc. | Split `ui.rs` by tab: e.g. `ui/dashboard.rs`, `ui/positions.rs`, `ui/orders.rs`, `ui/alerts.rs`, `ui/yield_curve.rs`, `ui/logs.rs`, and `ui/mod.rs` that matches `app.active_tab` and delegates. Keeps `ui.rs` from growing into a single huge file. |
| **Typed “adapter” for data** | `listTasksViaMCP`, `callToolText` in `tui_mcp_adapter.go`. | We already have `run_yield_fetcher` in main that uses `request_json`; optionally group all NATS request helpers (curve, benchmarks, snapshot, strategy) into a small `nats_requests` or `tui_nats` module with typed functions (e.g. `fetch_yield_curve(symbol)`, `fetch_benchmarks()`) so TUI code stays thin. |
| **Reserved subcommands / script-friendly CLI** | `exarp-go -tool X -args '{}'`; reserved list prevents rewriting. | When/if we add a unified CLI (e.g. `backend_service` or `cli` with subcommands): define reserved subcommands (e.g. `serve`, `config`, `health`) and one canonical script form (e.g. `cli --json health` or `cli config show`). Document in CURSOR_PROJECT_COMMANDS.md. |

### Lower priority / when we grow

| Pattern | From exarp-go | Apply here |
|--------|----------------|------------|
| **Config export/convert** | Export config to YAML/JSON for editing; convert back. | Only if we move to a binary or non-editable format; currently TOML/JSON are human-editable. Optional: `cli config show` or `backend_service --show-config` that prints merged config (for debugging). |
| **TTY vs non-TTY mode** | DetectMode(): TTY → CLI, else MCP server. | We already run TUI non-interactive when !is_terminal; if we add a combined binary (CLI + server), use TTY + flags to choose (e.g. `--serve` → server, no TTY + no flags → print “use --serve or run tui_service”). |
| **Single loader with categories** | timeouts, thresholds, tasks, database in one struct. | Backend config is minimal (market_data, rest); when we add more (e.g. timeouts, feature flags), group into a single `BackendConfig` with sections and one `load_config()` that merges base + optional local file + optional env overlay. |

### Summary: what to do next

1. **Document** the “TUI only calls NATS” rule in one place (TUI_LEGACY_DESIGN_LEARNINGS or AGENTS.md).
2. **Optional:** Add a local override file (e.g. `config.local.toml` gitignored) and merge it in `TuiConfig::load()` and backend `load_config()` when we need per-machine overrides.
3. **When ui.rs gets large:** Split by tab into `ui/dashboard.rs`, `ui/yield_curve.rs`, etc.
4. **When we add a richer CLI:** Reserve subcommands and one script-friendly invocation; mirror exarp-go’s flow (flags first, then CLI vs server).
