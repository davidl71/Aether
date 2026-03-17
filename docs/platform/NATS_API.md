# NATS API (NATS-only backend)

Backend is NATS-only (no REST). This document defines (1) subject naming, (2) request/reply patterns, and (3) scope of the NATS-only API.

**Design overview**

- **Subjects:** Dot-separated tokens, lowercase; concrete subjects for publish, wildcards (`*`, `>`) for subscribe. Patterns: `snapshot.{backend_id}`, `api.<area>.<action>`, `<domain>.<type>.<resource>`, `system.<topic>`, optional `rpc.<area>.<action>`. See §1.
- **Request/reply:** One request → one reply on `api.<area>.<action>`; client sets reply inbox; server responds with payload (JSON or protobuf); 5s default timeout. No streaming on API subjects. See §2.
- **Scope:** Snapshot, Discount Bank, Loans, optional FMP, finance rates, strategy start/stop are in scope; calculate, config API, and deferred orders/positions list are out of scope. See §3.

---

## 1. Subject naming

### Convention

- **Tokens:** Lowercase, dot-separated; use `{placeholder}` only in this doc to denote a variable segment (e.g. `{backend_id}`, `{symbol}`). In code, use the actual value.
- **Wildcards:** `*` (single token), `>` (one or more tokens). Subscribers use these; publishers use concrete subjects.
- **Max length:** 256 characters (see `nats_adapter::topics::validate_topic`).

### Patterns

| Pattern | Example | Purpose |
|--------|---------|--------|
| **Snapshot** | `snapshot.{backend_id}` | Periodic full state from backend to TUI. `backend_id` from env `BACKEND_ID` (default `ib`). Subscribe to all: `snapshot.>`. |
| **API request/reply** | `api.<area>.<action>` | Request/reply for former REST areas. One subject per operation (e.g. `api.loans.list`, `api.discount_bank.balance`). |
| **Event (fire-and-forget)** | `<domain>.<type>.<resource>` | Stream of events; no reply. Examples: `market-data.tick.{symbol}`, `strategy.decision.{symbol}`. |
| **System** | `system.<topic>` | Health, DLQ, config. Examples: `system.health`, `system.dlq.{component}.{error_type}`. |
| **RPC (legacy/optional)** | `rpc.<area>.<action>` | Optional request/reply; see topics.rs. Examples: `rpc.strategy.status`, `rpc.system.snapshot`. |

### Registry (code)

- **Event and system subjects:** `agents/backend/crates/nats_adapter/src/topics.rs` (domains: `market_data`, `strategy`, `orders`, `positions`, `risk`, `system`, `dlq`, `snapshot`, `rpc`).
- **API subjects:** `agents/backend/services/backend_service/src/api_handlers.rs` (constants `api.<area>.<action>`).

### Wiring table (subject → code)

| Area / subject prefix | Subscriptions (server) | Implementation (api crate / backend) | Clients (request) |
|------------------------|------------------------|----------------------------------------|--------------------|
| `api.discount_bank.*` | `backend_service/src/api_handlers.rs` | `api/src/discount_bank.rs`; mock: `api/src/mock_data.rs` | — |
| `api.loans.*` | `backend_service/src/api_handlers.rs` | `api/src/loans.rs`; mock: `api/src/mock_data.rs` | — |
| `api.fmp.*` | `backend_service/src/api_handlers.rs` | `market_data/src/fmp.rs`; mock: `api/src/mock_data.rs` | — |
| `api.finance_rates.*` | `backend_service/src/api_handlers.rs` | `api/src/finance_rates.rs`; mock: `api/src/mock_data.rs` (sofr, treasury) | TUI: `tui_service/src/main.rs` (build_curve, benchmarks); `tui_service/src/ui/yield_curve.rs` |
| `api.strategy.start` / `stop` / `cancel_all` | `backend_service/src/api_handlers.rs` | `api_handlers.rs` (StrategyController); cancel_all reads snapshot, broker cancel not wired | TUI: `tui_service/src/main.rs` (StrategyCommand) |
| `snapshot.{backend_id}` | — | Published: `backend_service` (snapshot_publisher, rest_snapshot); NATS only | Subscribed: `tui_service` (single source for dashboard) |

