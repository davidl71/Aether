# Alpaca Integration Plan v2 (UPDATED)

**Date**: 2025-01-16
**Status**: Active - Alpaca SUPPORTS Options Trading
**Previous Version**: DEPRECATED (claimed Alpaca doesn't support options)

---

## ✅ CRITICAL UPDATE: Alpaca Supports Options!

**I was wrong in my initial assessment.** Alpaca launched **Level 3 Options Trading** and **multi-leg support** (announced Feb 2025).

### What Alpaca Supports

✅ **Options Trading**: Yes, with levels 1-3
✅ **Multi-leg Orders**: Up to 4 legs per order
✅ **Supported Strategies**:
- Call spreads
- Put spreads
- Iron condors
- Iron butterflies
- Strangles
- Straddles

### ⚠️ Box Spread Compatibility - NEEDS VERIFICATION

**Critical Question**: Are box spreads supported?

**What we know**:
- Box spreads = 4-leg strategy ✅ (Alpaca allows up to 4 legs)
- Alpaca requires: "All legs must be covered within the same MLeg order"
- Box spreads are theoretically covered (risk-free arbitrage)
- **BUT**: Box spreads are NOT explicitly mentioned in documentation

**Box Spread Composition**:
```
Long call @ K1   (covered by short put @ K1)
Short call @ K2  (covered by long put @ K2)
Long put @ K2    (covers short call @ K2)
Short put @ K1   (covers long call @ K1)
```

**Recommendation**: Contact Alpaca support to verify:
- Email: support@alpaca.markets
- Ask: "Does Level 3 support box spread arbitrage strategies?"

---

## Comparison: IBKR vs Alpaca for Box Spreads

| Feature | IBKR | Alpaca |
|---------|------|--------|
| **Options Support** | ✅ Full | ✅ Yes (Level 3) |
| **Box Spreads** | ✅ Explicit support | ❓ Unknown (needs verification) |
| **Multi-leg Orders** | ✅ BAG orders | ✅ MLeg orders (up to 4 legs) |
| **Atomic Execution** | ✅ Combo orders | ✅ MLeg orders |
| **API Complexity** | 🟡 C++ Socket API | ✅ Simple REST API |
| **Setup Difficulty** | 🟡 TWS/Gateway required | ✅ API keys only |
| **Commission (options)** | $0.65/contract | $0.50/contract |
| **Commission (stocks)** | $0.0035/share | $0 |
| **Paper Trading** | ✅ Free | ✅ Free |
| **Data Feeds** | 🟡 Paid (real-time) | ✅ Free (IEX) |
| **Min Account** | $0 | $0 |
| **Market Hours** | 🟡 Complex setup | ✅ Simple |

**Winner for Ease of Use**: Alpaca
**Winner for Explicit Box Support**: IBKR
**Winner for Cost**: Tie (very close)

---

## Alpaca Integration Architecture

### Option 1: Replace IBKR with Alpaca (Simplest)

**Pros**:
- Simpler API (REST vs Socket)
- No TWS/Gateway setup
- Lower commissions
- Free market data

**Cons**:
- Box spreads may not be supported (need verification)
- Less mature options platform
- Fewer advanced features

**Effort**: 3-5 days

---

### Option 2: Broker Abstraction (Both IBKR + Alpaca)

**Pros**:
- Use best of both platforms
- Fallback if one broker has issues
- Diversified execution

**Cons**:
- More complexity
- Maintain two integrations

**Effort**: 1-2 weeks

---

## Alpaca Integration Implementation

### Phase 1: Verify Box Spread Support (Day 1)

**Before writing code, verify box spreads work:**

1. **Create Paper Trading Account**:
   - Sign up: https://alpaca.markets/
   - Enable paper trading
   - Request Level 3 options approval

2. **Test Box Spread via Dashboard**:
   - Try to place a box spread manually
   - Use SPY or another liquid underlying
   - Document if it's accepted

3. **Test via API**:
   ```bash
   curl -X POST https://paper-api.alpaca.markets/v2/orders \
     -H "APCA-API-KEY-ID: YOUR_KEY" \
     -H "APCA-API-SECRET-KEY: YOUR_SECRET" \
     -H "Content-Type: application/json" \
     -d '{
       "order_class": "mleg",
       "type": "limit",
       "time_in_force": "day",
       "limit_price": "9.95",
       "legs": [
         {
           "symbol": "SPY250620C00500000",
           "ratio_qty": "1",
           "side": "buy",
           "position_intent": "buy_to_open"
         },
         {
           "symbol": "SPY250620C00510000",
           "ratio_qty": "1",
           "side": "sell",
           "position_intent": "sell_to_open"
         },
         {
           "symbol": "SPY250620P00510000",
           "ratio_qty": "1",
           "side": "buy",
           "position_intent": "buy_to_open"
         },
         {
           "symbol": "SPY250620P00500000",
           "ratio_qty": "1",
           "side": "sell",
           "position_intent": "sell_to_open"
         }
       ]
     }'
   ```

4. **If Rejected**: Document error message, contact support
5. **If Accepted**: Proceed with integration ✅

---

### Phase 2: Create Alpaca Adapter (Days 2-3)

**File**: `native/include/alpaca_adapter.h`

```cpp
#pragma once
#include "broker_adapter.h"
#include <string>
#include <memory>

namespace broker {

struct AlpacaConfig {
    std::string api_key;
    std::string secret_key;
    bool paper_trading = true;
    std::string base_url = "https://paper-api.alpaca.markets";
    std::string data_url = "https://data.alpaca.markets";
};

class AlpacaAdapter : public BrokerAdapter {
public:
    explicit AlpacaAdapter(const AlpacaConfig& config);
    ~AlpacaAdapter() override;

    // Connection
    bool connect() override;
    void disconnect() override;
    bool is_connected() const override;

    // Market data
    bool get_option_chain(
        const std::string& symbol,
        const std::string& expiry,
        option_chain::OptionChain& chain) override;

    int subscribe_market_data(
        const types::OptionContract& contract) override;

    // Orders
    int place_order(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        double limit_price = 0,
        types::TimeInForce tif = types::TimeInForce::Day) override;

    int place_box_spread(
        const types::BoxSpreadLeg& spread) override;

    bool cancel_order(int order_id) override;

    // Positions & account
    std::vector<types::Position> get_positions() const override;
    std::optional<types::Order> get_order(int order_id) const override;

    // Capabilities
    bool supports_options() const override { return true; }
    bool supports_box_spreads() const override { return box_spreads_supported_; }
    bool supports_combo_orders() const override { return true; }

private:
    class Impl;
    std::unique_ptr<Impl> pimpl_;
    bool box_spreads_supported_ = false;  // Set after verification
};

} // namespace broker
```

---

### Phase 3: Implement Alpaca REST Client (Days 3-4)

**Dependencies** (add to `CMakeLists.txt`):
```cmake
find_package(CURL REQUIRED)
find_package(nlohmann_json 3.11.2 REQUIRED)

target_link_libraries(ib_box_spread PRIVATE
    CURL::libcurl
    nlohmann_json::nlohmann_json
)
```

**File**: `native/src/alpaca_adapter.cpp`

```cpp
#include "alpaca_adapter.h"
#include <curl/curl.h>
#include <nlohmann/json.hpp>
#include <spdlog/spdlog.h>

using json = nlohmann::json;

namespace broker {

class AlpacaAdapter::Impl {
public:
    Impl(const AlpacaConfig& config) : config_(config) {
        curl_global_init(CURL_GLOBAL_DEFAULT);
    }

    ~Impl() {
        curl_global_cleanup();
    }

    // HTTP GET request
    std::string http_get(const std::string& endpoint) {
        CURL* curl = curl_easy_init();
        if (!curl) {
            throw std::runtime_error("Failed to initialize CURL");
        }

        std::string url = config_.base_url + endpoint;
        std::string response;

        struct curl_slist* headers = nullptr;
        headers = curl_slist_append(headers, ("APCA-API-KEY-ID: " + config_.api_key).c_str());
        headers = curl_slist_append(headers, ("APCA-API-SECRET-KEY: " + config_.secret_key).c_str());

        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);

        CURLcode res = curl_easy_perform(curl);
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);

        if (res != CURLE_OK) {
            throw std::runtime_error("CURL error: " + std::string(curl_easy_strerror(res)));
        }

        return response;
    }

    // HTTP POST request
    std::string http_post(const std::string& endpoint, const json& body) {
        CURL* curl = curl_easy_init();
        if (!curl) {
            throw std::runtime_error("Failed to initialize CURL");
        }

        std::string url = config_.base_url + endpoint;
        std::string response;
        std::string body_str = body.dump();

        struct curl_slist* headers = nullptr;
        headers = curl_slist_append(headers, ("APCA-API-KEY-ID: " + config_.api_key).c_str());
        headers = curl_slist_append(headers, ("APCA-API-SECRET-KEY: " + config_.secret_key).c_str());
        headers = curl_slist_append(headers, "Content-Type: application/json");

        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, body_str.c_str());
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);

        CURLcode res = curl_easy_perform(curl);
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);

        if (res != CURLE_OK) {
            throw std::runtime_error("CURL error: " + std::string(curl_easy_strerror(res)));
        }

        return response;
    }

    // Get option chain from Alpaca
    bool get_option_chain(const std::string& symbol, const std::string& expiry,
                         option_chain::OptionChain& chain) {
        try {
            // Alpaca endpoint: GET /v2/options/contracts
            std::string endpoint = "/v2/options/contracts?underlying_symbols=" + symbol;
            if (!expiry.empty()) {
                endpoint += "&expiration_date=" + expiry;
            }

            std::string response = http_get(endpoint);
            json contracts = json::parse(response);

            // Parse into option_chain::OptionChain
            // ... (implementation details)

            return true;
        } catch (const std::exception& e) {
            spdlog::error("Failed to get option chain: {}", e.what());
            return false;
        }
    }

    // Place box spread as multi-leg order
    std::string place_box_spread(const types::BoxSpreadLeg& spread) {
        try {
            // Build Alpaca MLeg order
            json order = {
                {"order_class", "mleg"},
                {"type", "limit"},
                {"time_in_force", "day"},
                {"limit_price", std::to_string(spread.net_debit)},
                {"legs", json::array()}
            };

            // Add all 4 legs
            order["legs"].push_back({
                {"symbol", format_option_symbol(spread.long_call)},
                {"ratio_qty", "1"},
                {"side", "buy"},
                {"position_intent", "buy_to_open"}
            });

            order["legs"].push_back({
                {"symbol", format_option_symbol(spread.short_call)},
                {"ratio_qty", "1"},
                {"side", "sell"},
                {"position_intent", "sell_to_open"}
            });

            order["legs"].push_back({
                {"symbol", format_option_symbol(spread.long_put)},
                {"ratio_qty", "1"},
                {"side", "buy"},
                {"position_intent", "buy_to_open"}
            });

            order["legs"].push_back({
                {"symbol", format_option_symbol(spread.short_put)},
                {"ratio_qty", "1"},
                {"side", "sell"},
                {"position_intent", "sell_to_open"}
            });

            spdlog::info("Placing box spread MLeg order: {}", order.dump(2));

            // POST to /v2/orders
            std::string response = http_post("/v2/orders", order);
            json order_response = json::parse(response);

            return order_response["id"].get<std::string>();

        } catch (const std::exception& e) {
            spdlog::error("Failed to place box spread: {}", e.what());
            throw;
        }
    }

    // Format option contract to Alpaca OCC symbol
    // Example: SPY250620C00500000
    std::string format_option_symbol(const types::OptionContract& contract) {
        // Alpaca uses OCC standard format:
        // [Root symbol][Expiry YYMMDD][C/P][Strike price * 1000, 8 digits]

        std::string symbol = contract.symbol;
        std::string expiry = contract.expiry.substr(2);  // YYYYMMDD -> YYMMDD
        char type = (contract.type == types::OptionType::Call) ? 'C' : 'P';
        int strike_int = static_cast<int>(contract.strike * 1000);

        std::ostringstream oss;
        oss << symbol << expiry << type << std::setw(8) << std::setfill('0') << strike_int;
        return oss.str();
    }

private:
    AlpacaConfig config_;

    static size_t write_callback(void* contents, size_t size, size_t nmemb, void* userp) {
        ((std::string*)userp)->append((char*)contents, size * nmemb);
        return size * nmemb;
    }
};

// AlpacaAdapter public methods
AlpacaAdapter::AlpacaAdapter(const AlpacaConfig& config)
    : pimpl_(std::make_unique<Impl>(config)) {
    spdlog::info("AlpacaAdapter created (paper_trading={})", config.paper_trading);
}

AlpacaAdapter::~AlpacaAdapter() = default;

bool AlpacaAdapter::connect() {
    try {
        // Test connection by getting account info
        auto response = pimpl_->http_get("/v2/account");
        auto account = json::parse(response);

        spdlog::info("Connected to Alpaca (account: {})", account["account_number"].get<std::string>());

        // Check if Level 3 options enabled
        std::string options_trading_level = account.value("options_trading_level", "0");
        if (options_trading_level != "3") {
            spdlog::warn("Options trading level is {}, not Level 3. Box spreads may not work.", options_trading_level);
            box_spreads_supported_ = false;
        } else {
            spdlog::info("Level 3 options enabled");
            box_spreads_supported_ = true;  // Assume supported, will verify in testing
        }

        return true;
    } catch (const std::exception& e) {
        spdlog::error("Failed to connect to Alpaca: {}", e.what());
        return false;
    }
}

bool AlpacaAdapter::get_option_chain(const std::string& symbol, const std::string& expiry,
                                     option_chain::OptionChain& chain) {
    return pimpl_->get_option_chain(symbol, expiry, chain);
}

int AlpacaAdapter::place_box_spread(const types::BoxSpreadLeg& spread) {
    std::string order_id = pimpl_->place_box_spread(spread);
    // Convert Alpaca UUID to int (hash it or use a mapping)
    return std::hash<std::string>{}(order_id);  // Simplified
}

} // namespace broker
```

---

### Phase 4: Configuration (Day 4)

**File**: `config/config.json`

```json
{
  "broker": "alpaca",  // "ibkr" or "alpaca"

  "alpaca": {
    "api_key": "${ALPACA_API_KEY}",
    "secret_key": "${ALPACA_SECRET_KEY}",
    "paper_trading": true,
    "base_url": "https://paper-api.alpaca.markets",
    "data_url": "https://data.alpaca.markets"
  },

  "ibkr": {
    "host": "127.0.0.1",
    "port": 7497,
    "client_id": 1,
    "auto_reconnect": true
  }
}
```

**Environment Variables**:
```bash
export ALPACA_API_KEY="PK..."
export ALPACA_SECRET_KEY="..."
```

---

### Phase 5: Update Main App (Day 5)

**File**: `native/src/ib_box_spread.cpp`

```cpp
// Select broker based on config
std::unique_ptr<broker::BrokerAdapter> broker;

if (config.broker == "alpaca") {
    broker::AlpacaConfig alpaca_config;
    alpaca_config.api_key = config.alpaca.api_key;
    alpaca_config.secret_key = config.alpaca.secret_key;
    alpaca_config.paper_trading = config.alpaca.paper_trading;

    broker = std::make_unique<broker::AlpacaAdapter>(alpaca_config);
    spdlog::info("Using Alpaca broker");

} else if (config.broker == "ibkr") {
    broker = std::make_unique<broker::IBKRAdapter>(config.ibkr);
    spdlog::info("Using IBKR broker");

} else {
    spdlog::error("Unknown broker: {}", config.broker);
    return 1;
}

// Connect
if (!broker->connect()) {
    spdlog::error("Failed to connect to broker");
    return 1;
}

// Verify box spread support
if (!broker->supports_box_spreads()) {
    spdlog::warn("⚠️  Broker may not support box spreads. Proceed with caution.");
}

// Rest of application uses broker interface
BoxSpreadStrategy strategy(broker.get(), &order_mgr, params);
```

---

## Testing Plan

### Week 1: Verification & Setup
- [ ] Create Alpaca paper trading account
- [ ] Get Level 3 options approval
- [ ] Test box spread via dashboard (manual)
- [ ] Test box spread via API (curl)
- [ ] Document results

### Week 2: Integration
- [ ] Implement AlpacaAdapter
- [ ] Write unit tests
- [ ] Test option chain fetching
- [ ] Test single-leg orders
- [ ] Test multi-leg orders

### Week 3: Box Spread Testing
- [ ] Place test box spread (paper)
- [ ] Monitor execution
- [ ] Verify atomic fill
- [ ] Test edge cases
- [ ] Performance testing

---

## Decision Matrix

### Choose Alpaca if:
✅ You want simpler API (REST)
✅ You're frustrated with TWS setup
✅ Lower commissions matter
✅ Free market data is important
✅ Box spreads are verified to work

### Choose IBKR if:
✅ Box spread support is 100% confirmed
✅ You need advanced features
✅ More mature options platform preferred
✅ You can handle TWS complexity

### Do Both (Hybrid) if:
✅ You want redundancy
✅ Broker diversification is important
✅ You have time for dual integration

---

## Next Steps

1. **VERIFY BOX SPREADS WORK** (Critical, Day 1)
   - Create Alpaca account
   - Test manually or via API
   - Contact support if unclear

2. **If Box Spreads Work**:
   - Proceed with integration (5-7 days)
   - Test thoroughly in paper
   - Compare performance with IBKR

3. **If Box Spreads Don't Work**:
   - Fix TWS connection instead (30 min)
   - Or use Alpaca for other strategies
   - Or switch to Tradier

---

## Resources

- **Alpaca Docs**: https://docs.alpaca.markets/docs/options-trading
- **Level 3 Options**: https://docs.alpaca.markets/docs/options-level-3-trading
- **Support**: support@alpaca.markets
- **Discord**: https://alpaca.markets/community
- **API Reference**: https://docs.alpaca.markets/reference

---

## Cost Estimate

**Integration Effort**: 5-7 days
**Testing Effort**: 1 week (paper trading)
**Total**: 2-3 weeks to production

**Savings vs TWS**:
- Setup time: -4 hours (no TWS/Gateway)
- Maintenance: Easier (REST vs Socket)
- Commissions: $0.15/contract savings
- Data costs: $0/month (vs $10-50/month for IBKR real-time)

---

## Recommendation

1. **Test box spreads in paper trading FIRST** (1 day)
2. **If successful**: Integrate Alpaca (1 week)
3. **If not**: Fix TWS connection (30 min) or use Tradier

**My gut feeling**: Box spreads SHOULD work since:
- They're fully covered (no naked positions)
- 4 legs fit Alpaca's limit
- All legs offset each other

But **verify first** before investing dev time!
