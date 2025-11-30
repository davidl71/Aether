# OpenAlgo Integration Patterns

**Purpose**: Document patterns and features from [OpenAlgo](https://github.com/marketcalls/openalgo) that enhance this project.

**Reference**: [OpenAlgo GitHub](https://github.com/marketcalls/openalgo) - Open Source Algo Trading Platform

---

## Overview

OpenAlgo is a Python Flask-based open-source algo trading platform with comprehensive features for broker integration, security, rate limiting, and API management. This document captures relevant patterns for integration into this C++/Python trading application.

---

## 1. MCP (Model Context Protocol) Integration

### OpenAlgo Approach

OpenAlgo includes native MCP server capabilities, enabling AI assistants to execute trades and manage portfolios directly through natural language commands.

**Key Features**:

- Full trading capabilities (place, modify, cancel orders)
- Smart orders (automatic position sizing, basket orders)
- Position management (close positions, track P&L)
- Market data (real-time quotes, market depth, historical data)
- Account information (funds, holdings, order book)

**Available MCP Tools**:

- `place_order`, `place_smart_order`, `place_basket_order`
- `modify_order`, `cancel_order`
- `get_open_position`, `get_position_book`, `close_all_positions`
- `get_quote`, `get_market_depth`, `get_historical_data`
- `get_funds`, `get_holdings`, `get_order_book`, `get_trade_book`
- `search_instruments`, `get_symbol_info`, `get_expiry_dates`

### Implementation in This Project

**Location**: `mcp/trading_server/server.py`

**Features Implemented**:

- ✅ Order placement (`place_order`, `place_box_spread`)
- ✅ Order cancellation (`cancel_order`)
- ✅ Position tracking (`get_open_positions`)
- ✅ Market data (`get_quote`)
- ✅ Account information (`get_funds`)
- ✅ Rate limiting per endpoint
- ✅ API key authentication
- ✅ Dry-run mode support

**Integration Points**:

- Native C++ `OrderManager` for order execution
- TWS API for market data and account info
- Backend REST API for position tracking

**Configuration**:
Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "trading": {
      "command": "python",
      "args": ["-m", "mcp.trading_server.server"],
      "env": {
        "TRADING_API_KEY": "your-api-key",
        "TWS_HOST": "127.0.0.1",
        "TWS_PORT": "7497",
        "DRY_RUN": "true"
      }
    }
  }
}
```

---

## 2. Rate Limiting Patterns

### OpenAlgo Approach

OpenAlgo implements comprehensive rate limiting with:

- **Configurable Rate Limits**: Environment variable controlled
- **Moving Window Strategy**: Flask-Limiter with moving-window for accurate limiting
- **IP-based Limiting**: Rate limits applied per IP address
- **Per-Endpoint Limits**: Different limits for different operations
- **Graceful Handling**: Clear error messages when limits exceeded

**Rate Limit Categories**:

- `LOGIN_RATE_LIMIT_MIN`: Login attempts per minute (default: 5)
- `LOGIN_RATE_LIMIT_HOUR`: Login attempts per hour (default: 25)
- `API_RATE_LIMIT`: General API endpoints (default: 10 per second)
- `ORDER_RATE_LIMIT`: Order operations (default: 10 per second)
- `SMART_ORDER_RATE_LIMIT`: Smart order operations
- `WEBHOOK_RATE_LIMIT`: Webhook endpoint limits
- `STRATEGY_RATE_LIMIT`: Strategy operation limits

### Current Implementation

**Location**: `native/src/rate_limiter.cpp`, `native/include/rate_limiter.h`

**Current Features**:

- ✅ Message rate limiting (messages per second)
- ✅ Historical request limiting (concurrent requests)
- ✅ Market data line limiting (concurrent subscriptions)
- ✅ Configurable limits
- ✅ Status reporting

**Enhancements Needed** (inspired by OpenAlgo):

1. **IP-based Rate Limiting**:

   ```cpp
   class IPRateLimiter {
     std::unordered_map<std::string, RateLimiter> per_ip_limiters_;
     // Track rate limits per IP address
   };
   ```

2. **Per-Endpoint Rate Limits**:

   ```cpp
   struct EndpointRateLimit {
     std::string endpoint;
     int max_requests_per_second;
     int max_requests_per_minute;
   };
   ```

3. **Moving Window Implementation**:
   - Current implementation uses fixed 1-second windows
   - OpenAlgo uses moving windows for more accurate limiting
   - Consider implementing sliding window algorithm

4. **Rate Limit Headers**:
   - Return rate limit information in responses
   - Include `X-RateLimit-Remaining`, `X-RateLimit-Reset` headers

**Recommended Enhancements**:

```cpp
// Enhanced rate limiter with IP-based and per-endpoint limits
class EnhancedRateLimiter {
public:
  struct EndpointConfig {
    std::string endpoint;
    int max_per_second;
    int max_per_minute;
  };