Single source for all API subject constants and handler wiring: `agents/backend/services/backend_service/src/api_handlers.rs`. JSON request/reply helpers: `nats_adapter` (e.g. `request_json` / reply).

### Publish / Subscribe / Request–Reply component table

| Pattern | Component | Subject(s) | Payload / role | Notes | Protocol | Next steps |
|---------|-----------|------------|----------------|-------|----------|------------|
| **Publish (broadcast)** | backend_service | `snapshot.{backend_id}` | `NatsEnvelope(SystemSnapshot)` | Periodic full state; interval from `SNAPSHOT_PUBLISH_INTERVAL_MS` (default 1000 ms). When `NATS_USE_JETSTREAM=1`, also stored in JetStream stream `SNAPSHOTS` for replay (late-joining clients). | NATS Core pub/sub or JetStream | Stream `SNAPSHOTS` (subjects `snapshot.>`, max 1 msg per subject, 1h max age) when JetStream enabled. Interval is backend-wide; per-client tuning deferred. |
| **Publish (broadcast)** | backend_service | `market-data.tick.{symbol}` | `MarketDataEvent` | Real-time bid/ask; one message per symbol/tick. | NATS Core pub/sub | Optional: set `NATS_TICK_BATCH_MS` to buffer ticks and flush at interval (reduces KV/QuestDB write rate). |
| **Publish (broadcast)** | backend_service | `strategy.signal.{symbol}` | `StrategySignal` | Strategy signals; DLQ on failure. | NATS Core pub/sub | — |
| **Publish (broadcast)** | backend_service | `strategy.decision.{symbol}` | `StrategyDecision` | Strategy decisions. | NATS Core pub/sub | — |
| **Publish (broadcast)** | backend_service | `system.dlq.backend.{error_type}` | (varies) | Dead-letter when publish fails. | NATS Core pub/sub | Backend subscribes to `system.dlq.backend.>` and logs each DLQ message (dlq_consumer). |
| **Subscribe** | backend_service | `system.health` | — | Health aggregation; sets `health_state.nats_connected`. | NATS Core pub/sub | — |
| **Subscribe** | backend_service | `system.dlq.backend.>` | — | DLQ consumer; logs each message for alerting/replay visibility. | NATS Core pub/sub | See `backend_service/src/dlq_consumer.rs`. |
| **Subscribe** | backend_service | `NATS_SUBJECTS` (env, e.g. `LIVE_STATE`) | — | Collection aggregation; merged into snapshot and optional KV/QuestDB. When `NATS_TICK_BATCH_MS` > 0, `market-data.tick.>` messages are buffered and flushed at that interval (last value per symbol). | NATS Core (queue sub optional) | Document exact subject list per env. |
| **Subscribe** | backend_service | `api.discount_bank.*`, `api.loans.*`, `api.fmp.*`, `api.finance_rates.*`, `api.strategy.start` / `stop` / `cancel_all` | Request body (JSON) | Request–reply **server**: receives request, replies to message reply inbox. Uses **queue subscriptions** (queue group from `NATS_API_QUEUE_GROUP`, default `api`) so multiple backends share load. | NATS Core request–reply | New api.* endpoints may use protobuf via `nats_adapter::rpc`; see § Proto for new endpoints. |
| **Subscribe** | tui_service | `snapshot.{backend_id}` | — | Single source for dashboard state (positions, orders, risk, alerts). | NATS Core pub/sub | — |
| **Request** | tui_service (or any client) | `api.loans.list`, `api.finance_rates.build_curve`, `api.finance_rates.benchmarks`, `api.strategy.start` / `stop` / `cancel_all`, etc. | Request body (JSON) | Client sends one request; expects one reply (or timeout). | NATS Core request–reply | `nats_adapter::request_json_with_retry` / `request_json_with_retry_timeout` with exponential backoff (default 3 retries). TUI uses them for strategy and finance_rates. |
| **Reply** | backend_service | (reply inbox from request) | Response body (JSON) | One reply per request; 5s default timeout. | NATS Core request–reply | — |

