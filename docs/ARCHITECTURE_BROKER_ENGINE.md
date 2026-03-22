# broker_engine Architecture

**Date**: 2026-03-21
**Status**: ✅ Steps 1–4, 6 complete. See §6 for current state.
**Owner**: backend / ib_adapter team
**Related tasks**: T-1773949780944708000, T-1773949802510897000, T-1773949816501720000, T-1773949859122015000, T-1773949896022157000, T-1773949903377577000

---

## 1. Overview

`broker_engine` is a new Rust crate that abstracts all broker operations behind a trait, enabling the backend to switch between different broker implementations (IBKR, yatws, mock, paper-trading) without code changes.

```
┌─────────────────────────────────────────────────────┐
│              backend_service                         │
│  (no direct ibapi/yatws imports — uses BrokerEngine)│
└──────────────────────┬────────────────────────────┘
                       │ uses
                       ▼
              ┌─────────────────────┐
              │   BrokerEngine      │  ← trait (broker_engine)
              │   trait + domain    │
              └──────────┬──────────┘
          implements     │  implements
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
   ┌──────────┐   ┌───────────┐   ┌──────────┐
   │ IbAdapter │   │YatWSEngine│   │MockEngine │
   │(ib_adapter)│  │(yatws_    │   │(tests)    │
   │ ibapi dep │   │ adapter)  │   │ no deps   │
   └──────────┘   └───────────┘   └──────────┘
```

---

## 2. Motivation

**Problem**: `backend_service` historically imported `ibapi` directly in `tws_market_data.rs` and `tws_positions.rs`. Any alternative broker (yatws, mock, paper-trading engine) required copy-paste refactoring of business logic.

**Current state** (post-migration):
- `IbAdapter` in `ib_adapter` crate implements `BrokerEngine` backed by `ibapi::Client`
- `YatWSEngine` in `yatws_adapter` crate implements `BrokerEngine` backed by `yatws`
- `backend_service` uses the `BrokerEngine` trait — no direct `ibapi` imports
- `tws_market_data.rs` and `tws_positions.rs` were removed from `backend_service` (orphaned files, never compiled)

**Goal**: ✅ **Achieved** — Backend depends only on `BrokerEngine` trait + domain types. Zero `ibapi` references outside `ib_adapter`.

---

## 3. Design Goals

1. **Zero dependencies in `broker_engine`** — pure domain types + trait; no external runtime
2. **`async_trait`** for the `BrokerEngine` trait (matches existing workspace pattern)
3. **Domain types are broker-agnostic** — `OptionContract`, `OrderAction`, `TimeInForce`, `Order`, `Position`, `AccountInfo`, `MarketData`, `BagOrderLeg`, `PlaceBagOrderRequest`
4. **Event types in domain** — `MarketDataEvent`, `PositionEvent`, `OrderStatusEvent` (not in `ib_adapter`)
5. **Backwards compatible** — existing `IbAdapter` stays; `IbAdapter` implements `BrokerEngine` (previously a wrapper without the trait)
6. **Minimal surface area** — trait has exactly the operations backend actually calls

---

## 4. Crate Structure

```
crates/broker_engine/
├── Cargo.toml            # name = "broker_engine", deps: async-trait, serde, tokio (macros only)
├── src/
│   ├── lib.rs            # re-exports
│   ├── error.rs          # BrokerError enum
│   ├── domain.rs         # all domain types (OptionContract, OrderAction, events, config, box spread helper)
│   └── traits.rs         # BrokerEngine trait definition
```

**`domain.rs`** contains everything that is not a trait:

