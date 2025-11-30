# Action Plan: Top 4 Priorities

**Date**: 2025-01-27
**Based on**: Learnings from icli and ibkrbox projects

---

## Priority 1: Complete Option Chain Scanning Implementation

**Status**: 🔴 Not Started
**File**: `src/box_spread_strategy.cpp`
**Method**: `find_box_spreads_in_chain()`

### Current State

- Method exists but is stubbed (returns empty vector)
- Option chain structure exists but not fully utilized

### Implementation Required

1. Group options by expiry date
2. For each expiry, identify all strike pairs (K1, K2) where K1 < K2
3. For each strike pair, verify all 4 legs exist:
   - Long call at K1
   - Short call at K2
   - Long put at K2
   - Short put at K1
4. Calculate profitability for each valid combination
5. Sort by profit/ROI and return top opportunities

### Code Location

```cpp
// src/box_spread_strategy.cpp:99-113
std::vector<BoxSpreadOpportunity> BoxSpreadStrategy::find_box_spreads_in_chain(
    const option_chain::OptionChain& chain,
    double underlying_price) {
    // TODO: Implement full scanning logic
}
```

### Dependencies

- `OptionChain` class must be populated with market data
- Market data subscriptions for all options in chain
- Strike pair generation algorithm

---

## Priority 2: Implement Atomic Execution (All-or-Nothing)

**Status**: 🔴 Not Started
**File**: `src/order_manager.cpp`
**Method**: `place_box_spread()`

### Current State

- Places 4 separate orders sequentially
- No guarantee of atomic execution
- Risk of partial fills breaking box spread structure

### Implementation Required

1. **Option A (Preferred)**: Use IBKR combo orders
   - Create `ComboLeg` structures for all 4 legs
   - Place as single combo order
   - Guarantees all-or-nothing execution

2. **Option B (Fallback)**: Implement rollback logic
   - Place all 4 orders rapidly
   - Monitor fill status
   - If any leg fails, cancel remaining orders
   - Track order IDs for rollback

### Code Location

```cpp
// src/order_manager.cpp:128-194
ExecutionResult OrderManager::place_box_spread(
    const types::BoxSpreadLeg& spread,
    const std::string& strategy_id) {
    // Currently places 4 separate orders
    // TODO: Implement combo order or rollback logic
}
```

### Risk Mitigation

- Partial fills expose position to market risk
- Early assignment risk if not all legs filled
- Need rapid cancellation capability

---

## Priority 3: Add Comprehensive Validation Rules

**Status**: 🟡 Partially Complete
**File**: `src/box_spread_strategy.cpp`
**Class**: `BoxSpreadValidator`

### Current State

- Basic validation exists (strikes, expiries, symbols, pricing)
- Missing some critical validations

### Additional Validations Needed

1. **Strike Width Validation**

   ```cpp
   // Verify: theoretical_value == (strike_high - strike_low)
   double strike_width = spread.short_call.strike - spread.long_call.strike;
   if (std::abs(spread.theoretical_value - strike_width) > 0.01) {
       errors.push_back("Theoretical value must equal strike width");
   }
   ```

2. **Market Data Availability**
   - All 4 legs must have valid bid/ask prices
   - Check for stale data (timestamp validation)
   - Verify exchange compatibility (SMART routing or same exchange)

3. **Liquidity Checks**
   - Minimum bid/ask spread threshold
   - Minimum volume/open interest
   - Execution probability based on liquidity

4. **Arbitrage Validation**
   - Net debit < Theoretical value (for profitable arbitrage)
   - After commission, profit > minimum threshold
   - ROI meets minimum requirements

### Code Location

```cpp
// src/box_spread_strategy.cpp:284-316
bool BoxSpreadValidator::validate(
    const types::BoxSpreadLeg& spread,
    std::vector<std::string>& errors) {
    // TODO: Add additional validation rules
}
```

---

## Priority 4: Market Data Quality Checks

**Status**: 🔴 Not Started
**File**: `src/box_spread_strategy.cpp`
**Method**: `evaluate_box_spread()`

