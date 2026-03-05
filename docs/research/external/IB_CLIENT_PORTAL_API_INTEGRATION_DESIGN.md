# IB Client Portal API Integration Architecture Design

**Date**: 2025-11-18
**Status**: Design Document
**Purpose**: Design architecture for integrating IB Client Portal REST API into the unified multi-broker system

---

## Overview

This document designs the IB Client Portal API integration architecture following the unified broker interface pattern established in `docs/MULTI_BROKER_ARCHITECTURE_DESIGN.md`.

---

## Research Summary

### IB Client Portal API Capabilities

**REST API:**

- Trading operations (orders, positions, account)
- Market data (quotes, trades, bars)
- Options trading support
- Multi-leg orders (spreads, combos)
- No TWS/Gateway required

**Authentication:**

- OAuth 2.0 flow (redirect → callback → tokens)
- Session-based authentication (login → session token)
- Alternative: Username/password (less secure)

**Base URL:**

- `https://localhost:5001/v1/api` (local Client Portal)
- Or cloud-hosted Client Portal instance

**Key Differences from TWS API:**

- REST-based (request/response) vs Socket-based (callbacks)
- No TWS/Gateway needed
- Session tokens vs persistent connection
- JSON responses vs binary protocol

---

## Architecture Design

### Unified Broker Interface

The IB Client Portal adapter will implement the `IBroker` interface defined in `docs/MULTI_BROKER_ARCHITECTURE_DESIGN.md`:

```cpp
class IBClientPortalAdapter : public IBroker {
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

**OAuth 2.0 Flow:**

```cpp
class IBClientPortalAuth {
public:
    // Step 1: Initiate OAuth flow
    std::string get_authorization_url() {
        // Redirect user to: https://localhost:5001/v1/api/oauth/authorize
        return "https://localhost:5001/v1/api/oauth/authorize?response_type=code&client_id=" + client_id_;
    }

    // Step 2: Exchange code for tokens (after user authorizes)
    bool exchange_code_for_tokens(const std::string& code) {
        // POST /v1/api/oauth/token
        // Returns: access_token, refresh_token, expires_in
        // Store tokens securely
    }

