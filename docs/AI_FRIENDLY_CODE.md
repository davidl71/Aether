# Making Code AI-Friendly

This guide describes best practices for writing code that AI assistants (like Cursor) can understand and work with effectively.

## Principles

### 1. **Explicit Over Implicit**
AI works better with explicit, clear code than clever abstractions.

**❌ Bad:**
```cpp
auto result = process(data);
```

**✅ Good:**
```cpp
std::vector<BoxSpread> opportunities = strategy.scan_opportunities(option_chain);
```

### 2. **Self-Documenting Names**
Use descriptive names that explain intent.

**❌ Bad:**
```cpp
void calc(double a, double b);
```

**✅ Good:**
```cpp
double calculate_arbitrage_profit(double strike_width, double net_debit);
```

### 3. **Type Clarity**
Make types explicit, especially in function signatures.

**❌ Bad:**
```cpp
void process(const auto& item);
```

**✅ Good:**
```cpp
void process_order(const types::Order& order);
```

### 4. **Documentation Comments**
Add comments explaining *why*, not *what*.

**❌ Bad:**
```cpp
// Increment counter
counter++;
```

**✅ Good:**
```cpp
// Increment opportunity counter after successful scan
// This tracks how many arbitrage opportunities we've found this session
stats_.opportunities_found++;
```

## Code Documentation Patterns

### Function Documentation

**Use clear function documentation:**
```cpp
/// Scans option chain for box spread arbitrage opportunities.
///
/// @param symbol The underlying symbol to scan (e.g., "SPY")
/// @param option_chain The current option chain data
/// @return Vector of actionable box spread opportunities
///
/// @note This function filters opportunities based on:
///   - Minimum profit threshold (from config)
///   - Minimum ROI threshold (from config)
///   - Liquidity requirements (bid/ask spread, volume)
///   - Risk limits (max position size, exposure)
std::vector<BoxSpreadOpportunity> scan_opportunities(
    const std::string& symbol,
    const OptionChain& option_chain
) const;
```

### Class Documentation

**Document class purpose and usage:**
```cpp
/// Manages multi-leg box spread orders and tracks execution status.
///
/// This class coordinates the placement and tracking of complex multi-leg
/// orders required for box spread strategies. It ensures all four legs of
/// a box spread are submitted atomically and tracks their execution status.
///
/// @example
///   OrderManager mgr(config);
///   Order order = create_box_spread_order(spread);
///   mgr.submit_order(order);
///   // Order status updates come via TWS callbacks
class OrderManager {
  // ...
};
```

### Inline Comments for Complex Logic

**Explain non-obvious logic:**
```cpp
// Calculate arbitrage profit: strike width minus net debit
// Box spreads are risk-free if net debit < strike width
// Example: Strike width = $10, Net debit = $9.50 → Profit = $0.50
double profit = strike_width - net_debit;

// Validate against minimum thresholds
// ROI is calculated as (profit / net_debit) * 100
double roi_percent = (profit / net_debit) * 100.0;
if (profit >= config.get_min_profit() &&
    roi_percent >= config.get_min_roi())
{
  return true;  // Valid arbitrage opportunity
}
```

## Error Messages

### Structured Error Messages

**Make errors actionable:**
```cpp
// ❌ Bad
throw std::runtime_error("Error");

// ✅ Good
throw std::runtime_error(
    "Failed to connect to TWS API: " + error_msg + "\n"
    "Possible causes:\n"
    "  1. TWS/Gateway is not running\n"
    "  2. API connections not enabled in TWS settings\n"
    "  3. Incorrect port number (7497 = paper, 7496 = live)\n"
    "  4. Firewall blocking connection\n"
    "Solution: Check TWS → Configure → API → Settings"
);
```

### Error Context

**Include context in error messages:**
```cpp
if (order.status == OrderStatus::Rejected)
{
  spdlog::error(
      "Order {} rejected: {}\n"
      "  Symbol: {}\n"
      "  Quantity: {}\n"
      "  Price: ${}\n"
      "  Reason: {}",
      order.order_id,
      order.status_message,
      order.contract.symbol,
      order.quantity,
      order.limit_price,
      rejection_reason
  );
}
```

## Type Definitions

### Use Type Aliases for Clarity

**Create type aliases for complex types:**
```cpp
// In types.h
using OrderId = int;
using RequestId = int;
using ContractId = std::string;
using Symbol = std::string;

// Usage
OrderId place_order(const Contract& contract, int quantity);
```

### Strong Types for Domain Concepts

**Use strong types instead of primitives:**
```cpp
// ❌ Bad
double calculate_profit(double strike_width, double net_debit);

// ✅ Good
struct StrikeWidth {
  double value;
  explicit StrikeWidth(double v) : value(v) {}
};

struct NetDebit {
  double value;
  explicit NetDebit(double v) : value(v) {}
};

double calculate_profit(StrikeWidth strike_width, NetDebit net_debit);
```

