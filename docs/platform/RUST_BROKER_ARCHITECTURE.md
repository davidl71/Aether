# Rust Broker Adapter Architecture

**Date**: 2026-03-21
**Status**: Active
**Supersedes**: `docs/archive/architecture/MULTI_BROKER_ARCHITECTURE_DESIGN.md` and `docs/archive/architecture/BROKER_ADAPTER_DESIGN.md` (C++ designs from 2025)

---

## Overview

The Rust broker layer implements the `BrokerEngine` async trait, providing a broker-agnostic interface for IBKR operations. Two adapters exist: `IbAdapter` (ibapi TWS socket) and `YatWSEngine` (yatws TWS socket). A third path — `client_portal_options` — uses the IB Client Portal REST API but does not implement the trait.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Application Layer (backend_service, tui_service)      │
│         Uses BrokerEngine trait + channels                             │
└───────────────────────────────┬───────────────────────────────────────┘
                                │
┌──────────────────────────────▼───────────────────────────────────────┐
│              BrokerEngine trait (async)                                  │
│  Traits: Connect, Market Data, Options Chain, Orders, Positions         │
│  Crate: broker_engine/src/traits.rs                                    │
└───────────────────────────────┬───────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────▼────────┐   ┌────────▼───────┐   ┌─────────▼────────┐
│   IbAdapter     │   │  YatWSEngine   │   │ client_portal_   │
│  (ibapi TWS)   │   │ (yatws TWS)    │   │ options (REST)   │
│                 │   │                │   │                   │
│  ib_adapter/    │   │ yatws_adapter/ │   │ api/              │
│  impl BrokerEng │   │ impl BrokerEng  │   │ NOT a BrokerEng  │
└─────────────────┘   └────────────────┘   └───────────────────┘

Event channels (mpsc) flow back from all adapters into:
  MarketDataEvent → MarketDataIngestor → SystemSnapshot
  PositionEvent    → tws_positions / ib_positions → SystemSnapshot.positions
  OrderStatusEvent→ backend_service → NATS
```

---

## BrokerEngine Trait

File: `broker_engine/src/traits.rs`

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
    async fn cancel_all_orders(&self) -> Result<(), BrokerError>;  // Added 2026-03-21

    // Positions & account
    async fn request_positions(&self) -> Result<Vec<PositionEvent>, BrokerError>;
    async fn request_account(&self) -> Result<AccountInfo, BrokerError>;

    // Event channels
    fn market_data_tx(&self) -> mpsc::Sender<MarketDataEvent>;
    fn position_tx(&self) -> mpsc::Sender<PositionEvent>;
    fn order_tx(&self) -> mpsc::Sender<OrderStatusEvent>;
}
```

---

## Adapter Implementations

### IbAdapter — `crates/ib_adapter/src/lib.rs`

Uses `ibapi` crate (v2.10) for TWS/Gateway socket communication.

| Method | → ibapi call | Notes |
|--------|-------------|-------|
| `connect()` | `Client::connect()` | Auto-retry in `backend_service` |
| `cancel_order(id)` | `client.cancel_order(id, "")` | |
| `cancel_all_orders()` | `client.global_cancel()` | **NEW** — added 2026-03-21 |
| `request_positions()` | `client.positions()` | Stream of `PositionUpdate::Position` |
| `request_account()` | `client.managed_accounts()` + `account_summary()` | Two-step |
| `place_order(...)` | `client.place_order()` | |
| `request_option_chain()` | `option_chain()` (ibapi high-level) | Single call |

### YatWSEngine — `crates/yatws_adapter/src/lib.rs`

Uses `yatws` crate (v0.1.7) for TWS socket communication via `IBKRClient`.

| Method | → yatws/IBKRClient call | Notes |
|--------|--------------------------|-------|
| `cancel_order(id)` | `client.orders().cancel_order(&id_str)` | Takes `&str` not `i32` |
| `cancel_all_orders()` | `client.orders().cancel_all()` | ⚠️ Method name unverified |
| `request_positions()` | `client.account().list_open_positions()` | |
| `request_account()` | `client.account().get_account_info()` | |
| `request_option_chain()` | `get_option_chain_params()` | Direct `reqSecDefOptParams` |

### client_portal_options — `crates/api/src/client_portal_options.rs`

**Does NOT implement `BrokerEngine`** — separate REST-based flow.

| Step | Function | Endpoint |
|------|----------|----------|
| 1 | `search()` | `GET /iserver/secdef/search?symbol=` |
| 2 | `strikes()` | `GET /iserver/secdef/strikes?conid=&secType=OPT&month=` |
| 3 | `info()` | `GET /iserver/secdef/info?conid=&month=&strike=&right=` |

