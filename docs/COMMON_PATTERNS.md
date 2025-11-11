# Common Patterns & Conventions

This document describes common coding patterns, conventions, and idioms used throughout the codebase. Use `@docs COMMON_PATTERNS.md` in Cursor to ensure code suggestions follow established patterns.

## Code Style Patterns

### Naming Conventions

**Types**: `PascalCase`
```cpp
class BoxSpreadStrategy;
struct OptionContract;
enum class OrderStatus;
```

**Functions**: `snake_case`
```cpp
void calculate_profit();
bool validate_order();
std::vector<BoxSpread> scan_opportunities();
```

**Variables**: `snake_case`
```cpp
double net_debit;
int order_id;
std::string symbol;
```

**Constants**: `k` prefix + `PascalCase`
```cpp
constexpr int kMaxPositions = 5;
constexpr double kMinProfitThreshold = 10.0;
constexpr int kDefaultPort = 7497;
```

**Member Variables**: `snake_case` (no prefix/suffix)
```cpp
class OrderManager {
  std::vector<Order> orders_;
  std::mutex orders_mutex_;
  int next_order_id_;
};
```

### Indentation & Formatting

**Indentation**: 2 spaces (not tabs)
```cpp
if (condition) {
  do_something();
  if (nested) {
    do_nested();
  }
}
```

**Braces**: Allman style for multi-line scopes
```cpp
if (condition)
{
  do_something();
}
```

**Line Length**: 100 character soft wrap
```cpp
// This is a long comment that wraps at 100 characters to maintain
// readability and consistency with the codebase style.
```

## Error Handling Patterns

### Exception-Based Error Handling

**Throw exceptions for unrecoverable errors**:
```cpp
if (!config.is_valid())
{
  throw std::runtime_error("Invalid configuration");
}
```

**Catch and log at appropriate level**:
```cpp
try
{
  strategy.execute();
}
catch (const std::exception& e)
{
  spdlog::error("Strategy execution failed: {}", e.what());
  // Continue or rethrow based on context
}
```

### Error Codes for Recoverable Errors

**Return error codes for recoverable failures**:
```cpp
enum class ErrorCode {
  Success,
  InvalidInput,
  ConnectionFailed,
  OrderRejected
};

ErrorCode place_order(const Order& order);
```

## Logging Patterns

### Log Levels

**Use appropriate log levels**:
```cpp
spdlog::trace("Detailed debugging info");
spdlog::debug("Debug information");
spdlog::info("Informational message");
spdlog::warn("Warning message");
spdlog::error("Error message");
spdlog::critical("Critical error");
```

### Log Formatting

**Use structured logging with placeholders**:
```cpp
spdlog::info("Order {} placed: {} contracts of {}",
             order_id, quantity, symbol);
spdlog::error("Connection failed: {} (code: {})",
              error_msg, error_code);
```

**Include context**:
```cpp
spdlog::debug("Box spread found: profit=${}, roi={}%, strikes={}-{}",
              profit, roi_percent, strike_low, strike_high);
```

## Resource Management Patterns

### RAII (Resource Acquisition Is Initialization)

**Use RAII for all resources**:
```cpp
class TWSClient {
public:
  TWSClient() {
    connect();  // Acquire in constructor
  }

  ~TWSClient() {
    disconnect();  // Release in destructor
  }
};
```

**Use smart pointers**:
```cpp
std::unique_ptr<OrderManager> order_mgr =
    std::make_unique<OrderManager>(config);
```

### Mutex Locking

**Use lock guards for mutex protection**:
```cpp
std::lock_guard<std::mutex> lock(orders_mutex_);
orders_.push_back(order);
```

**Use unique_lock for condition variables**:
```cpp
std::unique_lock<std::mutex> lock(cv_mutex_);
cv_.wait(lock, [this] { return ready_; });
```

## Configuration Patterns

### Configuration Loading

**Load and validate in constructor**:
```cpp
ConfigManager::ConfigManager(const std::string& config_path)
{
  load_config(config_path);
  validate_config();
}
```

**Provide typed accessors**:
```cpp
double ConfigManager::get_min_profit() const
{
  return config_.strategy.min_arbitrage_profit;
}
```

## Trading-Specific Patterns

### Price Calculations

**Use decimal arithmetic for financial calculations**:
```cpp
// Use Intel Decimal Library for precision
double net_debit = calculate_net_debit(box_spread);
double strike_width = box_spread.strike_high - box_spread.strike_low;
double profit = strike_width - net_debit;
```

