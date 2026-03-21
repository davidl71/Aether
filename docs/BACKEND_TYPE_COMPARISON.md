# Backend Type Comparative Reference

Generated from codebase analysis. Last updated: 2026-03-21.

---

## 1. Position Types

| Type | Crate | Location | Key Fields | Used By |
|------|-------|----------|-----------|---------|
| `broker_engine::domain::Position` | `broker_engine` | `domain.rs:105` | `contract: OptionContract`, `quantity`, `avg_price`, `market_value`, `unrealized_pnl` | `IbAdapter`, `YatWSEngine`, `broker_engine` |
| `broker_engine::domain::PositionEvent` | `broker_engine` | `domain.rs:305` | `account`, `symbol`, `position: i32`, `avg_cost: f64` | Broker → backend via channel |
| `common::PositionSnapshot` | `common` | `snapshot.rs:14` | `id`, `symbol`, `quantity`, `cost_basis`, `mark`, `unrealized_pnl`, `account_id?`, `source?` | NATS, `SystemSnapshot`, proto |
| `api::IbPositionDto` | `api` | `ib_positions.rs:26` | `account_id`, `symbol`, `quantity`, `cost`, `mark`, `avg_cost`, `realized_pnl`, `unrealized_pnl`, `account_id` | Client Portal REST |
| `api::RuntimePositionDto` | `api` | `runtime_state.rs:948` | All `PositionSnapshot` fields + `position_type?`, `strategy?`, `apr_pct?`, `market_value`, `age_days` | TUI, API |
| `api::RuntimePositionState` | `api` | `runtime_state.rs:788` | Intermediate: `symbol`, `quantity`, `cost_basis`, `mark`, `position_type?`, `strategy?`, `apr_pct?` | `runtime_state.rs` projection |

**Conversions that exist:**
- `From<&PositionSnapshot> for RuntimePositionState` → `runtime_state.rs:25`
- `From<RuntimePositionState> for PositionSnapshot` → `runtime_state.rs:39`
- `From<&PositionSnapshot> for RuntimePositionDto` → `runtime_state.rs:780`
- `PositionEvent` → `PositionSnapshot` → `RuntimePositionDto` (multi-step via `broker_engine::domain`)

**Conversions MISSING (TODOs):**
- ❌ `From<IbPositionDto> for PositionSnapshot` — blocks unified position handling
- ❌ `From<IbPositionDto> for RuntimePositionDto` — blocks Client Portal positions in TUI combo view
- ❌ `From<PositionEvent> for PositionSnapshot` — `account` field dropped when going through `tws_positions.rs`

---

## 2. Order Types

| Type | Crate | Location | Key Fields | Used By |
|------|-------|----------|-----------|---------|
| `broker_engine::domain::Order` | `broker_engine` | `domain.rs:89` | `order_id`, `contract: OptionContract`, `action: OrderAction`, `quantity`, `limit_price`, `tif: TimeInForce`, `status: OrderStatus` | Internal |
| `broker_engine::domain::OrderStatus` | `broker_engine` | `domain.rs:77` | Enum: `Submitted`, `Filled`, `PartiallyFilled`, `Cancelled`, `Rejected`, `Pending` | Internal |
| `broker_engine::domain::OrderStatusEvent` | `broker_engine` | `domain.rs:313` | `order_id`, `status: String`, `filled`, `remaining`, `avg_fill_price` | Broker → backend via channel |
| `common::OrderSnapshot` | `common` | `snapshot.rs:37` | `id`, `symbol`, `side`, `quantity`, `status`, `submitted_at: DateTime` | NATS, `SystemSnapshot` |
| `api::RuntimeOrderDto` | `api` | `runtime_state.rs:948` | `id`, `symbol`, `side`, `quantity`, `status`, `submitted_at`, `filled?`, `avg_fill_price?` | TUI, API |

---

## 3. Market Data Types

