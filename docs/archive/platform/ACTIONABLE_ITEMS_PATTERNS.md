# Actionable Items Patterns

**Purpose:** Reusable patterns for **what to do next** after an event or response—gating actions on readiness, order-id lifecycle, callback→follow-up, retries, and non-blocking side effects. Drawn from the sibling **tws-api** samples and our own backend/TUI code.

**Use when:** Adding new TWS/IB flows, order placement, scanner or market-data pipelines, or any "receive → decide → act" logic.

---

## 1. Gate actions on readiness

**Pattern:** Do not perform dependent actions until a readiness signal is received.

**TWS (tws-api):**
- **Place orders only after nextValidId.** The VB Testbed blocks: `While wrapperImpl.nextOrderId = 0 ... Thread.Sleep(100)` then calls `testIBMethods(socketClient, wrapperImpl.nextOrderId)`. So "session ready" = nextValidId received.
- **startApi only after connectAck.** Async samples call `startApi()` inside the `connectAck()` callback. So "can send API requests" = after connectAck.

**Our code:**
- We don’t place orders from the market-data path until strategy is running and we have a valid order id from the adapter when we do place (e.g. BAG). For any new order flow, **gate submission on "session ready"** (e.g. nextValidId or equivalent from ib_adapter).

**Actionable:** Before adding an order-placement or scanner path, define the readiness gate (e.g. "connected + nextValidId received") and enforce it in code and docs.

---

## 2. Order ID lifecycle (never reuse)

**Pattern:** Each order uses a distinct order id; after placing, advance the id so the next order gets a new one.

**TWS (tws-api):**
- Comment in Testbed: *"Placing/modifying an order - remember to ALWAYS increment the nextValidId after placing an order so it can be used for the next one!"*
- Pattern: `client.placeOrder(increment(nextOrderId), contract, order)` where `increment` returns current and bumps the stored id. Multiple clients must use ids greater than any id returned for that account.

**Our code:**
- `ib_adapter` uses `client.next_order_id()` for placement. Ensure we **never reuse** an order id: one id per place_order/place_bag_order call, and rely on the crate/TWS for next id.

**Actionable:** In any path that submits orders, use a single source of truth for "next order id" (TWS nextValidId or crate’s next_order_id) and never submit the same id twice.

---

## 3. Callback → next action (one logical step)

**Pattern:** In the callback, do one clear follow-up: update state, then optionally trigger the next request or side effect.

**TWS (tws-api):**
- **connectAck** → `startApi()` (single next step).
- **nextValidId** → store id, enable order UI or call `testIBMethods(client, nextOrderId)`.
- **scannerData** → display/store row; **scannerDataEnd** → `cancelScannerSubscription(reqId)` and optionally `disconnect()`. If you wanted to subscribe to market data for scanner results, the next action would be `reqMktData` for selected contracts after scannerData/scannerDataEnd.

**Our code:**
- **Market tick** → `apply_market_event` (state), optional `alerts.push` (wide spread), then `strategy_signal.send` + NATS publish. State update first, then side effects (signal, publish).

**Actionable:** For each callback or handler, document "state update" and "next action" (if any). Prefer: update state first, then one or two side effects (send message, spawn task), not long chains inside the callback.

---

## 4. Retry and backoff

**Pattern:** Distinguish retryable vs non-retryable errors; use bounded retries with exponential backoff for RPC; use loop + backoff for connection.

**Our code (nats_adapter/rpc.rs):**
- **RetryConfig:** `max_retries` (default 3), `initial_backoff` (500ms), `max_backoff` (5s). Used by `request_json_with_retry` and `request_json_with_retry_timeout`.
- **is_retryable:** Only retry on `Publish` or `Connection` errors.
- **next_backoff:** Exponential (double per attempt), capped at max_backoff.

**Our code (TWS / IB / Swiftness):**
- Connection failure: loop with `sleep(10s)` or similar, then retry (tws_market_data, tws_positions, ib_positions, swiftness). No exponential backoff in those loops today; could adopt a cap (e.g. 60s) and backoff for consistency with NATS.

**Actionable:** For **request-reply** (NATS api.*, strategy, finance_rates): use `request_json_with_retry` or `request_json_with_retry_timeout` with explicit timeout for slow endpoints. For **connection loops**: consider exponential backoff with a max delay (e.g. 2s → 60s) and clear "Retrying in Xs" logging.

---

## 5. Non-blocking side effects after state update

**Pattern:** Update shared state first; then spawn or send side effects so the hot path doesn’t block on I/O.

**Our code (api/state.rs):**
- `apply_strategy_execution`: applies decision to snapshot; then for `ClosedPosition` (and similar) does `tokio::spawn(async move { ledger::record_position_close(...) })` so ledger write doesn’t block the snapshot update.

**Actionable:** When a handler must both update state and perform I/O (DB, ledger, external API), do: (1) update state, (2) spawn a task or send to a channel for the I/O. Log failures in the spawned task; don’t fail the main callback.

---

## 6. Alerts as actionable feedback

**Pattern:** Push short, user-visible messages into shared state so the UI can show "what’s wrong" or "what to do next."

**Our code:**
- **Wide spread:** `snapshot.alerts.push(Alert::warning("Wide spread detected on {symbol}: {spread}"))` in `handle_market_event`.
- **Cap:** Keep last 32 alerts; drop oldest. Same pattern in swiftness (info after merge).
- TUI shows connection state (e.g. "Retrying", "DOWN") and alerts.

**Actionable:** For conditions that need operator attention (connection down, wide spread, risk limit, scanner no data), push an **Alert** (or equivalent) with a short message. Cap the list and age out old entries. Use in runbooks: "if you see X alert, do Y."

---

## 7. Request-reply timeout and retry (TUI / clients)

**Pattern:** For user-initiated or critical RPC, use retry with backoff; for slow endpoints, use a longer timeout.

**Our code (tui_service/main.rs):**
- Strategy start/stop: `request_json_with_retry` (default timeout and RetryConfig).
- Benchmarks (FRED): `request_json_with_retry_timeout` with longer timeout (e.g. 15s) and RetryConfig so slow external calls don’t fail on first timeout.

**Actionable:** When adding a new NATS request from TUI or CLI: choose `request_json_with_retry` vs `request_json_with_retry_timeout` based on expected latency; document timeout and retry in NATS_API or the handler doc.

---

## 8. Summary table

| Pattern | TWS/samples | Our code | Actionable takeaway |
|--------|-------------|----------|---------------------|
| Gate on readiness | nextValidId before placeOrder; connectAck before startApi | Gate order flow on session/nextValidId | Define readiness; enforce before dependent actions |
| Order id lifecycle | Increment after each placeOrder | Use next_order_id once per order | One id per order; never reuse |
| Callback → next action | connectAck→startApi; scannerDataEnd→cancel | Tick→state + signal + publish | State first; then one or two side effects |
| Retry/backoff | Loop + sleep on connection fail | RetryConfig + exponential backoff for RPC | Retry RPC with config; consider backoff for connection loops |
| Non-blocking side effects | — | apply_strategy_execution → spawn ledger | Update state; spawn I/O |
| Alerts | — | alerts.push; cap 32 | Push actionable messages; cap and age out |
| Request timeout/retry | — | request_json_with_retry(_timeout) | Use retry + longer timeout for slow endpoints |

These patterns are **actionable**: we can apply them when adding or refactoring order placement, scanner flows, connection handling, and TUI/CLI request-reply logic.
