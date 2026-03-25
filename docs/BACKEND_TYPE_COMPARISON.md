# Backend Type Comparative Reference

Generated from codebase analysis. Last updated: 2026-03-24.

## 1. Position Types

| Type | Crate | Location | Key Fields | Used By |
|------|-------|----------|-----------|---------|
| `broker_engine::domain::Position` | `broker_engine` | `domain.rs` | `contract`, `quantity`, `avg_price`, `market_value`, `unrealized_pnl` | `IbAdapter`, `broker_engine` |
| `broker_engine::domain::PositionEvent` | `broker_engine` | `domain.rs` | `account`, `symbol`, `position`, `avg_cost` | Broker → backend via channel |
| `common::PositionSnapshot` | `common` | `snapshot.rs` | `id`, `symbol`, `quantity`, `cost_basis`, `mark`, `unrealized_pnl`, `account_id?`, `source?` | NATS, snapshots, proto |
| `api::IbPositionDto` | `api` | `ib_positions.rs` | portal position DTO | Client Portal REST |
| `api::RuntimePositionDto` | `api` | `runtime_state.rs` | TUI/API-facing projected position DTO | TUI, API |

## 2. Order Types

| Type | Crate | Location | Key Fields | Used By |
|------|-------|----------|-----------|---------|
| `broker_engine::domain::Order` | `broker_engine` | `domain.rs` | `order_id`, `contract`, `action`, `quantity`, `limit_price`, `tif`, `status` | Internal |
| `broker_engine::domain::OrderStatusEvent` | `broker_engine` | `domain.rs` | `order_id`, `status`, `filled`, `remaining`, `avg_fill_price` | Broker → backend via channel |
| `common::OrderSnapshot` | `common` | `snapshot.rs` | snapshot order fields | NATS, `SystemSnapshot` |
| `api::RuntimeOrderDto` | `api` | `runtime_state.rs` | TUI/API-facing order DTO | TUI, API |

## 3. Market Data Types

| Type | Crate | Location | Key Fields | Used By |
|------|-------|----------|-----------|---------|
| `broker_engine::domain::MarketData` | `broker_engine` | `domain.rs` | `bid`, `ask`, `last`, `volume`, `timestamp` | `IbAdapter` internal |
| `broker_engine::domain::MarketDataEvent` | `broker_engine` | `domain.rs` | broker event channel payload | `BrokerEngine` channels |
| `market_data::MarketDataEvent` | `market_data` | `model.rs` | normalized provider event | Aggregator, providers |
| `common::CandleSnapshot` | `common` | `snapshot.rs` | OHLCV snapshot | `SymbolSnapshot` |
| `api::SymbolSnapshot` | `api` | `state.rs` | symbol read model + candle | `SystemSnapshot.symbols` |

Note: `broker_engine::domain::MarketDataEvent` and `market_data::MarketDataEvent`
have similar names but different roles. The former is the broker boundary event.
The latter is the provider/aggregator event shape.

## 4. Account / Metrics Types

| Type | Crate | Location | Key Fields | Notes |
|------|-------|----------|-----------|-------|
| `broker_engine::domain::AccountInfo` | `broker_engine` | `domain.rs` | broker-neutral account fields | Raw broker data |
| `common::Metrics` | `common` | `snapshot.rs` | snapshot metric fields | API snapshot view |

## 5. BrokerEngine Trait Coverage

| Method | Trait | `IbAdapter` | Notes |
|--------|-------|------------|-------|
| `connect()` | ✅ | ✅ | TWS socket via ibapi |
| `disconnect()` | ✅ | ✅ | |
| `state()` | ✅ | ✅ | |
| `request_market_data()` | ✅ | ✅ | |
| `request_option_chain()` | ✅ | ✅ | |
| `place_order()` | ✅ | ✅ | |
| `place_bag_order()` | ✅ | ✅ | |
| `cancel_order()` | ✅ | ✅ | |
| `cancel_all_orders()` | ✅ | ✅ | |
| `request_positions()` | ✅ | ✅ | |
| `request_account()` | ✅ | ✅ | |
| `market_data_tx()` | ✅ | ✅ | Channel sender |
| `position_tx()` | ✅ | ✅ | Channel sender |
| `order_tx()` | ✅ | ✅ | Channel sender |

## 6. Option Chain Resolution

| Adapter | Transport | Function | Returns | Resolution Method |
|---------|----------|----------|---------|-----------------|
| `ib_adapter` | TWS socket (ibapi) | `request_option_chain(symbol)` | `Vec<OptionContract>` | High-level `option_chain()` |
| `client_portal_options` | Client Portal REST | `search` → `strikes` → `info` | REST DTOs | 3-step REST flow |

## 7. Adapter → IBKR API Mapping

### IbAdapter

| `IbAdapter` method | → ibapi method | Notes |
|--------------------|----------------|-------|
| `cancel_order(id)` | `client.cancel_order(id, "")` | |
| `cancel_all_orders()` | `client.global_cancel()` | |
| `request_positions()` | `client.positions()` | Returns position stream |
| `request_account()` | `client.managed_accounts()` + `client.account_summary()` | |
| `place_order(...)` | `client.place_order(order_id, contract, order)` | |
| `place_bag_order(...)` | BAG contract + `place_order()` | |

## 8. Pipeline Note

Active broker-backed market data is expected to flow:

```text
IbAdapter -> broker_engine::MarketDataEvent -> backend bridge ->
market_data aggregator -> NATS / snapshots / TUI
```

That end-to-end bridge is still incomplete in the backend service.
