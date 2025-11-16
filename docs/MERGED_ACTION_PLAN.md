# Merged Action Plan (2025-01-16)
**Status**: Active
**Replaces**: ACTION_PLAN.md, CODE_IMPROVEMENTS_ACTION_PLAN.md (both deprecated as out of date)
**Based On**: CODE_VERIFICATION_REVIEW.md

---

## Executive Summary

Most core functionality is **already implemented**. This plan focuses on:
1. **Critical fixes** (exception handling, contract ID lookup)
2. **Testing and validation** (integration tests, paper trading)
3. **Production readiness** (monitoring, error handling enhancements)

**Timeline**: 2-3 weeks to production-ready

---

## Phase 1: Critical Fixes (Week 1, Days 1-2)

### Priority 1.1: Add Try-Catch to Active Callbacks ⚡ HIGH
**Status**: 🔴 Must Fix Before Production
**Effort**: 30 minutes
**Risk**: System crashes on market data exceptions

**Files**:
- `native/src/tws_client.cpp:998` - tickSize()
- `native/src/tws_client.cpp:1026` - tickOptionComputation()

**Implementation**:
```cpp
void TWSClient::Impl::tickSize(TickerId tickerId, TickType field, Decimal size) {
    try {
        // Existing implementation
    } catch (const std::exception& e) {
        spdlog::error("Exception in tickSize: {} (tickerId={}, field={})",
                     e.what(), tickerId, field);
    } catch (...) {
        spdlog::error("Unknown exception in tickSize (tickerId={}, field={})",
                     tickerId, field);
    }
}

void TWSClient::Impl::tickOptionComputation(TickerId tickerId, TickType tickType,
    int tickAttrib, double impliedVol, double delta, /* ... */) {
    try {
        // Existing implementation
    } catch (const std::exception& e) {
        spdlog::error("Exception in tickOptionComputation: {} (tickerId={}, type={})",
                     e.what(), tickerId, tickType);
    } catch (...) {
        spdlog::error("Unknown exception in tickOptionComputation (tickerId={}, type={})",
                     tickerId, tickType);
    }
}
```

**Success Criteria**:
- [ ] Both callbacks wrapped in try-catch
- [ ] Exceptions logged with context
- [ ] System continues running after exceptions
- [ ] Unit test triggers exception, verifies no crash

---

### Priority 1.2: Contract Details Lookup for Combo Orders ⚡ HIGH
**Status**: 🟡 Partially Complete (fallback works, combo order disabled)
**Effort**: 2-3 hours
**Benefit**: Atomic execution reduces partial fill risk

**Current State**: native/src/order_manager.cpp:191
```cpp
// TODO: Add contract details lookup to get conIds for combo orders
bool use_combo_order = false;  // Set to true when contract IDs are available
```

**Implementation Required**:
1. **Add contract details request method** to TWSClient
   ```cpp
   void TWSClient::request_contract_details(
       const types::OptionContract& contract,
       std::function<void(long conId)> callback);
   ```

2. **Implement contractDetails callback** in TWSClient::Impl
   ```cpp
   void contractDetails(int reqId, const ContractDetails& contractDetails) override {
       try {
           long conId = contractDetails.contract.conId;
           // Store or invoke callback with conId
       } catch (...) { /* ... */ }
   }
   ```

3. **Update place_box_spread** to request contract details first
   ```cpp
   // Request contract details for all 4 legs
   // When all conIds received, enable use_combo_order = true
   // Place combo order with real conIds
   ```

**Files to Modify**:
- `native/src/tws_client.cpp` - Add reqContractDetails, contractDetails callback
- `native/include/tws_client.h` - Add interface
- `native/src/order_manager.cpp` - Update place_box_spread logic

**Success Criteria**:
- [ ] Contract details requested for all 4 legs
- [ ] conIds stored and passed to combo order
- [ ] use_combo_order flag enabled when conIds available
- [ ] Combo order executes atomically in paper trading
- [ ] Fallback to individual orders if combo fails

**Testing**:
- Paper trade a box spread with combo order enabled
- Verify all-or-nothing execution
- Test fallback when combo order unavailable

---

## Phase 2: Testing & Validation (Week 1-2)

### Priority 2.1: Integration Tests 🧪 HIGH
**Status**: 🟡 Partial (unit tests exist, integration tests minimal)
**Effort**: 1 week
**See**: TESTING_STRATEGY.md for comprehensive plan

