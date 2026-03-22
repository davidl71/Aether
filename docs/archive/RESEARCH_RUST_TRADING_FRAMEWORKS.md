# Rust Trading Frameworks Research

**Date:** 2026-03-19  
**Project:** ib_box_spread_full_universal (Aether)  
**Purpose:** Evaluate Rust frameworks for stock/options trading and market simulation

> **⚠️ API Documentation Update:** The TWS API docs at `interactivebrokers.github.io/tws-api` are deprecated.  
> **New canonical path:** https://ibkrcampus.com/campus/ibkr-api-page/twsapi-doc/
- **TWS API Reference:** https://ibkrcampus.com/campus/ibkr-api-page/twsapi-ref/
- **ProtoBuf Reference:** https://ibkrcampus.com/campus/ibkr-api-page/protobuf-reference/
- **Web API (REST/CPAPI):** https://ibkrcampus.com/campus/ibkr-api-page/web-api/

Key architectural note from docs: *"The TWS API is a TCP Socket Protocol... any library that implements the TWS API must be sending and receiving the same data in the same format."* This confirms `yatws` (with its own native IBKR implementation) is a conformant alternative to `ibapi`.

---

## Executive Summary

Three crates stand out as directly relevant:

1. **yatws** — Production-grade async IBKR/TWS wrapper, used for 9-figure trading volume
2. **matchcore** — High-performance deterministic order book for market simulation
3. **longbridge-terminal** — Production-grade ratatui TUI trading terminal (623 stars), best reference for our TUI architecture

**Barter** (crypto-focused) and **NautilusTrader** (heavyweight platform) are not suitable for this project's IBKR/options focus. **plotters** could render option charts but is general-purpose charting, not trading-focused.

---

## 1. Barter Ecosystem

**Status: NOT SUITABLE** — Crypto-only, no IBKR support

| Crate | Purpose | IBKR Support |
|-------|---------|--------------|
| barter | Trading engine framework | ❌ Crypto exchanges only |
| barter-data | WebSocket market data | ❌ Binance, Coinbase, OKX, Gateio |
| barter-execution | ExecutionClient trait | ❌ No IBKR implementation |
| barter-integration | WebSocket/HTTP framework | ❌ |

**Verdict:** Barter is designed for crypto exchange connectivity. The `ExecutionClient` trait is interesting as a pattern but has no IBKR adapter. No multi-leg combo (box spread) support.

---

## 2. YATWS (Yet Another TWS Implementation)

**Status: HIGHLY RELEVANT** — Direct IBKR/TWS competitor to `ib_adapter`

### Key Features
- **Production-tested:** Used for 9 figures (hundreds of millions) of trading volume
- **Low latency:** ~3ms per order
- **Async + sync:** Observer pattern, subscription model, blocking calls
- **Options strategy builder:** `OptionsStrategyBuilder` with `bull_call_spread()`, etc.
- **Rate limiting:** Built-in compliance with IBKR API limits
- **Session recording/replay:** SQLite-based for testing/debugging
- **Thread-safe, type-safe**

### Architecture
```
IBKRClient
├── OrderManager      # Order placement, modification, tracking
├── AccountManager    # Portfolio, P&L, positions, executions
├── DataMarketManager # Real-time/historical market data
├── DataRefManager   # Contract details
├── DataNewsManager  # News headlines
├── DataFundamentalsManager
└── FinancialAdvisorManager
```

### Comparison with `ib_adapter`
| Feature | ib_adapter | yatws |
|---------|------------|-------|
| IBKR support | ✅ Via ibapi crate | ✅ Native |
| Async | ✅ tokio-based | ✅ AsyncObserver + Subscription |
| Order types | Basic | Market, Limit, Conditions, Strategies |
| Options strategies | ❌ None | ✅ Box, BullCallSpread, etc. |
| Rate limiting | ❌ | ✅ Built-in |
| Session replay | ❌ | ✅ SQLite |
| Production volume | Unknown | 9 figures |
| Order latency | Unknown | ~3ms |

