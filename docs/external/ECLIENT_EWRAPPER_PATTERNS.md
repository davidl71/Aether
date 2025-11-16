# EClient/EWrapper Patterns

**Source**: Based on IBKR TWS API best practices and project implementation
**Last Updated**: 2025-01-27

This document describes common patterns and best practices for using EClient and EWrapper in TWS API applications.

---

## Architecture Pattern

### Basic Structure

```cpp
class MyTradingApp : public DefaultEWrapper {
private:
  EClientSocket client_;
  EReaderOSSignal signal_;
  std::thread reader_thread_;

public:
  MyTradingApp() : client_(this, &signal_) {}

  // EWrapper callbacks
  void tickPrice(...) override { /* handle */ }
  void orderStatus(...) override { /* handle */ }

  // EClient methods
  void request_market_data(...) {
    client_.reqMktData(...);
  }
};
```

---

## Pattern 1: Connection Management

### Connect and Wait Pattern

```cpp
class TWSClient {
  std::mutex mutex_;
  std::condition_variable cv_;
  bool connected_ = false;
  OrderId next_valid_id_ = 0;

  bool connect(const std::string& host, int port, int client_id) {
    client_.eConnect(host.c_str(), port, client_id);

    // Wait for connection acknowledgment
    std::unique_lock<std::mutex> lock(mutex_);
    if (cv_.wait_for(lock, std::chrono::seconds(5),
                     [this] { return connected_ && next_valid_id_ > 0; })) {
      return true;
    }
    return false;  // Timeout
  }

  void connectAck() override {
    std::lock_guard<std::mutex> lock(mutex_);
    connected_ = true;
    cv_.notify_all();
  }

  void nextValidId(OrderId orderId) override {
    std::lock_guard<std::mutex> lock(mutex_);
    next_valid_id_ = orderId;
    cv_.notify_all();
  }

  void connectionClosed() override {
    std::lock_guard<std::mutex> lock(mutex_);
    connected_ = false;
    next_valid_id_ = 0;
  }
};
```

---

## Pattern 2: Request-Response Tracking

### Track Pending Requests

```cpp
class MarketDataManager {
  std::mutex mutex_;
  std::map<TickerId, std::shared_ptr<MarketData>> pending_requests_;
  TickerId next_id_ = 1;

  TickerId request_market_data(const Contract& contract) {
    TickerId id = next_id_++;

    {
      std::lock_guard<std::mutex> lock(mutex_);
      pending_requests_[id] = std::make_shared<MarketData>();
    }

    client_.reqMktData(id, contract, "", false, false, TagValueListSPtr());
    return id;
  }

  void tickPrice(TickerId tickerId, TickType field, double price,
                 const TickAttrib& attribs) override {
    std::lock_guard<std::mutex> lock(mutex_);
    auto it = pending_requests_.find(tickerId);
    if (it != pending_requests_.end()) {
      it->second->update_price(field, price);
    }
  }

  void tickSize(TickerId tickerId, TickType field, Decimal size) override {
    std::lock_guard<std::mutex> lock(mutex_);
    auto it = pending_requests_.find(tickerId);
    if (it != pending_requests_.end()) {
      it->second->update_size(field, size);
    }
  }
};
```

---

## Pattern 3: Order Management

### Order Lifecycle Tracking