| Type | Description |
|------|-------------|
| `OptionContract` | symbol, expiry, strike, is_call, con_id |
| `OrderAction` | Buy / Sell |
| `TimeInForce` | Day / GTC / IOC / FOK |
| `OrderStatus` | Submitted / Filled / PartiallyFilled / Cancelled / Rejected / Pending |
| `Order` | order_id, contract, action, quantity, limit_price, tif, status |
| `Position` | contract, quantity, avg_price, market_value, unrealized_pnl |
| `AccountInfo` | account_id, net_liquidation, cash_balance, buying_power, maintenance_margin, initial_margin |
| `MarketData` | bid, ask, last, volume, timestamp |
| `BagOrderLeg` | contract, ratio, action |
| `PlaceBagOrderRequest` | underlying_symbol, currency, exchange, legs, quantity, limit_price, tif, order_action |
| `construct_box_spread_order()` | constructs a 4-leg BAG request (moved from `ib_adapter::types`) |
| `MarketDataEvent` | contract_id, symbol, bid, ask, last, volume |
| `PositionEvent` | account, symbol, position, avg_cost |
| `OrderStatusEvent` | order_id, status, filled, remaining, avg_fill_price |
| `ConnectionState` | Disconnected / Connecting / Connected / Error(String) |
| `IbConfig` | host, port, client_id, paper_trading (moved from `ib_adapter`) |

**`traits.rs`** defines the `BrokerEngine` trait:

```rust
#[async_trait]
pub trait BrokerEngine: Send + Sync {
    // Lifecycle
    async fn connect(&self) -> Result<(), BrokerError>;
    async fn disconnect(&self) -> Result<(), BrokerError>;
    async fn state(&self) -> ConnectionState;

    // Market data
    async fn request_market_data(&self, symbol: &str, contract_id: i64) -> Result<(), BrokerError>;
    async fn request_option_chain(&self, symbol: &str) -> Result<Vec<OptionContract>, BrokerError>;

    // Orders
    async fn place_order(&self, contract: OptionContract, action: OrderAction, quantity: i32, limit_price: f64) -> Result<i32, BrokerError>;
    async fn place_bag_order(&self, request: PlaceBagOrderRequest) -> Result<i32, BrokerError>;
    async fn cancel_order(&self, order_id: i32) -> Result<(), BrokerError>;

    // Positions & account
    async fn request_positions(&self) -> Result<Vec<PositionEvent>, BrokerError>;
    async fn request_account(&self) -> Result<AccountInfo, BrokerError>;

    // Event channels (cloneable senders for consumers)
    fn market_data_tx(&self) -> mpsc::Sender<MarketDataEvent>;
    fn position_tx(&self) -> mpsc::Sender<PositionEvent>;
    fn order_tx(&self) -> mpsc::Sender<OrderStatusEvent>;
}
```

**`error.rs`**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    #[error("not connected")]
    NotConnected,
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("order failed: {0}")]
    OrderFailed(String),
    #[error("contract error: {0}")]
    ContractError(String),
    #[error("timeout")]
    Timeout,
    #[error("other: {0}")]
    Other(String),
}
```

---

## 5. File Layout After Migration

```
agents/backend/
├── Cargo.toml                   # + "crates/broker_engine"
└── crates/
    ├── broker_engine/           # NEW: traits + domain (0 external deps beyond workspace)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs
    │       ├── domain.rs
    │       └── traits.rs
    │
    ├── ib_adapter/              # EXISTING: IBKR implementation
    │   └── src/
    │       ├── lib.rs           # re-exports broker_engine::*; implements BrokerEngine for IbAdapter
    │       ├── types.rs         # DELEGATE to broker_engine (keep compat shim or delete)
    │       ├── scanner.rs        # KEEP (TWS scanner, not in trait)
    │       └── tws_wire.rs       # KEEP (wire format docs)
    │
    ├── tws_yield_curve/          # EXISTING: yield curve (will migrate to BrokerEngine)
    │   └── src/
    │       └── ...
    │
    └── ...

services/
    └── backend_service/
        └── src/
            ├── main.rs           # creates IbAdapter (or YatWSEngine) via config
            ├── api_handlers.rs   # uses BrokerEngine trait via StrategyController
            └── backend_service.rs # uses BrokerEngine trait for account/positions
