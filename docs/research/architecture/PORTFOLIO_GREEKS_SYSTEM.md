# Portfolio Greeks Calculation System

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document

## Overview

This document designs a comprehensive portfolio Greeks calculation system that calculates risk sensitivities (Greeks) for both option and non-option products, and aggregates them at the portfolio level for unified risk management. The system integrates with the investment strategy framework to provide portfolio-level risk metrics.

## Greeks Overview

**Greeks** are risk sensitivity measures representing how the value of a financial instrument changes with respect to changes in underlying parameters ([Wikipedia: Greeks (finance)](https://en.wikipedia.org/wiki/Greeks_(finance))).

### First-Order Greeks (Primary Risks)

1. **Delta (Δ):** Sensitivity to underlying asset price changes
2. **Vega (V):** Sensitivity to volatility changes
3. **Theta (Θ):** Sensitivity to time decay
4. **Rho (ρ):** Sensitivity to interest rate changes

### Second-Order Greeks (Secondary Risks)

1. **Gamma (Γ):** Rate of change of Delta (second derivative with respect to price)
2. **Vanna:** Sensitivity of Delta to volatility changes
3. **Charm:** Sensitivity of Delta to time decay
4. **Vomma:** Sensitivity of Vega to volatility changes
5. **Veta:** Sensitivity of Vega to time decay

## Greeks for Non-Option Products

While Greeks are traditionally associated with options, the concepts can be adapted for non-option products in portfolio risk management:

### Stocks and ETFs

**Delta:**
- **Definition:** For stocks, Delta = 1.0 (stock price moves 1:1 with itself)
- **Portfolio Delta:** Sum of all shares held (total equity exposure)
- **Formula:** `Delta_stock = 1.0 × quantity`

**Gamma:**
- **Definition:** Gamma = 0 (linear relationship, no curvature)
- **Rationale:** Stock price changes are linear; no acceleration of price movements

**Vega:**
- **Definition:** Vega = 0 (stocks don't have implied volatility pricing)
- **Note:** However, stocks are exposed to realized volatility; may want to track volatility exposure separately

**Theta:**
- **Definition:** Theta = 0 (no time decay for stocks)
- **Rationale:** Stocks don't expire; their value doesn't erode with time passage

**Rho:**
- **Definition:** Rho ≈ 0 for stocks (minimal direct interest rate sensitivity)
- **Rationale:** Interest rates affect stocks indirectly (via discount rates, earnings, etc.), but not directly priced in like bonds

### Bonds and Bond ETFs

**Delta:**
- **Definition:** Delta ≈ 0 (bonds don't move 1:1 with a stock index)
- **Alternative:** Can use **Duration** as Delta-equivalent for price sensitivity
- **Formula:** `Delta_bond = -Duration × Price × Δyield` (modified duration)

**Gamma:**
- **Definition:** Gamma = Convexity (second-order price sensitivity)
- **Formula:** `Gamma_bond = Convexity × Price × (Δyield)²`

**Vega:**
- **Definition:** Vega = 0 (bonds don't have implied volatility)
- **Note:** However, credit spreads and liquidity can be volatile

**Theta:**
- **Definition:** Theta ≈ 0 for bonds (time decay minimal unless approaching maturity)
- **Alternative:** Track time to maturity as separate metric

**Rho:**
- **Definition:** Rho is significant for bonds (interest rate sensitivity)
- **Formula:** `Rho_bond = -Duration × Price` (dollar duration)
- **Calculation:** Based on modified duration and bond price

### Cash and Cash Equivalents

**Delta:** 0 (no price sensitivity)
**Gamma:** 0
**Vega:** 0
**Theta:** 0
**Rho:** Minimal (only if earning variable interest rates)

### Foreign Currency Positions (ILS, EUR, etc.)

**Delta (Currency Exposure):**
- **Definition:** Sensitivity to exchange rate changes
- **Formula:** `Delta_currency = Position_Value_ILS × ΔFX_Rate_ILS_USD`
- **Example:** 100,000 ILS position has Delta of 100,000 × (1/ILS_USD_rate)

**Other Greeks:** Typically 0 unless currency has volatility/rate characteristics

## Portfolio-Level Greeks Aggregation

### Aggregation Formula

**Portfolio Delta:**
```cpp
PortfolioDelta = Σ(Delta_i × Quantity_i × Multiplier_i × FX_Rate_i)
```

Where:
- `Delta_i`: Delta for position i
- `Quantity_i`: Number of shares/contracts
- `Multiplier_i`: Contract multiplier (100 for options, 1 for stocks)
- `FX_Rate_i`: FX rate to convert to base currency (USD)

**Portfolio Gamma:**
```cpp
PortfolioGamma = Σ(Gamma_i × Quantity_i × Multiplier_i × FX_Rate_i)
```

**Portfolio Vega:**
```cpp
PortfolioVega = Σ(Vega_i × Quantity_i × Multiplier_i × FX_Rate_i)
```

**Portfolio Theta:**
```cpp
PortfolioTheta = Σ(Theta_i × Quantity_i × Multiplier_i × FX_Rate_i)
```

**Portfolio Rho:**
```cpp
PortfolioRho = Σ(Rho_i × Quantity_i × Multiplier_i × FX_Rate_i)
```

### Currency-Adjusted Greeks

For positions in foreign currencies (e.g., ILS), Greeks must be converted to base currency:

```cpp
Greeks_USD = Greeks_Local × Position_Value_USD / Position_Value_Local
```

**Example:**
- ILS position with Delta = 1.0 (local)
- Position value: 100,000 ILS = $25,000 USD (at 0.25 ILS/USD)
- Delta_USD = 1.0 × $25,000 / 100,000 ILS = 0.25

## Implementation

### Greeks Calculation for Options

```cpp
// native/src/risk_calculator.cpp
types::RiskMetrics RiskCalculator::calculate_option_greeks(
    const types::OptionContract& option,
    double underlying_price,
    double strike,
    double time_to_expiry,
    double volatility,
    double risk_free_rate,
    double dividend_yield) const {

    types::RiskMetrics greeks{};

    // Use Black-Scholes formulas (or more sophisticated model)
    // Delta
    greeks.delta = black_scholes_delta(
        option.type == types::OptionType::Call,
        underlying_price, strike, time_to_expiry,
        volatility, risk_free_rate, dividend_yield
    );

    // Gamma
    greeks.gamma = black_scholes_gamma(
        underlying_price, strike, time_to_expiry,
        volatility, risk_free_rate, dividend_yield
    );

    // Vega
    greeks.vega = black_scholes_vega(
        underlying_price, strike, time_to_expiry,
        volatility, risk_free_rate, dividend_yield
    );

    // Theta
    greeks.theta = black_scholes_theta(
        option.type == types::OptionType::Call,
        underlying_price, strike, time_to_expiry,
        volatility, risk_free_rate, dividend_yield
    );

    // Rho
    greeks.rho = black_scholes_rho(
        option.type == types::OptionType::Call,
        underlying_price, strike, time_to_expiry,
        volatility, risk_free_rate, dividend_yield
    );

    return greeks;
}
```

### Greeks Calculation for Non-Option Products

```cpp
// native/src/risk_calculator.cpp
types::RiskMetrics RiskCalculator::calculate_non_option_greeks(
    const types::Position& position,
    const std::string& product_type,  // "stock", "bond", "etf", "cash"
    double market_price,
    double fx_rate_usd = 1.0) const {

    types::RiskMetrics greeks{};
    greeks.delta = 0.0;
    greeks.gamma = 0.0;
    greeks.vega = 0.0;
    greeks.theta = 0.0;
    greeks.rho = 0.0;

    if (product_type == "stock" || product_type == "etf") {
        // Stock/ETF: Delta = 1.0 per share
        greeks.delta = 1.0;
        // Gamma = 0 (linear)
        // Vega = 0 (no IV)
        // Theta = 0 (no time decay)
        // Rho ≈ 0 (minimal direct rate sensitivity)

    } else if (product_type == "bond" || product_type == "bond_etf") {
        // Bond: Use duration and convexity
        double duration = get_bond_duration(position);  // Modified duration
        double convexity = get_bond_convexity(position);
        double bond_price = market_price;

        // Delta equivalent: -Duration × Price (price sensitivity to yield)
        // For 1% yield change: ΔPrice = -Duration × Price × 0.01
        greeks.delta = -duration * bond_price * 0.01;  // Dollar duration

        // Gamma equivalent: Convexity (curvature of price-yield relationship)
        greeks.gamma = convexity * bond_price * 0.0001;  // Approximate

        // Rho: Interest rate sensitivity (dollar duration)
        greeks.rho = -duration * bond_price;

        // Vega = 0, Theta = 0 (unless tracking time to maturity)

    } else if (product_type == "cash") {
        // Cash: All Greeks = 0
        // (unless earning variable interest rates, then minimal Rho)

    } else if (product_type == "foreign_currency") {
        // Foreign currency: Delta = FX rate sensitivity
        double position_value_usd = market_price * position.quantity * fx_rate_usd;
        // Delta represents sensitivity to FX rate changes
        greeks.delta = position_value_usd / fx_rate_usd;  // Simplified
    }

    // Convert to USD if needed
    if (fx_rate_usd != 1.0 && position.currency != "USD") {
        greeks.delta *= fx_rate_usd;
        greeks.gamma *= fx_rate_usd;
        greeks.vega *= fx_rate_usd;
        greeks.theta *= fx_rate_usd;
        greeks.rho *= fx_rate_usd;
    }

    return greeks;
}
```

### Portfolio-Level Greeks Aggregation

```cpp
// native/src/risk_calculator.cpp
types::RiskMetrics RiskCalculator::calculate_portfolio_greeks(
    const std::vector<types::Position>& positions,
    const std::map<std::string, double>& fx_rates) const {

    types::RiskMetrics portfolio_greeks{};

    for (const auto& pos : positions) {
        types::RiskMetrics position_greeks{};

        // Get FX rate for this position
        double fx_rate = 1.0;
        if (pos.currency != "USD") {
            auto it = fx_rates.find(pos.currency + "_USD");
            if (it != fx_rates.end()) {
                fx_rate = it->second;
            }
        }

        // Calculate Greeks for this position
        if (pos.contract.is_option()) {
            // Option Greeks
            position_greeks = calculate_option_greeks(
                pos.contract, pos.current_price,
                pos.contract.strike, pos.get_days_to_expiry(),
                get_implied_volatility(pos),
                get_risk_free_rate(),
                get_dividend_yield(pos.contract.symbol)
            );
        } else {
            // Non-option Greeks
            std::string product_type = determine_product_type(pos);
            position_greeks = calculate_non_option_greeks(
                pos, product_type, pos.current_price, fx_rate
            );
        }

        // Scale by quantity and contract multiplier
        double multiplier = pos.contract.is_option() ? 100.0 : 1.0;
        double position_value_usd = pos.current_price * pos.quantity * multiplier * fx_rate;

        // Aggregate Greeks (weighted by position value)
        portfolio_greeks.delta += position_greeks.delta * pos.quantity * multiplier;
        portfolio_greeks.gamma += position_greeks.gamma * pos.quantity * multiplier;
        portfolio_greeks.vega += position_greeks.vega * pos.quantity * multiplier;
        portfolio_greeks.theta += position_greeks.theta * pos.quantity * multiplier;
        portfolio_greeks.rho += position_greeks.rho * pos.quantity * multiplier;

        // Convert to USD if position is in foreign currency
        if (fx_rate != 1.0) {
            portfolio_greeks.delta *= fx_rate;
            portfolio_greeks.gamma *= fx_rate;
            portfolio_greeks.vega *= fx_rate;
            portfolio_greeks.theta *= fx_rate;
            portfolio_greeks.rho *= fx_rate;
        }
    }

    return portfolio_greeks;
}
```

### Bond Duration and Convexity Calculation

```cpp
// native/src/risk_calculator.cpp
double RiskCalculator::calculate_bond_duration(
    const types::Position& bond_position,
    double yield_to_maturity) const {

    // Modified Duration calculation
    // Duration = Σ(t × PV(CF_t)) / Price

    // For bond ETFs, use average duration from ETF prospectus
    // or calculate based on weighted average of holdings

    double duration = 0.0;

    if (bond_position.contract.symbol == "TLT") {
        // iShares 20+ Year Treasury Bond ETF
        duration = 18.5;  // Approximate modified duration
    } else if (bond_position.contract.symbol == "SHY") {
        // iShares 1-3 Year Treasury Bond ETF
        duration = 1.9;  // Approximate modified duration
    } else if (bond_position.contract.symbol == "BIL") {
        // SPDR Bloomberg 1-3 Month T-Bill ETF
        duration = 0.15;  // Very short duration
    } else {
        // Calculate from bond parameters if available
        // Otherwise use ETF prospectus data
        duration = get_etf_duration(bond_position.contract.symbol);
    }

    return duration;
}

double RiskCalculator::calculate_bond_convexity(
    const types::Position& bond_position,
    double yield_to_maturity) const {

    // Convexity calculation
    // For bond ETFs, use average convexity

    double convexity = 0.0;

    if (bond_position.contract.symbol == "TLT") {
        convexity = 450.0;  // Approximate convexity
    } else if (bond_position.contract.symbol == "SHY") {
        convexity = 5.0;  // Low convexity for short-term bonds
    } else {
        convexity = get_etf_convexity(bond_position.contract.symbol);
    }

    return convexity;
}
```

## Portfolio Risk Metrics Using Greeks

### Portfolio Delta Risk

**Delta-Neutral Portfolio:**
- **Target:** Portfolio Delta ≈ 0
- **Benefit:** Hedged against small price movements
- **Calculation:** `|PortfolioDelta| < threshold` (e.g., < 5% of portfolio value)

**Delta Exposure Limits:**
- **Maximum Delta:** Limit total delta exposure (e.g., ±50% of portfolio value)
- **Delta per Position:** Limit individual position delta (e.g., ±20% of portfolio)

### Portfolio Gamma Risk

**Gamma Exposure:**
- **High Positive Gamma:** Portfolio benefits from large price movements (favorable)
- **High Negative Gamma:** Portfolio suffers from large price movements (unfavorable)
- **Gamma Limits:** `|PortfolioGamma| < threshold`

### Portfolio Vega Risk

**Vega Exposure:**
- **Positive Vega:** Portfolio benefits from volatility increases
- **Negative Vega:** Portfolio suffers from volatility increases
- **Vega Limits:** `|PortfolioVega| < threshold` (e.g., < $10,000 per 1% vol change)

**Volatility Risk Management:**
- Monitor portfolio vega for options-heavy portfolios
- Hedge vega exposure if exceeds limits
- Consider volatility regime when setting vega targets

### Portfolio Theta Risk

**Theta Exposure:**
- **Negative Theta:** Portfolio loses value over time (time decay)
- **Positive Theta:** Portfolio gains value over time (rare)
- **Theta Management:** Monitor daily theta cost (dollars per day)

**Time Decay Management:**
- For option-heavy portfolios, monitor daily theta cost
- Consider closing positions with high theta before expiry
- Balance theta cost against expected returns

### Portfolio Rho Risk

**Rho Exposure:**
- **Positive Rho:** Portfolio benefits from interest rate increases
- **Negative Rho:** Portfolio suffers from interest rate increases
- **Rho Calculation:** Aggregate across bonds, options, and rate-sensitive positions

**Interest Rate Risk Management:**
- Monitor portfolio rho for bond-heavy portfolios
- Consider correlation with SHIR-based loans (variable rate loans)
- Hedge rho exposure if exceeds limits

## Integration with Investment Strategy Framework

### Portfolio Allocation Manager Integration

```cpp
// native/include/portfolio_allocation_manager.h
class PortfolioAllocationManager {
    // ... existing methods ...

    types::RiskMetrics get_portfolio_greeks() const {
        // Aggregate Greeks from all positions (IBKR + Israeli brokers)
        auto all_positions = get_all_positions();
        auto fx_rates = get_fx_rates();  // ILS/USD, EUR/USD, etc.
        return risk_calculator_.calculate_portfolio_greeks(all_positions, fx_rates);
    }

    bool is_delta_neutral(double tolerance = 0.05) const {
        auto greeks = get_portfolio_greeks();
        double portfolio_value = get_total_portfolio_value();
        double delta_percent = std::abs(greeks.delta) / portfolio_value;
        return delta_percent < tolerance;
    }

    double get_delta_exposure_percent() const {
        auto greeks = get_portfolio_greeks();
        double portfolio_value = get_total_portfolio_value();
        return (greeks.delta / portfolio_value) * 100.0;
    }

    bool check_greeks_limits(const types::RiskMetrics& greeks) const {
        // Check if portfolio Greeks are within acceptable limits
        double portfolio_value = get_total_portfolio_value();

        // Delta limit: ±50% of portfolio value
        if (std::abs(greeks.delta) > portfolio_value * 0.50) {
            return false;
        }

        // Vega limit: $10,000 per 1% vol change
        if (std::abs(greeks.vega) > 10000.0) {
            return false;
        }

        // Theta limit: -$500 per day (time decay cost)
        if (greeks.theta < -500.0) {
            return false;
        }

        // Rho limit: ±$50,000 per 1% rate change
        if (std::abs(greeks.rho) > 50000.0) {
            return false;
        }

        return true;
    }
};
```

### Greeks-Based Rebalancing Triggers

Add to rebalancing triggers:
- Portfolio Delta exceeds ±50% of portfolio value
- Portfolio Vega exceeds $10,000 per 1% volatility change
- Portfolio Theta exceeds -$500 per day (time decay cost)
- Portfolio Rho exceeds ±$50,000 per 1% rate change
- Greeks correlation with loan payments (SHIR rate changes)

## Configuration

```json
{
  "portfolio_allocation": {
    "greeks_limits": {
      "delta_max_percent": 50.0,      // ±50% of portfolio value
      "gamma_max": 1000.0,             // Maximum gamma exposure
      "vega_max_dollars": 10000.0,     // $10k per 1% vol change
      "theta_max_dollars_per_day": -500.0,  // -$500/day time decay
      "rho_max_dollars": 50000.0,      // ±$50k per 1% rate change
      "delta_neutral_target": true,    // Target delta-neutral portfolio
      "delta_tolerance_percent": 5.0   // ±5% tolerance for delta-neutral
    },
    "greeks_alert_thresholds": {
      "delta_alert_percent": 30.0,     // Alert if delta > 30%
      "vega_alert_dollars": 5000.0,    // Alert if vega > $5k
      "theta_alert_dollars_per_day": -250.0  // Alert if theta < -$250/day
    }
  }
}
```

## Risk Management Rules

### Portfolio-Level Greeks Limits

**Delta:**
- Maximum exposure: ±50% of portfolio value
- Target delta-neutral for hedged strategies
- Monitor correlation with loan payments (SHIR rate changes)

**Gamma:**
- Monitor for large positive/negative gamma
- High gamma = high sensitivity to large price movements
- Limit extreme gamma exposure

**Vega:**
- Maximum exposure: $10,000 per 1% volatility change
- Monitor in high-volatility environments
- Consider vega hedging for option-heavy portfolios

**Theta:**
- Maximum time decay cost: -$500 per day
- Monitor for option-heavy portfolios
- Balance theta cost against expected returns

**Rho:**
- Maximum exposure: ±$50,000 per 1% interest rate change
- Monitor correlation with variable loans (SHIR-based)
- Consider rho hedging for bond-heavy portfolios

### Greeks-Based Alerts

- Alert when portfolio Greeks exceed thresholds
- Alert when approaching Greeks limits
- Alert on Greeks correlation with loan rate changes (SHIR)

## Example Calculations

### Example 1: Mixed Portfolio (Stocks + Bonds + Options)

**Positions:**
- 100 shares SPY @ $500 = $50,000 (Delta = 100)
- 50 shares TLT @ $100 = $5,000 (Delta = -9.25, Rho = -$462.50)
- 10 SPY call options @ $5 (Delta = 0.5 each, Vega = $200 each, Theta = -$10/day each)

**Portfolio Greeks:**
- Delta = 100 (SPY) - 9.25 (TLT) + 500 (options) = 590.75
- Gamma = 0 (SPY) + 0 (TLT) + 50 (options) = 50
- Vega = 0 + 0 + 2,000 = $2,000 per 1% vol change
- Theta = 0 + 0 - 100 = -$100/day
- Rho = 0 - $462.50 + 50 = -$412.50 per 1% rate change

### Example 2: Israeli Broker Positions (ILS)

**Positions:**
- 10,000 ILS in Israeli stocks @ 100 ILS/share = 1,000,000 ILS
- FX Rate: 0.25 ILS/USD → $250,000 USD equivalent

**Greeks (USD):**
- Delta = 10,000 shares × 1.0 × 0.25 = 2,500 (USD equivalent)
- Gamma = 0
- Vega = 0
- Theta = 0
- Rho = 0

## Implementation Roadmap

### Phase 1: Greeks Calculation for Options (Week 1)
- [ ] Implement Black-Scholes Greeks formulas
- [ ] Add Greeks calculation to RiskCalculator
- [ ] Test with option positions
- [ ] Validate against known option pricing models

### Phase 2: Greeks Calculation for Non-Options (Week 2)
- [ ] Implement stock/ETF Greeks (Delta = 1.0)
- [ ] Implement bond Greeks (Duration-based Rho, Convexity-based Gamma)
- [ ] Add bond duration/convexity lookup (ETF data)
- [ ] Implement currency Greeks for foreign positions
- [ ] Test with various non-option products

### Phase 3: Portfolio Aggregation (Week 3)
- [ ] Implement portfolio-level Greeks aggregation
- [ ] Add currency conversion for foreign positions
- [ ] Integrate with PortfolioAllocationManager
- [ ] Add Greeks limits checking
- [ ] Test with mixed portfolios (IBKR + Israeli brokers)

### Phase 4: Risk Management Integration (Week 4)
- [ ] Add Greeks-based rebalancing triggers
- [ ] Implement Greeks limits and alerts
- [ ] Add Greeks reporting and monitoring
- [ ] Integrate with investment strategy framework
- [ ] Test end-to-end risk management

## Testing

### Unit Tests

- Option Greeks calculation (Black-Scholes formulas)
- Stock/ETF Greeks (Delta = 1.0)
- Bond Greeks (Duration, Convexity)
- Currency conversion for Greeks
- Portfolio aggregation

### Integration Tests

- Mixed portfolio Greeks calculation
- IBKR + Israeli broker position aggregation
- Greeks limits enforcement
- Rebalancing triggers based on Greeks

## References

1. [Wikipedia: Greeks (finance)](https://en.wikipedia.org/wiki/Greeks_(finance))
2. Black-Scholes Model
3. Bond Duration and Convexity
4. Portfolio Risk Management

---

**Next Steps:**
1. Review and approve design
2. Begin Phase 1 implementation (option Greeks)
3. Then Phase 2 (non-option Greeks)