| Type | Crate | Location | Key Fields | Used By |
|------|-------|----------|-----------|---------|
| `broker_engine::domain::MarketData` | `broker_engine` | `domain.rs:130` | `bid`, `ask`, `last`, `volume`, `timestamp: i64` | `IbAdapter` internal |
| `broker_engine::domain::MarketDataEvent` | `broker_engine` | `domain.rs:293` | `contract_id: i64`, `symbol`, `bid`, `ask`, `last`, `volume`, `quote_quality: QuoteQuality` | `BrokerEngine` trait channels |
| `market_data::MarketDataEvent` | `market_data` | `model.rs:6` | `contract_id: i64`, `symbol`, `bid`, `ask`, `last`, `volume`, `timestamp: DateTime`, `quote_quality: u32` | `MarketDataPipeline`, `FmpClient`, `PolygonMarketDataSource` |
| `common::CandleSnapshot` | `common` | `snapshot.rs:51` | `open`, `high`, `low`, `close`, `volume`, `entry`, `updated: DateTime` | `SymbolSnapshot` |
| `api::SymbolSnapshot` | `api` | `state.rs:94` | `symbol`, `last`, `bid`, `ask`, `spread`, `roi`, `maker_count`, `taker_count`, `volume`, `candle: CandleSnapshot` | `SystemSnapshot.symbols` |

**Note:** `broker_engine::domain::MarketDataEvent` and `market_data::MarketDataEvent` are **both** named `MarketDataEvent` but have different field types (`QuoteQuality` vs `u32`, `i64` vs `DateTime` timestamp). The `BrokerEngine::market_data_tx()` channel carries `broker_engine::MarketDataEvent`.

---

## 4. Account / Metrics Types

| Type | Crate | Location | Key Fields | Notes |
|------|-------|----------|-----------|-------|
| `broker_engine::domain::AccountInfo` | `broker_engine` | `domain.rs:115` | `account_id`, `net_liquidation`, `cash_balance`, `buying_power`, `maintenance_margin`, `initial_margin` | Broker-agnostic raw data |
| `common::Metrics` | `common` | `snapshot.rs:87` | `net_liq`, `buying_power`, `excess_liquidity`, `margin_requirement`, `commissions`, `portal_ok`, `tws_ok`, `tws_address?`, `questdb_ok`, `nats_ok` | API snapshot view |

**Conversion MISSING:**
- ❌ `From<AccountInfo> for Metrics` — manual field mapping done in `backend_service` instead

---

## 5. BrokerEngine Trait — Method Coverage

| Method | Trait | `IbAdapter` | `YatWSEngine` | Notes |
|--------|-------|------------|--------------|-------|
| `connect()` | ✅ | ✅ `connect()` | ✅ `connect()` | TWS socket via ibapi |
| `disconnect()` | ✅ | ✅ `disconnect()` | ✅ `disconnect()` | |
| `state()` | ✅ | ✅ `state()` | ✅ `state()` | |
| `request_market_data()` | ✅ | ✅ | ✅ | Both via TWS socket |
| `request_option_chain()` | ✅ | ✅ `option_chain()` | ✅ `get_option_chain_params()` | Different APIs |
| `place_order()` | ✅ | ✅ | ✅ | |
| `place_bag_order()` | ✅ | ✅ | ✅ | |
| `cancel_order()` | ✅ | ✅ `cancel_order(order_id, "")` | ✅ `cancel_order(order_id)` | |
| `cancel_all_orders()` | ✅ | ✅ `global_cancel()` | ⚠️ `cancel_all()` (method name uncertain) | New in trait |
| `request_positions()` | ✅ | ✅ `positions()` | ✅ `list_open_positions()` | Different APIs |
| `request_account()` | ✅ | ✅ `managed_accounts()` + `account_summary()` | ✅ `get_account_info()` | |
| `market_data_tx()` | ✅ | ✅ | ✅ | Channel sender |
| `position_tx()` | ✅ | ✅ | ✅ | Channel sender |
| `order_tx()` | ✅ | ✅ | ✅ | Channel sender |

**Legend:** ✅ = implemented, ❌ = not implemented, ⚠️ = uncertain (method name not confirmed in yatws source)

---

## 6. Option Chain Resolution — Three Approaches

| Adapter | Transport | Function | Returns | Resolution Method |
|---------|----------|----------|---------|-----------------|
| `ib_adapter` | TWS socket (ibapi) | `request_option_chain(symbol)` | `Vec<OptionContract>` | High-level `option_chain()` — single call |
| `yatws_adapter` | TWS socket (yatws) | `request_option_chain(symbol)` | `Vec<OptionContract>` | Direct `get_option_chain_params()` — single call |
| `client_portal_options` | Client Portal REST | `search` → `strikes` → `info` | `SearchResult`, `StrikesResult`, `ContractInfo` | 3-step REST flow |

**All return different types** — no unified `OptionChain` struct. `OptionContract` (broker_engine) is the closest but only has `symbol`, `expiry`, `strike`, `is_call`, `con_id`.

---

## 7. Adapter → IBKR API Mapping

