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
    bool log_raw_messages = false;   // Log raw API messages/data for debugging (verbose, use with trace log_level)
    bool use_mock = false;           // Use in-process mock TWS client (no network connection)

    // PCAP Capture Configuration
    bool enable_pcap_capture = false;  // Enable PCAP packet capture for TWS API traffic
    std::string pcap_output_file = ""; // Output file path (empty = auto-generate with timestamp)
    bool pcap_nanosecond_precision = false;  // Use nanosecond precision timestamps (default: microsecond)
};

// ============================================================================
// Broker Selection
// ============================================================================
struct BrokerConfig {
    // Priority-ordered list of brokers to attempt.
    // Recognized values (case-insensitive): "mock", "alpaca", "ib", "fibi", "meitav", "ibi", "discount"
    std::vector<std::string> priorities;
};

// ============================================================================
// Commission Configuration (IBKR Pro)
// ============================================================================

enum class CommissionTier {
    Standard,        // 0-10,000 contracts/month: $0.65/contract
    Tier1,           // 10,001-50,000: $0.60/contract
    Tier2,           // 50,001-100,000: $0.55/contract
    Tier3            // 100,001+: $0.50/contract
};

struct CommissionConfig {
    bool use_ibkr_pro_rates = true;    // Use IBKR Pro published rates
    double per_contract_fee = 0.65;    // Default per contract fee (IBKR Pro standard)
    double minimum_order_fee = 1.00;   // Minimum per order (waived if monthly > $30)
    double maximum_order_fee_pct = 1.0; // Maximum 1% of trade value (for orders < $1/share)

    // Volume-based tiers (monthly contract volume)
    int monthly_contract_volume = 0;   // Track monthly volume for tier calculation
    CommissionTier current_tier = CommissionTier::Standard;

    // Tier thresholds and rates
    static double get_tier_rate(CommissionTier tier);
    static CommissionTier get_tier_from_volume(int monthly_volume);
    double get_effective_rate() const;  // Get current effective rate based on tier
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
    CommissionConfig commissions;       // Commission configuration

    // Benchmark rate configuration for financing comparison
    double benchmark_rate_percent = 5.0;           // Default benchmark rate (%)
    std::string benchmark_source = "static";       // "static", "treasury_api", "sofr"
    std::string treasury_api_url;                  // Treasury API URL (empty = default)
    double min_spread_over_benchmark_bps = 50.0;   // Min spread to flag opportunity (bps)
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
    double risk_free_rate_override = 0.0;       // 0 = auto-detect from market data
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
// Massive.com Configuration
// ============================================================================

struct MassiveConfig {
    bool enabled = false;
    std::string api_key;
    std::string base_url = "https://api.massive.com";

    bool use_for_historical_data = true;
    bool use_for_realtime_quotes = true;
    bool use_for_dividend_data = true;
    bool use_for_fundamental_data = true;

    bool websocket_enabled = false;
    std::string websocket_url = "wss://api.massive.com/ws";

    double min_market_cap = 1e9;  // $1B minimum
    double max_pe_ratio = 50.0;
    bool avoid_penny_stocks = true;

    int dividend_blackout_days = 2;

    int cache_duration_seconds = 300;
    int rate_limit_per_second = 10;
};

// ============================================================================
// Main Configuration Structure
// ============================================================================

struct Config {
    TWSConfig tws;
    BrokerConfig broker;
    StrategyParams strategy;
    RiskConfig risk;
    LogConfig logging;
    MassiveConfig massive;

    // Runtime options
    bool dry_run = false;                       // Simulate without real trades
    int loop_delay_ms = 1000;                   // Main loop delay
    bool continue_on_error = false;             // Continue trading on errors
};

// ============================================================================
// Commission Config Helper Methods
// ============================================================================

inline double CommissionConfig::get_tier_rate(CommissionTier tier) {
    switch (tier) {
        case CommissionTier::Standard: return 0.65;
        case CommissionTier::Tier1: return 0.60;
        case CommissionTier::Tier2: return 0.55;
        case CommissionTier::Tier3: return 0.50;
        default: return 0.65;
    }
}

inline CommissionTier CommissionConfig::get_tier_from_volume(int monthly_volume) {
    if (monthly_volume <= 10000) return CommissionTier::Standard;
    if (monthly_volume <= 50000) return CommissionTier::Tier1;
    if (monthly_volume <= 100000) return CommissionTier::Tier2;
    return CommissionTier::Tier3;
}

inline double CommissionConfig::get_effective_rate() const {
    if (use_ibkr_pro_rates) {
        return get_tier_rate(current_tier);
    }
    return per_contract_fee;
}

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
    static void validate_massive_config(const MassiveConfig& massive);
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
        {"max_reconnect_attempts", config.max_reconnect_attempts},
        {"log_raw_messages", config.log_raw_messages},
        {"use_mock", config.use_mock},
        {"enable_pcap_capture", config.enable_pcap_capture},
        {"pcap_output_file", config.pcap_output_file},
        {"pcap_nanosecond_precision", config.pcap_nanosecond_precision}
    };
}

// Broker Config
inline void to_json(nlohmann::json& j, const BrokerConfig& broker) {
    j = nlohmann::json{
        {"priorities", broker.priorities}
    };
}

