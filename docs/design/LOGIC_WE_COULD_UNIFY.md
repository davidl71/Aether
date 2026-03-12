# Logic We Could Unify

**Purpose:** Concrete areas where logic is duplicated across C++, Python, or Rust and how to unify them (single canonical implementation + bindings or shared spec). Complements `docs/planning/CROSS_LANGUAGE_DEDUP_PLAN.md` and `docs/design/DSL_AND_PROTO_OPPORTUNITIES.md`.

**Last updated:** 2026-03-12

---

## 1. Statistical helpers (mean, percentile, correlation)

**Status: Done ✅**

| Where | What |
|-------|------|
| **C++** | `calculate_mean`, `calculate_percentile`, `calculate_correlation` in `native/src/risk_calculator_stats.cpp`. |
| **Rust** | `agents/backend/crates/risk/src/stats.rs` — native Rust ports used by the Rust backend API. |

Python layer has been archived; the C++ implementation remains canonical for the C++ engine.

---

## 2. DTE (days to expiry) and trading-day logic

**Status: Done ✅**

| Where | What |
|-------|------|
| **C++** | `tws::calculate_dte(expiry_str)` in `tws_conversions.cpp` (uses `market_hours::MarketHours` internally). |

Python layer has been archived. `calculate_dte` is the internal C++ boundary function; no
cross-language binding is needed.

---

## 3. Risk calculator (beyond stats)

**Status: Done ✅ — Python layer archived; Rust has native implementation**

| Where | What |
|-------|------|
| **C++** | Full `RiskCalculator` (VaR, correlation risk, position sizing, aggregate Greeks) in `native/src/risk_calculator.cpp`. Canonical for C++ callers. |
| **Rust** | `agents/backend/crates/risk/` — pure-Rust implementations of sizing, stats, VaR, ES, drawdown, Sharpe/Sortino/Calmar/IR (63 tests). |
| **Python** | `python/` has been archived. No Python risk layer remains. |

The pybind11 binding surface described in earlier versions of this doc was never built — the source
file was removed along with the Python layer. The Rust crate is now the cross-language risk
implementation used by the Rust backend API.

---

## 4. Config loading and validation

**Status: Done ✅**

| Where | What |
|-------|------|
| **C++** | `ConfigManager::load()`, `from_json` for `StrategyParams`, `RiskConfig`, etc. in `native/`. |
| **Python** | `SharedConfigLoader.load_config()` in `python/tui/config.py`; reads `~/.config/ib_box_spread/config.json`. |

`config/schema.json` (JSON Schema v2020-12) is the single schema covering TUI and C++ CLI config.
TUI uses it as the primary load path; C++ config manager validates against the same structure.
PWA section is retained in the schema for historical reference but is inactive (PWA archived).

---

## 5. Option / contract string parsing

**Status: Not true duplication — different input formats, each parser is canonical**

| Where | What |
|-------|------|
| **C++** | `parse_option_symbol()` in `native/src/tws_client.cpp` — parses TWS OCC-style symbol strings. |
| **Python** | `parse_opt_contract_desc()` in `python/integration/combo_detector.py` — parses IB Client Portal `contractDesc` strings (e.g. `"SPX MAR2027 6825 C [...]"`). |

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
| **Python** | `python/integration/discount_bank_helpers.py` calls the Rust binary as the primary path; Python fallback is legacy/deprecated. |

Rust binary (`show_balances`) is the canonical path. `discount_bank_helpers.py` tries the compiled
binary first, then `cargo run`; the Python fallback parser is kept only for environments where the
binary is unavailable and is marked as deprecated in comments.
Matches CROSS_LANGUAGE_DEDUP_PLAN Phase 6 Done status.

---

## 7. Market hours and calendar

**Status: Done ✅ — Python layer archived; no open work**

| Where | What |
|-------|------|
| **C++** | `market_hours::MarketHours` in `native/src/market_hours.cpp` — holidays, early closes, session status, DST. Used internally by `calculate_dte`. |

The Python layer has been archived. `calculate_dte` remains the exported boundary function for
C++ callers. The full `MarketHours` class is internal to the C++ engine; no cross-language
exposure is needed.

---

## 8. Constants and enums

**Status: Done ✅**

| Where | What |
|-------|------|
| **C++** | `types.h` (`OptionType`, `OrderStatus`, etc.); config defaults. |
| **Python** | Generated types from `python/generated/` (betterproto output from `proto/messages.proto`); no manual `proto_types.py` mirror. |
| **Proto** | Enums and message field semantics in `proto/messages.proto`. |

`python/generated/` is the active generated output. The TUI NatsProvider imports directly:
`from python.generated.ib.platform import v1 as pb_v1`. There is no manual `proto_types.py`
mirror — the generated types are the source of truth for Python at boundaries.
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
C++ publishes via `ENABLE_NATS`; Rust owns collection and live-state fanout; Go agents
use structured NATS. No second bus was introduced.

---

## 10. TUI alignment (Python TUI and PWA archived)

**Status: Done ✅**

| Where | What |
|-------|------|
| **TUI (Ratatui)** | `agents/backend/crates/tui/` — active frontend; reads from Rust backend API and NATS. |
| **TUI (Python Textual)** | `python/tui/` — archived. |
| **PWA (React/TypeScript)** | `web/` — archived. |

All read paths are now Rust-backed. See `docs/platform/TUI_RUST_READ_PATH_AUDIT.md` for
migration-complete status table.

---

## Summary table

| Logic | Canonical | Status | Open work |
|-------|-----------|--------|-----------|
| Mean / percentile / correlation | C++ | **Done** ✅ | — |
| DTE (trading days to expiry) | C++ | **Done** ✅ | — |
| Risk calculator (full class) | C++ + Rust | **Done** ✅ | Python archived; Rust crate has native impl |
| Config validation | JSON Schema (`config/schema.json`) | **Done** ✅ | — |
| Option/contract string parsing | Each parser canonical for its source | **Clarified** | Optional: format spec doc |
| Discount bank parser | Rust | **Done** ✅ | — |
| Market hours (holidays, status) | C++ | **Done** ✅ | Python archived; no cross-language exposure needed |
| Enums / constants at boundaries | Proto + `python/generated/` | **Done** ✅ | — |
| Message bus (NATS) + cache/state | NATS / NATS KV | **Done** ✅ | — |
| TUI alignment | Rust backend + shared config | **Done** ✅ | Python TUI + PWA archived; Ratatui TUI active |

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
- `agents/backend/crates/risk/src/` — Rust native risk implementations (sizing, stats, var).
- `native/src/tws_conversions.cpp` — `tws::calculate_dte` and `MarketHours` usage.
- `config/schema.json` — Shared JSON Schema for all config loaders.