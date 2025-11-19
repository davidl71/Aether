# Investment Strategy Plan
## Portfolio Allocation Framework Based on Convexity, Skew, Cash Management, ETFs, and T-Bills/Bonds

**Date:** 2025-01-18
**Status:** Planning Phase
**Integration:** IBKR Box Spread Generator

---

## Executive Summary

This document outlines a comprehensive investment strategy that integrates:
- **Convexity optimization** through bond barbell strategies
- **Volatility skew** considerations for risk-adjusted returns
- **Long cash management** with tiered allocation
- **ETF investments** for diversification and liquidity
- **T-bill/bond targets** for short box rate management
- **Spare cash allocation** using box spreads and short-term instruments

The strategy is designed to work alongside the existing IBKR box spread arbitrage system, providing portfolio-level allocation decisions that optimize risk-adjusted returns while maintaining liquidity for trading opportunities.

---

## 1. Strategy Components

### 1.1 Convexity Optimization

**Objective:** Enhance portfolio convexity to improve risk-adjusted returns during interest rate fluctuations.

**Approach - Barbell Strategy:**
- Allocate bonds across short-term and long-term maturities
- Avoid intermediate-term bonds (which have lower convexity)
- **Target Allocation:**
  - Short-term bonds (1-3 years): 40% of bond allocation
  - Long-term bonds (20+ years): 60% of bond allocation
  - **ETF Implementation:**
    - Short-term: iShares 1-3 Year Treasury Bond ETF (SHY)
    - Long-term: iShares 20+ Year Treasury Bond ETF (TLT)

**Benefits:**
- Higher convexity = better price appreciation when rates fall
- Less depreciation when rates rise
- Better risk-adjusted performance in volatile rate environments

**Integration with Box Spreads:**
- Monitor box spread implied rates vs. bond yields
- Adjust allocation when box spreads offer superior risk-free returns

---

### 1.2 Volatility Skew Management

**Objective:** Incorporate assets with positive skewness to balance return distribution and reduce tail risk.

**Approach:**
- Identify assets with positive return skew (asymmetric upside)
- Allocate to equity ETFs with positive skew characteristics
- Balance negative-skew assets (like some fixed income) with positive-skew equity exposure

**Target Allocations:**
- **Positive Skew Assets:** 60% of equity allocation
  - SPDR S&P 500 ETF (SPY) - broad market exposure
  - Growth-oriented ETFs for asymmetric upside potential
- **Neutral/Negative Skew Assets:** 40% of equity allocation
  - Value ETFs, dividend-focused strategies

**Risk Management:**
- Monitor portfolio-level skewness metrics
- Rebalance when skewness deviates from target
- Use box spreads to hedge negative skew exposure in fixed income

---

### 1.3 Long Cash Management (Tiered Allocation)

**Objective:** Optimize cash holdings across liquidity tiers while maximizing risk-adjusted returns.

**Three-Tier Cash Structure:**

#### Tier 1: Immediate Liquidity (5-10% of portfolio)
- **Purpose:** Trading capital, emergency reserve, opportunity fund
- **Vehicles:**
  - Money market funds
  - Ultra-short T-bills (0-1 month)
  - **ETF:** SPDR Bloomberg 1-3 Month T-Bill ETF (BIL)
- **Characteristics:**
  - Maximum liquidity (same-day access)
  - Minimal yield (near zero)
  - Serves as buffer for box spread opportunities

#### Tier 2: Short-Term Cash (10-15% of portfolio)
- **Purpose:** Deployed cash earning competitive returns with moderate liquidity
- **Vehicles:**
  - **Primary:** Short box spreads (30-90 day DTE) targeting T-bill + 0.5-1.0% yield
  - **Alternative:** Short-term bond ETFs (SHY) if box spreads unavailable
  - T-bills (1-3 month maturity ladder)
- **Characteristics:**
  - Liquidity: 30-90 days
  - Yield: T-bill rate + 0.5-1.5% (via box spreads)
  - Tax-efficient (box spread gains often taxed as capital gains)

#### Tier 3: Strategic Cash (5-10% of portfolio)
- **Purpose:** Longer-term cash allocation earning higher returns
- **Vehicles:**
  - Short-term bond ETFs (SHY, IEF)
  - T-bill ladders (3-12 month maturities)
  - Longer-dated box spreads (90-180 day DTE) if rates justify
- **Characteristics:**
  - Liquidity: 3-12 months
  - Yield: Intermediate-term rates
  - Rebalancing flexibility

**Total Cash Allocation: 20-35% of portfolio**

---

### 1.4 ETF Investment Strategy

**Objective:** Provide diversified market exposure with liquidity and cost efficiency.

#### Equity ETFs (40-50% of portfolio)

