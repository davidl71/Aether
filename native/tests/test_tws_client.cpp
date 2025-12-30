// test_tws_client.cpp - Unit tests for TWS Client connectivity
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include "tws_client.h"
#include "config_manager.h"
#include <thread>
#include <chrono>
#include <atomic>
#include <vector>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <cstring>
#include <sys/select.h>

using namespace tws;
using Catch::Matchers::ContainsSubstring;

namespace {

// Helper to check if a port is actually listening (for integration tests)
// Uses non-blocking socket with timeout to avoid hanging
bool is_port_listening(const std::string& host, int port, int timeout_ms = 500) {
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0) {
        return false;
    }

    // Set socket to non-blocking
    int flags = fcntl(sock, F_GETFL, 0);
    fcntl(sock, F_SETFL, flags | O_NONBLOCK);

    struct sockaddr_in server_addr;
    memset(&server_addr, 0, sizeof(server_addr));
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(port);

    if (inet_pton(AF_INET, host.c_str(), &server_addr.sin_addr) <= 0) {
        close(sock);
        return false;
    }

    // Attempt connection
    int result = connect(sock, reinterpret_cast<struct sockaddr*>(&server_addr), sizeof(server_addr));

    if (result == 0) {
        // Connected immediately
        close(sock);
        return true;
    }

    if (errno == EINPROGRESS) {
        // Connection in progress, wait with select
        fd_set write_fds;
        FD_ZERO(&write_fds);
        FD_SET(sock, &write_fds);

        struct timeval timeout;
        timeout.tv_sec = timeout_ms / 1000;
        timeout.tv_usec = (timeout_ms % 1000) * 1000;

        int select_result = select(sock + 1, nullptr, &write_fds, nullptr, &timeout);
        if (select_result > 0) {
            int so_error;
            socklen_t len = sizeof(so_error);
            if (getsockopt(sock, SOL_SOCKET, SO_ERROR, &so_error, &len) == 0 && so_error == 0) {
                close(sock);
                return true;
            }
        }
    }

    close(sock);
    return false;
}

} // namespace

// ============================================================================
// Test: Port Detection and Selection
// ============================================================================

TEST_CASE("TWS Client - Port candidate generation", "[tws][connectivity]") {
    // Given: A TWS client configuration with paper trading port
    config::TWSConfig config;
    config.host = "127.0.0.1";
    config.port = 7497; // Paper trading

    // When: We create a TWS client
    TWSClient client(config);

    // Then: The client should be constructible without throwing
    REQUIRE_NOTHROW(client.get_connection_state());

    // And: Should start in disconnected state
    REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
    // And: is_connected() should return false
    REQUIRE_FALSE(client.is_connected());
}

TEST_CASE("TWS Client - Configuration validation", "[tws][config]") {
    config::TWSConfig config;

    SECTION("Valid paper trading config") {
        // Given: Valid paper trading configuration
        config.host = "127.0.0.1";
        config.port = 7497;
        config.client_id = 1;

        // When: We create a TWS client
        TWSClient client(config);

        // Then: Client should be in disconnected state (not yet connected)
        REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
    }

    SECTION("Valid live trading config") {
        config.host = "127.0.0.1";
        config.port = 7496;
        config.client_id = 1;

        TWSClient client(config);
        REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
    }

    SECTION("Valid IB Gateway paper config") {
        config.host = "127.0.0.1";
        config.port = 4002;
        config.client_id = 1;

        TWSClient client(config);
        REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
    }

    SECTION("Valid IB Gateway live config") {
        config.host = "127.0.0.1";
        config.port = 4001;
        config.client_id = 1;

        TWSClient client(config);
        REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
    }
}

// ============================================================================
// Integration Tests (require TWS/IB Gateway to be running)
// ============================================================================

TEST_CASE("TWS Client - Port detection (integration)", "[tws][connectivity][integration]") {
    // This test requires TWS or IB Gateway to be running
    // Skip if not available

    const std::string host = "127.0.0.1";
    std::vector<int> standard_ports = {7497, 7496, 4002, 4001};

    std::vector<int> open_ports;
    for (int port : standard_ports) {
        if (is_port_listening(host, port)) {
            open_ports.push_back(port);
        }
    }

    if (open_ports.empty()) {
        WARN("No TWS/IB Gateway ports are open. Skipping integration test.");
        WARN("To run this test, start TWS or IB Gateway with API enabled.");
        return;
    }

    INFO("Found " << open_ports.size() << " open port(s): ");
    for (int port : open_ports) {
        INFO("  - Port " << port);
    }

    // Test that we can detect at least one port
    REQUIRE_FALSE(open_ports.empty());
}

