# Code Improvements Action Plan

Based on NotebookLM research and code analysis, here's a prioritized action plan for improving the TWS API implementation.

## Current Status Assessment

### ✅ Already Implemented Well

1. **DefaultEWrapper** ✅ - Correctly inherits from `DefaultEWrapper`
2. **Separate Mutexes** ✅ - Uses `data_mutex_`, `order_mutex_`, `position_mutex_`, `account_mutex_`, `error_mutex_`
3. **EReader Thread Timing** ✅ - Starts before waiting for `nextValidId`
4. **Async Connection Mode** ✅ - Uses `asyncEConnect(true)`
5. **Exponential Backoff Reconnection** ✅ - Implemented in `attempt_reconnect_with_backoff()`
6. **State Synchronization** ✅ - Calls `reqAllOpenOrders()` in `nextValidId()`
7. **Error Guidance** ✅ - Has `kIbErrorGuidance` map with actionable advice
8. **Health Monitoring** ✅ - Implemented in `start_health_monitoring()`

### 🔧 Needs Improvement

## Priority 1: Add Try-Catch to All Callbacks (HIGH)

**Status**: Only 4 callbacks have try-catch protection:

- ✅ `tickPrice()` - Has try-catch
- ✅ `tickSize()` - Has try-catch
- ✅ `tickOptionComputation()` - Has try-catch
- ✅ `orderStatus()` - Has try-catch

**Missing**: All other callbacks need try-catch protection.

### Callbacks That Need Try-Catch

1. **Connection Callbacks**:
   - `connectAck()` - Line 536
   - `connectionClosed()` - Line 557
   - `managedAccounts()` - Line 576
   - `nextValidId()` - Line 587

2. **Order Callbacks**:
   - `openOrder()` - Line ~730
   - `execDetails()` - Line ~820
   - `openOrderEnd()` - If implemented

3. **Position Callbacks**:
   - `position()` - Line ~845
   - `positionEnd()` - Line ~877

4. **Account Callbacks**:
   - `updateAccountValue()` - Line ~891
   - `updatePortfolio()` - Line ~924
   - `accountDownloadEnd()` - Line ~944

5. **Error Callback**:
   - `error()` - Line 979 (Should be wrapped too!)

### Implementation Pattern

```cpp
void callbackName(...) override {
    try {
        // Existing implementation
    } catch (const std::exception& e) {
        spdlog::error("Exception in callbackName: {} (context: ...)", e.what());
        // Don't crash - just log
    } catch (...) {
        spdlog::error("Unknown exception in callbackName");
    }
}
```

**Estimated Effort**: 2-3 hours to add try-catch to all ~15 remaining callbacks

## Priority 2: Enhance Error Handling (HIGH)

**Current**: Has guidance map but could be more comprehensive.

### Improvements Needed

1. **Add More Error Codes to Guidance Map**
   - Current: Has some common codes
   - Needed: Add codes 162, 200, 1101, 1102, and more

2. **Improve Error 1100 Handling**
   - Current: Sets state to Error
   - Needed: Should trigger automatic reconnection if enabled

3. **Add Error Context Logging**
   - Current: Logs error code and message
   - Needed: Include request ID, contract details, order ID when available

### Example Enhancement

```cpp
void error(int id, time_t errorTime, int errorCode, const std::string& errorString,
          const std::string& advancedOrderRejectJson) override {
    try {
        // Store error
        {
            std::lock_guard<std::mutex> lock(error_mutex_);
            last_error_code_ = errorCode;
            last_error_message_ = errorString;
        }

        // Enhanced guidance with more codes
        std::string guidance = get_error_guidance(errorCode, errorString);

        // Log with context
        if (errorCode < 1100) {
            spdlog::error("[IB Error {}] ID: {} | {}", errorCode, id, errorString);
            if (!guidance.empty()) {
                spdlog::error("  → {}", guidance);
            }
        } else if (errorCode < 2000) {
            spdlog::warn("[IB System {}] ID: {} | {}", errorCode, id, errorString);
        } else {
            spdlog::info("[IB Info {}] ID: {} | {}", errorCode, id, errorString);
        }

        // Handle critical errors
        if (errorCode == 1100) {
            connected_ = false;
            state_ = ConnectionState::Error;
            if (config_.auto_reconnect) {
                attempt_reconnect_with_backoff();
            }
        }

        // ... rest of implementation
    } catch (const std::exception& e) {
        spdlog::error("Exception in error callback: {}", e.what());
    }
}
```

**Estimated Effort**: 1-2 hours

## Priority 3: Improve State Synchronization (MEDIUM)

**Current**: Calls `reqAllOpenOrders()` in `nextValidId()` ✅

