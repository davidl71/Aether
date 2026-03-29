# Market Data Provider Architecture

**Date**: 2026-03-26
**Status**: Analysis Complete - Refactoring Proposed

For an **operator-oriented “which source for what”** summary (intent and typical use cases), see [MARKET_DATA_INTEGRATION.md § Which source for what](./MARKET_DATA_INTEGRATION.md#which-source-for-what-operator-guide).

---

## 1. Current Trait Architecture

### 1.1 Quote Data Traits

| Trait | Location | Methods | Used By |
|-------|----------|---------|---------|
| `MarketDataSource` | model.rs:7 | `async fn next() -> Result<MarketDataEvent>` | CLI, TUI, backend |
| `MarketDataSourceFactory` | model.rs:19 | `name()`, `create()`, `requires_config()` | Provider registry |
| `SimpleMarketDataSourceFactory` | model.rs:35 | `name()`, `create()` | No-config providers |

### 1.2 Options Data Traits

| Trait | Location | Methods | Used By |
|-------|----------|---------|---------|
| `OptionsDataSource` | yahoo.rs:135 | `get_expirations(symbol)`, `get_chain(symbol, expiration_ts)` | Yield curve, CLI |

### 1.3 Position Data (Ad-hoc, no trait)

| Struct | Location | Type |
|--------|----------|------|
| `AlpacaPositionSource` | api/src/alpaca_positions.rs | struct (not trait) |
| `IbPositionSource` | api/src/ib_positions.rs | struct (not trait) |

---

## 2. Provider Implementation Matrix

| Provider | Quote Source | Options Source | Position Source | Factory |
|----------|--------------|-----------------|------------------|---------|
| **Yahoo** | `YahooFinanceSource` | `YahooOptionsSource` | — | ✓ |
| **Alpaca** | `AlpacaSource` (market_data) | — | `AlpacaPositionSource` (api) | ✓ |
| **Polygon** | `PolygonMarketDataSource` | `PolygonOptionsSource` | — | ✓ |
| **FMP** | `FmpMarketDataSource` | — | — | ✓ |
| **TWS/IB** | `IbAdapter` (ib_adapter) | — | `IbPositionSource` (api) | — |
| **Mock** | `MockMarketDataSource` | — | — | ✓ |

---

## 3. Factory Pattern Analysis

### 3.1 Current Usage

```rust
// Registry-based factory (works well)
pub fn provider_registry() -> &'static HashMap<&'static str, DynFactory> {
    static REGISTRY: OnceLock<HashMap<&'static str, DynFactory>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut m = HashMap::new();
        register(&mut m, "yahoo", YahooFinanceSourceFactory);
        register(&mut m, "fmp", FmpMarketDataSourceFactory);
        register(&mut m, "mock", MockMarketDataSourceFactory);
        register(&mut m, "polygon", PolygonMarketDataSourceFactory);
        register(&mut m, "alpaca", AlpacaSourceFactory);
        m
    })
}
```

### 3.2 Inconsistencies Found

#### Inconsistency 1: Quotes vs Options Separation
- Quotes use `MarketDataSource` trait
- Options use separate `OptionsDataSource` trait
- **Result**: Different abstractions for related data

#### Inconsistency 2: Factory Pattern Gaps
- Yahoo: `YahooFinanceSourceFactory` + `YahooOptionsSource` (no factory)
- Polygon: `PolygonMarketDataSourceFactory` + `PolygonOptionsSource` (no factory)
- **Result**: Options sources created manually, not via registry

#### Inconsistency 3: Position Sources Not Unified
- `AlpacaPositionSource` in `api` crate (not trait)
- `IbPositionSource` in `api` crate (not trait)
- **Result**: No common interface for positions

#### Inconsistency 4: Priority Mismatch
- `DataSource` enum (aggregator/mod.rs): hardcoded priorities
- `MarketDataEvent::source_priority`: set per-event
- **Result**: Two different priority systems not synced

---

## 4. Module Boundaries & Layering

### 4.1 Current Layer Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      tui_service                                │
│  (no direct market_data import - uses api crate)               │
└────────────────────────────┬────────────────────────────────────┘
                             │ api crate bridges
┌────────────────────────────▼────────────────────────────────────┐
│                         api crate                                │
│  - AlpacaPositionSource (struct)                                │
│  - IbPositionSource (struct)                                    │
│  - Shared config types                                          │
│  - Depends on: market_data, common, broker_engine               │
└────────────────────────────┬────────────────────────────────────┘
                             │ market_data crate is foundational
┌────────────────────────────▼────────────────────────────────────┐
│                      market_data crate                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Traits (model.rs, yahoo.rs)                            │    │
│  │  - MarketDataSource, MarketDataSourceFactory            │    │
│  │  - OptionsDataSource                                     │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Providers (alpaca.rs, yahoo.rs, polygon.rs, fmp.rs)   │    │
│  │  - Quote sources (implements MarketDataSource)         │    │
│  │  - Options sources (implements OptionsDataSource)       │    │
│  │  - Factories (for registry)                             │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Aggregator (aggregator/mod.rs)                         │    │
│  │  - DataSource enum, priority resolution                 │    │
│  └─────────────────────────────────────────────────────────┘    │
└────────────────────────────┬────────────────────────────────────┘
                             │ core types
┌────────────────────────────▼────────────────────────────────────┐
│                       common crate                               │
│  - MarketDataEvent (shared type)                                │
│  - No business logic                                            │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Current Dependency Issues

| Issue | Location | Problem |
|-------|----------|---------|
| Position sources in api crate | api/src/alpaca_positions.rs | Breaks separation - positions should be in market_data |
| Duplicate Alpaca | market_data + api | Quotes in market_data, positions in api |
| Factory not unified | OptionsDataSource | No factory pattern for options |
| TWS not in market_data | ib_adapter | TWS lives in separate crate |

### 4.3 Recommended Layer Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                        TUI / CLI                                │
│                         (consumers)                             │
└────────────────────────────┬────────────────────────────────────┘
                             │ uses crate::market_data
┌────────────────────────────▼────────────────────────────────────┐
│                     market_data crate                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  QUOTES MODULE                                          │    │
│  │  - MarketDataSource trait                               │    │
│  │  - {Provider}QuoteSource structs                        │    │
│  │  - {Provider}QuoteSourceFactory (for registry)          │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  OPTIONS MODULE                                         │    │
│  │  - OptionsDataSource trait                              │    │
│  │  - {Provider}OptionsSource structs                      │    │
│  │  - {Provider}OptionsSourceFactory (for registry)        │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  POSITIONS MODULE (proposed)                            │    │
│  │  - PositionSource trait                                │    │
│  │  - {Provider}PositionSource structs                     │    │
│  │  - {Provider}PositionSourceFactory                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  AGGREGATOR MODULE                                      │    │
│  │  - DataSource enum (priority config)                    │    │
│  │  - ResolvedQuote                                        │    │
│  └─────────────────────────────────────────────────────────┘    │
└────────────────────────────┬────────────────────────────────────┘
                             │ re-exports common types
┌────────────────────────────▼────────────────────────────────────┐
│                       common crate                               │
│  - MarketDataEvent, Quote, Position (shared types)              │
│  - NO business logic, NO provider code                           │
└─────────────────────────────────────────────────────────────────┘
```

### 4.4 Module Boundary Rules

| Rule | Description |
|------|-------------|
| **Quotes** | All quote sources in `market_data::quotes` module |
| **Options** | All options sources in `market_data::options` module |
| **Positions** | All position sources in `market_data::positions` module (new) |
| **Factories** | Each source type has a corresponding factory in same module |
| **No cross-layer** | api crate consumes market_data, does NOT implement sources |
| **common is shared** | Only types, no implementation |

---

## 5. Naming Conventions

### 5.1 Current Patterns

| Pattern | Example | Count |
|---------|---------|-------|
| `{Provider}Source` | `YahooFinanceSource`, `AlpacaSource` | 5 |
| `{Provider}MarketDataSource` | `FmpMarketDataSource`, `PolygonMarketDataSource` | 2 |
| `{Provider}OptionsSource` | `YahooOptionsSource`, `PolygonOptionsSource` | 2 |
| `{Provider}Factory` | `YahooFinanceSourceFactory` | 4 |
| `{Provider}Client` | `FmpClient`, `TaseClient` | 2 |

### 5.2 Recommendation

Standardize on:
- `{Provider}Source` for quote sources
- `{Provider}OptionsSource` for options sources  
- `{Provider}Factory` for factories
- `{Provider}Client` for HTTP clients (reused across source types)

---

## 6. Source Priority System

### 6.1 Current State

```rust
// aggregator/mod.rs
pub enum DataSource {
    Nautilus => 100,
    Tws => 100,
    Polygon => 70,
    Fmp => 60,
    Yahoo => 50,
    Mock => 0,
}
```

### 6.2 Issue: Not Synced with MarketDataEvent

The `source_priority` in `MarketDataEvent` is set manually per-source:
- Alpaca paper: 55
- Alpaca live: 75
- TWS: 100

These values are NOT derived from the `DataSource` enum.

### 6.3 Recommendation

Either:
1. **Option A**: Make `DataSource` the source of truth, derive `source_priority` from it
2. **Option B**: Keep per-event priority, add validation that values match expected ranges

---

## 7. Proposed Refactoring

### 7.1 Consistent Provider Structure

Each provider should follow this pattern:

```rust
// {provider}/{provider}.rs
mod provider {
    // HTTP client (reused)
    pub struct Client { ... }
    
    // Quote source
    pub struct QuoteSource { ... }
    impl MarketDataSource for QuoteSource { ... }
    
    // Quote factory (registered)
    pub struct QuoteSourceFactory;
    impl SimpleMarketDataSourceFactory for QuoteSourceFactory { ... }
    
    // Options source (optional)
    pub struct OptionsSource { ... }
    impl OptionsDataSource for OptionsSource { ... }
    
    // Options factory (optional, registered)
    pub struct OptionsSourceFactory;
    // impl SimpleMarketDataSourceFactory for OptionsSourceFactory { ... }
}
```

### 7.2 Registry Updates

Register both quote and options factories:

```rust
pub fn provider_registry() -> &'static HashMap<&'static str, DynFactory> {
    static REGISTRY: OnceLock<HashMap<&'static str, DynFactory>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut m = HashMap::new();
        // Quote factories
        register(&mut m, "yahoo", YahooQuoteSourceFactory);
        register(&mut m, "fmp", FmpQuoteSourceFactory);
        // Options factories  
        register(&mut m, "yahoo_options", YahooOptionsSourceFactory);
        register(&mut m, "polygon_options", PolygonOptionsSourceFactory);
        m
    })
}
```

### 7.3 Position Source Trait (Future)

```rust
pub trait PositionSource: Send + Sync {
    async fn fetch_positions(&self) -> anyhow::Result<Vec<Position>>;
    async fn fetch_account(&self) -> anyhow::Result<AccountInfo>;
}
```

---

## 7. Files to Update

| File | Change |
|------|--------|
| `market_data/src/yahoo.rs` | Rename to `QuoteSource`, add factory |
| `market_data/src/polygon.rs` | Add options factory, rename quote source |
| `market_data/src/fmp.rs` | Consistent naming |
| `market_data/src/lib.rs` | Update exports, registry |
| `aggregator/mod.rs` | Sync priorities with source_priority |
| `api/src/alpaca_positions.rs` | (Deferred - positions separate concern) |

---

## 8. Provider Comparison Table

### 8.1 Quote Sources (MarketDataSource)

| Provider | Source Struct | Factory | Trait Impl | Requires Config | Registry Key |
|----------|--------------|---------|------------|-----------------|--------------|
| Yahoo | `YahooFinanceSource` | `YahooFinanceSourceFactory` | SimpleMarketDataSourceFactory | No | "yahoo" |
| FMP | `FmpMarketDataSource` | `FmpMarketDataSourceFactory` | SimpleMarketDataSourceFactory | No | "fmp" |
| Polygon | `PolygonMarketDataSource` | `PolygonMarketDataSourceFactory` | MarketDataSourceFactory | Yes | "polygon" |
| Alpaca | `AlpacaSource` | `AlpacaSourceFactory` | MarketDataSourceFactory | Yes | "alpaca" |
| Mock | `MockMarketDataSource` | `MockMarketDataSourceFactory` | SimpleMarketDataSourceFactory | No | "mock" |

### 8.2 Options Sources (OptionsDataSource)

| Provider | Source Struct | Factory | Registry Key | Notes |
|----------|--------------|---------|--------------|-------|
| Yahoo | `YahooOptionsSource` | `YahooOptionsSourceFactory` | "yahoo" | No API key needed |
| Polygon | `PolygonOptionsSource` | `PolygonOptionsSourceFactory` | "polygon" | Requires API key |

### 8.3 Factory Pattern Comparison (Before vs After)

| Aspect | Before (Inconsistent) | After (Consistent) |
|--------|----------------------|-------------------|
| Quote factories | 4 implementations, different patterns | All use either `SimpleMarketDataSourceFactory` or `MarketDataSourceFactory` |
| Options factories | None - created manually | `YahooOptionsSourceFactory`, `PolygonOptionsSourceFactory` |
| Registry | Single `provider_registry()` for quotes | Two registries: `provider_registry()` (quotes) + `options_registry()` (options) |
| Creation | `create_provider()` for quotes only | `create_provider()` (quotes) + `create_options_provider()` (options) |
| Config handling | Ad-hoc per provider | `requires_config()` method on factory trait |

---

## 9. Acceptance Criteria

- [ ] All quote sources implement `MarketDataSource` consistently
- [ ] All options sources implement `OptionsDataSource` consistently
- [ ] Factory pattern used for all sources (via registry)
- [ ] Naming follows `{Provider}{Type}Source` pattern
- [ ] Priority values synced between enum and events
- [ ] Build passes, tests pass