# Alpaca Integration Plan

**Date**: 2025-01-16
**Status**: Planning
**Critical Issue**: Alpaca does not support options trading

---

## Problem Statement

You're experiencing TWS connection issues and want to integrate with Alpaca instead. However, there's a fundamental incompatibility:

- **Your codebase**: Designed for box spread arbitrage (requires **options**)
- **Alpaca API**: Only supports **stocks** and **crypto** (no options)

---

## Options Analysis

### Option 1: Fix TWS Connection (RECOMMENDED)
**Pros**:
- Keep box spread strategy intact
- Most professional options platform
- Comprehensive TWS API
- Your code already 90% complete

**Cons**:
- TWS setup complexity
- Connection maintenance

**Effort**: 30 minutes to fix connection

**Action Items**:
1. Install IB Gateway
2. Configure API settings (port 7497, enable socket clients)
3. Accept connection in TWS popup
4. Test with your existing code

---

### Option 2: Switch to Tradier (Options Alternative)
**Pros**:
- REST API (easier than TWS)
- Supports options trading
- Lower account minimums
- Good for options strategies

**Cons**:
- Need new account
- Different fee structure
- API differences from IBKR

**Effort**: 2-3 days to integrate

**Tradier API**: https://documentation.tradier.com/

---

### Option 3: Use Alpaca for Different Strategy
**Pros**:
- Simple REST API
- Free data API
- Good for stock/crypto strategies

**Cons**:
- **Cannot do box spreads** (no options)
- Need to completely redesign strategy
- Months of work to pivot

**Use Cases**:
- Stock arbitrage
- Pairs trading
- Momentum strategies
- NOT options strategies

**Effort**: 2-3 weeks to redesign strategy + 1 week integration

---

### Option 4: Hybrid Approach
**Pros**:
- Use IBKR for options (box spreads)
- Use Alpaca for stock strategies
- Diversified broker risk

**Cons**:
- Maintain two integrations
- More complexity

**Effort**: 1 week to add Alpaca alongside IBKR

---

## Recommended Path: Fix TWS Connection

Since your entire codebase is built for box spreads, fixing TWS is the fastest path to production.

### TWS Connection Debug Checklist

1. **Install IB Gateway**
   ```bash
   # macOS
   brew install --cask ib-gateway

   # Or download from:
   # https://www.interactivebrokers.com/en/trading/ibgateway-stable.php
   ```

2. **Configure IB Gateway**
   - Launch IB Gateway
   - Login with paper trading credentials
   - Go to: Configure → Settings → API → Settings
   - Enable:
     - ✅ Enable ActiveX and Socket Clients
     - ✅ Read-Only API (for safety)
     - Port: `7497` (paper trading)
     - Master API client ID: `0`
     - Trusted IPs: `127.0.0.1`

3. **Test Connection**
   ```bash
   # Check if port is listening
   lsof -iTCP:7497 -sTCP:LISTEN

   # Should show:
   # java    <PID> username   XX TCP localhost:7497 (LISTEN)
   ```

4. **Common Issues**

   **Issue**: "Couldn't connect to TWS"
   - **Fix**: Make sure IB Gateway is running
   - **Fix**: Check port 7497 is correct (7497=paper, 7496=live)

   **Issue**: "Connection timeout"
   - **Fix**: Accept connection in TWS popup dialog
   - **Fix**: Disable "Allow connections from localhost only"

   **Issue**: "Port already in use"
   - **Fix**: Change `client_id` in config.json (try 1, 2, 3, etc.)
   - **Fix**: Close other TWS connections

   **Issue**: TWS asks to accept connection every time
   - **Fix**: In TWS settings, uncheck "Prompt for client ID"

5. **Verify Connection in Code**
   ```bash
   # Run with debug logging
   ./ib_box_spread --config config/config.example.json --log-level debug --dry-run

   # Look for these log lines:
   # [info] Connecting to TWS at 127.0.0.1:7497 (client_id=1)
   # [info] Connected to TWS, waiting for nextValidId...
   # [info] Received nextValidId: 1 - Connection fully established
   ```

---

## If You Still Want Alpaca Integration

### Architecture: Broker Abstraction Layer

To support multiple brokers, we need to create an abstraction:

