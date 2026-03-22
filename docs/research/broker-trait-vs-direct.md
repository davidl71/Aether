# Broker Engine Trait vs Direct SDK Integration

**Source:** Comparison of Aether vs longbridge-terminal  
**Date:** 2026-03-22  
**Status:** Completed

## Architecture Comparison

### Aether: Trait-Based Abstraction

```rust
// broker_engine/src/traits.rs
#[async_trait]
pub trait BrokerEngine {
    async fn connect(&mut self) -> Result<()>;
    async fn get_positions(&self) -> Result<Vec<Position>>;
    async fn place_order(&self, order: Order) -> Result<OrderId>;
    // ...
}

// Multiple implementations
impl BrokerEngine for IbAdapter { /* ... */ }
impl BrokerEngine for YatwsAdapter { /* ... */ }
```

### longbridge-terminal: Direct SDK Integration

```rust
// Direct SDK usage
let ctx = longbridge::quote::QuoteContext::try_new(config).await?;
ctx.subscribe(&symbols, SubFlags::QUOTE).await?;
ctx.candlesticks("AAPL.US", Period::Day, 100, None).await?;
```

## Tradeoff Analysis

| Factor | Aether (Trait) | longbridge (Direct) |
|--------|----------------|---------------------|
| **Flexibility** | Multi-broker support | Single broker |
| **Complexity** | Higher (trait bounds, async_trait) | Lower |
| **Testing** | Mock-friendly via trait | Harder to mock |
| **SDK Features** | Limited to trait methods | Full SDK access |
| **Evolution** | Interface stability required | Fast updates OK |
| **Runtime Switch** | Yes | No |

## Why Aether Uses Trait

1. **Multi-broker roadmap** - IBKR primary, yatws experimental
2. **Testing strategy** - MockBroker for unit tests
3. **Interface contracts** - Stable API for api crate
4. **Domain modeling** - Broker-agnostic Position/Order types

## Why longbridge Uses Direct SDK

1. **Single broker** - Longbridge SDK is the only broker
2. **SDK-first** - Tight coupling is acceptable
3. **Rapid iteration** - No trait versioning overhead
4. **Simpler deployment** - No abstraction layer

## Recommendation for Aether

**Keep current architecture** - The `BrokerEngine` trait is appropriate given:

- Multi-broker support (IBKR, yatws, potential others)
- Need for mock implementations in testing
- Stable API surface for api crate consumers
- Long-term maintainability

**Consider**: Add a "direct mode" that bypasses trait when single-broker deployment is sufficient.

## Related Tasks

- T-1774192025497339000: Compare broker_engine trait (Done)
