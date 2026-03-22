# Alpaca API Integration Architecture Design

**Date**: 2025-11-18
**Status**: Design Document
**Purpose**: Design architecture for integrating Alpaca API into the unified multi-broker system

---

## Overview

This document designs the Alpaca API integration architecture following the unified broker interface pattern established in `docs/MULTI_BROKER_ARCHITECTURE_DESIGN.md`.

---

## Research Summary

### Alpaca API Capabilities

**REST API:**

- Trading operations (orders, positions, account)
- Market data (quotes, trades, bars)
- Options trading support (v2 API)
- Multi-leg orders (spreads, combos)
- Paper trading and live trading

**WebSocket API:**

- Real-time quotes
- Trade updates
- Account updates
- Market data streaming

**Authentication:**

- OAuth 2.0 (API keys)
- Base URL: `https://api.alpaca.markets` (live) or `https://paper-api.alpaca.markets` (paper)
- Rate limits: 200 requests/minute (free tier)

**Options Trading:**

- Options chain data via REST API
- Options order placement
- Multi-leg orders (spreads, combos)
- Options positions tracking
- Real-time options quotes

---

## Architecture Design

### Unified Broker Interface

The Alpaca adapter will implement the `IBroker` interface defined in `docs/MULTI_BROKER_ARCHITECTURE_DESIGN.md`:

```cpp
class AlpacaAdapter : public IBroker {
public:
    // Connection
    bool connect() override;
    void disconnect() override;
    bool is_connected() const override;

    // Market Data
    int subscribe_market_data(const types::OptionContract& contract) override;
    void unsubscribe_market_data(int subscription_id) override;

    // Orders
    int place_order(const types::Order& order) override;
    int place_combo_order(const std::vector<types::Order>& legs) override;
    void cancel_order(int order_id) override;

    // Positions
    std::vector<types::Position> get_positions() override;

    // Account
    types::AccountInfo get_account_info() override;
};
```

---

## Implementation Details

### 1. Authentication

**API Key Management:**

```cpp
struct AlpacaConfig {
    std::string api_key_id;
    std::string api_secret_key;
    std::string base_url;  // paper-api.alpaca.markets or api.alpaca.markets
    bool paper_trading;
};
```

**Authentication Headers:**

```cpp
// All REST API requests include:
headers["APCA-API-KEY-ID"] = config.api_key_id;
headers["APCA-API-SECRET-KEY"] = config.api_secret_key;
```

### 2. Market Data

**REST API Endpoints:**

- `GET /v2/stocks/{symbol}/quotes/latest` - Latest quote
- `GET /v2/stocks/{symbol}/trades/latest` - Latest trade
- `GET /v2/stocks/{symbol}/bars/latest` - Latest bar

**WebSocket API:**

- `wss://stream.data.alpaca.markets/v2/{feed}` - Real-time quotes
- Subscribe to symbols: `{"action": "subscribe", "quotes": ["SPY"]}`

**Options Data:**

- `GET /v2/options/contracts` - Options chain
- `GET /v2/options/quotes/latest` - Latest options quote

**Implementation:**

```cpp
int AlpacaAdapter::subscribe_market_data(const types::OptionContract& contract) {
    // Convert contract to Alpaca symbol format
    std::string alpaca_symbol = convert_to_alpaca_symbol(contract);

    // Subscribe via WebSocket
    websocket_client_->subscribe_quote(alpaca_symbol);

    // Store subscription mapping
    int subscription_id = next_subscription_id_++;
    subscriptions_[subscription_id] = alpaca_symbol;

    return subscription_id;
}
```

### 3. Order Placement

**Single Order:**

```cpp
int AlpacaAdapter::place_order(const types::Order& order) {
    // Convert to Alpaca order format
    json alpaca_order = {
        {"symbol", convert_to_alpaca_symbol(order.contract)},
        {"qty", order.quantity},
        {"side", order.action == OrderAction::Buy ? "buy" : "sell"},
        {"type", "limit"},
        {"limit_price", order.limit_price},
        {"time_in_force", "day"}
    };

    // POST /v2/orders
    auto response = http_client_->post("/v2/orders", alpaca_order);

    // Parse response and return order ID
    return response["id"];
}
```

**Multi-Leg Order (Combo):**

```cpp
int AlpacaAdapter::place_combo_order(const std::vector<types::Order>& legs) {
    // Alpaca supports multi-leg orders via "legs" array
    json alpaca_order = {
        {"class", "bracket"},  // or "oco", "oto"
        {"symbol", "SPY"},
        {"legs", json::array()}
    };

    for (const auto& leg : legs) {
        json leg_order = {
            {"symbol", convert_to_alpaca_symbol(leg.contract)},
            {"qty", leg.quantity},
            {"side", leg.action == OrderAction::Buy ? "buy" : "sell"},
            {"type", "limit"},
            {"limit_price", leg.limit_price}
        };
        alpaca_order["legs"].push_back(leg_order);
    }

    // POST /v2/orders
    auto response = http_client_->post("/v2/orders", alpaca_order);

    return response["id"];
}
```

### 4. Contract Conversion

**Alpaca Symbol Format:**

- Stock: `SPY`
- Option: `SPY240119C00450000` (SPY + YYYYMMDD + C/P + Strike*1000)

**Conversion Function:**

```cpp
std::string convert_to_alpaca_symbol(const types::OptionContract& contract) {
    // Format: SYMBOL + YYYYMMDD + C/P + STRIKE*1000
    std::string symbol = contract.symbol;
    std::string expiry = contract.expiration;  // YYYYMMDD
    char right = contract.right == OptionType::Call ? 'C' : 'P';
    int strike_cents = static_cast<int>(contract.strike * 1000);

    return symbol + expiry + right + std::to_string(strike_cents);
}
```