    // Step 3: Use access token for API requests
    void set_auth_headers(HttpHeaders& headers) {
        headers["Authorization"] = "Bearer " + access_token_;
    }
};
```

**Session-Based Authentication (Alternative):**

```cpp
bool IBClientPortalAdapter::connect() {
    // POST /iserver/auth/status
    // If not authenticated, POST /iserver/auth/login
    // Store session token
    // Use session token in subsequent requests
}
```

**Configuration:**

```cpp
struct IBClientPortalConfig {
    std::string base_url;  // https://localhost:5001/v1/api
    std::string client_id;  // OAuth client ID
    std::string client_secret;  // OAuth client secret
    std::string username;  // Alternative: username/password
    std::string password;  // Alternative: username/password
    bool use_oauth;  // true for OAuth, false for username/password
};
```

### 2. Market Data

**REST API Endpoints:**

- `GET /iserver/marketdata/snapshot?conids={conid}&fields={fields}` - Snapshot data
- `GET /iserver/marketdata/history?conid={conid}&period={period}` - Historical data
- `POST /iserver/marketdata/unsubscribeall` - Unsubscribe all

**Options Chain:**

- `GET /iserver/secdef/search?symbol={symbol}&sectype=OPT` - Search options
- `GET /iserver/secdef/info?conid={conid}` - Contract details

**Implementation:**

```cpp
int IBClientPortalAdapter::subscribe_market_data(const types::OptionContract& contract) {
    // Step 1: Get contract ID (conid)
    long conid = get_contract_id(contract);

    // Step 2: Subscribe to market data
    // GET /iserver/marketdata/snapshot?conids={conid}&fields=31,84,86
    // Fields: 31=bid, 84=ask, 86=last

    // Step 3: Poll for updates (REST API doesn't support push)
    // Or use WebSocket if available

    // Store subscription
    int subscription_id = next_subscription_id_++;
    subscriptions_[subscription_id] = conid;

    return subscription_id;
}
```

**Polling Strategy:**

```cpp
void IBClientPortalAdapter::poll_market_data() {
    // Poll every 1-2 seconds for subscribed contracts
    for (const auto& [sub_id, conid] : subscriptions_) {
        // GET /iserver/marketdata/snapshot?conids={conid}&fields=31,84,86
        auto snapshot = http_client_->get("/iserver/marketdata/snapshot", {
            {"conids", std::to_string(conid)},
            {"fields", "31,84,86"}
        });

        // Convert to types::MarketData
        auto market_data = convert_snapshot_to_market_data(snapshot);

        // Emit to subscribers
        emit_market_data(sub_id, market_data);
    }
}
```

### 3. Order Placement

**Single Order:**

```cpp
int IBClientPortalAdapter::place_order(const types::Order& order) {
    // Convert to IB Client Portal order format
    json ib_order = {
        {"conid", get_contract_id(order.contract)},
        {"orderType", "LMT"},
        {"side", order.action == OrderAction::Buy ? "BUY" : "SELL"},
        {"quantity", order.quantity},
        {"price", order.limit_price},
        {"tif", "DAY"}
    };

    // POST /iserver/account/{accountId}/orders
    auto response = http_client_->post(
        "/iserver/account/" + account_id_ + "/orders",
        ib_order
    );

    // Parse response and return order ID
    return response["id"];
}
```

**Multi-Leg Order (Combo):**

```cpp
int IBClientPortalAdapter::place_combo_order(const std::vector<types::Order>& legs) {
    // IB Client Portal supports combo orders via "legs" array
    json ib_order = {
        {"conid", get_combo_conid(legs)},  // Combo contract ID
        {"orderType", "LMT"},
        {"side", "BUY"},  // Net direction
        {"quantity", 1},
        {"price", calculate_combo_price(legs)},
        {"legs", json::array()}
    };

    for (const auto& leg : legs) {
        json leg_order = {
            {"conid", get_contract_id(leg.contract)},
            {"ratio", leg.quantity},
            {"action", leg.action == OrderAction::Buy ? "BUY" : "SELL"}
        };
        ib_order["legs"].push_back(leg_order);
    }

    // POST /iserver/account/{accountId}/orders
    auto response = http_client_->post(
        "/iserver/account/" + account_id_ + "/orders",
        ib_order
    );

    return response["id"];
}
```

### 4. Contract ID Lookup

**Search for Contract:**

```cpp
long IBClientPortalAdapter::get_contract_id(const types::OptionContract& contract) {
    // Search for contract
    // GET /iserver/secdef/search?symbol={symbol}&sectype=OPT&strike={strike}&right={C|P}&expiry={YYYYMMDD}

    auto response = http_client_->get("/iserver/secdef/search", {
        {"symbol", contract.symbol},
        {"sectype", "OPT"},
        {"strike", std::to_string(contract.strike)},
        {"right", contract.right == OptionType::Call ? "C" : "P"},
        {"expiry", contract.expiration}
    });

    // Parse response and return conid
    if (!response["conid"].empty()) {
        return response["conid"][0];  // Use first match
    }

    return -1;  // Not found
}
```

**Cache Contract IDs:**

```cpp
class ContractCache {
    std::unordered_map<std::string, long> cache_;  // contract_key -> conid

public:
    long get_or_lookup(const types::OptionContract& contract) {
        std::string key = contract.to_string();

        if (cache_.count(key)) {
            return cache_[key];
        }

        // Lookup via API
        long conid = lookup_contract_id(contract);
        if (conid > 0) {
            cache_[key] = conid;
        }

        return conid;
    }
};
```

### 5. Session Management

**Session Token Management:**

```cpp
class SessionManager {
    std::string session_token_;
    std::chrono::system_clock::time_point token_expiry_;
    std::mutex mutex_;

public:
    bool is_session_valid() {
        std::lock_guard<std::mutex> lock(mutex_);
        return !session_token_.empty() &&
               std::chrono::system_clock::now() < token_expiry_;
    }

    void refresh_session() {
        // POST /iserver/auth/status
        // If expired, POST /iserver/auth/login
        // Update session_token_ and token_expiry_
    }

    std::string get_session_token() {
        if (!is_session_valid()) {
            refresh_session();
        }
        return session_token_;
    }
};
```

---

## Data Flow

### Market Data Flow

```
IB Client Portal REST API → IBClientPortalAdapter::poll_market_data()
    → Convert to types::MarketData
    → Emit to subscribers
    → Strategy receives normalized data
```

### Order Flow

```
Strategy → IBClientPortalAdapter::place_order()
    → Convert to IB format
    → POST /iserver/account/{accountId}/orders
    → Parse response
    → Return order ID
    → Track order status
