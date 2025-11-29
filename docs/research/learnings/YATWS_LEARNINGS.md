# Learnings from yatws Rust TWS API Library

**Source**: [yatws Documentation](https://docs.rs/yatws/latest/yatws/)
**Date**: 2025-11-13
**Purpose**: Extract best practices and implementation patterns from a production Rust TWS API library

## Overview

yatws (Yet Another TWS API) is a Rust library for Interactive Brokers TWS API that has been used in production and has traded millions of dollars in volume. Key focus: **fast order placement (~3ms per order)**.

## Key Features Worth Adopting

### 1. Rate Limiting ⭐ **HIGH PRIORITY**

**Implementation**: Configurable rate limiter with multiple limits

**Features**:

- Maximum messages per second (default: 50)
- Maximum simultaneous historical data requests (default: 50)
- Maximum market data lines (default: 100)
- Stale request cleanup (removes requests older than threshold)

**Why Important**:

- **IBKR Compliance**: Prevents API violations
- **Account Protection**: Avoids account restrictions
- **Reliability**: Reduces connection issues from excessive requests

**Implementation Pattern**:

```cpp
// Enable rate limiting
client.enable_rate_limiting();

// Configure custom limits
RateLimiterConfig config;
config.enabled = true;
config.max_messages_per_second = 40;  // More conservative
config.max_historical_requests = 30;
client.configure_rate_limiter(config);

// Monitor status
auto status = client.get_rate_limiter_status();
if (status) {
    std::cout << "Active historical requests: "
              << status->active_historical_requests << std::endl;
}

// Cleanup stale requests
client.cleanup_stale_rate_limiter_requests(
    std::chrono::seconds(300)  // 5 minutes
);
```

**Status**: Not implemented in our codebase (mentioned in docs but not implemented)

### 2. Session Recording/Replay

**Implementation**: Record all TWS interactions to SQLite database

**Features**:

- Record requests/responses with timestamps
- Replay sessions for testing
- Useful for debugging without live TWS connection

**Why Important**:

- **Testing**: Test without live TWS connection
- **Debugging**: Reproduce issues with recorded data
- **Backtesting**: Test strategies with real API interactions

**Implementation Pattern**:

```cpp
// Enable recording
IBKRClient client("127.0.0.1", 7497, 0,
                  {"sessions.db", "my_session"});

// Later, replay
IBKRClient replay = IBKRClient::from_db("sessions.db", "my_session");
```

**Status**: Not implemented in our codebase

### 3. Manager-Based Architecture

**Implementation**: Organized access through specialized managers

**Structure**:

- `OrderManager` - Order operations
- `AccountManager` - Account data
- `MarketDataManager` - Market data
- `ReferenceDataManager` - Contract details
- `NewsManager` - News feeds
- `FundamentalsManager` - Financial data

**Why Important**:

- **Organization**: Clear separation of concerns
- **Maintainability**: Easier to find and modify code
- **Testability**: Managers can be tested independently

**Status**: Our codebase is more monolithic - consider refactoring if it grows

### 4. Options Strategy Builder

**Implementation**: Simplified creation of common options strategies

**Features**:

- Bull call spreads
- Bear put spreads
- Straddles
- Strangles
- Custom multi-leg strategies

**Why Important**:

- **Box Spreads**: Relevant for our use case
- **Ease of Use**: Simplifies complex order creation
- **Type Safety**: Prevents errors in strategy construction

**Example**:

```cpp
OptionsStrategyBuilder builder(client.data_ref(), "AAPL", 150.0, 10.0, SecType::Stock);

auto [contract, order] = builder
    .bull_call_spread(expiry, 150.0, 160.0)
    .with_limit_price(3.50)
    .build();
```

**Status**: Not implemented - could be valuable for box spreads

### 5. Strong Type Safety

**Implementation**: Uses enums instead of strings

**Examples**:

- `OrderType` enum instead of "LMT"/"MKT" strings
- `TimeInForce` enum instead of "DAY"/"GTC" strings
- `SecType` enum instead of "STK"/"OPT" strings
- `OrderAction` enum instead of "BUY"/"SELL" strings

**Why Important**:

- **Compile-time Safety**: Catches errors at compile time
- **IDE Support**: Better autocomplete and documentation
- **Prevents Typos**: Can't accidentally use wrong string value

**Status**: Partially implemented - we use some enums but still use strings in places

### 6. Multiple Programming Patterns

**Implementation**: Supports synchronous, asynchronous, and observer patterns

**Patterns**:

1. **Synchronous (Blocking)**: `get_quote(contract, timeout)`
2. **Asynchronous Request/Cancel**: `request_market_data()` / `cancel_market_data()`
3. **Observer Pattern**: Subscribe to events
4. **Subscription Model**: Iterator-based subscriptions

**Why Important**:

- **Flexibility**: Different patterns for different use cases
- **Performance**: Choose best pattern for each operation
- **Ease of Use**: Synchronous for simple operations, async for complex

**Status**: Our codebase supports some patterns but could be more comprehensive

### 7. Book-keeping

**Implementation**: Automatically maintains portfolio and order book

**Features**:

- Keeps portfolio with P&L up-to-date
- Maintains order book
- Tracks positions automatically

**Why Important**:

- **State Management**: Always know current state
- **P&L Tracking**: Real-time profit/loss
- **Order Tracking**: Know status of all orders

**Status**: Partially implemented - we track orders and positions but could improve P&L tracking

### 8. Error Handling

**Implementation**: Consistent Result pattern with custom error type

**Features**:

- `IBKRError` enum with specific error types
- `Timeout` error type
- `ApiError(code, message)` for TWS errors
- Pattern matching for error handling

**Why Important**:

- **Type Safety**: Can't ignore errors
- **Clear Error Messages**: Specific error types
- **Error Recovery**: Can handle different errors differently

**Status**: We have error handling but could improve with more specific error types

## Implementation Recommendations

### High Priority (Do First)

1. **Rate Limiting** ⭐
   - Critical for IBKR compliance
   - Prevents account restrictions
   - Estimated: 3-4 hours

### Medium Priority (Do Next)

1. **Type Safety Improvements**
   - Replace string parameters with enums
   - Better compile-time checking
   - Estimated: 2-3 hours

2. **Options Strategy Builder**
   - Simplify box spread creation
   - Better for multi-leg orders
   - Estimated: 2-3 hours

### Low Priority (Nice to Have)

1. **Session Recording/Replay**
   - Great for testing
   - Useful for debugging
   - Estimated: 4-6 hours

2. **Manager-Based Architecture**
   - Better organization
   - Only if codebase grows significantly
   - Estimated: 1-2 weeks (major refactoring)

## Comparison with Our Implementation

| Feature | yatws | Our Implementation | Priority |
|---------|-------|-------------------|----------|
| Rate Limiting | ✅ Full implementation | ❌ Not implemented | HIGH |
| Session Recording | ✅ SQLite-based | ❌ Not implemented | LOW |
| Manager Architecture | ✅ Organized | ⚠️ Monolithic | LOW |
| Strategy Builder | ✅ Options strategies | ❌ Not implemented | MEDIUM |
| Type Safety | ✅ Strong (enums) | ⚠️ Partial (some strings) | MEDIUM |
| Error Handling | ✅ Result pattern | ⚠️ Callbacks + exceptions | MEDIUM |
| Book-keeping | ✅ Auto-updates | ⚠️ Manual tracking | LOW |

## References

- [yatws Documentation](https://docs.rs/yatws/latest/yatws/) - Full API documentation
- [yatws Repository](https://github.com/drpngx/yatws) - Source code (if available)
- [Code Improvements Action Plan](../../archive/CODE_IMPROVEMENTS_ACTION_PLAN.md) - Our implementation plan
