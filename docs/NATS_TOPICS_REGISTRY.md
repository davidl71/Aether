# NATS topics registry

**Audience:** Operators and integrators wiring NATS clients (TUI, CLI, tools) to `backend_service`.

**Canonical definitions:** Rust constants in `agents/backend/crates/nats_adapter/src/topics.rs` (`nats_adapter::topics`). If this document and the crate disagree, **trust the crate** and update this file.

---

## Snapshot stream (not `api.*`)

| Topic pattern | Purpose |
|---------------|---------|
| `snapshot.{backend_id}` | Periodic full `SystemSnapshot` protobuf from backend. |
| `snapshot.>` | Subscribe to all backend snapshots. |

Helper: `topics::snapshot::backend(id)`, `topics::snapshot::all()`.

---

## Backend RPC subjects (`api.*`)

Request/reply and queue-subscribe handlers in `backend_service`. Constants live under `topics::api`.

### `api.discount_bank.*`

| Subject | Constant |
|---------|----------|
| `api.discount_bank.balance` | `topics::api::discount_bank::BALANCE` |
| `api.discount_bank.transactions` | `topics::api::discount_bank::TRANSACTIONS` |
| `api.discount_bank.bank_accounts` | `topics::api::discount_bank::BANK_ACCOUNTS` |
| `api.discount_bank.import_positions` | `topics::api::discount_bank::IMPORT_POSITIONS` |

### `api.loans.*`

| Subject | Constant |
|---------|----------|
| `api.loans.list` | `topics::api::loans::LIST` |
| `api.loans.list.proto` | `topics::api::loans::LIST_PROTO` |
| `api.loans.get` | `topics::api::loans::GET` |
| `api.loans.create` | `topics::api::loans::CREATE` |
| `api.loans.update` | `topics::api::loans::UPDATE` |
| `api.loans.delete` | `topics::api::loans::DELETE` |
| `api.loans.import_bulk` | `topics::api::loans::IMPORT_BULK` |

Request body: `LoansBulkImportRequest` JSON `{ "loans": [ LoanRecord, ... ] }`. Response: `Result<LoansBulkImportResponse, String>` — `applied` row count plus per-index `errors` for validation/DB failures (valid rows are still upserted).

### `api.finance_rates.*` and `api.yield_curve.*`

| Subject | Constant |
|---------|----------|
| `api.finance_rates.extract` | `topics::api::finance_rates::EXTRACT` |
| `api.finance_rates.build_curve` | `topics::api::finance_rates::BUILD_CURVE` |
| `api.finance_rates.compare` | `topics::api::finance_rates::COMPARE` |
| `api.finance_rates.yield_curve` | `topics::api::finance_rates::YIELD_CURVE` |
| `api.finance_rates.benchmarks` | `topics::api::finance_rates::BENCHMARKS` |
| `api.finance_rates.sofr` | `topics::api::finance_rates::SOFR` |
| `api.finance_rates.treasury` | `topics::api::finance_rates::TREASURY` |
| `api.yield_curve.refresh` | `topics::api::yield_curve::REFRESH` |

### `api.fmp.*`

| Subject | Constant |
|---------|----------|
| `api.fmp.income_statement` | `topics::api::fmp::INCOME_STATEMENT` |
| `api.fmp.balance_sheet` | `topics::api::fmp::BALANCE_SHEET` |
| `api.fmp.cash_flow` | `topics::api::fmp::CASH_FLOW` |
| `api.fmp.quote` | `topics::api::fmp::QUOTE` |

### `api.calculate.*`

| Subject | Constant |
|---------|----------|
| `api.calculate.greeks` | `topics::api::calculate::GREEKS` |
| `api.calculate.iv` | `topics::api::calculate::IV` |
| `api.calculate.historical_volatility` | `topics::api::calculate::HISTORICAL_VOLATILITY` |
| `api.calculate.risk_metrics` | `topics::api::calculate::RISK_METRICS` |
| `api.calculate.strategy` | `topics::api::calculate::STRATEGY` |
| `api.calculate.box_spread` | `topics::api::calculate::BOX_SPREAD` |
| `api.calculate.jelly_roll` | `topics::api::calculate::JELLY_ROLL` |
| `api.calculate.ratio_spread` | `topics::api::calculate::RATIO_SPREAD` |

### `api.strategy.*` (read-only / deprecated control)

| Subject | Constant |
|---------|----------|
| `api.strategy.start` | `topics::api::strategy::START` |
| `api.strategy.stop` | `topics::api::strategy::STOP` |
| `api.strategy.cancel_all` | `topics::api::strategy::CANCEL_ALL` |
| `api.strategy.execute` | `topics::api::strategy::EXECUTE` |

### `api.admin.*`, `api.snapshot.*`, `api.ib.*`

| Subject | Constant | Notes |
|---------|----------|--------|
| `api.admin.set_mode` | `topics::api::admin::SET_MODE` | Deprecated in data-exploration mode. |
| `api.snapshot.publish_now` | `topics::api::snapshot::PUBLISH_NOW` | RPC to force snapshot publish; not the `snapshot.{id}` stream module. |
| `api.ib.positions` | `topics::api::ib::POSITIONS` | IB positions fetch. |

---

## System topics

| Topic / pattern | Helper |
|-----------------|--------|
| `system.health` | `topics::system::health()` |
| `system.events` | `topics::system::events()` |
| `system.alerts` | `topics::system::alerts()` |
| `system.commands.{action}` | `topics::system::commands(action)` |
| `system.commands.>` | `topics::system::all_commands()` |

Command lifecycle payloads use the protobuf envelope around `SystemCommandEvent` (see `proto/messages.proto` and `api::command_proto`).

---

## Platform / strategy / market-data (non-`api`)

See `topics::market_data`, `topics::strategy`, `topics::orders`, `topics::positions`, `topics::risk`, `topics::rpc`, `topics::dlq` in the same Rust module for streaming and RPC naming patterns.

---

## Related docs

- `ARCHITECTURE.md` — service boundaries.
- `docs/QUICKSTART_RUST.md` — running backend and TUI with NATS.
