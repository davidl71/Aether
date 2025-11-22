# CppTrader Integration Points

**Date**: 2025-01-27
**Purpose**: Specific code locations and integration points for CppTrader migration

---

## Current Architecture

### 1. TWS Client Market Data Handling

**Location**: `native/src/tws_client.cpp`

**Current Implementation** (lines 872-957):

- `tickPrice()`: Updates simple `market_data_` map with price fields (BID, ASK, LAST, etc.)
- `tickSize()`: Updates size fields (BID_SIZE, ASK_SIZE, LAST_SIZE, VOLUME)
- Simple map-based storage: `std::map<TickerId, types::MarketData> market_data_`
- No order book depth management
- No market depth (level 2) data

**Integration Point**:

```cpp
// Line 883-910: tickPrice() callback
void tickPrice(TickerId tickerId, TickType field,
               double price, const TickAttrib& attribs) override {
    // REPLACE: Simple map update
    // WITH: OrderBookManager::update_tick_price()

    std::lock_guard<std::mutex> lock(data_mutex_);
    auto& market_data = market_data_[tickerId];
    // ... update market_data fields ...

    // NEW: Update order book
    std::string symbol = get_symbol_for_ticker_id(tickerId);
    order_book_manager_->update_tick_price(symbol, static_cast<int>(field), price);

    // Get updated market data from order book
    auto updated_data = order_book_manager_->get_market_data(symbol);
    if (updated_data && market_data_callbacks_.count(tickerId)) {
        market_data_callbacks_[tickerId](*updated_data);
    }
}
```

### 2. Market Data Structure

**Location**: `native/include/types.h`

**Current Structure** (lines 164-196):

- Simple `MarketData` struct with bid/ask/last prices
- No order book depth information
- No level 2 data

**Enhancement Needed**:

- Add order book depth methods
- Add level 2 data access
- Maintain backward compatibility

### 3. Python Market Data Handler

**Location**: `python/integration/market_data_handler.py`

**Current Implementation**:

- Converts nautilus_trader events to C++ `MarketData` format
- Data quality validation (stale data, spread thresholds)
- Callback-based event handling
- QuestDB integration

**Migration Target**:

- Replace with C++ `OrderBookManager`
- Move validation to C++ `MarketDataValidator`
- Maintain same callback interface for compatibility

### 4. Box Spread Calculator

**Location**: `native/include/box_spread_calc.h` (assumed)

**Enhancement Needed**:

- Use order book depth for pricing
- Consider level 2 data for better execution estimates
- Add depth-based opportunity detection

---

## Integration Points Summary

### Phase 1: Order Book Manager (Foundation)

1. **Create `native/include/order_book_manager.h`**
   - Wrapper around CppTrader `MarketManager`
   - Symbol ID mapping (TWS tickerId → CppTrader symbol ID)
   - Tick update methods
   - Market data extraction methods

2. **Create `native/src/order_book_manager.cpp`**
   - Implementation of OrderBookManager
   - TWS TickType → CppTrader order book update conversion
   - Order book depth extraction

3. **Modify `native/include/tws_client.h`**
   - Add `OrderBookManager` member
   - Optional: Add order book depth query methods

4. **Modify `native/src/tws_client.cpp`**
   - Replace `market_data_` map updates with `OrderBookManager` calls
   - Update `tickPrice()` and `tickSize()` methods
   - Maintain callback interface compatibility

### Phase 2: Data Validation (Migration)

1. **Create `native/include/market_data_validator.h`**
   - Port validation logic from Python
   - Stale data detection
   - Spread threshold validation

2. **Create `native/src/market_data_validator.cpp`**
   - Implementation of validation logic

3. **Update `OrderBookManager`**
   - Integrate validation before callbacks
   - Configurable validation thresholds

### Phase 3: Box Spread Enhancement

1. **Enhance `native/include/box_spread_calc.h`**
   - Add order book depth parameters
   - Depth-based opportunity detection methods

