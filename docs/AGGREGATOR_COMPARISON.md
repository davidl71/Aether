# NautilusTrader vs Aether Data Aggregators

## Summary

NautilusTrader and Aether have fundamentally different aggregation systems solving different problems. **They are complementary, not competitors.**

## NautilusTrader Aggregation

**Purpose**: OHLCV bar aggregation (tick → bar conversion)

### Architecture
- `BarAggregator` trait with `handle_quote()`, `handle_trade()`, `handle_bar()`
- `BarBuilder` for building OHLCV from tick stream
- Supports: time bars, tick bars, volume bars, value bars
- `OptionChainAggregator` for per-series option chain accumulation

### Key Characteristics
- **Source priority**: None (single source assumed)
- **Stale data**: Timer-based bar flushes
- **Python-first**: Full PyO3 bindings
- **Backfill**: `set_historical_mode()` for replay

### Data Flow
```
QuoteTick/TradeTick → BarAggregator → Bar → (feather/parquet)
```

## Aether Aggregation

**Purpose**: Multi-source quote resolution with priority-based selection

### Architecture
- `MarketDataAggregator` with HashMap + SQLite
- `DataSource` enum with priority weights (TWS=100, Polygon=70, FMP=60, Yahoo=50)
- TTL-based staleness (30s)
- Source metadata preserved in `ResolvedQuote`

### Key Characteristics
- **Source priority**: Core feature - selects best available quote
- **Stale data**: Explicit TTL with `staleness_ratio()`
- **Rust-only**: No Python dependency
- **Fallback**: Designed for TWS → Polygon → FMP fallback chain

### Data Flow
```
[Multi-source events] → Priority check → Best quote stored → [NATS/CLI/TUI]
```

## Comparison Table

| Aspect | Aether | Nautilus |
|--------|--------|----------|
| Primary purpose | Source selection | Bar building |
| Source priority | ✅ Priority enum | ❌ Not supported |
| Multi-source | ✅ Designed for | ❌ Single source |
| Stale detection | TTL + ratio | Timer-based |
| Option chains | Via external fetch | Built-in ATM tracking |
| Python required | No | Yes (PyO3) |
| Backfill support | Manual | Built-in |
| Output | Quotes/snapshots | OHLCV bars |

## Recommendation

**Keep both systems**. They serve different purposes:

1. **Aether aggregator** - Quote resolution for trading decisions (which price to use?)
2. **Nautilus bar aggregation** - Candlestick generation for charts (TUI volume pane)

### Integration Path

If we wanted to use Nautilus bar aggregation:
1. Use `nautilus_core` and `nautilus_model` crates (Rust-only, no Python)
2. Wire our `MarketDataEvent` → `QuoteTick` → Nautilus `BarAggregator`
3. Use output for TUI candlestick charts

The `nautilus_core` crate is edition 2024 and has significant API surface. Integration would require:
- Converting our `f64` prices to Nautilus `Price` type (fixed-precision)
- Mapping our `MarketDataEvent` to `QuoteTick`
- Handling the message bus integration

**For now**: Our aggregator is purpose-built and working. Nautilus is a potential future enhancement for bar aggregation specifically.

## References

- Nautilus aggregator: `nautilus_trader/crates/data/src/aggregation.rs`
- Nautilus option chains: `nautilus_trader/crates/data/src/option_chains/aggregator.rs`
- Aether aggregator: `agents/backend/crates/market_data/src/aggregator/`

## Additional Reading

- [A Basic Algo Trading System In Rust](https://medium.com/rustaceans/a-basic-algo-trading-system-in-rust-26a1c5488d47) - Paul Folbrecht
  - Similar trait+Arc DI pattern as Aether
  - Deliberately no async (thread-per-connection for websocket)
  - In-process channels vs NATS tradeoffs

## Python Trading Framework Ecosystem

Source: [pytrade.org trading frameworks](https://docs.pytrade.org/trading)

| Framework | Language | Stars | Key Strengths |
|-----------|----------|-------|---------------|
| freqtrade | Python | 28.7k | Crypto, ML optimization, Telegram |
| vnpy | Python | 25.7k | Chinese markets, event-driven |
| backtrader | Python | 14.6k | TA-Lib, IB, multi-asset |
| Lean | C# | 9.8k | QuantConnect, professional grade |
| hummingbot | Python | 8.2k | CEX+DEX, V2 strategy framework |
| jesse | Python | 5.7k | Crypto, multi-timeframe |
| nautilus_trader | Rust | 2.1k | High-perf, tokio async, Rust core |
| **Aether** | **Rust** | - | Multi-broker, fixed-income, NATS |

**Aether vs Python Frameworks:**
- Advantages: Native performance, type safety, multi-service NATS architecture, fixed-income focus
- Gaps: No backtesting engine, no ML integration, no built-in strategy library

**Aether vs NautilusTrader:**
- Both Rust-native with async
- Nautilus: Python-first (PyO3), bar aggregation focus
- Aether: Rust-first, multi-source priority aggregation, box spread optimization

## Symbol Discovery Resources

| Resource | Description |
|----------|-------------|
| [FinanceDatabase](https://github.com/JerBouma/FinanceDatabase) | 300k+ symbols (equities, ETFs, funds, indices, currencies, cryptos). Useful for instrument discovery and filtering by sector/country/exchange. |

**Aether note**: We rely on IBKR API for symbols. FinanceDatabase could complement for broader instrument discovery across multiple asset classes.
