# 5-Day Paper Trading Validation Plan

**Date**: 2025-11-17
**Status**: Active Validation Plan
**Purpose**: Comprehensive validation of box spread trading system in paper trading environment before production deployment

---

## Overview

This plan provides a structured 5-day validation process to ensure the box spread trading system is production-ready. All testing will be performed in Interactive Brokers paper trading environment (port 7497) to avoid any real financial risk.

**Prerequisites**:

- TWS or IB Gateway running with API enabled
- Paper trading account configured
- API settings: "Enable ActiveX and Socket Clients" checked
- IP address 127.0.0.1 trusted
- Market data subscriptions active (for option chains)

---

## Day 1: Basic Functionality & Setup

### Objectives

- Verify system can connect to paper trading
- Validate basic market data subscription
- Confirm opportunity detection works
- Test dry-run mode

### Test Checklist

#### 1.1 Connection & Configuration

- [ ] Connect to TWS paper trading (port 7497)
- [ ] Verify connection state: `Connected`
- [ ] Confirm account information retrieved
- [ ] Verify paper trading mode detected correctly
- [ ] Test configuration loading and validation

#### 1.2 Market Data Subscription

- [ ] Subscribe to SPY option chain
- [ ] Verify market data callbacks received
- [ ] Check bid/ask prices are valid (> 0)
- [ ] Verify option chain structure (expiries, strikes)
- [ ] Test multiple simultaneous subscriptions

#### 1.3 Opportunity Detection

- [ ] Run opportunity scan for SPY
- [ ] Find at least 5 valid box spread opportunities
- [ ] Verify all opportunities meet criteria:
  - [ ] `min_arbitrage_profit` threshold met
  - [ ] `min_roi_percent` threshold met
  - [ ] `min_days_to_expiry` and `max_days_to_expiry` within range
  - [ ] Bid/ask spreads within `max_bid_ask_spread` limit
  - [ ] All 4 legs have valid market data

- [ ] Verify opportunities sorted by profitability

#### 1.4 Dry-Run Mode

- [ ] Enable dry-run mode
- [ ] Place 1 box spread order in dry-run
- [ ] Verify order submission logged (not executed)
- [ ] Confirm no actual orders placed in TWS
- [ ] Verify order tracking works in dry-run

### Success Criteria

- ✅ System connects to paper trading successfully
- ✅ Market data subscriptions working
- ✅ At least 5 valid opportunities found
- ✅ Dry-run mode prevents actual order execution
- ✅ No exceptions or crashes

### Documentation

- Record connection details (port, client ID)
- Log sample opportunities found
- Screenshot TWS connection status
- Document any configuration issues

---

## Day 2: Execution Tests

### Objectives

- Test actual order placement (live paper trading)
- Verify order fills and position tracking
- Test P&L calculation accuracy
- Validate position reconciliation

### Test Checklist

#### 2.1 Order Placement

- [ ] Disable dry-run mode
- [ ] Select best opportunity from Day 1 results
- [ ] Place box spread with individual orders (4 legs)
- [ ] Monitor order submission in TWS
- [ ] Verify all 4 orders appear in TWS order window
- [ ] Check order IDs match system tracking

#### 2.2 Order Fills

- [ ] Monitor all 4 legs for fills
- [ ] Verify fill prices match market data (within bid/ask)
- [ ] Check fill quantities (should be 1 contract per leg)
- [ ] Confirm fill timestamps recorded
- [ ] Verify order status transitions: Pending → Submitted → Filled

#### 2.3 Position Tracking

- [ ] Verify positions appear in system after fills
- [ ] Check position quantities match expected:
  - [ ] Long call: +1
  - [ ] Short call: -1
  - [ ] Long put: +1
  - [ ] Short put: -1

- [ ] Verify position prices (average fill price)
- [ ] Compare system positions with TWS positions
- [ ] Test position reconciliation (should match exactly)

#### 2.4 P&L Calculation

- [ ] Calculate unrealized P&L for box spread
- [ ] Verify P&L matches expected (theoretical value - net debit)
- [ ] Check P&L updates with market data changes
- [ ] Verify P&L accuracy (within bid/ask spread tolerance)

#### 2.5 Position Closure

- [ ] Close position (reverse box spread)
- [ ] Place opposite orders for all 4 legs
- [ ] Monitor fills for closing orders
- [ ] Verify positions reduced to zero
- [ ] Calculate realized P&L
- [ ] Compare realized P&L with expected

### Success Criteria

- ✅ All 4 orders placed successfully
- ✅ All orders filled within reasonable time
- ✅ Positions match TWS exactly
- ✅ P&L calculations accurate (within $0.10 tolerance)
- ✅ Position closure successful

