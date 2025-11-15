# CppTrader Integration Plan

**Date**: 2025-01-27
**Source**: <https://github.com/chronoxor/CppTrader>
**Purpose**: Comprehensive integration plan for migrating market data processing and order book management from Python to C++ using CppTrader

---

## Executive Summary

CppTrader is a high-performance C++ library for building trading platforms with:

- **Ultra-fast matching engine** (millions of operations per second)
- **Order book processor** (9.7M+ messages/second throughput)
- **NASDAQ ITCH handler** (41M+ messages/second)
- **Cross-platform** (Linux, macOS, Windows)
- **MIT License** (permissive, production-ready)

This document outlines the migration strategy to:

1. Replace Python `MarketDataHandler` with C++ order book processor
2. Integrate CppTrader's order book management with TWS market data
3. Migrate market data processing from Python/Rust to native C++
4. Optimize box spread calculations with high-performance order books

---

## 1. Current Architecture Analysis

### 1.1 Current State

**Python Market Data Handler** (`python/integration/market_data_handler.py`):
- Converts nautilus_trader events to C++ `MarketData` format
- Validates data quality (stale data, spread thresholds)
- Callback-based event handling
- QuestDB integration for historical data

**Native C++ Market Data** (`native/include/types.h`):
- Simple `MarketData` struct with bid/ask/last prices
- No order book depth management
- No market depth (level 2) data
- Basic spread calculations

**TWS Client** (`native/src/tws_client.cpp`):
- Receives `tickPrice()` and `tickSize()` callbacks
- Stores basic market data in `market_data_` map
- No order book reconstruction
- No market depth management

**Rust Backend** (`agents/backend/`):
- Market data pipeline and ingestion
- State management
- Strategy signal generation

### 1.2 Integration Points

```text
┌─────────────────────────────────────────────────────────┐
│   TWS API (Interactive Brokers)                         │
│   - tickPrice() / tickSize() callbacks                  │
│   - Market depth data (level 2)                         │
└───────────────┬─────────────────────────────────────────┘
                │
        ┌───────▼────────────────────────────────┐
        │   TWS Client (C++)                     │
        │   - Receives tick updates              │
        │   - Current: Simple map storage        │
        │   - Target: CppTrader integration      │
        └───────┬────────────────────────────────┘
                │
        ┌───────▼────────────────────────────────┐
        │   CppTrader Order Book Processor       │
        │   - Market manager                     │
        │   - Order book reconstruction          │
        │   - Price level management             │
        │   - Best bid/ask tracking              │
        └───────┬────────────────────────────────┘
                │
        ┌───────▼────────────────────────────────┐
        │   Box Spread Calculator (C++)          │
        │   - Uses order book data               │
        │   - Calculates arbitrage opportunities │
        │   - High-performance pricing           │
        └───────┬────────────────────────────────┘
                │
        ┌───────▼────────────────────────────────┐
        │   Strategy & Execution                 │
        │   - Decision making                    │
        │   - Order placement                    │
        └────────────────────────────────────────┘
```

---

## 2. CppTrader Components

### 2.1 Market Manager

**Purpose**: Handles orders and builds order books from market data updates.

**Performance**:
- 3.2M messages/second (standard)
- 8.3M messages/second (optimized)
- 9.7M messages/second (aggressive optimized)

**Key Features**:
- Order book construction
- Price level management
- Best bid/ask tracking
- Market update notifications

**Integration**: Replace simple `market_data_` map with CppTrader `MarketManager`.

### 2.2 Order Book

**Purpose**: Represents bid/ask price levels with order depth.

**Structure**:
```cpp
class OrderBook {
  // Bid side (sorted descending)
  std::vector<PriceLevel> bids_;

  // Ask side (sorted ascending)
  std::vector<PriceLevel> asks_;

  // Best bid/ask
  PriceLevel* best_bid_;
  PriceLevel* best_ask_;
};
```

**Integration**: Store order books per symbol, update from TWS ticks.

### 2.3 NASDAQ ITCH Handler

**Purpose**: Parse and process NASDAQ ITCH market data files.

**Performance**: 41M+ messages/second

**Integration**: Optional - for historical data replay or NASDAQ direct feeds.

---

## 3. Implementation Strategy

### Phase 1: CppTrader Integration (Foundation)

#### 3.1 Add CppTrader as CMake Dependency

**File**: `native/CMakeLists.txt`

