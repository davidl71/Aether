# TWS API Troubleshooting Learnings

## Source

IBKR Campus: [Diagnosing Issues and Troubleshooting with the TWS API](https://www.interactivebrokers.com/campus/trading-lessons/diagnosing-issues-and-troubleshooting-with-the-tws-api/)

## Overview

This document captures potential learnings from IBKR's official troubleshooting guide and compares them with our current implementation. While we cannot directly access the page content, we can infer common troubleshooting patterns and best practices that IBKR likely covers.

## Common TWS API Troubleshooting Areas

Based on the URL and standard TWS API issues, IBKR's troubleshooting guide likely covers:

### 1. Connection Issues

### 2. Authentication Problems

### 3. Market Data Errors

### 4. Order Rejections

### 5. Performance Issues

### 6. Error Code Interpretation

---

## 1. Connection Issues

### Common Problems (Likely Covered by IBKR)

**Problem**: Cannot connect to TWS/IB Gateway

- **Causes**:
  - TWS/IB Gateway not running
  - API settings not enabled
  - Wrong port number
  - Firewall blocking connection
  - IP address not trusted
  - Network connectivity issues

**Problem**: Connection drops frequently

- **Causes**:
  - Network instability
  - TWS/IB Gateway timeout settings
  - Too many API connections
  - Resource exhaustion

### Our Current Implementation ✅

**What We Have:**

- ✅ Parallel port checking (`is_port_open()`)
- ✅ Automatic port detection (TWS vs IB Gateway, paper vs live)
- ✅ Paper/live mismatch detection and warnings
- ✅ Connection state tracking (`ConnectionState` enum)
- ✅ Auto-reconnect with exponential backoff
- ✅ Connection health monitoring (every 30 seconds)
- ✅ Comprehensive error messages with guidance

**Example from our code:**

```142:181:native/src/tws_client.cpp
const std::unordered_map<int, std::string> kIbErrorGuidance = {
    // Connection errors (500-599)
    {502, "Connection rejected. Enable 'ActiveX and Socket Clients' in TWS Settings > API > Settings. Verify IP is trusted (127.0.0.1) and port is correct."},

    // System messages (1100-1999)
    {1100, "Connection lost. Check TWS/IB Gateway and internet connection. Auto-reconnect will be attempted if enabled."},
    {1101, "Market data connection restored. Confirm subscriptions are active."},
    {1102, "Order routing connection restored."},
```

**Potential Improvements:**

- ⚠️ Could add diagnostic command to test connectivity (`--test-connection`)
- ⚠️ Could add network latency monitoring
- ⚠️ Could add connection quality metrics (packet loss, jitter)

---

## 2. Authentication Problems

### Common Problems (Likely Covered by IBKR)

**Problem**: Two-factor authentication (2FA) timeouts

- **Causes**:
  - Code card authentication enabled
  - Mobile app not responding
  - Timeout too short

**Problem**: API access denied

- **Causes**:
  - API settings not enabled in TWS
  - IP address not in trusted list
  - Account restrictions

### Our Current Implementation ✅

**What We Have:**

```183:200:native/src/tws_client.cpp
const std::pair<const char*, const char*> kErrorPhraseGuidance[] = {
    {
        "code card authentication",
        "IB triggered code card authentication. Approve the 2FA challenge in IBKR Mobile or disable code card auth.",
    },
    {
        "two factor authentication request timed out",
        "Two-factor approval timed out. Re-initiate login and approve promptly on your IBKR Mobile device.",
    },
```

**Potential Improvements:**

- ⚠️ Could add pre-connection check for API settings
- ⚠️ Could add diagnostic mode that verifies TWS API settings
- ⚠️ Could add guidance for disabling 2FA for API-only access

---

## 3. Market Data Errors

### Common Problems (Likely Covered by IBKR)

**Problem**: No market data received

- **Causes**:
  - No market data subscriptions
  - Market closed
  - Invalid contract specification
  - Data farm connection issues
  - Rate limiting

**Problem**: Delayed or stale data

- **Causes**:
  - Using delayed data subscription
  - Data farm connection problems
  - Network latency

### Our Current Implementation ✅

**What We Have:**

- ✅ Market data request tracking
- ✅ Rate limiting (`RateLimiter`)
- ✅ Error guidance for market data errors
- ✅ Timeout handling for market data requests

**Example:**

```166:173:native/src/tws_client.cpp
    // Market data errors (350-399)
    {354, "No market data permissions. Ensure your IB account has the required data subscriptions."},
    {355, "Market data request failed. Check contract details and market data subscriptions."},

    // Market data farm messages (2100-2199)
    {2104, "Market data farm connection restored."},
    {2106, "Market data farm is connecting. Expect delayed quotes until established."},
    {2107, "Market data farm connection failed. Check IB network status dashboard."},
    {2108, "Market data farm disconnected. Quotes will pause until reconnection."},
```

**Potential Improvements:**

- ⚠️ Could add market data subscription checker
- ⚠️ Could add data quality monitoring (stale data detection)
- ⚠️ Could add automatic retry for failed market data requests

---

## 4. Order Rejections

### Common Problems (Likely Covered by IBKR)

**Problem**: Orders rejected with error codes

- **Causes**:
  - Invalid contract specification
  - Insufficient buying power
  - Order parameters invalid (price, size, TIF)
  - Account restrictions
  - Market closed
  - Position limits exceeded

**Problem**: Orders not executing

- **Causes**:
  - Limit price too far from market
  - Market closed
  - Insufficient liquidity
  - Order type not supported

### Our Current Implementation ✅

**What We Have:**

- ✅ Comprehensive error code guidance
- ✅ Order validation before submission
- ✅ Order status tracking
- ✅ Error messages with actionable guidance

**Example:**

```152:180:native/src/tws_client.cpp
    // Contract/Order errors (100-299)
    {162, "Order rejected - Invalid order ticket. Check order parameters, contract ID, and trading permissions."},
    {200, "Invalid contract definition. Verify symbol, expiry, right, strike, and exchange values."},
    {201, "Order rejected due to contract error. Confirm contract fields before resubmitting."},
    {202, "Order rejected by IB. Check order parameters, size limits, and account permissions."},
    {203, "Order rejected - Order cannot be executed. Check market hours, order type, and account permissions."},
    {204, "Order rejected - Order size exceeds position limit. Reduce order size or check account limits."},
    {205, "Order rejected - Order price is outside acceptable range. Adjust limit price."},

    // Validation errors (300-399)
    {321, "Server validation failed. Review price increments, exchange routing, and TIF."},
    {322, "Order rejected - Duplicate order ID. Use unique order IDs for each order."},
    {323, "Order rejected - Order cannot be cancelled. Order may already be filled or cancelled."},
```

**Potential Improvements:**

- ⚠️ Could add order pre-flight validation (check buying power, market hours)
- ⚠️ Could add automatic order adjustment suggestions
- ⚠️ Could add order rejection pattern analysis

---

## 5. Performance Issues

### Common Problems (Likely Covered by IBKR)

**Problem**: Slow response times

- **Causes**:
  - Too many concurrent requests
  - Rate limiting
  - Network latency
  - TWS/IB Gateway resource constraints

**Problem**: High memory/CPU usage

- **Causes**:
  - Memory leaks in application
  - Too many open connections
  - Excessive logging
  - Inefficient data structures

### Our Current Implementation ✅

**What We Have:**

- ✅ Rate limiting (`RateLimiter` class)
- ✅ Connection pooling (single connection)
- ✅ Efficient data structures (unordered_map for lookups)
- ✅ Configurable logging levels
- ✅ Connection health monitoring

**Potential Improvements:**

- ⚠️ Could add performance metrics (request latency, throughput)
- ⚠️ Could add memory usage monitoring
- ⚠️ Could add request batching for efficiency

---

## 6. Error Code Interpretation

### Common Problems (Likely Covered by IBKR)

**Problem**: Unclear error messages

- **Causes**:
  - Generic error codes without context
  - Missing error code documentation
  - Error codes that require account-specific knowledge

### Our Current Implementation ✅

**What We Have:**

- ✅ Comprehensive error code mapping (`kIbErrorGuidance`)
- ✅ Phrase-based error matching (`kErrorPhraseGuidance`)
- ✅ Actionable error messages with guidance
- ✅ Context-aware error logging

**Example:**

```142:200:native/src/tws_client.cpp
const std::unordered_map<int, std::string> kIbErrorGuidance = {
    // ... comprehensive error mappings ...
};

const std::pair<const char*, const char*> kErrorPhraseGuidance[] = {
    // ... phrase-based error matching ...
};
```

**Potential Improvements:**

- ⚠️ Could add error code lookup tool (`--error-code 502`)
- ⚠️ Could add error history tracking
- ⚠️ Could add error pattern analysis

---

## Diagnostic Tools We Could Add

Based on common troubleshooting needs, here are diagnostic tools we could implement:

### 1. Connection Diagnostic Tool

```bash
./ib_box_spread --diagnose-connection
```

**Checks:**

- TWS/IB Gateway running
- Port availability
- API settings enabled
- IP address trusted
- Network connectivity

### 2. Market Data Diagnostic Tool

```bash
./ib_box_spread --diagnose-market-data SPY
```

**Checks:**

- Market data subscriptions
- Contract validity
- Data farm connection
- Market hours

### 3. Order Validation Tool

```bash
./ib_box_spread --validate-order SPY 20250117 580 CALL BUY 1 2.50
```

**Checks:**

- Contract specification
- Buying power
- Order parameters
- Market hours
- Position limits

### 4. Error Code Lookup Tool

```bash
./ib_box_spread --error-code 502
```

**Shows:**

- Error code meaning
- Common causes
- Resolution steps
- Related error codes

---

## Comparison with Our Documentation

### What We Already Have ✅

1. **TWS_INTEGRATION_STATUS.md** - Comprehensive integration status
2. **TWS_API_BEST_PRACTICES.md** - Best practices and patterns
3. **TWS_API_CODE_EXAMPLES_LEARNINGS.md** - Learnings from code examples
4. **IMPLEMENTATION_GUIDE.md** - Step-by-step implementation guide
5. **INTEGRATION_TESTING.md** - Testing and troubleshooting guide

### What We Could Add

1. **TWS_API_TROUBLESHOOTING_GUIDE.md** - Comprehensive troubleshooting guide
   - Common issues and solutions
   - Diagnostic commands
   - Error code reference
   - Performance tuning

2. **Diagnostic Tools** - Command-line diagnostic utilities
   - Connection testing
   - Market data validation
   - Order pre-flight checks
   - Error code lookup

---

## Recommendations

### Stage 0: Validate Environment with Official Samples

Before digging into our code, run the official IB sample client (or `scmhub/ibapi` Go sample) against the same TWS/Gateway instance. If the sample also fails to reach `nextValidId`, the issue is with the environment (API settings, trusted IPs, stale sessions, etc.). If the sample succeeds but our client does not, focus on our initialization flow.

### High Priority

1. **Add Diagnostic Commands**
   - `--test-connection` - Test TWS connectivity
   - `--check-market-data SYMBOL` - Validate market data access
   - `--error-code CODE` - Lookup error code meaning

2. **Enhance Error Messages**
   - Add more context to error messages
   - Include suggested actions
   - Link to relevant documentation

### Medium Priority

1. **Add Pre-flight Checks**
   - Validate API settings before connection
   - Check market data subscriptions
   - Verify account permissions

2. **Add Performance Monitoring**
   - Request latency tracking
   - Throughput metrics
   - Memory usage monitoring

### Low Priority

1. **Add Diagnostic Mode**
   - Comprehensive system check
   - Configuration validation
   - Network connectivity test

---

## Conclusion

Our current implementation already covers most common troubleshooting scenarios:

✅ **Connection Management** - Comprehensive with auto-reconnect
✅ **Error Handling** - Detailed error codes with guidance
✅ **Market Data** - Rate limiting and error handling
✅ **Order Management** - Validation and error tracking
✅ **Documentation** - Multiple guides covering different aspects

**Potential Improvements:**

- Add diagnostic command-line tools
- Enhance error messages with more context
- Add pre-flight validation checks
- Add performance monitoring

**Next Steps:**

1. Review IBKR's official troubleshooting page when accessible
2. Compare with our implementation
3. Add any missing patterns or best practices
4. Implement diagnostic tools if needed

---

## Related Documentation

- **Market Data Learnings**: See `docs/TWS_API_MARKET_DATA_LEARNINGS.md` for market data-specific troubleshooting and best practices

## References

- **IBKR Troubleshooting Page**: <https://www.interactivebrokers.com/campus/trading-lessons/diagnosing-issues-and-troubleshooting-with-the-tws-api/>
- **IBKR Market Data Page**: <https://www.interactivebrokers.com/campus/trading-lessons/requesting-market-data/>
- **Our TWS Integration Docs**: `docs/TWS_INTEGRATION_STATUS.md`
- **Our Best Practices**: `docs/TWS_API_BEST_PRACTICES.md`
- **TWS API Reference**: <https://interactivebrokers.github.io/tws-api/>
- **IBKR Support**: 1-877-442-2757

---

**Last Updated**: 2025-01-XX
**Status**: Analysis complete, ready for comparison with official IBKR guide
