# Rust Quantitative Finance Libraries

This document surveys Rust crates that could replace C++ QuantLib-dependent code.

## Overview

The C++ codebase currently owns these QuantLib-dependent calculations:

- Box spread profit/ROI/implied rate calculations
- Greeks (Delta, Gamma, Theta, Vega, Rho)
- Convexity calculations
- VaR (Value at Risk)
- Position sizing algorithms

## Candidate Libraries

### 1. RustQuant (Recommended)

**Crates.io:** https://crates.io/crates/RustQuant  
**GitHub:** https://github.com/avhz/RustQuant  
**Documentation:** https://docs.rs/RustQuant  
**License:** MIT / Apache-2.0  
**Stars:** 1.7k  
**Active:** ~65 releases, 23K SLoC

#### Modules

| Module | C++ Equivalent | Description |
|--------|----------------|-------------|
| `instruments` | `BoxSpreadCalculator` | Options, Bonds, Money pricing |
| `greeks` | `GreeksCalculator` | Option sensitivities |
| `stochastics` | QuantLib processes | Brownian motion, CIR, Vasicek, Hull-White |
| `math` | Eigen/NLopt | Distributions, FFT, optimization, integration |
| `autodiff` | — | Algorithmic differentiation for gradients |
| `models` | — | Short rate models, curve models |
| `portfolio` | — | Position collection with risk metrics |
| `data` | — | CSV/JSON/Parquet I/O, Yahoo Finance |
| `time` | `MarketHours` | DayCounter, calendars, schedules |

#### What We Could Replace

```rust
// Option pricing (replaces GreeksCalculator)
use RustQuant::instruments::options::*;

// Black-Scholes
let price = BlackScholes::new(S, K, T, r, sigma, OptionType::Call);

// Greeks
let delta = BlackScholes::delta(S, K, T, r, sigma);
let gamma = BlackScholes::gamma(S, K, T, r, sigma);
let theta = BlackScholes::theta(S, K, T, r, sigma);
let vega  = BlackScholes::vega(S, K, T, r, sigma);
let rho   = BlackScholes::rho(S, K, T, r, sigma);

// Implied volatility
use RustQuant::math::roots::*;
let iv = implied_volatility_bs(price, S, K, T, r, OptionType::Call);

// Stochastic processes (replaces custom models)
use RustQuant::stochastics::*;
let geometric_bm = GeometricBrownianMotion::new(mu, sigma);
```

#### Pros

- Most comprehensive Rust quant library
- Actively maintained (65 releases)
- Well-documented with examples
- MIT/Apache dual license
- Covers most QuantLib functionality

#### Cons

- Not as battle-tested as QuantLib
- Some edge cases may differ from QuantLib
- Requires validation against known outputs

---

### 2. black_scholes

**Crates.io:** https://crates.io/crates/black_scholes  
**Purpose:** Simple Black-Scholes option pricing

```rust
use black_scholes::{BlackScholes, OptionType};

let bs = BlackScholes::new(100.0, 100.0, 1.0, 0.05, 0.2, OptionType::Call);
let price = bs.price();
let delta = bs.delta();
let gamma = bs.gamma();
```

**Best for:** Simple BS model without dependencies

---

### 3. implied-vol

**Crates.io:** https://crates.io/crates/implied-vol  
**Purpose:** Implied volatility calculation (Newton-Raphson)

```rust
use implied_vol::implied_volatility;

let iv = implied_volatility(
    option_price,
    underlying_price,
    strike_price,
    time_to_expiry,
    risk_free_rate,
    OptionType::Call,
);
```

---

### 4. hull_white

**Crates.io:** https://crates.io/crates/hull_white  
**Purpose:** Hull-White short rate model

---

### 5. simm-rs

**Crates.io:** https://crates.io/crates/simm-rs  
**Purpose:** Standard Initial Margin Model (SIMM) for derivatives

---

### 6. volsurf

**Crates.io:** https://crates.io/crates/volsurf  
**Purpose:** Volatility surface for derivatives pricing

---

## Comparison Matrix

| Feature | C++ (QuantLib) | RustQuant | black_scholes | implied-vol |
|---------|----------------|-----------|---------------|------------|
| Black-Scholes | ✅ | ✅ | ✅ | ❌ |
| Greeks | ✅ | ✅ | ✅ | ❌ |
| Implied Vol | ✅ | ✅ | ❌ | ✅ |
| Monte Carlo | ✅ | ✅ | ❌ | ❌ |
| Short Rate Models | ✅ | ✅ | ❌ | ❌ |
| Vol Surfaces | ✅ | Planned | ❌ | ❌ |
| Bonds | ✅ | ✅ | ❌ | ❌ |
| VaR | ✅ | Partial | ❌ | ❌ |

## Migration Effort

### Phase 1: Replace Simple Calculations (Low Risk)

- Box spread ROI/implied rate → RustQuant `instruments`
- DTE calculation → RustQuant `time`

### Phase 2: Replace Greeks (Medium Risk)

- Greeks calculations → RustQuant `instruments::greeks`
- Requires validation against C++ output

### Phase 3: Replace Advanced (Higher Risk)

- VaR calculations → Custom Rust or RustQuant math
- Stochastic processes → RustQuant `stochastics`

## Validation

Before production use, validate RustQuant outputs against:

1. C++ QuantLib outputs (existing test vectors)
2. Known analytical results
3. IBKR calculations (where available)

## Recommendation

**Use RustQuant** as the primary replacement for C++ QuantLib code:

1. Most comprehensive feature coverage
2. Active maintenance
3. Good documentation
4. MIT/Apache license (permissive)

**Keep C++ for:**

- Complex bond convexity calculations (verify RustQuant bond module first)
- IBKR-specific margin calculations (proprietary)

## References

- RustQuant Docs: https://docs.rs/RustQuant
- RustQuant Book: https://avhz.github.io/RustQuant/
- crates.io Finance: https://crates.io/categories/finance
