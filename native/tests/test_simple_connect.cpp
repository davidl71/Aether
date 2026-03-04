// test_simple_connect.cpp - Minimal connection test with detailed diagnostics
// This version tries different client IDs and reports exactly what's happening

#include "tws_client.h"
#include <chrono>
#include <iostream>
#include <spdlog/spdlog.h>
#include <thread>

using namespace tws;

int main(int argc, char *argv[]) {
  spdlog::set_level(spdlog::level::info); // Less verbose
  spdlog::set_pattern("[%H:%M:%S] [%^%l%$] %v");

  std::string host = "127.0.0.1";
  int port = 4001; // Live Gateway

  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << "  IBKR Simple Connection Test" << std::endl;
  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << std::endl;

  // Try a few different client IDs
  std::vector<int> client_ids_to_try = {100, 200, 300, 999, 1234};

  if (argc >= 2) {
    client_ids_to_try = {std::stoi(argv[1])};
  }

  for (int client_id : client_ids_to_try) {
    std::cout << "Trying client ID: " << client_id << std::endl;

    config::TWSConfig cfg;
    cfg.host = host;
    cfg.port = port;
    cfg.client_id = client_id;

    TWSClient client(cfg);

    if (client.connect()) {
      std::cout << "✓ Connected successfully with client ID " << client_id
                << std::endl;
      std::cout << std::endl;

      // Wait for full handshake
      std::cout << "Waiting for account data..." << std::endl;
      for (int i = 0; i < 30; ++i) {
        client.process_messages(100);
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
      }

      // Try to get positions
      std::cout << "Requesting positions..." << std::endl;
      auto positions = client.request_positions_sync(5000);

      std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                << std::endl;
      std::cout << "Result: Found " << positions.size() << " positions"
                << std::endl;
      std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                << std::endl;
      std::cout << std::endl;

      if (!positions.empty()) {
        std::cout << "Sample positions:" << std::endl;
        for (size_t i = 0; i < std::min(size_t(3), positions.size()); ++i) {
          const auto &pos = positions[i];
          std::cout << "  " << pos.contract.symbol << " " << pos.contract.expiry
                    << " " << pos.contract.strike << " x " << pos.quantity
                    << std::endl;
        }
      }

      client.disconnect();
      std::cout << std::endl;
      std::cout << "✓ Test successful!" << std::endl;
      return 0;

    } else {
      std::cout << "✗ Failed with client ID " << client_id << std::endl;
      std::cout << std::endl;
    }

    // Wait before next attempt
    std::this_thread::sleep_for(std::chrono::milliseconds(500));
  }

  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << "All client IDs failed. Possible issues:" << std::endl;
  std::cout << "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            << std::endl;
  std::cout << std::endl;
  std::cout << "1. IB Gateway API not enabled" << std::endl;
  std::cout << "   → Configure → Settings → API → Settings" << std::endl;
  std::cout << "   → Check 'Enable ActiveX and Socket Clients'" << std::endl;
  std::cout << std::endl;
  std::cout << "2. Firewall blocking connection" << std::endl;
  std::cout << "   → Check localhost connections are allowed" << std::endl;
  std::cout << std::endl;
  std::cout << "3. IB Gateway requires connection approval" << std::endl;
  std::cout << "   → Look for popup dialog in IB Gateway" << std::endl;
  std::cout
      << "   → Or enable 'Accept incoming connection requests automatically'"
      << std::endl;
  std::cout << "   → In: Configure → Settings → API → Settings" << std::endl;
  std::cout << std::endl;
  std::cout << "4. Client ID restrictions" << std::endl;
  std::cout << "   → Check 'Master Client ID' setting" << std::endl;
  std::cout << std::endl;

  return 1;
}
