# Backend service config: when to add env overlay

**Status:** Decision recorded (defer)  
**Last updated:** 2026-03-15  
**Related:** [RUST_CRATE_OPPORTUNITIES_AUDIT.md §4](RUST_CRATE_OPPORTUNITIES_AUDIT.md#4-config-loading-backend_service)

## Context

`backend_service` loads configuration via `load_config()` in `agents/backend/services/backend_service/src/main.rs`:

- **Source:** Single TOML file path from `BACKEND_CONFIG` (default `config/default.toml`).
- **Parsing:** `toml::from_str` into `BackendConfig` (today: `market_data` only; `rest_addr` in default.toml is present but REST is currently disabled per [REMOVE_REST_OPTIONS.md](REMOVE_REST_OPTIONS.md)).
- **No env overlay:** There is no layer that overrides TOML keys from environment variables (e.g. `BACKEND_REST_ADDR`, `BACKEND_MARKET_DATA_PROVIDER`).

Some runtime behavior is already driven by env vars (e.g. `NATS_URL`, `BACKEND_ID`, `SNAPSHOT_PUBLISH_INTERVAL_MS`, `FMP_API_KEY`, `POLYGON_API_KEY` via `api_key_env`); these are ad hoc and not part of a unified config overlay.

## Decision

**Defer introducing a formal env overlay** (e.g. `BACKEND_*` prefix, or a crate like `config` / `figment`) until at least one of the following is true:

1. **Deployment/CI need:** We need to override bind address, port, or NATS URL without editing the TOML file (e.g. container/CI overrides).
2. **Multi-environment:** We run the same binary against multiple environments (dev/stage/prod) and want env-specific overrides on top of a base TOML.
3. **Twelve-factor alignment:** We explicitly adopt env-over-file precedence for backend_service and want a single, documented precedence order.

Until then:

- Keep **TOML-only** `load_config()` and **single file** via `BACKEND_CONFIG`.
- Continue using **ad hoc env vars** where they already exist (NATS, BACKEND_ID, FMP, Polygon api_key_env, etc.).
- When we introduce overlay: document precedence (e.g. env overrides TOML), choose a prefix (e.g. `BACKEND_`), and consider the **config** or **figment** crate as in [RUST_CRATE_OPPORTUNITIES_AUDIT.md §4](RUST_CRATE_OPPORTUNITIES_AUDIT.md#4-config-loading-backend_service).

## Follow-up

- **Implement overlay:** When one of the triggers above applies, add a task to implement env overlay (and optionally add `BACKEND_REST_ADDR` / `BACKEND_NATS_URL` etc.) and update this note with the chosen approach and precedence.

---

## Optional local override files (gitignored)

**Status:** Implemented  
**Last updated:** 2026-03-15

Both **backend_service** and **TUI** (shared config) support optional, gitignored local override files. These are loaded *after* the base config and merged (local keys override base). Intended for developer- or machine-specific settings (e.g. API keys, ports, NATS URL) without committing secrets or local paths.

### Backend (TOML)

- **Base config:** `BACKEND_CONFIG` (default `config/default.toml`).
- **Local override:** In the same directory as the base file, a file named `config.local.toml`, or `config/config.local.toml` if the base path has no parent. Example: if base is `config/default.toml`, local is `config/config.local.toml`.
- **Merge:** Deep merge; keys in the local file override the base. Full structure is re-deserialized into `BackendConfig` after merge.
- **Gitignore:** `config/config.local.toml`, `config.local.toml`.

### TUI / shared config (JSON/JSONC)

- **Base config:** First existing file from shared config candidate paths (e.g. `IB_BOX_SPREAD_CONFIG`, `config/config.json`, workspace `config/config.json`). See `api::project_paths::shared_config_candidate_paths()`.
- **Local override:** First existing file from `api::project_paths::shared_config_local_override_paths()`: `config/config.local.json`, or workspace `config/config.local.json`. Loaded after base; JSON merged (local overrides base).
- **Gitignore:** `config/*.local.json` already covers `config/config.local.json`.

### Precedence (summary)

1. Base config file (backend: TOML; TUI: shared JSON).
2. Optional local override file (backend: `config.local.toml`; TUI: `config.local.json`). Merged on top of base.
3. Environment variables (where supported; e.g. TUI `NATS_URL`, `BACKEND_ID`; backend ad hoc env such as `NATS_URL`, `FMP_API_KEY`).
