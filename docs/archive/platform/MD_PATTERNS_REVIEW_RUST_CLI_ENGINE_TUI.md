# MD Files Review: Patterns for Rust CLI / Engine / TUI

**Purpose:** Patterns and recommendations from project Markdown docs that can be implemented in the Rust CLI, engine (backend/ib_adapter/quant/risk), or TUI.  
**Scope:** Review of 100+ relevant `.md` files; synthesized by target (CLI, engine, TUI) and theme.  
**Date:** 2026-03-15.

---

## Sources reviewed (by theme)

| Theme | Key docs |
|-------|----------|
| **TUI** | TUI_LEGACY_DESIGN_LEARNINGS.md, TUI_CLI_FEATURE_PARITY.md, TUI_RUST_READ_PATH_AUDIT.md, TUI_SCENARIO_EXPLORER_DESIGN.md, TICKER_TUI_ANALYSIS.md |
| **CLI** | TUI_CLI_FEATURE_PARITY.md, CLI_TUI_TOOLS_RECOMMENDATIONS.md, SHELL_COMPLETION.md, DATA_SOURCE_CONFIG.md, MULTI_LANGUAGE_CODEBASE.md |
| **Engine / backend** | RUST_CRATE_OPPORTUNITIES_AUDIT.md, RUST_CORE_PATTERNS.md, IMPROVEMENT_PLAN.md, STUB_CODE_PLANNING.md, design/LOGIC_WE_COULD_UNIFY.md, IB_ADAPTER_REVIEW.md |
| **Config / data** | SHARED_CONFIGURATION_SCHEMA.md, BACKEND_CONFIG_ENV_OVERLAY.md, DATA_SOURCE_CONFIG.md, NATS_API.md, DATAFLOW_ARCHITECTURE.md |
| **Logging / testing** | BREADCRUMB_LOGGING_TRADING_TESTING.md, TESTING_STRATEGY.md, FUTURE_IMPROVEMENTS.md |
| **Tooling** | CLI_TUI_TOOLS_RECOMMENDATIONS.md, DEVELOPMENT_TOOLS.md |

---

## 1. Rust CLI patterns

### 1.1 Config and subcommands (from TUI_CLI_FEATURE_PARITY, MULTI_LANGUAGE_CODEBASE)

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **Shared JSON config** | TUI_CLI_FEATURE_PARITY, DATA_SOURCE_CONFIG | CLI should use same shared JSON discovery as TUI/backend (IB_BOX_SPREAD_CONFIG, config paths). Already started: CLI loads shared JSON first, then TOML fallback; `--init-config path.json` writes shared JSON sample. |
| **Subcommands** | TUI_CLI_FEATURE_PARITY | Add `cli init-config`, `cli validate`, `cli run`, `cli snapshot` so behavior is discoverable and scriptable (not only flags). |
| **Validate before run** | TUI_CLI_FEATURE_PARITY, CURSOR_PROJECT_COMMANDS | Keep `--validate`; run validation on load when not in init-config mode. |
| **Dry-run / mock** | DATA_SOURCE_CONFIG, TUI_CLI_FEATURE_PARITY | CLI keeps `--dry-run`, `--mock-tws`; document that TUI does not toggle these (backend concern). |

### 1.2 Shell and tooling (from SHELL_COMPLETION, CLI_TUI_TOOLS_RECOMMENDATIONS)

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **Shell completion** | SHELL_COMPLETION.md | Generate completions for CLI (bash/zsh/fish) via clap’s `Command::complete_*` or `scripts/generate_completions.sh` if it supports the Rust binary. Ensure `aether`/`cli` is the binary name in completions. |
| **ATAC / httpie / jq** | CLI_TUI_TOOLS_RECOMMENDATIONS | Use for testing backend/NATS from terminal; document in runbooks. CLI can stay thin; API testing via these tools. |
| **VHS for CLI demos** | CLI_TUI_TOOLS_RECOMMENDATIONS | Scripted `.tape` files for `cli --help`, `cli validate`, init-config; golden output in CI. |
| **tmux/zellij layout** | CLI_TUI_TOOLS_RECOMMENDATIONS | Document pane layout: backend, TUI, CLI, API testing. |

