# Adding a Broker Adapter

How to implement the `BrokerEngine` trait for a new broker.

## Overview

The `broker_engine` crate defines a broker-agnostic `BrokerEngine` trait. Each adapter (e.g., `IbAdapter`, `YatWSEngine`) implements this trait, allowing the backend to work with any broker through a unified interface.

**Key files:**
- Trait definition: `agents/backend/crates/broker_engine/src/traits.rs`
- Domain types: `agents/backend/crates/broker_engine/src/domain.rs`
- Example adapter: `agents/backend/crates/ib_adapter/src/lib.rs`

---

## 1. Implement the Trait

```rust
use broker_engine::{BrokerEngine, domain::*};
use tokio::sync::mpsc;

pub struct MyBrokerAdapter {
    // Your broker connection
    client: BrokerClient,
    
    // Channels for streaming data
    market_data_tx: mpsc::Sender<MarketDataEvent>,
    position_tx: mpsc::Sender<PositionEvent>,
    order_tx: mpsc::Sender<OrderStatusEvent>,
}

#[async_trait]
impl BrokerEngine for MyBrokerAdapter {
    fn connect(&self) -> impl Future<Output = Result<(), BrokerError>> + Send { ... }
    fn disconnect(&self) -> impl Future<Output = Result<(), BrokerError>> + Send { ... }
    fn state(&self) -> BrokerState { ... }
    
    fn request_market_data(&self, contract: &OptionContract) -> ... { ... }
    fn request_option_chain(&self, symbol: &str) -> ... { ... }
    fn place_order(&self, order: Order) -> ... { ... }
    fn place_bag_order(&self, legs: &[ComboLeg], price: f64) -> ... { ... }
    fn cancel_order(&self, order_id: &str) -> ... { ... }
    fn cancel_all_orders(&self) -> ... { ... }
    fn request_positions(&self) -> ... { ... }
    fn request_account(&self) -> ... { ... }
    
    fn market_data_tx(&self) -> mpsc::Sender<MarketDataEvent> { ... }
    fn position_tx(&self) -> mpsc::Sender<PositionEvent> { ... }
    fn order_tx(&self) -> mpsc::Sender<OrderStatusEvent> { ... }
}
```

---

## 2. Domain Types to Use

| Type | Purpose |
|------|---------|
| `OptionContract` | Symbol, expiry, strike, call/put, con_id |
| `Order` | order_id, contract, action, quantity, limit_price, tif |
| `OrderStatus` | Submitted, Filled, PartiallyFilled, Cancelled, Rejected, Pending |
| `Position` | contract, quantity, avg_price, market_value, unrealized_pnl |
| `AccountInfo` | account_id, net_liquidation, cash_balance, buying_power, margin |

---

## 3. Key Implementation Patterns

### Capability Flags

```rust
fn supports_box_spreads(&self) -> bool { true }
fn supports_options(&self) -> bool { true }
fn supports_combo_orders(&self) -> bool { true }
```

### Box Order (ComboLeg)

```rust
fn place_bag_order(&self, legs: &[ComboLeg], price: f64) -> ... {
    // Box spread = 4 legs:
    // Buy call lower strike, sell call upper strike
    // Buy put upper strike, sell put lower strike
    let order = OptionsStrategyBuilder::box_spread(legs, price);
    self.place_order(order)
}
```

### Async + Sync Fallback

```rust
fn request_positions_sync(&self, timeout_ms: u64) -> Vec<Position> {
    let (tx, mut rx) = mpsc::channel(1);
    self.request_positions(tx);
    match tokio::time::timeout(Duration::from_millis(timeout_ms), rx.recv()).await {
        Ok(Some(positions)) => positions,
        _ => self.cached_positions.clone(), // fallback
    }
}
```

### Rate Limiting

Avoid TWS API throttling:

```rust
use tokio::sync::Semaphore;
let rate_limit = Semaphore::new(50); // max 50 requests/sec

async fn request_with_limit(&self, req: Request) -> Result<Response> {
    let _permit = self.rate_limit.acquire().await?;
    self.client.send(req).await
}
```

---

## 4. Add to Backend

In `backend_service/src/lib.rs`:

```rust
// Add adapter to Cargo.toml
// [dependencies]
// my_broker_adapter = { path = "../crates/my_broker_adapter" }

// In backend builder:
let broker: Arc<dyn BrokerEngine> = match config.broker.as_str() {
    "ibkr" => Arc::new(IbAdapter::new(config.ibkr.clone())),
    "mybroker" => Arc::new(MyBrokerAdapter::new(config.mybroker.clone())),
    _ => return Err(anyhow!("Unknown broker: {}", config.broker)),
};
```

---

## 5. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_place_box_order() {
        let adapter = MyBrokerAdapter::new().await.unwrap();
        let legs = vec![
            ComboLeg { action: OrderAction::Buy, ... },
            ComboLeg { action: OrderAction::Sell, ... },
            // ...
        ];
        let result = adapter.place_bag_order(&legs, 1.00).await;
        assert!(result.is_ok());
    }
}
```

---

## 6. Reference Adapters

| Adapter | Key Features |
|---------|--------------|
| `ib_adapter` | TWS socket via ibapi, Box spreads, rate limiting |
| `yatws_adapter` | TWS socket via yatws crate, similar to ib_adapter |

See `docs/BACKEND_TYPE_COMPARISON.md` for detailed method coverage comparison.
