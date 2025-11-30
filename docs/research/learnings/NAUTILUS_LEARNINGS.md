# Learnings from NautilusTrader Documentation and Architecture

**Date**: 2025-01-27
**Source**: NautilusTrader official documentation and architecture
**Purpose**: Document patterns and recommendations from NautilusTrader that could enhance this project

---

## Overview

NautilusTrader is a high-performance algorithmic trading platform that uses **Rust for core components** and **Python for strategy development**. This hybrid architecture provides both performance and flexibility. This document outlines key learnings and recommendations for improving our box spread implementation.

---

## Key Architectural Patterns

### 1. Rust for Core Components

**NautilusTrader Approach:**

- Core components (`nautilus_core`) implemented in Rust
- Type-safe and memory-safe by construction
- High performance for latency-critical operations
- Python bindings via Cython (static linking) and PyO3

**Current State:**

- This project uses C++ for core calculations
- Python bindings via Cython
- Good performance, but Rust could offer better safety guarantees

**Recommendation:**

- **Consider Rust for performance-critical components:**
  - Option chain scanning algorithms
  - Real-time market data processing
  - Order matching and execution logic
  - Risk calculations

- **Keep C++ for:**
  - Existing TWS API integration (well-established)
  - Legacy code that works well

- **Hybrid approach:**
  - Use Rust for new performance-critical features
  - Maintain C++ for TWS client
  - Bridge between Rust and C++ via FFI or Python

### 2. Event-Driven Architecture

**NautilusTrader Approach:**

- Event-driven design with message passing
- Components communicate via events (market data, order updates, etc.)
- Asynchronous processing for low latency
- Clear separation between data, execution, and strategy layers

**Current State:**

- This project uses callbacks and polling
- Synchronous order placement
- Strategy loop with periodic evaluation

**Recommendation:**

- **Implement event-driven market data handling:**

  ```python
  # Instead of polling, use event callbacks
  def on_quote_tick(self, tick: QuoteTick):
      # Immediately process new quote
      self._evaluate_opportunities(tick.instrument_id)
  ```

- **Use async/await for non-blocking operations:**
  - Market data subscriptions
  - Order status monitoring
  - Position updates

- **Implement event bus for decoupling:**
  - Market data events → Strategy evaluation
  - Order events → Position tracking
  - Risk events → Order blocking

### 3. Strategy Class Pattern

**NautilusTrader Approach:**

- Strategies inherit from base `Strategy` class
- Lifecycle methods: `on_start()`, `on_stop()`, `on_reset()`
- Event handlers: `on_quote_tick()`, `on_trade_tick()`, `on_order_filled()`
- State management within strategy instance

**Current State:**

- `StrategyRunner` class exists but doesn't follow NautilusTrader pattern
- Missing lifecycle management
- Event handlers are basic

**Recommendation:**

- **Refactor to proper Strategy class:**

  ```python
  class BoxSpreadStrategy(Strategy):
      def on_start(self):
          # Subscribe to market data
          # Initialize state
          pass

      def on_quote_tick(self, tick: QuoteTick):
          # Evaluate opportunities on each quote
          self._evaluate_box_spreads(tick)

      def on_order_filled(self, event: OrderFilled):
          # Update position tracking
          # Check if box spread is complete
          pass
  ```

### 4. Instrument Management

**NautilusTrader Approach:**

- `InstrumentId` for all instruments (standardized format)
- Instrument definitions include contract specifications
- Proper instrument resolution from symbols
- Support for options chains via instrument queries

**Current State:**

- Uses string symbols directly
- Manual instrument ID construction
- No standardized instrument format

**Recommendation:**

- **Use NautilusTrader's InstrumentId:**

  ```python
  # Instead of: symbol = "SPY"
  instrument_id = InstrumentId.from_str("SPY.US")

  # For options:
  option_id = InstrumentId.from_str("SPY240412C00500000.US")
  ```