**Core Holdings:**
- **SPDR S&P 500 ETF (SPY):** 30% of equity allocation
  - Broad U.S. equity exposure
  - High liquidity for rebalancing
  - Low expense ratio (0.0945%)

**Supplemental Holdings:**
- **Growth ETFs:** 20% of equity allocation
  - Positive skew exposure
  - Growth-oriented for asymmetric upside
- **International ETFs:** 20% of equity allocation
  - Geographic diversification
- **Sector ETFs:** 15% of equity allocation
  - Targeted exposure based on market conditions
- **Alternative ETFs:** 15% of equity allocation
  - Commodities, REITs for diversification

#### Fixed Income ETFs (25-35% of portfolio)

**Short-Term (Barbell - Short End):**
- **iShares 1-3 Year Treasury Bond ETF (SHY):** 40% of bond allocation
  - Duration: ~2 years
  - Low interest rate risk
  - High liquidity

**Long-Term (Barbell - Long End):**
- **iShares 20+ Year Treasury Bond ETF (TLT):** 60% of bond allocation
  - Duration: ~17 years
  - High convexity
  - Significant interest rate sensitivity

**Alternative:**
- **iShares 7-10 Year Treasury Bond ETF (IEF):** Used for rebalancing or tactical adjustments
- **Corporate Bond ETFs:** For credit spread exposure (if desired)

---

### 1.5 T-Bill/Bond Target Strategy for Short Box Rate Management

**Objective:** Use T-bills and bonds to establish rate targets for evaluating box spread opportunities.

**Rate Targeting Framework:**

#### Target Rate Calculation
```
Target Short Box Rate = max(
    T-bill rate (appropriate maturity) + 0.5%,
    Current best box spread implied rate,
    Minimum acceptable return (e.g., 4% annualized)
)
```

#### Implementation:
1. **Monitor T-bill Rates:**
   - 1-month T-bill rate: For Tier 1 cash benchmarks
   - 3-month T-bill rate: For Tier 2 cash benchmarks
   - 6-12 month T-bill rates: For Tier 3 cash benchmarks

2. **T-bill Ladder:**
   - Construct ladder with maturities matching box spread DTE targets
   - Example: 30, 60, 90-day T-bills for matching 30, 60, 90-day box spreads

3. **Box Spread Evaluation:**
   - Compare box spread implied rate vs. corresponding T-bill rate
   - Execute box spreads when spread > T-bill + 0.5% (after commissions)
   - Prefer box spreads for tax efficiency (capital gains vs. ordinary income)

4. **Bond Positioning:**
   - Use bond ETFs for longer-term rate exposure
   - Adjust bond allocation based on rate outlook and convexity optimization

**Rate Target Allocation:**
- **T-bills (direct):** 10-15% of portfolio
  - Ladder structure matching box spread opportunities
  - Direct ownership for precise maturity matching
- **T-bill ETFs (BIL):** 5-10% of portfolio
  - For ultra-short liquidity needs
  - Automatic rolling for convenience

---

### 1.6 Spare Cash Allocation Strategy

**Objective:** Deploy excess cash beyond operational needs into optimal risk-adjusted investments.

**Allocation Priority (when spare cash identified):**

1. **First Priority: Box Spread Opportunities**
   - Check if any box spreads meet:
     - Implied rate > T-bill rate + 0.5%
     - DTE: 30-180 days
     - Minimum ROI: 0.5% absolute
     - Liquidity score acceptable
   - Allocate up to 50% of spare cash to box spreads

2. **Second Priority: Short-Term Bond ETFs**
   - If no box spreads available
   - Allocate to SHY or IEF for intermediate yield
   - Provides liquidity for future box spread opportunities

3. **Third Priority: T-bill Ladder**
   - Direct T-bill purchases matching targeted maturities
   - Reinvests into box spreads when rates improve

4. **Fourth Priority: Equity ETFs (if risk budget allows)**
   - Deploy to SPY or other equity ETFs
   - Only if spare cash exceeds cash allocation targets
   - Maintains strategic asset allocation

**Spare Cash Definition:**
- Cash exceeding Tier 1 immediate liquidity needs
- Cash not needed for upcoming box spread opportunities (within 30 days)
- Cash beyond operational reserve requirements

---

## 2. Portfolio Allocation Framework

### 2.1 Target Allocation Model

**Conservative Model:**
```
Cash & Cash Equivalents:     30%  (Tier 1: 8%, Tier 2: 12%, Tier 3: 10%)
Fixed Income ETFs:           35%  (Short: 14%, Long: 21%)
Equity ETFs:                 30%  (SPY: 9%, Growth: 6%, Intl: 6%, Other: 9%)
Box Spreads:                  5%  (Deployed from Tier 2 cash)
```

