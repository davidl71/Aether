# Logic We Could Unify

**Purpose:** Concrete areas where logic is duplicated across C++, Python, or Rust and how to unify them (single canonical implementation + bindings or shared spec). Complements `docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md` and `docs/design/DSL_AND_PROTO_OPPORTUNITIES.md`.

**Last updated:** 2026-03-11

---

## 1. Statistical helpers (mean, percentile, correlation)

**Status: Done ✅**

| Where | What |
|-------|------|
| **C++** | `calculate_mean`, `calculate_percentile`, `calculate_correlation` in `native/src/risk_calculator_stats.cpp`. |
| **Python helper surface** | The active Python-facing path is the pybind11 module plus helper/test code under `native/tests/python/`; the former `python/integration/risk_calculator.py` path was part of the retired Python app tree. |

All three functions are exposed in `native/src/box_spread_pybind.cpp` (lines 169–182).
Python imports them as `_cxx_calculate_mean`, `_cxx_calculate_percentile`, `_cxx_calculate_correlation`
from `box_spread_bindings`; the Python fallback is intentional for environments without the compiled extension.

---

## 2. DTE (days to expiry) and trading-day logic

**Status: Done ✅**

| Where | What |
|-------|------|
| **C++** | `tws::calculate_dte(expiry_str)` in `tws_conversions.cpp` (uses `market_hours::MarketHours` internally). |
| **Python** | `calculate_dte` exposed via pybind11; Python code that previously used ad-hoc date math can call the binding. |

`calculate_dte` is bound in `native/src/box_spread_pybind.cpp` (lines 187–190).
`MarketHours` is used internally — Python callers get correct trading-day counts without
needing direct `MarketHours` access. For the broader `MarketHours` class, see §7.

---

## 3. Risk calculator (beyond stats)

**Status: Partial — stats done; class-level methods still open**

| Where | What |
|-------|------|
| **C++** | Full `RiskCalculator` (VaR, correlation risk, position sizing, aggregate Greeks, etc.) in `native/src/risk_calculator.cpp`. |
| **Historical Python port** | The former `python/integration/risk_calculator.py` implementation was a 613-LOC Python port; stats helpers called C++ (§1) and the rest was a documented stub. |

Stats (`calculate_mean`, `calculate_percentile`, `calculate_correlation`) are exposed and used
(§1 above). The full `RiskCalculator` class methods (VaR, Greeks, correlation risk, position sizing)
are not yet bound. The Python port is accepted technical debt and formally documented as a stub
(Phase 5 of CROSS_LANGUAGE_DEDUP_PLAN).

**Next step (open):** Expose more `RiskCalculator` methods via pybind11 (e.g. `calculate_correlation_risk`,
`calculate_var`, position sizing) so any surviving Python helpers can stay thin wrappers around the
native implementation. This is a medium-effort change that would give a single implementation for
regulatory/audit purposes.

**Effort:** Medium (design binding surface; add `m.def()` calls; migrate callers).

---

## 4. Config loading and validation

**Status: Done ✅**

| Where | What |
|-------|------|
| **C++** | `ConfigManager::load()`, `from_json` for `StrategyParams`, `RiskConfig`, etc. in `native/`. |
| **Frontend config readers** | Historical Python TUI code and the current Rust TUI both read the shared config shape from `~/.config/ib_box_spread/config.json`; the active TUI runtime is now Rust. |

`config/schema.json` (JSON Schema v2020-12) is the single schema covering TUI and C++ CLI config.
TUI uses it as the primary load path; C++ config manager validates against the same structure.
PWA section is retained in the schema for historical reference but is inactive (PWA archived).

---

## 5. Option / contract string parsing

**Status: Not true duplication — different input formats, each parser is canonical**

| Where | What |
|-------|------|
| **C++** | `parse_option_symbol()` in `native/src/tws_client.cpp` — parses TWS OCC-style symbol strings. |
| **Historical Python helper** | `parse_opt_contract_desc()` previously lived in `python/integration/combo_detector.py` and parsed IB Client Portal `contractDesc` strings (e.g. `"SPX MAR2027 6825 C [...]"`). |

These parsers serve different data sources (TWS vs Client Portal) and use different input formats.
They are not duplicates of each other.

**Remaining gap:** A single format spec doc linking the two representations (contractDesc vs OCC symbol)
would help future contributors understand when each applies.

**Effort:** Low (write a short spec note in `docs/`).

---

## 6. Discount bank parser

**Status: Done ✅**

| Where | What |
|-------|------|
| **Rust** | Canonical parser in `agents/backend/crates/discount_bank_parser`. |
| **Historical Python helper** | `python/integration/discount_bank_helpers.py` called the Rust binary as the primary path; the Python fallback was already legacy/deprecated before that tree was retired. |

Rust binary (`show_balances`) is the canonical path. `discount_bank_helpers.py` tries the compiled
binary first, then `cargo run`; the Python fallback parser is kept only for environments where the
binary is unavailable and is marked as deprecated in comments.
Matches CROSS_LANGUAGE_DEDUP_PLAN Phase 6 Done status.

---

## 7. Market hours and calendar

**Status: Partial — DTE done (§2); full MarketHours class not yet exposed**

| Where | What |
|-------|------|
| **C++** | `market_hours::MarketHours` in `native/src/market_hours.cpp` — holidays, early closes, session status, DST. Used internally by `calculate_dte`. |
| **Python** | No full port; `calculate_dte` via pybind11 covers the most common use case. |

