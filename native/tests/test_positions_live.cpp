// test_positions_live.cpp - Test real position retrieval from IBKR
// This is a utility to verify we can pull positions from a live IBKR connection
// Run with: ./build/bin/test_positions_live

#include "tws_client.h"
#include "config_manager.h"
#include <spdlog/spdlog.h>
#include <iostream>
#include <thread>
#include <chrono>

int main(int argc, char** argv) {
  // Set up logging
  spdlog::set_level(spdlog::level::debug);
  spdlog::set_pattern("[%Y-%m-%d %H:%M:%S.%e] [%^%l%$] %v");

  std::cout << "=== IBKR Position Retrieval Test ===" << std::endl;
  std::cout << std::endl;

  // Load configuration
  config::Config cfg;
  try {
    cfg = config::ConfigManager::load("config/tws_config.json");
  } catch (const std::exception& e) {
    // Use default config if file doesn't exist
    spdlog::warn("Could not load config file, using defaults: {}", e.what());
    cfg = config::ConfigManager::get_default();
  }

  auto tws_config = cfg.tws;
  
  // Override with paper trading port if not already set
  if (tws_config.port != 7497) {
    std::cout << "⚠️  Current config uses port " << tws_config.port << std::endl;
    std::cout << "⚠️  Switching to paper trading port 7497 for safety" << std::endl;
    tws_config.port = 7497;
  }

  std::cout << "Configuration:" << std::endl;
  std::cout << "  Host: " << tws_config.host << std::endl;
  std::cout << "  Port: " << tws_config.port << " (paper trading)" << std::endl;
  std::cout << "  Client ID: " << tws_config.client_id << std::endl;
  std::cout << std::endl;

  // Create TWS client
  tws::TWSClient client(tws_config);

  // Connect to TWS/Gateway
  std::cout << "Connecting to IBKR TWS/Gateway..." << std::endl;
  if (!client.connect()) {
    spdlog::error("Failed to connect to TWS. Is TWS/Gateway running on port {}?", 
                  tws_config.port);
    std::cout << std::endl;
    std::cout << "💡 Troubleshooting:" << std::endl;
    std::cout << "   1. Ensure TWS or IB Gateway is running" << std::endl;
    std::cout << "   2. Verify 'Enable API' is checked in TWS Global Configuration" << std::endl;
    std::cout << "   3. Check the API port matches (paper=7497, live=7496)" << std::endl;
    std::cout << "   4. Ensure client ID " << tws_config.client_id 
              << " is not already in use" << std::endl;
    return 1;
  }

  std::cout << "✓ Connected successfully" << std::endl;
  std::cout << std::endl;

  // Give connection time to stabilize
  std::cout << "Waiting for connection to stabilize..." << std::endl;
  for (int i = 0; i < 20; ++i) {
    client.process_messages(100);
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
  }

  // Request positions (synchronous with 10 second timeout)
  std::cout << "Requesting current positions..." << std::endl;
  auto positions = client.request_positions_sync(10000);

  std::cout << std::endl;
  std::cout << "=== Position Summary ===" << std::endl;
  std::cout << "Total positions: " << positions.size() << std::endl;
  std::cout << std::endl;

  if (positions.empty()) {
    std::cout << "No positions found." << std::endl;
    std::cout << std::endl;
    std::cout << "💡 This is expected if your paper account has no positions." << std::endl;
    std::cout << "   To test with positions, place some test trades in TWS first." << std::endl;
  } else {
    std::cout << "Positions:" << std::endl;
    std::cout << std::string(80, '-') << std::endl;
    
    for (const auto& pos : positions) {
      std::cout << "Symbol:   " << pos.contract.symbol << std::endl;
      std::cout << "Type:     " << (pos.contract.type == types::OptionType::Call ? "CALL" : "PUT") 
                << std::endl;
      std::cout << "Strike:   " << pos.contract.strike << std::endl;
      std::cout << "Expiry:   " << pos.contract.expiry << std::endl;
      std::cout << "Quantity: " << pos.quantity << std::endl;
      std::cout << "Avg Cost: $" << pos.avg_price << std::endl;
      std::cout << "Current:  $" << pos.current_price << std::endl;
      
      double market_value = pos.quantity * pos.current_price * 100.0;  // Options are per 100 shares
      double cost_basis = pos.quantity * pos.avg_price * 100.0;
      double unrealized_pnl = market_value - cost_basis;
      
      std::cout << "P&L:      $" << unrealized_pnl 
                << " (" << (unrealized_pnl >= 0 ? "+" : "") 
                << (cost_basis != 0 ? (unrealized_pnl / cost_basis * 100.0) : 0.0) 
                << "%)" << std::endl;
      std::cout << std::string(80, '-') << std::endl;
    }
  }

  // Also test account info retrieval
  std::cout << std::endl;
  std::cout << "=== Account Information ===" << std::endl;
  auto account_info = client.request_account_info_sync(10000);
  
  if (account_info) {
    std::cout << "Account ID:         " << account_info->account_id << std::endl;
    std::cout << "Net Liquidation:    $" << account_info->net_liquidation << std::endl;
    std::cout << "Cash Balance:       $" << account_info->cash_balance << std::endl;
    std::cout << "Buying Power:       $" << account_info->buying_power << std::endl;
    std::cout << "Gross Position Val: $" << account_info->gross_position_value << std::endl;
    std::cout << "Unrealized P&L:     $" << account_info->unrealized_pnl << std::endl;
    std::cout << "Realized P&L:       $" << account_info->realized_pnl << std::endl;
  } else {
    std::cout << "Failed to retrieve account information" << std::endl;
  }

  std::cout << std::endl;
  std::cout << "Disconnecting..." << std::endl;
  client.disconnect();

  std::cout << "✓ Test complete" << std::endl;
  return 0;
}
