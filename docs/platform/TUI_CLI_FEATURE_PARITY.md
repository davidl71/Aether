# TUI / CLI Feature Parity

**Last updated**: 2026-03-15  
**Purpose**: Compare Rust TUI and Rust CLI capabilities and document parity gaps and recommendations.

## TUI features (`tui_service`)

- **Config**: Shared JSON (`IB_BOX_SPREAD_CONFIG` or `config/config.json`, home); env overrides `NATS_URL`, `BACKEND_ID`, `WATCHLIST`, `TICK_MS`, `SNAPSHOT_TTL_SECS`. Hot-reload every 5s.
- **Config validation**: Optional hint on load/reload (e.g. "NATS_URL empty") shown in status bar when invalid.
- **Live view**: Tabs — **Dashboard** (symbols, ROI sparklines, metrics), **Positions**, **Orders** (filter by `/`), **Alerts**, **Logs** (scroll, level filter).
- **Data**: NATS only — subscribes to `snapshot.{backend_id}`; request/reply to `api.*` for loans, finance rates, strategy; no REST. See [NATS_API.md](NATS_API.md) for full subject list. Shows NATS status (UP/DOWN) and detail when down; "Updated Xs ago" when snapshot present; [STALE] when older than TTL.
- **Strategy control**: **S** = strategy start, **T** = strategy stop (NATS request to backend); result in hint bar.
- **Keys**: `q` quit, `?` help overlay, Tab/←→ switch tab, 1–5 jump to tab, Orders: `/` filter / Esc clear, Logs: ↑↓ PgUp/PgDn scroll, `+`/`-` level.
- **Logging**: In-TUI Logs tab + file (`/tmp/tui_service.log` or `LOG_FILE`).

## CLI features (`cli`)

- **Config**: TOML (`config/config.toml` by default). **Init**: `--init-config`. **Validate**: `--validate`.
- **Run**: `--dry-run`, `--mock-tws`. Snapshot: `--snapshot-path` / `--no-snapshot` (stub loop does not publish).
- **Logging**: `--log-level`, `-v` to stderr.
- **Trading loop**: Placeholder (`thread::park()`); no TWS, no backend, no NATS publish.

## Current state (comparison)

| Capability | TUI (`tui_service`) | CLI (`cli`) |
|------------|---------------------|-------------|
| **Config** | Shared JSON + env + hot-reload | TOML |
| **Config init** | No | Yes (`--init-config`) |
| **Config validate** | Hint on load (status bar) | Yes (`--validate`) |
| **Dry run** | No | Yes (`--dry-run`) |
| **Mock TWS** | No | Yes (`--mock-tws`) |
| **Snapshot** | Consumes from NATS only | `--snapshot-path` / `--no-snapshot` (stub) |
| **Live view** | Dashboard, Positions, Orders, Alerts, Logs, strategy S/T, help `?` | No |
| **Data source** | NATS (backend_service) | None |
| **Logging** | In-TUI Logs tab + file | `--log-level`, `-v` stderr |

## Data flow

- **TUI**: Connects to NATS only. Snapshot data is produced by **backend_service** (and ultimately ib_adapter when TWS is connected). TUI is display-only.
- **CLI**: Loads TOML config, validates, then runs a **stub** trading loop that only parks the thread. It does not start a backend, connect to TWS, or publish snapshots. So today the CLI does not feed the TUI or any other consumer.

## CLI scope (decision)

**Current decision (2026-03-15):** **(b) Config + validate + external backend only.**

- The CLI does **not** start `backend_service` or publish to NATS. `aether run` is a placeholder loop for local use (e.g. config validation and dry-run flags).
- TUI and other consumers get data from **backend_service** via NATS. Run backend_service separately (e.g. `cargo run -p backend_service`).
- If in-process backend or snapshot publishing from the CLI is needed later, it will be an explicit scope change and tracked separately.

## Parity gaps

1. **Config format**: TUI uses shared JSON; CLI uses TOML. They are not the same schema or discovery path. MULTI_LANGUAGE_CODEBASE says "Config: JSON under config/; shared by Rust TUI, CLI, and backend" — currently only TUI and backend use the shared JSON; CLI uses its own TOML.
2. **CLI trading loop**: Placeholder only. No TWS connection, no snapshot publishing. Subcommands are implemented: `aether init-config`, `aether validate`, `aether run`, `aether snapshot` (see CLI scope above).
3. **TUI**: No config init entry point; no dry-run or mock-TWS toggles in the UI (those are runtime concerns of whoever runs the backend). TUI now shows a config validation hint on load/reload when NATS_URL or BACKEND_ID is missing.
4. **Feature parity script**: `scripts/check_feature_parity.sh` prints this doc path and a short summary; run via Cursor command `check:feature-parity` or `./scripts/check_feature_parity.sh`.

## Config unification decision (documented)