---

## Option Chain Resolution — Three Approaches

| Adapter | Transport | Return Type | Resolution Flow |
|---------|----------|-------------|----------------|
| `IbAdapter` | TWS socket | `Vec<OptionContract>` | Single ibapi call |
| `YatWSEngine` | TWS socket | `Vec<OptionContract>` | Single `reqSecDefOptParams` call |
| `client_portal_options` | REST | `SearchResult + StrikesResult + ContractInfo` | 3-step REST flow |

**Problem**: No unified `OptionChain` type. `OptionContract` (broker_engine) is the thinnest common denominator but is missing `con_id` (always `None`), `exchange`, and `multiplier`.

---

## Transferable Patterns from C++ Design

### 1. BrokerCapabilities (NOT YET IMPLEMENTED)

The C++ design defined a `BrokerCapabilities` struct per adapter. This is **missing from the Rust implementation** and should be added:

```rust
pub struct BrokerCapabilities {
    pub supports_options: bool,          // both adapters: yes
    pub supports_multi_leg_orders: bool, // both: yes (box spreads)
    pub supports_real_time_data: bool,   // both: yes (TWS socket)
    pub supports_historical_data: bool,  // both: via reqHistoricalData
    pub supports_market_data_snapshot: bool, // client_portal: yes, others: via snapshot
    pub max_orders_per_second: u32,
    pub rate_limit_per_minute: u32,
    pub requires_contract_resolution: bool, // IbAdapter: needs reqContractDetails
}

impl BrokerCapabilities {
    pub const IBKR_TWS: BrokerCapabilities = BrokerCapabilities {
        supports_options: true,
        supports_multi_leg_orders: true,
        supports_real_time_data: true,
        supports_historical_data: true,
        supports_market_data_snapshot: false,
        max_orders_per_second: 50,
        rate_limit_per_minute: 120,
        requires_contract_resolution: true,
    };

    pub const CLIENT_PORTAL: BrokerCapabilities = BrokerCapabilities {
        supports_options: true,
        supports_multi_leg_orders: false, // REST only, no combo orders
        supports_real_time_data: false,   // Snapshot only, no streaming
        supports_historical_data: true,
        supports_market_data_snapshot: true,
        max_orders_per_second: 10,
        rate_limit_per_minute: 60,
        requires_contract_resolution: false, // conId in search results
    };
}
```

Add `fn capabilities(&self) -> BrokerCapabilities` to `BrokerEngine`.

### 2. Structured BrokerError Taxonomy (NOT YET IMPLEMENTED)

The C++ design defined a structured error enum. The Rust `BrokerError` is currently flat:

```rust
// Current (broker_engine/src/error.rs)
pub enum BrokerError {
    NotConnected,
    ContractError(String),
    OrderFailed(String),
    Other(String),
}
```

Should be extended with structured variants (while maintaining `Other` for unknown errors):

```rust
pub enum BrokerError {
    NotConnected,
    ContractError(String),
    OrderFailed(String),
    RateLimited { retries: u32, backoff_secs: u64 },  // NEW
    AuthenticationFailed(String),                       // NEW
    InsufficientFunds { available: f64, required: f64 }, // NEW
    MarketDataUnavailable(String),                   // NEW
    Unknown(String),                                  // replaces Other
}
```

### 3. BrokerWithFallback (NOT YET IMPLEMENTED)

The C++ design had a `BrokerWithFallback` wrapper. The current `cancel_all_reply` says "broker not wired" — the fallback pattern is needed:

```rust
pub struct BrokerWithFallback {
    primary: Arc<dyn BrokerEngine>,
    fallback: Arc<dyn BrokerEngine>,
}

impl BrokerWithFallback {
    pub async fn request_positions(&self) -> Result<Vec<PositionEvent>, BrokerError> {
        match self.primary.request_positions().await {
            Ok(positions) => Ok(positions),
            Err(BrokerError::NotConnected) | Err(BrokerError::Other(_)) => {
                tracing::warn!("Primary broker unavailable, trying fallback");
                self.fallback.request_positions().await
            }
            Err(e) => Err(e),
        }
    }
}
```

**Current gap**: `backend_service` directly calls NATS handlers and `spawn_tws_position_fetcher` — it does not use `BrokerEngine` for position fetching. `cancel_all_reply` logs "broker not wired" because there's no path from NATS handler to the broker adapter.