```
┌─────────────────────────────────────┐
│     Box Spread Strategy             │
│  (business logic - broker agnostic) │
└─────────────┬───────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│     BrokerAdapter Interface         │
│  • get_option_chain()                │
│  • place_order()                     │
│  • get_positions()                   │
│  • subscribe_market_data()           │
└─────────────┬───────────────────────┘
              │
       ┌──────┴──────┐
       ▼             ▼
┌─────────────┐ ┌──────────────┐
│ IBKRAdapter │ │ AlpacaAdapter│
│   (Options) │ │   (Stocks)   │
└─────────────┘ └──────────────┘
```

### Implementation Plan

#### Step 1: Create Broker Abstraction (2-3 hours)

**File**: `native/include/broker_adapter.h`

```cpp
class BrokerAdapter {
public:
    virtual ~BrokerAdapter() = default;

    // Connection
    virtual bool connect() = 0;
    virtual void disconnect() = 0;
    virtual bool is_connected() const = 0;

    // Market data
    virtual bool get_option_chain(
        const std::string& symbol,
        const std::string& expiry,
        option_chain::OptionChain& chain) = 0;

    virtual int subscribe_market_data(
        const types::OptionContract& contract) = 0;

    // Orders
    virtual int place_order(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        double limit_price = 0,
        types::TimeInForce tif = types::TimeInForce::Day) = 0;

    virtual bool cancel_order(int order_id) = 0;

    // Positions & account
    virtual std::vector<types::Position> get_positions() const = 0;
    virtual std::optional<types::Order> get_order(int order_id) const = 0;

    // Capabilities
    virtual bool supports_options() const = 0;
    virtual bool supports_combo_orders() const = 0;
};
```

#### Step 2: IBKR Adapter (1-2 hours - refactor existing)

**File**: `native/include/ibkr_adapter.h`

```cpp
class IBKRAdapter : public BrokerAdapter {
public:
    IBKRAdapter(const config::TWSConfig& config);

    bool connect() override;
    void disconnect() override;
    bool is_connected() const override;

    // Implement all interface methods
    bool get_option_chain(...) override;
    int place_order(...) override;
    // ... etc

    bool supports_options() const override { return true; }
    bool supports_combo_orders() const override { return true; }

private:
    std::unique_ptr<tws::TWSClient> client_;
};
```

#### Step 3: Alpaca Adapter (4-6 hours - new implementation)

**File**: `native/include/alpaca_adapter.h`

```cpp
class AlpacaAdapter : public BrokerAdapter {
public:
    AlpacaAdapter(const std::string& api_key, const std::string& secret_key, bool paper_trading);

    bool connect() override;
    void disconnect() override;
    bool is_connected() const override;

    // Market data (stocks only)
    int subscribe_market_data(const types::StockContract& contract);

    // Orders (stocks only)
    int place_stock_order(
        const std::string& symbol,
        types::OrderAction action,
        int quantity,
        double limit_price = 0);

    bool supports_options() const override { return false; }  // NO OPTIONS
    bool supports_combo_orders() const override { return false; }

private:
    std::string api_key_;
    std::string secret_key_;
    bool paper_trading_;
    // HTTP client for REST API
};
```

#### Step 4: Alpaca REST Client (6-8 hours)

**Dependencies**:
```bash
# Add to CMakeLists.txt
find_package(CURL REQUIRED)
find_package(nlohmann_json REQUIRED)
```

**Example Alpaca API Call**:
```cpp
// Get account info
std::string AlpacaAdapter::get_account_info() {
    CURL* curl = curl_easy_init();

    std::string url = paper_trading_
        ? "https://paper-api.alpaca.markets/v2/account"
        : "https://api.alpaca.markets/v2/account";

    struct curl_slist* headers = nullptr;
    headers = curl_slist_append(headers, ("APCA-API-KEY-ID: " + api_key_).c_str());
    headers = curl_slist_append(headers, ("APCA-API-SECRET-KEY: " + secret_key_).c_str());

    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);

    // ... perform request, parse JSON response
}

// Place stock order
std::string AlpacaAdapter::place_stock_order(
    const std::string& symbol,
    const std::string& side,  // "buy" or "sell"
    int qty,
    const std::string& type,  // "market" or "limit"
    double limit_price) {

    nlohmann::json order_json = {
        {"symbol", symbol},
        {"qty", qty},
        {"side", side},
        {"type", type},
        {"time_in_force", "day"}
    };

    if (type == "limit") {
        order_json["limit_price"] = std::to_string(limit_price);
    }

    // POST to /v2/orders
    std::string url = base_url_ + "/v2/orders";
    // ... HTTP POST with JSON body
}
```