- **Leverage instrument definitions:**
  - Store contract specifications in instrument
  - Use instrument queries for option chains
  - Proper expiry and strike handling

### 5. Order Management Best Practices

**NautilusTrader Approach:**

- Order factory pattern for creating orders
- Order state machine (INITIALIZED → SUBMITTED → ACCEPTED → FILLED)
- Order event callbacks for status updates
- Position tracking via order fills

**Current State:**

- Basic order placement
- Manual order status checking
- No proper order factory

**Recommendation:**

- **Implement order factory:**

  ```python
  def create_box_spread_order(self, spread: BoxSpreadLeg) -> List[Order]:
      orders = []
      # Create all 4 legs using order factory
      for leg in spread.legs:
          order = self._order_factory.limit(
              instrument_id=leg.instrument_id,
              side=leg.side,
              quantity=leg.quantity,
              price=leg.price,
              time_in_force=TimeInForce.DAY
          )
          orders.append(order)
      return orders
  ```

- **Use order events for tracking:**
  - Subscribe to order status events
  - Update multi-leg order state automatically
  - Trigger rollback on order rejection

### 6. Market Data Handling

**NautilusTrader Approach:**

- Separate `QuoteTick` and `TradeTick` types
- Real-time streaming with minimal latency
- Data aggregation (bars, order book) handled by framework
- Efficient data structures (Rust-based)

**Current State:**

- Basic market data conversion
- Missing proper tick handling
- No data aggregation

**Recommendation:**

- **Improve market data conversion:**

  ```python
  def _convert_quote_tick(self, tick: QuoteTick) -> Dict:
      return {
          "symbol": str(tick.instrument_id),
          "bid": float(tick.bid_price),
          "ask": float(tick.ask_price),
          "bid_size": int(tick.bid_size),
          "ask_size": int(tick.ask_size),
          "timestamp": tick.ts_event,  # Use actual timestamp
      }
  ```

- **Handle both quote and trade ticks:**
  - Quote ticks for bid/ask updates
  - Trade ticks for execution price validation

- **Use NautilusTrader's data aggregation:**
  - Request bars if needed for analysis
  - Use order book snapshots for depth

### 7. Performance Optimization

**NautilusTrader Approach:**

- Rust core for maximum performance
- Cython for performance-critical Python code
- Zero-copy data structures where possible
- Efficient memory management

**Current State:**

- C++ for calculations (good performance)
- Python for integration (acceptable)
- Some data copying in conversions

**Recommendation:**

- **Use Cython for hot paths:**
  - Option chain scanning (already using Cython bindings)
  - Profitability calculations
  - Market data filtering

- **Minimize data copying:**
  - Pass references where possible
  - Use views instead of copies
  - Cache frequently accessed data

- **Profile and optimize:**
  - Identify bottlenecks
  - Optimize critical paths
  - Consider Rust for new performance-critical code

---

## Rust Implementation Recommendations

### What NautilusTrader Recommends for Rust

1. **Core Data Structures:**
   - Market data types (ticks, bars, order book)
   - Order and position tracking
   - Risk calculations
   - Time series data

2. **Performance-Critical Algorithms:**
   - Order matching logic
   - Market data aggregation
   - Indicator calculations
   - Portfolio optimization

3. **Safety-Critical Components:**
   - Risk checks
   - Position limits
   - Order validation
   - Account balance tracking

### What to Implement in Rust (for this project)

**High Priority:**

1. **Option Chain Scanner**
   - Fast scanning of large option chains
   - Strike pair generation
   - Liquidity filtering
   - Rust's iterator performance would excel here

2. **Market Data Processor**
   - Real-time quote/trade processing
   - Bid/ask spread calculations
   - Data quality validation
   - Zero-copy data structures

3. **Risk Calculator**
   - Position exposure calculations
   - Margin requirements
   - P&L calculations
   - Safety-critical, Rust's guarantees help

