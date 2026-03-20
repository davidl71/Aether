---
description: Use derive_builder for new event/data structs with 5+ fields; no manual Default when derive_builder suffices
alwaysApply: false
---

# DeriveBuilder Pattern

**Rule:** All new or refactored event, data, and snapshot structs with 5+ fields **must** use `#[derive(derive_builder::Builder)]` with field-level `#[builder(default)]` defaults, instead of manual `impl Default` or partial struct literals.

## When to Apply

| Scenario | Action |
|---|---|
| New `pub struct` with 5+ public fields | Add `#[derive(derive_builder::Builder)]` with `#[builder(default)]` per field |
| Existing struct with manual `impl Default` | Convert to derive_builder if fields are mostly defaulted |
| Partial struct literal (`Struct { a, b, .. }`) | Use `StructBuilder::default().field(value).build()` |
| Struct with `#[builder(setter(into))]` on `String` fields | Ensures `.field("string")` works (no `.field(String::from(...))`) |

## Pattern

```rust
// ✅ CORRECT — derive_builder with defaults
#[derive(Clone, Debug, Default, derive_builder::Builder)]
#[builder(setter(into), default)]
pub struct MarketDataEvent {
    #[builder(default = "0")]
    pub contract_id: i64,
    #[builder(setter(into))]
    pub symbol: String,
    #[builder(default = "0.0")]
    pub bid: f64,
    // ...
}

// ❌ WRONG — manual impl Default
impl Default for MarketDataEvent {
    fn default() -> Self {
        Self {
            contract_id: 0,
            symbol: String::new(),
            bid: 0.0,
            // ...
        }
    }
}

// ❌ WRONG — partial struct literal
let event = MarketDataEvent {
    symbol: "SPY".into(),
    bid: 500.0,
    ..Default::default()
};
```

## Conversion Steps

1. Add `derive_builder = { workspace = true }` to crate's `Cargo.toml` (if not already)
2. Add `#[derive(derive_builder::Builder, Default)]` (Default required for `#[builder(default)]`)
3. Annotate each field: `#[builder(default = "value")]` for primitives, `#[builder(default)]` for `String`/`Option`, `#[builder(setter(into))]` for `String` to allow `&str` inputs
4. Replace all construction sites with `StructBuilder::default().field(value).build()`

## Existing Approved Uses (Do Not Change)

- `MarketDataEvent` in `market_data/src/model.rs` — converted ✅
- `broker_engine::MarketDataEvent` — converted ✅
- `broker_engine::PositionEvent` / `OrderStatusEvent` — converted ✅
- Calculator/Config types (`BrokerConfig`, `Metrics`, etc.) — `Default` is acceptable as they are constructed via factories
- Proto-generated structs (`prost::Message`) — exempt

## Anti-Patterns to Flag

- Struct with 5+ fields but **no** `Default` or `Builder` derive → add builder
- `impl Default` that just zeros/blanks fields when builder would suffice → flag for refactor
- Multiple partial struct literals for same type in same crate → consolidate via builder
- Struct updated with new fields but existing construction sites not updated → must update all sites
