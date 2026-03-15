# NATS API (NATS-only backend)

Backend is NATS-only (no REST). This doc inventories current subjects and proposes the request/reply pattern for former REST areas.

---

## 1. Existing NATS subjects and usage

Source: `agents/backend/crates/nats_adapter/src/topics.rs`, backend_service, tui_service. All payloads use `nats_adapter` protobuf `NatsEnvelope` unless noted.

### Published by backend_service

| Subject | Publisher | Payload | Purpose |
|--------|-----------|---------|--------|
| `snapshot.{backend_id}` | snapshot_publisher | `NatsEnvelope(SystemSnapshot)` | Periodic full state for TUI; interval from config (e.g. 1000 ms). |
| `market-data.tick.{symbol}` | nats_integration | `MarketDataEvent` | Real-time bid/ask from market data loop. |
| `strategy.signal.{symbol}` | nats_integration | `StrategySignal` | Strategy signals (with DLQ on failure). |
| `strategy.decision.{symbol}` | nats_integration | `StrategyDecision` | Strategy decisions. |
| `system.dlq.backend.{error_type}` | DlqService | (varies) | Dead-letter when publish fails. |

`backend_id` comes from env `BACKEND_ID` (default `ib`). TUI subscribes to `snapshot.{backend_id}` only.

### Subscribed by backend_service

| Subject | Consumer | Purpose |
|--------|----------|--------|
| `system.health` | health_aggregation | Health status; sets `health_state.nats_connected`. |
| `NATS_SUBJECTS` (env) | collection_aggregation | Default `LIVE_STATE`; messages merged into shared snapshot and optional LIVE_STATE KV / QuestDB. |

### Subscribed by tui_service

| Subject | Purpose |
|--------|----------|
| `snapshot.{backend_id}` | Single source for dashboard state (positions, orders, risk, alerts, etc.). |

### Defined but not yet used (RPC in topics.rs)

| Subject | Intent |
|--------|--------|
| `rpc.strategy.status` | Request strategy status (reply). |
| `rpc.system.snapshot` | Request system snapshot (reply). |

### Client lifecycle

The **async_nats** `Client` has no explicit `close()`; it closes the connection when the value is dropped. For graceful shutdown, drop the client (or the wrapper that holds it) so the connection tears down.

### Env / config

- `NATS_URL` — Server URL (default `nats://localhost:4222`).
- `BACKEND_ID` — Snapshot subject suffix (default `ib`).
- `NATS_SUBJECTS` — Comma-separated subjects for collection aggregation (default `LIVE_STATE`).
- `NATS_KV_BUCKET`, `NATS_USE_JETSTREAM` — Optional JetStream/KV (Rust collection currently uses core NATS only).

---

## 2. Proposed request/reply pattern (for former REST areas)

**Naming:** `api.<area>.<action>` (e.g. `api.strategy.start`, `api.discount_bank.balance`).

**Flow:** Client sends request to subject; server subscribes and replies on the message’s `reply` inbox. One request → one reply.

**Timeout:** 5 seconds default (match `nats_adapter::rpc::DEFAULT_TIMEOUT`). Overridable per subject if needed.

**Payload:** Prefer **protobuf** (existing `nats_adapter::rpc::request_proto` / `serve_proto`). For areas that are JSON-only today, either define a small proto or use JSON with a dedicated helper (to be added).

**Example (strategy control):**

- Subject: `api.strategy.start` (no suffix; body identifies symbol/strategy if needed).
- Request: proto (e.g. `StrategyStartRequest { symbol: "XSP" }`).
- Reply: proto (e.g. `StrategyStartResponse { ok: true }`).
- Server: backend_service subscribes and calls `StrategyController::start()`; replies with success/failure.

**Scoped areas (to be decided per area in “Human: approve scope” task):**

- Discount Bank — `api.discount_bank.balance`, `.transactions`, `.bank_accounts`, `.import_positions`
- Finance rates — `api.finance_rates.extract`, `.build_curve`, `.compare`, `.yield_curve`, `.benchmarks`
- Calculate — `api.calculate.greeks`, `.iv`, `.hv`, `.risk`, `.strategy`, `.box_spread`, `.jelly_roll`, `.ratio_spread`
- Loans + Config — `api.loans.*`, `api.config.get`, `api.config.update`
- Strategy — `api.strategy.start`, `api.strategy.stop` (and optionally orders/positions read)
- FMP / chart / Swiftness / frontend — TBD per human approval

---

## 3. Approved scope per area (human review)

**Task T-1773515642013883000:** Scope approved. Expose: Discount Bank, Loans (+ Config). Remove: Finance rates, Calculate, Strategy, FMP/chart/Swiftness/frontend.

| Area | Current state | Scope (Expose / Remove) | Notes |
|------|----------------|-------------------------|--------|
| **Discount Bank** | `api/discount_bank.rs` — balance, transactions, bank_accounts, import_positions. No callers after REST removal. | **Expose** | Implement NATS request/reply for balance, transactions, bank_accounts, import_positions. |
| **Finance rates** | `api/finance_rates.rs` — extract, build_curve, compare, yield_curve, benchmarks. No callers after REST removal. | **Remove** | Remove dead code; document as deferred in NATS_API. |
| **Calculate** | `api/quant.rs` — greeks, iv, hv, risk, strategy, box_spread, jelly_roll, ratio_spread. Pure library; no REST left. | **Remove** | Document as library-only; remove any API surface / dead code. |
| **Loans + Config** | `api/loans.rs` (LoanRepository, CRUD). Config was REST-only, now gone. | **Expose** | Implement NATS request/reply for loans; config TBD if needed. |
| **Strategy** | `StrategyController` in backend_service; start/stop today in-process. TUI gets state via snapshot only. | **Remove** | No NATS strategy control; document that start/stop remain in-process only; remove or defer strategy RPC. |
| **FMP / chart / Swiftness / frontend** | FMP in `market_data` crate; chart/Swiftness/frontend were REST handlers, removed. | **Remove** | Remove dead code; document as deferred. |

After editing the table, run the corresponding area subtasks: for **Expose** run the “Implement NATS handlers…” task; for **Remove** run the “Remove dead code and document…” task.