```

---

## 6. Implementation Sequence

| Step | Task | ID | Status |
|------|------|----|--------|
| 1 | Create `broker_engine` crate with `Cargo.toml`, `lib.rs`, `error.rs`, `domain.rs`, `traits.rs` | T-1773949780944708000 | ✅ Done |
| 2 | Implement `BrokerEngine` for `IbAdapter` in `ib_adapter` | T-1773949802510897000 | ✅ Done |
| 3 | Migrate `backend_service` to use `BrokerEngine` trait | T-1773949816501720000 | ✅ Done |
| 4 | Create `yatws_adapter` crate implementing `BrokerEngine` | T-1773949896022157000 | ✅ Done |
| 5 | Migrate `tws_yield_curve` to `BrokerEngine` | T-1773949859122015000 | ⏳ Deferred (independent service) |
| 6 | Add `OptionChainProvider` trait | T-1774099701033026000 | ✅ Done (2026-03-21) |

---

## 7. Key Design Decisions

### Why zero dependencies in `broker_engine`?
Domain types (`OptionContract`, `OrderAction`, etc.) are plain data structures. They should not pull in `ibapi`, `tokio`, or any runtime. This ensures any crate can use them without inheriting broker-specific concerns.

### Why event types in domain, not in `ib_adapter`?
`MarketDataEvent`, `PositionEvent`, and `OrderStatusEvent` are the **outputs** of broker operations. They belong in the domain so that any `BrokerEngine` implementation (IBKR, yatws, mock) emits the same event types. Moving them out of `ib_adapter` also removes the mpsc dependency from domain.

### Why keep `IbAdapter` until migration is complete?
`backend_service` does not currently use `IbAdapter` — it uses `ibapi` directly. The migration path is:
1. Create `broker_engine` + `IbAdapter` (implements `BrokerEngine`, backed by `ibapi`)
2. Migrate `backend_service` from direct `ibapi` → `IbAdapter: BrokerEngine`
3. Delete `IbAdapter` struct (not the `ib_adapter` crate)

### What about `tws_yield_curve`?
It fetches SOFR/treasury rates via a **REST call** to IB Gateway's `/fsg/…` endpoint, not the TWS socket API. It should also implement `BrokerEngine` for completeness, but its connection model differs (HTTP REST vs socket). The trait may need a `request_yield_curve(&self, …) → Result<YieldCurveData, BrokerError>` method, or it can remain as a separate component that wraps `BrokerEngine`.

### Why `Arc<dyn BrokerEngine>` over generics?

Services construct a single engine instance at startup and pass it to long-lived tasks. The vtable overhead at that call frequency is negligible. Generics would propagate type parameters through every spawned task and every accepting function — `Arc<dyn BrokerEngine>` keeps signatures clean and enables runtime engine selection.

### Flat vs segregated trait surface

The current design uses a single `BrokerEngine` trait. An alternative is segregated traits (`MarketDataEngine`, `PositionEngine`, `OrderEngine`, `OptionChainEngine`, `AccountEngine`) so crates take only the slice they need. This is deferred — the flat trait is simpler to implement first and can be split once usage patterns stabilise.

---

## 8. yatws Implementation Notes (T-1773949896022157000)

[yatws](https://github.com/drpngx/yatws) is a production-tested Rust TWS API (cloned to `/Users/davidl/Projects/Trading/yatws`). Key differences from `ibapi` that affect the adapter design:

### sync/async bridge

`yatws` uses `parking_lot` mutexes and blocking Condvar waits — it is fundamentally synchronous. `YatWSEngine` bridges this via `tokio::task::spawn_blocking`:

```rust
async fn request_positions(&self) -> Result<Vec<PositionEvent>, BrokerError> {
    let mgr = self.client.account().clone();
    tokio::task::spawn_blocking(move || mgr.list_open_positions())
        .await
        .map_err(|e| BrokerError::Other(e.to_string()))?
        .map(|ps| ps.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}
```

Every `impl BrokerEngine for YatWSEngine` method that calls a yatws manager uses this pattern. The bridge is isolated to `yatws_adapter`; the `BrokerEngine` trait stays async throughout.

### Box spread via OptionsStrategyBuilder

`yatws` has a native `OptionsStrategyBuilder` that replaces the manual `construct_box_spread_order()` helper:

```rust
let (combo_contract, order_request) = OptionsStrategyBuilder::new(
    self.client.data_ref().clone(),
    "SPX", underlying_price, quantity, SecType::Index,
)?
.box_spread_nearest_expiry(expiry, lower_strike, upper_strike)?
.with_limit_price(limit)
.with_highest_liquidity()  // auto-selects exchange by liquidity score
.build()?;

let order_id = self.client.orders().place_order(combo_contract, order_request)?;
```

Features over `IbAdapter`:
- conId resolution handled automatically via `DataRefManager`
- Exchange selection by liquidity score (or primary, most-complete, SMART)
- All 4 legs constructed internally; no manual `BagOrderLeg` assembly

### Additional capabilities exposed by yatws

These can be surfaced via `BrokerEngine` extensions or as separate utilities in `yatws_adapter`:

| Capability | yatws API | Relevance |
|-----------|-----------|-----------|
| Rate limiting | `RateLimiterConfig` (50 msg/s default, configurable) | Prevents IBKR API violations |
| Session replay | `IBKRClient::from_db(db, session)` | Testing without live TWS |
| Algo orders | `IBKRAlgo::{Adaptive, VWAP, TWAP, ArrivalPrice, AccumulateDistribute, ...}` (12 types) | Execution slicing |
| Financial Advisor | `FinancialAdvisorManager` — groups, profiles, allocations | Multi-account (21+ accounts) order routing |
| WSH corporate events | `DataFundamentalsManager::get_wsh_events()` — 24+ event types | Risk monitoring (earnings, dividends, M&A, splits) |
| Fundamentals | `FundamentalReportType::{ReportsFinSummary, ReportSnapshot, ReportsFinStatements, RESC}` | Company data |
| Scanner | `DataMarketManager::get_scanner_results()` | Replaces stub `ScannerSubscription` in `ib_adapter` |
| Contract types | 21+ SecType variants incl. Bill, Bond, Forex, Crypto, CFD, SLB | Full platform asset class coverage |

### Runtime engine selection

Engine is selected at startup via config — no recompile required:

```rust
// backend_service/src/main.rs
fn create_engine(config: &BrokerConfig) -> Arc<dyn BrokerEngine> {
    match config.engine.as_deref().unwrap_or("ibapi") {
        "yatws" => Arc::new(YatWSEngine::new(config)),
        _       => Arc::new(IbAdapter::new(config)),
    }
}
```

Config field: `broker.engine = "ibapi" | "yatws"` (default: `ibapi`).

Cargo features can gate which adapter is compiled into the binary:

```toml
# backend_service/Cargo.toml
[features]
default = ["engine-ibapi"]
engine-ibapi = ["ib_adapter"]
engine-yatws = ["yatws_adapter"]
```

### MockBrokerEngine

A `MockBrokerEngine` in `broker_engine` behind `#[cfg(test)]` or a `testing` feature enables unit tests for `tws_market_data.rs` and `tws_positions.rs` without a live TWS connection — a significant gap in the current test suite.

---

## 9. Backwards Compatibility & Migration Notes

- `ib_adapter::IbAdapter` ✅ implements `BrokerEngine` trait (step 2 complete)
- `ib_adapter::types::*` delegates to `broker_engine::domain::*` to avoid duplication
- `backend_service` ✅ uses `BrokerEngine` trait — `tws_market_data.rs` and `tws_positions.rs` were removed (orphaned files, never compiled)
- `yatws_adapter::YatWSEngine` ✅ implements `BrokerEngine` (step 4 complete)
- No proto/gRPC changes required — this is a Rust-only trait refactor

---

## 10. References

- Task: T-1773949780944708000 — Create broker_engine crate ✅
- Task: T-1773949802510897000 — Implement BrokerEngine for IbAdapter ✅
- Task: T-1773949816501720000 — Migrate backend_service to broker_engine traits ✅
- Task: T-1773949896022157000 — Create yatws_adapter crate ✅
- Task: T-1774099701033026000 — Add OptionChainProvider trait ✅
- TWS protobuf wire format: `agents/backend/crates/ib_adapter/src/tws_wire.rs`
- yatws source: `/Users/davidl/Projects/Trading/yatws` (cloned from drpngx/yatws)
- yatws learnings: `docs/research/learnings/YATWS_LEARNINGS.md`
