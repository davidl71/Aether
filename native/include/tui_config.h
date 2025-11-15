// tui_config.h - TUI configuration structure
#pragma once

#include <string>
#include <chrono>
#include <vector>

namespace tui {

// TUI configuration for datafeeds and display
struct TUIConfig {
  // Data provider settings
  std::string provider_type = "mock";  // "mock", "rest", "nautilus", "ibkr_rest", "livevol"
  std::string rest_endpoint = "http://localhost:8080/api/snapshot";
  std::chrono::milliseconds update_interval{1000};  // Update frequency

  // Display settings
  bool show_colors = true;
  bool show_footer = true;
  int refresh_rate_ms = 500;  // UI refresh rate

  // Mock provider settings
  std::vector<std::string> mock_symbols = {"SPX", "ES50", "NANOS", "XSP"};

  // REST provider settings
  int rest_timeout_ms = 10000;  // HTTP timeout
  bool rest_verify_ssl = false;  // SSL verification (for testing)

  // Nautilus provider settings
  std::string nautilus_endpoint = "localhost:8000";

  // IBKR Client Portal REST API Settings
  std::string ibkr_rest_base_url = "https://localhost:5000/v1/portal";
  bool ibkr_rest_verify_ssl = false;  // Client Portal uses self-signed cert by default
  int ibkr_rest_timeout_ms = 10000;
  std::string ibkr_rest_account_id = "";  // Empty = auto-detect first account

  // LiveVol API Settings (Cboe LiveVol - Options Market Data)
  std::string livevol_base_url = "https://api.livevol.com/v1";
  std::string livevol_client_id = "";  // OAuth 2.0 client ID
  std::string livevol_client_secret = "";  // OAuth 2.0 client secret
  std::string livevol_access_token = "";  // Cached access token
  int livevol_timeout_ms = 10000;
  bool livevol_use_real_time = true;  // true = real-time, false = delayed

  // IBKR/TWS Connection Settings
  std::string ibkr_host = "127.0.0.1";
  int ibkr_port = 7497;  // 7497 = paper, 7496 = live
  int ibkr_client_id = 1;
  bool ibkr_paper_trading = true;  // true = paper (7497), false = live (7496)
  std::string ibkr_account_id = "";  // Account ID (empty = use first managed account)
  int ibkr_connection_timeout_ms = 60000;
  bool ibkr_auto_reconnect = true;
  int ibkr_max_reconnect_attempts = 10;
  bool ibkr_use_mock = false;  // Use mock TWS client

  // TA-Lib Settings
  bool use_ta_lib = false;  // Enable TA-Lib technical indicators
  std::vector<std::string> ta_lib_indicators = {"RSI", "MACD", "BBANDS"};  // Enabled indicators

  // Save/load helpers
  void SaveToFile(const std::string& path) const;
  void LoadFromFile(const std::string& path);
  static TUIConfig LoadDefault();
};

} // namespace tui