**Moderate Model:**
```
Cash & Cash Equivalents:     25%  (Tier 1: 7%, Tier 2: 10%, Tier 3: 8%)
Fixed Income ETFs:         30%  (Short: 12%, Long: 18%)
Equity ETFs:               40%  (SPY: 12%, Growth: 8%, Intl: 8%, Other: 12%)
Box Spreads:                5%  (Deployed from Tier 2 cash)
```

**Aggressive Model:**
```
Cash & Cash Equivalents:     20%  (Tier 1: 5%, Tier 2: 8%, Tier 3: 7%)
Fixed Income ETFs:           25%  (Short: 10%, Long: 15%)
Equity ETFs:                 50%  (SPY: 15%, Growth: 10%, Intl: 10%, Other: 15%)
Box Spreads:                  5%  (Deployed from Tier 2 cash)
```

### 2.2 Dynamic Rebalancing Rules

**Rebalancing Triggers:**
1. **Allocation Deviation:** Any asset class deviates >5% from target
2. **Box Spread Opportunity:** High-quality box spread identified requiring capital
3. **Rate Changes:** Significant rate movements (>25 bps) affecting targets
4. **Monthly Review:** Systematic monthly rebalancing check
5. **Market Regime Change:** Major market shifts requiring strategy adjustment

**Rebalancing Process:**
1. Assess current allocations vs. targets
2. Identify rebalancing needs (prioritize low-cost moves)
3. Evaluate tax implications (prefer tax-loss harvesting)
4. Execute trades (minimize transaction costs)
5. Update allocation tracking

---

## 3. Integration with Box Spread Strategy

### 3.1 Cash Flow Coordination

**Box Spread Execution Flow:**
1. **Opportunity Identified:** Box spread meets profitability criteria
2. **Cash Check:** Verify Tier 1 or Tier 2 cash availability
3. **Allocation:** Deploy from appropriate tier
4. **Tracking:** Monitor box spread until expiration
5. **Reinvestment:** Redeploy proceeds based on current allocation targets

**Cash Reserve Management:**
- Maintain minimum Tier 1 cash for:
  - Emergency reserves (2-3 months expenses)
  - Immediate trading opportunities
  - Margin requirements buffer
- Tier 2 cash available for box spreads with 30-90 day DTE
- Tier 3 cash can fund longer-dated box spreads if rates justify

### 3.2 Rate Comparison Framework

**Box Spread Evaluation Process:**
```
For each box spread opportunity:
  1. Calculate implied rate (annualized)
  2. Identify matching T-bill maturity
  3. Compare: Box Spread Rate vs. T-bill Rate + 0.5%
  4. Consider: Tax treatment (capital gains vs. ordinary income)
  5. Factor: Liquidity needs (can funds be locked for DTE period?)
  6. Decision: Execute if box spread > threshold, else use T-bill
```

**Rate Thresholds:**
- **Minimum Spread:** Box spread must beat T-bill + 0.5% (after commissions)
- **Tax-Adjusted:** Box spread gains often taxed as capital gains (lower rate)
  - Adjust threshold for tax efficiency (may accept smaller spread)
- **Liquidity Premium:** Willing to accept 0.25% lower yield for higher liquidity

---

## 4. Risk Management

### 4.1 Portfolio Risk Metrics

**Monitoring:**
- **Duration:** Weighted average duration of fixed income portfolio
- **Convexity:** Portfolio-level convexity metric
- **Skewness:** Return distribution skewness
- **VaR (Value at Risk):** 95% confidence, 1-day and 30-day
- **Maximum Drawdown:** Historical and forward-looking estimates

**Limits:**
- Maximum equity allocation: 60% (even in aggressive model)
- Maximum box spread allocation: 10% of portfolio
- Minimum Tier 1 cash: 5% of portfolio
- Maximum position concentration: 10% in single ETF

### 4.2 Box Spread Risk Management

**Existing Risk Controls (from codebase):**
- Minimum ROI: 0.5%
- Maximum position size: Per `StrategyParams::max_position_size`
- Minimum liquidity: Volume and open interest thresholds
- Maximum bid-ask spread: Per `StrategyParams::max_bid_ask_spread`

**Additional Portfolio-Level Controls:**
- Maximum box spread exposure: 10% of total portfolio
- Maximum per-symbol box spread: 25% of box spread allocation
- Diversification: Spread box spreads across multiple underlyings
- Maturity diversification: Spread DTE across 30-180 day range

---

## 5. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- [ ] Extend configuration system to support allocation targets
- [ ] Add portfolio allocation tracking module
- [ ] Integrate T-bill rate monitoring
- [ ] Build rate comparison framework

### Phase 2: Core Allocation Logic (Weeks 3-4)
- [ ] Implement tiered cash allocation system
- [ ] Build ETF allocation tracking
- [ ] Create rebalancing decision engine
- [ ] Integrate with existing box spread strategy

