# Box Spread Bag Implementation

**Date**: 2025-01-27
**Status**: ✅ Core Functionality Complete

---

## Overview

This document describes the implementation of **Box Spread Bags** - modeling box spreads as Cboe-style complex symbols with full Greeks calculation, OHLC candle tracking, and mock data generation for TUI testing.

---

## Features

### 1. Box Spread as Complex Instrument (BAG)

Each box spread is modeled as a Cboe complex symbol (BAG), including:

- **Cboe Symbol Format**: `SPX 25JAN24 4500/4600 BOX`
- **Market Data**: Bid, ask, last, spread, sizes, volume
- **Greeks**: Delta, gamma, theta, vega, rho (aggregated from individual legs)
- **Candle/OHLC**: Full OHLC tracking with history
- **Position Tracking**: Quantity, entry price, P&L

### 2. Greeks Calculation

Greeks are calculated by aggregating from individual option legs:

- **Delta**: Sum of deltas from all 4 legs (should be ~0 for perfect box)
- **Gamma**: Sum of gammas (should cancel out)
- **Theta**: Net time decay (aggregate theta from all legs)
- **Vega**: IV sensitivity (should cancel out for perfect box)
- **Rho**: Rate sensitivity (some exposure due to financing)

**Key Files**:

- `native/include/box_spread_bag.h` - Bag structure and manager interface
- `native/src/box_spread_bag.cpp` - Bag implementation and Greeks calculation

**Implementation**:

```cpp
// Calculate Greeks for box spread bag
auto greeks = BoxSpreadBagManager::calculate_bag_greeks(
    spread,
    underlying_price,
    time_to_expiry_years,
    volatility,
    risk_free_rate
);

// Greeks are aggregated from individual legs
// Delta = long_call_delta - short_call_delta + long_put_delta - short_put_delta
// Theta = long_call_theta - short_call_theta + long_put_theta - short_put_theta
// etc.
```

### 3. OHLC Candle Tracking

Each bag tracks OHLC candle data:

- **Current Candle**: Open, high, low, close, volume, entry price
- **Candle History**: Time series of historical candles
- **Automatic Updates**: Candle updates when market data changes
- **Period Management**: Reset candles for new periods (e.g., hourly, daily)

**Usage**:

```cpp
// Update candle with new price
bag.update_candle(mid_price, volume);

// Add current candle to history and start new one
bag.add_candle_to_history();
bag.reset_candle();

// Access candle data
auto current_candle = bag.candle;
auto historical_candles = bag.candle_history;
```

### 4. Mock Data Generator

Comprehensive mock data generator for testing and TUI development:

- **Realistic Box Spread Generation**: Creates bags with realistic pricing
- **Price Movement Simulation**: Random walk with mean reversion
- **Candle History Generation**: Historical OHLC data
- **Multi-Symbol Support**: Generate bags for SPX, ES, XSP, nanos
- **Yield Curve Bags**: Generate bags across different expirations

**Key Files**:

- `native/include/mock_data_generator.h` - Mock generator interface
- `native/src/mock_data_generator.cpp` - Mock generator implementation

**Usage Example**:

```cpp
MockBoxSpreadBagGenerator generator;

// Generate single bag
auto bag = generator.generate_bag("SPX", "", 100.0, 30);

// Generate multiple bags
auto bags = generator.generate_bags(10, "SPX");

// Simulate price movement
generator.simulate_price_movement(bag, 4500.0, 20, 5.0);  // 20 updates, 5 min intervals

// Generate candle history
generator.generate_candle_history(bag, 50);  // 50 candles

// Generate multi-symbol bags (ES, XSP, SPX)
std::vector<std::string> symbols = {"SPX", "ES", "XSP"};
auto multi_bags = generator.generate_multi_symbol_bags(symbols, 100.0, 30);

// Generate yield curve bags (same strike width, different expirations)
std::vector<int> dte_list = {7, 14, 21, 30, 45, 60, 90, 180};
auto yield_curve_bags = generator.generate_yield_curve_bags("SPX", 100.0, dte_list);
```

---

## Data Structures

### BoxSpreadBag

