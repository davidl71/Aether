# Protocol Buffers Migration Plan

**Date**: 2025-01-27  
**Status**: Future Enhancement (Not Required)  
**TWS API Version**: 10.40.01+

## Overview

TWS API 10.40.01 provides full Protocol Buffers support for all requests/responses. This document outlines a migration plan from classic callbacks to protobuf callbacks for improved performance.

**Important**: This migration is **optional**. The classic API continues to work perfectly. Only migrate if you need:
- Better performance (faster serialization/deserialization)
- More efficient network usage
- Future-proofing for new TWS API features

---

## Current Implementation

### Classic API (Current)
```cpp
class TWSClient::Impl : public DefaultEWrapper {
    void tickPrice(TickerId tickerId, TickType field,
                   double price, const TickAttrib& attribs) override {
        // Handle classic callback
    }
    
    void orderStatus(OrderId orderId, const std::string& status,
                    Decimal filled, Decimal remaining,
                    double avgFillPrice, ...) override {
        // Handle classic callback
    }
};
```

### Protocol Buffers API (Future)
```cpp
class TWSClient::Impl : public DefaultEWrapper {
    void tickPriceProtoBuf(TickerId tickerId, const TickPriceProto& tickPrice) override {
        // Handle protobuf callback - more efficient
    }
    
    void orderStatusProtoBuf(OrderId orderId, const OrderStatusProto& orderStatus) override {
        // Handle protobuf callback - more efficient
    }
};
```

---

## Migration Strategy

### Phase 1: Research & Preparation (1-2 hours)

1. **Identify Protobuf Message Types**
   - Review generated `.pb.h` files in `native/third_party/tws-api/IBJts/source/cppclient/client/`
   - Map classic callbacks to protobuf equivalents
   - Document message structures

2. **Create Conversion Utilities**
   - Helper functions to convert protobuf messages to our internal types
   - Reuse existing conversion logic where possible

3. **Performance Baseline**
   - Measure current callback performance
   - Identify bottlenecks (if any)

### Phase 2: Dual Implementation (2-4 hours)

**Approach**: Implement protobuf callbacks alongside classic callbacks. Both will work, allowing gradual migration.

```cpp
class TWSClient::Impl : public DefaultEWrapper {
    // Classic callbacks (keep existing)
    void tickPrice(TickerId tickerId, TickType field,
                   double price, const TickAttrib& attribs) override {
        // Existing implementation
    }
    
    // Protobuf callbacks (new)
    void tickPriceProtoBuf(TickerId tickerId, const TickPriceProto& tickPrice) override {
        // Convert protobuf to internal types
        // Reuse existing processing logic
        handle_tick_price(tickerId, tickPrice.field(), tickPrice.price(), ...);
    }
    
private:
    // Shared processing logic
    void handle_tick_price(TickerId tickerId, TickType field, double price, ...) {
        // Common implementation
    }
};
```

**Benefits**:
- No breaking changes
- Can test protobuf callbacks in parallel
- Easy rollback if issues arise
- Gradual migration path

### Phase 3: Protobuf-First (Optional, 1-2 hours)

Once protobuf callbacks are proven stable:

1. **Make protobuf callbacks primary**
   - Keep classic callbacks as fallback
   - Log when classic callbacks are used (shouldn't happen)

2. **Performance Testing**
   - Compare message processing times
   - Measure network efficiency
   - Validate correctness

### Phase 4: Classic API Removal (Optional, 1 hour)

Only if you want to remove classic callbacks entirely:

1. Remove classic callback implementations
2. Keep only protobuf callbacks
3. Update documentation

**Note**: Not recommended - keeping both provides flexibility.

---

## Key Protobuf Callbacks to Migrate

### Priority 1: High-Frequency Callbacks

These are called most often and benefit most from protobuf:

1. **Market Data**
   - `tickPriceProtoBuf` → `tickPrice`
   - `tickSizeProtoBuf` → `tickSize`
   - `tickOptionComputationProtoBuf` → `tickOptionComputation`

2. **Order Management**
   - `orderStatusProtoBuf` → `orderStatus`
   - `openOrderProtoBuf` → `openOrder`
   - `execDetailsProtoBuf` → `execDetails`

3. **Error Handling**
   - `errorProtoBuf` → `error`

### Priority 2: Medium-Frequency Callbacks

4. **Positions & Account**
   - `positionProtoBuf` → `position`
   - `updateAccountValueProtoBuf` → `updateAccountValue`
   - `updatePortfolioProtoBuf` → `updatePortfolio`

5. **Contract Data**
   - `contractDetailsProtoBuf` → `contractDetails`

### Priority 3: Low-Frequency Callbacks

6. **Historical Data**
   - `historicalDataProtoBuf` → `historicalData`

7. **Other Callbacks**
   - Migrate as needed based on usage

---

## Implementation Example

### Before (Classic API)

```cpp
void tickPrice(TickerId tickerId, TickType field,
               double price, const TickAttrib& attribs) override {
    std::lock_guard<std::mutex> lock(data_mutex_);
    auto& market_data = market_data_[tickerId];
    
    switch (field) {
        case BID:
            market_data.bid = price;
            break;
        case ASK:
            market_data.ask = price;
            break;
        // ... more cases
    }
    
    market_data.timestamp = std::chrono::system_clock::now();
    
    if (market_data_callbacks_.count(tickerId)) {
        market_data_callbacks_[tickerId](market_data);
    }
}
```

### After (Protobuf API)

```cpp
void tickPriceProtoBuf(TickerId tickerId, const TickPriceProto& tickPrice) override {
    // Extract data from protobuf message
    TickType field = static_cast<TickType>(tickPrice.field());
    double price = tickPrice.price();
    TickAttrib attribs;
    // Convert protobuf TickAttrib to classic TickAttrib if needed
    
    // Reuse existing processing logic
    handle_tick_price(tickerId, field, price, attribs);
}

private:
void handle_tick_price(TickerId tickerId, TickType field, 
                       double price, const TickAttrib& attribs) {
    std::lock_guard<std::mutex> lock(data_mutex_);
    auto& market_data = market_data_[tickerId];
    
    switch (field) {
        case BID:
            market_data.bid = price;
            break;
        case ASK:
            market_data.ask = price;
            break;
        // ... more cases
    }
    
    market_data.timestamp = std::chrono::system_clock::now();
    
    if (market_data_callbacks_.count(tickerId)) {
        market_data_callbacks_[tickerId](market_data);
    }
}
```

---

## Protobuf Message Types Reference

### Common Protobuf Types

Based on TWS API 10.40.01, protobuf messages are defined in:
- `native/third_party/tws-api/IBJts/source/cppclient/client/*.pb.h`

Key message types:
- `TickPriceProto` - Market data price updates
- `TickSizeProto` - Market data size updates
- `OrderStatusProto` - Order status updates
- `OpenOrderProto` - Open order details
- `ExecutionProto` - Execution details
- `ErrorProto` - Error messages
- `PositionProto` - Position updates
- `AccountValueProto` - Account value updates
- `ContractDetailsProto` - Contract details

### Finding Protobuf Definitions

```bash
# List all protobuf message types
find native/third_party/tws-api/IBJts/source/cppclient/client -name "*.pb.h" | xargs grep "class.*Proto" | head -20
```

---

## Testing Strategy

### Unit Tests

1. **Protobuf Conversion Tests**
   - Test protobuf → internal type conversion
   - Verify data integrity
   - Check edge cases

2. **Callback Tests**
   - Test protobuf callbacks receive data correctly
   - Verify same behavior as classic callbacks
   - Performance benchmarks

### Integration Tests

1. **Dual Mode Testing**
   - Run with both classic and protobuf callbacks
   - Compare outputs
   - Verify consistency

2. **Performance Testing**
   - Measure message processing latency
   - Compare network bandwidth usage
   - Validate improvements

---

## Rollback Plan

If issues arise:

1. **Immediate**: Disable protobuf callbacks, use classic only
2. **Configuration**: Add feature flag to enable/disable protobuf
3. **Gradual**: Migrate callbacks one at a time, not all at once

---

## Performance Expectations

### Expected Improvements

- **Message Processing**: 10-30% faster (protobuf is more efficient)
- **Network Usage**: 20-40% reduction (better compression)
- **Memory**: Slightly lower (protobuf messages are smaller)

### When to Migrate

Migrate if you experience:
- High message volume (1000+ messages/second)
- Network bandwidth constraints
- Latency-sensitive trading strategies
- Need for future TWS API features

**Don't migrate if**:
- Current performance is acceptable
- Classic API meets all needs
- Migration effort outweighs benefits

---

## Migration Checklist

### Preparation
- [ ] Review protobuf message definitions
- [ ] Create conversion utility functions
- [ ] Set up performance baseline measurements
- [ ] Plan testing strategy

### Implementation
- [ ] Implement protobuf callbacks for Priority 1 methods
- [ ] Add dual-mode support (classic + protobuf)
- [ ] Create shared processing logic
- [ ] Add comprehensive logging

### Testing
- [ ] Unit tests for protobuf conversions
- [ ] Integration tests with real TWS
- [ ] Performance benchmarks
- [ ] Stress testing with high message volume

### Deployment
- [ ] Feature flag for protobuf enable/disable
- [ ] Monitor for issues
- [ ] Gradual rollout (if needed)
- [ ] Documentation updates

---

## Code Locations

### Files to Modify

1. **`native/src/tws_client.cpp`**
   - Add protobuf callback implementations
   - Create shared processing methods
   - Add conversion utilities

2. **`native/include/tws_client.h`** (if needed)
   - Add protobuf-specific configuration options

3. **`config/config.example.json`** (optional)
   - Add `use_protobuf` flag

### Protobuf Headers Location

```
native/third_party/tws-api/IBJts/source/cppclient/client/*.pb.h
```

---

## Resources

- **TWS API Documentation**: https://interactivebrokers.github.io/tws-api/
- **Protocol Buffers C++ Guide**: https://protobuf.dev/cpp/
- **TWS API Release Notes**: https://ibkrguides.com/releasenotes/prod-2025.htm
- **Current Implementation**: `native/src/tws_client.cpp`

---

## Decision Matrix

| Factor | Classic API | Protobuf API |
|--------|-------------|--------------|
| **Performance** | Good | Better (10-30%) |
| **Complexity** | Simple | More complex |
| **Maintenance** | Low | Medium |
| **Future-proof** | Yes | Yes (more) |
| **Network Usage** | Standard | Lower (20-40%) |
| **Migration Effort** | N/A | 4-8 hours |

**Recommendation**: Start with classic API. Migrate to protobuf only if performance becomes a bottleneck or you need specific protobuf-only features.

---

## Questions to Answer Before Migrating

1. **Do you have performance issues?**
   - If no → Don't migrate
   - If yes → Consider migration

2. **What's your message volume?**
   - < 100 msg/sec → Classic API is fine
   - > 1000 msg/sec → Protobuf may help

3. **Is network bandwidth a concern?**
   - If no → Classic API is fine
   - If yes → Protobuf reduces bandwidth

4. **Do you need new TWS API features?**
   - Some future features may be protobuf-only
   - Check TWS API release notes

---

**Last Updated**: 2025-01-27  
**Status**: Planning Document - Not Yet Implemented

