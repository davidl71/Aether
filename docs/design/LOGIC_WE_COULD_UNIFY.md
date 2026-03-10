# Logic We Could Unify

**Purpose:** Concrete areas where logic is duplicated across C++, Python, or Rust and how to unify them (single canonical implementation + bindings or shared spec). Complements `docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md` and `docs/design/DSL_AND_PROTO_OPPORTUNITIES.md`.

---

## 1. Statistical helpers (mean, percentile, correlation)

| Where | What |
|-------|------|
| **C++** | `calculate_mean`, `calculate_percentile`, `calculate_correlation` in `native/src/risk_calculator.cpp` (and declared in `risk_calculator.h`). Used by risk and portfolio calculations. |
| **Python** | Same three functions (and uses of them) in `python/integration/risk_calculator.py`; tests in `python/tests/test_risk_calculator.py`. |

**Unify:** Expose C++ free functions via pybind11 (same module as box spread bindings or a small `risk_bindings` / `stats_bindings`). Python `risk_calculator.py` calls C++ for these and removes its own implementations. Keeps one source of truth and avoids drift (e.g. percentile interpolation).

**Effort:** Low (add a few `m.def()` in pybind; change Python to import from bindings).

---

## 2. DTE (days to expiry) and trading-day logic

| Where | What |
|-------|------|
| **C++** | `tws::calculate_dte(expiry_str)` in `tws_conversions.cpp` (uses `market_hours::MarketHours` for trading days). Used by `types::BoxSpreadLeg::get_days_to_expiry()`. |
| **Python** | DTE or `days_to_expiry` appears in many places (sofr_treasury_client, benchmarks, TUI models, risk-free rate). Some may compute from date; no single shared “trading days” implementation. |

**Unify:** (1) Expose `calculate_dte(expiry_yyyy mmdd: str) -> int` via pybind11 (already in the same build as box spread bindings via `tws_conversions.cpp`). (2) Optionally expose `MarketHours.get_market_status_at(time)` or `is_trading_day(date)` if Python needs “is this a trading day?” for rate curves or UI. Then Python uses the binding instead of ad‑hoc date math.

**Effort:** Low for DTE-only; medium if you add full MarketHours bindings.

---

## 3. Risk calculator (beyond stats)

| Where | What |
|-------|------|
| **C++** | Full `RiskCalculator` (VaR, correlation risk, position sizing, aggregate Greeks, etc.) in `native/src/risk_calculator.cpp`. |
| **Python** | Full port in `python/integration/risk_calculator.py` (documented as stub / subset in Phase 5). |

**Unify:** (1) **Already aligned:** Box spread and risk DTOs at boundaries use proto; Python prefers C++ for box spread math via bindings. (2) **Next step:** Expose more C++ `RiskCalculator` methods via pybind11 (e.g. `calculate_correlation_risk`, `calculate_var`, position sizing) so Python strategy runner and any risk checks call C++. Then thin `risk_calculator.py` to a wrapper that builds inputs (from proto or Python types), calls C++, and maps results. Reduces duplication and keeps one implementation for regulatory/audit clarity.

**Effort:** Medium (design which methods to expose; add bindings; migrate callers).

---

## 4. Config loading and validation

| Where | What |
|-------|------|
| **C++** | `ConfigManager::load()`, `from_json` for `StrategyParams`, `RiskConfig`, etc. in `native/` and `config_manager.h`. |
| **Python** | `SharedConfigLoader.load_config()`, TUI `load_config()`, various `from_json`/dict parsing. |

**Unify:** (1) **Schema:** Use a single **JSON Schema** for the shared config (TUI, PWA, native CLI) so all loaders validate the same structure. (2) **Parsing:** Keep language-specific loaders; no need to call C++ from Python for config unless you want one parser. (3) **DTOs at boundaries:** Config-derived values that cross processes (e.g. strategy params on NATS) already use proto; keep that.

**Effort:** Low for JSON Schema only; higher if you push “parse once in C++ and expose to Python.”

---

## 5. Option / contract string parsing

| Where | What |
|-------|------|
| **C++** | `types::OptionContract`; expiry/contract representation in TWS conversions and option chain. |
| **Python** | `combo_detector.parse_option_contract_desc()` (symbol, expiry, strike, right from IB contractDesc); other ad‑hoc parsing. |

**Unify:** (1) Define a **single spec** (e.g. “contractDesc format,” “OCC symbol format”) in docs and have both C++ and Python follow it. (2) Optional: Implement one canonical parser in C++, expose `parse_option_contract_desc(str) -> OptionContract` (or tuple) via pybind11, and have Python call it. Reduces drift and handles edge cases in one place.

**Effort:** Low for spec-only; medium for C++ parser + binding.

---

## 6. Discount bank parser