**Pattern summary:** Broadcast = publish to a subject (all subscribers get it). Request–reply = client publishes to `api.<area>.<action>`, server subscribes and replies to the request’s reply inbox. No streaming on API subjects.

### Current subject inventory

**Published by backend_service**

| Subject | Payload | Purpose |
|--------|---------|--------|
| `snapshot.{backend_id}` | `NatsEnvelope(SystemSnapshot)` | Periodic full state for TUI (interval from config, e.g. 1000 ms). |
| `market-data.tick.{symbol}` | `MarketDataEvent` | Real-time bid/ask from market data loop. |
| `strategy.signal.{symbol}` | `StrategySignal` | Strategy signals (DLQ on failure). |
| `strategy.decision.{symbol}` | `StrategyDecision` | Strategy decisions. |
| `system.dlq.backend.{error_type}` | (varies) | Dead-letter when publish fails. |

**Subscribed by backend_service**

| Subject / config | Purpose |
|------------------|--------|
| `system.health` | Health aggregation; sets `health_state.nats_connected`. |
| `NATS_SUBJECTS` (env) | Collection aggregation (default `LIVE_STATE`); merged into shared snapshot and optional LIVE_STATE KV / QuestDB. |

**Subscribed by tui_service**

| Subject | Purpose |
|--------|---------|
| `snapshot.{backend_id}` | Single source for dashboard state (positions, orders, risk, alerts). |

**Request/reply (API) — implemented**

| Subject | Request | Response | Handler |
|--------|---------|----------|--------|
| `api.discount_bank.balance` | (optional body) | JSON | Discount Bank balance |
| `api.discount_bank.transactions` | JSON `{ "limit": N }` | JSON | Transactions |
| `api.discount_bank.bank_accounts` | (optional) | JSON | Bank accounts |
| `api.discount_bank.import_positions` | JSON (query) | JSON | Import positions |
| `api.loans.list` | (optional) | JSON | List loans |
| `api.loans.get` | JSON `{ "loan_id": "..." }` | JSON | Get one loan |
| `api.loans.create` | JSON `LoanRecord` | JSON `Result<(), String>` | Create loan |
| `api.loans.update` | JSON `LoanRecord` | JSON `Result<(), String>` | Update loan |
| `api.loans.delete` | JSON `{ "loan_id": "..." }` | JSON `Result<bool, String>` | Delete loan |
| `api.fmp.income_statement` | JSON (symbol) | JSON | FMP fundamentals (when `FMP_API_KEY` set) |
| `api.fmp.balance_sheet` | JSON (symbol) | JSON | FMP fundamentals |
| `api.fmp.cash_flow` | JSON (symbol) | JSON | FMP fundamentals |
| `api.fmp.quote` | JSON (symbol) | JSON | FMP quote |
| `api.finance_rates.extract` | JSON `BoxSpreadInput` | JSON `RatePointResponse` or `{ "error": "..." }` | Extract single rate point from box spread input |
| `api.finance_rates.build_curve` | JSON `CurveRequest` (array or `{ opportunities, symbol }`) | JSON `CurveResponse` or error | Build rate curve from opportunities |
| `api.finance_rates.compare` | JSON `CompareRequest` + optional `symbol` | JSON `Vec<ComparisonResponse>` or error | Compare box spread curve to benchmarks (requires `FRED_API_KEY` for live benchmarks) |
| `api.finance_rates.yield_curve` | JSON `YieldCurveComparisonRequest` | JSON `YieldCurveComparisonResponse` | Yield curve comparison (box vs treasury/SOFR) |
| `api.finance_rates.benchmarks` | (optional body) | JSON `{ "sofr": ..., "treasury": ... }` | Combined SOFR + Treasury benchmarks |
| `api.finance_rates.sofr` | (optional body) | JSON `SofrBenchmarksResponse` | SOFR overnight + term rates |
| `api.finance_rates.treasury` | (optional body) | JSON `TreasuryBenchmarksResponse` | Treasury benchmark rates |
| `api.ib.positions` | (optional) JSON `{ "account_id": "DU123" }` | JSON array of `IbPositionDto` or `{ "error": "..." }` | IB Client Portal positions (requires `IB_PORTAL_URL`) |

