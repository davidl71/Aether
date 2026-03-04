// connection_utils.h - TWS connection utilities (port checking, mock detection)
#pragma once

#include "types.h"
#include "config_manager.h"
#include <string>
#include <vector>
#include <unordered_map>
#include <utility>

namespace tws {

bool is_port_open(const std::string& host, int port, int timeout_ms = 1000);

std::vector<int> get_port_candidates(int configured_port);

bool env_flag_enabled(const char* name);

bool should_use_mock_client(const config::TWSConfig& config);

types::OptionContract make_mock_contract(const std::string& symbol,
                                         const std::string& expiry,
                                         double strike,
                                         types::OptionType type);

types::MarketData generate_mock_market_data(const types::OptionContract& contract);

} // namespace tws
