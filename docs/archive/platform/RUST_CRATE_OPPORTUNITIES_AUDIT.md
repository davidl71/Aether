# Rust: reinventing the wheel – crate opportunities audit

Places where we use hand-rolled or duplicated logic that could be replaced or unified with existing crates. Covers both implemented and stub code.

---

## 1. JSON with comments (strip_json_comments)

**Status:** Resolved. TUI uses **`jsonc-parser::parse_to_serde_value`** in `services/tui_service/src/config.rs` for shared config. The api crate has no `rest.rs` or `strip_json_comments` in the current tree; only TUI loads the JSONC file. No duplicate custom parser remains.

| Crate | Notes |
|-------|--------|
| **jsonc** | Minimal: `parse_jsonc()`, `read_jsonc()`. Good if we only need “parse and then use with serde”. |
| **jsonc-parser** | Richer (AST/CST, serde feature). Heavier. |
| **fjson** | JSONC parse + format; C-style comments and trailing commas. |

**Recommendation:** Add **`jsonc`** (or **`jsonc-parser`** with serde) to a shared crate or to both `api` and `tui_service`; replace both `strip_json_comments` call sites with the crate’s parse, then `serde_json::from_str` on the result (or use the crate’s serde integration if available). Unifies behavior and avoids maintaining our own comment-stripping state machine.

---

## 2. Circuit breaker + exponential backoff (TUI NATS reconnect)

**Current:** `agents/backend/services/tui_service/src/circuit_breaker.rs` — custom circuit breaker (Closed / Open / HalfOpen) and exponential backoff (2^n s, cap 60s). Used by `nats.rs` for reconnect.

**Opportunity:** Use a generic backoff/retry crate; circuit breaker is optional (reconnect loop with backoff is often enough).

| Crate | Notes |
|-------|--------|
| **backoff** | `ExponentialBackoff`, optional jitter, `retry()` for async. No circuit breaker. |
| **tokio-retry** | Strategies (e.g. exponential), works with Tokio. |
| **fail** / **circuit-breaker** | Circuit breaker implementations (less common in async Rust). |

**Recommendation:** Consider **`backoff`** in `tui_service`: use it for the reconnect delay schedule and simplify or remove the custom circuit breaker if “retry with backoff” is enough. If we want to keep “stop trying for 30s after N failures”, we can keep a small state machine or look for a circuit-breaker crate. Low priority unless we add more reconnect/retry sites.

---

## 3. DLQ retry and backoff (nats_adapter)

**Current:** `agents/backend/crates/nats_adapter/src/dlq.rs` and `bridge.rs` — custom `DlqConfig` (initial/max delay, multiplier) and `calculate_retry_delay(attempt)` (exponential backoff). Used when publish fails before sending to DLQ.

**Opportunity:** Same as §2 — use **`backoff`** (or **`tokio-retry`**) for the delay schedule so we don’t maintain our own backoff math. DLQ-specific logic (topic, envelope, metadata) stays; only the “how long to wait before retry” part is replaced.

---

## 4. Config loading (backend_service)

**Current:** `agents/backend/services/backend_service/src/main.rs` — `load_config()`: read `BACKEND_CONFIG` path or default `config/default.toml`, then `toml::from_str`. No env overlay.

**Opportunity:** If we want env overrides (e.g. `BACKEND_REST_ADDR`) or multiple layers:

| Crate | Notes |
|-------|--------|
| **config** | Layered: file + `Environment::with_prefix("APP")`, multiple formats. |
| **figment** | TOML + env, merge order control. |
| **toml-env** | TOML + env overlay. |

**Recommendation:** Keep current approach until we need env-based overrides or multiple config files. Then introduce **`config`** or **`figment`** and document the precedence order. Not reinventing the wheel today; only a future improvement.

---

## 5. Mock market data source

**Current:** `agents/backend/crates/market_data/src/mock.rs` — `MockMarketDataSource` implements `MarketDataSource` with a fixed symbol list and interval; yields deterministic events. Used as the default provider in `backend_service` when no Polygon config.