```cpp
struct BoxSpreadBag {
    // Complex symbol identifiers
    std::string complex_symbol;      // "SPX BOX"
    std::string cboe_symbol;         // "SPX 25JAN24 4500/4600 BOX"

    // Original spread data
    types::BoxSpreadLeg spread;

    // Market data
    struct BagMarketData {
        double bid, ask, last, mid, spread;
        int bid_size, ask_size, volume;
        std::chrono::system_clock::time_point timestamp;
    } market_data;

    // Greeks
    struct BagGreeks {
        double delta, gamma, theta, vega, rho;
        std::chrono::system_clock::time_point calculated_at;
        bool is_neutral() const;  // Check if delta/gamma/vega near zero
    } greeks;

    // Candle data
    struct BagCandle {
        double open, high, low, close, entry, volume;
        std::chrono::system_clock::time_point period_start, period_end, updated;
    } candle;

    std::vector<BagCandle> candle_history;  // Historical candles

    // Position info
    struct BagPosition {
        int quantity;
        double entry_price, current_price, cost_basis;
        double unrealized_pnl, realized_pnl;
        std::chrono::system_clock::time_point entry_time;
    } position;

    // Metrics
    double theoretical_value, net_debit;
    double implied_rate, effective_rate;
    int days_to_expiry;
    double liquidity_score, execution_probability;
};
```

---

## Integration with TUI

The box spread bags can be integrated with the existing TUI data structures:

**TUI Data Format** (`tui/internal/data/types.go`):

```go
type Position struct {
    Name           string
    Quantity       int
    ROI            float64
    Vega           float64
    Theta          float64
    Candle         Candle
}
```

**Conversion**:

- Bag → TUI Position: Convert bag position data to TUI format
- Bag Greeks → TUI Greeks: Map bag.greeks to TUI vega/theta
- Bag Candle → TUI Candle: Convert bag.candle to TUI Candle structure

---

## Mock Data Examples

### Example 1: Generate Single Bag

```cpp
MockBoxSpreadBagGenerator gen;
auto bag = gen.generate_bag("SPX", "", 100.0, 30);

// Access bag data
std::cout << "Symbol: " << bag.cboe_symbol << std::endl;
std::cout << "Delta: " << bag.greeks.delta << std::endl;
std::cout << "Theta: " << bag.greeks.theta << std::endl;
std::cout << "Vega: " << bag.greeks.vega << std::endl;
```

### Example 2: Generate Yield Curve Bags

```cpp
// Generate bags for yield curve analysis
std::vector<int> dte_list = {7, 14, 21, 30, 45, 60, 90, 120, 180};
auto bags = gen.generate_yield_curve_bags("SPX", 100.0, dte_list);

// Each bag has different expiration but same strike width
for (const auto& bag : bags) {
    std::cout << bag.cboe_symbol << " - Rate: " << bag.implied_rate << "%" << std::endl;
}
```

### Example 3: Simulate Intraday Trading

```cpp
auto bag = gen.generate_bag("SPX", "", 100.0, 30);

// Simulate 100 price updates (5-minute intervals)
gen.simulate_price_movement(bag, 4500.0, 100, 5.0);

// Generate hourly candles
gen.generate_candle_history(bag, 20);  // 20 hourly candles

// Access candle history
for (const auto& candle : bag.candle_history) {
    std::cout << "OHLC: " << candle.open << "/" << candle.high
              << "/" << candle.low << "/" << candle.close << std::endl;
}
```

---

## Testing Recommendations

### Unit Tests

1. **Greeks Calculation**:
   - Verify delta/gamma/vega are near zero for perfect box
   - Test theta calculation (should be small positive for borrowing)
   - Test rho sensitivity

2. **Candle Tracking**:
   - Test OHLC updates
   - Test candle history management
   - Test candle reset/period management

3. **Mock Data Generation**:
   - Verify realistic pricing
   - Test price movement simulation
   - Test multi-symbol generation

### Integration Tests

1. **TUI Integration**:
   - Convert bags to TUI format
   - Display bags in TUI positions table
   - Render candle charts

2. **Yield Curve Integration**:
   - Use bags for yield curve visualization
   - Compare bags across symbols/expirations

---

## Future Enhancements

1. **Real-time Market Data Integration**:
   - Connect to TWS API for live bag quotes
   - Update Greeks in real-time
   - Stream candle updates

2. **Advanced Greeks**:
   - Second-order Greeks (gamma, vanna, volga)
   - Greeks surface (Greeks vs underlying price)

3. **Candle Aggregation**:
   - Multiple timeframes (1min, 5min, hourly, daily)
   - Candle aggregation from tick data

---

## Conclusion

✅ **Box Spread Bag Structure**: Complete
✅ **Greeks Calculation**: Implemented (aggregated from legs)
✅ **OHLC Candle Tracking**: Complete with history
✅ **Mock Data Generator**: Complete with realistic data

The implementation provides a solid foundation for modeling box spreads as complex instruments with full Greeks and candle tracking. Mock data generation enables TUI development and testing without live market data.

**Status**: Ready for TUI integration and testing.