### IbAdapter (`ib_adapter/src/lib.rs`)

| `IbAdapter` method | → ibapi method | Notes |
|--------------------|----------------|-------|
| `cancel_order(id)` | `client.cancel_order(id, "")` | |
| `cancel_all_orders()` | `client.global_cancel()` | NEW — added T-1774099603122156000 |
| `request_positions()` | `client.positions()` | Returns `PositionUpdate::Position` stream |
| `request_account()` | `client.managed_accounts()` + `client.account_summary()` | |
| `place_order(...)` | `client.place_order(order_id, contract, order)` | |
| `place_bag_order(...)` | `OptionsStrategyBuilder` + `place_order()` | |

### YatWSEngine (`yatws_adapter/src/lib.rs`)

| `YatWSEngine` method | → yatws/IBKRClient method | Notes |
|---------------------|---------------------------|-------|
| `cancel_order(id)` | `client.orders().cancel_order(&id_str)` | Takes `&str` not `i32` |
| `cancel_all_orders()` | `client.orders().cancel_all()` | ⚠️ Method name uncertain — blocked on yatws source |
| `request_positions()` | `client.account().list_open_positions()` | |
| `request_account()` | `client.account().get_account_info()` | |
| `request_option_chain()` | `client.get_option_chain_params()` | Direct `reqSecDefOptParams` |

---

## 8. NATS Topics & Proto Messages

### Proto Messages (`proto/messages.proto`)

| Message | Fields | Used For |
|---------|--------|---------|
| `MarketDataEvent` | contract_id, symbol, bid, ask, last, volume, timestamp, quote_quality | Real-time quotes |
| `CandleSnapshot` | open, high, low, close, volume, entry, updated | OHLCV bars |
| `SymbolSnapshot` | symbol, last, bid, ask, spread, roi, maker/taker_count, volume, candle | Aggregated quote |
| `Position` | id, symbol, quantity, cost_basis, mark, unrealized_pnl | **Missing**: account_id, source |
| `HistoricPosition` | id, symbol, quantity, realized_pnl, closed_at | Closed positions |
| `Order` | id, symbol, side, quantity, status, submitted_at | **Missing**: filled, avg_fill_price |
| `StrategyDecision` | symbol, quantity, side, mark, created_at | Decision log |
| `StrategySignal` | symbol, price, timestamp | Signal feed |
| `RiskStatus` | allowed, reason, updated_at | Risk state |
| `Metrics` | net_liq, buying_power, excess_liquidity, margin_requirement, commissions, portal_ok, tws_ok, questdb_ok, nats_ok | **Missing**: account_id |
| `Alert` | level, message, timestamp | System alerts |
| `SystemSnapshot` | generated_at, started_at, mode, strategy, account_id, metrics, symbols[], positions[], historic[], orders[], decisions[], alerts[], risk | Full snapshot |
| `BoxSpreadLeg` | long_call, short_call, long_put, short_put, net_debit, theoretical_value, ... | Box spread calc |
| `BoxSpreadScenario` | symbol, strike_width, theoretical_value, estimated_net_debit, implied_apr, scenario_type | Opportunity |
| `YieldCurve` | symbol, strike_width, benchmark_rate, points[] | Yield curve |

### NATS Topic Hierarchy

```
api.strategy.start/stop/cancel_all   — strategy control
api.admin.set_mode                   — snapshot mode switch
api.snapshot.publish_now             — force snapshot publish
api.calculator.*                     — finance calculations
system.health                        — backend health
market_data.*                        — market data events
strategy.*                           — strategy signals/decisions
```

---

## 9. Data Flow: Position Sources

```
┌─────────────────────────────────────────────────────────────────────┐
│                     IB Client Portal (REST)                         │
│  ib_positions.rs: fetch_ib_positions() → Vec<IbPositionDto>         │
│  Used when IB_PORTAL_URL env var is set                             │
└──────────────────────┬──────────────────────────────────────────────┘
                       │ (no conversion yet)
                       ▼
┌─────────────────────────────────────────────────────────────────────┐
│  IbPositionDto ──❌──→ PositionSnapshot ──❌──→ RuntimePositionDto  │
│  (api crate)        missing From impl        missing From impl        │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                     TWS Socket (ibapi / yatws)                     │
│  IbAdapter: client.positions() → PositionUpdate stream              │
│  YatWSEngine: client.account().list_open_positions()                │
└──────────────────────┬──────────────────────────────────────────────┘
                       │ broker_engine::domain::PositionEvent
                       ▼
              PositionEvent { account, symbol, position, avg_cost }
                       │
                       ▼
┌─────────────────────────────────────────────────────────────────────┐
│  tws_positions.rs ──→ PositionSnapshot { id, symbol, qty,           │
│                        cost_basis, mark, unrealized_pnl,             │
│                        account_id: Some(p.account),                 │
│                        source: Some("TWS") }                        │
│                        ⚠️ Also sets: position_type?, strategy?,      │
│                        combo_net_bid?, combo_quote_source? —        │
│                        FIELDS THAT DON'T EXIST in PositionSnapshot! │
└─────────────────────────────────────────────────────────────────────┘
                       │
                       ▼
              SystemSnapshot.positions: Vec<PositionSnapshot>
               (via spawn_tws_position_fetcher in backend_service)
```