**Opportunity:** For *unit tests* that need a mock `MarketDataSource`, we could use **mockall** to generate a mock impl. The current type is a real, in-process implementation used in production as the default provider, not only in tests.

**Recommendation:** Keep `MockMarketDataSource` as-is for production default. If we add tests that need a fake “one event then error” or “no events”, we could add a mockall-generated mock for the trait in tests only. Low priority.

---

## 6. Stub / placeholder code (ib_adapter and similar)

**Current:** `agents/backend/crates/ib_adapter/src/lib.rs` — `connect()` sets state to `Connected` without a real TWS connection; `get_account_info()` returns `AccountInfo::default()`. Marked with TODO for real `ibapi` wiring.

**Opportunity:** This is intentional stub code, not a missing crate. The real implementation will use **ibapi** (already in workspace). No replacement needed; only implement when wiring TWS.

**Other stubs:** Any `unimplemented!()` or `todo!()` or “return default” in api/ledger/risk/strategy are placeholders for features, not candidates for a generic crate.

---

## 7. Timeouts and one-off retries

**Current:** Various `tokio::time::timeout()` and manual retry (e.g. `ib_positions.rs` “retry once” on failure). Single-shot timeouts and one retry are simple and readable.

**Recommendation:** Keep ad hoc `timeout()` and “try twice” unless we standardize retry policy (attempts, backoff, which errors to retry). Then consider **backoff** or **tokio-retry** for a consistent policy.

---

## Duplicate code: YYYYMMDD expiry parsing

**Current:** The same “parse expiry string in YYYYMMDD form” logic appears in four places with different return types and error handling:

| Location | Function / code | Returns |
|----------|------------------|--------|
| `crates/quant/src/lib.rs` | `parse_expiry_to_days()` | `Result<i64, QuantError>` |
| `crates/risk/src/calculator.rs` | `parse_expiry_days()` | `Result<i64, ()>` |
| `crates/ib_adapter/src/lib.rs` | `parse_expiry_yyyymmdd()` | `Result<(u16, u8, u8), String>` |
| `crates/quant/src/option_chain.rs` | inline (slice `[0..4]` etc.) | `i32` (0 on error) |

**Recommendation:** Add a small shared helper (e.g. in `api` or a tiny `common` crate) that parses YYYYMMDD and returns a date or days-since-today; have quant, risk, and ib_adapter call it and map to their error types. Low priority unless expiry parsing rules or formats change.

**Dependencies that implement similar features:** The workspace already has crates that can parse YYYYMMDD: **time** (quant, risk) with features `parsing` and `macros` — `Date::parse(s, &format_description!("[year][month][day]"))`; **chrono** (api, ledger, tui_service) — `NaiveDate::parse_from_str(s, "%Y%m%d")`. We use a zero-dependency implementation in `crates/common` so `ib_adapter` does not need `time` or `chrono`. See `crates/common/src/expiry.rs` for links.

---

## Summary table

| Area | Current | Suggested crate | Priority |
|------|--------|------------------|----------|
| JSON with comments | Resolved: tui uses **jsonc-parser** | — | Done |
| Expiry parsing (YYYYMMDD) | Resolved: **common::expiry::parse_expiry_yyyy_mm_dd** in `crates/common` | — | Done |
| TUI NATS backoff/circuit breaker | Custom in tui_service; uses **backoff** crate | — | Low |
| nats_adapter DLQ retry delay | Uses **backoff** crate | — | Done |
| Backend config | toml + path only | **config** / **figment** when env overlay needed | Later |
| Mock market data | Concrete `MockMarketDataSource` | Keep; mockall only if we need test-only mocks | Low |
| ib_adapter stubs | Defaults + TODO | Implement with ibapi when ready | N/A |

---

## References

- **jsonc**: <https://docs.rs/jsonc>
- **jsonc-parser**: <https://docs.rs/jsonc-parser>
- **backoff**: <https://docs.rs/backoff>
- **config**: <https://docs.rs/config>
- **figment**: <https://docs.rs/figment>
- **mockall**: <https://docs.rs/mockall> (for trait mocks in tests)