### Documentation

- Screenshot TWS order window showing all 4 orders
- Log fill prices and timestamps
- Record position quantities and prices
- Document P&L calculations
- Screenshot final position reconciliation

---

## Day 3: Edge Cases & Error Handling

### Objectives

- Test partial fill scenarios
- Verify rollback logic
- Test low-liquidity filtering
- Validate stale data rejection
- Test rate limiting

### Test Checklist

#### 3.1 Partial Fill Scenario

- [ ] Place box spread order
- [ ] Manually cancel 1 leg in TWS (simulate partial fill)
- [ ] Verify system detects partial fill
- [ ] Check rollback logic triggers
- [ ] Confirm remaining 3 orders cancelled
- [ ] Verify no orphaned positions
- [ ] Test with different leg cancellations (call vs put)

#### 3.2 Low-Liquidity Filtering

- [ ] Identify low-liquidity options (low volume, low OI)
- [ ] Run opportunity scan
- [ ] Verify low-liquidity opportunities filtered out
- [ ] Check `min_volume` and `min_open_interest` thresholds
- [ ] Confirm no orders placed for filtered opportunities

#### 3.3 Stale Data Rejection

- [ ] Simulate stale market data (delay updates)
- [ ] Verify system detects stale data
- [ ] Confirm stale opportunities rejected
- [ ] Test timestamp validation
- [ ] Verify fresh data still accepted

#### 3.4 Rate Limiting

- [ ] Trigger rate limiter (burst 100+ requests)
- [ ] Verify rate limiter activates
- [ ] Check requests are throttled appropriately
- [ ] Confirm no requests dropped (queued instead)
- [ ] Verify system recovers after rate limit period
- [ ] Test market data line limits

#### 3.5 Error Handling

- [ ] Test invalid contract rejection
- [ ] Verify error messages are clear and actionable
- [ ] Test connection loss during order placement
- [ ] Verify error recovery (reconnection)
- [ ] Check error logging (all errors logged)

### Success Criteria

- ✅ Rollback logic works correctly
- ✅ Low-liquidity opportunities filtered
- ✅ Stale data rejected
- ✅ Rate limiting prevents violations
- ✅ All errors handled gracefully

### Documentation

- Record partial fill scenarios and outcomes
- Log rate limiter activations
- Document error messages and recovery
- Screenshot error handling in action

---

## Day 4: Reconnection Resilience

### Objectives

- Test system behavior during network interruptions
- Verify state synchronization after reconnection
- Test order recovery
- Validate position recovery

### Test Checklist

#### 4.1 Connection Loss During Operation

- [ ] Start system with active positions
- [ ] Place new box spread order
- [ ] Disconnect TWS/Gateway (simulate network loss)
- [ ] Verify system detects disconnection
- [ ] Check error handling (no crash)
- [ ] Verify auto-reconnect attempts (if enabled)

#### 4.2 Reconnection & State Sync

- [ ] Reconnect TWS/Gateway
- [ ] Verify system reconnects automatically
- [ ] Check connection state: `Connected`
- [ ] Verify state synchronization:
  - [ ] Positions recovered
  - [ ] Open orders recovered
  - [ ] Market data subscriptions restored

- [ ] Confirm no duplicate orders placed

#### 4.3 Order Recovery

- [ ] Verify pending orders recovered after reconnection
- [ ] Check order statuses are accurate
- [ ] Test order cancellation after reconnection
- [ ] Verify order tracking continues correctly

#### 4.4 Position Recovery

- [ ] Verify positions match TWS after reconnection
- [ ] Check position quantities accurate
- [ ] Confirm position prices correct
- [ ] Test position updates continue after reconnection

#### 4.5 Extended Disconnection

- [ ] Test 5-minute disconnection
- [ ] Test 30-minute disconnection
- [ ] Verify system handles extended outages
- [ ] Check reconnection after extended outage
- [ ] Validate state recovery accuracy

### Success Criteria

- ✅ System handles disconnection gracefully
- ✅ Auto-reconnect works (if enabled)
- ✅ State synchronization accurate
- ✅ No duplicate orders or positions
- ✅ System recovers from extended outages

### Documentation

- Record disconnection scenarios
- Log reconnection times
- Document state synchronization results
- Screenshot reconnection process

---

## Day 5: Multi-Symbol Testing & Extended Operation

### Objectives

- Test with multiple underlying symbols
- Run extended operation (8+ hours)
- Monitor for exceptions and memory leaks
- Validate efficiency ratio tracking
- Test auto-discovery across symbols

### Test Checklist