`calculate_dte` gives Python callers correct trading-day counts (§2 done).
The full `MarketHours` API (`get_market_status_at(time)`, `is_holiday(date)`, `is_early_close(date)`)
is not yet exposed. This is optional — no Python code currently needs it directly.

**Next step (open / optional):** If Python TUI, benchmarks, or rate-curve logic needs "is this a
trading day?" or "what is the market status right now?", expose a small `MarketHours` API surface
via pybind11. Otherwise defer.

**Effort:** Medium (bind API surface; document behaviour and holiday calendar coverage).

---

## 8. Constants and enums

**Status: Done ✅**

| Where | What |
|-------|------|
| **C++** | `types.h` (`OptionType`, `OrderStatus`, etc.); config defaults. |
| **Python** | Generated types from `native/generated/python/` (betterproto output from `proto/messages.proto`); no manual `proto_types.py` mirror. |
| **Proto** | Enums and message field semantics in `proto/messages.proto`. |

`native/generated/python/` is the active generated output for Python helpers and tests. There is no
manual `proto_types.py` mirror — the generated types are the source of truth for Python at boundaries.
Numeric constants (default min_roi, max_dte) live in `config/schema.json` defaults.

---

## 9. Message buses, caching, and state (NATS, memcached)

**Status: Done ✅**

| Where | What |
|-------|------|
| **Messaging** | NATS (Core + JetStream) is the standard bus for all languages. `docs/NATS_TOPICS_REGISTRY.md` is the single topic registry. |
| **State / cache** | NATS KV is the preferred shared live-state store. Memcached is optional/behind a C++ cache abstraction only. |
| **Wire format** | protobuf (`NatsEnvelope` wrapping inner payloads from `proto/messages.proto`) for all platform events. |

The topic registry (`docs/NATS_TOPICS_REGISTRY.md`) is comprehensive and current.
The retired Python TUI used a `NatsProvider` that subscribed to `snapshot.{backend_id}` and
`system.health` using the proto types. In the active runtime, Rust owns the TUI-side NATS reads.
C++ publishes via `ENABLE_NATS`; Rust owns collection and live-state fanout; Go agents use
structured NATS. No second bus was introduced.

---

## 10. TUI alignment (PWA archived)

**Status: TUI aligned ✅; PWA archived**

| Where | What |
|-------|------|
| **TUI (Rust/Ratatui)** | `agents/backend/services/tui_service/` — uses the shared config and Rust-owned NATS/read paths. |
| **Historical Python TUI** | The deleted `python/tui/` tree previously used shared config, a Python NatsProvider, and proto-generated types. |
| **PWA (React/TypeScript)** | `web/` — archived for now; not actively maintained. |

The active TUI uses the shared JSON Schema config, the Rust API base (`http://localhost:8080`) as the
default read path, and Rust-owned NATS/proto integrations.
PWA is archived; feature-parity tracking between TUI and PWA is no longer applicable.

**Remaining TUI work:** See `docs/platform/TUI_RUST_READ_PATH_AUDIT.md` for next migration
candidates (Discount Bank read path, deeper Python read-model reduction).

---

## Summary table

| Logic | Canonical | Status | Open work |
|-------|-----------|--------|-----------|
| Mean / percentile / correlation | C++ | **Done** ✅ | — |
| DTE (trading days to expiry) | C++ | **Done** ✅ | — |
| Risk calculator (full class) | C++ | **Partial** | Expose VaR/Greeks via pybind11; thin Python stub |
| Config validation | JSON Schema (`config/schema.json`) | **Done** ✅ | — |
| Option/contract string parsing | Each parser canonical for its source | **Clarified** | Optional: format spec doc |
| Discount bank parser | Rust | **Done** ✅ | — |
| Market hours (holidays, status) | C++ | **Partial** | `calculate_dte` done; MarketHours class optional/open |
| Enums / constants at boundaries | Proto + `native/generated/python/` | **Done** ✅ | — |
| Message bus (NATS) + cache/state | NATS / NATS KV | **Done** ✅ | — |
| TUI alignment | Rust backend + shared config | **Done** ✅ (PWA archived) | TUI Discount Bank path (see audit doc) |

---

## References

- `docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md` — Protobuf + single-source strategy; Phases 1–6 all Done.
- `docs/design/DSL_AND_PROTO_OPPORTUNITIES.md` — Where proto vs DSL fits; config and expressions.
- `docs/NATS_TOPICS_REGISTRY.md` — NATS subject names and payload semantics.
- `docs/NATS_SETUP.md` — Current NATS setup, bridge integration, and runtime paths.
- `docs/message_schemas/README.md` — Canonical protobuf/NATS message contracts.
- `docs/research/architecture/SHARED_CONFIGURATION_SCHEMA.md` — Unified config format for TUI and standalone.
- `docs/platform/TUI_RUST_READ_PATH_AUDIT.md` — TUI read-path migration status and next candidates.
- `docs/platform/MULTI_ACCOUNT_AGGREGATION_DESIGN.md` — TUI backend roles and ownership.
- `native/src/box_spread_pybind.cpp` — pybind11 bindings: stats helpers, `calculate_dte`, box spread math.
- `native/include/risk_calculator.h` — C++ `calculate_mean`, `calculate_percentile`, `calculate_correlation`.
- historical `python/integration/risk_calculator.py` — Python stub with C++ stats helpers; full class was still Python.
- `native/src/tws_conversions.cpp` — `tws::calculate_dte` and `MarketHours` usage.
- `native/generated/python/` — active generated protobuf output for Python helper surfaces.
- `config/schema.json` — Shared JSON Schema for all config loaders.
