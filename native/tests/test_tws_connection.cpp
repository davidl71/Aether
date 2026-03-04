// test_tws_connection.cpp - Standalone TWS API connection test (not a Catch2
// test) Build via main native CMake; binary: build/bin/test_tws_connection (or
// build-native/bin/...)
//
// Usage: ./test_tws_connection [host] [port] [client_id]
// Example: ./test_tws_connection 127.0.0.1 7497 1

#include "tws_client.h"
#include <chrono>
#include <iostream>
#include <spdlog/spdlog.h>
#include <thread>

using namespace tws;

int main(int argc, char *argv[]) {
  // Setup logging
  spdlog::set_level(spdlog::level::debug);
  spdlog::set_pattern("[%Y-%m-%d %H:%M:%S.%e] [%^%l%$] %v");

  // Parse arguments
  std::string host = "127.0.0.1";
  int port = 4002; // Default to IB Gateway Paper Trading (4002) instead of TWS
                   // Paper (7497)
  int client_id = 999; // Use a unique client ID for testing

  if (argc >= 2) {
    host = argv[1];
  }
  if (argc >= 3) {
    port = std::stoi(argv[2]);
  }
  if (argc >= 4) {
    client_id = std::stoi(argv[3]);
  }

  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
               "━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << "  TWS API Connection Test" << std::endl;
  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
               "━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << std::endl;
  std::cout << "Configuration:" << std::endl;
  std::cout << "  Host: " << host << std::endl;
  std::cout << "  Port: " << port << std::endl;
  std::cout << "  Client ID: " << client_id << std::endl;
  std::cout << std::endl;

  // Port reference
  std::cout << "Port Reference:" << std::endl;
  std::cout << "  4002 = IB Gateway Paper Trading (default)" << std::endl;
  std::cout << "  4001 = IB Gateway Live Trading" << std::endl;
  std::cout << "  7497 = TWS Paper Trading" << std::endl;
  std::cout << "  7496 = TWS Live Trading" << std::endl;
  std::cout << std::endl;

  // Create config
  config::TWSConfig config;
  config.host = host;
  config.port = port;
  config.client_id = client_id;
  config.connection_timeout_ms = 30000; // 30 second timeout
  config.auto_reconnect = false;        // Don't auto-reconnect in test

  // Create client
  std::cout << "Creating TWS client..." << std::endl;
  auto client = std::make_unique<TWSClient>(config);

  // Set error callback
  client->set_error_callback([](int code, const std::string &msg) {
    std::cerr << "[ERROR CALLBACK] Code: " << code << " | Message: " << msg
              << std::endl;
  });

  // Attempt connection
  std::cout << std::endl;
  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
               "━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << "Attempting connection..." << std::endl;
  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
               "━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << std::endl;

  auto connect_start = std::chrono::steady_clock::now();
  bool connected = client->connect();
  auto connect_duration = std::chrono::duration_cast<std::chrono::milliseconds>(
                              std::chrono::steady_clock::now() - connect_start)
                              .count();

  std::cout << std::endl;
  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
               "━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << "Connection Result" << std::endl;
  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
               "━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << std::endl;

  if (connected) {
    std::cout << "✓ SUCCESS: Connected to TWS in " << connect_duration << "ms"
              << std::endl;
    std::cout << std::endl;

    // Get connection state
    auto state = client->get_connection_state();
    std::cout << "Connection State: ";
    switch (state) {
    case ConnectionState::Connected:
      std::cout << "Connected" << std::endl;
      break;
    case ConnectionState::Connecting:
      std::cout << "Connecting" << std::endl;
      break;
    case ConnectionState::Disconnected:
      std::cout << "Disconnected" << std::endl;
      break;
    case ConnectionState::Error:
      std::cout << "Error" << std::endl;
      break;
    }

    // Get next order ID
    int next_id = client->get_next_order_id();
    std::cout << "Next Valid Order ID: " << next_id << std::endl;
    std::cout << std::endl;

    // Keep connection alive for a few seconds to verify it's stable
    std::cout
        << "Connection is stable. Keeping connection alive for 5 seconds..."
        << std::endl;
    for (int i = 0; i < 5; ++i) {
      std::this_thread::sleep_for(std::chrono::seconds(1));
      client->process_messages(100);

      if (!client->is_connected()) {
        std::cout << "⚠ Connection lost during stability test!" << std::endl;
        break;
      }
      std::cout << "  " << (5 - i) << " seconds remaining..." << std::endl;
    }

    if (client->is_connected()) {
      std::cout << "✓ Connection remained stable for 5 seconds" << std::endl;
    }

    // Disconnect
    std::cout << std::endl;
    std::cout << "Disconnecting..." << std::endl;
    client->disconnect();
    std::cout << "✓ Disconnected successfully" << std::endl;

    return 0;
  } else {
    std::cerr << "✗ FAILED: Could not connect to TWS after " << connect_duration
              << "ms" << std::endl;
    std::cerr << std::endl;
    std::cerr << "Troubleshooting Steps:" << std::endl;
    std::cerr << "  1. Verify TWS/IB Gateway is running" << std::endl;
    std::cerr << "     → Check if process is running: ps aux | grep -i tws"
              << std::endl;
    std::cerr << "     → Check if port is listening: lsof -i :" << port
              << std::endl;
    std::cerr << std::endl;
    std::cerr << "  2. Verify API is enabled in TWS/Gateway" << std::endl;
    std::cerr << "     → Go to: File → Global Configuration → API → Settings"
              << std::endl;
    std::cerr << "     → Enable: 'Enable ActiveX and Socket Clients'"
              << std::endl;
    std::cerr << "     → Verify port matches: " << port << std::endl;
    std::cerr << std::endl;
    std::cerr << "  3. Check TWS/Gateway window for connection prompts"
              << std::endl;
    std::cerr
        << "     → Look for 'A new API client is attempting to connect' message"
        << std::endl;
    std::cerr << "     → Click 'Accept' or 'OK' to allow the connection"
              << std::endl;
    std::cerr << std::endl;
    std::cerr << "  4. Verify client ID is unique" << std::endl;
    std::cerr << "     → Current client ID: " << client_id << std::endl;
    std::cerr
        << "     → Try a different client ID if another application is using "
        << client_id << std::endl;
    std::cerr << std::endl;
    std::cerr << "  5. Check firewall settings" << std::endl;
    std::cerr << "     → Ensure localhost connections are allowed" << std::endl;
    std::cerr << "     → Test: telnet 127.0.0.1 " << port << std::endl;
    std::cerr << std::endl;

    return 1;
  }
}
