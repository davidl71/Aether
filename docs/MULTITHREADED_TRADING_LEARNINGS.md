# Learnings from Multi-Threaded Stock Trading System Article

**Date**: 2025-01-27
**Source**: [Mastering a Multi-Threaded Stock Trading System in C++](https://medium.com/@haykuhimkrtchyan09/mastering-a-multi-threaded-stock-trading-system-in-c-a-comprehensive-guide-6c3a233a918d) by Haykuhi Mkrtchyan
**Purpose**: Document patterns and recommendations from the article that could enhance this project's multi-threading and order management architecture

---

## Overview

This article provides a comprehensive guide to building a multi-threaded stock trading system in C++ using object-oriented design, the Standard Template Library (STL), and design patterns. The system emphasizes real-time order processing, concurrent trading activities, and efficient data structures for high-performance trading applications.

---

## Key Components from the Article

### 1. OrderBook Class

**Article Approach:**
- Central component managing stocks, traders, and orders
- Uses `std::unordered_map` for fast access and retrieval
- Constant-time lookup (O(1)) for order operations
- Customized key-value pairs for efficient data organization

**Key Features:**
- Fast order insertion and retrieval
- Efficient trader and stock information management
- Thread-safe operations for concurrent access

**Current State:**
- This project uses `OrderManager` class (`native/src/order_manager.cpp`)
- Tracks multi-leg orders and execution statistics
- Uses standard containers but could benefit from optimized data structures
- Thread safety handled via mutexes in TWS client

**Recommendation:**
- **Consider implementing an OrderBook-style structure:**
  ```cpp
  class OrderBook {
  private:
      std::unordered_map<std::string, std::vector<Order>> orders_by_symbol_;
      std::unordered_map<int, Order*> orders_by_id_;
      std::mutex mutex_;
  public:
      void add_order(const Order& order);
      Order* find_order(int order_id);
      std::vector<Order> get_orders_for_symbol(const std::string& symbol);
  };
  ```
- **Benefits:**
  - Faster order lookups by ID (O(1) vs O(n))
  - Efficient symbol-based order queries
  - Better organization for multi-leg box spread orders
  - Easier to implement order matching logic

### 2. Trader Class

**Article Approach:**
- Manages information about stocks and orders
- Uses `std::vector` for dynamic flexibility
- Constant-time random access for order management
- Tracks trader-specific state and positions

**Key Features:**
- Dynamic order list management
- Position tracking per trader
- Efficient iteration over orders

**Current State:**
- This project doesn't have a dedicated Trader class
- Order tracking is handled in `OrderManager`
- Position tracking exists in backend Rust service (`agents/backend/crates/api/src/state.rs`)

**Recommendation:**
- **Consider adding a Trader/Account abstraction:**
  ```cpp
  class Trader {
  private:
      std::string trader_id_;
      std::vector<Order> active_orders_;
      std::unordered_map<std::string, Position> positions_;
      double account_balance_;
  public:
      void add_order(const Order& order);
      void update_position(const std::string& symbol, int quantity);
      double get_unrealized_pnl() const;
  };
  ```
- **Benefits:**
  - Better separation of concerns
  - Easier to implement multi-account support
  - Clearer position and P&L tracking
  - Supports future features like paper trading vs live trading

### 3. Multithreading Architecture

**Article Approach:**
- Multiple threads for concurrent trading activities
- Thread-safe data structures for shared state
- Efficient processing of multiple orders and updates in real-time
- Synchronization primitives for coordination

**Key Features:**
- Concurrent order processing
- Real-time market data updates
- Parallel order matching
- Thread-safe order book operations

**Current State:**
- This project uses:
  - **Main Thread**: Application logic, strategy execution
  - **TWS Callback Thread**: EWrapper callbacks (from TWS API)
  - **Order Tracking**: Thread-safe order status updates
  - **Market Data**: Thread-safe option chain updates
- Synchronization via mutexes, atomic flags, and condition variables
- Rust backend uses Tokio async runtime for concurrent operations

**Recommendation:**
- **Enhance C++ threading model:**
  ```cpp
  class ThreadedOrderProcessor {
  private:
      std::thread order_thread_;
      std::thread market_data_thread_;
      std::thread matching_thread_;
      std::atomic<bool> running_;
      std::queue<Order> order_queue_;
      std::mutex queue_mutex_;
      std::condition_variable queue_cv_;

  public:
      void start();
      void process_orders();
      void process_market_data();
      void match_orders();
  };
  ```
- **Benefits:**
  - Dedicated threads for different responsibilities
  - Better CPU utilization
  - Reduced latency for order processing
  - Clearer separation of concerns
- **Considerations:**
  - Current TWS API integration uses callback model (single callback thread)
  - Need to ensure thread safety when accessing TWS client
  - Balance between thread overhead and performance gains

### 4. Factory Method Pattern for Orders

**Article Approach:**
- Factory pattern to create different order types (Market Order, Limit Order)
- Flexible and extensible order creation
- Abstracts order creation process
- Easy to add new order types

**Key Features:**
- Type-safe order creation
- Centralized order validation
- Extensible design for new order types

**Current State:**
- This project creates orders directly in `OrderManager::place_box_spread()`
- Uses `Order` struct with type field
- No factory pattern currently

**Recommendation:**
- **Implement OrderFactory:**
  ```cpp
  class OrderFactory {
  public:
      static std::unique_ptr<Order> create_market_order(
          const std::string& symbol,
          int quantity,
          OrderSide side);

      static std::unique_ptr<Order> create_limit_order(
          const std::string& symbol,
          int quantity,
          OrderSide side,
          double limit_price);

      static std::unique_ptr<Order> create_stop_order(
          const std::string& symbol,
          int quantity,
          OrderSide side,
          double stop_price);
  };
  ```
- **Benefits:**
  - Centralized order creation logic
  - Easier to add new order types (e.g., stop-loss for box spreads)
  - Consistent order validation
  - Better testability (can mock factory)

### 5. Strategy Pattern for Order Generation

**Article Approach:**
- Strategy pattern for different order generation algorithms
- Flexible software system with simple interface
- Each order type has its own generation strategy
- Easy to swap strategies at runtime

**Key Features:**
- Pluggable order generation algorithms
- Strategy interface for extensibility
- Runtime strategy selection

**Current State:**
- This project has `BoxSpreadStrategy` class
- Strategy is fixed (box spread arbitrage)
- Could benefit from strategy pattern for different arbitrage types

**Recommendation:**
- **Consider strategy pattern for different arbitrage types:**
  ```cpp
  class ArbitrageStrategy {
  public:
      virtual ~ArbitrageStrategy() = default;
      virtual std::vector<Opportunity> find_opportunities(
          const MarketData& data) = 0;
      virtual ExecutionResult execute(const Opportunity& opp) = 0;
  };

  class BoxSpreadStrategy : public ArbitrageStrategy {
      // Current implementation
  };

  class CalendarSpreadStrategy : public ArbitrageStrategy {
      // Future implementation
  };

  class ButterflySpreadStrategy : public ArbitrageStrategy {
      // Future implementation
  };
  ```
- **Benefits:**
  - Easy to add new arbitrage strategies
  - Runtime strategy selection
  - Better code organization
  - Easier testing of individual strategies

---

## Design Patterns Summary

### Patterns Used in Article

1. **Factory Method Pattern**: Order creation
2. **Strategy Pattern**: Order generation algorithms
3. **Observer Pattern**: (Implied) Market data updates
4. **Singleton Pattern**: (Implied) OrderBook instance

### Patterns in Current Codebase

1. **Strategy Pattern**: `BoxSpreadStrategy` class
2. **Factory Pattern**: (Partial) Order creation in OrderManager
3. **Observer Pattern**: TWS callback system (EWrapper)
4. **RAII Pattern**: Smart pointers for resource management

---

## STL Data Structure Recommendations

### Current Usage
- `std::vector` for option chains and opportunities
- `std::unordered_map` for some lookups
- `std::mutex` for thread synchronization

### Article Recommendations
- **`std::unordered_map`** for O(1) lookups (OrderBook)
- **`std::vector`** for dynamic lists (Trader orders)
- **`std::queue`** for order processing queues
- **`std::priority_queue`** for priority-based order matching

### Enhancements for This Project

1. **Order Lookup Optimization:**
   ```cpp
   // Instead of linear search
   std::unordered_map<int, Order*> orders_by_id_;
   std::unordered_map<std::string, std::vector<Order*>> orders_by_symbol_;
   ```

2. **Priority Queue for Opportunities:**
   ```cpp
   // Sort opportunities by profit
   std::priority_queue<BoxSpreadOpportunity> opportunities_;
   ```

3. **Queue for Async Order Processing:**
   ```cpp
   // Thread-safe order queue
   std::queue<Order> pending_orders_;
   std::mutex queue_mutex_;
   ```

---

## Threading Model Comparison

### Article's Threading Model
- Multiple worker threads for order processing
- Dedicated thread for market data
- Thread pool for concurrent operations
- Lock-free data structures where possible

### Current Project's Threading Model
- Main thread: Strategy execution
- TWS callback thread: API callbacks
- Thread-safe updates via mutexes
- Rust backend: Tokio async runtime

### Recommendations

1. **Consider Thread Pool for Order Processing:**
   ```cpp
   class OrderThreadPool {
   private:
       std::vector<std::thread> workers_;
       std::queue<std::function<void()>> tasks_;
       std::mutex queue_mutex_;
       std::condition_variable condition_;
       bool stop_;
   public:
       void enqueue(std::function<void()> task);
   };
   ```

2. **Lock-Free Structures Where Possible:**
   - Use `std::atomic` for simple flags
   - Consider lock-free queues for high-frequency operations
   - Use read-write locks (`std::shared_mutex`) for read-heavy operations

3. **Async/Await Pattern (C++20):**
   - Consider coroutines for async order processing
   - Better integration with TWS async callbacks
   - Cleaner code than callback chains

---

## Performance Considerations

### Article's Performance Optimizations

1. **Fast Data Structures**: O(1) lookups with hash maps
2. **Concurrent Processing**: Multiple threads for parallel operations
3. **Efficient Memory Management**: STL containers with custom allocators
4. **Lock Minimization**: Fine-grained locking, lock-free where possible

### Current Project's Performance

1. **Option Chain Caching**: Avoid redundant TWS requests
2. **Incremental Updates**: Only update changed options
3. **Parallel Scanning**: (Future) Scan multiple symbols concurrently
4. **Efficient Lookups**: Hash maps for option lookup

### Recommendations

1. **Implement OrderBook for Faster Lookups:**
   - O(1) order retrieval by ID
   - Efficient symbol-based queries
   - Better organization for multi-leg orders

2. **Add Thread Pool for Concurrent Processing:**
   - Parallel opportunity scanning
   - Concurrent order placement
   - Better CPU utilization

3. **Optimize Memory Allocations:**
   - Use object pools for frequently allocated objects
   - Pre-allocate containers where size is known
   - Consider custom allocators for hot paths

---

## Integration Points

### Where These Patterns Could Be Applied

1. **OrderManager Enhancement:**
   - Add OrderBook-style structure
   - Implement Factory pattern for order creation
   - Add thread pool for concurrent order processing

2. **BoxSpreadStrategy Enhancement:**
   - Use Strategy pattern for different arbitrage types
   - Add priority queue for opportunity sorting
   - Implement concurrent opportunity scanning

3. **TWS Client Enhancement:**
   - Better thread safety for callback handling
   - Queue-based order submission
   - Async order status updates

4. **Market Data Processing:**
   - Dedicated thread for market data updates
   - Lock-free structures for high-frequency updates
   - Efficient option chain updates

---

## Implementation Priority

### High Priority (Immediate Value)

1. **OrderBook Structure** - Faster order lookups
2. **Factory Pattern for Orders** - Better code organization
3. **Priority Queue for Opportunities** - Better opportunity selection

### Medium Priority (Performance Gains)

4. **Thread Pool for Order Processing** - Better concurrency
5. **Lock-Free Structures** - Reduced contention
6. **Trader/Account Abstraction** - Better position tracking

### Low Priority (Future Enhancements)

7. **Strategy Pattern for Arbitrage Types** - Extensibility
8. **Custom Allocators** - Memory optimization
9. **C++20 Coroutines** - Async improvements

---

## Code Examples

### OrderBook Implementation Skeleton

```cpp
class OrderBook {
private:
    std::unordered_map<int, std::unique_ptr<Order>> orders_by_id_;
    std::unordered_map<std::string, std::vector<Order*>> orders_by_symbol_;
    std::mutex mutex_;

public:
    void add_order(std::unique_ptr<Order> order) {
        std::lock_guard<std::mutex> lock(mutex_);
        int id = order->id;
        orders_by_id_[id] = std::move(order);
        orders_by_symbol_[order->symbol].push_back(orders_by_id_[id].get());
    }

    Order* find_order(int order_id) {
        std::lock_guard<std::mutex> lock(mutex_);
        auto it = orders_by_id_.find(order_id);
        return (it != orders_by_id_.end()) ? it->second.get() : nullptr;
    }

    std::vector<Order*> get_orders_for_symbol(const std::string& symbol) {
        std::lock_guard<std::mutex> lock(mutex_);
        auto it = orders_by_symbol_.find(symbol);
        return (it != orders_by_symbol_.end()) ? it->second : std::vector<Order*>();
    }
};
```

### OrderFactory Implementation

```cpp
class OrderFactory {
public:
    static std::unique_ptr<Order> create_market_order(
            const std::string& symbol,
            int quantity,
            OrderSide side) {
        auto order = std::make_unique<Order>();
        order->symbol = symbol;
        order->quantity = quantity;
        order->side = side;
        order->type = OrderType::MARKET;
        order->id = generate_order_id();
        return order;
    }

    static std::unique_ptr<Order> create_limit_order(
            const std::string& symbol,
            int quantity,
            OrderSide side,
            double limit_price) {
        auto order = create_market_order(symbol, quantity, side);
        order->type = OrderType::LIMIT;
        order->limit_price = limit_price;
        return order;
    }

private:
    static int generate_order_id() {
        static std::atomic<int> counter{0};
        return ++counter;
    }
};
```

---

## Conclusion

The article provides valuable insights into building a high-performance, multi-threaded trading system in C++. Key takeaways:

1. **Data Structure Choice Matters**: Use `std::unordered_map` for O(1) lookups
2. **Design Patterns Help**: Factory and Strategy patterns improve extensibility
3. **Threading Requires Care**: Proper synchronization is critical
4. **STL is Powerful**: Leverage STL containers and algorithms

For this project, the most impactful improvements would be:
- Implementing an OrderBook-style structure for faster order lookups
- Adding a Factory pattern for order creation
- Considering a thread pool for concurrent opportunity scanning
- Using priority queues for opportunity selection

These changes would improve both performance and code maintainability while preserving the existing TWS API integration and box spread strategy logic.