**Medium Priority:**
4. **Order Matching Logic**

- Multi-leg order coordination
- Fill matching
- Rollback logic
- State machine implementation

1. **Performance Metrics**
   - Latency measurements
   - Throughput tracking
   - Statistical calculations

**Low Priority (Future):**
6. **Backtesting Engine**

- Historical data processing
- Strategy simulation
- Performance analysis

### Rust Integration Strategy

**Option 1: Rust FFI from C++**

```rust
// Rust library

#[no_mangle]

pub extern "C" fn scan_option_chain(
    chain_ptr: *const OptionChain,
    opportunities: *mut Vec<BoxSpreadOpportunity>
) -> i32 {
    // Implementation
}
```

**Option 2: Rust via Python (PyO3)**

```rust
use pyo3::prelude::*;

#[pyfunction]

fn scan_option_chain_py(chain: PyObject) -> PyResult<Vec<BoxSpreadOpportunity>> {
    // Implementation
    Ok(opportunities)
}
```

**Option 3: Standalone Rust Service**

- Separate Rust service for calculations
- Communicate via message queue or gRPC
- Maximum isolation and performance

**Recommendation:** Start with Option 2 (PyO3) for easier integration, migrate to Option 1 if needed for tighter C++ integration.

---

## Specific Improvements for Current Implementation

### 1. StrategyRunner Improvements

**Current Issues:**

- Missing proper lifecycle management
- No event-driven architecture
- Basic market data handling

**Improvements:**

```python
class BoxSpreadStrategyRunner:
    def __init__(self, ...):
        self._cache = {}  # Cache option chains
        self._pending_orders = {}  # Track multi-leg orders
        self._event_bus = EventBus()  # Event system

    def on_start(self):
        # Subscribe to all required instruments
        # Initialize state
        # Start event loop
        pass

    def on_quote_tick(self, tick: QuoteTick):
        # Immediately evaluate on new quote
        instrument_id = tick.instrument_id
        if self._is_option(instrument_id):
            self._update_option_chain(instrument_id, tick)
            self._evaluate_opportunities(instrument_id)

    def on_order_filled(self, event: OrderFilled):
        # Update position
        # Check if box spread complete
        # Trigger next action
        pass
```

### 2. Market Data Handler Improvements

**Current Issues:**

- Missing timestamp from actual tick
- No data quality checks
- Basic conversion

**Improvements:**

```python
def _convert_quote_tick(self, tick: QuoteTick) -> Dict:
    # Use actual timestamp from tick
    timestamp = datetime.fromtimestamp(tick.ts_event / 1e9)

    # Validate data quality
    if not tick.bid_price or not tick.ask_price:
        return None  # Invalid data

    # Check for stale data
    age = (datetime.now() - timestamp).total_seconds()
    if age > 5.0:  # 5 second threshold
        logger.warning(f"Stale data for {tick.instrument_id}")
        return None

    return {
        "symbol": str(tick.instrument_id),
        "bid": float(tick.bid_price),
        "ask": float(tick.ask_price),
        "bid_size": int(tick.bid_size),
        "ask_size": int(tick.ask_size),
        "timestamp": timestamp,
        "spread": float(tick.ask_price - tick.bid_price),
        "spread_pct": self._calculate_spread_pct(tick),
    }
```

### 3. Execution Handler Improvements

**Current Issues:**

- Order factory not implemented
- Missing proper order construction
- No combo order support

**Improvements:**

```python
def create_box_spread_orders(self, spread: BoxSpreadLeg) -> List[Order]:
    """Create all 4 legs as combo order if supported."""
    # Try combo order first (atomic execution)
    if self._supports_combo_orders():
        return self._create_combo_order(spread)

    # Fallback to individual orders
    orders = []
    for leg in [spread.long_call, spread.short_call,
                spread.long_put, spread.short_put]:
        order = self._order_factory.limit(
            instrument_id=leg.instrument_id,
            side=leg.side,
            quantity=leg.quantity,
            price=leg.price,
            time_in_force=TimeInForce.DAY,
            post_only=False,  # Allow immediate execution
        )
        orders.append(order)
    return orders
```