```

---

## Error Handling

### Common Errors

| Error Code | Description | Handling |
|------------|-------------|----------|
| 401 | Unauthorized | Re-authenticate, refresh session |
| 403 | Forbidden | Check account permissions |
| 429 | Rate limit exceeded | Wait and retry with backoff |
| 500 | Server error | Retry with exponential backoff |
| 200 | Success | Process response |

### Implementation

```cpp
void IBClientPortalAdapter::handle_error(int status_code, const json& error) {
    switch (status_code) {
        case 401:
            spdlog::error("IB Client Portal authentication failed");
            // Refresh session or re-authenticate
            session_manager_.refresh_session();
            break;
        case 403:
            spdlog::error("IB Client Portal forbidden: {}", error["message"]);
            break;
        case 429:
            spdlog::warn("IB Client Portal rate limit exceeded, backing off");
            rate_limiter_.backoff();
            break;
        default:
            spdlog::error("IB Client Portal API error {}: {}", status_code, error["message"]);
    }
}
```

---

## Comparison with TWS API

| Feature | TWS API | Client Portal API |
|---------|---------|-------------------|
| **Connection** | Socket-based | REST-based |
| **TWS Required** | Yes | No |
| **Pattern** | Callbacks | Request/Response |
| **Market Data** | Push (callbacks) | Pull (polling) |
| **Orders** | Callback-based | REST API |
| **Session** | Persistent connection | Session tokens |
| **Complexity** | Higher (callbacks) | Lower (REST) |

**Recommendation**: Use Client Portal API for simpler integration, TWS API for lower latency and push-based market data.

---

## Testing Strategy

### Unit Tests

- Contract ID lookup
- Order format conversion
- Session management
- Error handling

### Integration Tests

- Local Client Portal connection
- Market data polling
- Order placement (paper trading)
- Position tracking
- Error scenarios

### Test Scenarios

1. **Connection Test**
   - Connect to local Client Portal
   - Verify OAuth flow
   - Check session token

2. **Market Data Test**
   - Subscribe to SPY options
   - Poll for quotes
   - Test unsubscribe

3. **Order Test**
   - Place single order (paper)
   - Place multi-leg order (paper)
   - Cancel order
   - Verify order status

4. **Error Test**
   - Invalid credentials
   - Session expiration
   - Rate limit exceeded
   - Network timeout

---

## Configuration

### Configuration File

```json
{
  "brokers": {
    "ib_client_portal": {
      "enabled": true,
      "base_url": "https://localhost:5001/v1/api",
      "use_oauth": true,
      "client_id": "YOUR_CLIENT_ID",
      "client_secret": "YOUR_CLIENT_SECRET",
      "account_id": "YOUR_ACCOUNT_ID",
      "poll_interval_ms": 2000
    }
  }
}
```

---

## Dependencies

### Required Libraries

- **HTTP Client**: `libcurl` or `cpp-httplib` for REST API
- **JSON**: `nlohmann/json` for JSON parsing
- **OAuth**: Custom OAuth 2.0 implementation or `oauth2-cpp`
- **Threading**: Standard library `std::thread`, `std::mutex`

### Optional Libraries

- **Rate Limiting**: Custom implementation or `ratelimit` library
- **Retry Logic**: Custom implementation or `cpp-retry` library

---

## Implementation Phases

### Phase 1: Authentication & Connection (Week 1)

- OAuth 2.0 flow implementation
- Session management
- Account info retrieval

### Phase 2: Market Data (Week 2)

- Contract ID lookup
- Market data polling
- Data normalization

### Phase 3: Order Placement (Week 3)

- Single order placement
- Multi-leg orders
- Order status tracking

### Phase 4: Integration (Week 4)

- Unified broker interface implementation
- Integration with order manager
- Integration with strategy
- Error handling and retry logic

---

## Success Criteria

- ✅ IB Client Portal adapter implements IBroker interface
- ✅ OAuth authentication works
- ✅ Market data polling works
- ✅ Order placement works (paper trading)
- ✅ Multi-leg orders work
- ✅ Error handling comprehensive
- ✅ Session management robust
- ✅ Integration tests pass
- ✅ Paper trading validation successful

---

## References

- [IB Client Portal API Documentation](https://www.interactivebrokers.com/api/doc.html)
- [IB Client Portal Authentication](https://www.interactivebrokers.com/api/doc.html#tag/Authentication)
- [IB Client Portal Trading API](https://www.interactivebrokers.com/api/doc.html#tag/Orders)
- [Multi-Broker Architecture Design](../../research/architecture/MULTI_BROKER_ARCHITECTURE_DESIGN.md)
- [Unified Broker Interface](../../research/architecture/MULTI_BROKER_ARCHITECTURE_DESIGN.md)

---

**Status**: ✅ Design Complete - Ready for Implementation
**Next Step**: Implement IB Client Portal adapter (T-36)
