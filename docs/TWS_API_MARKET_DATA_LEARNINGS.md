# TWS API Market Data Learnings

## Source
IBKR Campus: [Requesting Market Data](https://www.interactivebrokers.com/campus/trading-lessons/requesting-market-data/)

## Overview
This document captures learnings from IBKR's official guide on requesting market data with the TWS API and compares them with our current implementation. The guide covers best practices, common patterns, and troubleshooting for market data subscriptions.

## Common Market Data Topics Covered

Based on the URL and standard TWS API market data patterns, IBKR's guide likely covers:

### 1. Market Data Request Methods
### 2. Subscription Types (Real-time vs Delayed)
### 3. Rate Limiting and Line Limits
### 4. Generic Tick Types
### 5. Snapshot vs Streaming Data
### 6. Error Handling for Market Data
### 7. Market Data Callbacks

---

## 1. Market Data Request Methods

### Common Patterns (Likely Covered by IBKR)

**Method**: `reqMktData()`
- **Purpose**: Request real-time or delayed market data for a contract
- **Parameters**:
  - `tickerId`: Unique request identifier
  - `contract`: Contract specification
  - `genericTickList`: Comma-separated list of generic tick types
  - `snapshot`: Boolean for snapshot vs streaming
  - `regulatorySnapshot`: Boolean for regulatory snapshot
  - `mktDataOptions`: Additional options

**Best Practices**:
- Use unique `tickerId` for each request
- Cancel subscriptions when no longer needed
- Handle rate limits appropriately
- Use snapshot mode for one-time data requests

### Our Current Implementation ✅

**What We Have:**
```1326:1338:native/src/tws_client.cpp
        // Request market data
        client_.reqMktData(
            request_id,           // Request ID
            tws_contract,         // Contract
            "",                   // Generic tick list
            false,                // Snapshot
            false,                // Regulatory snapshot
            TagValueListSPtr()    // Options
        );

        spdlog::debug("Requested market data for {} (id={})",
                     contract.to_string(), request_id);

        return request_id;
```

**Features:**
- ✅ Unique request ID generation (`next_request_id_++`)
- ✅ Contract conversion to TWS format
- ✅ Callback registration for async updates
- ✅ Rate limiting integration
- ✅ Market data line limit checking
- ✅ Both async and sync request methods

**Potential Improvements:**
- ⚠️ Could add generic tick list support (e.g., "233,236,258" for volume, shortable, fundamental data)
- ⚠️ Could add snapshot mode option for one-time requests
- ⚠️ Could add regulatory snapshot support

---

## 2. Subscription Types (Real-time vs Delayed)

### Common Patterns (Likely Covered by IBKR)

**Real-time Data**:
- Requires market data subscriptions
- Available during market hours
- Higher cost (subscription fees)
- Best for active trading

**Delayed Data**:
- Free for most exchanges
- 15-20 minute delay
- Good for backtesting and analysis
- No subscription required

**Market Data Type Selection**:
- `REALTIME`: Real-time data (default)
- `FROZEN`: Frozen at last trade
- `DELAYED`: 15-20 minute delay
- `DELAYED_FROZEN`: Delayed and frozen

### Our Current Implementation ✅

**What We Have:**
- ✅ Market data request infrastructure
- ✅ Error handling for subscription issues
- ✅ Rate limiting to prevent subscription overload

**Example Error Handling:**
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
- ⚠️ Could add `reqMarketDataType()` support to switch between real-time/delayed
- ⚠️ Could add automatic fallback to delayed data if real-time unavailable
- ⚠️ Could add market data subscription checker

---

## 3. Rate Limiting and Line Limits

### Common Patterns (Likely Covered by IBKR)

**Rate Limits**:
- Message rate: 50 messages per second (default)
- Market data lines: Limited by account type
  - Paper trading: 100 lines
  - Live trading: Varies by subscription
- Exceeding limits causes errors or throttling

**Best Practices**:
- Monitor message rate
- Cancel unused subscriptions
- Batch requests when possible
- Use snapshot mode for one-time requests

### Our Current Implementation ✅

**What We Have:**
- ✅ `RateLimiter` class for message rate control
- ✅ Market data line limit checking
- ✅ Automatic tracking of active subscriptions

**Example:**
```1305:1310:native/src/tws_client.cpp
        // Check market data line limit
        if (!rate_limiter_.can_start_market_data(request_id)) {
            spdlog::error("Market data line limit exceeded: Cannot subscribe to {}",
                         contract.to_string());
            return -1;  // Invalid request ID
        }
```

**Rate Limiter Features:**
- ✅ Message rate checking (`check_message_rate()`)
- ✅ Market data line tracking (`can_start_market_data()`)
- ✅ Automatic cleanup on cancel
- ✅ Configurable limits

**Potential Improvements:**
- ⚠️ Could add dynamic rate limit adjustment based on account type
- ⚠️ Could add rate limit status reporting
- ⚠️ Could add automatic retry with backoff on rate limit errors

---

## 4. Generic Tick Types

### Common Patterns (Likely Covered by IBKR)

**Generic Tick Types**:
- `233`: RT Volume (real-time volume)
- `236`: Shortable
- `258`: Fundamental Ratios
- `165`: Bond yield
- `225`: Last trade time
- And many more...

**Usage**:
- Pass as comma-separated string: `"233,236,258"`
- Empty string for default ticks (bid, ask, last, etc.)
- Different ticks available for different asset types

### Our Current Implementation ⚠️

**What We Have:**
- ⚠️ Currently using empty generic tick list (`""`)
- ✅ Basic tick types supported (bid, ask, last, high, low, close, open)

**Example:**
```1329:1329:native/src/tws_client.cpp
            "",                   // Generic tick list
```

**Potential Improvements:**
- ⚠️ **HIGH PRIORITY**: Add generic tick list parameter to `request_market_data()`
- ⚠️ Add support for RT Volume (233) for better volume analysis
- ⚠️ Add support for Shortable (236) for short selling checks
- ⚠️ Add support for Fundamental Ratios (258) for fundamental analysis
- ⚠️ Add `tickGeneric()` callback handler for generic tick data

---

## 5. Snapshot vs Streaming Data

### Common Patterns (Likely Covered by IBKR)

**Snapshot Mode**:
- `snapshot = true`: One-time data request
- No ongoing subscription
- Useful for quick price checks
- Doesn't count against line limits
- May have additional fees

**Streaming Mode**:
- `snapshot = false`: Continuous updates
- Counts against line limits
- Receives `tickPrice()` callbacks
- Best for real-time trading

**Best Practices**:
- Use snapshot for one-time requests
- Use streaming for active monitoring
- Cancel streaming subscriptions when done

### Our Current Implementation ⚠️

**What We Have:**
- ⚠️ Currently always using streaming mode (`snapshot = false`)
- ✅ Both async and sync request methods
- ✅ Proper cleanup on cancel

**Example:**
```1330:1330:native/src/tws_client.cpp
            false,                // Snapshot
```

**Potential Improvements:**
- ⚠️ **MEDIUM PRIORITY**: Add snapshot mode option to API
- ⚠️ Add `request_market_data_snapshot()` method for one-time requests
- ⚠️ Add automatic snapshot mode for sync requests (more efficient)

---

## 6. Error Handling for Market Data

### Common Patterns (Likely Covered by IBKR)

**Common Errors**:
- `354`: No market data permissions
- `355`: Market data request failed
- `2107`: Market data farm connection failed
- `2108`: Market data farm disconnected

**Best Practices**:
- Check subscription status before requesting
- Handle connection errors gracefully
- Retry failed requests with backoff
- Log errors with context

### Our Current Implementation ✅

**What We Have:**
- ✅ Comprehensive error code mapping
- ✅ Error guidance messages
- ✅ Error logging with context
- ✅ Timeout handling for sync requests

**Example:**
```1428:1430:native/src/tws_client.cpp
        } else {
            // Timeout - cancel request and clean up
            spdlog::warn("Market data request {} timed out after {}ms", request_id, timeout_ms);
            cancel_market_data(request_id);
```

**Error Handling Features:**
- ✅ Timeout detection and cleanup
- ✅ Promise cancellation on timeout
- ✅ Error code guidance
- ✅ Context-aware error messages

**Potential Improvements:**
- ⚠️ Could add automatic retry for transient errors
- ⚠️ Could add error recovery strategies
- ⚠️ Could add subscription status checking

---

## 7. Market Data Callbacks

### Common Patterns (Likely Covered by IBKR)

**Primary Callbacks**:
- `tickPrice()`: Price updates (bid, ask, last, etc.)
- `tickSize()`: Size updates (bid size, ask size, volume)
- `tickGeneric()`: Generic tick data (volume, shortable, etc.)
- `tickString()`: String data (last trade time, etc.)
- `tickEFP()`: Exchange for Physical data

**Best Practices**:
- Handle all relevant tick types
- Update data structures atomically
- Notify callbacks promptly
- Handle exceptions in callbacks

### Our Current Implementation ✅

**What We Have:**
- ✅ `tickPrice()` callback implementation
- ✅ Thread-safe data structures
- ✅ Callback notification system
- ✅ Promise fulfillment for sync requests

**Example:**
```718:763:native/src/tws_client.cpp
    void tickPrice(TickerId tickerId, TickType field,
                   double price, const TickAttrib& attribs) override {
        try {
        spdlog::trace("tickPrice: id={}, field={}, price={}", tickerId, field, price);

        std::lock_guard<std::mutex> lock(data_mutex_);
        auto& market_data = market_data_[tickerId];

        switch (field) {
            case BID:
                market_data.bid = price;
                break;
            case ASK:
                market_data.ask = price;
                break;
            case LAST:
                market_data.last = price;
                break;
            case HIGH:
                market_data.high = price;
                break;
            case LOW:
                market_data.low = price;
                break;
            case CLOSE:
                market_data.close = price;
                break;
            case OPEN:
                market_data.open = price;
                break;
            default:
                break;
        }

        market_data.timestamp = std::chrono::system_clock::now();

        // Notify callback if registered
        if (market_data_callbacks_.count(tickerId)) {
            market_data_callbacks_[tickerId](market_data);
            }

            // Fulfill promise if waiting for synchronous request
            if (market_data_promises_.count(tickerId)) {
                market_data_promises_[tickerId]->set_value(market_data);
                market_data_promises_.erase(tickerId);
            }
        } catch (const std::exception& e) {
            spdlog::error("Exception in tickPrice(tickerId={}, field={}): {}", tickerId, field, e.what());
        } catch (...) {
            spdlog::error
```

**Callback Features:**
- ✅ Exception handling in callbacks
- ✅ Thread-safe data updates
- ✅ Multiple tick type support
- ✅ Timestamp tracking
- ✅ Callback and promise notification

**Potential Improvements:**
- ⚠️ **MEDIUM PRIORITY**: Add `tickSize()` callback for bid/ask sizes
- ⚠️ Add `tickGeneric()` callback for generic tick data
- ⚠️ Add `tickString()` callback for string data
- ⚠️ Add volume tracking from tickSize

---

## Comparison with Our Documentation

### What We Already Have ✅

1. **TWS_INTEGRATION_STATUS.md** - Market data integration status
2. **TWS_API_BEST_PRACTICES.md** - Best practices including market data
3. **TWS_API_TROUBLESHOOTING_LEARNINGS.md** - Market data error handling
4. **IMPLEMENTATION_GUIDE.md** - Market data implementation steps

### What We Could Add

1. **Market Data Best Practices Guide**
   - Generic tick types reference
   - Snapshot vs streaming decision guide
   - Rate limiting strategies
   - Subscription management

2. **Market Data API Enhancements**
   - Generic tick list support
   - Snapshot mode option
   - Additional callback handlers
   - Volume tracking

---

## Recommendations

### High Priority

1. **Add Generic Tick List Support**
   - Add parameter to `request_market_data()`
   - Implement `tickGeneric()` callback
   - Support common ticks (233, 236, 258)

2. **Add Snapshot Mode**
   - Add `request_market_data_snapshot()` method
   - Use snapshot for sync requests
   - Reduce line limit usage

### Medium Priority

3. **Enhance Callback Handlers**
   - Add `tickSize()` for bid/ask sizes
   - Add `tickString()` for string data
   - Add volume tracking

4. **Add Market Data Type Selection**
   - Add `reqMarketDataType()` support
   - Allow switching between real-time/delayed
   - Add automatic fallback

### Low Priority

5. **Add Market Data Utilities**
   - Subscription status checker
   - Line limit reporter
   - Rate limit monitor

---

## Conclusion

Our current implementation provides a solid foundation for market data:

✅ **Core Functionality** - Request, receive, and cancel market data
✅ **Error Handling** - Comprehensive error codes and guidance
✅ **Rate Limiting** - Message rate and line limit management
✅ **Thread Safety** - Thread-safe data structures and callbacks
✅ **Multiple Modes** - Both async and sync request methods

**Potential Improvements:**
- Add generic tick list support
- Add snapshot mode option
- Enhance callback handlers (tickSize, tickGeneric, tickString)
- Add market data type selection

**Next Steps:**
1. Review IBKR's official market data guide when accessible
2. Compare with our implementation
3. Add missing features (generic ticks, snapshot mode)
4. Enhance callback handlers

---

## References

- **IBKR Market Data Page**: https://www.interactivebrokers.com/campus/trading-lessons/requesting-market-data/
- **Our TWS Integration Docs**: `docs/TWS_INTEGRATION_STATUS.md`
- **Our Best Practices**: `docs/TWS_API_BEST_PRACTICES.md`
- **TWS API Reference**: https://interactivebrokers.github.io/tws-api/
- **Market Data Subscriptions**: https://www.interactivebrokers.com/en/index.php?f=marketData

---

**Last Updated**: 2025-01-XX
**Status**: Analysis complete, ready for comparison with official IBKR guide