### Code Example (YATWS)
```rust
// Create and place an order
let (contract, order_request) = OrderBuilder::new(OrderSide::Buy, 100.0)
    .for_stock("AAPL")
    .with_exchange("SMART")
    .with_currency("USD")
    .limit(150.0)
    .with_tif(TimeInForce::Day)
    .build()?;

let order_id = client.orders().place_order(contract, order_request)?;

// Subscribe to order lifecycle
let subscription = client.orders().subscribe_new_order(contract, order_request)?;
```

### Decision: Consider replacing `ib_adapter` with `yatws`
- ✅ More robust, production-tested
- ✅ Options strategy builder could handle box spreads
- ⚠️ Would require rewrite of `place_bag_order()` logic
- ⚠️ Dependency change — evaluate migration effort

---

## 3. Matchcore

**Status: RELEVANT** — Virtual market simulator candidate

### Key Features
- **High performance:** ~123ns per order submission
- **Deterministic:** Same input → Same output (replay, backtesting)
- **Single-threaded:** No locks, predictable latency
- **LMAX-inspired:** Disruptor pattern, ring buffer architecture

### Architecture
```
Command Reader → Ring Buffer → Matchcore Engine → Ring Buffer → Outcome Writer
```

### Use Cases
1. **Virtual market simulator** — Paper trading without IBKR connection
2. **Backtesting** — Deterministic replay
3. **Exchange simulation** — Test IB adapter against mock exchange

### Comparison with `barter-execution` mock
| Feature | barter MockExecution | matchcore |
|---------|---------------------|----------|
| Performance | Unknown | ~123ns/order |
| Deterministic | No | ✅ Yes |
| Order types | Basic | Market, Limit, Pegged, Iceberg |
| Latency model | Unknown | Predictable |

### Decision: Good candidate for market simulation layer
- Could be used to simulate IBKR-like market for paper trading
- Integrates with event-driven architecture

---

## 4. NautilusTrader / nautilus-execution

**Status: NOT SUITABLE** — Overkill, no IBKR native

### Key Features
- Full algorithmic trading platform
- ExecutionEngine, MatchingEngine, OrderEmulator
- Python bindings (PyO3)
- Backtesting + live deployment

### Why Not
- No native IBKR — would need custom adapter anyway
- Heavyweight (MB of dependencies)
- Designed for systematic/automated trading
- This project uses manual/半自动 box spread trading

---

## 5. Market Data Crates

### Databento (dbn)
- Official Databento client
- Real-time + historical tick data
- DBN (Databento Binary Encoding) for efficient传输
- **Use case:** If we ever switch from IBKR market data to Databento

### Yahoo Finance (yfinance-rs)
- Free market data
- Quotes, options, fundamentals
- **Use case:** Fallback when IBKR data is unavailable

### IBKR FLEX Parser (ib-flex)
- Type-safe Rust parser for IBKR FLEX XML statements
- Supports all major FLEX sections: Trades, Positions, Cash Transactions, etc.
- 20 asset categories (stocks, options, futures, warrants, T-Bills, CFDs, fractional shares)
- `rust_decimal` for financial precision (no floating-point errors)
- Optional API client — fetch statements programmatically from IB Client Portal
- **Use case:** Import historical trade data, tax reporting, performance analysis

### Alpaca CLI (apcacli)
- CLI for Alpaca trading API (`alpaca.markets`)
- Paper + live trading, US equities only (no options)
- **Use case:** Future broker diversification, US stock-only workflows

### Investments (konishchevdmitry/investments)
- CLI portfolio analyzer with IBKR support (statement parsing)
- Features: performance analysis, backtesting, tax reporting, portfolio rebalancing
- **Use case:** Reference for CLI portfolio display patterns; IBKR statement import insight

### OFX Parser (ofxy)
- Parses Open Financial Exchange (OFX) 1.6 files from banks and credit cards
- No unsafe code, idiomatic Rust, well-tested
- Used by rustledger-importer for ledger import
- **Use case:** Future bank/loan statement import into Loans tab

---

## 6. Quantitative Finance Crates

### RustQuant
- Option pricing (Black-Scholes, binomial trees)
- Monte Carlo simulation
- Greeks calculation
- **Use case:** Verify box spread theoretical values

