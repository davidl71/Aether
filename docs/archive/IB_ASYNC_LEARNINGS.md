# ib_async Library Learnings

This document summarizes learnings from the [ib_async](https://github.com/ib-api-reloaded/ib_async) library, a modern Python framework for the Interactive Brokers TWS API that provides both synchronous and asynchronous interfaces.

## Overview

`ib_async` is a Python library that serves as a successor to `ib_insync`, providing:

- **Asynchronous support** using Python's `asyncio`
- **Synchronous interface** for traditional blocking calls
- **Type annotations** for better code clarity
- **Modern Python patterns** and best practices

While `ib_async` is Python-based and our implementation is C++, we can learn valuable architectural patterns and design principles.

## Key Architectural Patterns

### 1. Dual Interface Pattern (Sync/Async)

**ib_async Approach:**

- Provides both synchronous and asynchronous methods
- Users choose the interface that fits their needs
- Async methods use `async/await` syntax
- Sync methods provide blocking behavior

**Example Pattern:**

```python

# Async interface

async def get_market_data(self, contract):
    await self.reqMktData(contract)
    return await self.wait_for_tick()

# Sync interface (wrapper around async)

def get_market_data_sync(self, contract):
    return asyncio.run(self.get_market_data(contract))
```

**Our C++ Implementation:**

- We use **threading** for async behavior (EReader thread)
- Methods are **non-blocking** by default (callbacks)
- We provide **synchronous wrappers** with timeouts
- Similar pattern but using C++ threading instead of async/await

**Comparison:**

```cpp
// Our approach: Non-blocking with callbacks
void request_market_data(const Contract& contract, MarketDataCallback callback) {
    client_.reqMktData(requestId, contract, ...);
    // Callback will be invoked when data arrives
}

// Could add synchronous wrapper:
std::optional<MarketData> request_market_data_sync(const Contract& contract, int timeout_ms) {
    std::promise<MarketData> promise;
    auto future = promise.get_future();

    request_market_data(contract, &promise {
        promise.set_value(data);
    });

    if (future.wait_for(std::chrono::milliseconds(timeout_ms)) == std::future_status::ready) {
        return future.get();
    }
    return std::nullopt;
}
```

### 2. Request-Response Correlation

**ib_async Pattern:**

- Tracks pending requests with unique IDs
- Correlates responses to requests
- Provides futures/promises for async operations

**Our Implementation:**

- We use `request_id` and `ticker_id` for correlation
- Callbacks are registered per request ID
- Similar pattern but with callback-based approach

**Potential Improvement:**

```cpp
// Could add request tracking with futures
class RequestTracker {
    std::unordered_map<int, std::promise<Response>> pending_requests_;

    std::future<Response> request_async(int request_id) {
        std::promise<Response> promise;
        auto future = promise.get_future();
        pending_requests_[request_id] = std::move(promise);
        return future;
    }

    void complete_request(int request_id, const Response& response) {
        if (pending_requests_.count(request_id)) {
            pending_requests_[request_id].set_value(response);
            pending_requests_.erase(request_id);
        }
    }
};
```

### 3. Connection State Management

**ib_async Pattern:**

- Clear connection state machine
- Automatic reconnection with backoff
- Connection health monitoring

**Our Implementation:**

- ✅ We have connection state management (`ConnectionState` enum)
- ✅ We have automatic reconnection with exponential backoff
- ✅ We have connection health monitoring
- **Status:** Already implemented, aligns with ib_async patterns

### 4. Error Handling and Recovery

**ib_async Pattern:**

- Comprehensive error categorization
- Automatic retry for transient errors
- Error callbacks with context

**Our Implementation:**

- ✅ We categorize errors (connection, authentication, system, informational)
- ✅ We provide error guidance messages
- ✅ We have error callbacks
- **Potential Improvement:** Could add automatic retry for specific error codes

### 5. Data Structure Management

**ib_async Pattern:**

- Maintains live data structures (contracts, orders, positions)
- Automatic updates via callbacks
- Thread-safe access patterns

**Our Implementation:**

- ✅ We maintain live data structures (`market_data_`, `orders_`, `positions_`)
- ✅ Automatic updates via EWrapper callbacks
- ✅ Thread-safe with mutexes
- **Status:** Already follows similar patterns

## Async Patterns in C++ vs Python

### Python (ib_async) - asyncio

```python
async def main():
    client = IB()
    await client.connect('127.0.0.1', 7497, clientId=1)

    contract = Stock('AAPL', 'SMART', 'USD')
    ticker = await client.reqMktData(contract)

    # Non-blocking wait for data
    await ticker.updateEvent
    print(ticker.marketPrice())
```

### C++ (Our Implementation) - Threading

```cpp
// Non-blocking request
client.request_market_data(contract, [](const MarketData& data) {
    // Callback invoked when data arrives
    std::cout << data.bid << std::endl;
});

// EReader thread processes messages in background
// Callbacks are invoked from EReader thread
```

## Key Learnings

### 1. Request Tracking

**Pattern:** Track pending requests and correlate responses

**Our Status:** ✅ Partially implemented

- We track request IDs
- We register callbacks per request
- **Could improve:** Add timeout handling for requests

### 2. Synchronous Wrappers

**Pattern:** Provide blocking wrappers around async operations

**Our Status:** ⚠️ Not fully implemented

- Our methods are callback-based (non-blocking)
- **Could add:** Synchronous wrappers using `std::future`/`std::promise`

### 3. Type Safety

**Pattern:** Use strong typing for better code clarity

**Our Status:** ✅ Implemented

- We use C++ types (`types::OptionContract`, `types::Order`, etc.)
- Type-safe conversions between TWS and our types

### 4. Connection Lifecycle

**Pattern:** Clear connection lifecycle management

**Our Status:** ✅ Implemented

- Connection states: Disconnected → Connecting → Connected → Error
- Automatic reconnection
- Health monitoring

### 5. Error Recovery

**Pattern:** Automatic recovery from transient errors

**Our Status:** ⚠️ Partially implemented

- We handle connection errors
- **Could improve:** Add retry logic for specific API errors

## Potential Improvements

### 1. Add Synchronous Wrappers

```cpp
// Current: Callback-based (async)
void request_market_data(const Contract& contract, MarketDataCallback callback);

// Could add: Synchronous wrapper
std::optional<MarketData> request_market_data_sync(
    const Contract& contract,
    int timeout_ms = 5000
) {
    std::promise<MarketData> promise;
    auto future = promise.get_future();
    std::atomic<bool> received{false};

    request_market_data(contract, &promise, &received {
        promise.set_value(data);
        received = true;
    });

    if (future.wait_for(std::chrono::milliseconds(timeout_ms)) == std::future_status::ready) {
        return future.get();
    }

    // Cancel request if timeout
    if (!received) {
        cancel_market_data(request_id);
    }

    return std::nullopt;
}
```

### 2. Request Timeout Handling

```cpp
class RequestManager {
    struct PendingRequest {
        std::chrono::steady_clock::time_point start_time;
        int timeout_ms;
        std::function<void()> timeout_callback;
    };

    std::unordered_map<int, PendingRequest> pending_requests_;

    void register_request(int request_id, int timeout_ms, std::function<void()> on_timeout) {
        pending_requests_[request_id] = {
            std::chrono::steady_clock::now(),
            timeout_ms,
            on_timeout
        };
    }

    void check_timeouts() {
        auto now = std::chrono::steady_clock::now();
        for (auto it = pending_requests_.begin(); it != pending_requests_.end();) {
            auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                now - it->second.start_time).count();
            if (elapsed > it->second.timeout_ms) {
                it->second.timeout_callback();
                it = pending_requests_.erase(it);
            } else {
                ++it;
            }
        }
    }
};
```

### 3. Automatic Retry for Transient Errors

```cpp
// Could add retry logic for specific error codes
void place_order_with_retry(const Order& order, int max_retries = 3) {
    int attempts = 0;
    while (attempts < max_retries) {
        int order_id = place_order(order);

        // Wait for order status
        auto status = wait_for_order_status(order_id, 5000);

        if (status == OrderStatus::Submitted) {
            return; // Success
        } else if (status == OrderStatus::Rejected) {
            // Check if retryable error
            if (is_retryable_error(last_error_code_)) {
                attempts++;
                std::this_thread::sleep_for(std::chrono::milliseconds(1000 * attempts));
                continue;
            } else {
                return; // Non-retryable error
            }
        }
    }
}
```

### 4. Event-Driven Architecture

**ib_async Pattern:**

- Uses events for coordination
- `updateEvent`, `fillEvent`, etc.

**Our Implementation:**

- We use callbacks (similar pattern)
- **Could improve:** Add event objects for better coordination

```cpp
// Could add event-based coordination
class MarketDataEvent {
    std::condition_variable cv_;
    std::mutex mutex_;
    MarketData data_;
    bool updated_{false};

public:
    void wait_for_update(int timeout_ms) {
        std::unique_lock<std::mutex> lock(mutex_);
        cv_.wait_for(lock, std::chrono::milliseconds(timeout_ms),
                    [this] { return updated_; });
    }

    void update(const MarketData& data) {
        std::lock_guard<std::mutex> lock(mutex_);
        data_ = data;
        updated_ = true;
        cv_.notify_all();
    }
};
```

## Comparison Summary

| Feature | ib_async (Python) | Our Implementation (C++) | Status |
|---------|-------------------|-------------------------|--------|
| Async Operations | asyncio/await | Threading + Callbacks | ✅ Implemented |
| Sync Wrappers | Yes | No | ⚠️ Could add |
| Request Tracking | Yes | Partial | ⚠️ Could improve |
| Connection Management | Yes | Yes | ✅ Implemented |
| Error Recovery | Yes | Partial | ⚠️ Could improve |
| Type Safety | Type hints | Strong types | ✅ Implemented |
| Thread Safety | N/A (single-threaded async) | Mutexes | ✅ Implemented |
| Health Monitoring | Yes | Yes | ✅ Implemented |
| Automatic Reconnection | Yes | Yes | ✅ Implemented |

## Recommendations

### High Priority

1. **Add Synchronous Wrappers**
   - Provide blocking versions of async methods
   - Use `std::future`/`std::promise` for coordination
   - Useful for simpler use cases

2. **Improve Request Timeout Handling**
   - Track pending requests with timeouts
   - Automatically cancel timed-out requests
   - Notify callers of timeouts

### Medium Priority

1. **Add Automatic Retry Logic**
   - Retry transient errors automatically
   - Configurable retry policies
   - Exponential backoff for retries

2. **Event-Based Coordination**
   - Add event objects for better coordination
   - Similar to ib_async's event pattern
   - Easier to use than raw callbacks

### Low Priority

1. **Request Correlation Improvements**
   - Better tracking of request-response pairs
   - Automatic cleanup of completed requests
   - Request statistics and monitoring

## Conclusion

While `ib_async` is Python-based and uses `asyncio`, we can learn valuable patterns:

1. **Dual Interface Pattern**: Providing both sync and async interfaces
2. **Request Tracking**: Better correlation of requests and responses
3. **Error Recovery**: Automatic retry for transient errors
4. **Event Coordination**: Event-based patterns for better coordination

Our current implementation already follows many best practices:

- ✅ Thread-safe data structures
- ✅ Connection state management
- ✅ Automatic reconnection
- ✅ Health monitoring
- ✅ Comprehensive error handling

**Main gaps:**

- ⚠️ Synchronous wrappers for simpler use cases
- ⚠️ Request timeout handling
- ⚠️ Automatic retry for transient errors

These improvements would make our API more user-friendly while maintaining the performance benefits of our callback-based async approach.

## References

- [ib_async GitHub Repository](https://github.com/ib-api-reloaded/ib_async)
- [ib_async Documentation](https://ib-api-reloaded.github.io/ib_async/)
- [Python asyncio Documentation](https://docs.python.org/3/library/asyncio.html)
- [C++ Futures and Promises](https://en.cppreference.com/w/cpp/thread/future)