### Enhancements Needed

1. **Add Position Synchronization**
   - Current: Not called in `nextValidId()`
   - Needed: Call `reqPositions()` after reconnection

2. **Add Account Synchronization**
   - Current: Not called in `nextValidId()`
   - Needed: Call `reqAccountUpdates(true, "")` after reconnection

3. **Track Reconnection State**
   - Current: Doesn't distinguish initial connection from reconnection
   - Needed: Track if this is a reconnection to sync state

### Example Enhancement

```cpp
void nextValidId(OrderId orderId) override {
    try {
        spdlog::info("Received nextValidId: {} - Connection fully established", orderId);

        std::lock_guard<std::mutex> lock(connection_mutex_);
        next_order_id_ = orderId;
        connected_ = true;

        bool is_reconnection = (reconnect_attempts_.load() > 0);

        if (is_reconnection) {
            spdlog::info("Reconnection detected. Synchronizing state with TWS...");
        }

        // Always sync orders (important for reconnection)
        client_.reqAllOpenOrders();

        // Sync positions and account after reconnection
        if (is_reconnection) {
            client_.reqPositions();
            client_.reqAccountUpdates(true, "");
            reconnect_attempts_ = 0; // Reset after successful sync
        }

        start_health_monitoring();
        connection_cv_.notify_all();

    } catch (const std::exception& e) {
        spdlog::error("Exception in nextValidId: {}", e.what());
    }
}
```

**Estimated Effort**: 30 minutes

## Priority 4: Rate Limiting Implementation (HIGH - IBKR Compliance)

**Status**: Not implemented (mentioned in docs but not implemented)

