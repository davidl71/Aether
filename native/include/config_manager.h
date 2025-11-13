// config_manager.h - Configuration management for IBKR Box Spread Generator
#pragma once

#include <string>
#include <vector>
#include <memory>
#include <nlohmann/json.hpp>

namespace config {

// ============================================================================
// TWS Connection Configuration
// ============================================================================

struct TWSConfig {
    std::string host = "127.0.0.1";
    int port = 7497;              // 7497 for paper, 7496 for live
    int client_id = 1;
    int connection_timeout_ms = 60000;  // 60 seconds - timeout for waiting for nextValidId after socket connection
    bool auto_reconnect = true;
    int reconnect_delay_ms = 3000;  // Deprecated: use exponential backoff instead
    int max_reconnect_attempts = 10; // Maximum reconnection attempts (0 = unlimited)
};

// ============================================================================
// Strategy Parameters
// ============================================================================

struct StrategyParams {
    std::vector<std::string> symbols;   // Underlying symbols to trade
    double min_arbitrage_profit = 0.10; // Minimum profit in dollars
    double min_roi_percent = 0.5;       // Minimum ROI percentage
    double max_position_size = 10000.0; // Maximum capital per trade
    int min_days_to_expiry = 30;        // Minimum DTE
    int max_days_to_expiry = 90;        // Maximum DTE
    double max_bid_ask_spread = 0.10;   // Maximum acceptable bid-ask spread
    int min_volume = 100;               // Minimum daily volume
    int min_open_interest = 500;        // Minimum open interest
};

// ============================================================================
// Risk Management Configuration
// ============================================================================

struct RiskConfig {
    double max_total_exposure = 50000.0;        // Maximum total capital deployed
    int max_positions = 10;                     // Maximum number of positions
    double max_loss_per_position = 1000.0;      // Maximum loss per position
    double max_daily_loss = 2000.0;             // Maximum daily loss
    double position_size_percent = 0.1;         // Position size as % of account
    bool enable_stop_loss = true;
    double stop_loss_percent = 0.2;             // Stop loss at 20% of investment
};

// ============================================================================
// Logging Configuration
// ============================================================================

struct LogConfig {
    std::string log_file = "logs/ib_box_spread.log";
    std::string log_level = "info";             // trace, debug, info, warn, error
    int max_file_size_mb = 10;
    int max_files = 5;
    bool log_to_console = true;
    bool use_colors = true;
};

// ============================================================================
// Main Configuration Structure
// ============================================================================

struct Config {
    TWSConfig tws;
    StrategyParams strategy;
    RiskConfig risk;
    LogConfig logging;

    // Runtime options
    bool dry_run = false;                       // Simulate without real trades
    int loop_delay_ms = 1000;                   // Main loop delay
    bool continue_on_error = false;             // Continue trading on errors
};

// ============================================================================
// Configuration Manager Class
// ============================================================================

class ConfigManager {
public:
    // Load configuration from JSON file
    static Config load(const std::string& config_file);

    // Save configuration to JSON file
    static void save(const Config& config, const std::string& config_file);

    // Validate configuration
    static void validate(const Config& config);

    // Load from JSON object
    static Config from_json(const nlohmann::json& j);

    // Convert to JSON object
    static nlohmann::json to_json(const Config& config);

    // Get default configuration
    static Config get_default();

