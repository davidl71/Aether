# Investment Strategy Requirements & Assumptions

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Requirements Document

## Purpose

This document captures user requirements, investment goals, risk tolerance, and assumptions that guide the investment strategy implementation. These requirements should be reviewed and updated as investment goals evolve.

## User Requirements

### Investment Goals

**Primary Objectives:**
- [ ] **To be defined by user:** Primary investment goal (capital appreciation, income generation, capital preservation, etc.)
- [ ] **To be defined by user:** Target return (e.g., T-bill rate + 100 bps, specific percentage)
- [ ] **To be defined by user:** Time horizon (short-term <1 year, medium-term 1-5 years, long-term >5 years)

**Secondary Objectives:**
- [ ] **To be defined by user:** Additional goals (tax efficiency, liquidity needs, etc.)

### Risk Tolerance

**Risk Profile:**
- [ ] **To be defined by user:** Risk tolerance level
  - Conservative: Low volatility, capital preservation
  - Moderate: Balanced risk-return
  - Aggressive: Higher risk for higher returns

**Risk Constraints:**
- [ ] **To be defined by user:** Maximum acceptable drawdown (e.g., 10%, 20%)
- [ ] **To be defined by user:** Volatility tolerance (e.g., standard deviation limits)
- [ ] **To be defined by user:** Minimum cash buffer requirements

### Cash Management Preferences

**Liquidity Needs:**
- [ ] **To be defined by user:** Immediate cash requirements (emergency fund size, monthly expenses)
- [ ] **To be defined by user:** Frequency of cash access needs (daily, weekly, monthly)
- [ ] **To be defined by user:** Maximum acceptable cash drag (idle cash percentage)

**Spare Cash Allocation:**
- [ ] **To be defined by user:** Preference for box spreads vs. T-bills vs. bonds
- [ ] **To be defined by user:** Minimum rate spread to prefer box spreads (e.g., 20 bps)
- [ ] **To be defined by user:** Liquidity vs. yield tradeoff preference

### ETF Investment Preferences

**Equity ETFs:**
- [ ] **To be defined by user:** Preferred equity ETFs (e.g., SPY, VOO, QQQ)
- [ ] **To be defined by user:** International allocation preference (percentage)
- [ ] **To be defined by user:** Sector/style tilts desired

**Bond ETFs:**
- [ ] **To be defined by user:** Preferred bond ETFs (e.g., SHY, TLT)
- [ ] **To be defined by user:** Duration preference (short-term, long-term, or barbell)
- [ ] **To be defined by user:** Credit quality preference (Treasury, corporate, municipal)

### T-Bill/Bond Target Rate Preferences

**Target Rate:**
- [ ] **To be defined by user:** Minimum acceptable T-bill rate (e.g., 4.5%, 5.0%)
- [ ] **To be defined by user:** Target rate for spare cash (e.g., T-bill rate + 50 bps)
- [ ] **To be defined by user:** Maximum duration for T-bills/bonds (e.g., 3 months, 6 months)

**Ladder Strategy:**
- [ ] **To be defined by user:** Number of maturity buckets (e.g., 3, 4, 6)
- [ ] **To be defined by user:** Staggering frequency (e.g., monthly, quarterly)
- [ ] **To be defined by user:** Rolling strategy preference (reinvest vs. reallocate)

### Existing Loan Liabilities

**Variable Rate SHIR-Based Loans (Israel):**
- [x] **User has:** Variable rate loans with interest = SHIR (Shekel Interbank Rate) + spread/addition
- [ ] **To be defined by user:** Total outstanding principal amount
- [ ] **To be defined by user:** Current monthly payment amount
- [ ] **To be defined by user:** Current SHIR rate + spread (effective interest rate)
- [ ] **To be defined by user:** Loan maturity/term remaining
- [ ] **To be defined by user:** Currency (likely ILS)

**Fixed Rate CPI-Linked Loans (Israel):**
- [x] **User has:** Fixed rate CPI-linked loans (principal adjusts with CPI)
- [ ] **To be defined by user:** Total outstanding principal amount
- [ ] **To be defined by user:** Current monthly payment amount
- [ ] **To be defined by user:** Fixed interest rate
- [ ] **To be defined by user:** Loan maturity/term remaining
- [ ] **To be defined by user:** Currency (likely ILS)
- [ ] **To be defined by user:** CPI index used (e.g., Israeli CPI)

**Loan Payment Considerations:**
- [ ] **To be defined by user:** Total monthly loan payment obligation
- [ ] **To be defined by user:** Preferred currency for loan payments (ILS vs. USD conversion)
- [ ] **To be defined by user:** Loan payment frequency and dates
- [ ] **To be defined by user:** Desired cash reserve for loan payments (months)

### Convexity & Skew Preferences

**Convexity:**
- [ ] **To be defined by user:** Target portfolio convexity value
- [ ] **To be defined by user:** Interest rate environment expectations (rising, falling, stable)
- [ ] **To be defined by user:** Barbell vs. bullet preference

