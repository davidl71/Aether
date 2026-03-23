# NautilusTrader Integration - Rust Architecture

**Date**: 2026-03-23
**Status**: Researching integration with Rust core
**Purpose**: Document patterns from NautilusTrader for Rust-first architecture

---

## Overview

NautilusTrader is a high-performance algorithmic trading platform with **Rust core** and **Python bindings**. Aether has migrated to Rust-first, so this document focuses on **Rust-only integration patterns**.

Key difference from original research:
- We now use `nautilus-model` crate directly (not Python bindings)
- Our backend is pure Rust (`agents/backend`)
- Python agent (`agents/nautilus`) bridges to NATS

---

## Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Aether Architecture                      │
├─────────────────────────────────────────────────────────────┤
│  Rust Backend (agents/backend)                             │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ market_data │  │ broker_engine │  │  api            │  │
│  │ (sources)   │  │ (IBKR trait)  │  │  (REST/NATS)    │  │
│  └─────────────┘  └──────────────┘  └─────────────────┘  │
│         │                │                   │             │
│         └────────────────┼───────────────────┘             │
│                          │                                 │
│                   ┌───────▼───────┐                       │
│                   │ NATS/JetStream │                       │
│                   └───────────────┘                       │
├───────────────────────────────────────────────────────────┤
│  Python Agent (agents/nautilus)                           │
│  - NautilusTrader IBKR adapter                           │
│  - Publishes to NATS via protobuf                         │
└───────────────────────────────────────────────────────────┘
```

---

## NautilusTrader Crates for Rust Integration

| Crate | Purpose | Aether Use Case |
|-------|---------|-----------------|
| `nautilus-model` | Data types (QuoteTick, Bar) | Replace `MarketDataEvent` |
| `nautilus-data` | Data client, aggregation | Bar aggregation |
| `nautilus-core` | UnixNanos, timestamps | Type safety |
| `nautilus-common` | Clock, timers | Async handling |

---

## Data Type Mapping

### Current Aether Types

```rust
// market_data/src/model.rs
pub struct MarketDataEvent {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub source_priority: u32,
}
```

### NautilusTrader Types

```rust
// nautilus_model::data::QuoteTick
pub struct QuoteTick {
    pub instrument_id: InstrumentId,
    pub bid_price: Price,    // Fixed precision
    pub ask_price: Price,
    pub bid_size: Quantity,
    pub ask_size: Quantity,
    pub ts_event: UnixNanos,
    pub ts_init: UnixNanos,
}
```

### Comparison

| Aspect | Aether | Nautilus |
|--------|---------|----------|
| Price type | `f64` | `Price` (fixed precision) |
| Symbol | `String` | `InstrumentId` |
| Timestamp | `DateTime<Utc>` | `UnixNanos` |
| Source tracking | `source_priority` | Adapter-level |
| Quote quality | `u32` flags | Built-in validation |

---

## Integration Strategy

### Option 1: Direct Crate Dependency

```toml
# agents/backend/Cargo.toml
[dependencies]
nautilus-model = "0.55"
```

**Pros:**
- Type safety for prices
- Battle-tested data types
- No Python needed

**Cons:**
- Edition 2024 required
- Heavy dependency
- Version coupling

### Option 2: Use Concepts Only

Keep current types but mirror Nautilus patterns:
- Fixed-precision price wrapper
- `InstrumentId` format for symbols
- `ts_event`/`ts_init` timestamp distinction

**Pros:**
- No dependency
- Gradual adoption
- Lower risk

**Cons:**
- Duplicate type definitions
- No battle-testing

### Option 3: Proto Bridge (Current)

```
NautilusTrader (Python)
    ↓ QuoteTick
protobuf (messages.proto)
    ↓
Rust Backend (MarketDataEvent)
    ↓