### 4. Data Normalization Per Adapter (Partial — Missing Conversions)

The C++ `DataNormalizer` pattern — each adapter normalizes to a unified type. The Rust `From` conversions are the idiomatic equivalent:

| Conversion | Status | File |
|-----------|--------|------|
| `From<&PositionSnapshot> for RuntimePositionDto` | ✅ Exists | `runtime_state.rs:780` |
| `From<IbPositionDto> for PositionSnapshot` | ❌ Missing | T-1774099649464475000 |
| `From<IbPositionDto> for RuntimePositionDto` | ❌ Missing | T-1774099722292613000 |
| `From<AccountInfo> for Metrics` | ❌ Missing | T-1774099656848744000 |
| `From<PositionEvent> for PositionSnapshot` | ❌ Missing — `account` dropped | `tws_positions.rs` sets phantom fields |

### 5. Phased Migration (Partially Done)

The C++ migration path was: **Interface → TWS Adapter → Refactor Core → Add New Adapters → Broker Manager**.

The Rust equivalent:
- ✅ Phase 1-2: `BrokerEngine` trait + `IbAdapter` + `YatWSEngine` exist
- ⚠️ Phase 3: `backend_service` still bypasses trait for positions (uses `ibapi::Client` directly in `tws_positions.rs`)
- ❌ Phase 4: `BrokerWithFallback` not implemented
- ❌ Phase 5: `BrokerCapabilities` not added to trait

---

## Channel-Based Event Flow

Unlike the C++ `std::function<void(...)>` callback model, Rust uses async channels:

```
BrokerAdapter                     backend_service
     │                                  │
     ├─ market_data_tx ──────────────► │ MarketDataIngestor
     │    Sender<MarketDataEvent>      │   → SymbolSnapshot
     │                                  │   → SystemSnapshot.symbols
     ├─ position_tx ─────────────────►  │ tws_positions.rs / ib_positions.rs
     │    Sender<PositionEvent>         │   → PositionSnapshot
     │                                  │   → SystemSnapshot.positions
     └─ order_tx ────────────────────►  │ OrderStatusEvent
          Sender<OrderStatusEvent>       │   → NATS
```

Each adapter owns its senders. The receiver side (`backend_service`) manages the subscription.

---

## Configuration

Broker config is in `shared_config.yaml`:

```yaml
broker:
  ib_tws:
    host: "127.0.0.1"
    port: 7497          # Paper (live: 7496)
    client_id: 0
    paper_trading: true

# IB Client Portal (alternative to TWS)
ib_portal_url: "https://localhost:5001"  # Empty = use TWS socket
```

---

## Known Gaps

| Gap | Severity | Task |
|-----|----------|------|
| `backend_service` bypasses `BrokerEngine` for positions | High | T-1774099624024314000 (add account_id) |
| `From<IbPositionDto> for PositionSnapshot` missing | High | T-1774099649464475000 |
| `cancel_all_orders` not wired to NATS handler | High | T-1774099603122156000 |
| `BrokerCapabilities` not in trait | Medium | New task needed |
| `BrokerError` taxonomy is flat | Medium | New task needed |
| `BrokerWithFallback` not implemented | Medium | New task needed |
| `OptionChainProvider` trait for unified option chain | Medium | T-1774099701033026000 |
| `account_id` missing from `broker_engine::domain::Position` | High | T-1774099624024314000 |
| `tws_positions.rs` sets non-existent fields on `PositionSnapshot` | High | Bug — needs fix |

---

## Testing Strategy

| Type | What | Status |
|------|------|--------|
| Unit | Each adapter independently | ✅ `ib_adapter` and `yatws_adapter` have internal tests |
| Unit | `combo_strategy` inference | ✅ 11 tests in `api/src/combo_strategy.rs` |
| Integration | Backend + TWS (paper) | ⚠️ Manual — no automated integration test |
| Integration | Backend + Client Portal | ⚠️ Manual |
| Fallback | Primary → fallback on disconnect | ❌ Not implemented |

---

## References

- `broker_engine/src/traits.rs` — trait definition
- `broker_engine/src/domain.rs` — domain types
- `crates/ib_adapter/src/lib.rs` — IbAdapter implementation
- `crates/yatws_adapter/src/lib.rs` — YatWSEngine implementation
- `crates/api/src/client_portal_options.rs` — REST option chain
- `docs/BACKEND_TYPE_COMPARISON.md` — cross-type reference
- `docs/archive/architecture/MULTI_BROKER_ARCHITECTURE_DESIGN.md` — C++ design (superseded)