### fin-primitives
- Validated price/quantity types
- Lock-free order book
- Technical indicators
- **Use case:** Could replace manual calculations in `crates/quant`

### implied-vol
- Pure Rust implied volatility (Peter Jäckel algorithm)
- **Use case:** IV calculation for options

---

## Recommendations

### Immediate (This Session)
1. **Fix `ib_adapter` compilation errors** (T-1773939738708565000)
   - `BagOrderLeg.con_id` — should use `contract.con_id` or add field
   - `PlaceBagOrderRequest.order_action` — missing field or wrong usage

2. **Evaluate yatws migration** (Future task)
   - Compare `place_bag_order` semantics with yatws options builder
   - yatws `OptionsStrategyBuilder` supports box spreads natively
   - Estimate migration effort vs. fixing current adapter

### Future Work
3. **Virtual market simulator** using matchcore
   - Paper trading without live IBKR connection
   - Deterministic backtesting

4. **Integrate RustQuant** for box spread pricing validation
   - Verify theoretical values match IBKR marks

5. **Study Longbridge Terminal for TUI patterns** (new)
   - ratatui multi-panel dashboard layout
   - Options chain display (strike ladder, expiration grid)
   - OAuth authentication flow
   - `--format json` dual-mode CLI design for AI agents
   - Watchlist / portfolio views

---

## Appendix: IBKR API Box Spread Workflow

Critical discovery from TWS API docs:

### Placing a BAG (combo) order requires conId resolution:

```
1. reqSecDefOptParams(underlying)  → get expirations + strikes (no throttling)
2. For each combo leg:
   a. Build incomplete Contract { symbol, secType:"OPT", expiry, strike }
   b. reqContractDetails(contract)  → get contract_id (conId)
3. Build Contract { secType:"BAG", combo_legs: [ComboLeg { conId, ratio, action, exchange }] }
4. placeOrder(order_id, contract, order)
```

### Key IBKR API methods for options:
- `reqSecDefOptParams` — option chain (expirations + strikes, no throttling)
- `reqContractDetails` — resolve conId for each leg
- `reqMktData` — live quotes + Greeks (delta, gamma, theta, vega)
- `calculateImpliedVolatility` / `calculateOptionPrice` — pricing validation
- `tickOptionComputation` — callback delivering all Greeks
- `exerciseOptions` — exercise/assign
- `reqScannerSubscription` with `Instrument:"NATCOMB"` — combo opportunity discovery

### Root issue resolved (2026-03-21):
`OptionContract` has `con_id: Option<i32>` (in `broker_engine::domain`). `place_bag_order` accesses it via `leg.contract.con_id` — the field exists and is correctly nested. ConId resolution is handled by `resolve_contract_details()` in `IbAdapter` and by `OptionsStrategyBuilder` in `yatws_adapter`.

### ProtoBuf Configuration (TWSAPI v10.35.01+)
IBKR migrated to Google Protocol Buffers. Key `ApiSettingsConfig` fields:
- `socketPort`: default 7497 (paper trading)
- `createApiMessageLogFile` / `includeMarketDataInLogFile`: **debugging**
- `loggingLevel`: "error" | "warning" | "info" | "detail"
- `downloadOpenOrdersOnConnection`: auto-download open orders on connect
- `masterClientId`: elevated privileges for API operations
- `rejectMessagesAboveMaxRate`: pacing limitation setting
- `maintainAndResubmitOrdersOnReconnect`: auto-reconnect orders

### TickAttrib (Tick Attributes for Price Ticks)
From `tickPrice` callback via `reqMktData`. Important for quote quality validation:

| Field | Type | Description |
|-------|------|-------------|
| `CanAutoExecute` | bool | Price available for auto-execution |
| `PastLimit` | bool | Bid < day's low OR ask > day's high |
| `PreOpen` | bool | Bid/ask from pre-open session |
| `Unreported` | bool | Trade is 'unreportable' (odd lots) |
| `BidPastLow` | bool | Bid lower than day's lowest low |
| `AskPastHigh` | bool | Ask higher than day's highest ask |

**Use case:** Flag unreliable quotes in combo net quote display. Show warning when `PastLimit` or `!CanAutoExecute`.