  bool check_rate_limit(const std::string& ip,
                         const std::string& endpoint);
  RateLimitInfo get_rate_limit_info(const std::string& ip,
                                    const std::string& endpoint);
};
```

---

## 3. Security Features

### OpenAlgo Approach

OpenAlgo implements comprehensive security at multiple levels:

#### Browser-Level Security

- **Content Security Policy (CSP)**: Configurable CSP headers to prevent XSS attacks
- **CORS Protection**: Controlled Cross-Origin Resource Sharing
- **CSRF Protection**: Built-in CSRF token validation
- **Secure Headers**: X-Frame-Options, X-Content-Type-Options, etc.
- **Session Security**: Secure session management with proper cookie settings

#### Database-Level Security

- **Password Hashing**: Argon2 (winner of Password Hashing Competition)
- **Token Encryption**: Fernet symmetric encryption with PBKDF2 key derivation
- **API Key Security**:
  - Hashed storage using Argon2 with pepper
  - Encrypted storage for retrieval
  - Time-based caching with TTL

- **SQL Injection Protection**: SQLAlchemy ORM with parameterized queries
- **Connection Pooling**: Optimized database connections

### Current Implementation

**Current Security**:

- ✅ Credentials not committed to git (`.gitignore`, `.cursorignore`)
- ✅ Environment variable configuration
- ✅ Dry-run mode for testing
- ⚠️ No credential encryption
- ⚠️ No API key management system
- ⚠️ No token encryption

**Recommended Enhancements**:

1. **Credential Encryption**:

   ```cpp
   class CredentialManager {
     std::string encrypt_token(const std::string& token);
     std::string decrypt_token(const std::string& encrypted);
     // Use Fernet or similar encryption
   };
   ```

2. **API Key Management**:

   ```cpp
   class APIKeyManager {
     bool validate_api_key(const std::string& key);
     std::string hash_api_key(const std::string& key);
     // Store hashed keys, validate on use
   };
   ```

3. **Password Hashing** (for future user authentication):
   - Use Argon2 for password hashing
   - Consider integrating with existing C++ libraries or Python bindings

4. **Secure Configuration Storage**:
   - Encrypt sensitive config values
   - Use secure key derivation (PBKDF2)
   - Store encryption keys in secure keychain (macOS Keychain, Linux keyring)

**Implementation Priority**:

1. **High**: API key management for MCP server
2. **Medium**: Credential encryption for stored tokens
3. **Low**: Full password hashing system (if user auth added)

---

## 4. API Analyzer Pattern

### OpenAlgo Approach

OpenAlgo includes a comprehensive API Analyzer tool that provides:

**For Traders**:

- Risk-free testing of all trading operations
- Real-time validation of order parameters
- Strategy monitoring
- Visual feedback with notifications
- Cost savings by avoiding trading errors

**For Developers**:

- Complete API testing with detailed request/response analysis
- Automatic parameter validation
- WebSocket monitoring
- Performance metrics
- Debug tools

### Application to This Project

**Breadcrumb Logging as API Analyzer**:

The breadcrumb logging system can serve a similar purpose:

1. **Trading Operation Tracking**:
   - Log all order placements with full parameters
   - Track order lifecycle (submitted → filled → cancelled)
   - Record state changes during order execution

2. **Validation and Testing**:
   - Capture order parameters before submission
   - Log validation failures with context
   - Track rate limit violations

3. **Debugging and Analysis**:
   - Complete trail of trading operations
   - State dumps at critical points
   - Error context for failed operations

**Enhanced Breadcrumb Logging for Trading**:

```cpp
// In order_manager.cpp
void OrderManager::place_order(...) {
  // Log order attempt
  TUI_BREADCRUMB_ACTION("place_order", "order_manager",
                        "symbol=" + contract.symbol +
                        " side=" + action_to_string(action) +
                        " qty=" + std::to_string(quantity));

  // Log validation
  if (!validate_order(...)) {
    TUI_BREADCRUMB_ERROR("order_manager", "validation_failed",
                          "error=" + error_msg);
    return;
  }

  // Log state before execution
  std::string state = DumpOrderState();
  TUI_BREADCRUMB_STATE_CHANGE("order_manager", "before_execution", state);

  // Execute order...

  // Log result
  TUI_BREADCRUMB_ACTION("order_placed", "order_manager",
                        "order_id=" + std::to_string(order_id));
}
```

**Benefits**:

- Complete audit trail of all trading operations
- Easy reproduction of issues
- Performance analysis (timing between operations)
- Compliance and regulatory requirements

---

## 5. Rate Limiting Implementation Details

### Moving Window Algorithm

OpenAlgo uses Flask-Limiter's moving window strategy. For C++ implementation:

```cpp
class MovingWindowRateLimiter {
private:
  struct RequestWindow {
    std::deque<std::chrono::steady_clock::time_point> requests;
    std::chrono::seconds window_size;
    int max_requests;
  };

