// tui_config.cpp - TUI configuration implementation
#include "tui_config.h"
#include <fstream>
#include <nlohmann/json.hpp>
#include <spdlog/spdlog.h>

namespace tui {

void TUIConfig::SaveToFile(const std::string& path) const {
  nlohmann::json j;
  j["provider_type"] = provider_type;
  j["rest_endpoint"] = rest_endpoint;
  j["update_interval_ms"] = update_interval.count();
  j["show_colors"] = show_colors;
  j["show_footer"] = show_footer;
  j["refresh_rate_ms"] = refresh_rate_ms;
  j["mock_symbols"] = mock_symbols;
  j["rest_timeout_ms"] = rest_timeout_ms;
  j["rest_verify_ssl"] = rest_verify_ssl;
  j["nautilus_endpoint"] = nautilus_endpoint;

  // IBKR Client Portal REST API settings
  j["ibkr_rest_base_url"] = ibkr_rest_base_url;
  j["ibkr_rest_verify_ssl"] = ibkr_rest_verify_ssl;
  j["ibkr_rest_timeout_ms"] = ibkr_rest_timeout_ms;
  j["ibkr_rest_account_id"] = ibkr_rest_account_id;

  // LiveVol API settings
  j["livevol_base_url"] = livevol_base_url;
  j["livevol_client_id"] = livevol_client_id;
  // Note: client_secret and access_token are sensitive, but stored for convenience
  // In production, consider using secure storage
  j["livevol_client_secret"] = livevol_client_secret;
  j["livevol_access_token"] = livevol_access_token;
  j["livevol_timeout_ms"] = livevol_timeout_ms;
  j["livevol_use_real_time"] = livevol_use_real_time;

  // IBKR/TWS settings
  j["ibkr_host"] = ibkr_host;
  j["ibkr_port"] = ibkr_port;
  j["ibkr_client_id"] = ibkr_client_id;
  j["ibkr_paper_trading"] = ibkr_paper_trading;
  j["ibkr_account_id"] = ibkr_account_id;
  j["ibkr_connection_timeout_ms"] = ibkr_connection_timeout_ms;
  j["ibkr_auto_reconnect"] = ibkr_auto_reconnect;
  j["ibkr_max_reconnect_attempts"] = ibkr_max_reconnect_attempts;
  j["ibkr_use_mock"] = ibkr_use_mock;

  // TA-Lib settings
  j["use_ta_lib"] = use_ta_lib;
  j["ta_lib_indicators"] = ta_lib_indicators;

  std::ofstream file(path);
  if (!file.is_open()) {
    throw std::runtime_error("Failed to open config file for writing: " + path);
  }
  file << j.dump(4) << std::endl;
}

void TUIConfig::LoadFromFile(const std::string& path) {
  std::ifstream file(path);
  if (!file.is_open()) {
    spdlog::warn("Config file not found: {}, using defaults", path);
    *this = LoadDefault();
    return;
  }

  nlohmann::json j;
  try {
    file >> j;
  } catch (const nlohmann::json::exception& e) {
    spdlog::error("Failed to parse config JSON: {}", e.what());
    *this = LoadDefault();
    return;
  }

  if (j.contains("provider_type")) provider_type = j["provider_type"];
  if (j.contains("rest_endpoint")) rest_endpoint = j["rest_endpoint"];
  if (j.contains("update_interval_ms")) {
    update_interval = std::chrono::milliseconds(j["update_interval_ms"]);
  }
  if (j.contains("show_colors")) show_colors = j["show_colors"];
  if (j.contains("show_footer")) show_footer = j["show_footer"];
  if (j.contains("refresh_rate_ms")) refresh_rate_ms = j["refresh_rate_ms"];
  if (j.contains("mock_symbols")) mock_symbols = j["mock_symbols"].get<std::vector<std::string>>();
  if (j.contains("rest_timeout_ms")) rest_timeout_ms = j["rest_timeout_ms"];
  if (j.contains("rest_verify_ssl")) rest_verify_ssl = j["rest_verify_ssl"];
  if (j.contains("nautilus_endpoint")) nautilus_endpoint = j["nautilus_endpoint"];

  // IBKR Client Portal REST API settings
  if (j.contains("ibkr_rest_base_url")) ibkr_rest_base_url = j["ibkr_rest_base_url"];
  if (j.contains("ibkr_rest_verify_ssl")) ibkr_rest_verify_ssl = j["ibkr_rest_verify_ssl"];
  if (j.contains("ibkr_rest_timeout_ms")) ibkr_rest_timeout_ms = j["ibkr_rest_timeout_ms"];
  if (j.contains("ibkr_rest_account_id")) ibkr_rest_account_id = j["ibkr_rest_account_id"];

  // LiveVol API settings
  if (j.contains("livevol_base_url")) livevol_base_url = j["livevol_base_url"];
  if (j.contains("livevol_client_id")) livevol_client_id = j["livevol_client_id"];
  if (j.contains("livevol_client_secret")) livevol_client_secret = j["livevol_client_secret"];
  if (j.contains("livevol_access_token")) livevol_access_token = j["livevol_access_token"];
  if (j.contains("livevol_timeout_ms")) livevol_timeout_ms = j["livevol_timeout_ms"];
  if (j.contains("livevol_use_real_time")) livevol_use_real_time = j["livevol_use_real_time"];

  // IBKR/TWS settings
  if (j.contains("ibkr_host")) ibkr_host = j["ibkr_host"];
  if (j.contains("ibkr_port")) ibkr_port = j["ibkr_port"];
  if (j.contains("ibkr_client_id")) ibkr_client_id = j["ibkr_client_id"];
  if (j.contains("ibkr_paper_trading")) ibkr_paper_trading = j["ibkr_paper_trading"];
  if (j.contains("ibkr_account_id")) ibkr_account_id = j["ibkr_account_id"];
  if (j.contains("ibkr_connection_timeout_ms")) ibkr_connection_timeout_ms = j["ibkr_connection_timeout_ms"];
  if (j.contains("ibkr_auto_reconnect")) ibkr_auto_reconnect = j["ibkr_auto_reconnect"];
  if (j.contains("ibkr_max_reconnect_attempts")) ibkr_max_reconnect_attempts = j["ibkr_max_reconnect_attempts"];
  if (j.contains("ibkr_use_mock")) ibkr_use_mock = j["ibkr_use_mock"];

  // TA-Lib settings
  if (j.contains("use_ta_lib")) use_ta_lib = j["use_ta_lib"];
  if (j.contains("ta_lib_indicators")) ta_lib_indicators = j["ta_lib_indicators"].get<std::vector<std::string>>();
}

TUIConfig TUIConfig::LoadDefault() {
  return TUIConfig{};  // Uses default values from struct
}

} // namespace tui
