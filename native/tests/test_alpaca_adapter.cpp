// test_alpaca_adapter.cpp - Unit tests for Alpaca adapter
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include "brokers/alpaca_adapter.h"
#include "brokers/broker_interface.h"
#include "types.h"
#include <thread>
#include <chrono>
#include <atomic>
#include <vector>
#include <memory>

using namespace brokers;
using namespace box_spread::types;
using namespace box_spread::brokers;
using Catch::Matchers::ContainsSubstring;

namespace {

// Helper to create mock Alpaca config for testing
AlpacaAdapter::Config create_mock_config() {
  AlpacaAdapter::Config config;
  config.api_key_id = "TEST_API_KEY";
  config.api_secret_key = "TEST_SECRET_KEY";
  config.base_url = "https://paper-api.alpaca.markets";
  config.paper_trading = true;
  config.poll_interval_ms = 2000;
  return config;
}

// Helper to create a test option contract
OptionContract create_test_contract() {
  OptionContract contract;
  contract.symbol = "SPY";
  contract.expiry = "20250117";  // Jan 17, 2025
  contract.strike = 500.0;
  contract.type = OptionType::Call;
  contract.style = OptionStyle::American;
  contract.exchange = "CBOE";
  return contract;
}

// Helper to wait for connection state change
bool wait_for_state(AlpacaAdapter& adapter, ConnectionState expected_state, int timeout_ms = 5000) {
  auto start = std::chrono::steady_clock::now();
  while ((std::chrono::steady_clock::now() - start) < std::chrono::milliseconds(timeout_ms)) {
    if (adapter.get_connection_state() == expected_state) {
      return true;
    }
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
  }
  return false;
}

// Helper to wait for market data callback
struct MarketDataWaiter {
  std::mutex mutex;
  std::condition_variable cv;
  std::atomic<bool> received{false};
  MarketData received_data;
  int timeout_ms;

  MarketDataWaiter(int timeout = 5000) : timeout_ms(timeout) {}

  void callback(const MarketData& data) {
    std::lock_guard<std::mutex> lock(mutex);
    received_data = data;
    received = true;
    cv.notify_one();
  }

  bool wait() {
    std::unique_lock<std::mutex> lock(mutex);
    return cv.wait_for(lock, std::chrono::milliseconds(timeout_ms),
                      [this] { return received.load(); });
  }
};

} // namespace

// ============================================================================
// Test: Adapter Construction and Configuration
// ============================================================================

TEST_CASE("Alpaca Adapter - Construction", "[alpaca][adapter]") {
  AlpacaAdapter::Config config = create_mock_config();

  SECTION("Default construction") {
    REQUIRE_NOTHROW(AlpacaAdapter adapter(config));
  }

  SECTION("Initial state") {
    AlpacaAdapter adapter(config);
    REQUIRE_FALSE(adapter.is_connected());
    REQUIRE(adapter.get_connection_state() == ConnectionState::Disconnected);
    REQUIRE(adapter.get_broker_type() == BrokerType::ALPACA);
  }

  SECTION("Capabilities") {
    AlpacaAdapter adapter(config);
    auto caps = adapter.get_capabilities();
    REQUIRE(caps.supports_options == true);
    REQUIRE(caps.supports_multi_leg_orders == true);
    REQUIRE(caps.rate_limit_per_minute == 200);
  }
}

// ============================================================================
// Test: Connection Management
// ============================================================================

TEST_CASE("Alpaca Adapter - Connection management", "[alpaca][adapter][connection]") {
  AlpacaAdapter::Config config = create_mock_config();
  AlpacaAdapter adapter(config);

  SECTION("Disconnect when not connected") {
    REQUIRE_NOTHROW(adapter.disconnect());
    REQUIRE_FALSE(adapter.is_connected());
  }

  SECTION("Connection state transitions") {
    // Start disconnected
    REQUIRE(adapter.get_connection_state() == ConnectionState::Disconnected);

    // Attempt connection (will fail without real API keys, but should handle gracefully)
    bool connected = adapter.connect();

    // Should either connect (if API keys valid) or fail gracefully
    if (!connected) {
      // Connection failed - should be in Error or Disconnected state
      auto state = adapter.get_connection_state();
      REQUIRE((state == ConnectionState::Disconnected || state == ConnectionState::Error));
    } else {
      // Connection succeeded - should be Connected
      REQUIRE(adapter.get_connection_state() == ConnectionState::Connected);
      REQUIRE(adapter.is_connected());

      // Disconnect
      adapter.disconnect();
      REQUIRE_FALSE(adapter.is_connected());
      REQUIRE(adapter.get_connection_state() == ConnectionState::Disconnected);
    }
  }
}