**Skewness:**
- [ ] **To be defined by user:** Skewness preference (positive, neutral, negative tolerance)
- [ ] **To be defined by user:** Volatility skew monitoring requirements
- [ ] **To be defined by user:** Options overlay preference for skew management

## Assumptions

### Market Assumptions

**Interest Rates:**
- Current T-bill rates are available and reliable
- Short-term rates remain above 4% (adjust if environment changes)
- Box spread rates track T-bill rates within 0-50 bps spread
- IBKR pays interest on positive settled cash balances ([IBKR Interest Rates](https://www.interactivebrokers.com/en/accounts/fees/pricing-interest-rates.php))
- IBKR interest rates should be compared against T-bills and box spreads
- For large accounts (10M+ USD), FX Swap Program may offer competitive rates (2-10 bps spreads)
- **SHIR Rate:** Israeli Shekel Interbank Rate (SHIR) is available and can be monitored; rate changes affect variable loan payments
- **Israeli CPI:** Israeli Consumer Price Index is available for tracking CPI-linked loan principal adjustments

**Box Spread Market:**
- Sufficient liquidity exists for 30-90 DTE box spreads on SPX/XSP/NDX
- Box spread execution probability remains high (confidence score >70)
- Tax treatment: 60/40 long-term/short-term capital gains if held >1 year

**ETF Markets:**
- ETFs maintain tight bid-ask spreads (<0.10%)
- ETF liquidity sufficient for rebalancing without significant slippage
- ETF expense ratios: <0.10% for equity, <0.15% for bond

### Strategy Assumptions

**Allocation Assumptions:**
- Default allocation: 70-80% core investments, 10-15% cash, 10-15% T-bill ladder
- Barbell strategy provides adequate convexity enhancement
- Positive skew from equity ETFs enhances risk-return profile

**Rebalancing Assumptions:**
- Transaction costs: ~$1 per ETF trade, ~$2-5 per box spread (IBKR)
- Tax-efficient rebalancing preferred (avoid short-term gains)
- Rebalancing threshold: 5% deviation from target allocation

**Performance Assumptions:**
- Box spreads can achieve T-bill rate + 0-50 bps consistently
- ETF tracking error: <0.10% annual
- Portfolio yield target: T-bill rate + 50-100 bps

### Technical Assumptions

**IBKR Integration:**
- IBKR API provides reliable market data for:
  - ETF quotes and volume
  - T-bill rates and availability
  - Box spread option chains
  - Account summary including cash balances and accrued interest (`AccruedCash` tag)
  - Margin requirements (`InitMarginReq`, `MaintMarginReq`, `AvailableFunds`, `ExcessLiquidity`, `Cushion`)
- IBKR interest rates on cash balances are available via account summary or web portal
- IBKR margin rates are available via web portal ([IBKR Margin Rates](https://www.interactivebrokers.com/en/trading/margin-rates.php))
- Order execution is reliable for all asset classes
- Account minimums and margin requirements are met
- Interest is paid on positive settled cash balances (rates published on IBKR website)
- Margin interest is charged on borrowed funds; rates vary by account size and currency

**Codebase Integration:**
- Existing box spread strategy can be extended for spare cash allocation
- Risk calculator provides adequate position sizing
- Config manager can be extended with portfolio allocation parameters

### Risk Assumptions

**Liquidity Risk:**
- Box spreads can be closed prior to expiration if needed (with slippage)
- ETFs maintain liquidity during normal and stressed market conditions
- T-bills are highly liquid (bid-ask spread <1 bp)

**Interest Rate Risk:**
- Long-term bonds may experience price volatility with rate changes
- Barbell strategy mitigates but doesn't eliminate rate risk
- Convexity provides partial protection against rate volatility

**Margin Risk:**
- Box spreads may require margin/collateral depending on position structure
- Margin interest costs reduce net returns if margin is used
- Margin calls risk if excess liquidity drops too low
- Monitor margin requirements and maintain excess liquidity > 25% of net liquidation
- Fully collateralized box spreads preferred to avoid margin costs

**Loan Liability Risk:**
- **SHIR Rate Risk:** Variable SHIR-based loans expose portfolio to Israeli interest rate changes; rising SHIR increases loan payments and reduces available capital
- **Currency Risk:** If loans are in ILS and portfolio is USD-denominated, exchange rate changes affect loan payment costs
- **Cash Flow Risk:** Loan payments reduce available cash flow for investment opportunities; maintain adequate reserves
- **CPI Risk (for CPI-linked loans):** While CPI-linked loans provide inflation hedge, principal increases can affect cash flow
- **Interest Rate Correlation:** Monitor correlation between SHIR changes and portfolio interest rate exposure (e.g., bond holdings)

**Execution Risk:**
- Box spread fills may differ from theoretical values (use confidence scores)
- ETF rebalancing can be executed within reasonable slippage limits
- T-bill orders execute at market rates

## Constraints

### Account Constraints

**Capital Constraints:**
- [ ] **To be defined by user:** Total account value
- [ ] **To be defined by user:** Minimum position sizes
- [ ] **To be defined by user:** Maximum position sizes

**Regulatory Constraints:**
- Pattern day trader rules (if applicable)
- Margin requirements for box spreads
- Tax lot accounting preferences

### Operational Constraints

**Trading Constraints:**
- [ ] **To be defined by user:** Maximum number of trades per day/week
- [ ] **To be defined by user:** Trading hours preferences (avoid pre-market/after-hours)
- [ ] **To be defined by user:** Manual approval required for large trades (threshold)

**Monitoring Constraints:**
- [ ] **To be defined by user:** Review frequency (daily, weekly, monthly)
- [ ] **To be defined by user:** Alert thresholds for deviations
- [ ] **To be defined by user:** Reporting preferences

### Tax Constraints

**Tax Preferences:**
- [ ] **To be defined by user:** Tax status (individual, IRA, 401k, trust, etc.)
- [ ] **To be defined by user:** Tax-loss harvesting preferences
- [ ] **To be defined by user:** Minimize short-term capital gains

**Tax Implications:**
- Box spreads: 60/40 treatment if held >1 year (subject to IRS approval)
- T-bills: State tax exempt, federal taxable
- ETFs: Distributions and capital gains may be taxable
- Rebalancing: May trigger capital gains

## Open Questions for User

### Strategy Parameters

1. **What is your primary investment goal?**
   - Capital appreciation
   - Income generation
   - Capital preservation
   - Tax efficiency
   - Combination (specify)

2. **What is your risk tolerance level?**
   - Conservative
   - Moderate
   - Aggressive
   - Custom (specify)

3. **What is your investment time horizon?**
   - <1 year (short-term)
   - 1-5 years (medium-term)
   - >5 years (long-term)
   - Perpetual (no specific end date)

### Allocation Preferences

4. **What is your preferred allocation between equity and bonds?**
   - Aggressive (70% equity, 30% bonds)
   - Balanced (60% equity, 40% bonds)
   - Conservative (50% equity, 50% bonds)
   - Custom (specify percentages)

5. **What percentage of portfolio should be in cash equivalents?**
   - Minimal (5-10%)
   - Moderate (10-15%)
   - High (15-20%)
   - Custom (specify)

6. **How much spare cash should be allocated to box spreads?**
   - Minimal (20-30% of spare cash)
   - Moderate (50-60% of spare cash)
   - Maximum (70-80% of spare cash)

### Rate Targets

7. **What is your minimum acceptable T-bill rate?**
   - Current market rate
   - T-bill rate - 50 bps
   - T-bill rate - 100 bps
   - Custom (specify percentage)

8. **What rate spread is required to prefer box spreads over T-bills?**
   - Box spread rate ≥ T-bill rate (within 0 bps)
   - Box spread rate ≥ T-bill rate - 20 bps
   - Box spread rate ≥ T-bill rate - 50 bps
   - Always prefer T-bills

### Operational Preferences

9. **How frequently should the strategy rebalance?**
   - Monthly (tactical)
   - Quarterly (strategic)
   - When deviation exceeds threshold
   - Manual only

10. **What level of automation is desired?**
    - Fully automated (system executes all trades)
    - Semi-automated (alerts with approval)
    - Manual (system recommends only)

## Default Values (To Be Confirmed)

If user preferences are not specified, the following defaults will be used:

### Allocation Defaults
- Equity ETFs: 45%
- Bond ETFs: 35% (17.5% short-term, 17.5% long-term)
- Immediate Cash: 4%
- Spare Cash: 8%
- T-Bill Ladder: 8%

### Strategy Defaults
- Risk Tolerance: Moderate
- Time Horizon: Long-term (>5 years)
- Minimum T-Bill Rate: Current market rate
- Box Spread Rate Spread: 20 bps vs. T-bills
- Rebalancing Frequency: Quarterly strategic, monthly tactical
- Rebalancing Threshold: 5% deviation

### Rate Targets
- Target Portfolio Yield: T-bill rate + 75 bps
- Box Spread Target: T-bill rate + 0-50 bps
- Minimum Box Spread ROI: 0.5% (existing strategy parameter)

## Validation Checklist

Before implementation, verify:

- [ ] Investment goals are clearly defined
- [ ] Risk tolerance is specified
- [ ] Cash management needs are understood
- [ ] ETF preferences are documented
- [ ] T-bill/bond target rates are set
- [ ] Convexity/skew preferences are clear
- [ ] Account constraints are known
- [ ] Tax status is documented
- [ ] Operational preferences are specified
- [ ] All open questions are answered

## Change Management

**Requirements Review:**
- Review quarterly or when investment goals change
- Update assumptions when market conditions change significantly
- Adjust targets when strategy performance deviates persistently

**Version History:**
- **v1.0.0 (2025-11-18):** Initial requirements document
  - Framework structure created
  - Open questions identified
  - Default values specified
  - Assumptions documented

---

**Next Steps:**
1. User completes requirements questionnaire
2. Validate requirements against strategy framework
3. Update defaults based on user preferences
4. Begin implementation with confirmed requirements
