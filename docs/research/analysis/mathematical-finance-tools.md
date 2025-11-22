# Mathematical Tools in Mathematical Finance

**Reference:** [Mathematical Finance - Mathematical Tools](https://en.wikipedia.org/wiki/Mathematical_finance#Mathematical_tools)

## Overview

Mathematical finance employs a variety of mathematical tools to model and analyze financial markets and instruments. This document outlines the key mathematical tools used in finance and their specific application to box spread arbitrage calculations in this project.

## Core Mathematical Tools

### 1. Stochastic Calculus

**Purpose:** Modeling random processes in financial markets, particularly in derivative pricing.

**Key Concepts:**
- **Brownian Motion (Wiener Process):** Models random price movements
- **Itô's Lemma:** Fundamental tool for stochastic differential equations
- **Stochastic Differential Equations (SDEs):** Describe asset price evolution

**Application to Box Spreads:**
While box spreads are theoretically risk-free (delta-neutral, gamma-neutral, vega-neutral), stochastic calculus underlies:
- The pricing models for individual options (Black-Scholes framework)
- Understanding why box spreads converge to their theoretical value
- Modeling the time decay component (theta) of individual legs

**References:**
- Used implicitly in option pricing models that determine individual leg prices
- Box spreads eliminate stochastic risk through delta-neutral construction

### 2. Partial Differential Equations (PDEs)

**Purpose:** Describe the evolution of financial variables over time.

**Key Equation:** Black-Scholes PDE
```
∂V/∂t + (1/2)σ²S²(∂²V/∂S²) + rS(∂V/∂S) - rV = 0
```

**Application to Box Spreads:**
- Individual options are priced using Black-Scholes or similar models
- Box spreads eliminate PDE complexity through synthetic position construction
- The theoretical value of a box spread is simply the strike width difference (no PDE solving needed)

**In This Codebase:**
```278:278:native/src/box_spread_strategy.cpp
        spread.buy_implied_rate = ((spread.buy_net_debit - strike_width) / strike_width) * (365.0 / days_to_expiry) * 100.0;
```

The implied rate calculation uses simple arithmetic, not PDE solving, because box spreads have deterministic payoffs.

### 3. Numerical Methods

**Purpose:** Approximate solutions to complex financial models that cannot be solved analytically.

**Key Techniques:**
- **Finite Difference Methods:** Solve PDEs numerically
- **Monte Carlo Simulations:** Price complex derivatives through random sampling
- **Binomial/Trinomial Trees:** Discrete-time option pricing models

**Application to Box Spreads:**
- **Not Required:** Box spreads have analytical solutions (strike width difference)
- **Used Indirectly:** Individual option prices may be calculated using numerical methods
- **Monte Carlo:** Could be used for stress testing or scenario analysis, but not for pricing

**In This Codebase:**
Box spread calculations use direct arithmetic rather than numerical methods:
```257:260:native/src/box_spread_strategy.cpp
    spread.net_debit = BoxSpreadCalculator::calculate_net_debit(spread);
    spread.theoretical_value = BoxSpreadCalculator::calculate_theoretical_value(spread);
    spread.arbitrage_profit = BoxSpreadCalculator::calculate_max_profit(spread);
    spread.roi_percent = BoxSpreadCalculator::calculate_roi(spread);
```

### 4. Optimization Techniques

**Purpose:** Determine optimal asset allocation strategies and portfolio optimization.

**Key Methods:**
- **Linear Programming:** Portfolio optimization with constraints
- **Quadratic Programming:** Mean-variance optimization
- **Dynamic Programming:** Multi-period optimization problems

**Application to Box Spreads:**
- **Position Sizing:** Kelly Criterion or fixed fractional sizing
- **Portfolio Allocation:** Determining how much capital to allocate to box spreads vs other strategies
- **Yield Curve Optimization:** Finding the best expiration dates and strike widths

**In This Codebase:**
```394:404:native/src/risk_calculator.cpp
    double kelly_fraction = (b * p - q) / b;

    // Use fractional Kelly (half Kelly is common)
    kelly_fraction *= 0.5;

    // Clamp to reasonable values
    kelly_fraction = std::max(0.0, std::min(kelly_fraction, 0.25));

    double position_size = account_value * kelly_fraction;

    return static_cast<int>(position_size / 100.0);  // Convert to contracts
```

The Kelly Criterion is used for optimal position sizing based on expected return and win probability.

### 5. Statistical Methods

**Purpose:** Analyze financial data, estimate model parameters, and assess risk.

**Key Techniques:**
- **Time Series Analysis:** Modeling price movements over time
- **Regression Analysis:** Estimating relationships between variables
- **Value at Risk (VaR):** Quantifying potential losses
- **Correlation Analysis:** Understanding relationships between assets

**Application to Box Spreads:**
- **Historical Analysis:** Analyzing past box spread opportunities
- **Risk Metrics:** VaR calculations for portfolio risk assessment
- **Yield Curve Analysis:** Statistical modeling of implied rates across expirations
- **Put-Call Parity Validation:** Statistical tests for market efficiency

**In This Codebase:**
```443:456:native/src/risk_calculator.cpp
double RiskCalculator::calculate_var_historical(
    const std::vector<double>& returns,
    double confidence_level) const {

    if (returns.empty()) return 0.0;

    std::vector<double> sorted_returns = returns;
    std::sort(sorted_returns.begin(), sorted_returns.end());

    size_t index = static_cast<size_t>((1.0 - confidence_level) * sorted_returns.size());
    index = std::min(index, sorted_returns.size() - 1);

    return -sorted_returns[index];
}
```

Historical VaR calculation for risk assessment.

## Box Spread Specific Mathematics

### Put-Call Parity

**Fundamental Relationship:**
```
C - P = S - K * e^(-rT)
```

Where:
- C = Call price
- P = Put price
- S = Underlying price
- K = Strike price
- r = Risk-free rate
- T = Time to expiration

**Box Spread Application:**
A box spread is constructed from:
- Long call at K1, Short call at K2
- Long put at K2, Short put at K1

The box spread value equals (K2 - K1) regardless of underlying price, creating a risk-free position.

**In This Codebase:**
```300:306:native/src/box_spread_strategy.cpp
    // Calculate disparity and put-call parity violation
    spread.buy_sell_disparity = BoxSpreadCalculator::calculate_buy_sell_disparity(
        spread.buy_profit, spread.sell_profit
    );
    spread.put_call_parity_violation = BoxSpreadCalculator::calculate_put_call_parity_violation(
        spread, spread.buy_implied_rate, spread.sell_implied_rate
    );
```

Put-call parity violations are detected and measured to identify arbitrage opportunities.

### Implied Rate Calculation

**Borrowing Scenario (Buying Box Spread):**
```
Implied Rate = ((Net Debit - Strike Width) / Strike Width) * (365 / Days to Expiry) * 100
```

**Lending Scenario (Selling Box Spread):**
```
Implied Rate = ((Strike Width - Net Credit) / Net Credit) * (365 / Days to Expiry) * 100
```

**In This Codebase:**
```276:281:native/src/box_spread_strategy.cpp
    if (days_to_expiry > 0 && spread.buy_net_debit > 0) {
        // Implied rate when buying (borrowing scenario: pay now, receive at expiry)
        spread.buy_implied_rate = ((spread.buy_net_debit - strike_width) / strike_width) * (365.0 / days_to_expiry) * 100.0;
    } else {
        spread.buy_implied_rate = 0.0;
    }
```

```293:298:native/src/box_spread_strategy.cpp
    if (days_to_expiry > 0 && spread.sell_net_credit > 0) {
        // Implied rate when selling (lending scenario: receive now, pay at expiry)
        spread.sell_implied_rate = ((strike_width - spread.sell_net_credit) / spread.sell_net_credit) * (365.0 / days_to_expiry) * 100.0;
    } else {
        spread.sell_implied_rate = 0.0;
    }
```

### Risk Metrics (Greeks)

**Box Spread Greeks:**
- **Delta:** 0.0 (delta-neutral by construction)
- **Gamma:** 0.0 (no sensitivity to price movements)
- **Theta:** ~0.0 (minimal time decay due to offsetting positions)
- **Vega:** 0.0 (no sensitivity to volatility changes)

**In This Codebase:**
```75:82:native/src/risk_calculator.cpp
    // Box spreads are delta-neutral
    risk.delta = 0.0;
    risk.gamma = 0.0;
    risk.theta = 0.0;  // Minimal time decay
    risk.vega = 0.0;   // IV-neutral

    risk.leverage = 1.0;  // No leverage
    risk.probability_of_profit = 1.0;  // Guaranteed if held to expiry
```

Box spreads are constructed to eliminate all Greeks, creating a risk-free position.

## Practical Applications in This Project

### 1. Opportunity Identification

**Mathematical Tools Used:**
- **Arithmetic Operations:** Net debit/credit calculations
- **Rate Conversions:** Annualized implied rate calculations
- **Comparison Logic:** Benchmark rate comparisons

**Implementation:**
```531:538:native/src/box_spread_strategy.cpp
bool BoxSpreadStrategy::beats_benchmark(
    const types::BoxSpreadLeg& spread,
    double benchmark_rate_percent,
    double min_spread_bps) const {

    double spread_bps = BoxSpreadCalculator::compare_to_benchmark(spread, benchmark_rate_percent);
    return spread_bps >= min_spread_bps;
}
```

### 2. Risk Management

**Mathematical Tools Used:**
- **Kelly Criterion:** Optimal position sizing
- **Value at Risk (VaR):** Historical and parametric methods
- **Risk-Reward Ratios:** Simple division operations

**Implementation:**
```62:98:native/src/risk_calculator.cpp
PositionRisk RiskCalculator::calculate_box_spread_risk(
    const types::BoxSpreadLeg& spread,
    double underlying_price,
    double implied_volatility) const {

    PositionRisk risk{};

    // Box spreads have defined risk
    risk.position_size = spread.net_debit * 100.0;  // Per contract
    risk.max_loss = spread.net_debit * 100.0;
    risk.max_gain = (spread.get_strike_width() - spread.net_debit) * 100.0;
    risk.expected_value = risk.max_gain;  // Box spreads converge to max value

    // Box spreads are delta-neutral
    risk.delta = 0.0;
    risk.gamma = 0.0;
    risk.theta = 0.0;  // Minimal time decay
    risk.vega = 0.0;   // IV-neutral

    risk.leverage = 1.0;  // No leverage
    risk.probability_of_profit = 1.0;  // Guaranteed if held to expiry

    if (risk.max_loss > 0) {
        risk.risk_reward_ratio = risk.max_gain / risk.max_loss;
    }
```

### 3. Yield Curve Analysis

**Mathematical Tools Used:**
- **Interpolation:** Connecting implied rates across expirations
- **Statistical Analysis:** Comparing rates across symbols
- **Time Value Calculations:** Days to expiry conversions

**Implementation:**
```222:228:native/include/box_spread_strategy.h
    types::YieldCurve build_yield_curve(
        const std::string& symbol,
        double strike_width,
        double benchmark_rate_percent = 5.0,
        int min_dte = 7,    // Minimum days to expiry
        int max_dte = 180   // Maximum days to expiry
    );
```

## Key Insights

### Why Box Spreads Are Mathematically Simple

1. **Deterministic Payoff:** The payoff is always (K2 - K1), regardless of underlying price
2. **No Stochastic Modeling Required:** Unlike individual options, box spreads don't require stochastic calculus for pricing
3. **Simple Arithmetic:** All calculations use basic arithmetic operations
4. **Risk Elimination:** Greeks are eliminated by construction, simplifying risk calculations

### When Advanced Math Is Still Relevant

1. **Individual Option Pricing:** The options themselves are priced using Black-Scholes or similar models
2. **Market Microstructure:** Understanding bid-ask spreads and liquidity requires statistical analysis
3. **Portfolio Optimization:** Determining optimal allocation across multiple box spreads
4. **Risk Management:** VaR and other risk metrics require statistical methods

## References

- [Mathematical Finance - Wikipedia](https://en.wikipedia.org/wiki/Mathematical_finance#Mathematical_tools)
- Black-Scholes Model: Used implicitly in option pricing
- Put-Call Parity: Fundamental relationship for box spread construction
- Kelly Criterion: Position sizing optimization
- Value at Risk (VaR): Risk measurement methodology

## Summary

While mathematical finance employs sophisticated tools like stochastic calculus, PDEs, and numerical methods, box spread arbitrage calculations in this project primarily use:

1. **Simple Arithmetic:** Net debit/credit, implied rates, profit calculations
2. **Put-Call Parity:** Fundamental relationship for box spread construction
3. **Statistical Methods:** Risk metrics, VaR, position sizing
4. **Optimization:** Kelly Criterion for position sizing

The mathematical simplicity of box spreads is their key advantage: they eliminate stochastic risk through construction, allowing for straightforward calculations while still providing risk-free returns.
