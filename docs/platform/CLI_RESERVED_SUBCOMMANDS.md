# CLI reserved subcommands and script-friendly form

Canonical design for the Aether CLI (`aether` / `cargo run -p cli`) when adding richer subcommands and script/automation support.

## Reserved subcommands

The following subcommand names are **reserved** so scripts, docs, and tooling can depend on them. New subcommands must not collide with these.

| Subcommand | Purpose | Status |
|------------|---------|--------|
| **serve** | Start backend/API server (e.g. HTTP or NATS) | Reserved; not implemented |
| **config** | Config show / export / validate (extends current `validate`, `init-config`) | Reserved; partial today |
| **health** | Health check (e.g. NATS/backend liveness) | Reserved; not implemented |

Existing subcommands today: `init-config`, `validate`, `run`, `snapshot`, `benchmarks`.

## Script-friendly (canonical) form

For scripts and automation, use one canonical invocation form so output is stable and parseable:

- **`aether <subcommand> --json`**

Examples:

- `aether health --json` — machine-readable health status to stdout
- `aether config show --json` — current config as JSON
- `aether benchmarks --json` — already supported today

Rules:

- **`--json`** (global or per-subcommand): when present, output is JSON to stdout for parsing; when absent, output is human-oriented.
- Prefer **subcommand before global flags** for clarity (e.g. `aether health --json`, not necessarily `aether --json health`; both can be supported if desired).
- Scripts should rely on exit code for success/failure and parse stdout only when `--json` was passed.

## References

- CLI binary: `agents/backend/bin/cli/src/main.rs` — add the same reserved-subcommands and script-form summary in the module doc (top of `main.rs`) when editing the CLI.
- TUI/CLI patterns: `docs/platform/EXARP_GO_TUI_CLI_CONFIG_LEARNINGS.md`.
- Feature parity: `docs/platform/TUI_CLI_FEATURE_PARITY.md`.
