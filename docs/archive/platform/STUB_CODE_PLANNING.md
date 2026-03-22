# Stub code audit and planning

Audit of stub, placeholder, and dead-code paths in the Rust backend. Classified into **implementable with no questions** vs **human review / design**.

---

## 1. Implementable with no questions

| Location | Current state | Action |
|----------|----------------|--------|
| **api/rest.rs** — FMP fundamentals | ~~Handlers exist; routes commented out; `#[allow(dead_code)]`.~~ **Done.** | Implemented: routes wired, dead_code removed. Fix: added `Serialize` to FMP response types in `market_data/src/fmp.rs` (`IncomeStatement`, `BalanceSheet`, `CashFlowStatement`, `FmpQuote`) so `Result<Json<Vec<T>>, ...>` satisfies `IntoResponse`. Requires `FMP_API_KEY` env for real data. |

---

## 2. Human review / design

| Location | Current state | Decision needed |
|----------|----------------|-----------------|
| **ib_adapter** | Full stub: `connect`/`disconnect` no-op; `request_market_data`, `request_option_chain`, `place_order`, `cancel_order`, `request_positions`, `request_account` return empty/default. TODOs reference ibapi and task IDs. | Scope: paper vs live, account selection, which methods to implement first. Security/review before wiring real TWS. |
| **api/runtime_state.rs** | `decisions_by_producer_type`, `find_by_correlation_id`, `positions_by_account` return `Vec::new()`; comments say "placeholder for when decisions store producer metadata" / "account_id". | Data model: add `producer_type`, `correlation_id`, `account_id` to state structs? Where do values come from (NATS envelope, config)? |
| **tui_service/models.rs** | `SnapshotSource` has `#[allow(dead_code)]` "remove when REST fallback is wired". `SnapshotSource` is used by `TuiSnapshot.source`. | Confirm REST fallback is wired and used; if yes, remove `#[allow(dead_code)]`. |
| **tui_service/events.rs** | Module has `#![allow(dead_code)]` "remove when structured event routing is wired". `EventRouter`, `EventPriority`, `EventCategory` exist but may not be fully used. | **Deferred.** Per-component event routing is deferred until the app grows to need it. Trigger and approach documented in `tui_service/src/main.rs` `run_loop()` (TODO T-1773509396768932000): consider ratatui/async-template component model when adding per-component routing. Parent evaluation task: T-1773357423959019000. |
| **tui_service/Cargo.toml** | TODOs for tui-widgets, ratatui-textarea, sparkline column. | Product/UX: which of these to add and when. |
| **risk/var.rs** | Comment: "Monte Carlo and scenario analysis stubs from C++ are omitted — they were unimplemented there too." | No stub code to implement. Optional: task "Consider Monte Carlo VaR if needed" for product. |

---

## 3. Exarp tasks created

- **Implementation (no questions):** Wire FMP fundamentals routes to API router — **done** (T-1773514701329764000); in Review.
- **Human review:**  
  - IB adapter implementation scope and approach.  
  - runtime_state placeholders — add producer/correlation/account fields and populate?  
  - TUI REST fallback — remove SnapshotSource dead_code once verified?  
  - TUI structured event routing — wire now or defer? **Resolved:** Deferred until app grows to need per-component routing (see §2 table and main.rs run_loop TODO T-1773509396768932000).

---

## 4. References

- FMP handlers: `agents/backend/crates/api/src/rest.rs` (fundamentals_*).
- Router: same file, `router()` around lines 217–220 (commented fundamentals routes).
- ib_adapter: `agents/backend/crates/ib_adapter/src/lib.rs`.
- runtime_state: `agents/backend/crates/api/src/runtime_state.rs` (decisions_by_producer_type, find_by_correlation_id, positions_by_account).
- TUI: `agents/backend/services/tui_service/src/models.rs` (SnapshotSource), `events.rs` (EventRouter).
