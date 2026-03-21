# Broker Adapter Design

**Version:** 1.0.0
**Last Updated:** 2025-12-25
**Status:** Design Document

## Overview

This document designs a unified broker adapter system that supports multiple brokers (IBKR TWS, IBKR Client Portal, Alpaca) through a common interface. The design is based on research findings from T-142, T-143, and T-144.

## Architecture Pattern

### Unified Interface (IBroker)

The `IBroker` interface (`native/include/brokers/broker_interface.h`) provides a common API for all broker adapters:

```cpp
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
  virtual int request_market_data(...) = 0;
  virtual void cancel_market_data(int request_id) = 0;
  virtual std::optional<MarketData> request_market_data_sync(...) = 0;

  // Options Chain
  virtual std::vector<OptionContract> request_option_chain(...) = 0;

  // Order Management
  virtual int place_order(...) = 0;
  virtual bool cancel_order(int order_id) = 0;
  virtual int place_combo_order(...) = 0;  // Multi-leg orders (box spreads)

  // Positions & Account
  virtual std::vector<Position> get_positions() = 0;
  virtual std::optional<AccountInfo> get_account_info() = 0;
};
```

## Broker Adapters

### 1. TWS Adapter (Existing)

**File:** `native/src/brokers/tws_adapter.cpp`
**Status:** ✅ Implemented
**API Type:** Socket-based (TWS API)

**Key Features:**

- Direct socket connection to TWS/Gateway
- EClient/EWrapper pattern
- Comprehensive options support
- Multi-leg order support

### 2. IB Client Portal Adapter (Design)

**File:** `native/src/brokers/ib_client_portal_adapter.cpp` (to be created)
**Status:** ⏳ Design phase
**API Type:** REST API (OAuth 1.0a/2.0)

**Design Based on T-143 Research:**

**Authentication:**

- OAuth 1.0a for individual accounts
- OAuth 2.0 for institutional accounts
- Client Portal Gateway required (Java-based, local installation)
- Automatic session token renewal
- Web browser authentication on same machine

**Implementation Pattern:**

```cpp
class IBClientPortalAdapter : public IBroker {
private:
  struct Config {
    std::string gateway_url = "https://localhost:5001";
    std::string oauth_consumer_key;
    std::string oauth_consumer_secret;
    std::string oauth_token;
    std::string oauth_token_secret;
    bool paper_trading = true;
  };

  Config config_;
  HTTPClient http_client_;  // REST API client
  std::string session_token_;
  std::chrono::system_clock::time_point token_expiry_;

  bool ensure_session() {
    if (session_expired()) {
      return refresh_session();
    }
    return true;
  }

  bool refresh_session() {
    // OAuth 1.0a flow or token refresh
    // Store session token
    return true;
  }
};
```

**Key Endpoints:**

- `/iserver/account/{account_id}/positions` - Get positions
- `/iserver/marketdata/snapshot` - Market data
- `/iserver/secdef/search` - Contract search
- `/iserver/account/{account_id}/orders` - Order management

**Error Handling:**

- Handle gateway connection errors gracefully
- Retry with exponential backoff
- Fallback to TWS adapter if gateway unavailable

### 3. Alpaca Adapter (Design)

**File:** `native/src/brokers/alpaca_adapter.cpp` (to be created)
**Status:** ⏳ Design phase
**API Type:** REST API (API Key authentication)

**Design Based on T-142 Research:**

**Authentication:**

- API key-based (APCA-API-KEY-ID, APCA-API-SECRET-KEY headers)
- Paper trading: `paper-api.alpaca.markets`
- Live trading: `api.alpaca.markets`
- Store keys in environment variables (secure)

**Rate Limiting:**

- Trading API: 200 requests/minute
- Market Data API: 10,000 requests/minute (paid subscription)
- Monitor response headers: `X-Ratelimit-Limit`, `X-Ratelimit-Remaining`, `X-Ratelimit-Reset`
- Implement exponential backoff for 429 errors
- Use `Retry-After` header for retry timing

**Implementation Pattern:**

```cpp
class AlpacaAdapter : public IBroker {
private:
  struct Config {
    std::string api_key_id;
    std::string api_secret_key;
    std::string base_url = "https://paper-api.alpaca.markets";
    std::string data_url = "https://data.alpaca.markets";
    bool paper_trading = true;
  };

  Config config_;
  HTTPClient http_client_;
  RateLimiter rate_limiter_;  // Track rate limits

  std::map<std::string, std::string> get_headers() {
    return {
      {"APCA-API-KEY-ID", config_.api_key_id},
      {"APCA-API-SECRET-KEY", config_.api_secret_key}
    };
  }

  HTTPResponse make_request_with_retry(const std::string& endpoint,
                                       const std::string& method = "GET",
                                       const json& body = {}) {
    int retries = 0;
    int backoff_factor = 1;

    while (retries < 5) {
      auto response = http_client_.request(endpoint, method, get_headers(), body);

      if (response.status_code == 429) {
        int retry_after = response.headers.get("Retry-After", backoff_factor);
        std::this_thread::sleep_for(std::chrono::seconds(retry_after));
        backoff_factor *= 2;
        retries++;
        continue;
      }

      // Update rate limit tracking
      rate_limiter_.update_limits(
        response.headers.get("X-Ratelimit-Remaining", 0),
        response.headers.get("X-Ratelimit-Reset", 0)
      );

      return response;
    }

    throw AlpacaError("Max retries exceeded");
  }
};
```

