# Multi-Broker Architecture Design

**Date**: 2025-11-18
**Status**: Design Phase
**Purpose**: Unified architecture for TWS API, IB Client Portal API, and Alpaca API integration

---

## Overview

This document outlines the architecture for integrating multiple broker APIs (TWS API, IB Client Portal API, and Alpaca API) through a unified interface. This design enables:

- **Broker Abstraction**: Single interface for all broker operations
- **Easy Integration**: Add new brokers without changing core logic
- **Broker Switching**: Select broker at runtime or configuration
- **Unified Data**: Common data structures across all brokers
- **Error Handling**: Consistent error handling across brokers

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  (BoxSpreadStrategy, OrderManager, RiskManager)            │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ Uses IBroker interface
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                   Broker Manager                            │
│  - BrokerFactory (creates brokers)                          │
│  - BrokerSelector (selects active broker)                   │
│  - ConfigurationManager (manages broker configs)            │
│  - FallbackStrategy (handles broker failures)               │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ Manages
                       │
        ┌──────────────┼──────────────┐
        │              │              │
┌───────▼──────┐ ┌────▼──────┐ ┌────▼──────┐
│  TWS Broker  │ │  Alpaca   │ │ IB Client │
│   Adapter    │ │  Adapter  │ │  Portal   │
│              │ │           │ │  Adapter  │
└───────┬──────┘ └────┬──────┘ └────┬──────┘
        │              │              │
        │              │              │
┌───────▼──────┐ ┌────▼──────┐ ┌────▼──────┐
│  TWS Client  │ │  Alpaca    │ │ IB Client │
│  (Socket)    │ │  REST API  │ │ Portal    │
│              │ │            │ │ REST API  │
└──────────────┘ └────────────┘ └───────────┘
```

---

## Unified Broker Interface

### IBroker Abstract Base Class

```cpp
// native/include/brokers/broker_interface.h

namespace brokers {

enum class BrokerType {
    TWS,
    IB_CLIENT_PORTAL,
    ALPACA
};

enum class ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error
};

struct BrokerCapabilities {
    bool supports_options;
    bool supports_multi_leg_orders;
    bool supports_real_time_data;
    bool supports_historical_data;
    int max_orders_per_second;
    int rate_limit_per_minute;
};

class IBroker {
public:
    virtual ~IBroker() = default;

    // Connection Management
    virtual bool connect() = 0;
    virtual void disconnect() = 0;
    virtual bool is_connected() const = 0;
    virtual ConnectionState get_connection_state() const = 0;
    virtual BrokerType get_broker_type() const = 0;
    virtual BrokerCapabilities get_capabilities() const = 0;

    // Market Data
    virtual int request_market_data(
        const types::OptionContract& contract,
        std::function<void(const types::MarketData&)> callback
    ) = 0;
    virtual void cancel_market_data(int request_id) = 0;
    virtual std::optional<types::MarketData> request_market_data_sync(
        const types::OptionContract& contract,
        int timeout_ms = 5000
    ) = 0;

    // Options Chain
    virtual std::vector<types::OptionContract> request_option_chain(
        const std::string& symbol,
        const std::string& expiry = ""
    ) = 0;

    // Contract Details
    virtual int request_contract_details(
        const types::OptionContract& contract,
        std::function<void(long conId)> callback
    ) = 0;
    virtual long request_contract_details_sync(
        const types::OptionContract& contract,
        int timeout_ms = 5000
    ) = 0;

    // Order Management
    virtual int place_order(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        double limit_price = 0.0,
        types::TimeInForce tif = types::TimeInForce::Day
    ) = 0;
    virtual bool cancel_order(int order_id) = 0;
    virtual std::optional<types::Order> get_order_status(int order_id) const = 0;

    // Multi-Leg Orders (Box Spreads)
    virtual int place_combo_order(
        const std::vector<types::OptionContract>& contracts,
        const std::vector<types::OrderAction>& actions,
        const std::vector<int>& quantities,
        const std::vector<long>& contract_ids,
        const std::vector<double>& limit_prices
    ) = 0;

    // Positions
    virtual std::vector<types::Position> get_positions() = 0;
    virtual std::optional<types::Position> get_position(
        const types::OptionContract& contract
    ) = 0;

