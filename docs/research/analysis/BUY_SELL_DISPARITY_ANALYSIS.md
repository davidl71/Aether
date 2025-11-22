# Buy vs Sell Box Spread Disparity Analysis

## Overview

This document explains the factors that cause disparities between buying and selling box spreads, and how the system tracks and visualizes these differences.

## Key Factors Causing Buy/Sell Disparity

### 1. **Bid-Ask Spread Width**

- **Impact**: Primary driver of buy/sell differences
- **Mechanism**: Different spreads on each leg create a gap between buying (ASK) and selling (BID)
- **Intraday Changes**: Spreads widen during low liquidity (pre-market, after-hours, lunch)
- **Visualization**: Track average bid-ask spread across all legs

### 2. **Put-Call Parity Violations**

- **Impact**: Different implied rates on call side vs put side
- **Causes**:
  - Dividend expectations (affects synthetic forward price)
  - Early exercise risk (American options)
  - Interest rate changes
- **Visualization**: Show put-call parity violation in basis points

### 3. **Liquidity Imbalance**

- **Impact**: More buyers vs sellers at different times
- **Times**:
  - Market open: High liquidity, narrow disparity
  - Lunch: Lower liquidity, wider disparity
  - Market close: Mixed, depends on expiration
- **Visualization**: Track liquidity score over time

### 4. **Market Maker Inventory**

- **Impact**: Market makers adjust prices based on their inventory
- **Mechanism**: If they're long, they may offer better BID (buy from them)
- **Visualization**: Track when one side (buy vs sell) becomes more favorable

### 5. **Early Exercise Risk (American Options)**

- **Impact**: Creates premium that differs for long vs short
- **Mechanism**: American options can be exercised early, affecting pricing
- **Visualization**: Show early exercise risk premium

### 6. **Dividend Expectations**

- **Impact**: Expected dividends affect synthetic forward price
- **Mechanism**: Dividends reduce call prices, increase put prices (put-call parity)
- **Visualization**: Show dividend-adjusted rates

### 7. **Interest Rate Changes (Intraday)**

- **Impact**: Changes in SOFR/T-bill rates during the day
- **Mechanism**: Box spreads are interest rate instruments
- **Visualization**: Compare box spread rates to benchmark over time

### 8. **Volatility Skew**

- **Impact**: Different implied volatility at different strikes
- **Mechanism**: Skew affects call vs put pricing differently
- **Visualization**: Show IV skew impact

### 9. **Order Flow Imbalance**

- **Impact**: Large institutional orders create temporary imbalances
- **Mechanism**: More buying pressure → ASK prices rise
- **Visualization**: Track order flow indicators

### 10. **Time Decay Asymmetry**

- **Impact**: Theta differs slightly between long and short positions
- **Mechanism**: Time decay affects buy vs sell profitability
- **Visualization**: Show theta impact

## System Implementation

### Data Structures

```cpp
struct BoxSpreadLeg {
    // Mid prices (traditional calculation)
    double net_debit;
    double arbitrage_profit;

    // Buy vs Sell (using bid/ask)
    double buy_net_debit;         // Cost to buy (ASK for long, BID for short)
    double buy_profit;
    double buy_implied_rate;

    double sell_net_credit;       // Credit from selling (BID for long, ASK for short)
    double sell_profit;
    double sell_implied_rate;

    double buy_sell_disparity;    // Difference in profitability
    double put_call_parity_violation;  // Put-call parity violation (bps)
};
```

### Calculations

**Buying Box Spread:**

- Long call: Use ASK (what we pay)
- Short call: Use BID (what we receive)
- Long put: Use ASK (what we pay)
- Short put: Use BID (what we receive)
- Net Debit = (Long Call ASK + Long Put ASK) - (Short Call BID + Short Put BID)

**Selling Box Spread:**

- Long call: Use BID (what we receive)
- Short call: Use ASK (what we pay)
- Long put: Use BID (what we receive)
- Short put: Use ASK (what we pay)
- Net Credit = (Long Call BID + Long Put BID) - (Short Call ASK + Short Put ASK)

**Disparity:**

- Buy-Sell Disparity = Buy Profit - Sell Profit
- Positive = Buying more profitable
- Negative = Selling more profitable

## Visualization in TUI/PWA

### Web UI Enhancements

1. **Side-by-Side Comparison Table**
   - Column: "Buy" vs "Sell" profitability
   - Show disparity in basis points
   - Color-code: Green = Buy better, Red = Sell better

2. **Disparity Chart**
   - Line chart showing buy vs sell profitability over time
   - Highlight when disparity widens/narrows

3. **Put-Call Parity Indicator**
   - Show violation amount in bps
   - Alert when violation exceeds threshold

4. **Time-of-Day Analysis**
   - Track average disparity by hour
   - Identify optimal times for buy vs sell

### TUI Enhancements

1. **Buy/Sell Columns**
   - Add columns showing buy profit and sell profit
   - Show disparity inline

2. **Disparity Tracking**
   - Track changes throughout the day
   - Show trend (widening/narrowing)

3. **Alert System**
   - Alert when buy/sell opportunity appears
   - Highlight best side (buy or sell)

## Use Cases

1. **Opportunity Detection**: Identify when buying vs selling is more favorable
2. **Market Making**: Understand when to provide liquidity on which side
3. **Arbitrage**: Detect when both sides are profitable (rare)
4. **Timing**: Know the best times of day for buy vs sell
5. **Risk Management**: Understand the spread you'll pay/receive

## Future Enhancements

- Historical analysis of disparity patterns
- Machine learning to predict disparity changes
- Integration with order flow data
- Real-time alerts for favorable buy/sell opportunities
- Comparison to market maker quotes