**Required Test Suites**:

1. **TWS Connection & Reconnection Tests**
   - Test initial connection
   - Test disconnection handling
   - Test reconnection with state sync
   - Test rate limiting during reconnection
   - **File**: `native/tests/test_tws_integration.cpp` (new)

2. **Market Data Pipeline Tests**
   - Subscribe to option chain
   - Verify market data updates
   - Test stale data detection
   - Test liquidity filtering
   - **File**: `native/tests/test_market_data_integration.cpp` (new)

3. **Box Spread End-to-End Tests**
   - Find opportunities in live option chain
   - Validate all 4 legs
   - Execute box spread (dry run)
   - Verify order placement
   - Test rollback on partial fill
   - **File**: `native/tests/test_box_spread_e2e.cpp` (new)

4. **Order Manager Integration Tests**
   - Place individual orders
   - Place combo orders (when implemented)
   - Test order cancellation
   - Test multi-leg tracking
   - **File**: Extend `native/tests/test_order_manager.cpp`

**Success Criteria**:
- [ ] All integration tests pass with mock TWS
- [ ] Connection/reconnection tests pass
- [ ] Market data pipeline validated
- [ ] End-to-end box spread execution works in dry-run
- [ ] CI/CD runs all tests on commit

---

### Priority 2.2: Paper Trading Validation 🎯 CRITICAL
**Status**: 🔴 Must Do Before Production
**Effort**: 3-5 days
**Risk**: Real money at stake

**Test Scenarios**:

1. **Basic Functionality** (Day 1)
   - [ ] Connect to paper trading account
   - [ ] Subscribe to SPY option chain
   - [ ] Find 5 valid box spread opportunities
   - [ ] Validate all opportunities meet criteria
   - [ ] Place 1 box spread in dry-run mode
   - [ ] Verify order submission (not execution)

2. **Execution Tests** (Day 2)
   - [ ] Place box spread with individual orders (live paper)
   - [ ] Monitor all 4 legs for fills
   - [ ] Verify positions match expected
   - [ ] Verify P&L tracking
   - [ ] Close position (reverse box spread)

3. **Edge Cases** (Day 3)
   - [ ] Test partial fill scenario (manually cancel 1 leg)
   - [ ] Verify rollback logic triggers
   - [ ] Test low-liquidity opportunities (should be filtered)
   - [ ] Test stale market data (should reject)
   - [ ] Test rate limiting (burst 100 requests)

4. **Reconnection Resilience** (Day 4)
   - [ ] Place box spread, disconnect TWS during execution
   - [ ] Reconnect and verify state sync
   - [ ] Verify open orders recovered
   - [ ] Verify positions recovered

5. **Multi-Symbol Testing** (Day 5)
   - [ ] Test with SPY, QQQ, IWM
   - [ ] Run for 8 hours with auto-discovery
   - [ ] Monitor for exceptions, crashes, memory leaks
   - [ ] Verify efficiency ratio tracking

**Documentation**:
- Record all test results in `docs/PAPER_TRADING_LOG.md`
- Include screenshots of TWS order window
- Log any errors or unexpected behavior

**Success Criteria**:
- [ ] All scenarios pass without crashes
- [ ] No unhandled exceptions
- [ ] Order efficiency ratio > 5% (if executing)
- [ ] No partial fills without rollback
- [ ] Positions reconcile with TWS
- [ ] P&L matches expected (within bid/ask spread)

---

## Phase 3: Production Readiness (Week 2-3)

### Priority 3.1: Enhanced Error Handling 🛡️ MEDIUM
**Status**: 🟡 Basic implementation exists
**Effort**: 1-2 hours
**File**: `native/src/tws_client.cpp`

**Expand Error Guidance Map**:

Add missing error codes to `kIbErrorGuidance`:
```cpp
static const std::unordered_map<int, std::string> kIbErrorGuidance = {
    // Connection errors
    {502, "Couldn't connect to TWS. Ensure TWS/Gateway is running on correct port."},
    {504, "Not connected. Check TWS connection."},
    {1100, "Connectivity lost. Auto-reconnecting..."},
    {1101, "Connectivity restored - data maintained."},
    {1102, "Connectivity restored - data lost. Re-requesting..."},
    {2110, "Connectivity between TWS and server broken. Data may be delayed."},

    // Market data errors
    {162, "Historical data request pacing violation. Rate limiter should prevent this."},
    {200, "No security definition found for request."},
    {354, "Subscription cancelled. Check market data permissions."},
    {10167, "Requested market data is not subscribed. Check data permissions."},

    // Order errors
    {201, "Order rejected - invalid contract."},
    {202, "Order cancelled."},
    {10148, "Order size exceeds account limits."},

    // Add more from: https://interactivebrokers.github.io/tws-api/message_codes.html
};
```

**Verify Error 1100 Auto-Reconnect**:
```cpp
if (errorCode == 1100) {
    connected_ = false;
    state_ = ConnectionState::Error;
    spdlog::error("Connectivity lost (1100). Auto-reconnect enabled: {}",
                 config_.auto_reconnect);
    if (config_.auto_reconnect) {
        attempt_reconnect_with_backoff();
    }
}
```

**Success Criteria**:
- [ ] 20+ error codes in guidance map
- [ ] Error 1100 triggers auto-reconnect
- [ ] All critical errors have actionable guidance
- [ ] Error context includes request ID when available

---

### Priority 3.2: Monitoring & Alerts 📊 MEDIUM
**Status**: 🔴 Not Implemented
**Effort**: 3-4 hours
**Purpose**: Production monitoring

**Add Health Check Endpoint** (if web interface exists):
```cpp
struct SystemHealth {
    bool tws_connected;
    int active_positions;
    int pending_orders;
    double efficiency_ratio;
    int rate_limiter_messages_per_sec;
    int rate_limiter_active_market_data;
    int rate_limiter_active_historical;
    std::string last_error;
    int error_count_last_hour;
};

SystemHealth get_system_health() const;
```

**Add Logging Enhancements**:
1. Structured logging with timestamps
2. Log rotation (daily, keep 30 days)
3. Separate error log for critical issues
4. Performance metrics logging (latency, throughput)

**Alert Triggers**:
- Disconnection from TWS
- Order efficiency ratio < 5% (already implemented)
- Rate limiter triggered (already logged)
- Unhandled exceptions in callbacks
- Position mismatch with TWS

**Files**:
- `native/src/tws_client.cpp` - Add health check method
- `native/src/ib_box_spread.cpp` - Add monitoring integration

**Success Criteria**:
- [ ] Health check returns current system state
- [ ] Logs rotate daily
- [ ] Critical errors trigger alerts (console/email/webhook)
- [ ] Metrics logged every 5 minutes

---

### Priority 3.3: Configuration Validation 🔧 LOW
**Status**: 🟡 Basic validation exists
**Effort**: 1 hour

**Add Configuration Sanity Checks**:
```cpp
bool validate_config(const config::StrategyParams& params) {
    if (params.min_roi <= 0) {
        spdlog::error("Invalid min_roi: {}. Must be > 0.", params.min_roi);
        return false;
    }
    if (params.max_position_size <= 0) {
        spdlog::error("Invalid max_position_size: {}. Must be > 0.", params.max_position_size);
        return false;
    }
    if (params.min_days_to_expiry >= params.max_days_to_expiry) {
        spdlog::error("Invalid DTE range: min={}, max={}. Min must be < max.",
                     params.min_days_to_expiry, params.max_days_to_expiry);
        return false;
    }
    // Add more validations...
    return true;
}
```

**Success Criteria**:
- [ ] All config parameters validated on startup
- [ ] Invalid config fails fast with clear error
- [ ] Example valid config in docs/

---

## Phase 4: Nice-to-Have Enhancements (Week 3+)

### Priority 4.1: Type Safety Improvements 🔒 LOW
**Status**: 🔴 Not Implemented (strings used for enums)
**Effort**: 2-3 hours
**Benefit**: Compile-time type safety, prevents typos

**Replace String Parameters with Enums**:

Currently:
```cpp
order.orderType = "LMT";
order.tif = "DAY";
order.action = "BUY";
contract.secType = "OPT";
```

After:
```cpp
enum class OrderType { Market, Limit, Stop, StopLimit };
enum class TimeInForce { Day, GTC, IOC, GTD };
enum class OrderAction { Buy, Sell };
enum class SecType { Stock, Option, Future, Forex, Index };

order.orderType = OrderType::Limit;
order.tif = TimeInForce::Day;
order.action = OrderAction::Buy;
contract.secType = SecType::Option;
```