// ============================================================================
// Test: Symbol Conversion
// ============================================================================

TEST_CASE("Alpaca Adapter - Symbol conversion", "[alpaca][adapter][symbol]") {
  AlpacaAdapter::Config config = create_mock_config();
  AlpacaAdapter adapter(config);

  SECTION("Convert option contract to Alpaca symbol") {
    OptionContract contract = create_test_contract();

    // SPY + 20250117 + C + 500000 (strike * 1000, zero-padded to 8 digits)
    // Expected: SPY20250117C00500000
    // Note: This is a private method, so we test via public methods that use it

    // Test via contract details request (uses symbol conversion internally)
    long contract_id = adapter.request_contract_details_sync(contract, 1000);
    REQUIRE(contract_id > 0);  // Should generate a contract ID (hash of symbol)
  }

  SECTION("Multiple contracts generate different IDs") {
    OptionContract call = create_test_contract();
    OptionContract put = create_test_contract();
    put.type = OptionType::Put;

    long call_id = adapter.request_contract_details_sync(call, 1000);
    long put_id = adapter.request_contract_details_sync(put, 1000);

    REQUIRE(call_id != put_id);  // Different contracts should have different IDs
  }
}

// ============================================================================
// Test: Market Data
// ============================================================================

TEST_CASE("Alpaca Adapter - Market data", "[alpaca][adapter][market-data]") {
  AlpacaAdapter::Config config = create_mock_config();
  AlpacaAdapter adapter(config);

  SECTION("Request market data when not connected") {
    OptionContract contract = create_test_contract();

    // Should return error code when not connected
    int request_id = adapter.request_market_data(
      contract,
      [](const MarketData&) {}
    );

    REQUIRE(request_id < 0);  // Error code
  }

  SECTION("Cancel market data") {
    // Should handle gracefully even if not subscribed
    REQUIRE_NOTHROW(adapter.cancel_market_data(999));
  }

  SECTION("Sync market data request when not connected") {
    OptionContract contract = create_test_contract();

    auto data = adapter.request_market_data_sync(contract, 1000);
    // Without connection, should return nullopt
    REQUIRE_FALSE(data.has_value());
  }
}

// ============================================================================
// Test: Options Chain
// ============================================================================

TEST_CASE("Alpaca Adapter - Options chain", "[alpaca][adapter][options-chain]") {
  AlpacaAdapter::Config config = create_mock_config();
  AlpacaAdapter adapter(config);

  SECTION("Request options chain") {
    std::vector<OptionContract> chain = adapter.request_option_chain("SPY", "");

    // Without connection, should return empty chain
    // With connection, would return actual chain
    // This test verifies the method doesn't crash
    REQUIRE_NOTHROW(chain);
  }

  SECTION("Request options chain with expiry filter") {
    std::vector<OptionContract> chain = adapter.request_option_chain("SPY", "20250117");

    REQUIRE_NOTHROW(chain);
  }
}

// ============================================================================
// Test: Order Management
// ============================================================================

TEST_CASE("Alpaca Adapter - Order management", "[alpaca][adapter][orders]") {
  AlpacaAdapter::Config config = create_mock_config();
  AlpacaAdapter adapter(config);

  SECTION("Place order when not connected") {
    OptionContract contract = create_test_contract();

    int order_id = adapter.place_order(
      contract,
      OrderAction::Buy,
      1,
      100.0,
      TimeInForce::Day
    );

    REQUIRE(order_id < 0);  // Error code when not connected
  }

  SECTION("Cancel order when not connected") {
    bool cancelled = adapter.cancel_order(999);
    REQUIRE_FALSE(cancelled);
  }

  SECTION("Get order status when not connected") {
    auto order = adapter.get_order_status(999);
    REQUIRE_FALSE(order.has_value());
  }
}

// ============================================================================
// Test: Multi-Leg Orders
// ============================================================================