**Request/reply (API) — not implemented**

| Subject pattern | Note |
|-----------------|------|
| *(none)* | Finance rates implemented; see above. |

**RPC (defined in topics.rs, not yet wired)**

| Subject | Intent |
|--------|--------|
| `rpc.strategy.status` | Request strategy status (reply). |
| `rpc.system.snapshot` | Request system snapshot (reply). |

---

## 2. Request/reply patterns

### Flow

1. Client publishes a **request** to a subject (e.g. `api.loans.list`).
2. Server has a **queue subscription** (or single subscription) on that subject.
3. Server handles the message and publishes the **reply** to the message’s `reply` inbox (NATS core request/reply).
4. Client receives exactly **one reply** per request (or times out).

### Contract

- **One request → one reply.** No streaming reply on these API subjects; for streams use event subjects or snapshot.
- **Reply inbox:** Set by the client when sending; server uses `message.respond(payload).await` (or equivalent). No custom reply subject in payload.
- **Timeout:** 5 seconds default. Overridable per subject if needed (match `nats_adapter::rpc` if/when used).
- **Payload:** Prefer **protobuf** for new endpoints (e.g. `request_proto` / `serve_proto`). Current implemented handlers use **JSON** (see api_handlers.rs). For JSON-only areas, use a dedicated helper; document request/response shape in this doc or in API_DOCUMENTATION_INDEX. For proto opportunities and wiring existing areas to proto, see [PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md](PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md).

### Errors

- Server returns HTTP-style error shape in JSON body (e.g. `{ "error": "..." }`) or appropriate proto field. Client should treat non-2xx or missing reply as failure.
- Timeout: client reports timeout; no retry specified here (client policy).

---

## 3. Scope of NATS-only API

**In scope (exposed via NATS)**

- **Snapshot:** `snapshot.{backend_id}` — primary way for TUI (and other clients) to get full system state. No REST equivalent.
- **Discount Bank:** `api.discount_bank.balance`, `.transactions`, `.bank_accounts`, `.import_positions` — implemented in backend_service.
- **Loans:** Full CRUD via NATS: `api.loans.list`, `api.loans.get`, `api.loans.create`, `api.loans.update`, `api.loans.delete` — implemented in backend_service; aggregation in `api` crate. Former REST loans CRUD is fully re-exposed via these subjects (task T-1773515423634118000).
- **Config (replaced):** Former REST config get/update are not re-exposed via NATS. Backend config is file/env only (`BACKEND_CONFIG`, `config/default.toml`, env vars); no `api.config.get` or `api.config.update` (task T-1773515423634118000).
- **FMP (optional):** `api.fmp.income_statement`, `.balance_sheet`, `.cash_flow`, `.quote` — implemented when `FMP_API_KEY` is set; consider “in scope for deployments that enable FMP.”

**Strategy control and orders/positions (NATS-only; no HTTP)**

There is no HTTP API for strategy control or for reading orders/positions. Both are **NATS-only** in the following sense:

- **Strategy start/stop/cancel-all:** Exposed via NATS request/reply: **`api.strategy.start`**, **`api.strategy.stop`**, and **`api.strategy.cancel_all`**. Backend_service subscribes and calls `StrategyController` for start/stop; for cancel_all it reads snapshot open-order count and replies with `{"ok": true, "message": "..."}` (broker cancel not wired yet). Reply JSON: `{"ok": true}` or `{"ok": true, "message": "..."}` on success, `{"ok": false, "error": "..."}` on failure. To obtain strategy status, subscribe to **`snapshot.{backend_id}`** and read the `SystemSnapshot.strategy` field. The RPC subject `rpc.strategy.status` is defined in `topics.rs` but not wired; optional request/reply may be added later.
- **Orders and positions list/details:** Not exposed via NATS request/reply. To list orders or positions (or get their details), subscribe to **`snapshot.{backend_id}`** and use `SystemSnapshot.orders` and `SystemSnapshot.positions` from the periodic snapshot. The TUI uses this as its single source for dashboard state. Optional `api.orders.list` / `api.positions.list` request/reply is deferred.

