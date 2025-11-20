# SyntheticFi Box Spread Lending/Borrowing Analysis

**Date**: 2025-01-27
**Source**: https://app.syntheticfi.com/cob, https://www.syntheticfi.com/
**Purpose**: Analyze SyntheticFi's approach to box spread-based lending/borrowing for implementation guidance

---

## Overview

[SyntheticFi](https://www.syntheticfi.com/) is a Y Combinator-backed fintech company that uses box spreads to provide securities-backed lending at rates 1-3% lower than traditional lenders. They integrate with existing brokerage accounts to offer loans without selling investments or incurring capital gains taxes.

## SyntheticFi's Business Model

### Core Value Proposition

1. **Low-Cost Borrowing**: Interest rates typically 1-3% below traditional lenders
2. **Tax Efficiency**: Interest expenses are fully tax-deductible regardless of loan purpose
3. **No Paperwork**: Seamless integration with existing brokerage accounts
4. **No Credit Checks**: Leverages securities as collateral via box spreads
5. **Capital Preservation**: Access liquidity without selling investments

### How They Use Box Spreads

**Box Spread Construction**:
- SyntheticFi constructs box spreads using SPX (S&P 500 Index) options
- Four-leg structure: Long Call (K1), Short Call (K2), Long Put (K2), Short Put (K1)
- The strike width (K2 - K1) represents the loan principal
- The net debit/credit represents the implied interest rate

**Lending Mechanism**:
1. Client wants to borrow funds (e.g., $100,000)
2. SyntheticFi constructs a box spread with $100 strike width
3. Net credit received from the spread = loan proceeds
4. At expiration, box spread pays out strike width ($100) automatically
5. Difference between credit received and $100 strike width = implied interest rate

**Rate Calculation**:
- Implied Annual Interest Rate = ((Strike Width - Net Credit) / Net Credit) × (365 / Days to Expiry)
- This rate is typically competitive with or better than T-bills, repo rates, or margin loans

### Technical Implementation Details

#### Box Spread Selection Criteria

Based on industry best practices (including CBOE and OCC guidance):

1. **Liquidity Requirements**:
   - High volume and open interest on all four legs
   - Tight bid-ask spreads (preferably < $0.50 per leg)
   - SPX options are preferred due to European-style exercise and cash settlement

2. **Expiration Selection**:
   - Typically 30-90 days to expiration for lending
   - Longer expirations for longer-term borrowing needs
   - Balance between rate competitiveness and time flexibility

3. **Strike Width**:
   - Determined by loan amount needed
   - Multiplied by SPX contract multiplier (100)
   - Example: $100,000 loan = $100 strike width × 10 contracts × 100 multiplier

4. **Rate Optimization**:
   - Compare implied rates across different expirations
   - Select expirations with best implied rates relative to benchmarks (T-bills, SOFR)
   - Monitor for rate improvements intraday

#### Risk Management

1. **Counterparty Risk**: Eliminated via OCC clearing
2. **Early Assignment Risk**: SPX options are European-style, no early exercise
3. **Liquidity Risk**: Managed through careful leg selection and monitoring
4. **Rate Risk**: Locked in at execution, but positions can be rolled if rates improve

#### Position Management

**Intraday Monitoring**:
- Monitor bid/ask spreads for all legs
- Watch for rate improvements (better implied rates on different expirations)
- Identify opportunities to improve position economics

**Position Improvement Strategies**:

1. **Leg Rolling**:
   - If rates improve on a different expiration, roll the entire box spread
   - Close existing position, open new position at better rate
   - Net benefit must exceed transaction costs

2. **Early Close**:
   - If rates move significantly in your favor, close position early
   - Realize the improved economics immediately
   - Reinvest at current market rates

3. **Partial Adjustment**:
   - If only some legs become more favorable, consider adjusting those legs
   - Maintain box spread structure while improving economics

## Implementation Guidance for This Project

### Primary Goal: Synthetic Lending and Borrowing

#### 1. Implied Interest Rate Calculation

**Formula**:
```cpp
// For lending (receiving net credit):
// implied_rate = ((strike_width - net_credit) / net_credit) * (365 / days_to_expiry) * 100

// For borrowing (paying net debit):
// implied_rate = ((net_debit - strike_width) / strike_width) * (365 / days_to_expiry) * 100
```

**Implementation**:
- Add `calculate_implied_interest_rate()` to `BoxSpreadCalculator`
- Calculate annualized rate in percentage terms
- Compare against benchmarks (T-bills, SOFR, traditional margin rates)

#### 2. Rate Comparison and Benchmarking

**Benchmarks to Compare Against**:
- **T-Bills**: Risk-free rate benchmark
- **SOFR**: Secured Overnight Financing Rate
- **Traditional Margin Loans**: Typical rates (e.g., 6-8% APR)
- **Repo Rates**: Overnight repurchase agreement rates

**Implementation**:
- Add `compare_to_benchmark()` function
- Return spread over benchmark in basis points
- Flag opportunities where box spread rate beats benchmark by threshold (e.g., 50-100 bps)

#### 3. Box Spread Selection for Lending/Borrowing

**Criteria Shift** (from arbitrage to financing):

**Current (Arbitrage Focus)**:
- Seek arbitrage profit (theoretical value > net debit)
- Maximize ROI from mispricing

**New (Lending/Borrowing Focus)**:
- Seek competitive implied interest rates
- Match or beat benchmark rates (T-bills, margin loans)
- Optimize for loan amount and duration requirements
- Minimize transaction costs and bid-ask spreads

**Implementation**:
- Add `find_lending_opportunities()` method to `BoxSpreadStrategy`
- Filter by implied rate competitiveness vs benchmarks
- Rank by effective rate (after transaction costs)
- Consider loan amount and duration requirements

### Secondary Goal: Intraday Position Improvement

#### 1. Position Monitoring

**Metrics to Track**:
- Current implied rate vs entry implied rate
- Current bid/ask spreads vs entry spreads
- Unrealized P&L (mark-to-market)
- Available rate improvements on alternative expirations
- Days remaining to expiration

**Implementation**:
- Enhance `monitor_positions()` in `BoxSpreadStrategy`
- Add real-time rate calculation for active positions
- Compare current rates to entry rates
- Flag positions with improvement opportunities

#### 2. Position Improvement Logic

**Scenarios for Improvement**:

**Scenario 1: Rate Improvement Available**
- New expiration offers better implied rate
- Calculate net benefit after transaction costs
- If beneficial, execute roll (close + open new)

**Scenario 2: Early Close Opportunity**
- Market moves have improved economics significantly
- Early close + reinvestment at current rates is better than holding to expiry
- Calculate break-even rate for early close decision

**Scenario 3: Partial Leg Improvement**
- One or two legs have improved pricing
- Evaluate cost/benefit of partial leg adjustments
- Ensure box spread structure remains intact

**Implementation**:
- Add `evaluate_position_improvement()` method
- Add `roll_box_spread()` method for position rolling
- Add `calculate_early_close_value()` method
- Add decision logic to trigger improvements when thresholds are met

#### 3. Intraday Monitoring Loop

**Flow**:
```
1. Monitor active box spread positions every N seconds (e.g., 30-60 seconds)
2. For each position:
   a. Calculate current implied rate
   b. Compare to entry rate and benchmarks
   c. Evaluate improvement opportunities
   d. If improvement exceeds threshold:
      - Calculate net benefit after costs
      - Execute improvement action (roll/close/adjust)
3. Log all decisions and actions
```

**Implementation**:
- Enhance `monitor_positions()` to run on configurable interval
- Add improvement evaluation for each position
- Add action execution when improvements are identified
- Add comprehensive logging for audit trail

## Comparison: SyntheticFi vs. Our Implementation

### Similarities

1. **Box Spread Structure**: Both use four-leg SPX box spreads
2. **OCC Clearing**: Both leverage OCC clearing for counterparty risk elimination
3. **Rate Focus**: Both calculate and optimize implied interest rates
4. **Integration**: Both integrate with brokerage accounts (SyntheticFi as service, ours as tool)

### Differences

| Aspect | SyntheticFi | Our Implementation |
|--------|-------------|-------------------|
| **Service Model** | Managed service (they execute) | Self-directed tool (you execute) |
| **Loan Structure** | Pre-packaged loan products | Flexible position management |
| **Rate Optimization** | Their algorithm finds best rates | Your algorithm finds best rates |
| **Position Management** | Fully automated | Configurable automation |
| **Intraday Improvements** | Handled by their system | Your control and automation |
| **Transparency** | Black-box service | Full visibility and control |

### Key Advantages of Self-Directed Approach

1. **Control**: Full control over position management and improvement decisions
2. **Transparency**: See exactly what rates you're getting and why
3. **Flexibility**: Adjust strategy based on your specific needs and risk tolerance
4. **Cost**: Potentially lower costs (no service fees, though you pay transaction costs)
5. **Learning**: Understand the mechanics and market dynamics yourself

## Recommended Implementation Priorities

### Phase 1: Core Lending/Borrowing Functionality (Primary Goal)

1. ✅ **Add Implied Interest Rate Calculation**
   - Calculate annualized rate from box spread pricing
   - Handle both lending (credit) and borrowing (debit) scenarios

2. ✅ **Add Benchmark Comparison**
   - Compare implied rates to T-bills, SOFR, margin loan rates
   - Flag opportunities where box spread beats benchmarks

3. ✅ **Add Lending Opportunity Scanner**
   - Scan for box spreads with competitive implied rates
   - Filter and rank by effective rate (after costs)

### Phase 2: Intraday Position Improvement (Secondary Goal)

4. ✅ **Enhanced Position Monitoring**
   - Real-time implied rate tracking
   - Mark-to-market P&L calculation
   - Improvement opportunity detection

5. ✅ **Position Improvement Actions**
   - Roll positions when rates improve
   - Early close when economically beneficial
   - Partial leg adjustments (if applicable)

6. ✅ **Automated Improvement Loop**
   - Configurable monitoring interval
   - Threshold-based action triggers
   - Comprehensive logging and audit trail

## Additional Resources

- **CBOE Box Spread Article**: https://www.cboe.com/insights/posts/why-consider-box-spreads-as-an-alternative-borrowing-lending-strategy/
- **OCC Box Spread Guide**: https://www.optionseducation.org/getmedia/2ae6c8bd-9a8e-4d2f-8168-19b6ff9e3589/listed-options-box-spread-strategies-for-borrowing-or-lending-cash.pdf
- **SyntheticFi Website**: https://www.syntheticfi.com/
- **Y Combinator Profile**: https://www.ycombinator.com/companies/syntheticfi

---

## Next Steps

1. Review this analysis and confirm alignment with goals
2. Implement Phase 1 features (implied rate calculation, benchmarking)
3. Test with paper trading accounts
4. Implement Phase 2 features (intraday improvements)
5. Deploy and monitor in production