### Current State

- Market data can be requested but quality not validated
- No checks for stale data or missing prices
- No liquidity assessment before execution

### Implementation Required

1. **Data Freshness Validation**

   ```cpp
   // Check market data timestamp
   auto data_age = std::chrono::system_clock::now() - market_data.timestamp;
   if (data_age > max_data_age_threshold) {
       return std::nullopt; // Stale data
   }
   ```

2. **Bid/Ask Availability**
   - Verify all 4 legs have both bid and ask prices
   - Check for zero or negative spreads (invalid data)
   - Validate prices are within reasonable bounds

3. **Liquidity Assessment**
   - Calculate bid/ask spread percentage
   - Check volume and open interest
   - Estimate execution probability
   - Filter out low-liquidity opportunities

4. **Spread Width Validation**
   - Maximum acceptable bid/ask spread (e.g., 5% of mid price)
   - Minimum volume threshold
   - Minimum open interest threshold

### Code Location

```cpp
// src/box_spread_strategy.cpp:115-124
std::optional<BoxSpreadOpportunity> BoxSpreadStrategy::evaluate_box_spread(
    const types::OptionContract& long_call,
    const types::OptionContract& short_call,
    const types::OptionContract& long_put,
    const types::OptionContract& short_put,
    const option_chain::OptionChain& chain) {
    // TODO: Add market data quality checks
}
```

### Integration Points

- `TWSClient::request_market_data()` - ensure data is fresh
- `types::MarketData` - add timestamp field if missing
- `BoxSpreadOpportunity` - add liquidity_score field

---

## Implementation Order

### Phase 1: Foundation (Week 1)

1. ✅ Priority 3: Add validation rules (foundation for everything)
2. ✅ Priority 4: Market data quality checks (required before execution)

### Phase 2: Core Functionality (Week 2)

1. ✅ Priority 1: Option chain scanning (enables opportunity detection)
2. ✅ Priority 2: Atomic execution (enables safe execution)

---

## Success Criteria

### Priority 1 Complete When

- [ ] `find_box_spreads_in_chain()` returns valid opportunities
- [ ] All strike pairs are evaluated
- [ ] Opportunities are sorted by profitability
- [ ] Unit tests pass for chain scanning

### Priority 2 Complete When

- [ ] Box spreads execute atomically (all-or-nothing)
- [ ] Rollback logic works if any leg fails
- [ ] No partial fills break box spread structure
- [ ] Integration tests pass with paper trading

### Priority 3 Complete When

- [ ] All validation rules implemented
- [ ] Validation catches invalid box spreads
- [ ] Clear error messages for each validation failure
- [ ] Unit tests cover all validation scenarios

### Priority 4 Complete When

- [ ] Market data freshness is validated
- [ ] Low-liquidity opportunities are filtered out
- [ ] Execution probability is calculated
- [ ] No executions occur with stale or invalid data

---

## Related Files to Modify

### Core Implementation

- `src/box_spread_strategy.cpp` - Main strategy logic
- `src/order_manager.cpp` - Order execution
- `include/box_spread_strategy.h` - Strategy interface
- `include/order_manager.h` - Order management interface

### Supporting Files

- `include/types.h` - Data structures (may need additions)
- `include/option_chain.h` - Option chain structure
- `src/tws_client.cpp` - TWS API integration (for combo orders)

### Tests

- `tests/test_box_spread_strategy.cpp` - Strategy tests
- `tests/test_order_manager.cpp` - Order execution tests

---

## Notes

- All priorities are interdependent
- Start with validation (Priority 3) as it's needed by everything else
- Market data quality (Priority 4) should be implemented before execution
- Option chain scanning (Priority 1) enables the strategy to work
- Atomic execution (Priority 2) is critical for risk management

---

## References

- See `docs/IBKRBOX_LEARNINGS.md` for detailed patterns
- See `docs/ICLI_LEARNINGS.md` for API usage patterns
- IBKR Combo Orders: <https://interactivebrokers.github.io/tws-api/combo_orders.html>
