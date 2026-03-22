# R Integration for Box Spread Analytics

## Overview

R is well-suited for quantitative finance analytics, particularly options pricing and risk metrics. This document outlines the integration strategy between Aether (Rust) and R for enhanced box spread analytics.

## R Packages for Box Spread Analytics

### Core Pricing Packages

| Package | Purpose | Functions |
|---------|---------|-----------|
| `roptions` | Box spread pricing | `box.spread()`, `butterfly.*()`, `straddle.*()` |
| `RQuantLib` | QuantLib bindings | `EuropeanOption()`, `AmericanOption()`, `BinaryOption()` |
| `fOptions` | Option pricing | Black-Scholes, binomial models |

### Risk & Analytics Packages

| Package | Purpose | Functions |
|---------|---------|-----------|
| `PerformanceAnalytics` | Risk metrics | `VaR()`, `CVaR()`, `SharpeRatio()`, `maxDrawdown()` |
| `quantstrat` | Strategy backtesting | Full backtesting framework |
| `termstrc` | Yield curve | `NelsonSiegel()`, `Svensson()` |
| `YieldCurve` | Curve estimation | `Yield()`, `bootstrap()` |

### Visualization

| Package | Purpose |
|---------|---------|
| `ggplot2` | Static charts |
| `plotly` | Interactive 3D (vol surface) |
| `dygraphs` | Time series |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Aether (Rust)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │
│  │ Yahoo/Polygon│  │     TWS      │  │    Yield Curve      │ │
│  │  (quotes)   │  │  (options)   │  │     Writer          │ │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘ │
│         │                  │                      │             │
│         └──────────────────┼──────────────────────┘             │
│                            ▼                                    │
│                   ┌─────────────────┐                           │
│                   │    QuestDB      │                           │
│                   │  (historical)   │                           │
│                   └────────┬────────┘                           │
└──────────────────────────┼──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    R Plumber API                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ /box_spread    - Price box spreads                       │  │
│  │ /yield_curve   - Estimate term structure                 │  │
│  │ /backtest     - Historical strategy simulation            │  │
│  │ /risk         - VaR, CVaR, Greeks                      │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │
│  │ roptions     │  │ RQuantLib   │  │ PerformanceAnalytics │ │
│  │ termstrc     │  │ fOptions    │  │ quantstrat         │ │
│  └──────────────┘  └──────────────┘  └──────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Results → Aether TUI                         │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Details

### 1. Box Spread Pricing (roptions)

```r
library(roptions)

# Box spread: bull call + bear put at same strikes
result <- box.spread(
  k_long_call = 500,
  k_short_call = 510,
  k_long_put = 510,
  k_short_put = 500,
  c1 = 15.00,  # long call premium
  c2 = 8.50,   # short call premium
  p1 = 3.00,   # long put premium
  p2 = 1.50    # short put premium
)

# Theoretical value = (K2 - K1) * e^(-rT)
# Profit = theoretical - actual cost
```

### 2. Greeks via RQuantLib

```r
library(RQuantLib)

# European option Greeks
european <- EuropeanOption(
  type = "call",
  underlying = 500,
  strike = 500,
  dividendYield = 0.01,
  riskFreeRate = 0.05,
  maturity = 0.25,  # 3 months
  volatility = 0.20
)

# Returns: value, delta, gamma, vega, theta, rho
cat("Delta:", european$delta, "\n")
cat("Gamma:", european$gamma, "\n")
```

### 3. Yield Curve Estimation (termstrc)

```r
library(termstrc)

# Bootstrap yield curve from bond prices
yields <- c(0.05, 0.052, 0.055, 0.058, 0.06)
maturities <- c(1, 2, 3, 5, 10)

# Nelson-Siegel fit
ns <- NelsonSiegel(yields, maturities)

# Svensson extended model
svensson <- Svensson(yields, maturities)
```

### 4. Risk Metrics (PerformanceAnalytics)

```r
library(PerformanceAnalytics)

# Calculate VaR/CVaR from returns
returns <- c(0.01, -0.02, 0.005, -0.01, 0.02)
VaR(returns, p = 0.95, method = "historical")
CVaR(returns, p = 0.95)

# Sharpe ratio
SharpeRatio.annualized(returns)

# Maximum drawdown
maxDrawdown(returns)
```

### 5. Historical Backtesting (quantstrat)

```r
library(quantstrat)

# Initialize strategy
initDate <- "2023-01-01"
initEq <- 100000

# Add signals, rules, indicators
add.signal("macrossignal", ...)
add.rule("ruleSignal", ...)

# Run backtest
out <- applyStrategy(strategy, portfolios)

# Analyze results
blotter <- getPortfolio(portfolio.st)
returns <- blotter$summary$Net.Trading.PL
SharpeRatio(returns)
maxDrawdown(returns)
```