## Configuration Documentation

### Document Configuration Options

**Add comments to config files:**
```json
{
  "tws": {
    "host": "127.0.0.1",  // TWS/Gateway hostname (localhost)
    "port": 7497,         // 7497 = Paper Trading, 7496 = Live Trading
    "client_id": 1        // Unique client ID (1-32), must be unique per connection
  },
  "strategy": {
    "symbols": ["SPY"],   // Underlying symbols to monitor for opportunities
    "min_arbitrage_profit": 0.10,  // Minimum profit in dollars to execute
    "min_roi_percent": 0.5,        // Minimum ROI percentage (0.5% = 0.5)
    "max_position_size": 10000.0   // Maximum capital per trade in dollars
  }
}
```

## Test Documentation

### Document Test Intent

**Explain what each test validates:**
```cpp
TEST_CASE("Box spread profit calculation with valid inputs", "[strategy][profit]")
{
  // Given: A box spread with strike width $10 and net debit $9.50
  BoxSpread spread = create_test_spread(10.0, 9.50);

  // When: We calculate the profit
  double profit = calculate_profit(spread);

  // Then: Profit should be $0.50 (strike width - net debit)
  REQUIRE(profit == 0.50);
  REQUIRE(profit > 0.0);  // Must be profitable
}

TEST_CASE("Risk limits prevent oversized positions", "[risk][limits]")
{
  // Given: A position that exceeds max position size
  double position_size = 20000.0;  // Exceeds max of 10000.0

  // When: We validate the position
  // Then: Validation should fail
  REQUIRE_THROWS_AS(
    risk_calculator.validate_position(position_size),
    RiskLimitExceeded
  );
}
```

## Code Organization

### Group Related Code

**Keep related functionality together:**
```cpp
// ============================================================================
// Connection Management
// ============================================================================

bool TWSClient::connect() { /* ... */ }
void TWSClient::disconnect() { /* ... */ }
bool TWSClient::is_connected() const { /* ... */ }

// ============================================================================
// Market Data Operations
// ============================================================================

int TWSClient::request_market_data(const Contract& contract) { /* ... */ }
void TWSClient::cancel_market_data(int request_id) { /* ... */ }

// ============================================================================
// Order Operations
// ============================================================================

int TWSClient::place_order(const Order& order) { /* ... */ }
void TWSClient::cancel_order(int order_id) { /* ... */ }
```

### Use Section Comments

**Organize large files with section comments:**
```cpp
// ============================================================================
// Public Interface
// ============================================================================

// ============================================================================
// Private Implementation
// ============================================================================

// ============================================================================
// Helper Functions
// ============================================================================
```

## Naming Conventions

### Use Domain Language

**Use trading/finance terminology:**
```cpp
// ✅ Good - uses domain language
double calculate_net_debit(const BoxSpread& spread);
double get_strike_width(const BoxSpread& spread);
double calculate_roi(double profit, double investment);

// ❌ Bad - generic names
double calc(double a, double b);
double get_diff(double x, double y);
```

### Be Consistent

**Use consistent naming patterns:**
```cpp
// All getters use "get_" prefix
double get_strike_width() const;
double get_net_debit() const;
double get_profit() const;

// All validators use "is_" or "validate_" prefix
bool is_valid() const;
bool validate_order(const Order& order) const;
bool validate_risk_limits(double position_size) const;
```

## Examples and Usage

### Provide Usage Examples

**Include examples in documentation:**
```cpp
/// Creates a box spread order from a box spread opportunity.
///
/// @param opportunity The box spread opportunity to execute
/// @return Order ready for submission to TWS
///
/// @example
///   BoxSpreadOpportunity opp = find_best_opportunity();
///   Order order = create_box_spread_order(opp);
///   order_manager.submit_order(order);
Order create_box_spread_order(const BoxSpreadOpportunity& opportunity);
```

## Summary Checklist

When writing code, ask yourself:

- [ ] **Names**: Are function/variable names self-explanatory?
- [ ] **Types**: Are types explicit and clear?
- [ ] **Comments**: Do comments explain *why*, not *what*?
- [ ] **Errors**: Are error messages actionable and include context?
- [ ] **Documentation**: Is there documentation for public APIs?
- [ ] **Examples**: Are there usage examples for complex functions?
- [ ] **Consistency**: Does code follow established patterns?
- [ ] **Domain Language**: Are domain-specific terms used correctly?

## Related Documentation

- **Common Patterns**: `docs/COMMON_PATTERNS.md`
- **Architecture**: `docs/CODEBASE_ARCHITECTURE.md`
- **API Reference**: `docs/API_DOCUMENTATION_INDEX.md`