**Always validate against thresholds**:
```cpp
if (profit >= config.get_min_profit() &&
    roi_percent >= config.get_min_roi())
{
  return true;  // Valid opportunity
}
```

### Order Management

**Create orders with all legs atomically**:
```cpp
Order order;
order.contracts = box_spread.legs;
order.quantities = {1, -1, 1, -1};  // Long, Short, Long, Short
order.status = OrderStatus::Pending;
```

**Track order lifecycle**:
```cpp
void OrderManager::update_order_status(int order_id, OrderStatus status)
{
  std::lock_guard<std::mutex> lock(orders_mutex_);
  auto it = std::find_if(orders_.begin(), orders_.end(),
                         [order_id](const Order& o) {
                           return o.order_id == order_id;
                         });
  if (it != orders_.end())
  {
    it->status = status;
  }
}
```

## Testing Patterns

### Test Structure

**Use Catch2 test macros**:
```cpp
TEST_CASE("Box spread profit calculation", "[strategy]")
{
  BoxSpread spread = create_test_spread();
  double profit = calculate_profit(spread);
  REQUIRE(profit > 0.0);
}
```

**Test edge cases**:
```cpp
TEST_CASE("Risk limits enforcement", "[risk]")
{
  RiskCalculator calc(config);
  REQUIRE_THROWS(calc.validate_position(1000000.0));
}
```

### Mock Objects

**Mock TWS client for unit tests**:
```cpp
class MockTWSClient : public TWSClient {
public:
  MOCK_METHOD(void, place_order, (const Order&), (override));
};
```

## Threading Patterns

### Thread Safety

**Protect shared data with mutexes**:
```cpp
class OptionChain {
  std::mutex data_mutex_;
  std::unordered_map<std::string, OptionContract> contracts_;

public:
  void update_contract(const OptionContract& contract)
  {
    std::lock_guard<std::mutex> lock(data_mutex_);
    contracts_[contract.symbol] = contract;
  }
};
```

### Callback Handling

**Handle TWS callbacks safely**:
```cpp
void TWSClient::onTickPrice(TickPrice tick)
{
  // Callbacks run on TWS thread
  std::lock_guard<std::mutex> lock(prices_mutex_);
  prices_[tick.request_id] = tick.price;
  price_cv_.notify_one();
}
```

## Common Idioms

### Range-Based For Loops

**Prefer range-based for loops**:
```cpp
for (const auto& order : orders_)
{
  process_order(order);
}
```

### Auto Keyword

**Use auto for iterator types**:
```cpp
auto it = std::find_if(orders_.begin(), orders_.end(),
                       [id](const Order& o) { return o.id == id; });
```

### Const Correctness

**Mark methods const when appropriate**:
```cpp
double calculate_profit() const;  // Doesn't modify state
void update_status();              // Modifies state
```

**Use const references for parameters**:
```cpp
void process_order(const Order& order);  // Don't copy
```

## Anti-Patterns to Avoid

### ❌ Don't Do This

**Raw pointers**:
```cpp
OrderManager* mgr = new OrderManager();  // Use smart pointers
```

**C-style arrays**:
```cpp
int prices[100];  // Use std::vector
```

**Magic numbers**:
```cpp
if (profit > 10.0)  // Use named constants
```

**Ignoring errors**:
```cpp
place_order(order);  // Check return value or catch exceptions
```

**Thread-unsafe access**:
```cpp
orders_.push_back(order);  // Protect with mutex
```

### ✅ Do This Instead

**Smart pointers**:
```cpp
auto mgr = std::make_unique<OrderManager>();
```

**STL containers**:
```cpp
std::vector<double> prices;
```

**Named constants**:
```cpp
if (profit > kMinProfitThreshold)
```

**Error handling**:
```cpp
try {
  place_order(order);
} catch (const std::exception& e) {
  spdlog::error("Failed to place order: {}", e.what());
}
```

**Thread-safe access**:
```cpp
{
  std::lock_guard<std::mutex> lock(orders_mutex_);
  orders_.push_back(order);
}
```

## Related Documentation

- **Architecture**: `docs/CODEBASE_ARCHITECTURE.md`
- **API Reference**: `docs/API_DOCUMENTATION_INDEX.md`
- **Code Style**: `.cursorrules`