## Appendix: TUI / Candlestick Charting Reference

### Longbridge Terminal (623 stars — MOST RELEVANT)

**Terminal repo:** https://github.com/longbridge/longbridge-terminal  
**OpenAPI docs:** https://open.longbridge.com/docs  
**Language:** Rust (518K LOC) + TypeScript (10K) + Shell  
**Topics:** `ai-native`, `cli`, `longbridge`, `ratatui`, `terminal`, `tui`

**Status: BEST TUI REFERENCE** — Production-grade Rust TUI trading terminal using ratatui

#### Key Features
- **CLI + full-screen TUI** — Both command-line and interactive terminal UI
- **ratatui-based** — Same TUI framework as our tui_service
- **OAuth 2.0 authentication** — Clean auth pattern (no manual token management)
- **Real-time quotes** — WebSocket-based market data subscriptions
- **Options support** — `option-quote`, `option-chain` commands
- **Order management** — buy, sell, cancel, replace, order history
- **Portfolio/positions** — positions, balance, cash-flow
- **AI-native design** — `--format json` on all commands, designed for AI-agent tool-calling
- **Cross-platform** — Homebrew (macOS/Linux), install script

#### Architecture Patterns to Learn From
```
longbridge (CLI binary)
├── quote          # Real-time quotes with table formatting
├── kline          # OHLCV candlestick data
├── option-chain   # Option expiration/strike chain
├── orders         # Order management
├── positions      # Portfolio positions
└── watchlist      # Watchlist groups
```

- All commands support `--format json` for AI/ scripting consumption
- ANSI table rendering in CLI
- ratatui-based full-screen TUI for interactive monitoring
- Clean separation between CLI (flags/args) and TUI (interactive)

#### What We Can Learn
1. **ratatui TUI patterns** — How to structure a multi-panel trading dashboard
2. **Options data display** — Option chain rendering, strike ladders
3. **OAuth flow** — How to handle authentication cleanly in a CLI/TUI
4. **JSON output** — Dual-mode CLI (human-readable + JSON for AI agents)
5. **Watchlist/watchlist management** — Could extend our positions view

#### Relevance to Aether
- ratatui is already our TUI framework ✅
- We need options chain display for box spread strikes/expirations
- AI-native JSON output matches our TUI→API→AI pipeline
- Auth pattern could replace current manual token handling

---

### Other TUI Candlestick Projects

| Repo | Stars | Language | Description |
|------|-------|----------|-------------|
| codingskynet/tui-candlestick-chart | 9 | Rust | Basic candlestick rendering |
| a-khushal/TickerTUI | 1 | Rust | Real-time candlestick ASCII TUI |
| bubba311/chart_tui | 1 | Rust | Candlestick Charts TUI |

**Status:** All too small (≤9 stars) to be reliable references. Longbridge Terminal (623 stars) is the production-grade reference.

---

## Appendix: Crate Comparison Matrix

| Crate | IBKR | Options | Multi-leg | Sim | Perf | Prod Ready | TUI |
|-------|------|---------|-----------|-----|------|-----------|-----|
| ib_adapter (current) | ✅ | ⚠️ Basic | ❌ BAG broken | ❌ | Unknown | ⚠️ | ❌ |
| yatws | ✅✅ Native | ✅ Strategies | ✅ | ❌ | ~3ms | ✅✅ | ❌ |
| matchcore | ❌ | ❌ | ❌ | ✅✅ | ~123ns | ✅ | ❌ |
| pricelevel | ❌ | ❌ | ❌ | ✅✅ LOB | Unknown | ✅ | ❌ |
| barter | ❌ | ❌ | ❌ | ⚠️ | Unknown | ⚠️ | ❌ |
| nautilus | ⚠️ Custom | ✅ | ✅ | ✅✅ | High | ✅ | ❌ |
| longbridge-terminal | ⚠️ Longbridge only | ✅ | ❌ | ❌ | Unknown | ✅✅ | ✅✅ ratatui |
| plotters | ❌ | ⚠️ Candlestick | ❌ | ⚠️ | — | ⚠️ | ❌ |
