# Box Spread Mathematics

Mathematical foundations for box spread pricing and analytics.

## Key Formulas

### Put-Call Parity

```
C - P = S - K * e^(-rT)
```

Where: C=call, P=put, S=underlying, K=strike, r=risk-free rate, T=time

### Box Spread Value

Theoretical value (deterministic):
```
Box Value = (K2 - K1) * e^(-rT)
```

### Implied Rate (Buying Box)

```
Implied Rate = ((Net Debit - Strike Width) / Strike Width) * (365 / Days) * 100
```

### Implied Rate (Selling Box)

```
Implied Rate = ((Strike Width - Net Credit) / Net Credit) * (365 / Days) * 100
```

## Greeks for Box Spreads

| Greek | Value | Reason |
|-------|-------|--------|
| Delta | 0 | Delta-neutral by construction |
| Gamma | 0 | No price sensitivity |
| Theta | ~0 | Minimal time decay |
| Vega | 0 | IV-neutral |

## Risk Metrics

### VaR (Value at Risk)

Historical method:
```r
VaR = -quantile(returns, 1 - confidence)
```

### CVaR (Conditional VaR)

```r
CVaR = -mean(returns[returns < -VaR])
```

### Kelly Criterion

```r
kelly <- (b * p - q) / b  # b=odds, p=win prob, q=1-p
kelly_fraction <- kelly * 0.5  # half Kelly for safety
```

## R Implementation

### roptions Package

```r
library(roptions)

# Box spread pricing
box.spread(
  k_long_call = 500,
  k_short_call = 510,
  k_long_put = 510,
  k_short_put = 500,
  c1 = 15.00,  # long call premium
  c2 = 8.50,   # short call premium
  p1 = 3.00,   # long put premium
  p2 = 1.50    # short put premium
)
```

### RQuantLib Greeks

```r
library(RQuantLib)

# European option with Greeks
european <- EuropeanOption(
  type = "call",
  underlying = 500,
  strike = 500,
  dividendYield = 0.01,
  riskFreeRate = 0.05,
  maturity = 0.25,
  volatility = 0.20
)

# Returns: value, delta, gamma, vega, theta, rho
```

## References

- [Mathematical Finance - Wikipedia](https://en.wikipedia.org/wiki/Mathematical_finance)
- [roptions CRAN](https://cran.r-project.org/web/packages/roptions)
- [RQuantLib CRAN](https://cran.r-project.org/web/packages/RQuantLib)