### 4. Option Chain Management

**Current Issues:**

- No caching of option chains
- Manual chain construction
- No efficient lookup

**Improvements:**

```python
class OptionChainManager:
    def __init__(self):
        self._chains: Dict[str, OptionChain] = {}
        self._expiry_cache: Dict[str, List[str]] = {}

    def update_chain(self, instrument_id: InstrumentId, tick: QuoteTick):
        """Update option chain with new market data."""
        symbol = self._get_underlying(instrument_id)
        expiry = self._get_expiry(instrument_id)
        strike = self._get_strike(instrument_id)

        if symbol not in self._chains:
            self._chains[symbol] = OptionChain(symbol)

        chain = self._chains[symbol]
        chain.update_option(expiry, strike, tick)

    def get_box_spread_opportunities(self, symbol: str) -> List[BoxSpreadOpportunity]:
        """Get all valid box spreads for symbol."""
        if symbol not in self._chains:
            return []

        chain = self._chains[symbol]
        return chain.find_box_spreads()
```

---

## Performance Benchmarks and Targets

### NautilusTrader Performance Characteristics

- **Market Data Processing:** < 1 microsecond per tick
- **Order Submission:** < 100 microseconds
- **Strategy Evaluation:** < 10 microseconds per opportunity
- **Memory Usage:** Efficient zero-copy where possible

### Current Project Targets

Based on NautilusTrader benchmarks, aim for:

- **Option Chain Scan:** < 10ms for 1000 options
- **Box Spread Evaluation:** < 1ms per combination
- **Order Placement:** < 50ms for 4-leg order
- **Market Data Update:** < 100 microseconds processing

---

## Migration Path

### Phase 1: Improve Python Integration (Immediate)

1. Implement proper Strategy class pattern
2. Add event-driven market data handling
3. Improve order factory and management
4. Add proper lifecycle management

### Phase 2: Optimize Critical Paths (Short-term)

1. Profile current implementation
2. Identify bottlenecks
3. Optimize with Cython where needed
4. Add caching and memoization

### Phase 3: Rust Integration (Medium-term)

1. Implement option chain scanner in Rust
2. Create PyO3 bindings
3. Integrate with existing Python code
4. Benchmark and compare performance

### Phase 4: Full Rust Core (Long-term)

1. Migrate more components to Rust
2. Create comprehensive Rust library
3. Maintain Python API for strategies
4. Optimize for maximum performance

---

## Key Takeaways

1. **Event-Driven > Polling:** Use event callbacks for real-time processing
2. **Rust for Performance:** Consider Rust for performance-critical components
3. **Proper Lifecycle:** Implement on_start, on_stop, on_reset methods
4. **Instrument Management:** Use standardized InstrumentId format
5. **Order Factory:** Use factory pattern for order creation
6. **Data Quality:** Always validate market data freshness and completeness
7. **Caching:** Cache option chains and frequently accessed data
8. **Async Processing:** Use async/await for non-blocking operations

---

## References

- NautilusTrader Documentation: <https://nautilustrader.io/docs/nightly/>
- Architecture Guide: <https://docs.nautilustrader.io/concepts/architecture>
- Tutorials: <https://nautilustrader.io/docs/nightly/tutorials>
- GitHub Repository: <https://github.com/nautechsystems/nautilus_trader>
- Rust Integration: <https://docs.nautilustrader.io/advanced/rust>

---

## Notes

- NautilusTrader's architecture is well-designed for high-frequency trading
- The Rust + Python hybrid approach is proven in production
- Event-driven architecture reduces latency significantly
- Proper instrument and order management is critical for reliability
- Performance optimization should be data-driven (profile first)