TEST_CASE("Alpaca Adapter - Multi-leg orders", "[alpaca][adapter][combo-orders]") {
  AlpacaAdapter::Config config = create_mock_config();
  AlpacaAdapter adapter(config);

  SECTION("Place combo order when not connected") {
    std::vector<OptionContract> contracts = {create_test_contract()};
    std::vector<OrderAction> actions = {OrderAction::Buy};
    std::vector<int> quantities = {1};
    std::vector<long> contract_ids = {123};
    std::vector<double> limit_prices = {100.0};

    int order_id = adapter.place_combo_order(
      contracts,
      actions,
      quantities,
      contract_ids,
      limit_prices
    );

    REQUIRE(order_id < 0);  // Error code when not connected
  }

  SECTION("Place combo order with mismatched parameters") {
    std::vector<OptionContract> contracts = {create_test_contract()};
    std::vector<OrderAction> actions = {OrderAction::Buy, OrderAction::Sell};  // Mismatch
    std::vector<int> quantities = {1};
    std::vector<long> contract_ids = {123};
    std::vector<double> limit_prices = {100.0};

    int order_id = adapter.place_combo_order(
      contracts,
      actions,
      quantities,
      contract_ids,
      limit_prices
    );

    REQUIRE(order_id < 0);  // Error code for mismatch
  }
}

// ============================================================================
// Test: Positions
// ============================================================================

TEST_CASE("Alpaca Adapter - Positions", "[alpaca][adapter][positions]") {
  AlpacaAdapter::Config config = create_mock_config();
  AlpacaAdapter adapter(config);

  SECTION("Get positions when not connected") {
    std::vector<Position> positions = adapter.get_positions();

    // Without connection, should return empty vector
    REQUIRE(positions.empty());
  }

  SECTION("Get position for contract when not connected") {
    OptionContract contract = create_test_contract();
    auto position = adapter.get_position(contract);

    REQUIRE_FALSE(position.has_value());
  }
}

// ============================================================================
// Test: Account Information
// ============================================================================

TEST_CASE("Alpaca Adapter - Account information", "[alpaca][adapter][account]") {
  AlpacaAdapter::Config config = create_mock_config();
  AlpacaAdapter adapter(config);

  SECTION("Get account info when not connected") {
    auto account = adapter.get_account_info();
    REQUIRE_FALSE(account.has_value());
  }

  SECTION("Get buying power when not connected") {
    double buying_power = adapter.get_buying_power();
    REQUIRE(buying_power == 0.0);
  }

  SECTION("Get net liquidation value when not connected") {
    double net_liq = adapter.get_net_liquidation_value();
    REQUIRE(net_liq == 0.0);
  }
}

// ============================================================================
// Test: Error Handling
// ============================================================================

TEST_CASE("Alpaca Adapter - Error handling", "[alpaca][adapter][error-handling]") {
  AlpacaAdapter::Config config = create_mock_config();
  AlpacaAdapter adapter(config);

  SECTION("Set error callback") {
    bool callback_called = false;
    std::string error_message;
    int error_code = 0;

    adapter.set_error_callback([&](int code, const std::string& msg) {
      callback_called = true;
      error_code = code;
      error_message = msg;
    });

    REQUIRE_NOTHROW(adapter.set_error_callback(nullptr));
  }
}

// ============================================================================
// Test: Integration with Paper Trading (Optional)
// ============================================================================

TEST_CASE("Alpaca Adapter - Paper trading integration", "[alpaca][adapter][integration][!mayfail]") {
  // This test requires valid Alpaca API keys and paper trading account
  // Skip gracefully if not available

  const char* api_key = std::getenv("ALPACA_API_KEY_ID");
  const char* api_secret = std::getenv("ALPACA_API_SECRET_KEY");

  if (!api_key || !api_secret) {
    WARN("Alpaca API keys not found in environment - skipping integration test");
    WARN("Set ALPACA_API_KEY_ID and ALPACA_API_SECRET_KEY to run this test");
    return;
  }

  AlpacaAdapter::Config config;
  config.api_key_id = api_key;
  config.api_secret_key = api_secret;
  config.base_url = "https://paper-api.alpaca.markets";
  config.paper_trading = true;
  config.poll_interval_ms = 2000;

  AlpacaAdapter adapter(config);

  SECTION("Real connection to paper trading") {
    bool connected = adapter.connect();

    if (!connected) {
      WARN("Failed to connect to Alpaca paper trading - check API keys");
      return;
    }

    REQUIRE(adapter.is_connected());
    REQUIRE(adapter.get_connection_state() == ConnectionState::Connected);

    SECTION("Get account info") {
      auto account = adapter.get_account_info();
      REQUIRE(account.has_value());
      REQUIRE_FALSE(account->account_id.empty());
    }

    SECTION("Get positions") {
      auto positions = adapter.get_positions();
      REQUIRE_NOTHROW(positions);
    }

    // Clean up
    adapter.disconnect();
    REQUIRE_FALSE(adapter.is_connected());
  }
}