**In scope — Finance rates (implemented)**

- **Finance rates:** `api.finance_rates.extract`, `.build_curve`, `.compare`, `.yield_curve`, `.benchmarks`, `.sofr`, `.treasury` — implemented in backend_service. Live benchmark data (SOFR, Treasury) requires **`FRED_API_KEY`** (St. Louis Fed). Extract and build_curve work without it; compare, yield_curve, sofr, treasury return live data when key is set.

**Out of scope (deferred or removed)**

- *(Finance rates were deferred; scope approved and implemented — see above.)*
- **Calculate:** `api.calculate.*` (greeks, iv, hv, risk, strategy, box_spread, etc.) — library-only; no NATS surface. See **Calculate API: NATS-only** below.
- **Strategy control (request/reply):** `api.strategy.start` and `api.strategy.stop` — implemented (task T-1773515657237625000). `api.strategy.status` not implemented; see "Strategy control and orders/positions" above for NATS-only read path via snapshot.
- **Config:** `api.config.get` / `api.config.update` — **not exposed via NATS**. Backend config is file/env only (e.g. `BACKEND_CONFIG`, `config/default.toml`); no NATS config API.
- **Orders/positions (request/reply):** `api.orders.list` / `api.positions.list` — deferred; see "Strategy control and orders/positions" above for NATS-only read path via snapshot.

**Removed/deferred (no NATS exposure) — FMP chart, Swiftness proxy, frontend**

- **FMP fundamentals:** Re-exposed via NATS when `FMP_API_KEY` is set (`api.fmp.income_statement`, `.balance_sheet`, `.cash_flow`, `.quote`). See "In scope" above.
- **Chart (OHLCV / historical):** No NATS subject. Former REST chart endpoint removed; deferred. If needed later, add e.g. `api.chart.ohlcv` or use market-data events.
- **Swiftness proxy:** No NATS request/reply. Backend may merge Swiftness positions into the shared snapshot when `ENABLE_SWIFTNESS=1` (internal background task only). No client-facing Swiftness API over NATS.
- **Frontend read models:** Cash-flow timeline, opportunity simulation, and other former `/api/v1/frontend/*` routes are not exposed via NATS. TUI and other clients get state via `snapshot.{backend_id}` only. Re-add NATS subjects for these only if a client requires them.

**Calculate API: NATS-only (no calculate endpoints)**

The backend is NATS-only; there are **no** NATS request/reply subjects for calculate operations. Former REST calculate endpoints (greeks, implied volatility, historical vol, risk metrics, strategy payoffs, box spread, etc.) are **not** re-exposed over NATS by design (scope approved in T-1773515642013883000).

- **Where the logic lives:**  
  - **Rust:** `agents/backend/crates/api/src/quant.rs` — `api::quant::calculate_greeks`, `calculate_iv`, `calculate_historical_volatility`, `calculate_risk_metrics`, `calculate_strategy`, `calculate_box_spread`, `calculate_jelly_roll`, `calculate_ratio_spread` (request/response types + dispatch to `quant` crate).  
  - **Quant crate:** `agents/backend/crates/quant/src/lib.rs` — `QuantCalculator` (BSM option price, greeks, IV, HV, VaR/CVaR, straddle/strangle/butterfly/iron condor/box spread/jelly roll/ratio spread).  
  - **Risk crate:** `agents/backend/crates/risk` — `calculate_box_spread_risk`, `calculate_position_risk`, `calculate_portfolio_risk` (uses quant for greeks).  
- **How to get calculate-style results:**  
  - **From Rust:** Call `api::quant::calculate_*` (or `quant::QuantCalculator` / `risk` crate) in-process; no NATS round-trip.  
  - **Live risk/strategy state:** Consumed via `snapshot.{backend_id}` (risk status, strategy decisions, positions); no ad-hoc calculate RPC.  
  - **Native CLI:** C++ engine provides box spread table and greeks via CLI; no backend calculate API.