**Files to Modify**:
- `native/include/types.h` - Add enums
- `native/src/tws_client.cpp` - Convert enums to strings for TWS API
- All files using order parameters

**Success Criteria**:
- [ ] Compile-time type checking
- [ ] No string typos possible
- [ ] All existing tests pass with new types

---

### Priority 4.2: Session Recording/Replay (Optional) 📹 LOW
**Status**: 🔴 Not Implemented
**Effort**: 4-6 hours
**Benefit**: Testing without live TWS, debugging

**Implementation**:
1. SQLite database to record all TWS API calls
2. Record: timestamp, request_id, method, parameters, response
3. Replay mode: Read from database instead of TWS
4. Use for regression testing, debugging

**Reference**: yatws (Rust TWS library) has this feature

**Success Criteria**:
- [ ] Record mode captures all TWS interactions
- [ ] Replay mode reproduces exact sequence
- [ ] Can test strategies with historical recordings

---

## Testing Strategy Summary

**See TESTING_STRATEGY.md for comprehensive details**

**Test Pyramid**:
1. **Unit Tests** (3,123 lines) - ✅ Already comprehensive
2. **Integration Tests** (NEW) - Test component interactions
3. **End-to-End Tests** (NEW) - Test full workflows
4. **Paper Trading** (CRITICAL) - Real TWS with fake money
5. **Production Monitoring** (NEW) - Live system health

**Timeline**:
- Week 1: Integration tests, paper trading basics
- Week 2: Paper trading edge cases, production prep
- Week 3: Extended paper trading, monitoring setup

---

## Implementation Checklist

### Week 1 (Days 1-2): Critical Fixes
- [ ] Add try-catch to tickSize and tickOptionComputation
- [ ] Implement contract details lookup
- [ ] Enable combo orders with contract IDs
- [ ] Test combo orders in paper trading

### Week 1 (Days 3-5): Basic Integration Tests
- [ ] Write TWS connection integration tests
- [ ] Write market data pipeline tests
- [ ] Write box spread E2E tests (dry-run)
- [ ] All integration tests passing

### Week 2 (Days 1-5): Paper Trading
- [ ] Basic functionality tests (Day 1)
- [ ] Execution tests (Day 2)
- [ ] Edge case tests (Day 3)
- [ ] Reconnection tests (Day 4)
- [ ] Multi-symbol 8-hour test (Day 5)
- [ ] Document all results

### Week 2 (Days 6-7): Production Prep
- [ ] Expand error guidance map
- [ ] Add monitoring/health checks
- [ ] Configuration validation
- [ ] Log rotation setup

### Week 3: Extended Testing & Monitoring
- [ ] Run 1 week of paper trading
- [ ] Monitor for memory leaks, crashes
- [ ] Tune parameters (ROI thresholds, liquidity minimums)
- [ ] Document production deployment procedure

---

## Success Criteria (Overall)

**Before Production**:
- [ ] All critical fixes complete (try-catch, contract IDs)
- [ ] All integration tests passing
- [ ] 1 week of stable paper trading (no crashes)
- [ ] Order efficiency ratio > 5%
- [ ] No partial fills without rollback
- [ ] Reconnection works reliably
- [ ] Error handling comprehensive
- [ ] Monitoring in place

**Production Deployment**:
- [ ] Start with small position sizes (1 contract)
- [ ] Monitor for 1 week
- [ ] Gradually increase position sizes
- [ ] Document all issues
- [ ] Have rollback plan ready

---

## Related Documentation

- **CODE_VERIFICATION_REVIEW.md** - Detailed analysis of current implementation
- **TESTING_STRATEGY.md** - Comprehensive testing plan
- **TWS_API_BEST_PRACTICES.md** - TWS API integration patterns
- **IBKRBOX_LEARNINGS.md** - Lessons from previous projects

---

## Notes

- Most core functionality already exists; focus on **testing and validation**
- Paper trading is **critical** before production
- Combo orders are **optional** but highly recommended for atomic execution
- Monitoring and alerts are essential for production stability

---

## Change Log

- 2025-01-16: Initial version based on code verification review
- Replaces: ACTION_PLAN.md (2025-01-27), CODE_IMPROVEMENTS_ACTION_PLAN.md
