# Rust Quantitative Finance Libraries

Rust crates for quantitative finance, options pricing, and risk analytics.

## Overview

| Task | Rust | R |
|------|------|-----|
| Real-time data | ✅ Excellent | ❌ |
| Greeks | ✅ RustQuant | ✅ Built-in |
| VaR/CVaR | ⚠️ Manual | ✅ PerformanceAnalytics |
| Backtesting | ⚠️ Manual | ✅ quantstrat |
| Visualization | ⚠️ Plotly.rs | ✅ ggplot2 |

## Reinventing Detection

| Our Implementation | Alternative | Recommendation |
|------------------|-------------|----------------|
| `market_data::yahoo` | `yfinance-rs` crate | Consider replacing |
| Custom Greeks | RustQuant | Add as dependency |
| Manual VaR | RustQuant math | Add when needed |

## Recommended: RustQuant

**Crates.io:** https://crates.io/crates/RustQuant  
**GitHub:** https://github.com/avhz/RustQuant  
**License:** MIT / Apache-2.0

### Features

| Module | Description |
|--------|-------------|
| `instruments` | Options, bonds, box spreads |
| `greeks` | Delta, gamma, theta, vega, rho |
| `stochastics` | Brownian motion, CIR, Vasicek |
| `math` | Distributions, FFT, optimization |
| `autodiff` | Algorithmic differentiation |
| `data` | CSV/JSON/Parquet, **Yahoo Finance** |
| `time` | DayCounter, calendars, schedules |

### Usage

```rust
use RustQuant::instruments::{BlackScholes, OptionType};

// Black-Scholes pricing
let bs = BlackScholes::new(S, K, T, r, sigma, OptionType::Call);

// Greeks
let delta = BlackScholes::delta(S, K, T, r, sigma);
let gamma = BlackScholes::gamma(S, K, T, r, sigma);

// Implied volatility
use RustQuant::math::roots::*;
let iv = implied_volatility_bs(price, S, K, T, r, OptionType::Call);
```

### Yahoo Finance (REPLACING OURS?)

RustQuant has built-in Yahoo Finance:
```rust
use RustQuant::data::Data;
let data = Data::yahoo("SPY", start_date, end_date).unwrap();
```

**Alternative: yfinance-rs** (more active, options chains):
```rust
// yfinance-rs - has options chains!
let options = ticker.option_chain().await?;
```

## yfinance-rs (Recommended Alternative)

**Better than our Yahoo implementation:**
- Options chains
- Real-time streaming
- Polars DataFrame output
- Actively maintained (v0.7.2, Nov 2025)

```rust
use yfinance_rs::{Ticker, YfClient, Interval, Range};

let ticker = Ticker::new(&client, "SPX");
let options = ticker.option_chain().await?;  // SPX options!
let history = ticker.history(Some(Range::Y1), Some(Interval::D1), false).await?;
```

## Other Crates

| Crate | Purpose |
|-------|---------|
| `black_scholes` | Simple BS pricing |
| `implied-vol` | Implied volatility calculation |
| `hull_white` | Hull-White short rate model |
| `volsurf` | Volatility surface |

## Comparison with R

| Feature | RustQuant | roptions/RQuantLib |
|---------|-----------|-------------------|
| Black-Scholes | ✅ | ✅ |
| Greeks | ✅ | ✅ |
| Box spread | ✅ | ✅ |
| Yahoo Finance | ✅ | ❌ |
| Options chains | ❌ | ❌ |
| PerformanceAnalytics | ❌ | ✅ |
| quantstrat | ❌ | ✅ |

## NautilusTrader (NOT REINVENTING)

NautilusTrader is a general trading framework. We're focused on box spreads specifically.

**Similarities:**
- Rust core + Python control plane
- Event-driven architecture
- Multi-venue support

**Differences:**
- We: Box spread financing optimization
- NautilusTrader: General algorithmic trading

## Recommendation

**Phase 1:** Replace our Yahoo with `yfinance-rs` (has options chains!)
**Phase 2:** Add RustQuant for Greeks
**Phase 3:** R for backtesting (quantstrat, PerformanceAnalytics)

## See Also

- `docs/R_INTEGRATION_RESEARCH.md` - R integration architecture
- `docs/BOX_SPREAD_MATHEMATICS.md` - Box spread formulas