**Decision (2026-03):** **(b) TOML remains CLI-only; optional bridge for shared config.**

- **TUI and backend** use shared JSON config (e.g. `IB_BOX_SPREAD_CONFIG` or `config/config.json`) and env overrides; discovery and hot-reload are documented.
- **CLI** keeps TOML (`config/config.toml` by default) for init/validate/run. No change to CLI schema or discovery in this decision.
- **Bridge (optional):** Add CLI support to validate the **shared JSON** (e.g. `aether validate --config-json <path>` or `--config config/config.json` when that path is the shared JSON) so operators can check TUI/backend config from the CLI without unifying formats. **Implemented:** `aether validate` uses the same discovery as TUI (IB_BOX_SPREAD_CONFIG, then config/config.json) and validates shared JSON with the same rules (NATS_URL, BACKEND_ID, tui fields); exits 0 on success, 1 on validation failure.
- **Unify config (option a)** — making CLI use shared JSON + same discovery as TUI/backend — remains a possible future change and would be an explicit scope/design task.

This decision is documented here so that "Unify config" in Recommendations is understood as either (a) full unification or (b) bridge-only, with (b) as the current documented choice.

## Recommendations

| Priority | Item | Notes |
|----------|------|--------|
| **High** | Unify config | Either (a) make CLI use shared JSON + same discovery as TUI/backend, or (b) document TOML as CLI-only and add a bridge (e.g. CLI `--config-json` to validate shared config). |
| **High** | ~~Define CLI scope~~ | Done: external backend only (see "CLI scope (decision)" above). |
| **Medium** | ~~CLI subcommands~~ | Done: `aether init-config`, `aether validate`, `aether run`, `aether snapshot`; legacy flags `--init-config` and `--validate` still work. |
| **Medium** | ~~TUI config validation~~ | ✅ Done: hint on load/reload when NATS_URL or BACKEND_ID empty (status bar). Full schema validation still optional. |
| **Low** | ~~Restore or replace `check_feature_parity.sh`~~ ✅ | Done: `scripts/check_feature_parity.sh` prints this doc path and a short summary; run via `check:feature-parity` or `./scripts/check_feature_parity.sh`. |

## Current vs planned (summary)

### TUI (`tui_service`)

| Aspect | Current | Planned / deferred |
|--------|---------|--------------------|
| **Config** | Shared JSON + env + 5s hot-reload; validation hint when NATS_URL/BACKEND_ID empty | — |
| **Live view** | Dashboard, Positions, Orders (filter `/`), Alerts, Logs; strategy **S**/**T**; help **?** | No explicit roadmap; config unification (see below) is the main cross-cutting item |
| **Data** | NATS only: subscribe `snapshot.{backend_id}`; request/reply `api.*` (loans, finance rates, strategy) | — |
| **Config init / dry-run / mock-TWS** | Not in TUI (backend operator concern) | Not planned in TUI |

### CLI (`cli`)

| Aspect | Current | Planned / reserved |
|--------|---------|--------------------|
| **Subcommands** | `init-config`, `validate`, `run`, `snapshot`, `benchmarks` (+ legacy `--init-config`, `--validate`) | **Reserved (not implemented):** `serve`, `config` (show/export/validate), `health` — see [CLI_RESERVED_SUBCOMMANDS.md](CLI_RESERVED_SUBCOMMANDS.md) |
| **Config** | TOML or JSON (default `config/config.json`); init and validate; `validate` uses same shared JSON discovery as TUI and exits 0/1 | Bridge implemented; full unification remains optional |
| **Run** | Placeholder loop only; `--dry-run`, `--mock-tws`, `--snapshot-path`, `--no-snapshot`, `--log-level`, `-v` | **Scope decision:** CLI does **not** start backend or publish to NATS; if needed later, explicit scope change |
| **Data / NATS** | None; no backend in process | Not in scope (external backend only) |
| **Script-friendly** | `benchmarks --json` supported | Canonical form: `aether <subcommand> --json` (e.g. `health --json`, `config show --json`) when those subcommands exist |

### Decisions in effect

- **CLI scope:** Config + validate + external backend only. Run backend separately (`cargo run -p backend_service`); TUI and others consume via NATS.
- **Parity gap:** Config format (TUI = shared JSON; CLI = TOML or same JSON path). High-priority recommendation: unify or document and add a bridge.

## References

- **TUI read path**: `docs/platform/TUI_RUST_READ_PATH_AUDIT.md`
- **Topology**: `docs/platform/CURRENT_TOPOLOGY.md`
- **Dataflow**: `docs/platform/DATAFLOW_ARCHITECTURE.md`
- **Shared config**: `config/schema.json` (JSON Schema for shared config); CLI uses separate TOML schema.
- **CLI reserved subcommands**: `docs/platform/CLI_RESERVED_SUBCOMMANDS.md`