### Phase 3: Advanced Features (Weeks 5-6)
- [ ] Add convexity calculation and optimization
- [ ] Implement skewness monitoring
- [ ] Build dynamic rate targeting
- [ ] Create spare cash allocation logic

### Phase 4: Testing & Validation (Weeks 7-8)
- [ ] Backtest allocation strategy
- [ ] Paper trade integration
- [ ] Performance monitoring
- [ ] Documentation and training

---

## 6. Configuration Extensions

### Proposed Configuration Structure

```json
{
  "allocation": {
    "model": "moderate",
    "targets": {
      "cash_tier1": 0.07,
      "cash_tier2": 0.10,
      "cash_tier3": 0.08,
      "fixed_income_short": 0.12,
      "fixed_income_long": 0.18,
      "equity_core": 0.12,
      "equity_growth": 0.08,
      "equity_international": 0.08,
      "equity_other": 0.12,
      "box_spreads": 0.05
    },
    "rebalancing": {
      "threshold_percent": 0.05,
      "frequency_days": 30,
      "tax_aware": true
    }
  },
  "rate_targets": {
    "t_bill_monitoring": true,
    "box_spread_spread_bps": 50,
    "tax_adjusted": true,
    "liquidity_premium_bps": 25
  },
  "convexity": {
    "optimize": true,
    "barbell_short_pct": 0.40,
    "barbell_long_pct": 0.60
  }
}
```

---

## 7. Performance Monitoring

### Key Metrics to Track

**Allocation Metrics:**
- Actual vs. target allocation percentages
- Rebalancing frequency and costs
- Tax efficiency (capital gains vs. ordinary income)

**Return Metrics:**
- Total portfolio return
- Return by asset class
- Box spread return vs. T-bill return
- Risk-adjusted returns (Sharpe ratio, Sortino ratio)

**Risk Metrics:**
- Portfolio convexity
- Return skewness
- VaR and maximum drawdown
- Correlation between assets

**Operational Metrics:**
- Cash utilization rate
- Box spread execution rate (vs. opportunities identified)
- Rebalancing costs
- Tax impact

---

## 8. Assumptions & Limitations

### Assumptions
1. **Tax Treatment:** Box spread gains treated as capital gains (may vary by jurisdiction)
2. **Liquidity:** ETFs and T-bills maintain adequate liquidity
3. **Market Access:** Ability to execute box spreads when opportunities arise
4. **Rate Environment:** T-bill rates remain positive and box spreads remain available

### Limitations
1. **Execution Risk:** Box spreads require all 4 legs to execute simultaneously
2. **Liquidity Constraints:** May not always have sufficient cash for opportunities
3. **Tax Complexity:** Tax treatment of box spreads can be complex
4. **Market Regime:** Strategy assumes relatively normal market conditions
5. **Rebalancing Costs:** Transaction costs reduce returns from frequent rebalancing

### Mitigation Strategies
- Maintain adequate cash reserves
- Diversify across multiple underlyings
- Use tax-loss harvesting where possible
- Monitor execution quality
- Adjust strategy for market regime changes

---

## 9. References & Resources

### Academic & Research
- [arXiv: Mean-Variance-Skewness Portfolio Selection](https://arxiv.org/abs/2201.06233)
- [Mechanician: Bond Convexity Strategies](https://mechnician.wordpress.com/)

### Industry Resources
- [CBOE: Box Spreads as Alternative Borrowing/Lending](https://www.cboe.com/insights/posts/why-consider-box-spreads-as-an-alternative-borrowing-lending-strategy/)
- [Schwab: Bonds and Cash Strategies](https://www.schwab.com/learn/story/3-strategies-bonds-cash-wealth-management)
- [Picture Perfect Portfolios: BOXX ETF Review](https://pictureperfectportfolios.com/boxx-etf-review-alpha-architect-1-3-month-box-strategy/)

### ETF Resources
- **SPY:** SPDR S&P 500 ETF Trust
- **SHY:** iShares 1-3 Year Treasury Bond ETF
- **TLT:** iShares 20+ Year Treasury Bond ETF
- **IEF:** iShares 7-10 Year Treasury Bond ETF
- **BIL:** SPDR Bloomberg 1-3 Month T-Bill ETF

---

## 10. Next Steps

1. **Review & Approval:** Stakeholder review of strategy document
2. **Risk Assessment:** Detailed risk analysis and stress testing
3. **Configuration Design:** Finalize configuration structure
4. **Implementation Planning:** Detailed technical implementation plan
5. **Backtesting:** Historical performance simulation
6. **Paper Trading:** Live paper trading validation
7. **Production Deployment:** Gradual rollout with monitoring

---

**Document Status:** Draft - Awaiting Review
**Last Updated:** 2025-01-18
**Next Review:** After stakeholder feedback
