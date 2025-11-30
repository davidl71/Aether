# Algorithms and Expected Behavior

This document provides comprehensive documentation of the algorithms, mathematical formulas, and expected behavior used throughout the codebase. This helps Cursor AI understand the design intent and generate better code and unit tests.

## Table of Contents

1. [Box Spread Arbitrage Algorithm](#box-spread-arbitrage-algorithm)
2. [Mathematical Formulas](#mathematical-formulas)
3. [Expected Behavior Patterns](#expected-behavior-patterns)
4. [Algorithm Implementation Details](#algorithm-implementation-details)
5. [Test Design Patterns](#test-design-patterns)

---

## Box Spread Arbitrage Algorithm

### Overview

A box spread is a four-leg options strategy that creates a risk-free arbitrage opportunity when the net debit is less than the strike width. The strategy consists of:

1. **Long Call** at lower strike (K1)
2. **Short Call** at higher strike (K2)
3. **Long Put** at higher strike (K2)
4. **Short Put** at lower strike (K1)

### Algorithm Steps

```
1. Scan option chain for all available expirations
2. For each expiration:
   a. Generate all strike pairs (K1, K2) where K1 < K2
   b. For each strike pair:
      - Verify all 4 legs exist in the option chain
      - Check liquidity requirements (volume, open interest, bid/ask spread)
      - Calculate net debit (cost to enter the spread)
      - Calculate theoretical value (should equal K2 - K1)
      - Calculate arbitrage profit (theoretical_value - net_debit)
      - Calculate ROI ((profit / net_debit) * 100)
      - Validate against minimum thresholds
      - If profitable, add to opportunities list
3. Sort opportunities by profitability
4. Return top opportunities
```

### Key Algorithm Properties

- **Risk-Free**: Box spreads have limited risk - maximum loss is the net debit
- **Theoretical Value**: Always equals strike width (K2 - K1) at expiration
- **Arbitrage Condition**: Profit exists when `net_debit < theoretical_value`
- **Time Decay**: Minimal impact due to offsetting positions
- **Early Assignment**: Risk exists but is manageable with proper monitoring

---

## Mathematical Formulas

### Core Calculations

#### 1. Net Debit Calculation

**Formula:**

```
net_debit = long_call_price - short_call_price + long_put_price - short_put_price
```

**Explanation:**

- We pay for long positions (long_call, long_put)
- We receive for short positions (short_call, short_put)
- Net debit is the total cost to enter the spread

**Expected Behavior:**

- Positive value: We pay to enter (net debit)
- Negative value: We receive to enter (net credit)
- Zero: Costless entry (rare)

#### 2. Theoretical Value

**Formula:**

```
theoretical_value = strike_width = K2 - K1
```

**Explanation:**

- At expiration, regardless of underlying price, the box spread is worth exactly the strike width
- This is because:
  - If S > K2: Long call worth (S - K1), short call worth -(S - K2), long put worth 0, short put worth 0
  - Net: (S - K1) - (S - K2) = K2 - K1
  - Similar logic applies for other price ranges

**Expected Behavior:**

- Always equals strike width
- Independent of underlying price at expiration
- Used as the benchmark for arbitrage detection

#### 3. Arbitrage Profit

**Formula:**

```
arbitrage_profit = theoretical_value - net_debit
```

**Explanation:**

- Profit is the difference between what we receive at expiration (theoretical value) and what we pay to enter (net debit)
- Positive profit indicates an arbitrage opportunity

**Expected Behavior:**

- Must be positive for arbitrage opportunity
- Should exceed minimum profit threshold (configurable)
- Should account for transaction costs (commissions)

#### 4. ROI Calculation

**Formula:**

```
roi_percent = (arbitrage_profit / net_debit) * 100.0
```

**Explanation:**

- Return on investment as a percentage
- Measures profitability relative to capital deployed

**Expected Behavior:**

- Positive when arbitrage_profit > 0
- Should exceed minimum ROI threshold (configurable)
- Higher ROI indicates better opportunity

#### 5. Implied Interest Rate (Lending/Borrowing)

**Formula for Lending (net credit > 0):**

```
implied_rate = ((strike_width - net_credit) / net_credit) * (365 / days_to_expiry) * 100
```

**Formula for Borrowing (net debit > 0):**

```
implied_rate = ((net_debit - strike_width) / strike_width) * (365 / days_to_expiry) * 100
```

**Explanation:**

- Box spreads can be used as synthetic lending/borrowing instruments
- The implied rate represents the annualized interest rate
- For lending: We receive net credit, and at expiration we pay strike_width
- For borrowing: We pay net debit, and at expiration we receive strike_width

**Expected Behavior:**

- Positive rate for lending opportunities
- Negative rate for borrowing opportunities
- Annualized to 365 days
- Should be compared to benchmark rates (e.g., SOFR, Fed Funds)

#### 6. Effective Interest Rate (After Transaction Costs)

**Formula:**

```
effective_rate = implied_rate - (total_commission / net_debit) * (365 / days_to_expiry) * 100
```

**Explanation:**

- Accounts for transaction costs (commissions)
- More accurate representation of actual return
- Should be used for comparison with benchmark rates

**Expected Behavior:**

- Lower than implied rate (due to costs)
- Should still exceed benchmark rate for actionable opportunity
- Minimum spread over benchmark should be configurable (e.g., 50 bps)

---

## Expected Behavior Patterns

### 1. Opportunity Detection

**Input:** Option chain data, configuration parameters

**Expected Behavior:**

- Scans all expirations within DTE range (min_dte to max_dte)
- Generates all valid strike pairs
- Filters by liquidity requirements:
  - Minimum volume threshold
  - Maximum bid/ask spread
  - Minimum open interest

- Calculates profitability metrics
- Validates against risk limits
- Returns sorted list of opportunities

**Edge Cases:**

- Missing legs: Skip strike pair
- Stale market data: Use last known prices with warning
- Zero volume: Skip if liquidity required
- Invalid strikes: Skip and log warning

### 2. Order Execution

**Input:** Box spread opportunity, order manager

**Expected Behavior:**

- Validates opportunity is still valid (prices haven't changed)
- Checks risk limits (position size, exposure)
- Places all 4 orders atomically (or as close as possible)
- Monitors order status
- If any leg fails: Cancel remaining orders (rollback)
- If all legs succeed: Track multi-leg order
- Returns execution result with order IDs

**Edge Cases:**

- Partial fill: Monitor and potentially cancel remaining
- Order rejection: Rollback all orders, return error
- Price movement: Re-validate opportunity before execution
- Connection loss: Retry with exponential backoff

### 3. Position Monitoring

**Input:** Active positions, current market data

**Expected Behavior:**

- Track P&L for each position
- Monitor time decay (theta)
- Check for early assignment risk
- Evaluate improvement opportunities:
  - Rolling to better rate
  - Early close if beneficial

- Close positions near expiry
- Close positions if profit target reached
- Close positions if stop-loss triggered

**Edge Cases:**

- Missing market data: Use last known prices
- Early assignment: Handle assignment and close remaining legs
- Expiration approaching: Close position to avoid assignment risk
- Market closure: Resume monitoring when market reopens

### 4. Risk Validation

**Input:** Position, existing positions, account info

**Expected Behavior:**

- Check position size against limits
- Check total exposure against limits
- Validate Greeks (delta, gamma, theta, vega)
- Check concentration risk
- Check liquidity risk
- Return validation result with error messages

**Edge Cases:**

- Zero account value: Reject all positions
- Negative account value: Reject all positions
- Exceeds limits: Return detailed error message
- Missing data: Use conservative estimates

---

## Algorithm Implementation Details

### Box Spread Detection Algorithm

**Location:** `native/src/box_spread_strategy.cpp::find_box_spreads_in_chain()`

**Algorithm:**

```cpp
// Pseudo-code
for each expiry in option_chain:
  for each strike_pair (K1, K2) where K1 < K2:
    long_call = get_option(expiry, K1, Call, Long)
    short_call = get_option(expiry, K2, Call, Short)
    long_put = get_option(expiry, K2, Put, Long)
    short_put = get_option(expiry, K1, Put, Short)

    if all_legs_exist and all_legs_have_market_data:
      if liquidity_check_passes:
        calculate_metrics()
        if is_profitable():
          add_to_opportunities()

sort_opportunities_by_profit()
return opportunities
```

**Key Functions:**

- `get_option()`: Retrieves option contract from chain
- `all_legs_exist()`: Verifies all 4 legs are available
- `liquidity_check_passes()`: Validates volume, open interest, spreads
- `calculate_metrics()`: Computes net debit, profit, ROI
- `is_profitable()`: Checks against minimum thresholds

### Profitability Calculation Algorithm

**Location:** `native/src/box_spread_strategy.cpp::is_profitable()`

**Algorithm:**

```cpp
// Pseudo-code
profit = calculate_arbitrage_profit(spread)
roi = calculate_roi(spread)

return profit >= min_arbitrage_profit AND roi >= min_roi_percent
```

**Key Functions:**

- `calculate_arbitrage_profit()`: theoretical_value - net_debit
- `calculate_roi()`: (profit / net_debit) * 100.0

### Implied Rate Calculation Algorithm

**Location:** `native/src/box_spread_strategy.cpp::calculate_implied_interest_rate()`

**Algorithm:**

```cpp
// Pseudo-code
strike_width = K2 - K1
days_to_expiry = calculate_dte(expiry_date)

if days_to_expiry <= 0:
  return 0.0

net_cost = net_debit

if net_cost > 0:  // Borrowing
  implied_rate = ((net_cost - strike_width) / strike_width) * (365 / days_to_expiry) * 100
else:  // Lending
  implied_rate = ((strike_width - abs(net_cost)) / abs(net_cost)) * (365 / days_to_expiry) * 100

return implied_rate
```

**Key Functions:**

- `calculate_dte()`: Days to expiry from current date
- `get_strike_width()`: K2 - K1

---

## Test Design Patterns

### 1. Given-When-Then Pattern

**Structure:**

```cpp
TEST_CASE("Description", "[category]") {
  // Given: Set up test data and initial state
  BoxSpreadLeg spread = create_test_spread(/* parameters */);

  // When: Execute the function under test
  double profit = calculate_arbitrage_profit(spread);

  // Then: Verify expected results
  REQUIRE(profit == expected_profit);
  REQUIRE(profit > 0.0);
}
```

**Example:**

```cpp
TEST_CASE("Box spread profit calculation with valid inputs", "[strategy][profit]") {
  // Given: A box spread with strike width $10 and net debit $9.50
  BoxSpreadLeg spread;
  spread.long_call.strike = 500.0;
  spread.short_call.strike = 510.0;
  spread.long_put.strike = 510.0;
  spread.short_put.strike = 500.0;
  spread.net_debit = 9.50;
  spread.theoretical_value = 10.0;  // strike_width

  // When: We calculate the profit
  double profit = BoxSpreadCalculator::calculate_max_profit(spread);

  // Then: Profit should be $0.50 (theoretical_value - net_debit)
  REQUIRE_THAT(profit, Catch::Matchers::WithinRel(0.50, 0.01));
  REQUIRE(profit > 0.0);  // Must be profitable
}
```

### 2. Boundary Value Testing

**Test edge cases:**

- Zero net debit
- Zero profit
- Maximum strike width
- Minimum days to expiry (1 day)
- Maximum days to expiry (365 days)
- Negative net debit (net credit)

**Example:**

```cpp
TEST_CASE("Box spread with zero profit", "[strategy][boundary]") {
  // Given: Net debit equals theoretical value
  BoxSpreadLeg spread;
  spread.net_debit = 10.0;
  spread.theoretical_value = 10.0;

  // When: We calculate profit
  double profit = BoxSpreadCalculator::calculate_max_profit(spread);

  // Then: Profit should be zero
  REQUIRE(profit == 0.0);
}
```

### 3. Error Condition Testing

**Test invalid inputs:**

- Missing legs
- Invalid strikes
- Mismatched expiries
- Zero or negative days to expiry

**Example:**

```cpp
TEST_CASE("Implied rate calculation with zero DTE", "[strategy][error]") {
  // Given: Box spread expiring today (0 DTE)
  BoxSpreadLeg spread;
  spread.long_call.expiry = today();  // Expires today
  spread.net_debit = 9.50;

  // When: We calculate implied rate
  double rate = BoxSpreadCalculator::calculate_implied_interest_rate(spread);

  // Then: Rate should be zero (division by zero protection)
  REQUIRE(rate == 0.0);
}
```

### 4. Algorithm Correctness Testing

**Test mathematical correctness:**

- Formula verification
- Rounding behavior
- Precision handling

**Example:**

```cpp
TEST_CASE("Net debit calculation formula", "[strategy][formula]") {
  // Given: Known option prices
  BoxSpreadLeg spread;
  spread.long_call_price = 5.50;
  spread.short_call_price = 2.00;
  spread.long_put_price = 3.00;
  spread.short_put_price = 1.00;

  // When: We calculate net debit
  double net_debit = BoxSpreadCalculator::calculate_net_debit(spread);

  // Then: Should equal: 5.50 - 2.00 + 3.00 - 1.00 = 5.50
  double expected = 5.50 - 2.00 + 3.00 - 1.00;
  REQUIRE_THAT(net_debit, Catch::Matchers::WithinRel(expected, 0.01));
}
```

### 5. Integration Testing

**Test component interactions:**

- Strategy → Order Manager
- Order Manager → TWS Client
- Risk Calculator → Strategy

**Example:**

```cpp
TEST_CASE("End-to-end box spread execution", "[integration]") {
  // Given: Valid opportunity and configured components
  auto client = create_mock_tws_client();
  auto order_mgr = create_order_manager(client);
  auto strategy = create_strategy(client, order_mgr);

  // When: We execute a box spread
  BoxSpreadOpportunity opp = find_test_opportunity();
  bool success = strategy.execute_box_spread(opp);

  // Then: Order should be placed and tracked
  REQUIRE(success == true);
  auto orders = order_mgr.get_active_orders();
  REQUIRE(orders.size() == 4);  // All 4 legs
}
```

---

## Related Documentation

- **AI-Friendly Code**: `docs/AI_FRIENDLY_CODE.md` - Best practices for writing AI-friendly code
- **Common Patterns**: `docs/COMMON_PATTERNS.md` - Coding patterns and conventions
- **Architecture**: `docs/CODEBASE_ARCHITECTURE.md` - System design and component interactions
- **Static Analysis Annotations**: `docs/STATIC_ANALYSIS_ANNOTATIONS.md` - Annotations for static analyzers

---

## Usage in Cursor

When working with algorithms in Cursor, reference this document:

```
@docs ALGORITHMS_AND_BEHAVIOR.md
```

This helps Cursor understand:

- Expected algorithm behavior
- Mathematical formulas and their implementations
- Edge cases and error conditions
- Test design patterns
- Integration points between components
