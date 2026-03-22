# Broker Engine Trait vs Direct SDK Integration

**Source:** longbridge-terminal vs Aether comparison  
**Date:** 2026-03-22  
**Status:** Todo

## Architecture Comparison

### Aether: Trait-Based Abstraction

```rust
// broker_engine/src/lib.rs
pub trait BrokerEngine {
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn get_positions(&self) -> Result<Vec<Position>>;
    async fn place_order(&self, order: Order) -> Result<OrderId>;
    // ...
}

// Adapters implement the trait
impl BrokerEngine for IbAdapter { /* ... */ }
impl BrokerEngine for YatwsAdapter { /* ... */ }
```

**Benefits:**
- Multiple broker support (IBKR, yatws)
- Testability via mock implementations
- Swappable at runtime

### longbridge-terminal: Direct SDK Integration

```rust
// Direct use of longbridge SDK
let ctx = longbridge::quote::QuoteContext::try_new(config).await?;
ctx.subscribe(&symbols, SubFlags::QUOTE).await?;
```

**Benefits:**
- Simpler code
- Full SDK feature access
- No abstraction overhead

## Tradeoffs

| Factor | Aether (Trait) | longbridge (Direct) |
|--------|----------------|---------------------|
| Flexibility | Multi-broker | Single broker |
| Complexity | Higher | Lower |
| Testing | Mock-friendly | Harder to mock |
| SDK features | Limited to trait | Full access |
| Evolution | Interface stability | Fast SDK updates |

## Recommendation for Aether

The `broker_engine` trait is appropriate given:
- Multi-broker roadmap (IBKR + others)
- Need for IBKR adapter testing
- Long-term maintainability

Consider adding a **direct mode** flag for when abstraction overhead isn't needed.

## Related Tasks

- T-1774192025497339000: Compare broker_engine trait vs direct SDK integration