  std::unordered_map<std::string, RequestWindow> windows_;

public:
  bool check_rate_limit(const std::string& key,
                       int max_requests,
                       std::chrono::seconds window) {
    auto now = std::chrono::steady_clock::now();
    auto& window = windows_[key];

    // Remove requests outside window
    while (!window.requests.empty() &&
           (now - window.requests.front()) > window.window_size) {
      window.requests.pop_front();
    }

    if (window.requests.size() >= max_requests) {
      return false;  // Rate limited
    }

    window.requests.push_back(now);
    return true;  // Allowed
  }
};
```

### IP-based Rate Limiting

```cpp
class IPRateLimiter {
private:
  std::unordered_map<std::string, MovingWindowRateLimiter> ip_limiters_;
  RateLimiterConfig default_config_;

public:
  bool check_ip_rate_limit(const std::string& ip,
                          const std::string& endpoint) {
    std::string key = ip + ":" + endpoint;

    if (ip_limiters_.find(key) == ip_limiters_.end()) {
      ip_limiters_[key] = MovingWindowRateLimiter(default_config_);
    }

    return ip_limiters_[key].check_rate_limit(
      key,
      get_limit_for_endpoint(endpoint),
      std::chrono::seconds(1)
    );
  }
};
```

---

## 6. Security Implementation Details

### Token Encryption (Fernet-style)

```cpp
class TokenEncryption {
private:
  std::vector<uint8_t> key_;  // 32-byte key

  std::vector<uint8_t> derive_key(const std::string& password,
                                    const std::vector<uint8_t>& salt) {
    // PBKDF2 key derivation
    // Return 32-byte key
  }

public:
  std::string encrypt_token(const std::string& token) {
    // Generate IV
    // Encrypt with AES-256
    // Return base64 encoded: IV + ciphertext
  }

  std::string decrypt_token(const std::string& encrypted) {
    // Decode base64
    // Extract IV
    // Decrypt
    // Return plaintext
  }
};
```

### API Key Hashing

```cpp
class APIKeyManager {
private:
  std::string pepper_;  // Application-wide secret

  std::string hash_key(const std::string& key) {
    // Argon2 hash with pepper
    // Return hashed value
  }

public:
  bool validate_api_key(const std::string& provided_key,
                       const std::string& stored_hash) {
    std::string hash = hash_key(provided_key);
    return hash == stored_hash;
  }
};
```

---

## 7. Integration Checklist

### MCP Trading Server

- [x] Create MCP server structure
- [x] Implement basic order operations
- [x] Add rate limiting
- [x] Add API key authentication
- [ ] Integrate with native C++ OrderManager
- [ ] Add position management tools
- [ ] Add market data tools
- [ ] Add account information tools
- [ ] Add comprehensive error handling
- [ ] Add request/response logging

### Rate Limiting Enhancements

- [ ] Add IP-based rate limiting
- [ ] Add per-endpoint rate limits
- [ ] Implement moving window algorithm
- [ ] Add rate limit headers to responses
- [ ] Add rate limit status endpoints
- [ ] Add configuration via environment variables

### Security Enhancements

- [ ] Implement credential encryption
- [ ] Add API key management system
- [ ] Add secure configuration storage
- [ ] Add token encryption/decryption
- [ ] Add password hashing (if user auth needed)
- [ ] Add secure keychain integration

### Breadcrumb Logging for Trading

- [ ] Add breadcrumb logging to OrderManager
- [ ] Add breadcrumb logging to TWS client
- [ ] Add breadcrumb logging to strategy execution
- [ ] Add state dumps for trading operations
- [ ] Add performance timing breadcrumbs
- [ ] Document breadcrumb usage for trading testing

---

## 8. References

- [OpenAlgo GitHub](https://github.com/marketcalls/openalgo)
- [OpenAlgo Documentation](https://docs.openalgo.in)
- [OpenAlgo MCP Integration](https://github.com/marketcalls/openalgo/tree/main/mcp)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Flask-Limiter Documentation](https://flask-limiter.readthedocs.io/)
- [Argon2 Password Hashing](https://github.com/P-H-C/phc-winner-argon2)

---

## 9. Next Steps

1. **Immediate**: Complete MCP server integration with native C++ code
2. **Short-term**: Enhance rate limiting with IP-based and per-endpoint limits
3. **Medium-term**: Implement credential encryption and API key management
4. **Long-term**: Full security audit and compliance features

---

**Last Updated**: 2025-01-27
