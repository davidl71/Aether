# Pseudocode Implementation Strategy

**Date**: 2025-11-17
**Status**: Recommendations Document
**Purpose**: Provide actionable recommendations for using pseudocode to reduce code drift across C++, Python, Rust, Go, and TypeScript implementations

---

## Executive Summary

Based on comprehensive research (T-1) and code drift analysis (T-2), this document provides specific recommendations for implementing a pseudocode-based approach to maintain consistency across the multi-language codebase.

**Key Recommendation**: Adopt **Enhanced Pseudocode with Literate Programming** approach, focusing on critical business logic areas where code drift poses the highest risk.

---

## Research Summary (T-1)

### Approaches Evaluated

1. **Enhanced Pseudocode** ✅ **RECOMMENDED**
   - Language-agnostic algorithm documentation
   - Standard keywords (SET, IF/THEN/ELSE, FOR, WHILE)
   - Mathematical notation for formulas
   - **Pros**: Simple, maintainable, no tool dependencies
   - **Cons**: Manual maintenance required

2. **Literate Programming** ✅ **RECOMMENDED**
   - Natural language explanations interspersed with code
   - Single source of truth (documentation + code)
   - **Pros**: Excellent for complex algorithms, explains "why"
   - **Cons**: Requires discipline to maintain

3. **DRAKON Visual Programming** ⚠️ **OPTIONAL**
   - Visual flowchart-based algorithm representation
   - Code generation from charts
   - **Pros**: Visual clarity, code generation
   - **Cons**: Learning curve, tool adoption

4. **Flowcharts** ❌ **NOT RECOMMENDED**
   - Standard flowchart symbols
   - **Pros**: Simple, widely understood
   - **Cons**: No code generation, documentation only

5. **Formal Specification Languages** ❌ **NOT RECOMMENDED**
   - TLA+, Alloy, Z notation
   - **Pros**: Mathematical rigor
   - **Cons**: Overkill for this project, steep learning curve

---

## Code Drift Analysis Summary (T-2)

### High-Risk Drift Areas Identified

1. **Box Spread Calculation Logic** 🔴 **CRITICAL**
   - **C++**: `native/src/box_spread_strategy.cpp` - Full implementation
   - **Python**: `python/bindings/box_spread_bindings.pyx` - Wrapper + fallback
   - **Risk**: Python fallback uses different formula path (`spread.arbitrage_profit` vs `calculate_max_profit()`)
   - **Impact**: Financial accuracy critical

2. **Broker API Integration Patterns** 🔴 **CRITICAL**
   - **IB API**: TWS callback-based (C++)
   - **Alpaca**: REST API + WebSocket (Python)
   - **Risk**: Different patterns (callbacks vs polling) could lead to inconsistent error handling, retry logic, data transformation
   - **Impact**: Trading execution reliability

3. **Strategy Decision Logic** 🟠 **HIGH**
   - **C++**: `native/src/box_spread_strategy.cpp` - Opportunity detection
   - **Rust**: `agents/backend/crates/strategy/src/engine.rs` - Decision making
   - **Python**: `python/integration/strategy_runner.py` - Execution wrapper
   - **Risk**: Decision trees could diverge, leading to different trading decisions
   - **Impact**: Trading strategy consistency

4. **Risk Calculation Logic** 🟠 **HIGH**
   - **C++**: `native/src/risk_calculator.cpp` - Structured calculations
   - **Rust**: `agents/backend/crates/risk/src/checks.rs` - Validation patterns
   - **Risk**: Business rules could diverge
   - **Impact**: Financial risk management

5. **Data Transformation Logic** 🟡 **MEDIUM**
   - **Alpaca Service**: `build_snapshot_payload()` - API response transformation
   - **TUI Providers**: Multiple providers constructing `SnapshotPayload`
   - **C++ TUI**: Similar structure but C++ types
   - **Risk**: Data transformation could drift, causing UI inconsistencies
   - **Impact**: User experience consistency

---

## Recommended Implementation Strategy

### Phase 1: Establish Pseudocode Standards (Week 1)

**Objective**: Create standardized pseudocode format and documentation structure

**Actions**:

1. **Adopt Codecademy Pseudocode Structure**
   - Use `BEGIN`/`END` for algorithm boundaries
   - Standard keywords: `SET`, `IF/THEN/ELSE`, `FOR`, `WHILE`, `INPUT`, `OUTPUT`, `CALL`
   - Language-agnostic approach
   - Mathematical notation for formulas

2. **Create Pseudocode Template**

   ```markdown
   ## Algorithm Name

   **Purpose**: Brief description
   **Inputs**: List of inputs
   **Outputs**: List of outputs
   **Complexity**: Time/space complexity if relevant

   ### Pseudocode
   ```

   BEGIN Algorithm Name
     SET variable to value
     FOR each item in collection:
       IF condition THEN
         DO action
       ELSE
         DO alternative
       END IF
     END FOR
     OUTPUT result
   END

   ```

   ### Implementation Notes
   - Language-specific considerations
   - Performance optimizations
   - Edge cases
   ```