```cpp
enum class OrderState {
  Pending,
  Submitted,
  PartiallyFilled,
  Filled,
  Cancelled,
  Rejected
};

struct OrderInfo {
  OrderId id;
  Contract contract;
  Order order;
  OrderState state;
  Decimal filled;
  Decimal remaining;
  double avg_fill_price;
};

class OrderManager : public DefaultEWrapper {
  std::mutex mutex_;
  std::map<OrderId, OrderInfo> orders_;
  OrderId next_order_id_ = 1;

  OrderId place_order(const Contract& contract, const Order& order) {
    OrderId id = next_order_id_++;

    {
      std::lock_guard<std::mutex> lock(mutex_);
      orders_[id] = OrderInfo{
        id, contract, order, OrderState::Pending,
        Decimal(0), order.totalQuantity, 0.0
      };
    }

    client_.placeOrder(id, contract, order);
    return id;
  }

  void orderStatus(OrderId orderId, const std::string& status,
                   Decimal filled, Decimal remaining, double avgFillPrice,
                   int permId, int parentId, double lastFillPrice,
                   int clientId, const std::string& whyHeld,
                   double mktCapPrice) override {
    std::lock_guard<std::mutex> lock(mutex_);
    auto it = orders_.find(orderId);
    if (it != orders_.end()) {
      it->second.filled = filled;
      it->second.remaining = remaining;
      it->second.avg_fill_price = avgFillPrice;

      if (status == "Filled") {
        it->second.state = OrderState::Filled;
      } else if (status == "Cancelled") {
        it->second.state = OrderState::Cancelled;
      } else if (filled > 0) {
        it->second.state = OrderState::PartiallyFilled;
      } else {
        it->second.state = OrderState::Submitted;
      }
    }
  }

  void openOrder(OrderId orderId, const Contract& contract,
                 const Order& order, const OrderState& orderState) override {
    std::lock_guard<std::mutex> lock(mutex_);
    auto it = orders_.find(orderId);
    if (it != orders_.end()) {
      it->second.contract = contract;
      it->second.order = order;
    }
  }
};
```

---

## Pattern 4: Error Handling

### Comprehensive Error Handling

```cpp
class ErrorHandler : public DefaultEWrapper {
  std::mutex mutex_;
  std::vector<ErrorInfo> errors_;

  void error(int id, int errorCode, const std::string& errorString,
             const std::string& advancedOrderRejectJson) override {
    std::lock_guard<std::mutex> lock(mutex_);

    ErrorSeverity severity;
    if (errorCode < 1100) {
      severity = ErrorSeverity::Error;
    } else if (errorCode < 2000) {
      severity = ErrorSeverity::Warning;
    } else {
      severity = ErrorSeverity::Info;
    }

    errors_.push_back({id, errorCode, errorString, severity});

    // Log based on severity
    if (severity == ErrorSeverity::Error) {
      logger_->error("TWS Error [{}]: {}", errorCode, errorString);
    } else if (severity == ErrorSeverity::Warning) {
      logger_->warn("TWS Warning [{}]: {}", errorCode, errorString);
    } else {
      logger_->info("TWS Info [{}]: {}", errorCode, errorString);
    }

    // Handle connection errors
    if (errorCode == 502 || errorCode == 504) {
      handle_connection_error();
    }
  }

  void handle_connection_error() {
    connected_ = false;
    // Attempt reconnection logic
  }
};
```

---

## Pattern 5: Thread Safety

### Safe Data Access

```cpp
class ThreadSafeDataStore {
  mutable std::shared_mutex mutex_;  // Read-write lock
  std::map<TickerId, Quote> quotes_;

  // Multiple readers allowed
  std::optional<Quote> get_quote(TickerId id) const {
    std::shared_lock<std::shared_mutex> lock(mutex_);
    auto it = quotes_.find(id);
    if (it != quotes_.end()) {
      return it->second;
    }
    return std::nullopt;
  }

  // Single writer
  void update_quote(TickerId id, const Quote& quote) {
    std::unique_lock<std::shared_mutex> lock(mutex_);
    quotes_[id] = quote;
  }

  // Called from EReader thread (single writer)
  void tickPrice(TickerId tickerId, TickType field, double price,
                 const TickAttrib& attribs) override {
    std::unique_lock<std::shared_mutex> lock(mutex_);
    quotes_[tickerId].update_price(field, price);
  }

  // Called from application thread (multiple readers)
  double get_last_price(TickerId id) const {
    std::shared_lock<std::shared_mutex> lock(mutex_);
    auto it = quotes_.find(id);
    if (it != quotes_.end()) {
      return it->second.last_price;
    }
    return 0.0;
  }
};
```