| Where | What |
|-------|------|
| **Rust** | Canonical parser in `agents/backend/crates/discount_bank_parser`. |
| **Python** | Fallback parser in `python/integration/discount_bank_parser.py` (deprecated; prefer Rust binary). |

**Unify:** Already in plan (Phase 6): Rust canonical; Python calls Rust binary; remove or keep Python fallback only for environments without the binary. No further unification needed except completing deprecation.

---

## 7. Market hours and calendar

| Where | What |
|-------|------|
| **C++** | `market_hours::MarketHours` (holidays, early closes, session status, DST). Used by `calculate_dte` and TWS connection. |
| **Python** | No full port; ad‑hoc date logic where needed. |

**Unify:** (1) **Minimal:** Expose `calculate_dte` only (see §2). (2) **Fuller:** Expose `MarketHours` (e.g. `get_market_status_at(time)`, `is_holiday(date)`, `is_early_close(date)`) via pybind11 so Python TUI, benchmarks, and risk-free rate logic use the same calendar as C++. Avoids reimplementing US holidays/early closes in Python.

**Effort:** Medium (bind a small API surface; document behaviour).

---

## 8. Constants and enums

| Where | What |
|-------|------|
| **C++** | `types.h` (OptionType, OrderStatus, etc.); config defaults. |
| **Python** | Mirrors in proto_types, box_spread_models, enums in integration code. |
| **Proto** | Enums and message field semantics in `proto/messages.proto`. |

**Unify:** (1) **At boundaries:** Use proto-generated enums and types so NATS/REST and cross-service code share one definition. (2) **In Python:** Prefer generated types from `python/generated/` (or `proto_types` if still used) instead of redefining enums by hand. (3) **Numeric constants** (e.g. default min_roi, max_dte): Either define once in config schema defaults or in a small shared constants module used by C++ and codegen/docs.

**Effort:** Low if you’re already on generated types; just migrate remaining manual mirrors.

---

## 9. Message buses, caching, and state (NATS, memcached, Redis)

| Where | What |
|-------|------|
| **Messaging** | **NATS** (Core + JetStream) is the primary message bus: pub/sub (`snapshot.{backend}`, `system.health`, `market-data.tick.>`, `strategy.signal.*`), request-reply, JetStream persistence. Python (`nats_client.py`, `questdb_nats_writer.py`), Rust (NATS adapter), C++ (when `ENABLE_NATS`), Go (nats-questdb-bridge) all use it. |
| **State / cache** | **NATS KV** (JetStream Key-Value) is the preferred shared state store (`nats_kv_state.py`; bucket `ib_box_spread_state` for current account, mode). **Redis** exists as an alternative (`redis_cache.py`; NATS KV as fallback today – plan flips to NATS KV primary). **Memcached** appears in backlog (CacheClient abstraction with memcached backend, Ansible provisioning, Python integration cache). |

**Unify:**

- **One message bus for app messaging:** Standardise on **NATS** (Core for fire-and-forget, JetStream for persistence/replay). Keep a single **topic registry** (`docs/NATS_TOPICS_REGISTRY.md`) and **one wire format** (protobuf for NATS payloads; see CROSS_LANGUAGE_DEDUP_PLAN and proto migration tasks). Don’t introduce a second bus (e.g. RabbitMQ, Kafka) for the same use cases unless there is a clear reason (e.g. Kafka for heavy event-sourcing).
- **One abstraction for cache/state:** Use a **unified client** (e.g. `CacheClient` protocol / state factory) so callers don’t branch on “NATS vs Redis vs memcached” in business logic. **Canonical order:** (1) **NATS KV** when NATS is available – one URL for pub/sub and state, no extra process. (2) **Redis** when you need richer structures (hashes, lists, TTL, sorted sets) or a dedicated cache server. (3) **Memcached** only where it’s already mandated (e.g. existing infra) or for pure key-value cache with no KV/store semantics; implement behind the same abstraction so switching is config-driven.
- **C++ engine:** When `ENABLE_NATS` is on, C++ publishes to NATS; any future **market-data or strategy cache** in C++ (e.g. memcached market data cache in backlog) should go through a small adapter that can be swapped (NATS KV, Redis, or memcached) so “where we put cached quotes” is one decision, not duplicated per consumer.

**Effort:** Low for “NATS only + single topic/wire spec”; low–medium for unified state factory (NATS KV primary, Redis/memcached behind same interface); medium if you add C++ cache abstraction and wire it to NATS KV or memcached.

---

## 10. TUI and PWA (unified client behaviour and config)