## Plumber API Endpoints

### /box_spread

```r
#* Price a box spread
#* @param strike1 Lower strike
#* @param strike2 Upper strike  
#* @param expiry_days Days to expiration
#* @param volatility Implied volatility
#* @post /box_spread
function(strike1, strike2, expiry_days, volatility) {
  # Calculate box spread price using roptions
  # Return theoretical value, Greeks, profit/loss
}
```

### /yield_curve

```r
#* Estimate yield curve from options
#* @param symbol Underlying symbol (SPX, NDX)
#* @param option_chain JSON of option chain
#* @post /yield_curve
function(symbol, option_chain) {
  # Bootstrap or Nelson-Siegel
  # Return term structure
}
```

### /backtest

```r
#* Run historical backtest
#* @param start_date Start date (YYYY-MM-DD)
#* @param end_date End date
#* @param symbols Vector of symbols
#* @post /backtest
function(start_date, end_date, symbols) {
  # Load historical data from QuestDB
  # Run quantstrat backtest
  # Return equity curve, metrics
}
```

## Data Flow

### Historical Data Collection

```
TWS options → Backend → QuestDB (ILP)
                         ↓
                  Historical CSV/Parquet
                         ↓
                    R (plumber)
```

### Real-time Analytics

```
Yahoo/Polygon quotes → Backend → R plumber API → Greeks/VaR → TUI
```

## Dependencies

### R Packages (DESCRIPTION)

```r
Imports:
    roptions,
    RQuantLib,
    PerformanceAnalytics,
    quantstrat,
    termstrc,
    YieldCurve,
    ggplot2,
    plotly,
    plumber
```

### System Requirements

```bash
# Install QuantLib (required for RQuantLib)
brew install quantlib

# Install R packages
R -e 'install.packages(c("roptions", "RQuantLib", "PerformanceAnalytics"))'
```

## Comparison: Rust vs R

| Task | Rust | R |
|------|------|---|
| Real-time data | ✅ Excellent | ❌ Slow |
| Web API serving | ✅ Axum/Actix | ⚠️ Plumber |
| Option pricing | ⚠️ QuantLib bindings | ✅ Native |
| Greeks | ⚠️ Manual | ✅ Built-in |
| Vol surface | ⚠️ Manual | ✅ plotly |
| VaR/CVaR | ⚠️ Manual | ✅ PerformanceAnalytics |
| Backtesting | ⚠️ Manual | ✅ quantstrat |
| Visualization | ⚠️ Plotly.rs | ✅ ggplot2 |

## Recommendations

### Phase 1: Quick Wins (1-2 weeks)
1. Box spread pricing via `roptions` + plumber
2. Greeks via `RQuantLib`
3. Risk metrics via `PerformanceAnalytics`

### Phase 2: Enhanced Analytics (2-4 weeks)
1. Yield curve estimation
2. Vol surface visualization
3. Historical backtesting framework

### Phase 3: Production (ongoing)
1. Performance optimization
2. Caching layer (Redis)
3. Monitoring and alerting

## References

- [roptions CRAN](https://cran.r-project.org/web/packages/roptions/roptions.pdf)
- [RQuantLib documentation](https://cran.r-project.org/web/packages/RQuantLib/RQuantLib.pdf)
- [PerformanceAnalytics](https://cran.r-project.org/web/packages/PerformanceAnalytics/index.html)
- [quantstrat](https://cran.r-project.org/web/packages/quantstrat/quantstrat.pdf)

## Related Documentation

| Document | Description |
|----------|-------------|
| `docs/archive/mathematical-finance-tools.md` | Box spread math, Greeks, VaR, put-call parity |
| `docs/archive/RUST_FINANCE_LIBRARIES.md` | RustQuant comparison, Rust finance libraries |
| `docs/archive/LEAN_STRATEGY_ARCHITECTURE.md` | C++/Python integration patterns |
| `docs/archive/NAUTILUS_LEARNINGS.md` | Rust+Python hybrid patterns (NautilusTrader) |

### Key Insights from Archived Docs

**Box spreads are mathematically simple:**
- Deterministic payoff: `(K2 - K1) * e^(-rT)`
- No stochastic calculus needed for pricing
- Greeks eliminated by construction (delta=0, gamma=0, vega=0)

**VaR is already implemented:**
```cpp
// From docs/archive/mathematical-finance-tools.md
double RiskCalculator::calculate_var_historical(
    const std::vector<double>& returns,
    double confidence_level) const {
    // Historical VaR - reference for R implementation
}
```