3. **Update `docs/ALGORITHMS_AND_BEHAVIOR.md`**
   - Review existing pseudocode
   - Make it more language-agnostic (less C++-specific)
   - Add missing algorithms

**Deliverables**:

- Pseudocode style guide document
- Updated `docs/ALGORITHMS_AND_BEHAVIOR.md`
- Template for new algorithm documentation

---

### Phase 2: Document Critical Algorithms (Week 2-3)

**Objective**: Document high-risk drift areas with pseudocode

**Priority Order**:

1. **Box Spread Calculations** (Highest Priority)
   - `calculate_net_debit()`
   - `calculate_theoretical_value()`
   - `calculate_arbitrage_profit()`
   - `calculate_roi()`
   - `calculate_implied_interest_rate()`
   - `calculate_buy_net_debit()` vs `calculate_sell_net_debit()`

2. **Broker API Integration Patterns**
   - Trading level validation (Alpaca Level 3, IB permissions)
   - Contract identification (Alpaca format vs IB contract ID)
   - Multi-leg order placement (Alpaca REST vs IB combo orders)
   - Market data retrieval (Alpaca REST/WebSocket vs IB callbacks)
   - Exercise/assignment handling (Alpaca polling vs IB callbacks)
   - Expiration handling (3:30 PM cutoff, ITM evaluation)

3. **Strategy Decision Logic**
   - Opportunity detection algorithm
   - Profitability evaluation
   - Confidence scoring
   - Risk validation

4. **Risk Calculations**
   - Box spread risk calculation
   - Position risk assessment
   - Max loss/gain calculations

**Deliverables**:

- Pseudocode documentation for each critical algorithm
- Cross-language implementation verification checklist
- Test cases derived from pseudocode

---

### Phase 3: Literate Programming Enhancement (Week 4)

**Objective**: Enhance algorithm documentation with literate programming approach

**Actions**:

1. **Expand Algorithm Documentation**
   - Add "why" explanations, not just "what"
   - Document design decisions
   - Explain edge cases and their handling
   - Include performance considerations

2. **Create Algorithm Decision Log**
   - Document when algorithms change
   - Record reasons for changes
   - Track cross-language consistency

**Deliverables**:

- Enhanced algorithm documentation
- Decision log template

---

### Phase 4: Optional - DRAKON for Complex Decision Trees (Future)

**Objective**: Evaluate DRAKON for visualizing complex decision flows

**Considerations**:

- Learning curve for team
- Tool adoption and maintenance
- Code generation capabilities
- Visual clarity benefits

**Recommendation**: Defer until Phase 1-3 complete and team evaluates need

---

## Specific Recommendations by Area

### 1. Box Spread Calculations

**Current State**:

- C++: Full implementation in `BoxSpreadCalculator` class
- Python: Wrapper around C++ + fallback implementation
- **Issue**: Python fallback uses different calculation path

**Recommendation**:

1. Document all calculation formulas in pseudocode
2. Create test suite that validates C++ and Python implementations produce identical results
3. Remove Python fallback or ensure it matches C++ exactly
4. Use pseudocode as reference for any future implementations (Rust, Go)

**Pseudocode Example**:

```
BEGIN Calculate Arbitrage Profit
  INPUT: box_spread_leg
  SET theoretical_value to box_spread_leg.strike_width
  SET net_debit to sum of:
    - long_call_price (ASK)
    - long_put_price (ASK)
    - short_call_price (BID, negative)
    - short_put_price (BID, negative)
  SET arbitrage_profit to theoretical_value - net_debit
  RETURN arbitrage_profit
END
```

---

### 2. Broker API Integration

**Current State**:

- IB API: TWS callback-based (C++)
- Alpaca: REST API + WebSocket (Python)
- Different patterns for same operations

**Recommendation**:

1. Document broker-agnostic pseudocode for all operations
2. Create abstraction layer that implements pseudocode
3. Broker-specific implementations follow pseudocode structure
4. Document differences in implementation notes

**Pseudocode Example**:

```
BEGIN Place Box Spread Order (Broker-Agnostic)
  INPUT: box_spread_leg, account_id, broker_type

  SET required_trading_level to 3
  IF NOT ValidateTradingLevel(account_id, required_trading_level, broker_type) THEN
    RETURN error: "Insufficient trading level"
  END IF

  SET legs to [
    {contract: long_call, action: BUY, quantity: 1},
    {contract: short_call, action: SELL, quantity: 1},
    {contract: long_put, action: BUY, quantity: 1},
    {contract: short_put, action: SELL, quantity: 1}
  ]

  IF broker_type is "ALPACA" THEN
    CALL PlaceAlpacaMultiLegOrder(legs)
  ELSE IF broker_type is "IB" THEN
    CALL PlaceIBComboOrder(legs)
  END IF

  RETURN order_id
END
```