### 1.3 Data and scope (from TUI_CLI_FEATURE_PARITY, IMPROVEMENT_PLAN)

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **CLI scope** | TUI_CLI_FEATURE_PARITY | Decided: (b) config + validate + external backend only. See docs/platform/TUI_CLI_FEATURE_PARITY.md § CLI scope (decision). |
| **Snapshot path** | TUI_CLI_FEATURE_PARITY | CLI has `--snapshot-path` / `--no-snapshot`; stub loop does not publish. When scope is “in-process”, CLI could trigger or consume snapshot. |

---

## 2. Rust engine patterns

### 2.1 Crates and dependencies (from RUST_CRATE_OPPORTUNITIES_AUDIT)

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **Backoff/retry** | RUST_CRATE_OPPORTUNITIES_AUDIT | Use **backoff** (or **tokio-retry**) for TUI NATS reconnect and nats_adapter DLQ retry instead of custom backoff math. Audit says backoff is done for DLQ; consider for TUI if not already. |
| **Config layers** | RUST_CRATE_OPPORTUNITIES_AUDIT, BACKEND_CONFIG_ENV_OVERLAY | When backend needs env overrides (e.g. BACKEND_REST_*), introduce **config** or **figment** for layered config; document precedence. |
| **YYYYMMDD expiry** | RUST_CRATE_OPPORTUNITIES_AUDIT | Single shared helper (e.g. `common::expiry::parse_expiry_yyyy_mm_dd`); quant, risk, ib_adapter use it. Audit says done in `crates/common`. |
| **JSON with comments** | RUST_CRATE_OPPORTUNITIES_AUDIT | TUI and api use jsonc-parser/jsonc; no custom strip_json_comments. Keep. |

### 2.2 Architecture (from RUST_CORE_PATTERNS, IMPROVEMENT_PLAN)

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **Event-driven position tracking** | RUST_CORE_PATTERNS | Replace mutable state with event sourcing where useful; deterministic replay. High priority for engine. |
| **Order state machine** | RUST_CORE_PATTERNS | Track order lifecycle (submit → accept → fill → cancel); handle rejections. Implement in api/ib_adapter when wiring real TWS. |
| **Margin calculator** | RUST_CORE_PATTERNS | Real-time margin checking; Reg-T vs portfolio margin. Enhance in risk crate when needed. |
| **Position P&L** | RUST_CORE_PATTERNS | Real-time unrealized/realized P&L by strategy/underlying. Expose in snapshot/read models. |
| **Persistence first** | IMPROVEMENT_PLAN | Every market event, order, position change lands in durable storage before UI updates. Single writer per store. |
| **Single source of truth** | IMPROVEMENT_PLAN, DATAFLOW_ARCHITECTURE | One writer per data store; NATS KV or snapshot as canonical live-state; Rust backend owns it. |

### 2.3 Stubs and implementation (from STUB_CODE_PLANNING, IB_ADAPTER_REVIEW)

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **ib_adapter** | STUB_CODE_PLANNING | Full stub until scope approved. Implement connect, request_market_data, place_order, cancel_order, request_positions, request_account with ibapi when ready; security/review before live. |
| **Runtime state** | STUB_CODE_PLANNING | decisions_by_producer_type, find_by_correlation_id, positions_by_account — metadata at write path; document NATS envelope → metadata. |
| **FMP / REST** | STUB_CODE_PLANNING | FMP fundamentals routes wired; requires FMP_API_KEY for real data. |

### 2.4 Unification (from LOGIC_WE_COULD_UNIFY)

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **Config loading** | LOGIC_WE_COULD_UNIFY | Single schema (config/schema.json) for TUI and backend; CLI now shares same discovery (shared JSON). C++ config manager legacy. |
| **Risk calculator** | LOGIC_WE_COULD_UNIFY | Stats exposed via pybind11; full RiskCalculator (VaR, Greeks, etc.) not bound. If Python needs it, add more bindings; else keep Rust-only and document. |

---

## 3. Rust TUI patterns

### 3.1 Already documented (TUI_LEGACY_DESIGN_LEARNINGS)

- See **docs/platform/TUI_LEGACY_DESIGN_LEARNINGS.md** for layout, shortcuts, tabs, color, scenario explorer, multiscreen, and patterns from completed exarp TUI tasks.
- Exarp tasks created for: arrow-key scroll, account in header, ROI column, Enter detail popup, K cancel all, scenario explorer, split-pane.