---

## Pattern 6: EReader Thread Management

### Proper Thread Lifecycle

```cpp
class TWSClient {
  EReaderOSSignal signal_;
  std::thread reader_thread_;
  std::atomic<bool> running_{false};

  void start_reader_thread() {
    running_ = true;
    reader_thread_ = std::thread([this]() {
      EReader reader(&client_, &signal_);
      reader.processMsgs();

      while (running_ && client_.isConnected()) {
        signal_.waitForSignal();
        reader.processMsgs();
      }
    });
  }

  void stop_reader_thread() {
    running_ = false;
    signal_.issueSignal();

    if (reader_thread_.joinable()) {
      reader_thread_.join();
    }
  }

  ~TWSClient() {
    stop_reader_thread();
    if (client_.isConnected()) {
      client_.eDisconnect();
    }
  }
};
```

---

## Pattern 7: Callback Exception Safety

### Safe Callback Execution

```cpp
class SafeEWrapper : public DefaultEWrapper {
  void tickPrice(TickerId tickerId, TickType field, double price,
                 const TickAttrib& attribs) override {
    try {
      // Your callback logic
      handle_tick_price(tickerId, field, price, attribs);
    } catch (const std::exception& e) {
      logger_->error("Exception in tickPrice callback: {}", e.what());
      // Don't let exceptions propagate to TWS API code
    } catch (...) {
      logger_->error("Unknown exception in tickPrice callback");
    }
  }

  // Macro helper for all callbacks
  #define SAFE_CALLBACK(callback_name, ...) \
    void callback_name(__VA_ARGS__) override { \
      try { \
        callback_name##_impl(__VA_ARGS__); \
      } catch (const std::exception& e) { \
        logger_->error("Exception in {}: {}", #callback_name, e.what()); \
      } \
    }
};
```

---

## Pattern 8: Contract Building

### Helper Functions for Contracts

```cpp
Contract make_stock_contract(const std::string& symbol,
                              const std::string& exchange = "SMART",
                              const std::string& currency = "USD") {
  Contract contract;
  contract.symbol = symbol;
  contract.secType = "STK";
  contract.exchange = exchange;
  contract.currency = currency;
  return contract;
}

Contract make_option_contract(const std::string& symbol,
                               const std::string& expiry,
                               double strike,
                               const std::string& right,  // "CALL" or "PUT"
                               const std::string& exchange = "SMART",
                               const std::string& currency = "USD") {
  Contract contract;
  contract.symbol = symbol;
  contract.secType = "OPT";
  contract.exchange = exchange;
  contract.currency = currency;
  contract.lastTradeDateOrContractMonth = expiry;
  contract.strike = strike;
  contract.right = right;
  contract.multiplier = "100";
  return contract;
}

Contract make_index_contract(const std::string& symbol,
                              const std::string& exchange = "CBOE",
                              const std::string& currency = "USD") {
  Contract contract;
  contract.symbol = symbol;
  contract.secType = "IND";
  contract.exchange = exchange;
  contract.currency = currency;
  return contract;
}
```

---

## Best Practices Summary

1. **Always use mutexes** when accessing shared data from callbacks
2. **Keep callbacks fast** - don't block the EReader thread
3. **Handle exceptions** in all callbacks to prevent crashes
4. **Track request IDs** to match responses with requests
5. **Wait for nextValidId** before placing orders
6. **Use connection state** to prevent operations when disconnected
7. **Log errors appropriately** based on error code ranges
8. **Clean up resources** properly in destructor

---

## References

- [EClient and EWrapper Architecture](../ECLIENT_EWRAPPER_ARCHITECTURE.md)
- [TWS API Quick Reference](./TWS_API_QUICK_REFERENCE.md)
- [TWS Integration Status](../TWS_INTEGRATION_STATUS.md)