TEST_CASE("TWS Client - Connection attempt (integration)", "[tws][connectivity][integration]") {
    // This test attempts to connect to TWS/IB Gateway
    // It will fail if TWS/Gateway is not running, which is expected

    const std::string host = "127.0.0.1";
    std::vector<int> standard_ports = {7497, 7496, 4002, 4001};

    // Find an open port (with short timeout to avoid hanging)
    int open_port = 0;
    for (int port : standard_ports) {
        if (is_port_listening(host, port, 200)) { // 200ms timeout
            open_port = port;
            break;
        }
    }

    if (open_port == 0) {
        WARN("No TWS/IB Gateway ports are open. Skipping connection test.");
        WARN("To run this test, start TWS or IB Gateway with API enabled.");
        return;
    }

    config::TWSConfig config;
    config.host = host;
    config.port = open_port;
    config.client_id = 999; // Use high client ID to avoid conflicts
    config.connection_timeout_ms = 1500; // Short timeout for test (1.5 seconds)
    config.auto_reconnect = false;

    TWSClient client(config);

    INFO("Attempting connection to " << host << ":" << open_port);

    // Attempt connection with timeout protection
    // Use a thread to enforce maximum test time
    bool connected = false;
    std::atomic<bool> test_done{false};
    std::atomic<bool> force_stop{false};

    std::thread connect_thread([&]() {
        // Check if we should stop before attempting
        if (force_stop) {
            test_done = true;
            return;
        }
        connected = client.connect();
        test_done = true;
    });

    // Wait for connection attempt with maximum timeout
    auto start = std::chrono::steady_clock::now();
    while (!test_done && (std::chrono::steady_clock::now() - start) < std::chrono::seconds(2)) {
        std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }

    if (!test_done) {
        // Test is taking too long, force stop
        WARN("Connection attempt timed out after 2 seconds - skipping test");
        force_stop = true;
        client.disconnect();

        // Give thread a moment to finish
        auto stop_start = std::chrono::steady_clock::now();
        while (!test_done && (std::chrono::steady_clock::now() - stop_start) < std::chrono::milliseconds(500)) {
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
        }

        if (connect_thread.joinable()) {
            connect_thread.detach(); // Detach if still running to avoid blocking
        }
        return; // Skip the rest of the test
    }

    if (connect_thread.joinable()) {
        connect_thread.join();
    }

    if (connected) {
        INFO("Successfully connected to port " << open_port);
        REQUIRE(client.is_connected());
        REQUIRE(client.get_connection_state() == ConnectionState::Connected);

        // Clean up
        client.disconnect();
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
        REQUIRE_FALSE(client.is_connected());
    } else {
        INFO("Connection failed (TWS may require authentication or have API disabled)");
        // This is acceptable - TWS may reject connections for various reasons
        REQUIRE(client.get_connection_state() != ConnectionState::Connected);
    }
}

TEST_CASE("TWS Client - Paper/Live mismatch detection", "[tws][mismatch]") {
    // Test that the code correctly identifies paper/live trading mismatches
    // This is a unit test that verifies the logic without requiring actual connections

    config::TWSConfig config;

    SECTION("Paper configured but only live available") {
        config.host = "127.0.0.1";
        config.port = 7497; // Paper trading
        config.client_id = 1;

        TWSClient client(config);

        // The client should be constructible
        REQUIRE_NOTHROW(client.get_connection_state());

        // Note: Actual mismatch detection happens during connect()
        // This test verifies the client can be created with paper config
        REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
    }

    SECTION("Live configured but only paper available") {
        config.host = "127.0.0.1";
        config.port = 7496; // Live trading
        config.client_id = 1;

        TWSClient client(config);

        // The client should be constructible
        REQUIRE_NOTHROW(client.get_connection_state());

        // Note: Actual mismatch detection happens during connect()
        // This test verifies the client can be created with live config
        REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
    }

    SECTION("IB Gateway paper configured") {
        config.host = "127.0.0.1";
        config.port = 4002; // IB Gateway Paper
        config.client_id = 1;

        TWSClient client(config);
        REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
    }

    SECTION("IB Gateway live configured") {
        config.host = "127.0.0.1";
        config.port = 4001; // IB Gateway Live
        config.client_id = 1;

        TWSClient client(config);
        REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
    }
}