### 3.2 From other MDs

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **NATS-only** | DATA_SOURCE_CONFIG, STUB_CODE_PLANNING | TUI consumes NATS only; no REST snapshot fallback. Keep. |
| **Breadcrumb logging** | BREADCRUMB_LOGGING_TRADING_TESTING | Apply breadcrumb-style (structured) logging in Rust order/strategy paths: place_order_attempt, validation_failed, before_order_execution, order_placed. Use tracing spans/events; TUI or logs tab can show last N. |
| **VHS / asciinema** | CLI_TUI_TOOLS_RECOMMENDATIONS | Scripted TUI demos (VHS .tape) for CI; asciinema for human demos. |
| **Event routing** | STUB_CODE_PLANNING | Defer EventRouter until popups/multi-handler; then wire events.rs or adopt ratatui/async-template. |

---

## 4. Cross-cutting patterns

### 4.1 Config

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **Shared JSON schema** | SHARED_CONFIGURATION_SCHEMA, DATA_SOURCE_CONFIG | One schema (config/schema.json); TUI, backend, CLI (when using shared JSON) use same discovery and env overrides (NATS_URL, BACKEND_ID, WATCHLIST, etc.). |
| **Env overlay** | BACKEND_CONFIG_ENV_OVERLAY | Backend may add env overlay for BACKEND_REST_* etc.; use config/figment when introduced. |

### 4.2 Logging and testing

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **Structured logging** | BREADCRUMB_LOGGING_TRADING_TESTING | tracing + JSON or key-value events for order placement, TWS connection, strategy actions in api, ib_adapter. Enables audit trail and debugging. |
| **Test commands** | FUTURE_IMPROVEMENTS | Rust: `just test` or `cargo test` in agents/backend. TUI E2E: `just test-tui-e2e`. No C++ tests; docs updated. |
| **Golden / VHS** | CLI_TUI_TOOLS_RECOMMENDATIONS | Use VHS for CLI and TUI golden output in CI. |

### 4.3 Data flow and NATS

| Pattern | Doc | Recommendation |
|---------|-----|----------------|
| **NATS API** | NATS_API.md | Request/reply subjects (api.strategy.start/stop, etc.); validate topic length (e.g. 256); document in NATS_API.md. |
| **Single writer** | IMPROVEMENT_PLAN | Rust backend is single writer to ledger, live-state, NATS KV; no dual-write. |

---

## 5. Priority summary

| Priority | Area | Pattern | Doc reference |
|----------|------|---------|----------------|
| **High** | CLI | Subcommands (init-config, validate, run, snapshot) | TUI_CLI_FEATURE_PARITY |
| **High** | CLI | Define scope (in-process vs external backend) | TUI_CLI_FEATURE_PARITY |
| **High** | Engine | Event-driven position tracking; order state machine | RUST_CORE_PATTERNS |
| **Medium** | CLI | Shell completion for Rust CLI | SHELL_COMPLETION |
| **Medium** | Engine | Backoff crate for TUI reconnect if not already | RUST_CRATE_OPPORTUNITIES_AUDIT |
| **Medium** | TUI | All items in TUI_LEGACY_DESIGN_LEARNINGS §10–11 | TUI_LEGACY_DESIGN_LEARNINGS |
| **Medium** | Cross-cut | Breadcrumb-style structured logging in order/strategy paths | BREADCRUMB_LOGGING_TRADING_TESTING |
| **Low** | CLI | VHS tapes for CLI help/validate | CLI_TUI_TOOLS_RECOMMENDATIONS |
| **Low** | Backend | config/figment when env overlay needed | RUST_CRATE_OPPORTUNITIES_AUDIT, BACKEND_CONFIG_ENV_OVERLAY |

---

## 6. How to use this doc

- **Implementing CLI:** Use §1 and §4; align with TUI_CLI_FEATURE_PARITY and shared config.
- **Implementing engine/backend:** Use §2 and §4; align with IMPROVEMENT_PLAN, STUB_CODE_PLANNING, RUST_CORE_PATTERNS.
- **Implementing TUI:** Use §3 and TUI_LEGACY_DESIGN_LEARNINGS.md; create exarp tasks from §10–11 there and from §3.2 here.
- **Adding config/logging/tests:** Use §4.

This file is a living index; when adding features, check the referenced MDs for full context and update this table if new patterns emerge from docs.