    // Account Information
    virtual std::optional<types::AccountInfo> get_account_info() = 0;
    virtual double get_buying_power() = 0;
    virtual double get_net_liquidation_value() = 0;

    // Error Handling
    virtual void set_error_callback(
        std::function<void(int code, const std::string& msg)> callback
    ) = 0;
};
} // namespace brokers
```

---

## Broker Adapters

### 1. TWS Broker Adapter

**Purpose**: Wraps existing `TWSClient` to implement `IBroker` interface.

**Implementation**:

- Delegates to existing `TWSClient` methods
- Maps TWS-specific data to common types
- Handles TWS connection state
- Implements TWS-specific error handling

**File**: `native/src/brokers/tws_broker_adapter.cpp`

**Key Methods**:

```cpp
class TWSBrokerAdapter : public brokers::IBroker {
private:
    std::unique_ptr<tws::TWSClient> tws_client_;
    // ... implementation
};
```

### 2. IB Client Portal Adapter

**Purpose**: Implements `IBroker` interface using IB Client Portal REST API.

**Implementation**:

- OAuth 2.0 authentication
- REST API client (HTTP/HTTPS)
- Session token management
- JSON request/response handling

**File**: `native/src/brokers/ib_client_portal_adapter.cpp`

**Key Methods**:

```cpp
class IBClientPortalAdapter : public brokers::IBroker {
private:
    std::string base_url_;
    std::string session_token_;
    std::unique_ptr<HttpClient> http_client_;
    // ... implementation
};
```

**Authentication Flow**:

1. Redirect to IB login page
2. User authenticates
3. Receive OAuth code
4. Exchange code for session token
5. Use session token for API requests

### 3. Alpaca Broker Adapter

**Purpose**: Implements `IBroker` interface using Alpaca REST API.

**Implementation**:

- API key authentication
- REST API client
- Rate limiting (200 req/min)
- WebSocket for real-time data (optional)

**File**: `native/src/brokers/alpaca_broker_adapter.cpp`

**Key Methods**:

```cpp
class AlpacaBrokerAdapter : public brokers::IBroker {
private:
    std::string api_key_;
    std::string api_secret_;
    std::string base_url_;  // Paper or live
    std::unique_ptr<HttpClient> http_client_;
    // ... implementation
};
```

**Note**: Verify Alpaca options trading support in 2025. If not available, mark as "stocks/crypto only" in capabilities.

---

## Broker Manager

### BrokerFactory

Creates broker instances based on configuration.

```cpp
class BrokerFactory {
public:
    static std::unique_ptr<brokers::IBroker> create(
        BrokerType type,
        const BrokerConfig& config
    );
};
```

### BrokerSelector

Manages broker selection and switching.

```cpp
class BrokerSelector {
public:
    void set_active_broker(BrokerType type);
    brokers::IBroker* get_active_broker();
    bool switch_broker(BrokerType type);
    std::vector<BrokerType> get_available_brokers() const;
};
```

### Fallback Strategy

Handles broker failures and automatic fallback.

```cpp
class FallbackStrategy {
public:
    void on_broker_failure(BrokerType failed_broker);
    BrokerType select_fallback_broker(BrokerType failed_broker);
    bool should_retry(BrokerType broker, int attempt_count);
};
```

---

## Data Normalization

### Option Contract Mapping

Each broker has different contract formats. Normalize to common `types::OptionContract`:

**TWS API**:

- Uses `Contract` object with `conId`, `symbol`, `lastTradeDateOrContractMonth`, etc.
- Map to `OptionContract` structure

**IB Client Portal**:

- JSON format: `{"symbol": "SPY", "expiry": "20250117", "strike": 500, "right": "C"}`
- Map to `OptionContract` structure

**Alpaca API**:

- JSON format: `{"symbol": "SPY250117C00500000", "underlying": "SPY"}`
- Parse symbol to extract expiry, strike, type
- Map to `OptionContract` structure

### Market Data Mapping

Normalize bid/ask/last/volume across all brokers:

```cpp
types::MarketData normalize_market_data(
    BrokerType broker,
    const BrokerSpecificData& data
);
```

### Order Status Mapping

Normalize order statuses to common `types::OrderStatus` enum:

- TWS: `PreSubmitted`, `Submitted`, `Filled`, `Cancelled`
- IB Client Portal: `PreSubmitted`, `Submitted`, `Filled`, `Cancelled`
- Alpaca: `new`, `accepted`, `filled`, `canceled`

---

## Configuration

### Broker Configuration Schema

```json
{
  "brokers": {
    "active": "TWS",
    "tws": {
      "enabled": true,
      "host": "127.0.0.1",
      "port": 7497,
      "client_id": 1
    },
    "ib_client_portal": {
      "enabled": true,
      "base_url": "https://localhost:5001",
      "oauth": {
        "client_id": "...",
        "redirect_uri": "..."
      }
    },
    "alpaca": {
      "enabled": true,
      "api_key": "...",
      "api_secret": "...",
      "base_url": "https://paper-api.alpaca.markets",
      "paper_trading": true
    }
  }
}
```

---

## Error Handling

### Unified Error Codes

```cpp
enum class BrokerError {
    ConnectionFailed,
    AuthenticationFailed,
    RateLimitExceeded,
    InvalidContract,
    OrderRejected,
    InsufficientFunds,
    MarketDataUnavailable
};
```

### Error Mapping

Map broker-specific errors to unified error codes:

- TWS: Error codes 502, 504, 1100, etc. → `BrokerError`
- IB Client Portal: HTTP status codes → `BrokerError`
- Alpaca: HTTP status codes → `BrokerError`

---

## Migration Path

### Phase 1: Create Interface and TWS Adapter

1. Define `IBroker` interface
2. Create `TWSBrokerAdapter` wrapping existing `TWSClient`
3. Test with existing code

### Phase 2: Refactor OrderManager

1. Change `OrderManager` to use `IBroker*` instead of `TWSClient*`
2. Update all method calls to use interface
3. Test with TWS adapter

### Phase 3: Add IB Client Portal Adapter

1. Implement `IBClientPortalAdapter`
2. Add OAuth authentication
3. Test with paper trading

### Phase 4: Add Alpaca Adapter

1. Implement `AlpacaBrokerAdapter`
2. Add API key authentication
3. Test with paper trading (if options supported)

### Phase 5: Add Broker Manager

1. Implement `BrokerFactory`
2. Implement `BrokerSelector`
3. Add broker switching UI (if needed)
4. Test broker switching

---

## Testing Strategy

### Unit Tests

- Test each adapter independently
- Mock broker APIs
- Test data normalization
- Test error handling

### Integration Tests

- Test with real broker APIs (paper trading)
- Test broker switching
- Test fallback strategies
- Test concurrent operations

### End-to-End Tests

- Test full trading flow with each broker
- Test box spread execution
- Test error recovery

---

## Security Considerations

1. **API Keys**: Store in secure configuration, never commit
2. **OAuth Tokens**: Secure token storage and refresh
3. **Session Management**: Secure session token handling
4. **Rate Limiting**: Respect broker rate limits
5. **Error Logging**: Don't log sensitive data

---

## Performance Considerations

1. **Connection Pooling**: Reuse HTTP connections
2. **Rate Limiting**: Implement client-side rate limiting
3. **Caching**: Cache market data when appropriate
4. **Async Operations**: Use async/await for non-blocking operations
5. **Connection Monitoring**: Monitor broker connection health

---

## Future Enhancements

1. **Simultaneous Multi-Broker**: Support trading on multiple brokers simultaneously
2. **Smart Routing**: Route orders to best broker based on price/liquidity
3. **Broker Comparison**: Compare prices across brokers
4. **Load Balancing**: Distribute load across brokers
5. **Broker Health Monitoring**: Monitor broker availability and performance

---

## References

- [TWS API Documentation](https://interactivebrokers.github.io/tws-api/)
- [IB Client Portal API](https://www.interactivebrokers.com/api/doc.html)
- [Alpaca API Documentation](https://alpaca.markets/docs/api-documentation/)
- [Adapter Pattern](https://refactoring.guru/design-patterns/adapter)
- [Strategy Pattern](https://refactoring.guru/design-patterns/strategy)