```cmake
# CppTrader integration
option(ENABLE_CPPTRADER "Enable CppTrader order book processing" ON)

if(ENABLE_CPPTRADER)
  # CppTrader uses git submodules via gil (git links)
  find_package(Git REQUIRED)

  # Clone CppTrader using gil
  include(FetchContent)
  FetchContent_Declare(
    cpptrader
    GIT_REPOSITORY https://github.com/chronoxor/CppTrader.git
    GIT_TAG master
  )

  # Or use as git submodule:
  # git submodule add https://github.com/chronoxor/CppTrader.git native/third_party/cpptrader

  set(CPPTRADER_DIR "${CMAKE_SOURCE_DIR}/native/third_party/cpptrader")
  if(EXISTS "${CPPTRADER_DIR}/CMakeLists.txt")
    add_subdirectory("${CPPTRADER_DIR}" cpptrader)
    target_link_libraries(${PROJECT_NAME} PRIVATE cpptrader)
    target_include_directories(${PROJECT_NAME} PRIVATE "${CPPTRADER_DIR}/include")
  endif()
endif()
```

#### 3.2 Create Order Book Manager Wrapper

**File**: `native/include/order_book_manager.h`

```cpp
#pragma once

#include <memory>
#include <string>
#include <unordered_map>
#include <functional>
#include "types.h"

// Forward declarations
namespace CppTrader {
  class MarketManager;
  class OrderBook;
}

namespace ib_box_spread {

class OrderBookManager {
 public:
  OrderBookManager();
  ~OrderBookManager();

  // Update order book from TWS tick data
  void update_tick_price(
    const std::string& symbol,
    int field,  // TickType from TWS
    double price
  );

  void update_tick_size(
    const std::string& symbol,
    int field,  // TickType from TWS
    int size
  );

  // Get current market data
  std::optional<types::MarketData> get_market_data(
    const std::string& symbol
  ) const;

  // Get order book depth (level 2)
  struct OrderBookLevel {
    double price;
    int size;
    int orders;
  };

  struct OrderBookDepth {
    std::vector<OrderBookLevel> bids;  // Best bid first
    std::vector<OrderBookLevel> asks;  // Best ask first
  };

  std::optional<OrderBookDepth> get_order_book_depth(
    const std::string& symbol,
    int max_levels = 10
  ) const;

  // Callback for market updates
  using MarketUpdateCallback = std::function<void(
    const std::string& symbol,
    const types::MarketData& data
  )>;

  void set_market_update_callback(MarketUpdateCallback callback);

 private:
  std::unique_ptr<CppTrader::MarketManager> market_manager_;
  std::unordered_map<std::string, uint32_t> symbol_ids_;  // Symbol -> CppTrader ID
  std::unordered_map<uint32_t, std::string> id_symbols_;  // CppTrader ID -> Symbol
  uint32_t next_symbol_id_;

  MarketUpdateCallback market_update_callback_;

  // Convert TWS TickType to CppTrader order book update
  void process_market_update(const std::string& symbol, uint32_t symbol_id);
};

}  // namespace ib_box_spread
```

#### 3.3 Implement Order Book Manager

**File**: `native/src/order_book_manager.cpp`

Key implementation points:

1. **Symbol Management**: Map TWS symbols to CppTrader symbol IDs
2. **Tick Conversion**: Convert TWS tick updates to CppTrader order book updates
3. **Data Extraction**: Extract best bid/ask, depth from CppTrader order books
4. **Callback Integration**: Trigger callbacks on market updates

**TWS TickType Mapping**:
- `BID` (1) → Bid price
- `ASK` (2) → Ask price
- `LAST` (4) → Last trade price
- `BID_SIZE` (0) → Bid size
- `ASK_SIZE` (3) → Ask size
- `LAST_SIZE` (5) → Last trade size

### Phase 2: TWS Client Integration

#### 3.4 Modify TWS Client to Use Order Book Manager

**File**: `native/src/tws_client.cpp`

Replace simple `market_data_` map with `OrderBookManager`:

```cpp
// In TWSClient class:
#include "order_book_manager.h"

class TWSClient : public EWrapper {
 private:
  std::unique_ptr<OrderBookManager> order_book_manager_;

 public:
  void tickPrice(TickerId tickerId, TickType field, double price,
                 const TickAttrib& attribs) override {
    // Get symbol for this tickerId
    std::string symbol = get_symbol_for_ticker_id(tickerId);

    // Update order book
    order_book_manager_->update_tick_price(symbol, static_cast<int>(field), price);

    // Get updated market data and notify callbacks
    auto market_data = order_book_manager_->get_market_data(symbol);
    if (market_data && market_data_callback_) {
      market_data_callback_(symbol, *market_data);
    }
  }

  void tickSize(TickerId tickerId, TickType field, Decimal size) override {
    std::string symbol = get_symbol_for_ticker_id(tickerId);
    order_book_manager_->update_tick_size(symbol, static_cast<int>(field), size);

    auto market_data = order_book_manager_->get_market_data(symbol);
    if (market_data && market_data_callback_) {
      market_data_callback_(symbol, *market_data);
    }
  }
};
```