| Where | What |
|-------|------|
| **TUI (Python Textual)** | `python/tui/`: one snapshot provider at a time (rest / mock / file / nats); `providers.py` (RestProvider, NatsProvider, MockProvider, FileProvider); config from `~/.config/ib_box_spread/tui_config.json` or shared config; bank accounts from Discount Bank/ledger merged into unified positions; status bar shows backend health pills. |
| **PWA (React/TypeScript)** | `web/`: fetches snapshot from `VITE_API_URL` (REST or LEAN); `useBackendServices` and `config/ports` for service URLs and health; account selector fetches from all configured backends in parallel; header status for backends. |

**Unify:**

- **Config:** Use a **single JSON Schema** and **one config file** (home: `~/.config/ib_box_spread/config.json`) for both TUI and PWA so service ports, backend list, and data-source priorities are shared. TUI uses `tui.providerType` / `tui.restEndpoint`; PWA uses `pwa.servicePorts` and env `VITE_*_PORT`. Same `services` and `broker.priorities` so both UIs see the same backends and ordering. See `docs/research/architecture/SHARED_CONFIGURATION_SCHEMA.md`.
- **Snapshot and health contract:** Same **payload shape** for snapshot and health on NATS and REST (proto-generated types in Python and TypeScript). Then TUI NATS provider and PWA (if it adds NATS/WS) consume the same messages; REST fallback returns the same structure. Reduces “TUI has one shape, PWA another.”
- **Provider semantics:** Keep TUI’s “one provider at a time” and PWA’s “parallel fetch from all backends” as UI choices, but **backend list and health** come from the same source (config + `system.health` / health aggregator). Optional: PWA could add a “single-backend” mode that mirrors TUI for consistency in testing or minimal views.
- **Feature parity and types:** Track parity (e.g. FinancingComparator, box spread table, benchmarks) in a short doc or checklist; implement shared behaviour (e.g. risk-free rate, benchmarks) using the same backend APIs and, where possible, **shared types** (proto-generated for TS and Python) so TUI and PWA stay aligned.

**Effort:** Low for “single config schema + same backend list”; low–medium for aligning snapshot/health to one proto shape and using generated types in both clients; medium for full feature-parity and optional PWA NATS/WS.

---

## Summary table

| Logic | Canonical today | Unify by | Effort |
|-------|------------------|----------|--------|
| Mean / percentile / correlation | C++ | Expose via pybind11; Python calls C++ | Low |
| DTE (trading days to expiry) | C++ | Expose `calculate_dte` via pybind11 | Low |
| Risk calculator (full) | C++ | Expose more methods via pybind11; thin Python to wrapper | Medium |
| Config validation | — | Single JSON Schema; same rules everywhere | Low |
| Option/contract string parsing | Split | One spec + optional C++ parser + binding | Low–Medium |
| Discount bank parser | Rust | Keep; finish Python deprecation | Done / low |
| Market hours (holidays, status) | C++ | Optional: expose MarketHours via pybind11 | Medium |
| Enums / constants at boundaries | Proto + generated code | Use generated types; remove manual mirrors | Low |
| Message bus (NATS) + cache/state (NATS KV, Redis, memcached) | NATS primary; NATS KV then Redis | One topic registry + proto wire format; unified CacheClient/state factory; C++ cache behind adapter | Low–Medium |

| TUI and PWA | Split config and payloads | Single config schema + shared snapshot/health shape (proto); same backend list; optional feature-parity checklist | Low–Medium |

---

## References

- `docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md` — Protobuf + single-source strategy; Phases 1–6 and §4.5 (statistical helpers).
- `docs/design/DSL_AND_PROTO_OPPORTUNITIES.md` — Where proto vs DSL fits; config and expressions.
- `docs/planning/NATS_KV_REDIS_LIFECYCLE_TIMESCALE.md` — NATS KV first, Redis later, unified state factory, background-task lifecycle.
- `docs/NATS_TOPICS_REGISTRY.md` — NATS subject names and payload semantics.
- `docs/NATS_SETUP.md` — Current NATS setup, bridge integration, and runtime paths.
- `docs/message_schemas/README.md` — Canonical protobuf/NATS message contracts.
- `docs/research/architecture/SHARED_CONFIGURATION_SCHEMA.md` — Unified config format for TUI, PWA, and standalone; `tui` and `pwa` sections.
- `docs/platform/MULTI_ACCOUNT_AGGREGATION_DESIGN.md` — TUI vs PWA behaviour (one provider vs parallel backends), backend roles.
- `python/integration/cache_client.py` — CacheClient protocol (Redis / memcached / NATS KV behind one interface).
- `python/integration/nats_kv_state.py` — NATS KV state (bucket, get/set helpers).
- `native/include/risk_calculator.h` — C++ `calculate_mean`, `calculate_percentile`, `calculate_correlation`.
- `python/integration/risk_calculator.py` — Python port and statistical helpers.
- `native/src/tws_conversions.cpp` — `tws::calculate_dte` and MarketHours usage.