**Key Endpoints:**

- `/v2/account` - Account information
- `/v2/positions` - Get positions
- `/v2/orders` - Order management
- `/v2/options/contracts` - Options chain
- `/v2/options/positions` - Options positions

**Options Trading:**

- Level 3 options trading support (announced Feb 2025)
- Multi-leg orders (up to 4 legs) - perfect for box spreads
- OCC symbol format for options

## Broker Selection & Switching (Based on T-144 Research)

### Broker Factory Pattern

```cpp
class BrokerFactory {
public:
  static std::unique_ptr<IBroker> create_broker(
    const BrokerConfig& config,
    BrokerSelectionStrategy strategy = BrokerSelectionStrategy::Primary
  ) {
    switch (strategy) {
      case BrokerSelectionStrategy::Primary:
        return create_primary_broker(config);
      case BrokerSelectionStrategy::PerformanceBased:
        return create_performance_based_broker(config);
      case BrokerSelectionStrategy::Fallback:
        return create_fallback_broker(config);
      default:
        return create_primary_broker(config);
    }
  }

private:
  static std::unique_ptr<IBroker> create_primary_broker(const BrokerConfig& config) {
    // Return configured primary broker (IBKR TWS by default)
    return std::make_unique<TWSAdapter>(config.tws);
  }

  static std::unique_ptr<IBroker> create_performance_based_broker(const BrokerConfig& config) {
    // Algo wheel pattern: Select broker based on performance metrics
    // Track: execution speed, fill rates, costs, reliability
    BrokerMetrics metrics = get_broker_metrics();
    if (metrics.alpaca.fill_rate > metrics.ibkr.fill_rate) {
      return std::make_unique<AlpacaAdapter>(config.alpaca);
    }
    return std::make_unique<TWSAdapter>(config.tws);
  }

  static std::unique_ptr<IBroker> create_fallback_broker(const BrokerConfig& config) {
    // Fallback chain: Primary → Secondary → Tertiary
    try {
      return create_primary_broker(config);
    } catch (...) {
      try {
        return std::make_unique<AlpacaAdapter>(config.alpaca);
      } catch (...) {
        return std::make_unique<IBClientPortalAdapter>(config.ib_portal);
      }
    }
  }
};
```

### Broker Manager

```cpp
class BrokerManager {
private:
  std::vector<std::unique_ptr<IBroker>> brokers_;
  IBroker* primary_broker_ = nullptr;
  IBroker* secondary_broker_ = nullptr;

public:
  void add_broker(std::unique_ptr<IBroker> broker, bool is_primary = false) {
    brokers_.push_back(std::move(broker));
    if (is_primary) {
      primary_broker_ = brokers_.back().get();
    }
  }

  IBroker* get_broker(BrokerType type) {
    for (auto& broker : brokers_) {
      if (broker->get_broker_type() == type) {
        return broker.get();
      }
    }
    return primary_broker_;  // Fallback to primary
  }

  IBroker* get_primary() { return primary_broker_; }

  bool switch_primary(BrokerType new_primary) {
    auto* new_broker = get_broker(new_primary);
    if (new_broker && new_broker->is_connected()) {
      primary_broker_ = new_broker;
      return true;
    }
    return false;
  }
};
```

## Data Normalization

### Unified Data Models

All brokers return data in unified formats:

**Position:**

```cpp
struct Position {
  std::string symbol;
  std::string broker;  // "ibkr", "alpaca", "ib_portal"
  int quantity;
  double avg_cost;
  double mark_price;
  double unrealized_pnl;
  Currency currency;  // Normalized to USD
};
```

**Order:**

```cpp
struct Order {
  int order_id;  // Broker-specific, but normalized
  std::string broker_order_id;  // Original broker ID
  OrderStatus status;
  std::vector<OrderLeg> legs;  // For multi-leg orders
  double limit_price;
  TimeInForce tif;
};
```

**Market Data:**

```cpp
struct MarketData {
  double bid;
  double ask;
  double last;
  int bid_size;
  int ask_size;
  std::chrono::system_clock::time_point timestamp;
};
```

### Field Mapping

Broker-specific data is mapped to unified models:

```cpp
class DataNormalizer {
public:
  static Position normalize_ibkr_position(const IBKRPosition& ibkr_pos) {
    Position pos;
    pos.symbol = ibkr_pos.localSymbol;
    pos.broker = "ibkr";
    pos.quantity = ibkr_pos.position;
    pos.avg_cost = ibkr_pos.averageCost;
    pos.mark_price = ibkr_pos.markPrice;
    pos.currency = normalize_currency(ibkr_pos.currency);
    return pos;
  }

  static Position normalize_alpaca_position(const AlpacaPosition& alpaca_pos) {
    Position pos;
    pos.symbol = alpaca_pos.symbol;
    pos.broker = "alpaca";
    pos.quantity = std::stoi(alpaca_pos.qty);
    pos.avg_cost = std::stod(alpaca_pos.avg_entry_price);
    pos.mark_price = std::stod(alpaca_pos.current_price);
    pos.currency = Currency::USD;  // Alpaca is USD-only
    return pos;
  }
};
```

## Error Handling & Fallback

### Fallback Strategy

```cpp
class BrokerWithFallback {
private:
  IBroker* primary_;
  IBroker* fallback_;

public:
  std::vector<Position> get_positions() {
    try {
      return primary_->get_positions();
    } catch (const BrokerError& e) {
      spdlog::warn("Primary broker failed: {}, trying fallback", e.what());
      try {
        return fallback_->get_positions();
      } catch (const BrokerError& e2) {
        spdlog::error("All brokers failed: {}", e2.what());
        throw;
      }
    }
  }
};
```

### Retry Logic

- Exponential backoff for transient errors
- Circuit breaker pattern for persistent failures
- Automatic failover to secondary broker
- Health checks to detect broker availability

## Integration Points

### 1. Order Manager

```cpp
class OrderManager {
private:
  BrokerManager broker_manager_;

public:
  int place_box_spread(const BoxSpread& spread) {
    auto* broker = broker_manager_.get_primary();

    // Convert box spread to broker-specific format
    auto order = convert_to_broker_order(spread, broker->get_broker_type());

    // Place order via unified interface
    return broker->place_combo_order(
      order.contracts,
      order.actions,
      order.quantities,
      order.contract_ids,
      order.limit_prices
    );
  }
};
```

### 2. Portfolio Aggregator

```cpp
class PortfolioAggregator {
private:
  BrokerManager broker_manager_;

public:
  Portfolio get_combined_portfolio() {
    Portfolio portfolio;

    // Get positions from all brokers
    for (auto& broker : broker_manager_.get_all_brokers()) {
      auto positions = broker->get_positions();
      for (auto& pos : positions) {
        portfolio.add_position(pos);
      }
    }

    // Aggregate by symbol, normalize currency
    return portfolio.aggregate();
  }
};
```

## Configuration

### Broker Configuration

```json
{
  "brokers": {
    "primary": "ibkr_tws",
    "fallback": "alpaca",
    "ibkr_tws": {
      "host": "127.0.0.1",
      "port": 7497,
      "client_id": 1
    },
    "ibkr_portal": {
      "gateway_url": "https://localhost:5001",
      "oauth_consumer_key": "${IBKR_OAUTH_KEY}",
      "oauth_consumer_secret": "${IBKR_OAUTH_SECRET}",
      "paper_trading": true
    },
    "alpaca": {
      "api_key_id": "${ALPACA_API_KEY_ID}",
      "api_secret_key": "${ALPACA_API_SECRET_KEY}",
      "base_url": "https://paper-api.alpaca.markets",
      "paper_trading": true
    }
  },
  "broker_selection": {
    "strategy": "performance_based",
    "metrics": {
      "track_fill_rate": true,
      "track_execution_speed": true,
      "track_costs": true
    }
  }
}
```

## Implementation Roadmap

### Phase 1: IB Client Portal Adapter

1. Implement OAuth 1.0a authentication
2. Create REST API client wrapper
3. Implement position/order/market data methods
4. Add session token refresh
5. Test with Client Portal Gateway

### Phase 2: Alpaca Adapter

1. Implement API key authentication
2. Create REST API client with rate limiting
3. Implement position/order/market data methods
4. Add exponential backoff retry logic
5. Test with paper trading account

### Phase 3: Broker Manager & Selection

1. Implement BrokerFactory
2. Implement BrokerManager
3. Add performance-based selection (algo wheel)
4. Implement fallback mechanisms
5. Add health checks

### Phase 4: Data Normalization

1. Create DataNormalizer class
2. Implement field mapping for each broker
3. Add currency conversion
4. Test data consistency across brokers

## Testing Strategy

1. **Unit Tests:** Test each adapter independently
2. **Integration Tests:** Test with real broker APIs (paper trading)
3. **Fallback Tests:** Test failover scenarios
4. **Performance Tests:** Test rate limiting and retry logic
5. **Data Consistency Tests:** Verify normalized data matches across brokers

## Security Considerations

1. **API Keys:** Store in environment variables, never in code
2. **OAuth Tokens:** Encrypt stored tokens, refresh automatically
3. **Rate Limiting:** Respect broker rate limits to avoid account restrictions
4. **Error Logging:** Don't log sensitive data (API keys, tokens)
5. **Connection Security:** Use HTTPS for all REST API calls

## Next Steps

1. Review and approve this design
2. Begin Phase 1 implementation (IB Client Portal Adapter)
3. Create implementation tasks for each phase
4. Set up paper trading accounts for testing
