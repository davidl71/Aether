// config_manager.cpp - Configuration management implementation
#include "config_manager.h"
#include <fstream>
#include <stdexcept>
#include <algorithm>
#include <regex>

// NOTE FOR AUTOMATION AGENTS:
// `ConfigManager` is the single authority for reading, validating, and writing
// runtime configuration. It performs schema validation before returning data to
// dependants so downstream modules can assume invariants (non-empty symbols list,
// bounded ROI thresholds, etc.). Extend validation here rather than scattering
// defensive checks across consumers.

namespace config {

// ============================================================================
// ConfigManager Implementation
// ============================================================================

Config ConfigManager::load(const std::string& config_file) {
    std::ifstream file(config_file);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open config file: " + config_file);
    }

    nlohmann::json j;
    try {
        file >> j;
    } catch (const nlohmann::json::exception& e) {
        throw std::runtime_error("Failed to parse JSON: " + std::string(e.what()));
    }

    Config config = from_json(j);
    validate(config);  // Enforce invariants immediately after deserialization

    return config;
}

void ConfigManager::save(const Config& config, const std::string& config_file) {
    validate(config);

    nlohmann::json j = to_json(config);

    std::ofstream file(config_file);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open config file for writing: " + config_file);
    }

    file << j.dump(4) << std::endl;
}

void ConfigManager::validate(const Config& config) {
    validate_tws_config(config.tws);
    validate_strategy_params(config.strategy);
    validate_risk_config(config.risk);
    validate_log_config(config.logging);
}

Config ConfigManager::from_json(const nlohmann::json& j) {
    Config config;
    j.get_to(config);
    return config;
}

nlohmann::json ConfigManager::to_json(const Config& config) {
    return nlohmann::json(config);
}

Config ConfigManager::get_default() {
    Config config;
    // Default values are already set in struct definitions
    return config;
}

void ConfigManager::validate_tws_config(const TWSConfig& tws) {
    if (tws.host.empty()) {
        throw std::invalid_argument("TWS host cannot be empty");
    }

    if (tws.port < 1024 || tws.port > 65535) {
        throw std::invalid_argument("TWS port must be between 1024 and 65535");
    }

    if (tws.client_id < 0) {
        throw std::invalid_argument("TWS client_id must be non-negative");
    }

    if (tws.connection_timeout_ms <= 0) {
        throw std::invalid_argument("Connection timeout must be positive");
    }
}

void ConfigManager::validate_strategy_params(const StrategyParams& strategy) {
    if (strategy.symbols.empty()) {
        throw std::invalid_argument("At least one symbol must be specified");
    }

    // Validate symbols format
    std::regex symbol_regex("^[A-Z]{1,5}$");
    for (const auto& symbol : strategy.symbols) {
        if (!std::regex_match(symbol, symbol_regex)) {
            throw std::invalid_argument("Invalid symbol format: " + symbol);
        }
    }

    if (strategy.min_arbitrage_profit < 0) {
        throw std::invalid_argument("Minimum arbitrage profit must be non-negative");
    }

    if (strategy.min_roi_percent < 0 || strategy.min_roi_percent > 100) {
        throw std::invalid_argument("ROI percent must be between 0 and 100");
    }

    if (strategy.max_position_size <= 0) {
        throw std::invalid_argument("Maximum position size must be positive");
    }

    if (strategy.min_days_to_expiry < 0) {
        throw std::invalid_argument("Minimum days to expiry must be non-negative");
    }

    if (strategy.max_days_to_expiry < strategy.min_days_to_expiry) {
        throw std::invalid_argument("Maximum DTE must be >= minimum DTE");
    }

    if (strategy.max_bid_ask_spread < 0) {
        throw std::invalid_argument("Maximum bid-ask spread must be non-negative");
    }

    if (strategy.min_volume < 0) {
        throw std::invalid_argument("Minimum volume must be non-negative");
    }

    if (strategy.min_open_interest < 0) {
        throw std::invalid_argument("Minimum open interest must be non-negative");
    }
}

void ConfigManager::validate_risk_config(const RiskConfig& risk) {
    if (risk.max_total_exposure <= 0) {
        throw std::invalid_argument("Maximum total exposure must be positive");
    }

    if (risk.max_positions <= 0) {
        throw std::invalid_argument("Maximum positions must be positive");
    }

    if (risk.max_loss_per_position < 0) {
        throw std::invalid_argument("Maximum loss per position must be non-negative");
    }

    if (risk.max_daily_loss < 0) {
        throw std::invalid_argument("Maximum daily loss must be non-negative");
    }

    if (risk.position_size_percent <= 0 || risk.position_size_percent > 1) {
        throw std::invalid_argument("Position size percent must be between 0 and 1");
    }

    if (risk.stop_loss_percent < 0 || risk.stop_loss_percent > 1) {
        throw std::invalid_argument("Stop loss percent must be between 0 and 1");
    }
}

void ConfigManager::validate_log_config(const LogConfig& logging) {
    if (logging.log_file.empty()) {
        throw std::invalid_argument("Log file path cannot be empty");
    }

    static const std::vector<std::string> valid_levels = {
        "trace", "debug", "info", "warn", "error"
    };

    if (std::find(valid_levels.begin(), valid_levels.end(), logging.log_level)
        == valid_levels.end()) {
        throw std::invalid_argument("Invalid log level: " + logging.log_level);
    }

    if (logging.max_file_size_mb <= 0) {
        throw std::invalid_argument("Maximum file size must be positive");
    }

    if (logging.max_files <= 0) {
        throw std::invalid_argument("Maximum files must be positive");
    }
}

} // namespace config
