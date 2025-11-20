# EClient and EWrapper Architecture

This document explains the fundamental architecture of the TWS API, based on the [official IBKR Campus documentation](https://www.interactivebrokers.com/campus/ibkr-quant-news/the-eclient-and-ewrapper-api-classes/).

## Overview

The TWS API uses a **request-response pattern** with two primary components:

1. **EClient** (or `EClientSocket` in C++) - **Sends requests TO TWS**
2. **EWrapper** - **Receives callbacks FROM TWS**

This design separates concerns: your application sends commands through `EClient`, and receives data through `EWrapper` callbacks.

## EClient / EClientSocket

### Purpose

`EClientSocket` is responsible for **sending requests** from your application to TWS/IB Gateway.

### Key Responsibilities

- **Connection Management**: `eConnect()`, `eDisconnect()`, `isConnected()`
- **Order Management**: `placeOrder()`, `cancelOrder()`, `reqAllOpenOrders()`
- **Market Data**: `reqMktData()`, `cancelMktData()`, `reqHistoricalData()`
- **Account Data**: `reqAccountUpdates()`, `reqPositions()`
- **Contract Information**: `reqContractDetails()`, `reqMktDepth()`

### Usage Pattern

```cpp
// EClientSocket is used to send requests
EClientSocket client(wrapper, &signal);

// Connect to TWS
client.eConnect("127.0.0.1", 7497, 1);

// Send a request
client.reqMktData(tickerId, contract, "", false, false, TagValueListSPtr());

// Place an order
client.placeOrder(orderId, contract, order);
```

### In Our Implementation

```cpp
class TWSClient::Impl : public DefaultEWrapper {
    EClientSocket client_;  // Used to send requests to TWS

    // Example: Request market data
    client_.reqMktData(requestId, contract, "", false, false, TagValueListSPtr());

    // Example: Place order
    client_.placeOrder(orderId, contract, order);

    // Example: Request positions
    client_.reqPositions();
};
```

## EWrapper

### Purpose

`EWrapper` is an **interface** that defines callback methods for receiving data from TWS.

### Key Responsibilities

- **Connection Callbacks**: `connectAck()`, `nextValidId()`, `connectionClosed()`
- **Market Data Callbacks**: `tickPrice()`, `tickSize()`, `tickOptionComputation()`
- **Order Callbacks**: `orderStatus()`, `openOrder()`, `execDetails()`
- **Position Callbacks**: `position()`, `updatePortfolio()`
- **Account Callbacks**: `updateAccountValue()`, `updateAccountTime()`
- **Error Handling**: `error()`

### Implementation Pattern

You create a class that **inherits from `EWrapper`** (or `DefaultEWrapper`) and **overrides** the callback methods you need:

```cpp
class MyWrapper : public DefaultEWrapper {
public:
    // Override callbacks you need
    void tickPrice(TickerId tickerId, TickType field, double price,
                   const TickAttrib& attribs) override {
        // Handle market data update
    }

    void orderStatus(OrderId orderId, const std::string& status,
                    Decimal filled, Decimal remaining,
                    double avgFillPrice, ...) override {
        // Handle order status update
    }

    void error(int id, int errorCode, const std::string& errorString) override {
        // Handle errors
    }
};
```

### In Our Implementation

```cpp
class TWSClient::Impl : public DefaultEWrapper {
    // We inherit from DefaultEWrapper which provides default (empty) implementations
    // We override only the callbacks we need

    void tickPrice(TickerId tickerId, TickType field,
                   double price, const TickAttrib& attribs) override {
        // Process market data
    }

    void orderStatus(OrderId orderId, const std::string& status, ...) override {
        // Process order updates
    }

    void nextValidId(OrderId orderId) override {
        // Connection fully established
    }
};
```

## Integration Pattern

### The Connection

`EClientSocket` and `EWrapper` work together:

1. **Create EWrapper implementation** (your callback handler)
2. **Create EClientSocket** with a pointer to your EWrapper
3. **Send requests** via EClientSocket
4. **Receive responses** via EWrapper callbacks

### Code Structure

```cpp
// 1. Create your EWrapper implementation
class MyWrapper : public DefaultEWrapper {
    // Implement callbacks
};

// 2. Create EClientSocket with pointer to your wrapper
EReaderOSSignal signal;
MyWrapper wrapper;
EClientSocket client(&wrapper, &signal);

// 3. Connect
client.eConnect("127.0.0.1", 7497, 1);

// 4. Send requests (EClientSocket)
client.reqMktData(1, contract, "", false, false, TagValueListSPtr());

// 5. Receive callbacks (EWrapper)
// tickPrice() will be called when market data arrives
```

### In Our Implementation

```cpp
class TWSClient::Impl : public DefaultEWrapper {
    EReaderOSSignal signal_;      // Signal for EReader thread
    EClientSocket client_;        // Client for sending requests

    explicit Impl(const config::TWSConfig& config)
        : config_(config)
        , signal_(2000)
        , client_(this, &signal_)  // 'this' is the EWrapper implementation
    {
        // Enable async connection
        client_.asyncEConnect(true);
    }

    // EClientSocket methods (sending requests)
    void request_market_data(...) {
        client_.reqMktData(...);  // Send request via EClientSocket
    }

    // EWrapper callbacks (receiving responses)
    void tickPrice(...) override {
        // Receive market data via EWrapper callback
    }
};
```

## Request-Response Flow

### Example: Requesting Market Data

```
1. Application calls: client_.reqMktData(tickerId, contract, ...)
   └─> EClientSocket sends request to TWS

2. TWS processes request and starts sending market data

3. TWS sends market data updates
   └─> EReader thread receives messages
   └─> EWrapper callbacks are invoked:
       - tickPrice(tickerId, BID, price, ...)
       - tickPrice(tickerId, ASK, price, ...)
       - tickSize(tickerId, BID_SIZE, size)
       - ...

4. Your EWrapper implementation processes the callbacks
   └─> Update internal data structures
   └─> Trigger application callbacks
   └─> Update UI/logs
```

### Example: Placing an Order

```
1. Application calls: client_.placeOrder(orderId, contract, order)
   └─> EClientSocket sends order to TWS

2. TWS receives and processes order

3. TWS sends order status updates
   └─> EWrapper callbacks are invoked:
       - openOrder(orderId, contract, order, orderState)
       - orderStatus(orderId, "Submitted", ...)
       - orderStatus(orderId, "Filled", filled, remaining, ...)
       - execDetails(reqId, contract, execution)

4. Your EWrapper implementation processes the callbacks
   └─> Update order tracking
   └─> Notify application of status changes
```

## Threading Model

### EReader Thread

The TWS API uses a **dedicated thread** (EReader) to receive messages from TWS:

```cpp
// Start EReader thread to process incoming messages
EReader reader(&client, &signal);
reader.start();  // Starts background thread

// The EReader thread:
// 1. Reads messages from socket
// 2. Decodes messages
// 3. Calls appropriate EWrapper callbacks
```

### Thread Safety

**Important**: EWrapper callbacks are called from the **EReader thread**, not your main thread.

**Best Practices**:
- Use mutexes to protect shared data
- Don't perform long-running operations in callbacks
- Use condition variables for synchronization
- Queue work for main thread if needed

### In Our Implementation

```cpp
class TWSClient::Impl : public DefaultEWrapper {
    std::mutex data_mutex_;  // Protect market data
    std::mutex order_mutex_; // Protect orders

    void tickPrice(TickerId tickerId, TickType field, double price, ...) override {
        std::lock_guard<std::mutex> lock(data_mutex_);  // Thread-safe access
        // Update market data
    }

    void start_reader_thread() {
        auto reader = std::make_unique<EReader>(&client_, &signal_);
        reader->start();

        reader_thread_ = std::make_unique<std::thread>([this, r = std::move(reader)]() mutable {
            while (connected_) {
                signal_.waitForSignal();
                r->processMsgs();  // Processes messages and calls EWrapper callbacks
            }
        });
    }
};
```

## DefaultEWrapper

### Purpose

`DefaultEWrapper` is a **convenience base class** that provides default (empty) implementations of all EWrapper methods.

### Benefits

- **Only override what you need**: Don't implement every callback
- **Compile-time safety**: Compiler warns if you misspell method names
- **Cleaner code**: Focus on callbacks you actually use

### Usage

```cpp
class MyWrapper : public DefaultEWrapper {
    // Only override callbacks you need
    void tickPrice(...) override { /* your code */ }
    void orderStatus(...) override { /* your code */ }
    // All other callbacks have empty default implementations
};
```

### In Our Implementation

```cpp
class TWSClient::Impl : public DefaultEWrapper {
    // We override ~30 callbacks we actually use
    // The other ~100+ callbacks use default empty implementations
};
```

## Common Patterns

### Pattern 1: Request-Response with State Tracking

```cpp
class MyWrapper : public DefaultEWrapper {
    std::atomic<bool> request_complete_{false};
    std::mutex data_mutex_;
    MarketData latest_data_;

    void requestMarketData(int tickerId, const Contract& contract) {
        request_complete_ = false;
        client_.reqMktData(tickerId, contract, "", false, false, TagValueListSPtr());
    }

    void tickPrice(TickerId tickerId, TickType field, double price, ...) override {
        std::lock_guard<std::mutex> lock(data_mutex_);
        latest_data_.update(field, price);
        request_complete_ = true;
    }

    MarketData getLatestData() {
        std::lock_guard<std::mutex> lock(data_mutex_);
        return latest_data_;
    }
};
```

### Pattern 2: Callback Registration

```cpp
class MyWrapper : public DefaultEWrapper {
    using MarketDataCallback = std::function<void(const MarketData&)>;
    std::unordered_map<int, MarketDataCallback> callbacks_;

    void registerCallback(int tickerId, MarketDataCallback cb) {
        callbacks_[tickerId] = cb;
    }

    void tickPrice(TickerId tickerId, TickType field, double price, ...) override {
        if (callbacks_.count(tickerId)) {
            MarketData data;
            data.update(field, price);
            callbacks_[tickerId](data);  // Notify registered callback
        }
    }
};
```

### Pattern 3: Order Tracking

```cpp
class MyWrapper : public DefaultEWrapper {
    std::unordered_map<OrderId, Order> orders_;
    std::mutex order_mutex_;

    void placeOrder(OrderId orderId, const Contract& contract, const Order& order) {
        {
            std::lock_guard<std::mutex> lock(order_mutex_);
            orders_[orderId] = Order{...};  // Track order
        }
        client_.placeOrder(orderId, contract, order);
    }

    void orderStatus(OrderId orderId, const std::string& status, ...) override {
        std::lock_guard<std::mutex> lock(order_mutex_);
        if (orders_.count(orderId)) {
            orders_[orderId].status = status;  // Update tracked order
        }
    }
};
```

## Best Practices

### 1. Always Use DefaultEWrapper

**Good:**
```cpp
class MyWrapper : public DefaultEWrapper {
    // Only override what you need
};
```

**Avoid:**
```cpp
class MyWrapper : public EWrapper {
    // Must implement ALL ~130+ callbacks, even if empty
};
```

### 2. Thread Safety in Callbacks

**Always protect shared data:**
```cpp
void tickPrice(...) override {
    std::lock_guard<std::mutex> lock(data_mutex_);
    // Update shared data
}
```

### 3. Exception Handling

**Wrap callbacks in try-catch:**
```cpp
void tickPrice(...) override {
    try {
        // Your code
    } catch (const std::exception& e) {
        spdlog::error("Exception in tickPrice: {}", e.what());
    }
}
```

### 4. Keep Callbacks Fast

**Good:**
```cpp
void tickPrice(...) override {
    // Quick update to data structure
    // Queue work for main thread if needed
}
```

**Avoid:**
```cpp
void tickPrice(...) override {
    // Don't do heavy computation here
    // Don't make network calls
    // Don't block the EReader thread
}
```

### 5. Request State Management

**Track pending requests:**
```cpp
std::atomic<int> pending_requests_{0};

void requestMarketData(...) {
    pending_requests_++;
    client_.reqMktData(...);
}

void tickPrice(...) override {
    // Process data
    pending_requests_--;  // Mark request as complete
}
```

## Our Implementation Summary

### Architecture

```cpp
TWSClient::Impl
├── Inherits from: DefaultEWrapper (receives callbacks)
├── Contains: EClientSocket client_ (sends requests)
├── Contains: EReaderOSSignal signal_ (thread synchronization)
└── Contains: EReader thread (processes incoming messages)
```

### Request Flow

```
Application Code
    ↓
TWSClient::Impl::request_market_data()
    ↓
EClientSocket::reqMktData()
    ↓
TWS/IB Gateway
```

### Response Flow

```
TWS/IB Gateway
    ↓
EReader thread (reads socket)
    ↓
EWrapper callbacks (tickPrice, tickSize, etc.)
    ↓
TWSClient::Impl::tickPrice() (processes data)
    ↓
Application callbacks (notifies application)
```

## References

- [IBKR Campus: EClient and EWrapper](https://www.interactivebrokers.com/campus/ibkr-quant-news/the-eclient-and-ewrapper-api-classes/)
- [TWS API Quick Reference](https://www.interactivebrokers.com/download/C++APIQuickReference.pdf)
- [TWS API Documentation](https://interactivebrokers.github.io/tws-api/)
- [Essential Components of TWS API Programs (Video)](https://www.youtube.com/watch?v=n-9bdREECTQ)

## Key Takeaways

1. **EClientSocket** = Send requests TO TWS
2. **EWrapper** = Receive callbacks FROM TWS
3. **DefaultEWrapper** = Convenience base class (only override what you need)
4. **EReader thread** = Processes incoming messages and calls EWrapper callbacks
5. **Thread safety** = Always protect shared data in callbacks
6. **Exception handling** = Wrap callbacks in try-catch to prevent crashes
7. **Keep callbacks fast** = Don't block the EReader thread

Our implementation follows all these best practices and provides a robust, thread-safe wrapper around the TWS API.