inline void from_json(const nlohmann::json& j, BrokerConfig& broker) {
    if (j.contains("priorities"))
        j.at("priorities").get_to(broker.priorities);
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
    if (j.contains("log_raw_messages"))
        j.at("log_raw_messages").get_to(config.log_raw_messages);
    if (j.contains("use_mock"))
        j.at("use_mock").get_to(config.use_mock);
    if (j.contains("enable_pcap_capture"))
        j.at("enable_pcap_capture").get_to(config.enable_pcap_capture);
    if (j.contains("pcap_output_file"))
        j.at("pcap_output_file").get_to(config.pcap_output_file);
    if (j.contains("pcap_nanosecond_precision"))
        j.at("pcap_nanosecond_precision").get_to(config.pcap_nanosecond_precision);
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
        {"min_open_interest", params.min_open_interest},
        {"benchmark_rate_percent", params.benchmark_rate_percent},
        {"benchmark_source", params.benchmark_source},
        {"treasury_api_url", params.treasury_api_url},
        {"min_spread_over_benchmark_bps", params.min_spread_over_benchmark_bps}
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
    if (j.contains("benchmark_rate_percent"))
        j.at("benchmark_rate_percent").get_to(params.benchmark_rate_percent);
    if (j.contains("benchmark_source"))
        j.at("benchmark_source").get_to(params.benchmark_source);
    if (j.contains("treasury_api_url"))
        j.at("treasury_api_url").get_to(params.treasury_api_url);
    if (j.contains("min_spread_over_benchmark_bps"))
        j.at("min_spread_over_benchmark_bps").get_to(params.min_spread_over_benchmark_bps);
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

// Massive Config
inline void to_json(nlohmann::json& j, const MassiveConfig& config) {
    j = nlohmann::json{
        {"enabled", config.enabled},
        {"api_key", config.api_key},
        {"base_url", config.base_url},
        {"use_for_historical_data", config.use_for_historical_data},
        {"use_for_realtime_quotes", config.use_for_realtime_quotes},
        {"use_for_dividend_data", config.use_for_dividend_data},
        {"use_for_fundamental_data", config.use_for_fundamental_data},
        {"websocket_enabled", config.websocket_enabled},
        {"websocket_url", config.websocket_url},
        {"min_market_cap", config.min_market_cap},
        {"max_pe_ratio", config.max_pe_ratio},
        {"avoid_penny_stocks", config.avoid_penny_stocks},
        {"dividend_blackout_days", config.dividend_blackout_days},
        {"cache_duration_seconds", config.cache_duration_seconds},
        {"rate_limit_per_second", config.rate_limit_per_second}
    };
}

inline void from_json(const nlohmann::json& j, MassiveConfig& config) {
    if (j.contains("enabled"))
        j.at("enabled").get_to(config.enabled);
    if (j.contains("api_key"))
        j.at("api_key").get_to(config.api_key);
    if (j.contains("base_url"))
        j.at("base_url").get_to(config.base_url);
    if (j.contains("use_for_historical_data"))
        j.at("use_for_historical_data").get_to(config.use_for_historical_data);
    if (j.contains("use_for_realtime_quotes"))
        j.at("use_for_realtime_quotes").get_to(config.use_for_realtime_quotes);
    if (j.contains("use_for_dividend_data"))
        j.at("use_for_dividend_data").get_to(config.use_for_dividend_data);
    if (j.contains("use_for_fundamental_data"))
        j.at("use_for_fundamental_data").get_to(config.use_for_fundamental_data);
    if (j.contains("websocket_enabled"))
        j.at("websocket_enabled").get_to(config.websocket_enabled);
    if (j.contains("websocket_url"))
        j.at("websocket_url").get_to(config.websocket_url);
    if (j.contains("min_market_cap"))
        j.at("min_market_cap").get_to(config.min_market_cap);
    if (j.contains("max_pe_ratio"))
        j.at("max_pe_ratio").get_to(config.max_pe_ratio);
    if (j.contains("avoid_penny_stocks"))
        j.at("avoid_penny_stocks").get_to(config.avoid_penny_stocks);
    if (j.contains("dividend_blackout_days"))
        j.at("dividend_blackout_days").get_to(config.dividend_blackout_days);
    if (j.contains("cache_duration_seconds"))
        j.at("cache_duration_seconds").get_to(config.cache_duration_seconds);
    if (j.contains("rate_limit_per_second"))
        j.at("rate_limit_per_second").get_to(config.rate_limit_per_second);
}

// Main Config
inline void to_json(nlohmann::json& j, const Config& config) {
    j = nlohmann::json{
        {"tws", config.tws},
        {"broker", config.broker},
        {"strategy", config.strategy},
        {"risk", config.risk},
        {"logging", config.logging},
        {"massive", config.massive},
        {"dry_run", config.dry_run},
        {"loop_delay_ms", config.loop_delay_ms},
        {"continue_on_error", config.continue_on_error}
    };
}

inline void from_json(const nlohmann::json& j, Config& config) {
    j.at("tws").get_to(config.tws);
    if (j.contains("broker"))
        j.at("broker").get_to(config.broker);
    j.at("strategy").get_to(config.strategy);
    if (j.contains("risk"))
        j.at("risk").get_to(config.risk);
    if (j.contains("logging"))
        j.at("logging").get_to(config.logging);
    if (j.contains("massive"))
        j.at("massive").get_to(config.massive);
    if (j.contains("dry_run"))
        j.at("dry_run").get_to(config.dry_run);
    if (j.contains("loop_delay_ms"))
        j.at("loop_delay_ms").get_to(config.loop_delay_ms);
    if (j.contains("continue_on_error"))
        j.at("continue_on_error").get_to(config.continue_on_error);
}

} // namespace config