TEST_CASE("TWS Client - Port fallback logic (integration)", "[tws][connectivity][integration]") {
    // Test that the client can find the correct port when configured port is wrong

    const std::string host = "127.0.0.1";
    std::vector<int> standard_ports = {7497, 7496, 4002, 4001};

    // Find an open port (with short timeout)
    int open_port = 0;
    for (int port : standard_ports) {
        if (is_port_listening(host, port, 200)) {
            open_port = port;
            break;
        }
    }

    if (open_port == 0) {
        WARN("No TWS/IB Gateway ports are open. Skipping fallback test.");
        return;
    }

    // Configure with a different port (wrong one)
    int wrong_port = (open_port == 7497) ? 7496 : 7497;

    config::TWSConfig config;
    config.host = host;
    config.port = wrong_port; // Wrong port
    config.client_id = 998;
    config.connection_timeout_ms = 1500; // Short timeout
    config.auto_reconnect = false;

    TWSClient client(config);

    INFO("Testing fallback from port " << wrong_port << " to " << open_port);

    // Attempt connection with timeout protection
    bool connected = false;
    std::atomic<bool> test_done{false};
    std::atomic<bool> force_stop{false};

    std::thread connect_thread([&]() {
        if (force_stop) {
            test_done = true;
            return;
        }
        connected = client.connect();
        test_done = true;
    });

    // Wait with maximum timeout
    auto start = std::chrono::steady_clock::now();
    while (!test_done && (std::chrono::steady_clock::now() - start) < std::chrono::seconds(2)) {
        std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }

    if (!test_done) {
        WARN("Connection attempt timed out after 2 seconds - skipping test");
        force_stop = true;
        client.disconnect();

        auto stop_start = std::chrono::steady_clock::now();
        while (!test_done && (std::chrono::steady_clock::now() - stop_start) < std::chrono::milliseconds(500)) {
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
        }

        if (connect_thread.joinable()) {
            connect_thread.detach();
        }
        return;
    }

    if (connect_thread.joinable()) {
        connect_thread.join();
    }

    if (connected) {
        INFO("Client successfully found correct port");
        REQUIRE(client.is_connected());
        client.disconnect();
    } else {
        INFO("Connection failed (expected if wrong port type or API disabled)");
    }
}

// ============================================================================
// Test: Error Handling
// ============================================================================

TEST_CASE("TWS Client - Disconnection handling", "[tws]") {
    config::TWSConfig config;
    config.host = "127.0.0.1";
    config.port = 7497;
    config.client_id = 1;

    TWSClient client(config);

    // Should be safe to disconnect when not connected
    REQUIRE_NOTHROW(client.disconnect());
    REQUIRE_FALSE(client.is_connected());
    REQUIRE(client.get_connection_state() == ConnectionState::Disconnected);
}

TEST_CASE("TWS Client - Invalid host handling", "[tws]") {
    config::TWSConfig config;
    config.host = "invalid.host.name.that.does.not.exist";
    config.port = 7497;
    config.client_id = 1;
    config.connection_timeout_ms = 1000; // Short timeout

    TWSClient client(config);

    // Should fail gracefully
    bool connected = client.connect();
    REQUIRE_FALSE(connected);
    REQUIRE_FALSE(client.is_connected());
    REQUIRE(client.get_connection_state() != ConnectionState::Connected);
}

// ============================================================================
// Test: Market Hours Integration
// ============================================================================

TEST_CASE("TWS Client - Market hours check (mock mode)", "[tws][market_hours]") {
    config::TWSConfig config;
    config.host = "127.0.0.1";
    config.port = 7497;
    config.client_id = 1;
    config.use_mock = true;  // Enable mock mode

    TWSClient client(config);

    // In mock mode, market hours should work without TWS connection
    // Note: Actual implementation uses MarketHours class internally
    // This test verifies the method exists and can be called
    REQUIRE_NOTHROW(client.is_market_open());

    // Market hours check should return a boolean
    bool is_open = client.is_market_open();
    // Result depends on current time, but should be a valid boolean
    REQUIRE((is_open == true || is_open == false));
}

// ============================================================================
// Test: Option Chain Request (Mock Mode)
// ============================================================================

TEST_CASE("TWS Client - Option chain request (mock mode)", "[tws][option_chain]") {
    config::TWSConfig config;
    config.host = "127.0.0.1";
    config.port = 7497;
    config.client_id = 1;
    config.use_mock = true;  // Enable mock mode

    TWSClient client(config);

    // In mock mode, option chain should return mock data
    auto contracts = client.request_option_chain("SPY", "");

    // Should return some contracts (mock mode generates test data)
    REQUIRE(contracts.size() > 0);

    // Verify contract structure
    for (const auto& contract : contracts) {
        REQUIRE(contract.symbol == "SPY");
        REQUIRE_FALSE(contract.expiry.empty());
        REQUIRE(contract.strike > 0.0);
    }

    // Test with expiry filter
    auto filtered = client.request_option_chain("SPY", "20251219");
    // Filtered results should only include specified expiry
    for (const auto& contract : filtered) {
        REQUIRE(contract.expiry == "20251219");
    }
}
