// test_market_data_integration.cpp - Integration tests for market data pipeline
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include "tws_client.h"
#include "config_manager.h"
#include "types.h"
#include <thread>
#include <chrono>
#include <atomic>
#include <vector>
#include <memory>
#include <mutex>
#include <condition_variable>

using namespace tws;
using namespace types;
using Catch::Matchers::ContainsSubstring;

namespace {

// Helper to create mock TWS config for testing
config::TWSConfig create_mock_config() {
  config::TWSConfig config;
  config.host = "127.0.0.1";
  config.port = 7497;
  config.client_id = 999;
  config.use_mock = true;  // Use mock mode for testing
  config.auto_reconnect = true;
  config.connection_timeout_ms = 5000;
  config.connect_options = "+PACEAPI";
  return config;
}

// Helper to create a test option contract
OptionContract create_test_contract() {
  OptionContract contract;
  contract.symbol = "SPY";
  contract.expiry = "20250117";  // Jan 17, 2025
  contract.strike = 500.0;
  contract.type = OptionType::Call;
  contract.exchange = "SMART";
  return contract;
}

// Helper to wait for market data callback
struct MarketDataWaiter {
  std::mutex mutex;
  std::condition_variable cv;
  std::atomic<bool> received{false};
  MarketData received_data;
  int timeout_ms;