**Reference**

- Scope was approved in task T-1773515642013883000: Expose Discount Bank, Loans, Finance rates, strategy start/stop (+ optional FMP when key set). Library-only / not exposed: Calculate, Config. Removed/deferred: FMP chart, Swiftness proxy, frontend read models.
- Implementation: `agents/backend/services/backend_service/src/api_handlers.rs`; subject constants and handlers.
- Task T-1773515431075544000: FMP fundamentals re-exposed via NATS when key set; chart, Swiftness proxy, frontend documented as removed/deferred.
- Task T-1773515427981197000: Strategy control and orders/positions documented as NATS-only (read path via `snapshot.{backend_id}`; no request/reply).
- Task T-1773515652084062000: Finance rates NATS handlers (api.finance_rates.extract, .build_curve, .compare, .yield_curve, .benchmarks, .sofr, .treasury); FRED_API_KEY for live benchmarks.

---

## 4. Client lifecycle and configuration

- **async_nats Client:** No explicit `close()`; connection closes when the client (or wrapper) is dropped. For graceful shutdown, drop the client so the connection tears down.
- **Env / config:** `NATS_URL` (default `nats://localhost:4222`), `BACKEND_ID` (default `ib`), `NATS_SUBJECTS` (collection aggregation), `NATS_KV_BUCKET`, `NATS_USE_JETSTREAM` (optional: when `1`/`true`/`yes`, snapshot is published to JetStream stream `SNAPSHOTS` for replay; backend creates stream if missing). **`NATS_API_QUEUE_GROUP`** (default `api`): queue group name for all api.* request/reply subscriptions; when multiple backend instances run, they share load (only one instance receives each request). **`NATS_TICK_BATCH_MS`** (default `0`): when > 0, collection aggregation buffers `market-data.tick.>` messages and flushes every this many ms (last value per symbol); reduces write rate to LIVE_STATE KV and QuestDB under high tick volume. **`SNAPSHOT_PUBLISH_INTERVAL_MS`** (default `1000`): snapshot publish interval. **`FRED_API_KEY`** (optional): St. Louis Fed API key for live finance rates (SOFR, Treasury) used by `api.finance_rates.compare`, `.yield_curve`, `.benchmarks`, `.sofr`, `.treasury`. See backend_service and tui_service config.

### NATS_SUBJECTS subject list per environment

Used by **backend_service** collection aggregation only. Comma-separated list of NATS subjects (wildcards allowed). Backend subscribes to each; received messages are decoded as `NatsEnvelope`, merged into the shared snapshot, and optionally written to LIVE_STATE KV and QuestDB. Source: `backend_service/src/collection_aggregation.rs` (`CollectionConfig::from_env`).

| Environment | `NATS_SUBJECTS` value (default if unset) | Purpose |
|-------------|------------------------------------------|---------|
| **Default (dev)** | `market-data.tick.>`, `strategy.signal.>`, `strategy.decision.>` | Collect ticks, signals, and decisions; feed snapshot and optional KV/QuestDB. |
| **Production** | Same, or override via env to add/remove subjects (e.g. `market-data.>`, `strategy.>`) | Narrow or broaden collection; keep wildcards to match publishers. |
| **Custom** | Set `NATS_SUBJECTS` to a comma-separated list; spaces after commas are trimmed. | Add domains (e.g. `orders.>`, `risk.>`) when new publishers are deployed. |

If `NATS_SUBJECTS` is unset, the default above is used. Empty or invalid subjects are filtered out. To disable collection aggregation, do not set `NATS_URL` (or collection is skipped when URL is empty).

### Edge cases and validation (subject lists)