### Phase 3: Python Replacement

#### 3.5 Migrate Market Data Handler to C++

**Current Python Flow**:
1. nautilus_trader → `MarketDataHandler` → C++ `MarketData` format
2. Data quality validation in Python
3. Callback-based event handling

**New C++ Flow**:
1. TWS API → `TWSClient` → `OrderBookManager` → C++ `MarketData` format
2. Data quality validation in C++ (order book integrity checks)
3. Same callback interface (maintain compatibility)

**File**: `native/include/market_data_validator.h`

```cpp
#pragma once

#include "types.h"

namespace ib_box_spread {

class MarketDataValidator {
 public:
  struct ValidationConfig {
    double max_data_age_seconds = 5.0;
    double min_spread = 0.01;
    double max_spread_percent = 10.0;
  };

  static bool validate(const types::MarketData& data,
                      const ValidationConfig& config = ValidationConfig());

  static bool is_stale(const types::MarketData& data,
                      double max_age_seconds = 5.0);

  static bool is_spread_valid(const types::MarketData& data,
                              double min_spread = 0.01,
                              double max_spread_pct = 10.0);
};

}  // namespace ib_box_spread
```

### Phase 4: Box Spread Integration

#### 3.6 Enhance Box Spread Calculator with Order Book Data

**File**: `native/include/box_spread_calc.h`

Add methods to use order book depth:

```cpp
// Enhanced box spread calculation using order book depth
struct BoxSpreadOpportunity {
  OptionContract leg1, leg2, leg3, leg4;
  double net_credit;
  double implied_rate;
  double min_spread_required;  // Minimum spread for profitability
  OrderBookDepth order_book;    // Current order book state
};

std::vector<BoxSpreadOpportunity> find_box_spreads_with_depth(
  const OptionChain& chain,
  const OrderBookDepth& underlying_book,
  double min_profit_threshold
);
```

---

## 4. Build System Updates

### 4.1 CMake Configuration

**File**: `native/CMakeLists.txt`

Add CppTrader dependency with git submodule or FetchContent:

```cmake
# CppTrader requires gil (git links) tool
find_program(GIL_EXECUTABLE gil)
if(NOT GIL_EXECUTABLE)
  message(WARNING "gil tool not found. Install with: pip3 install gil")
  message(WARNING "CppTrader integration will be disabled")
  set(ENABLE_CPPTRADER OFF)
endif()

if(ENABLE_CPPTRADER AND GIL_EXECUTABLE)
  # Option 1: Use git submodule
  if(EXISTS "${CMAKE_SOURCE_DIR}/native/third_party/cpptrader/CMakeLists.txt")
    add_subdirectory(native/third_party/cpptrader cpptrader)
  else()
    # Option 2: FetchContent (simpler, but no submodule)
    include(FetchContent)
    FetchContent_Declare(
      cpptrader
      GIT_REPOSITORY https://github.com/chronoxor/CppTrader.git
      GIT_TAG master
    )
    FetchContent_MakeAvailable(cpptrader)
  endif()

  target_link_libraries(${PROJECT_NAME} PRIVATE trader)
  target_include_directories(${PROJECT_NAME} PRIVATE
    ${cpptrader_SOURCE_DIR}/include
  )
endif()
```

### 4.2 Build Script Updates

**File**: `scripts/setup_worktree.sh`

Add CppTrader submodule initialization:

```bash
# Initialize git submodules (including CppTrader)
git submodule update --init --recursive

# If using gil, update submodules
if command -v gil &> /dev/null; then
  cd native/third_party/cpptrader 2>/dev/null && gil update || true
fi
```

---

## 5. Performance Optimization

### 5.1 CppTrader Optimized Mode

CppTrader offers three optimization levels:

1. **Standard**: Balanced performance, full features
2. **Optimized**: Pre-allocated arrays, sorted price levels (2.5x faster)
3. **Aggressive**: Minimal structures, no symbols, integer prices (3x faster)

**Recommendation**: Start with **Optimized** mode for best balance.

**Configuration**:
```cpp
// Use optimized market manager
#include <trader/market/market_manager_optimized.h>

using MarketManager = CppTrader::MarketManagerOptimized;
```