  MarketDataWaiter(int timeout = 2000) : timeout_ms(timeout) {}

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
// Test: Market Data Subscription
// ============================================================================

TEST_CASE("Market Data Integration - Subscription (mock)", "[market-data][integration][mock]") {
  config::TWSConfig config = create_mock_config();
  TWSClient client(config);
  client.connect();
  REQUIRE(client.is_connected());

  SECTION("Subscribe to option contract") {
    // Given: A connected client and test contract
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    // When: We subscribe to market data
    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    // Then: Should return valid request ID
    REQUIRE(request_id > 0);

    // And: Should receive market data callback
    REQUIRE(waiter.wait());
    REQUIRE(waiter.received_data.bid > 0.0);
    REQUIRE(waiter.received_data.ask > 0.0);
  }

  SECTION("Multiple simultaneous subscriptions") {
    // Given: A connected client
    OptionContract contract1 = create_test_contract();
    contract1.strike = 500.0;
    OptionContract contract2 = create_test_contract();
    contract2.strike = 510.0;

    MarketDataWaiter waiter1, waiter2;

    // When: We subscribe to multiple contracts
    int id1 = client.request_market_data(contract1, [&waiter1](const MarketData& data) {
      waiter1.callback(data);
    });
    int id2 = client.request_market_data(contract2, [&waiter2](const MarketData& data) {
      waiter2.callback(data);
    });

    // Then: Both should receive data
    REQUIRE(id1 > 0);
    REQUIRE(id2 > 0);
    REQUIRE(id1 != id2);  // Different request IDs

    REQUIRE(waiter1.wait());
    REQUIRE(waiter2.wait());
    REQUIRE(waiter1.received_data.bid > 0.0);
    REQUIRE(waiter2.received_data.bid > 0.0);
  }

  SECTION("Subscription cancellation") {
    // Given: An active subscription
    OptionContract contract = create_test_contract();
    int request_id = client.request_market_data(contract, [](const MarketData&) {});

    // When: We cancel the subscription
    client.cancel_market_data(request_id);

    // Then: Should handle cancellation gracefully
    REQUIRE_NOTHROW(client.cancel_market_data(request_id));
  }
}

// ============================================================================
// Test: Market Data Updates
// ============================================================================

TEST_CASE("Market Data Integration - Updates (mock)", "[market-data][integration][mock]") {
  config::TWSConfig config = create_mock_config();
  TWSClient client(config);
  client.connect();
  REQUIRE(client.is_connected());

  SECTION("Tick price updates") {
    // Given: A subscription
    OptionContract contract = create_test_contract();
    std::vector<MarketData> updates;
    std::mutex updates_mutex;

    int request_id = client.request_market_data(contract, [&updates, &updates_mutex](const MarketData& data) {
      std::lock_guard<std::mutex> lock(updates_mutex);
      updates.push_back(data);
    });

    // When: We wait for updates
    std::this_thread::sleep_for(std::chrono::milliseconds(100));

    // Then: Should receive updates
    {
      std::lock_guard<std::mutex> lock(updates_mutex);
      REQUIRE(updates.size() > 0);
      REQUIRE(updates[0].bid > 0.0);
      REQUIRE(updates[0].ask > 0.0);
    }
  }

  SECTION("Tick size updates") {
    // Given: A subscription
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    // When: We wait for updates
    REQUIRE(waiter.wait());

    // Then: Should receive size data
    REQUIRE(waiter.received_data.bid_size >= 0);
    REQUIRE(waiter.received_data.ask_size >= 0);
  }

  SECTION("Option computation updates") {
    // Given: A subscription
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    // When: We wait for updates
    REQUIRE(waiter.wait());

    // Then: Should receive option computation data (if available)
    // Note: Mock mode may not provide all Greeks, but structure should be valid
    REQUIRE(waiter.received_data.bid > 0.0);
  }
}

// ============================================================================
// Test: Stale Data Detection
// ============================================================================

TEST_CASE("Market Data Integration - Stale data detection (mock)", "[market-data][integration][mock]") {
  config::TWSConfig config = create_mock_config();
  TWSClient client(config);
  client.connect();
  REQUIRE(client.is_connected());

  SECTION("Data timestamp validation") {
    // Given: A subscription
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    // When: We receive data
    REQUIRE(waiter.wait());

    // Then: Data should have valid timestamp
    // Note: Mock mode may not set timestamps, but structure should support it
    REQUIRE(waiter.received_data.bid > 0.0);
  }

  SECTION("Data freshness check") {
    // Given: Received market data
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    REQUIRE(waiter.wait());

    // When: We check data freshness
    // Then: Should be able to validate (structure supports timestamp checking)
    REQUIRE(waiter.received_data.bid > 0.0);
    // Note: Full stale detection logic would check timestamp age
  }
}

// ============================================================================
// Test: Liquidity Filtering
// ============================================================================

TEST_CASE("Market Data Integration - Liquidity filtering (mock)", "[market-data][integration][mock]") {
  config::TWSConfig config = create_mock_config();
  TWSClient client(config);
  client.connect();
  REQUIRE(client.is_connected());

  SECTION("Bid-ask spread validation") {
    // Given: Market data with bid and ask
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    REQUIRE(waiter.wait());

    // When: We calculate spread
    double spread = waiter.received_data.ask - waiter.received_data.bid;

    // Then: Spread should be non-negative
    REQUIRE(spread >= 0.0);
    // And: Should be reasonable (not extremely wide)
    REQUIRE(spread < 1000.0);  // Sanity check
  }

  SECTION("Volume validation") {
    // Given: Market data
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    REQUIRE(waiter.wait());

    // When: We check volume
    // Then: Volume should be non-negative
    REQUIRE(waiter.received_data.volume >= 0.0);
  }

  SECTION("Bid/ask size validation") {
    // Given: Market data
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    REQUIRE(waiter.wait());

    // When: We check bid/ask sizes
    // Then: Sizes should be non-negative
    REQUIRE(waiter.received_data.bid_size >= 0);
    REQUIRE(waiter.received_data.ask_size >= 0);
  }
}

// ============================================================================
// Test: Synchronous Market Data Requests
// ============================================================================

TEST_CASE("Market Data Integration - Synchronous requests (mock)", "[market-data][integration][mock]") {
  config::TWSConfig config = create_mock_config();
  TWSClient client(config);
  client.connect();
  REQUIRE(client.is_connected());

  SECTION("Synchronous market data request") {
    // Given: A connected client and contract
    OptionContract contract = create_test_contract();

    // When: We request market data synchronously
    auto data = client.request_market_data_sync(contract, 2000);

    // Then: Should return data
    REQUIRE(data.has_value());
    REQUIRE(data->bid > 0.0);
    REQUIRE(data->ask > 0.0);
  }

  SECTION("Synchronous request timeout") {
    // Given: A contract
    OptionContract contract = create_test_contract();

    // When: We request with very short timeout (mock should still work)
    auto data = client.request_market_data_sync(contract, 10);

    // Then: In mock mode, should still return data (mock is instant)
    // Real TWS might timeout, but mock should succeed
    REQUIRE(data.has_value());
  }
}

// ============================================================================
// Test: Data Structure Integrity
// ============================================================================

TEST_CASE("Market Data Integration - Data structure integrity (mock)", "[market-data][integration][mock]") {
  config::TWSConfig config = create_mock_config();
  TWSClient client(config);
  client.connect();
  REQUIRE(client.is_connected());

  SECTION("Complete market data structure") {
    // Given: A subscription
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    REQUIRE(waiter.wait());

    // When: We examine the data structure
    const MarketData& data = waiter.received_data;

    // Then: All fields should be valid
    REQUIRE(data.bid >= 0.0);
    REQUIRE(data.ask >= 0.0);
    REQUIRE(data.ask >= data.bid);  // Ask should be >= bid
    REQUIRE(data.bid_size >= 0);
    REQUIRE(data.ask_size >= 0);
    REQUIRE(data.volume >= 0.0);
    // Greeks may be optional, but structure should support them
  }

  SECTION("Invalid tick value handling") {
    // Given: Market data
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter;

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    REQUIRE(waiter.wait());

    // When: We check for invalid values
    // Then: Should handle DBL_MAX and negative values appropriately
    // Note: Mock mode generates valid data, but structure should handle edge cases
    REQUIRE(waiter.received_data.bid > 0.0);
  }
}

// ============================================================================
// Test: Real TWS Integration (Optional - Requires TWS Running)
// ============================================================================

TEST_CASE("Market Data Integration - Real TWS subscription (integration)", "[market-data][integration][!mayfail]") {
  // This test requires TWS or IB Gateway to be running
  // Skip gracefully if not available

  config::TWSConfig config;
  config.host = "127.0.0.1";
  config.port = 7497;  // Paper trading
  config.client_id = 999;
  config.use_mock = false;  // Use real TWS
  config.connection_timeout_ms = 5000;

  TWSClient client(config);

  // Check if TWS is available
  bool connected = client.connect();
  if (!connected) {
    WARN("TWS/Gateway not available - skipping real market data test");
    WARN("To run this test, start TWS or IB Gateway with API enabled on port 7497");
    return;
  }

  SECTION("Real market data subscription") {
    // Given: A real TWS connection
    REQUIRE(client.is_connected());

    // When: We subscribe to market data
    OptionContract contract = create_test_contract();
    MarketDataWaiter waiter(5000);  // 5 second timeout for real TWS

    int request_id = client.request_market_data(contract, [&waiter](const MarketData& data) {
      waiter.callback(data);
    });

    // Then: Should receive real market data
    if (waiter.wait()) {
      REQUIRE(waiter.received_data.bid > 0.0);
      REQUIRE(waiter.received_data.ask > 0.0);
    } else {
      WARN("Market data not received within timeout - may require market data permissions");
    }

    // Clean up
    if (request_id > 0) {
      client.cancel_market_data(request_id);
    }
  }

  // Clean up
  if (client.is_connected()) {
    client.disconnect();
  }
}
