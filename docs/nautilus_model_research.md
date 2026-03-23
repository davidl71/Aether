# NautilusTrader Integration Research

## Context

Aether's TUI currently falls back to mock data when TWS is disconnected. The goal is to improve market data resilience by integrating additional data providers and potentially adopting battle-tested data types from NautilusTrader.

## Data Sources Comparison

| Source | Priority | Latency | Availability | Cost |
|--------|----------|---------|--------------|------|
| TWS (IBKR) | 100 | Live | Requires connection | IBKR fees |
| NautilusTrader (IB) | 100 | Live | Requires connection | IBKR fees |
| Polygon | 70 | Real-time | API key needed | Subscription |
| FMP | 60 | Delayed (15min) | API key | Free/Limited |
| Yahoo | 50 | Delayed (15min) | No key needed | Free |

## NautilusTrader Architecture

### Core Crates
- `nautilus_core` - UnixNanos, Timestamp, basic types
- `nautilus_model` - QuoteTick, TradeTick, Bar, Greeks, identifiers
- `nautilus_data` - Data client, aggregation, caching
- `nautilus_common` - Clock, timers, enums
- `nautilus_trading` - Strategy, portfolio, risk

### Key Data Types

```rust
// QuoteTick - bid/ask with fixed precision
struct QuoteTick {
    instrument_id: InstrumentId,
    bid_price: Price,      // Fixed precision decimal
    ask_price: Price,
    bid_size: Quantity,
    ask_size: Quantity,
    ts_event: UnixNanos,  // Event timestamp
    ts_init: UnixNanos,  // Init timestamp
}

// Bar - OHLCV candle
struct Bar {
    bar_type: BarType,
    open: Price,
    high: Price,
    low: Price,
    close: Price,
    volume: Quantity,
    ts_event: UnixNanos,
    ts_init: UnixNanos,
}

// Greeks - option sensitivities
struct GreeksData {
    delta: f64,
    gamma: f64,
    theta: f64,
    vega: f64,
    rho: f64,
}
```

## Current Aether Data Types

```rust
// MarketDataEvent - current implementation
struct MarketDataEvent {
    contract_id: i64,
    symbol: String,
    bid: f64,
    ask: f64,
    last: f64,
    volume: u64,
    timestamp: DateTime<Utc>,
    quote_quality: u32,
    source: String,        // NEW
    source_priority: u32,   // NEW
}
```

## Integration Options

### Option 1: Full NautilusTrader Core
Add entire `nautilus_trader` Rust workspace as dependency.

**Pros:**
- Battle-tested data types
- Built-in bar aggregation
- IBKR adapter already implemented
- Order book support

**Cons:**
- Heavy dependency (~50+ crates)
- Requires Rust edition 2024
- Complex upgrade path
- Version lock-in

### Option 2: Nautilus-Model Only
Use only `nautilus-model` crate for data types.

**Pros:**
- Lightweight (just types)
- No edition upgrade needed initially
- Easy to swap later
- Proven types

**Cons:**
- No aggregation built-in
- Still need our own data client
- May need version sync with Python agent

### Option 3: Current Architecture + Aggregator
Keep current types, enhance with `MarketDataAggregator`.

**Pros:**
- No new dependencies
- Uses source_priority we just added
- Gradual improvement
- Lower risk

**Cons:**
- Still using f64 (precision issues)
- Duplicate type definitions
- No bar aggregation

## FMP/Yahoo Integration Status

| Feature | FMP | Yahoo |
|---------|-----|-------|
| Quotes | ✅ | ✅ |
| Historical OHLCV | ✅ | ✅ |
| Options chain | Limited | ✅ |
| Fundamentals | ✅ | ❌ |
| News | ✅ | ❌ |
| Free tier | 250/day | Unlimited |

## Recommendations

### Immediate (This Sprint)
1. Wire `MarketDataAggregator` into backend_service
2. Add FMP/Yahoo as fallback sources when TWS disconnects
3. Use `source_priority` for conflict resolution

### Short-term (Next Sprint)
1. Add `nautilus-model` for types only
2. Map `QuoteTick` → internal types
3. Benchmark f64 vs fixed-precision

### Long-term
1. Evaluate full `nautilus_trader` integration
2. Use Nautilus bar aggregation
3. Integrate IBKR adapter via Nautilus

## Open Questions

1. Should we upgrade to Rust edition 2024 for `nautilus_model`?
2. How to handle Python agent compatibility?
3. Which data types to prioritize first?
4. FMP vs Yahoo for primary backup?

## References

- [NautilusTrader Docs](https://docs.nautilustrader.io/)
- [nautilus-model crate](https://crates.io/crates/nautilus-model)
- [Aether Market Data Architecture](./docs/MARKET_DATA_ARCHITECTURE.md)