#### Step 5: Update Strategy to Use Adapter (1-2 hours)

```cpp
// Before:
BoxSpreadStrategy strategy(&tws_client, &order_mgr, params);

// After:
std::unique_ptr<BrokerAdapter> broker;

if (config.broker == "ibkr") {
    broker = std::make_unique<IBKRAdapter>(config.tws);
} else if (config.broker == "alpaca") {
    broker = std::make_unique<AlpacaAdapter>(
        config.alpaca.api_key,
        config.alpaca.secret_key,
        config.alpaca.paper_trading
    );
}

// Strategy uses broker interface
BoxSpreadStrategy strategy(broker.get(), &order_mgr, params);
```

---

## Alpaca Integration Checklist

### Prerequisites
- [ ] Alpaca account created (https://alpaca.markets)
- [ ] API keys generated (paper trading first!)
- [ ] Understand Alpaca limitations (no options)
- [ ] Decide on use case (stock strategies only)

### Implementation Tasks
- [ ] Create BrokerAdapter interface
- [ ] Refactor TWSClient into IBKRAdapter
- [ ] Implement AlpacaAdapter
- [ ] Add Alpaca REST client (CURL + JSON)
- [ ] Add Alpaca to config.json
- [ ] Write integration tests
- [ ] Test with Alpaca paper trading

### Configuration Example

**File**: `config/config.json`

```json
{
  "broker": "alpaca",  // or "ibkr"

  "alpaca": {
    "api_key": "${ALPACA_API_KEY}",
    "secret_key": "${ALPACA_SECRET_KEY}",
    "paper_trading": true,  // Use paper trading first!
    "base_url": "https://paper-api.alpaca.markets"
  },

  "ibkr": {
    "host": "127.0.0.1",
    "port": 7497,
    "client_id": 1
  }
}
```

---

## Alpaca API Reference

### Key Endpoints

**Account**:
- GET `/v2/account` - Get account info
- GET `/v2/positions` - Get positions

**Market Data**:
- GET `/v2/stocks/{symbol}/quotes/latest` - Latest quote
- GET `/v2/stocks/{symbol}/bars` - Historical bars
- WS `wss://stream.data.alpaca.markets/v2/iex` - Real-time streaming

**Orders**:
- POST `/v2/orders` - Place order
- GET `/v2/orders` - List orders
- DELETE `/v2/orders/{order_id}` - Cancel order

**Documentation**: https://alpaca.markets/docs/api-references/trading-api/

---

## Cost Comparison

| Feature | IBKR | Alpaca |
|---------|------|--------|
| Options | ✅ Yes | ❌ No |
| Stocks | ✅ Yes | ✅ Yes |
| Crypto | ❌ No | ✅ Yes |
| Commission (stocks) | $0.0035/share | $0 |
| Commission (options) | $0.65/contract | N/A |
| API Access | Free | Free |
| Min Account | $0 (paper) | $0 |
| Data Feeds | Paid (real-time) | Free (IEX) |

---

## Recommendation

### For Box Spread Strategy: Use IBKR
- Fix TWS connection (30 min)
- Keep existing codebase
- Production-ready in 2-3 weeks

### For Stock Strategies: Use Alpaca
- Implement broker abstraction (1 week)
- Add Alpaca adapter (1 week)
- Design new stock-based strategy (2-3 weeks)

### Hybrid: Use Both
- IBKR for options (box spreads)
- Alpaca for stocks (different strategies)
- Best of both worlds
- More complexity

---

## Next Steps

1. **Decision Point**: What do you want to trade?
   - Options (box spreads) → Fix TWS connection (30 min)
   - Stocks only → Implement Alpaca (1-2 weeks)
   - Both → Broker abstraction + both adapters (2-3 weeks)

2. **If fixing TWS**: Follow debug checklist above

3. **If adding Alpaca**: Start with broker abstraction layer

4. **If unsure**: Fix TWS first (fastest), add Alpaca later

---

## Questions to Answer

1. Do you want to keep doing box spread arbitrage? (Requires options → IBKR)
2. Or pivot to stock-based strategies? (Alpaca works)
3. Is TWS complexity the issue, or IBKR as a broker?
4. What's your timeline? (TWS fix = 30 min, Alpaca integration = 1-2 weeks)

---

## Contact

- **Alpaca Support**: support@alpaca.markets
- **IBKR Support**: https://www.interactivebrokers.com/en/support/contact-us.php
- **Alpaca Discord**: https://alpaca.markets/community