### 5. Rate Limiting

**Implementation:**

```cpp
class RateLimiter {
    std::chrono::steady_clock::time_point last_request_;
    int requests_per_minute_ = 200;
    std::queue<std::chrono::steady_clock::time_point> request_times_;

public:
    void wait_if_needed() {
        // Remove requests older than 1 minute
        auto now = std::chrono::steady_clock::now();
        while (!request_times_.empty() &&
               (now - request_times_.front()) > std::chrono::minutes(1)) {
            request_times_.pop();
        }

        // Wait if at limit
        if (request_times_.size() >= requests_per_minute_) {
            auto wait_time = std::chrono::minutes(1) - (now - request_times_.front());
            std::this_thread::sleep_for(wait_time);
        }

        request_times_.push(std::chrono::steady_clock::now());
    }
};
```

---

## Data Flow

### Market Data Flow

```
Alpaca WebSocket → AlpacaAdapter::on_quote()
    → Convert to types::MarketData
    → Emit to subscribers
    → Strategy receives normalized data
```

### Order Flow

```
Strategy → AlpacaAdapter::place_order()
    → Convert to Alpaca format
    → POST /v2/orders
    → Parse response
    → Return order ID
    → Track order status
```

---

## Error Handling

### Common Errors

| Error Code | Description | Handling |
|------------|-------------|----------|
| 401 | Unauthorized | Re-authenticate, check API keys |
| 403 | Forbidden | Check account permissions, trading level |
| 429 | Rate limit exceeded | Wait and retry with backoff |
| 422 | Invalid request | Log error, return to caller |
| 500 | Server error | Retry with exponential backoff |

### Implementation

```cpp
void AlpacaAdapter::handle_error(int status_code, const json& error) {
    switch (status_code) {
        case 401:
            spdlog::error("Alpaca authentication failed: {}", error["message"]);
            // Re-authenticate
            break;
        case 429:
            spdlog::warn("Alpaca rate limit exceeded, backing off");
            rate_limiter_.backoff();
            break;
        case 422:
            spdlog::error("Alpaca invalid request: {}", error["message"]);
            break;
        default:
            spdlog::error("Alpaca API error {}: {}", status_code, error["message"]);
    }
}
```

---

## Testing Strategy

### Unit Tests

- Contract conversion functions
- Order format conversion
- Rate limiter logic
- Error handling

### Integration Tests

- Paper trading account connection
- Market data subscription
- Order placement (paper trading)
- Position tracking
- Error scenarios

### Test Scenarios

1. **Connection Test**
   - Connect to paper trading
   - Verify authentication
   - Check account info

2. **Market Data Test**
   - Subscribe to SPY options
   - Verify quotes received
   - Test unsubscribe

3. **Order Test**
   - Place single order (paper)
   - Place multi-leg order (paper)
   - Cancel order
   - Verify order status

4. **Error Test**
   - Invalid API keys
   - Rate limit exceeded
   - Invalid contract
   - Network timeout

---

## Configuration

### Configuration File

```json
{
  "brokers": {
    "alpaca": {
      "enabled": true,
      "api_key_id": "YOUR_API_KEY_ID",
      "api_secret_key": "YOUR_SECRET_KEY",
      "base_url": "https://paper-api.alpaca.markets",
      "paper_trading": true,
      "rate_limit_per_minute": 200
    }
  }
}
```

---

## Dependencies

### Required Libraries

- **HTTP Client**: `libcurl` or `cpp-httplib` for REST API
- **WebSocket Client**: `websocketpp` or `libwebsockets` for real-time data
- **JSON**: `nlohmann/json` for JSON parsing
- **Threading**: Standard library `std::thread`, `std::mutex`

### Optional Libraries

- **Rate Limiting**: Custom implementation or `ratelimit` library
- **Retry Logic**: Custom implementation or `cpp-retry` library

---

## Implementation Phases

### Phase 1: Basic REST API Client (Week 1)

- HTTP client setup
- Authentication
- Account info retrieval
- Basic order placement

### Phase 2: Market Data (Week 2)

- REST API market data
- WebSocket client setup
- Real-time quote subscription
- Data normalization

### Phase 3: Options Trading (Week 3)

- Options contract conversion
- Options chain retrieval
- Options order placement
- Multi-leg orders

### Phase 4: Integration (Week 4)

- Unified broker interface implementation
- Integration with order manager
- Integration with strategy
- Error handling and retry logic

---

## Success Criteria

- ✅ Alpaca adapter implements IBroker interface
- ✅ Market data subscription works
- ✅ Order placement works (paper trading)
- ✅ Multi-leg orders work
- ✅ Error handling comprehensive
- ✅ Rate limiting implemented
- ✅ Integration tests pass
- ✅ Paper trading validation successful

---

## References

- [Alpaca API Documentation](https://alpaca.markets/docs/api-documentation/)
- [Alpaca Options Trading](https://alpaca.markets/docs/api-documentation/how-to/options/)
- [Alpaca WebSocket API](https://alpaca.markets/docs/api-documentation/streaming/)
- [Multi-Broker Architecture Design](./research/architecture/MULTI_BROKER_ARCHITECTURE_DESIGN.md)
- [Unified Broker Interface](./research/architecture/MULTI_BROKER_ARCHITECTURE_DESIGN.md)

---

**Status**: ✅ Design Complete - Ready for Implementation
**Next Step**: Implement Alpaca adapter (T-35)