- **Empty or missing:** If `NATS_SUBJECTS` is unset, the default `market-data.tick.>,strategy.signal.>,strategy.decision.>` is used. If set to an empty string or only commas/whitespace, the list after split/trim is empty; backend subscribes to no collection subjects (snapshot and api.* still work).
- **Comma and spaces:** Values are split on `,`; leading/trailing spaces per token are trimmed. Example: ` a , b , c ` → three subjects `a`, `b`, `c`.
- **Invalid subjects:** The backend does not run `nats_adapter::topics::validate_topic` on `NATS_SUBJECTS` entries. NATS server will reject invalid subjects (e.g. containing spaces or double dots) when subscribing; invalid entries can result in subscription errors in logs. Prefer lowercase, dot-separated tokens; wildcards `*` and `>` are allowed.
- **Length:** Each subject must be ≤ 256 characters (NATS limit; see `nats_adapter::topics::validate_topic`). Longer values can cause subscription failure.
- **Wildcards:** Only subscribers should use wildcards (`*`, `>`). Publishers use concrete subjects. For collection aggregation, use wildcards to match publishers (e.g. `market-data.tick.>`).

### Operator guidelines for NATS_SUBJECTS

- **Setting:** Set `NATS_SUBJECTS` in the environment (e.g. in systemd unit, `.env`, or shell) before starting `backend_service`. Comma-separated list; spaces after commas allowed.
- **Testing changes:** (1) Set `NATS_SUBJECTS` to the new value. (2) Start backend with `NATS_URL` set. (3) Check logs for "CollectionConfig" or subscription errors. (4) Optional: use `nats sub 'market-data.tick.>'` (or your subjects) to confirm messages when publishers are running.
- **Rollback:** Restart backend with previous `NATS_SUBJECTS` (or unset to use default). No persistent state is keyed by the subject list; only in-flight collection is affected.
- **Per-environment:** Use the same default in dev; in production override only if you add/remove domains (e.g. add `orders.>`, `risk.>` when new publishers deploy). Document overrides in runbooks or env templates.

### Proto for new api.* endpoints

Current api.* handlers use **JSON** request/reply (see `api_handlers.rs` and `nats_adapter::request_json`). For **new** api.* endpoints, you may use **protobuf** request/reply instead:

- **Wire format:** Define request/response messages in `proto/messages.proto` and generate Rust (prost) via the existing build.
- **Server:** Use `nats_adapter::rpc::subscribe_proto` (or equivalent) to subscribe to a subject and reply with encoded proto; see `nats_adapter/src/rpc.rs`.
- **Client:** Use `nats_adapter::rpc::request_proto` (or equivalent) to send a request and decode the reply.
- **Compatibility:** Existing api.* subjects remain JSON; new subjects (e.g. `api.v2.orders.submit`) can be proto-only. Document each subject’s payload format in this doc or in the proto file.

See `docs/platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md` for proto usage and buf config.

### Integration test for api.* request/reply

Optional E2E test: `backend_service` test `test_api_request_reply_benchmarks` (in `services/backend_service/tests/integration_test.rs`). It is **ignored** by default. With NATS and one backend_service running, run:

```bash
cargo test -p backend_service --test integration_test test_api_request_reply_benchmarks -- --ignored
```

See `docs/runbooks/NATS_VERIFICATION_CHECKLIST.md` §5.

---

## References

- `agents/backend/crates/nats_adapter/src/topics.rs` — topic registry and validation
- `agents/backend/services/backend_service/src/api_handlers.rs` — NATS API handlers
- `docs/platform/DATAFLOW_ARCHITECTURE.md` — data flow and NATS KV
- `docs/platform/NATS_KV_USAGE_AND_RECOMMENDATIONS.md` — KV and snapshot read path
- `docs/platform/NATS_CONSUMER_AUDIT.md` — which consumers use protobuf vs JSON
- `docs/platform/LIVE_STATE_KV_VERIFICATION.md` — how to verify LIVE_STATE KV with NATS CLI
- `docs/platform/MOCK_DATA_SOURCES.md` — mock data for all sources (no API keys)
- `docs/platform/CURRENT_TOPOLOGY.md` — runtime shape (NATS-only backend)
- `docs/platform/TUI_CLI_FEATURE_PARITY.md` — TUI/CLI data source (NATS only) and parity gaps
- `docs/platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md` — proto for new endpoints and wiring JSON areas to proto
- `docs/platform/STUB_CODE_PLANNING.md` — stub/audit; NATS API surface when wiring real TWS or new contracts