    // Validation methods (public for testing)
    static void validate_tws_config(const TWSConfig& tws);
    static void validate_strategy_params(const StrategyParams& strategy);
    static void validate_risk_config(const RiskConfig& risk);
    static void validate_log_config(const LogConfig& logging);
};

// ============================================================================
// JSON Serialization Functions
// ============================================================================

// TWS Config
inline void to_json(nlohmann::json& j, const TWSConfig& config) {
    j = nlohmann::json{
        {"host", config.host},
        {"port", config.port},
        {"client_id", config.client_id},
        {"connection_timeout_ms", config.connection_timeout_ms},
        {"auto_reconnect", config.auto_reconnect},
        {"reconnect_delay_ms", config.reconnect_delay_ms},
        {"max_reconnect_attempts", config.max_reconnect_attempts}
    };
}

inline void from_json(const nlohmann::json& j, TWSConfig& config) {
    j.at("host").get_to(config.host);
    j.at("port").get_to(config.port);
    j.at("client_id").get_to(config.client_id);
    if (j.contains("connection_timeout_ms"))
        j.at("connection_timeout_ms").get_to(config.connection_timeout_ms);
    if (j.contains("auto_reconnect"))
        j.at("auto_reconnect").get_to(config.auto_reconnect);
    if (j.contains("reconnect_delay_ms"))
        j.at("reconnect_delay_ms").get_to(config.reconnect_delay_ms);
    if (j.contains("max_reconnect_attempts"))
        j.at("max_reconnect_attempts").get_to(config.max_reconnect_attempts);
}

// Strategy Params
inline void to_json(nlohmann::json& j, const StrategyParams& params) {
    j = nlohmann::json{
        {"symbols", params.symbols},
        {"min_arbitrage_profit", params.min_arbitrage_profit},
        {"min_roi_percent", params.min_roi_percent},
        {"max_position_size", params.max_position_size},
        {"min_days_to_expiry", params.min_days_to_expiry},
        {"max_days_to_expiry", params.max_days_to_expiry},
        {"max_bid_ask_spread", params.max_bid_ask_spread},
        {"min_volume", params.min_volume},
        {"min_open_interest", params.min_open_interest}
    };
}

inline void from_json(const nlohmann::json& j, StrategyParams& params) {
    j.at("symbols").get_to(params.symbols);
    if (j.contains("min_arbitrage_profit"))
        j.at("min_arbitrage_profit").get_to(params.min_arbitrage_profit);
    if (j.contains("min_roi_percent"))
        j.at("min_roi_percent").get_to(params.min_roi_percent);
    if (j.contains("max_position_size"))
        j.at("max_position_size").get_to(params.max_position_size);
    if (j.contains("min_days_to_expiry"))
        j.at("min_days_to_expiry").get_to(params.min_days_to_expiry);
    if (j.contains("max_days_to_expiry"))
        j.at("max_days_to_expiry").get_to(params.max_days_to_expiry);
    if (j.contains("max_bid_ask_spread"))
        j.at("max_bid_ask_spread").get_to(params.max_bid_ask_spread);
    if (j.contains("min_volume"))
        j.at("min_volume").get_to(params.min_volume);
    if (j.contains("min_open_interest"))
        j.at("min_open_interest").get_to(params.min_open_interest);
}

// Risk Config
inline void to_json(nlohmann::json& j, const RiskConfig& config) {
    j = nlohmann::json{
        {"max_total_exposure", config.max_total_exposure},
        {"max_positions", config.max_positions},
        {"max_loss_per_position", config.max_loss_per_position},
        {"max_daily_loss", config.max_daily_loss},
        {"position_size_percent", config.position_size_percent},
        {"enable_stop_loss", config.enable_stop_loss},
        {"stop_loss_percent", config.stop_loss_percent}
    };
}

inline void from_json(const nlohmann::json& j, RiskConfig& config) {
    if (j.contains("max_total_exposure"))
        j.at("max_total_exposure").get_to(config.max_total_exposure);
    if (j.contains("max_positions"))
        j.at("max_positions").get_to(config.max_positions);
    if (j.contains("max_loss_per_position"))
        j.at("max_loss_per_position").get_to(config.max_loss_per_position);
    if (j.contains("max_daily_loss"))
        j.at("max_daily_loss").get_to(config.max_daily_loss);
    if (j.contains("position_size_percent"))
        j.at("position_size_percent").get_to(config.position_size_percent);
    if (j.contains("enable_stop_loss"))
        j.at("enable_stop_loss").get_to(config.enable_stop_loss);
    if (j.contains("stop_loss_percent"))
        j.at("stop_loss_percent").get_to(config.stop_loss_percent);
}

// Log Config
inline void to_json(nlohmann::json& j, const LogConfig& config) {
    j = nlohmann::json{
        {"log_file", config.log_file},
        {"log_level", config.log_level},
        {"max_file_size_mb", config.max_file_size_mb},
        {"max_files", config.max_files},
        {"log_to_console", config.log_to_console},
        {"use_colors", config.use_colors}
    };
}

inline void from_json(const nlohmann::json& j, LogConfig& config) {
    if (j.contains("log_file"))
        j.at("log_file").get_to(config.log_file);
    if (j.contains("log_level"))
        j.at("log_level").get_to(config.log_level);
    if (j.contains("max_file_size_mb"))
        j.at("max_file_size_mb").get_to(config.max_file_size_mb);
    if (j.contains("max_files"))
        j.at("max_files").get_to(config.max_files);
    if (j.contains("log_to_console"))
        j.at("log_to_console").get_to(config.log_to_console);
    if (j.contains("use_colors"))
        j.at("use_colors").get_to(config.use_colors);
}

// Main Config
inline void to_json(nlohmann::json& j, const Config& config) {
    j = nlohmann::json{
        {"tws", config.tws},
        {"strategy", config.strategy},
        {"risk", config.risk},
        {"logging", config.logging},
        {"dry_run", config.dry_run},
        {"loop_delay_ms", config.loop_delay_ms},
        {"continue_on_error", config.continue_on_error}
    };
}

inline void from_json(const nlohmann::json& j, Config& config) {
    j.at("tws").get_to(config.tws);
    j.at("strategy").get_to(config.strategy);
    if (j.contains("risk"))
        j.at("risk").get_to(config.risk);
    if (j.contains("logging"))
        j.at("logging").get_to(config.logging);
    if (j.contains("dry_run"))
        j.at("dry_run").get_to(config.dry_run);
    if (j.contains("loop_delay_ms"))
        j.at("loop_delay_ms").get_to(config.loop_delay_ms);
    if (j.contains("continue_on_error"))
        j.at("continue_on_error").get_to(config.continue_on_error);
}

} // namespace config
