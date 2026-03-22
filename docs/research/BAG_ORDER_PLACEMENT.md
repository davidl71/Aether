# IBKR BAG Order Placement Research

## Status: Stub vs Implemented

`IbAdapter::place_bag_order()` at `ib_adapter/src/lib.rs:354` is a **stub** returning `Ok(0)`.
It is NOT wired to any real IBKR API call.

## What Exists

### `IbAdapter::place_bag_order()` (stub)
- Location: `ib_adapter/src/lib.rs:354`
- Current: Returns `Ok(0)` without side effects
- Signature: `async fn place_bag_order(&self, req: PlaceBagOrderRequest) -> Result<i32, BrokerError>`

### `construct_box_spread_order()` (complete)
- Location: `broker_engine/src/domain.rs:230`
- Builds a `PlaceBagOrderRequest` with 4 legs (call_low, put_low, call_high, put_high)
- Each leg has `OptionContract` with symbol, expiry, strike, right (call/put), ratio=1
- **Missing**: conId resolution

### `PlaceBagOrderRequest`
- Location: `broker_engine/src/domain.rs:201`
- Fields: underlying_symbol, currency, exchange, legs, quantity, limit_price, tif, order_action
- `legs: Vec<BagOrderLeg>` where each `BagOrderLeg` has `contract: OptionContract` and `action: OrderAction`

### `OptionContract`
- Location: `broker_engine/src/domain.rs:145`
- Fields: symbol, expiry, strike, right (bool: true=call, false=put), con_id (Option<i32>)
- con_id is `None` when constructed via `construct_box_spread_order`

## What Needs to Be Done

### 1. conId Resolution (T-1774192087389849000)
Before placing a BAG order, each leg's `OptionContract.con_id` must be resolved via IBKR's `reqContractDetails`.

IBKR API workflow:
1. Create `Contract` object with symbol, secType="OPT", exchange, currency
2. Call `reqContractDetails(contract, callback)` 
3. Receive `ContractDetails` with `contract.conId`
4. Store conId in `OptionContract.con_id`

In `IbAdapter`, this means:
- Add `async fn resolve_contract_details(&self, contract: &OptionContract) -> Result<i32, BrokerError>`
- Uses existing `self.client` to send `reqContractDetails` via `IbClient`
- Returns the resolved conId

### 2. Wire conId Resolution into place_bag_order
In `IbAdapter::place_bag_order()`:
1. For each leg in `req.legs`, resolve conId via `resolve_contract_details()`
2. Build the BAG contract using resolved conIds (or symbol-based combo)
3. Place the order via `placeOrder()`

### 3. placeOrder Call
The actual order placement uses IBKR's `placeOrder()`:
```rust
self.client.place_order(order_id, contract, order)
```

For BAG orders, the `Contract` must be a combo contract:
```rust
Contract {
    sec_type: "BAG",
    symbol: underlying_symbol,
    exchange: req.exchange,
    currency: req.currency,
    combolegs: Some(vec![...]),  // one per leg
    ..Default::default()
}
```

## BAG Order Construction in IBKR

IBKR combo/BAG orders require:
1. `Contract` with `secType = "BAG"` 
2. `combolegs: Vec<ComboLeg>` — each leg specifies conId, ratio, action (BUY/SELL), exchange
3. `Order` with `orderType = "LMT"`, `lmtPrice = limit_price`, `tif = "FOK"`

The `IbClient` in `ibapi` crate should have:
- `place_order(order_id: i32, contract: Contract, order: Order)`
- `req_contract_details(contract: Contract, ...)`

## Files to Modify

| File | Change |
|------|--------|
| `ib_adapter/src/lib.rs` | Implement `resolve_contract_details()` and wire into `place_bag_order()` |
| `broker_engine/src/domain.rs` | `OptionContract` already has `con_id: Option<i32>` — may need helper to set it |

## Reference: yatws Adapter

`yatws_adapter` has `box_spread_nearest_expiry()` using native box spread support:
- Location: `yatws_adapter/src/lib.rs`
- Uses `OptionsStrategyBuilder::box_spread_nearest_expiry()` for native box spread

This is NOT the same as BAG orders — yatws uses a different API. Our IBKR adapter uses BAG/combo.

## Task Links

- T-1774191986657723000: Implement IbAdapter::place_bag_order() - replace stub with real IBKR API call
- T-1774192087389849000: Implement IBKR conId resolution workflow for combo/BAG orders