### 5.2 Memory Pool Pre-allocation

Pre-allocate symbol IDs and order book capacity:

```cpp
class OrderBookManager {
 private:
  // Pre-allocate for common symbols
  static constexpr size_t kMaxSymbols = 10000;
  static constexpr size_t kMaxPriceLevels = 1000;

  void preallocate_resources() {
    market_manager_->reserve_symbols(kMaxSymbols);
    // CppTrader handles internal pre-allocation
  }
};
```

---

## 6. Testing Strategy

### 6.1 Unit Tests

**File**: `native/tests/order_book_manager_test.cpp`

Test cases:
- Symbol registration and ID mapping
- Tick price updates → order book updates
- Best bid/ask extraction
- Order book depth queries
- Market update callbacks

### 6.2 Integration Tests

**File**: `native/tests/tws_order_book_integration_test.cpp`

Test cases:
- TWS tick stream → Order book reconstruction
- Multiple symbols simultaneous updates
- Market data callback delivery
- Performance benchmarks

### 6.3 Performance Benchmarks

**File**: `native/tests/order_book_performance_test.cpp`

Compare:
- Python `MarketDataHandler` vs C++ `OrderBookManager`
- Simple map vs CppTrader order book
- Latency measurements (tick → callback)

---

## 7. Migration Path

### 7.1 Phase 1: Parallel Implementation (Week 1-2)

- Add CppTrader dependency
- Implement `OrderBookManager` wrapper
- Test with mock tick data
- Unit tests

**Deliverable**: Working order book manager with tests

### 7.2 Phase 2: TWS Integration (Week 3-4)

- Integrate with `TWSClient`
- Replace `market_data_` map usage
- Maintain callback interface compatibility
- Integration tests

**Deliverable**: TWS client using CppTrader order books

### 7.3 Phase 3: Python Replacement (Week 5-6)

- Create C++ `MarketDataValidator`
- Migrate validation logic
- Update Python bindings (if needed)
- Performance comparison

**Deliverable**: C++-only market data processing

### 7.4 Phase 4: Box Spread Enhancement (Week 7-8)

- Enhance `BoxSpreadCalculator` with order book depth
- Add depth-based opportunity detection
- Optimize pricing calculations
- End-to-end tests

**Deliverable**: Production-ready C++ market data pipeline

---

## 8. Risk Mitigation

### 8.1 Backward Compatibility

- Maintain existing `MarketData` struct
- Keep callback interface unchanged
- Support both old and new implementations via CMake option

### 8.2 Performance Regression

- Benchmark before/after migration
- Fallback to simple map if CppTrader causes issues
- Monitor latency in production

### 8.3 Build Complexity

- Make CppTrader optional via `ENABLE_CPPTRADER` flag
- Document submodule/gil setup clearly
- Provide fallback implementation

---

## 9. Documentation Updates

### 9.1 API Documentation

- Add `OrderBookManager` to API docs
- Document CppTrader integration patterns
- Update architecture diagrams

### 9.2 Build Documentation

- Add CppTrader setup instructions
- Update `README.md` with submodule setup
- Document CMake options

### 9.3 Migration Guide

- Create guide for Python → C++ migration
- Performance comparison results
- Troubleshooting section

---

## 10. Future Enhancements

### 10.1 NASDAQ ITCH Handler

- Historical data replay
- Backtesting with ITCH files
- Market data archive processing

### 10.2 Matching Engine

- Internal order matching
- Price-time priority
- Order execution simulation

### 10.3 Multi-Exchange Support

- Aggregate order books across exchanges
- Best execution logic
- Smart order routing

---

## 11. References

- **CppTrader GitHub**: <https://github.com/chronoxor/CppTrader>
- **CppTrader API Docs**: <https://github.com/chronoxor/CppTrader/tree/master/documents>
- **Performance Benchmarks**: See CppTrader README for detailed metrics
- **MIT License**: Permissive, production-ready

---

## 12. Success Criteria

- [ ] CppTrader integrated as CMake dependency
- [ ] `OrderBookManager` implemented and tested
- [ ] TWS client using CppTrader order books
- [ ] Performance: <100μs tick → callback latency
- [ ] Throughput: >100K ticks/second
- [ ] Python `MarketDataHandler` deprecated
- [ ] Box spread calculator enhanced with order book depth
- [ ] Documentation updated
- [ ] All tests passing

---

**Status**: 📋 Planning
**Priority**: High
**Estimated Effort**: 8 weeks
**Dependencies**: None (CppTrader is standalone)