MarketDataAggregator
```

**Status:** Already implemented via `agents/nautilus/types.py`

---

## Source Priority System

We added `source_priority` to `MarketDataEvent`:

```rust
enum DataSource {
    Nautilus = 100,  // Via Python agent
    Tws = 100,       // Direct TWS
    Polygon = 70,
    Fmp = 60,
    Yahoo = 50,
    Mock = 0,
}
```

### Priority Resolution

When multiple sources provide quotes for same symbol:

```rust
if incoming.priority > existing.priority {
    // Replace with higher priority source
} else if incoming.priority == existing.priority 
           && incoming.timestamp > existing.timestamp {
    // Same source, fresher data
}
```

---

## Bar Aggregation Comparison

### Current (Candlestick)

```rust
// tui_service/src/ui/candlestick.rs
pub struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: Option<f64>,
}
```

### Nautilus Bar

```rust
// nautilus_model::data::bar::Bar
pub struct Bar {
    pub bar_type: BarType,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Quantity,
    pub ts_event: UnixNanos,
    pub ts_init: UnixNanos,
}
```

**Nautilus Advantages:**
- Built-in aggregation (tick, volume, time)
- Type-safe price/volume
- BarType for different intervals

---

## Implementation Plan

### Phase 1: Research (Current)
- [x] Evaluate `nautilus-model` crate
- [x] Compare data types
- [ ] Benchmark f64 vs fixed precision
- [ ] Document integration options

### Phase 2: Light Integration
- [ ] Use `nautilus-model` for new types only
- [ ] Add `Price` wrapper around f64
- [ ] Update protobuf with source_priority (done)

### Phase 3: Full Integration (Future)
- [ ] Upgrade to edition 2024
- [ ] Replace `MarketDataEvent` with `QuoteTick`
- [ ] Use Nautilus bar aggregation

---

## Key Learnings from NautilusTrader

1. **Event-Driven Architecture**
   - Market data → Strategy evaluation via events
   - We implement this via NATS pub/sub

2. **InstrumentId Format**
   - Standardized symbol format: `SPX240412C05100000.CBOE`
   - Better than raw strings

3. **Timestamp Distinction**
   - `ts_event`: When event occurred
   - `ts_init`: When instance created
   - Useful for latency measurement

4. **Fixed-Precision Prices**
   - Avoids f64 floating-point errors
   - Critical for financial calculations

---

## Databento DBN Integration

**Databento Binary Encoding (DBN)** is an alternative binary format for market data with Rust support.

### Features Summary

| Feature | Databento Rust |
|---------|---------------|
| Historical data | ✅ timeseries, batch jobs |
| Live streaming | ✅ LiveClient |
| Fixed-precision prices | ✅ |
| Zstandard compression | ✅ |
| Symbol mapping | Built-in |
| Options data | ✅ (via instrument_def) |
| Symbology resolution | ✅ continuous → instrument |

### Record Types

```rust
// Trade messages
TradeMsg { price, size, action, side, ts_event, ... }

// Top-of-book (MBP-1)
Mbp1Msg { levels: [BidAskPair { bid_px, ask_px, bid_sz, ask_sz }] }

// 10-level book (MBP-10)
Mbp10Msg { levels: [BidAskPair; 10] }

// OHLCV candles
OhlcvMsg { open, high, low, close, volume }

// Instrument definitions
InstrumentDefMsg { ... }

// Status messages
StatusMsg { ... }
```

### Live Streaming

```rust
use databento::{
    dbn::{Dataset, Schema, TradeMsg, Mbp1Msg, SType},
    live::{Subscription, LiveClient},
};

// Subscribe to multiple schemas
client.subscribe(
    Subscription::builder()
        .symbols(vec!["ES.FUT", "NQ.FUT"])
        .schema(Schema::Trades)
        .build(),
).await?;

client.subscribe(
    Subscription::builder()
        .symbols("ES.FUT")
        .schema(Schema::Mbp1)  // Top-of-book
        .build(),
).await?;

client.start().await?;

while let Some(rec) = client.next_record().await? {
    if let Some(trade) = rec.get::<TradeMsg>() {
        // process trade
    } else if let Some(quote) = rec.get::<Mbp1Msg>() {
        // process quote
    }
}
```

### Batch Jobs (Large Historical Requests)

```rust
// Submit async job for large data
let job = client.batch().submit_job(
    &SubmitJobParams::builder()
        .dataset(Dataset::XnasItch)
        .schema(Schema::Trades)
        .symbols(vec!["AAPL", "MSFT"])
        .date_time_range(start..end)
        .encoding(Encoding::Dbn)
        .compression(Compression::Zstd)
        .build(),
).await?;

// List jobs, download when done
let paths = client.batch().download(
    &DownloadParams::builder()
        .job_id(&job.id)
        .output_dir("./data")
        .build(),
).await?;
```

### Symbology Resolution

```rust
// Resolve continuous contracts to instrument IDs
let resolution = client.symbology().resolve(
    &ResolveParams::builder()
        .dataset(Dataset::GlbxMdp3)
        .symbols(vec!["ES.c.0", "NQ.c.0"])  // Front-month
        .stype_in(SType::Continuous)
        .stype_out(SType::InstrumentId)
        .date_range(start..end)
        .build(),
).await?;
```

### When to Use

**Use Databento for:**
- Historical backtesting (batch jobs)
- Multi-venue analysis (CME, ICE, EUREX, etc.)
- When IBKR data has gaps
- Compressed data storage

**Skip for:**
- Real-time box spread trading (IBKR sufficient)
- Cost-sensitive projects

### Comparison with Nautilus

| Feature | Databento | Nautilus |
|---------|-----------|----------|
| Price precision | ✅ fixed | ✅ fixed |
| Historical | ✅ excellent | ❌ |
| Live | ✅ good | ❌ |
| Options chain | ✅ | ❌ |
| Greek calculation | ❌ | ✅ |
| Bar aggregation | ❌ | ✅ |
| Cost | Subscription | Free (IBKR) |

## References

- [NautilusTrader Docs](https://docs.nautilustrader.io/)
- [nautilus-model crate](https://crates.io/crates/nautilus-model)
- [Rust API Docs](https://nautechsystems.github.io/nautilus_docs/rust-api-latest/)
- [Databento Rust Client](https://github.com/databento/databento-rs)
- [DBN Encoding](https://github.com/databento/dbn)
- [Our Research](./nautilus_model_research.md)