2. **Update box spread calculations**
   - Use order book depth for pricing
   - Consider level 2 data for execution estimates

### Phase 4: Python Replacement

1. **Deprecate `python/integration/market_data_handler.py`**
    - Mark as deprecated
    - Document migration path
    - Maintain for backward compatibility if needed

2. **Update Python bindings** (if needed)
    - Expose `OrderBookManager` to Python
    - Maintain API compatibility

---

## Specific Code Locations

### TWS Client

**File**: `native/src/tws_client.cpp`

| Line Range | Current Code | Integration Action |
|------------|--------------|-------------------|
| 883-923 | `tickPrice()` - map update | Replace with `OrderBookManager::update_tick_price()` |
| 931-957 | `tickSize()` - map update | Replace with `OrderBookManager::update_tick_size()` |
| 884 | `std::map<TickerId, types::MarketData> market_data_` | Replace with `std::unique_ptr<OrderBookManager> order_book_manager_` |

**File**: `native/include/tws_client.h`

| Line | Current Code | Integration Action |
|------|--------------|-------------------|
| 21 | `using MarketDataCallback = ...` | Keep as-is (maintain compatibility) |
| 68-69 | `request_market_data()` | May need to add order book depth option |

### Market Data Structure

**File**: `native/include/types.h`

| Line Range | Current Code | Integration Action |
|------------|--------------|-------------------|
| 164-196 | `struct MarketData` | Add order book depth access methods (non-breaking) |

### Python Integration

**File**: `python/integration/market_data_handler.py`

| Line Range | Current Code | Migration Action |
|------------|--------------|------------------|
| 48-69 | `register_callback()` | Deprecate, move to C++ |
| 71-106 | `on_quote_tick()` | Replace with C++ `OrderBookManager` |
| 133-214 | `_convert_quote_tick()` | Port validation to C++ `MarketDataValidator` |

---

## Migration Checklist

### Phase 1: Foundation

- [ ] Add CppTrader as CMake dependency
- [ ] Create `native/include/order_book_manager.h`
- [ ] Create `native/src/order_book_manager.cpp`
- [ ] Implement symbol ID mapping
- [ ] Implement tick update methods
- [ ] Unit tests for OrderBookManager

### Phase 2: TWS Integration

- [ ] Modify `TWSClient` to use `OrderBookManager`
- [ ] Update `tickPrice()` to use order book
- [ ] Update `tickSize()` to use order book
- [ ] Integration tests with TWS mock data
- [ ] Performance benchmarks

### Phase 3: Validation Migration

- [ ] Create `native/include/market_data_validator.h`
- [ ] Create `native/src/market_data_validator.cpp`
- [ ] Port validation logic from Python
- [ ] Integrate validation into OrderBookManager
- [ ] Unit tests for validation

### Phase 4: Box Spread Enhancement

- [ ] Enhance `box_spread_calc.h` with order book depth
- [ ] Update box spread calculations
- [ ] Integration tests with order book depth
- [ ] Performance comparison

### Phase 5: Python Replacement

- [ ] Mark Python handler as deprecated
- [ ] Update documentation
- [ ] Migration guide for Python users
- [ ] Backward compatibility tests

---

## Performance Targets

- **Latency**: <100μs tick → callback (from current ~10μs)
- **Throughput**: >100K ticks/second (from current ~10K ticks/second)
- **Order Book Depth**: Support 100+ levels per side
- **Memory**: <1MB per order book (pre-allocated)

---

## Risk Mitigation

1. **Backward Compatibility**: Maintain existing `MarketData` struct and callback interface
2. **Gradual Migration**: Make CppTrader optional via CMake flag (`ENABLE_CPPTRADER`)
3. **Fallback**: Keep simple map implementation if CppTrader causes issues
4. **Testing**: Comprehensive unit and integration tests before migration

---

## Next Steps

1. Review this integration points document
2. Set up CppTrader submodule/git dependency
3. Start Phase 1 implementation (OrderBookManager)
4. Iterate on integration with feedback