---

## 10. Market Data Pipeline

```
┌──────────────────────────────────────────────────────┐
│  Sources (all implement MarketDataSource trait)        │
│  • FmpClient (HTTP REST — financial modeling prep)   │
│  • PolygonMarketDataSource (HTTP REST)               │
│  • IbAdapter via market_data_tx channel             │
│  • YatWSEngine via market_data_tx channel           │
└────────────────────┬─────────────────────────────────┘
                     │ MarketDataEvent (market_data crate)
                     ▼
┌──────────────────────────────────────────────────────┐
│  MarketDataPipeline / MarketDataIngestor             │
│  (market_data/src/pipeline.rs)                       │
└────────────────────┬─────────────────────────────────┘
                     │ MarketDataEvent (broker_engine crate)
                     ▼
┌──────────────────────────────────────────────────────┐
│  backend_service::tws_market_data.rs                │
│  spawn_tws_market_data() → run_tws_subscriptions()  │
│  Converts to SymbolSnapshot, updates SystemSnapshot  │
└──────────────────────────────────────────────────────┘
```

---

## 11. API Public Surface (`api/src/lib.rs`)

```rust
pub mod client_portal_options;  // secdef REST endpoints (search, strikes, info)
pub mod combo_strategy;         // infer_combo_strategy_type, apply_derived_strategy_types
pub mod discount_bank;          // discount bank parsing
pub mod finance_rates;          // finance rate calculations
pub mod ib_positions;           // fetch_ib_positions, fetch_ib_positions_all, IbPositionDto
pub mod loans;                  // LoanAggregationInput, LoanRecord, LoanRepository
pub mod mock_data;             // seed_snapshot
pub mod quant;                  // quantitative calculations
pub mod shared_config;          // load_shared_config, validate_shared_config
pub mod yield_curve_proto;     // yield curve proto conversions

pub use runtime_state::{
    ProducerMetadata, ProducerType,
    RuntimeDecisionDto, RuntimeExecutionState,
    RuntimeHistoricPositionDto, RuntimeMarketState,
    RuntimeOrderDto, RuntimePositionDto,
    RuntimeProducerDecision, RuntimeRiskState,
    RuntimeSnapshotDto, ScenarioDto,
};

pub use state::{
    Alert, AlertLevel,          // api-only Alert (different from common::Alert)
    SharedSnapshot, SystemSnapshot,
    SymbolSnapshot, CandleSnapshot, HistoricPosition, Metrics,
    OrderSnapshot, PositionSnapshot, RiskStatus, StrategyDecisionSnapshot,
    // CommonAlert re-export too
};
```

**Key:** `api::PositionSnapshot` and `common::PositionSnapshot` are the **same type** (re-export from common). `api::Alert` is **different** from `common::Alert` — both exist.

---

## 12. Shared Types Location Summary

```
common/src/snapshot.rs       ← SHOULD contain all cross-crate types
common/src/lib.rs             ← re-exports from snapshot module

api/src/state.rs             ← api-only: SystemSnapshot, SymbolSnapshot, Alert (api variant)
api/src/runtime_state.rs     ← RuntimePositionDto, RuntimeOrderDto, RuntimeMarketState
api/src/ib_positions.rs      ← IbPositionDto (Client Portal REST specific)

broker_engine/src/domain.rs  ← broker-agnostic: Position, OptionContract, Order, AccountInfo
broker_engine/src/traits.rs   ← BrokerEngine trait

market_data/src/model.rs      ← MarketDataEvent (market_data crate variant)
market_data/src/fmp.rs        ← FmpQuote, FmpClient
market_data/src/polygon.rs    ← PolygonMarketDataSource
```