---

### 3. Strategy Decision Logic

**Current State**:

- C++: Full strategy evaluation
- Rust: Strategy engine with different patterns
- Python: Execution wrapper

**Recommendation**:

1. Document decision tree in pseudocode
2. Create decision flowchart (optional DRAKON)
3. Ensure all implementations follow same decision logic
4. Use pseudocode for code reviews

---

### 4. Risk Calculations

**Current State**:

- C++: Structured risk calculations
- Rust: Similar patterns, different structures

**Recommendation**:

1. Document risk calculation formulas in pseudocode
2. Ensure mathematical consistency
3. Create shared test data for validation

---

## Implementation Guidelines

### Pseudocode Writing Standards

1. **Use Standard Keywords**
   - `BEGIN`/`END` for algorithm boundaries
   - `SET` for variable assignment
   - `IF/THEN/ELSE` for conditionals
   - `FOR`/`WHILE` for loops
   - `INPUT`/`OUTPUT` for I/O
   - `CALL` for function calls

2. **Language-Agnostic Approach**
   - Avoid language-specific syntax
   - Use mathematical notation for formulas
   - Focus on logic, not implementation

3. **Include Edge Cases**
   - Document error conditions
   - Handle boundary cases
   - Specify validation requirements

4. **Mathematical Formulas**
   - Use standard mathematical notation
   - Define all variables
   - Show calculation steps

### Documentation Structure

Each algorithm should include:

1. **Purpose**: What the algorithm does
2. **Inputs**: Required parameters
3. **Outputs**: Return values
4. **Preconditions**: Assumptions/requirements
5. **Postconditions**: Guarantees after execution
6. **Pseudocode**: Algorithm in pseudocode
7. **Implementation Notes**: Language-specific considerations
8. **Test Cases**: Key test scenarios
9. **Complexity**: Time/space complexity if relevant

---

## Migration Path

### Step 1: Document Existing Algorithms (Immediate)

- Start with box spread calculations
- Document broker API patterns
- Create pseudocode for critical paths

### Step 2: Validate Consistency (Week 2-3)

- Compare implementations against pseudocode
- Identify and document discrepancies
- Create test suite for validation

### Step 3: Refactor if Needed (Week 4+)

- Align implementations with pseudocode
- Remove drift where found
- Update pseudocode as algorithms evolve

### Step 4: Maintain (Ongoing)

- Update pseudocode when algorithms change
- Use pseudocode in code reviews
- Validate new implementations against pseudocode

---

## Decision Matrix

| Approach | Effort | Benefits | Maintenance | Recommendation |
|----------|--------|----------|-------------|----------------|
| **Enhanced Pseudocode** | Low | High | Low | ✅ **RECOMMENDED** |
| **Literate Programming** | Medium | Very High | Medium | ✅ **RECOMMENDED** |
| **DRAKON** | High | Medium | Medium | ⚠️ **OPTIONAL** |
| **Flowcharts** | Low | Low | Low | ❌ **NOT RECOMMENDED** |
| **Formal Specs** | Very High | Very High | Very High | ❌ **NOT RECOMMENDED** |

---

## Success Criteria

1. ✅ All critical algorithms documented in pseudocode
2. ✅ Cross-language implementations validated against pseudocode
3. ✅ Code reviews include pseudocode verification
4. ✅ New implementations follow pseudocode structure
5. ✅ Code drift reduced in high-risk areas
6. ✅ Alpaca integration patterns documented consistently

---

## Estimated Effort

- **Phase 1** (Standards): 1 week
- **Phase 2** (Documentation): 2-3 weeks
- **Phase 3** (Enhancement): 1 week
- **Phase 4** (DRAKON - Optional): 2-3 weeks (if pursued)

**Total**: 4-5 weeks for core implementation, 6-8 weeks if including DRAKON

---

## Next Steps

1. **Review and Approve**: Get team approval on recommended approach
2. **Create Style Guide**: Document pseudocode standards
3. **Start with Box Spread Calculations**: Highest priority, highest risk
4. **Iterate**: Document algorithms as they're reviewed/refactored
5. **Validate**: Use pseudocode in code reviews and testing

---

## References

- T-1 Research: Comprehensive pseudocode methodology research
- T-2 Analysis: Code drift patterns identified
- `docs/ALGORITHMS_AND_BEHAVIOR.md`: Existing algorithm documentation
- Alpaca Options Trading Docs: API patterns for pseudocode documentation

---

**Document Status**: ✅ Complete - Ready for implementation