#### 5.1 Multi-Symbol Testing

- [ ] Test with SPY (S&P 500 ETF)
- [ ] Test with QQQ (Nasdaq 100 ETF)
- [ ] Test with IWM (Russell 2000 ETF)
- [ ] Verify opportunity detection for each symbol
- [ ] Test simultaneous scanning across symbols
- [ ] Verify no symbol-specific issues

#### 5.2 Extended Operation

- [ ] Run system for 8+ hours continuously
- [ ] Monitor for exceptions (should be zero)
- [ ] Check for memory leaks (memory usage stable)
- [ ] Verify CPU usage reasonable (< 10% average)
- [ ] Test overnight operation (if possible)
- [ ] Monitor log file sizes (rotation working)

#### 5.3 Efficiency Ratio Tracking

- [ ] Verify efficiency ratio calculated correctly
- [ ] Check efficiency ratio > 5% (if executing)
- [ ] Monitor efficiency ratio over time
- [ ] Test efficiency ratio alerts (if < threshold)
- [ ] Document efficiency ratio trends

#### 5.4 Auto-Discovery

- [ ] Enable auto-discovery mode
- [ ] Verify system scans multiple symbols automatically
- [ ] Check opportunity ranking across symbols
- [ ] Test best opportunity selection
- [ ] Verify no symbol bias

#### 5.5 Performance Metrics

- [ ] Measure order placement latency
- [ ] Track market data update frequency
- [ ] Monitor system resource usage
- [ ] Record throughput (opportunities/hour)
- [ ] Document performance baselines

### Success Criteria

- ✅ System runs 8+ hours without crashes
- ✅ No memory leaks detected
- ✅ Efficiency ratio > 5% (if executing)
- ✅ All symbols work correctly
- ✅ Performance metrics acceptable

### Documentation

- Record 8-hour operation log
- Document efficiency ratio trends
- Log performance metrics
- Screenshot extended operation dashboard
- Create performance baseline report

---

## Validation Log Template

For each day, maintain a detailed log:

```markdown

## Day X: [Day Name] - [Date]

### Test Results
- Test 1: [Pass/Fail] - [Notes]
- Test 2: [Pass/Fail] - [Notes]
- ...

### Issues Found
- Issue 1: [Description] - [Severity] - [Status]
- Issue 2: [Description] - [Severity] - [Status]
- ...

### Metrics
- Opportunities Found: [count]
- Orders Placed: [count]
- Orders Filled: [count]
- Efficiency Ratio: [percentage]
- Exceptions: [count]
- Memory Usage: [MB]
- CPU Usage: [percentage]

### Screenshots
- [Screenshot 1]: [Description]
- [Screenshot 2]: [Description]
- ...
```

---

## Success Criteria Summary

**Overall Validation Success**:

- ✅ All Day 1-5 tests pass
- ✅ Zero unhandled exceptions
- ✅ Zero system crashes
- ✅ Order efficiency ratio > 5% (if executing)
- ✅ No partial fills without rollback
- ✅ Positions reconcile with TWS exactly
- ✅ P&L matches expected (within tolerance)
- ✅ System runs 8+ hours without issues
- ✅ All error scenarios handled gracefully

**Production Readiness Checklist**:

- [ ] All integration tests pass
- [ ] Paper trading validation complete
- [ ] Error handling verified
- [ ] Monitoring & alerts configured
- [ ] Configuration validation comprehensive
- [ ] Documentation complete
- [ ] Performance baselines established

---

## Issue Tracking

Use this template for tracking issues found during validation:

| Issue ID | Description | Severity | Day Found | Status | Resolution |
|----------|-------------|----------|-----------|--------|------------|
| VAL-001 | [Description] | High/Medium/Low | Day X | Open/Fixed | [Notes] |

**Severity Levels**:

- **High**: Blocks production deployment
- **Medium**: Should fix before production
- **Low**: Nice to have, can defer

---

## Next Steps After Validation

1. **Review Results**: Analyze all validation logs and metrics
2. **Fix Issues**: Address all High and Medium severity issues
3. **Re-test**: Re-run failed test scenarios after fixes
4. **Documentation**: Update production deployment guide
5. **Sign-off**: Get approval for production deployment

---

## References

- `docs/MERGED_ACTION_PLAN.md` - Source of validation requirements
- `docs/TESTING_STRATEGY.md` - Comprehensive testing strategy
- `native/tests/test_tws_integration.cpp` - Integration tests
- `native/tests/test_market_data_integration.cpp` - Market data tests
- `native/tests/test_box_spread_e2e.cpp` - End-to-end tests

---

**Document Status**: ✅ Complete - Ready for validation execution