**Reference**: [yatws rate limiter](https://docs.rs/yatws/latest/yatws/) provides a concrete implementation example

### Implementation Needed

Based on yatws implementation, add configurable rate limiter:

1. **Message Rate Limiting**
   - Maximum messages per second (default: 50, IBKR allows up to 50)
   - Track message timestamps
   - Throttle requests if limit exceeded

2. **Historical Data Request Limiting**
   - Maximum simultaneous historical requests (default: 50)
   - Track active historical data requests
   - Queue requests if limit exceeded

3. **Market Data Line Limiting**
   - Maximum market data lines (default: 100)
   - Track active market data subscriptions
   - Reject new subscriptions if limit exceeded

4. **Configuration Interface**

   ```cpp
   struct RateLimiterConfig {
       bool enabled = false;
       int max_messages_per_second = 50;
       int max_historical_requests = 50;
       int max_market_data_lines = 100;
   };

   void enable_rate_limiting();
   void configure_rate_limiter(const RateLimiterConfig& config);
   RateLimiterStatus get_rate_limiter_status() const;
   ```

5. **Stale Request Cleanup**
   - Clean up stale requests older than threshold (e.g., 5 minutes)
   - Prevent memory leaks in long-running applications

### Benefits

- **IBKR Compliance**: Prevents API violations and account restrictions
- **Prevents Throttling**: Avoids TWS rate limiting responses
- **Better Reliability**: Reduces connection issues from excessive requests

**Estimated Effort**: 3-4 hours (includes testing)

## Priority 5: Order Efficiency Ratio Tracking (MEDIUM)

**Status**: Not implemented

### Implementation Needed

Add tracking for:

- Total orders placed
- Executed trades
- Calculate ratio (trades/orders)
- Warn if ratio < 0.05 (1:20) and orders > 20

**Estimated Effort**: 1-2 hours

## Priority 6: Atomic Multi-Leg Execution (HIGH for Box Spreads)

**Status**: Need to verify current implementation

### Check Current Implementation

1. Review `place_box_spread_order()` or similar
2. Verify if using combo orders (BAG secType)
3. If using separate orders, add rollback logic

**Reference**: [yatws OptionsStrategyBuilder](https://docs.rs/yatws/latest/yatws/) provides examples of building multi-leg strategies

### Potential Enhancement: Options Strategy Builder

Consider adding a strategy builder similar to yatws:

```cpp
class BoxSpreadBuilder {
public:
    BoxSpreadBuilder& with_underlying(const std::string& symbol);
    BoxSpreadBuilder& with_expiry(const std::string& expiry);
    BoxSpreadBuilder& with_strikes(double lower_strike, double upper_strike);
    BoxSpreadBuilder& with_quantity(int quantity);
    std::pair<Contract, Order> build() const;
};
```

**Estimated Effort**: 2-3 hours (if not already implemented) + 2-3 hours for strategy builder (optional)

## Priority 7: Session Recording/Replay for Testing (LOW - Nice to Have)

**Status**: Not implemented

**Reference**: [yatws session recording](https://docs.rs/yatws/latest/yatws/) allows recording TWS interactions to SQLite for replay

### Implementation Needed

1. **Session Recording**
   - Record all TWS API requests/responses to SQLite database
   - Store timestamps, request IDs, contract details, order details
   - Enable/disable via configuration

2. **Session Replay**
   - Replay recorded sessions for testing
   - Useful for debugging without live TWS connection
   - Test order logic with historical data

3. **Benefits**
   - Test without live TWS connection
   - Debug order placement logic
   - Reproduce issues with recorded data
   - Backtest strategies with real API interactions

**Estimated Effort**: 4-6 hours (optional, but valuable for testing)

## Priority 8: Type Safety Improvements (MEDIUM)

**Status**: Partially implemented (uses some enums)

**Reference**: [yatws](https://docs.rs/yatws/latest/yatws/) uses strong type safety with enums instead of strings

### Improvements Needed

1. **Replace String Parameters with Enums**
   - Order types: "LMT", "MKT", "STP" → `OrderType` enum
   - Time in force: "DAY", "GTC" → `TimeInForce` enum
   - Action: "BUY", "SELL" → `OrderAction` enum
   - Security types: "STK", "OPT" → `SecType` enum

2. **Benefits**
   - Compile-time type checking
   - Prevents typos in order parameters
   - Better IDE autocomplete
   - Clearer API documentation

**Estimated Effort**: 2-3 hours (refactoring existing code)

## Priority 9: Manager-Based Architecture (LOW - Refactoring)

**Status**: Current implementation is monolithic

**Reference**: [yatws architecture](https://docs.rs/yatws/latest/yatws/) uses manager-based organization

### Potential Refactoring

Organize functionality into managers:

- `OrderManager` - Order operations
- `AccountManager` - Account data
- `MarketDataManager` - Market data
- `ReferenceDataManager` - Contract details
- `PositionManager` - Position tracking

**Note**: This is a larger refactoring effort. Consider only if codebase grows significantly.

**Estimated Effort**: 1-2 weeks (major refactoring)

## Implementation Order

### Week 1: Critical Safety (Priority 1-2)

1. Add try-catch to all callbacks (2-3 hours)
2. Enhance error handling (1-2 hours)
3. Test thoroughly

### Week 2: Reliability & Compliance (Priority 3-4)

1. Improve state synchronization (30 min)
2. Implement rate limiting (3-4 hours) ⭐ **NEW - High Priority**
3. Test reconnection scenarios
4. Test rate limiting

### Week 3: Compliance & Strategy (Priority 5-6)

1. Add order efficiency tracking (1-2 hours)
2. Review/implement atomic multi-leg execution (2-3 hours)
3. Consider options strategy builder (2-3 hours, optional)

### Week 4: Quality of Life (Priority 7-8, Optional)

1. Type safety improvements (2-3 hours)
2. Session recording/replay (4-6 hours, optional)

## Testing Checklist

After each improvement:

- [ ] Run unit tests: `ctest --test-dir build --output-on-failure`
- [ ] Test connection/disconnection
- [ ] Test reconnection scenarios
- [ ] Test error handling (simulate errors)
- [ ] Test with real TWS/Gateway (paper trading)
- [ ] Check logs for exceptions
- [ ] Verify no crashes from callback exceptions

## Files to Modify

1. `native/src/tws_client.cpp` - Main implementation
   - Add try-catch to ~15 callbacks
   - Enhance error() callback
   - Improve nextValidId() synchronization
   - Add rate limiting implementation ⭐ **NEW**
   - Add order efficiency tracking

2. `native/include/tws_client.h` - If needed for new members
   - Order efficiency tracker
   - Reconnection state tracking
   - Rate limiter configuration and status ⭐ **NEW**

3. `native/src/rate_limiter.cpp` - New file ⭐ **NEW**
   - Rate limiter implementation
   - Message rate tracking
   - Historical request tracking
   - Market data line tracking

4. `native/include/rate_limiter.h` - New file ⭐ **NEW**
   - RateLimiterConfig struct
   - RateLimiterStatus struct
   - RateLimiter class interface

## See Also

- [Code Improvements from NotebookLM](../research/analysis/CODE_IMPROVEMENTS_FROM_NOTEBOOKLM.md) - Detailed recommendations
- [TWS API Best Practices](../research/learnings/TWS_API_BEST_PRACTICES.md) - Current best practices
- [NotebookLM Status](../NOTEBOOKLM_STATUS.md) - Research source
- [yatws Documentation](https://docs.rs/yatws/latest/yatws/) - Rust TWS API library with rate limiting, session recording, and manager architecture
