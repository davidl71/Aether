# EWrapper Best Practices

This document summarizes best practices for implementing EWrapper callbacks in the TWS API, based on industry standards and common patterns.

## Overview

The `EWrapper` interface is the callback mechanism for receiving data from TWS/IB Gateway. Proper implementation of EWrapper callbacks is critical for:

- Order tracking and recovery
- Position management
- Market data handling
- Error handling and diagnostics

## Key EWrapper Callbacks

### Connection Callbacks

1. **`connectAck()`**: Called when socket connection is established and server version is received
   - Indicates connection is progressing
   - Should not be considered "fully connected" yet
   - Wait for `nextValidId()` for full confirmation

2. **`managedAccounts()`**: Called with list of managed accounts
   - Received after `connectAck()`
   - Indicates connection is progressing well
   - Happens before `nextValidId()`

3. **`nextValidId()`**: Called with the next valid order ID
   - **This is the final confirmation of a fully established connection**
   - Connection is ready for trading operations
   - Best practice: Request open orders here to sync state

4. **`connectionClosed()`**: Called when connection is closed
   - Trigger reconnection logic if auto-reconnect is enabled
   - Clean up resources

### Order Management Callbacks

1. **`openOrder()`**: Called for each open order
   - **Called for ALL open orders**, including ones from previous sessions
   - Important for order recovery after reconnection
   - Use `reqAllOpenOrders()` after connection to sync state
   - `OrderState` contains status string, but filled quantity/price come from `orderStatus()`

2. **`openOrderEnd()`**: Called after all open orders have been sent
   - Indicates open order sync is complete
   - Useful for tracking order recovery completion

3. **`orderStatus()`**: Called when order status changes
   - Provides filled quantity, remaining quantity, average fill price
   - More detailed than `openOrder()` for fill information
   - Primary source for tracking order fills

4. **`execDetails()`**: Called for each execution/fill
   - Provides execution details (shares, price, time)
   - Called for each fill, even partial fills

### Position & Portfolio Callbacks

1. **`updatePortfolio()`**: Called for each position in the portfolio
   - Provides real-time position updates
   - Includes market price, market value, average cost, P&L
   - Important for tracking current positions
   - May include positions from previous sessions

2. **`position()`**: Called for each position when requesting positions
   - Use `reqPositions()` to request all positions
   - Called once per position, then `positionEnd()`

3. **`positionEnd()`**: Called after all positions have been sent
   - Indicates position request is complete

### Account Callbacks

1. **`updateAccountValue()`**: Called for account value updates
   - Provides account information (NetLiquidation, CashBalance, BuyingPower, etc.)
   - Called multiple times with different keys
   - Use `reqAccountUpdates()` to subscribe to account updates

2. **`updateAccountTime()`**: Called with account update timestamp
   - Indicates when account data was last updated

3. **`accountDownloadEnd()`**: Called when account download is complete
   - Indicates account sync is finished

### Market Data Callbacks

1. **`tickPrice()`**: Called for price updates (bid, ask, last, etc.)
   - Real-time market data
   - Use `reqMktData()` to subscribe

2. **`tickSize()`**: Called for size updates (bid size, ask size, volume, etc.)
   - Real-time market data
   - Use `reqMktData()` to subscribe

3. **`tickOptionComputation()`**: Called for option Greeks (delta, gamma, vega, theta)
   - Real-time option analytics
   - Requires option market data subscription

### Error Handling

1. **`error()`**: Called for all errors and informational messages
   - Error codes:
     - 500-599: Connection errors (e.g., 502 = connection rejected)
     - 1100-1199: System messages (e.g., 1100 = connection lost, 1101 = connection restored)
     - 162, 200: Authentication/authorization errors
     - 2100-2999: Informational messages
   - Always check error codes and provide actionable guidance

## Best Practices

### 1. Order Recovery After Reconnection

**Pattern**: Request open orders immediately after `nextValidId()`:

```cpp
void nextValidId(OrderId orderId) override {
    // Connection fully established
    connected_ = true;

    // Sync open orders for recovery
    client_.reqAllOpenOrders();
}
```

**Why**: After reconnection, you need to sync your order state with TWS. `openOrder()` will be called for each open order, allowing you to rebuild your order tracking.

### 2. Proper Order Status Tracking

**Pattern**: Use both `openOrder()` and `orderStatus()`:

- `openOrder()`: Provides order state (status string) for all open orders
- `orderStatus()`: Provides detailed fill information (filled quantity, avg price)

**Why**: `OrderState` in `openOrder()` only contains the status string. Fill details come from `orderStatus()`.

### 3. Position Syncing

**Pattern**: Request positions after connection and use `updatePortfolio()` for real-time updates:

```cpp
void nextValidId(OrderId orderId) override {
    // After connection established
    client_.reqPositions();  // Request all positions
    // updatePortfolio() will be called for real-time updates
}
```

**Why**: `updatePortfolio()` provides real-time position updates, but you should also request positions explicitly to ensure you have the full state.

### 4. Account Updates Subscription

**Pattern**: Subscribe to account updates after connection:

```cpp
void nextValidId(OrderId orderId) override {
    // Subscribe to account updates
    client_.reqAccountUpdates(true, "DU123456");  // true = subscribe
}
```

**Why**: Account values change frequently (cash balance, buying power, etc.). Subscribing ensures you receive real-time updates.

### 5. Error Handling

**Pattern**: Provide actionable error messages:

```cpp
void error(int id, int errorCode, const std::string& errorString) override {
    if (errorCode == 502) {
        spdlog::error("Connection rejected. Check TWS API settings.");
    } else if (errorCode == 162 || errorCode == 200) {
        spdlog::error("Authentication required. Check TWS for connection prompt.");
    }
    // ... handle other errors
}
```

**Why**: Users need clear guidance on how to resolve connection and authentication issues.

### 6. Thread Safety

**Pattern**: Use mutexes to protect shared state:

```cpp
void openOrder(OrderId orderId, ...) override {
    std::lock_guard<std::mutex> lock(order_mutex_);
    // Update order state
}
```

**Why**: EWrapper callbacks are called from the EReader thread, which is separate from your main thread. All shared state must be protected.

### 7. Connection State Management

**Pattern**: Only consider connection "ready" after `nextValidId()`:

```cpp
void connectAck() override {
    // Connection progressing, but not ready yet
}

void nextValidId(OrderId orderId) override {
    // NOW connection is fully ready
    connected_ = true;
}
```

**Why**: `connectAck()` only indicates socket connection. `nextValidId()` confirms TWS is ready to accept orders.

## Implementation Status

Our current implementation follows these best practices:

- ✅ **Order Recovery**: `reqAllOpenOrders()` called in `nextValidId()`
- ✅ **Order Tracking**: Both `openOrder()` and `orderStatus()` implemented
- ✅ **Position Tracking**: `updatePortfolio()` implemented with proper state management
- ✅ **Error Handling**: Comprehensive error code handling with actionable messages
- ✅ **Thread Safety**: All callbacks use mutexes to protect shared state
- ✅ **Connection State**: Proper state management with `nextValidId()` as final confirmation
- ✅ **Account Updates**: `updateAccountValue()` implemented for account tracking

## References

- [IB API Quick Reference](https://www.interactivebrokers.com/download/C++APIQuickReference.pdf)
- [IB API Reference Guide](https://www.aesinternational.com/hubfs/Independent-Reviews/Interactive-Brokers-API-Reference-Guide.pdf)
- TWS API Documentation (included with TWS API download)
