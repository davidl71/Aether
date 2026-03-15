# TUI / CLI Feature Parity

**Last updated**: 2026-03-14  
**Purpose**: Compare Rust TUI and Rust CLI capabilities and document parity gaps and recommendations.

## Current state

| Capability | TUI (`tui_service`) | CLI (`cli`) |
|------------|---------------------|-------------|
| **Config** | Shared JSON (`IB_BOX_SPREAD_CONFIG` or `config/config.json`, home, etc.) | TOML (`config/config.toml` by default) |
| **Config init** | No | Yes (`--init-config`) |
| **Config validate** | No (load only) | Yes (`--validate`) |
| **Dry run** | No | Yes (`--dry-run`) |
| **Mock TWS** | No | Yes (`--mock-tws`) |
| **Snapshot output** | Consumes from NATS (and optional REST fallback) | `--snapshot-path` / `--no-snapshot` (stub: loop does not publish) |
| **Live view** | Dashboard, Positions, Orders, Alerts, Logs tabs | No (trading loop is `thread::park()` stub) |
| **Data source** | NATS snapshot subject (from backend_service) | None (CLI does not run backend or publish) |
| **Logging** | In-TUI Logs tab + file | `--log-level`, `-v` to stderr |

## Data flow

- **TUI**: Connects to NATS (and optionally REST). Snapshot data is produced by **backend_service** (and ultimately ib_adapter when TWS is connected). TUI is display-only.
- **CLI**: Loads TOML config, validates, then runs a **stub** trading loop that only parks the thread. It does not start a backend, connect to TWS, or publish snapshots. So today the CLI does not feed the TUI or any other consumer.

## Parity gaps

1. **Config format**: TUI uses shared JSON; CLI uses TOML. They are not the same schema or discovery path. MULTI_LANGUAGE_CODEBASE says "Config: JSON under config/; shared by Rust TUI, CLI, and backend" — currently only TUI and backend use the shared JSON; CLI uses its own TOML.
2. **CLI trading loop**: Placeholder only. No TWS connection, no snapshot publishing, no subcommands (e.g. `run`, `snapshot`, `validate`).
3. **TUI**: No config init/validate entry point; no dry-run or mock-TWS toggles in the UI (those are runtime concerns of whoever runs the backend).
4. **Feature parity script**: `scripts/check_feature_parity.sh` is referenced in CURSOR_PROJECT_COMMANDS.md and docs but the script is not present in the repo.

## Recommendations

| Priority | Item | Notes |
|----------|------|--------|
| **High** | Unify config | Either (a) make CLI use shared JSON + same discovery as TUI/backend, or (b) document TOML as CLI-only and add a bridge (e.g. CLI `--config-json` to validate shared config). |
| **High** | Define CLI scope | Decide whether CLI is “config + validate + run backend in-process” vs “config + validate + external backend only.” If in-process, implement or wire a minimal run path (e.g. start backend_service or ib_adapter and publish snapshot). |
| **Medium** | CLI subcommands | Add `cli init-config`, `cli validate` (existing flags) and e.g. `cli run`, `cli snapshot` so behavior is discoverable and scriptable. |
| **Medium** | TUI config validation | On load, optionally validate shared config against schema and show a hint or error in the TUI. |
| **Low** | Restore or replace `check_feature_parity.sh` | Implement a small script or just doc that compares TUI vs CLI features (e.g. from this doc) so automation/docs stay in sync. |

## References

- **TUI read path**: `docs/platform/TUI_RUST_READ_PATH_AUDIT.md`
- **Topology**: `docs/platform/CURRENT_TOPOLOGY.md`
- **Dataflow**: `docs/platform/DATAFLOW_ARCHITECTURE.md`
- **Shared config**: `config/schema.json` (JSON Schema for shared config); CLI uses separate TOML schema.
