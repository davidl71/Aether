// ib_box_spread.cpp - Main entry point for IBKR Box Spread Generator
#include <CLI/CLI.hpp>
#include <spdlog/spdlog.h>
#include <spdlog/sinks/rotating_file_sink.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <iostream>
#include <memory>
#include <csignal>
#include <atomic>
#include <thread>
#include <chrono>
#include <filesystem>
#include <cstdlib>
#include <sstream>
#include <algorithm>

#include "config_manager.h"
#include "tws_client.h"
#include "version.h"

// Conditionally include local or extracted library components
#ifdef USE_BOX_SPREAD_CPP_LIB
  // Using extracted library - include adapter and library headers
  #include "brokers/tws_adapter.h"
  #include <box_spread/order_manager.h>
  #include <box_spread/risk_calculator.h>
  #include <box_spread/box_spread_strategy.h>
  #include <box_spread/config.h>
#else
  // Using local implementation
  #include "strategies/box_spread/box_spread_strategy.h"
  #include "order_manager.h"
  #include "risk_calculator.h"
#endif

// NOTE FOR AUTOMATION AGENTS:
// This translation unit glues together the CLI/TUI runtime. It wires configuration,
// logging, component construction, and the long-running trading loop. Business logic
// (pricing, order routing, risk) remains encapsulated in their respective modules.
// When extending behaviour, prefer touching the specialized modules first and keep
// this entry point focused on orchestration and lifecycle concerns.

namespace {
    std::atomic<bool> g_running{true};  // Flipped to false when a termination signal arrives

    void signal_handler(int signal) {
        if (signal == SIGINT || signal == SIGTERM) {
            spdlog::info("Received termination signal, shutting down gracefully...");
            g_running = false;
        }
    }

    // ========================================================================
    // System Health Check
    // ========================================================================

    struct SystemHealth {
        bool tws_connected;
        int active_positions;
        int pending_orders;
        double efficiency_ratio;
        int rate_limiter_messages_per_sec;
        int rate_limiter_active_market_data;
        int rate_limiter_active_historical;
        std::string last_error;
        int error_count_last_hour;
        std::chrono::system_clock::time_point timestamp;
    };

    SystemHealth get_system_health(
        tws::TWSClient* tws_client,
        order::OrderManager* order_manager,
        strategy::BoxSpreadStrategy* strategy
    ) {
        SystemHealth health{};
        health.timestamp = std::chrono::system_clock::now();

        // TWS connection status
        if (tws_client) {
            health.tws_connected = tws_client->is_connected();
        } else {
            health.tws_connected = false;
        }

        // Active positions
        if (strategy) {
            auto positions = strategy->get_active_positions();
            health.active_positions = static_cast<int>(positions.size());
        } else {
            health.active_positions = 0;
        }

        // Pending orders (from order manager stats)
        if (order_manager) {
            auto order_stats = order_manager->get_statistics();
            // Estimate pending orders (placed but not filled)
            health.pending_orders = order_stats.total_orders_placed - order_stats.total_orders_filled;
            health.efficiency_ratio = order_stats.efficiency_ratio;
        } else {
            health.pending_orders = 0;
            health.efficiency_ratio = 0.0;
        }

        // Rate limiter metrics (placeholder - would need to expose from TWSClient)
        health.rate_limiter_messages_per_sec = 0;  // TODO: Expose from TWSClient
        health.rate_limiter_active_market_data = 0;  // TODO: Expose from TWSClient
        health.rate_limiter_active_historical = 0;  // TODO: Expose from TWSClient

        // Error tracking (placeholder - would need error tracking system)
        health.last_error = "";  // TODO: Track last error
        health.error_count_last_hour = 0;  // TODO: Track error count

        return health;
    }

    void log_system_health(const SystemHealth& health) {
        spdlog::info("═══════════════════════════════════════");
        spdlog::info("System Health Check:");
        spdlog::info("  TWS Connected: {}", health.tws_connected ? "✓" : "✗");
        spdlog::info("  Active Positions: {}", health.active_positions);
        spdlog::info("  Pending Orders: {}", health.pending_orders);
        spdlog::info("  Efficiency Ratio: {:.2f}%", health.efficiency_ratio * 100.0);
        if (!health.last_error.empty()) {
            spdlog::info("  Last Error: {}", health.last_error);
        }
        if (health.error_count_last_hour > 0) {
            spdlog::warn("  Errors (last hour): {}", health.error_count_last_hour);
        }
        spdlog::info("═══════════════════════════════════════");
    }

    void setup_logging(const config::LogConfig& log_config) {
        try {
            std::vector<spdlog::sink_ptr> sinks;

            // File sink (rotating)
            auto max_size = static_cast<size_t>(log_config.max_file_size_mb) * 1024 * 1024;
            auto file_sink = std::make_shared<spdlog::sinks::rotating_file_sink_mt>(
                log_config.log_file,
                max_size,
                log_config.max_files
            );
            sinks.push_back(file_sink);

            // Console sink (if enabled)
            if (log_config.log_to_console) {
                auto console_sink = std::make_shared<spdlog::sinks::stdout_color_sink_mt>();
                sinks.push_back(console_sink);
            }

            // Create logger with multiple sinks
            auto logger = std::make_shared<spdlog::logger>(
                "ib_box_spread",
                sinks.begin(),
                sinks.end()
            );

            // Set log level
            spdlog::level::level_enum level = spdlog::level::info;
            if (log_config.log_level == "trace") level = spdlog::level::trace;
            else if (log_config.log_level == "debug") level = spdlog::level::debug;
            else if (log_config.log_level == "warn") level = spdlog::level::warn;
            else if (log_config.log_level == "error") level = spdlog::level::err;

            logger->set_level(level);
            logger->set_pattern("[%Y-%m-%d %H:%M:%S.%e] [%l] [%t] %v");
            spdlog::set_default_logger(logger);

            spdlog::info("Logging initialized: file={}, level={}",
                        log_config.log_file, log_config.log_level);
        } catch (const spdlog::spdlog_ex& ex) {
            std::cerr << "Log initialization failed: " << ex.what() << std::endl;
            throw;
        }
    }

    namespace fs = std::filesystem;

    void add_candidate(std::vector<fs::path>& candidates, const fs::path& path) {
        if (path.empty()) {
            return;
        }
        fs::path normalized = path.lexically_normal();
        if (std::find(candidates.begin(), candidates.end(), normalized) == candidates.end()) {
            candidates.push_back(normalized);
        }
    }

    fs::path make_absolute(const fs::path& path) {
        if (path.is_absolute()) {
            return path.lexically_normal();
        }
        return (fs::current_path() / path).lexically_normal();
    }

    fs::path default_user_config_path() {
        if (const char* home_env = std::getenv("HOME")) {
            fs::path home(home_env);
#ifdef __APPLE__
            return (home / ".config" / "ib_box_spread" / "config.json").lexically_normal();
#else
            return (home / ".config" / "ib_box_spread" / "config.json").lexically_normal();
#endif
        }
        return (fs::current_path() / "config" / "config.json").lexically_normal();
    }

    fs::path resolve_config_path(const std::string& requested_path, bool user_override) {
        std::vector<fs::path> candidates;

        if (!requested_path.empty()) {
            fs::path requested(requested_path);
            add_candidate(candidates, make_absolute(requested));
            if (requested.is_relative()) {
                add_candidate(candidates, requested);
            }
        }

        if (const char* env_override = std::getenv("IB_BOX_SPREAD_CONFIG")) {
            fs::path env_path(env_override);
            add_candidate(candidates, make_absolute(env_path));
            if (env_path.is_relative()) {
                add_candidate(candidates, env_path);
            }
        }

        if (!user_override) {
            if (const char* home_env = std::getenv("HOME")) {
                fs::path home(home_env);
                add_candidate(candidates, home / ".config" / "ib_box_spread" / "config.json");
#ifdef __APPLE__
                add_candidate(candidates,
                              home / "Library" / "Application Support" / "ib_box_spread"
                                   / "config.json");
#endif
            }

            add_candidate(candidates, fs::path("/usr/local/etc/ib_box_spread/config.json"));
            add_candidate(candidates, fs::path("/etc/ib_box_spread/config.json"));
        }

        std::error_code ec;
        for (const auto& candidate : candidates) {
            if (fs::exists(candidate, ec) && fs::is_regular_file(candidate, ec)) {
                spdlog::debug("Resolved configuration candidate: {}", candidate.string());
                return candidate;
            }
            ec.clear();
        }

        std::ostringstream oss;
        oss << "Configuration file not found. Searched:";
        for (const auto& candidate : candidates) {
            oss << "\n  - " << candidate.string();
        }
        throw std::runtime_error(oss.str());
    }

    void print_banner() {
        spdlog::info("╔════════════════════════════════════════════════════════════╗");
        spdlog::info("║    IBKR Box Spread Generator v1.0.0                       ║");
        spdlog::info("║    Synthetic Financing Platform (Box Spread Rates)        ║");
        spdlog::info("╚════════════════════════════════════════════════════════════╝");
    }

    void print_config_summary(const config::Config& config) {
        spdlog::info("Configuration Summary:");
        spdlog::info("  TWS: {}:{} (client_id={})",
                    config.tws.host, config.tws.port, config.tws.client_id);
        spdlog::info("  Mode: {}", config.dry_run ? "DRY RUN (Paper Trading)" : "LIVE");
        spdlog::info("  Symbols: {}", fmt::join(config.strategy.symbols, ", "));
        spdlog::info("  Min Profit: ${:.2f}", config.strategy.min_arbitrage_profit);
        spdlog::info("  Min ROI: {:.2f}%", config.strategy.min_roi_percent);
        spdlog::info("  Max Position: ${:.2f}", config.strategy.max_position_size);
        spdlog::info("  Max Exposure: ${:.2f}", config.risk.max_total_exposure);
        spdlog::info("  Max Positions: {}", config.risk.max_positions);
    }
}

int main(int argc, char** argv) {
    // ========================================================================
    // Command-line Parsing
    // ========================================================================
    CLI::App app{"IBKR Box Spread Generator - Synthetic financing platform"};
    app.set_version_flag("-v,--version", VERSION_STRING);

    std::string config_file = "config/config.json";
    bool dry_run = false;
    bool validate_only = false;
    bool use_nautilus = false;
    bool mock_tws = false;
    std::string log_level_override;
    std::string init_config_override;

    auto* config_option = app.add_option(
        "-c,--config",
        config_file,
        "Configuration file path (defaults to ~/.config/ib_box_spread/config.json if present)"
    );
    auto* init_option = app.add_option(
        "--init-config",
        init_config_override,
        "Write a sample configuration to the provided path (default: ~/.config/ib_box_spread/config.json) and exit"
    )->expected(0, 1);
    app.add_flag("--dry-run", dry_run, "Simulate trading without executing orders");

    app.add_flag("--validate", validate_only, "Validate configuration and exit");

    app.add_flag("--use-nautilus", use_nautilus,
                 "Use nautilus_trader for market data and execution (requires Python)");

    app.add_flag("--mock-tws", mock_tws,
                 "Use the in-process mock TWS client (no live IB connection required)");

    app.add_option("--log-level", log_level_override,
                  "Override log level (trace|debug|info|warn|error)")
        ->check(CLI::IsMember({"trace", "debug", "info", "warn", "error"}));

    try {
        app.parse(argc, argv);
    } catch (const CLI::ParseError& e) {
        return app.exit(e);
    }

    if (init_option->count() > 0) {
        fs::path target = init_config_override.empty()
                              ? default_user_config_path()
                              : fs::path(init_config_override);
        if (!target.is_absolute()) {
            target = make_absolute(target);
        }

        const fs::path parent = target.parent_path();
        std::error_code ec;
        if (!parent.empty()) {
            fs::create_directories(parent, ec);
            if (ec) {
                std::cerr << "Failed to create directory " << parent << ": " << ec.message()
                          << std::endl;
                return EXIT_FAILURE;
            }
        }

        try {
            config::Config sample = config::ConfigManager::get_default();
            if (sample.strategy.symbols.empty()) {
                // Default to European-style symbols
                sample.strategy.symbols.push_back("SPX");
            }
            sample.dry_run = true;
            config::ConfigManager::save(sample, target.string());
        } catch (const std::exception& e) {
            std::cerr << "Failed to write sample configuration: " << e.what() << std::endl;
            return EXIT_FAILURE;
        }

        std::cout << "Sample configuration written to: " << target.string() << std::endl;
        std::cout << "Update the values before running live." << std::endl;
        return EXIT_SUCCESS;
    }

    const bool user_provided_config = config_option->count() > 0;
    try {
        config_file = resolve_config_path(config_file, user_provided_config).string();
    } catch (const std::exception& e) {
        std::cerr << e.what() << std::endl;
        return EXIT_FAILURE;
    }

    // ========================================================================
    // Configuration Loading
    // ========================================================================
    config::Config config;

    try {
        spdlog::info("Loading configuration from: {}", config_file);
        config = config::ConfigManager::load(config_file);

        // Override settings from command line / environment
        if (dry_run) {
            config.dry_run = true;
        }
        if (!log_level_override.empty()) {
            config.logging.log_level = log_level_override;
        }
        auto env_truthy = [](const char* value) {
            if (value == nullptr) {
                return false;
            }
            std::string str(value);
            std::transform(str.begin(), str.end(), str.begin(), [](unsigned char c) {
                return static_cast<char>(std::tolower(c));
            });
            return str == "1" || str == "true" || str == "yes" || str == "on";
        };
        if (mock_tws) {
            config.tws.use_mock = true;
        }
        if (env_truthy(std::getenv("TWS_MOCK"))) {
            config.tws.use_mock = true;
        }

        // Check for nautilus_trader usage
        if (use_nautilus) {
            spdlog::info("NautilusTrader mode requested via --use-nautilus flag");
            spdlog::info("Note: Use python/nautilus_strategy.py for full nautilus_trader integration");
            spdlog::info("C++ application will continue with TWS client");
        }

        if (config.tws.use_mock) {
            spdlog::warn("Mock TWS client enabled. No live IBKR connection will be attempted.");
        }

        // Validate configuration
        config::ConfigManager::validate(config);

        if (validate_only) {
            std::cout << "Configuration validation successful!" << std::endl;
            return EXIT_SUCCESS;
        }

    } catch (const std::exception& e) {
        std::cerr << "Configuration error: " << e.what() << std::endl;
        return EXIT_FAILURE;
    }

    // ========================================================================
    // Logging Setup
    // ========================================================================
    try {
        setup_logging(config.logging);
    } catch (const std::exception& e) {
        std::cerr << "Failed to setup logging: " << e.what() << std::endl;
        return EXIT_FAILURE;
    }

    // ========================================================================
    // Signal Handling
    // ========================================================================
    std::signal(SIGINT, signal_handler);
    std::signal(SIGTERM, signal_handler);

    // ========================================================================
    // Application Startup
    // ========================================================================
    print_banner();
    print_config_summary(config);

    if (config.dry_run) {
        spdlog::warn("⚠️  DRY RUN MODE - No real trades will be executed");
    } else {
        spdlog::warn("⚠️  LIVE TRADING MODE - Real money at risk!");
        spdlog::warn("⚠️  Press Ctrl+C to stop safely");
    }

    try {
        // ====================================================================
        // Component Initialization
        // ====================================================================
        spdlog::info("Initializing components...");

        // Broker selection (non-blocking): try in priority order, fall back to mock
        std::vector<std::string> priorities;
        if (!config.broker.priorities.empty()) {
            priorities = config.broker.priorities;
        } else {
            priorities = {"mock", "alpaca", "ib", "fibi", "meitav", "ibi", "discount"};
        }
        // Normalize to lowercase
        for (auto& p : priorities) {
            std::transform(p.begin(), p.end(), p.begin(),
                           [](unsigned char c) { return static_cast<char>(std::tolower(c)); });
        }

#ifdef USE_BOX_SPREAD_CPP_LIB
        // Using extracted library - create TWS adapter
        std::unique_ptr<brokers::TWSAdapter> broker_adapter;
        bool connected = false;

        for (const auto& broker : priorities) {
            if (broker == "mock") {
                spdlog::info("Selecting broker: MOCK (in-process)");
                config.tws.use_mock = true;
                broker_adapter = std::make_unique<brokers::TWSAdapter>(config.tws);
                connected = broker_adapter->connect();
                if (connected) {
                    spdlog::info("✓ Using mock TWS client via adapter");
                    break;
                }
                spdlog::warn("Mock TWS client failed to initialize, trying next broker");
            } else if (broker == "alpaca") {
                spdlog::warn("Alpaca broker selected, but C++ adapter is not implemented. "
                             "Continuing in analysis/dry-run mode using mock client.");
                config.tws.use_mock = true;
                broker_adapter = std::make_unique<brokers::TWSAdapter>(config.tws);
                connected = broker_adapter->connect();
                if (connected) {
                    break;
                }
            } else if (broker == "ib") {
                spdlog::info("Selecting broker: Interactive Brokers (TWS/Gateway)");
                config.tws.use_mock = false;
                spdlog::info("Connecting to TWS on {}:{}", config.tws.host, config.tws.port);
                broker_adapter = std::make_unique<brokers::TWSAdapter>(config.tws);
                connected = broker_adapter->connect();
                if (connected) {
                    spdlog::info("✓ Connected to TWS via adapter");
                    break;
                }
                spdlog::error("Failed to connect to TWS on {}:{}", config.tws.host, config.tws.port);
                spdlog::error("Check TWS/Gateway is running and ports are correct (7497 paper / 7496 live)");
                // Do not exit; move to next broker
            } else if (broker == "fibi" || broker == "meitav" || broker == "ibi" || broker == "discount") {
                spdlog::warn("{} broker selected, but C++ adapter is not implemented. "
                             "Continuing with mock client.", broker);
                config.tws.use_mock = true;
                broker_adapter = std::make_unique<brokers::TWSAdapter>(config.tws);
                connected = broker_adapter->connect();
                if (connected) {
                    break;
                }
            } else {
                spdlog::warn("Unknown broker identifier '{}', skipping", broker);
            }
        }

        if (!broker_adapter) {
            spdlog::warn("No broker connection established. Falling back to mock client.");
            config.tws.use_mock = true;
            broker_adapter = std::make_unique<brokers::TWSAdapter>(config.tws);
            connected = broker_adapter->connect();
        }

        if (!connected) {
            spdlog::critical("Unable to initialize any broker client (including mock).");
            if (config.continue_on_error || config.dry_run) {
                spdlog::warn("Proceeding without a live connection. Strategy will operate in analysis mode.");
            } else {
                return EXIT_FAILURE;
            }
        }

        // Convert config to library format
        box_spread::config::StrategyParams strategy_params;
        strategy_params.symbols = config.strategy.symbols;
        strategy_params.min_arbitrage_profit = config.strategy.min_arbitrage_profit;
        strategy_params.min_roi_percent = config.strategy.min_roi_percent;
        strategy_params.max_position_size = config.strategy.max_position_size;
        strategy_params.min_days_to_expiry = config.strategy.min_days_to_expiry;
        strategy_params.max_days_to_expiry = config.strategy.max_days_to_expiry;
        strategy_params.max_bid_ask_spread = config.strategy.max_bid_ask_spread;
        strategy_params.min_volume = config.strategy.min_volume;
        strategy_params.min_open_interest = config.strategy.min_open_interest;
        // Commission config: library uses generic CommissionConfig, local uses IBKR-specific
        // Copy basic fields - full conversion would require mapping IBKR tiers to generic tiers
        strategy_params.commissions.per_contract_fee = config.strategy.commissions.per_contract_fee;
        strategy_params.commissions.minimum_order_fee = config.strategy.commissions.minimum_order_fee;
        strategy_params.commissions.maximum_order_fee_pct = config.strategy.commissions.maximum_order_fee_pct;
        strategy_params.commissions.monthly_contract_volume = config.strategy.commissions.monthly_contract_volume;

        box_spread::config::RiskConfig risk_config;
        risk_config.max_total_exposure = config.risk.max_total_exposure;
        risk_config.max_positions = config.risk.max_positions;
        risk_config.max_loss_per_position = config.risk.max_loss_per_position;
        risk_config.max_daily_loss = config.risk.max_daily_loss;
        risk_config.position_size_percent = config.risk.position_size_percent;
        risk_config.enable_stop_loss = config.risk.enable_stop_loss;
        risk_config.stop_loss_percent = config.risk.stop_loss_percent;

        // Order Manager (extracted library)
        auto order_manager = std::make_unique<box_spread::order::OrderManager>(
            broker_adapter.get(),
            config.dry_run
        );

        spdlog::info("✓ Order manager initialized (extracted library)");

        // Risk Calculator (extracted library) - constructor may differ, check header
        auto risk_calculator = std::make_unique<box_spread::risk::RiskCalculator>(
            risk_config
        );

        spdlog::info("✓ Risk calculator initialized (extracted library)");

        // Box Spread Strategy (extracted library)
        auto strategy = std::make_unique<box_spread::strategy::BoxSpreadStrategy>(
            broker_adapter.get(),
            order_manager.get(),
            strategy_params
        );

        spdlog::info("✓ Strategy initialized (extracted library)");
#else
        // Using local implementation - create TWSClient directly
        std::unique_ptr<tws::TWSClient> tws_client;
        bool connected = false;

        for (const auto& broker : priorities) {
            if (broker == "mock") {
                spdlog::info("Selecting broker: MOCK (in-process)");
                config.tws.use_mock = true;
                tws_client = std::make_unique<tws::TWSClient>(config.tws);
                connected = tws_client->connect();
                if (connected) {
                    spdlog::info("✓ Using mock TWS client");
                    break;
                }
                spdlog::warn("Mock TWS client failed to initialize, trying next broker");
            } else if (broker == "alpaca") {
                spdlog::warn("Alpaca broker selected, but C++ adapter is not implemented. "
                             "Continuing in analysis/dry-run mode using mock client.");
                config.tws.use_mock = true;
                tws_client = std::make_unique<tws::TWSClient>(config.tws);
                connected = tws_client->connect();
                if (connected) {
                    break;
                }
            } else if (broker == "ib") {
                spdlog::info("Selecting broker: Interactive Brokers (TWS/Gateway)");
                config.tws.use_mock = false;
                spdlog::info("Connecting to TWS on {}:{}", config.tws.host, config.tws.port);
                tws_client = std::make_unique<tws::TWSClient>(config.tws);
                connected = tws_client->connect();
                if (connected) {
                    spdlog::info("✓ Connected to TWS");
                    break;
                }
                spdlog::error("Failed to connect to TWS on {}:{}", config.tws.host, config.tws.port);
                spdlog::error("Check TWS/Gateway is running and ports are correct (7497 paper / 7496 live)");
                // Do not exit; move to next broker
            } else if (broker == "fibi" || broker == "meitav" || broker == "ibi" || broker == "discount") {
                spdlog::warn("{} broker selected, but C++ adapter is not implemented. "
                             "Continuing with mock client.", broker);
                config.tws.use_mock = true;
                tws_client = std::make_unique<tws::TWSClient>(config.tws);
                connected = tws_client->connect();
                if (connected) {
                    break;
                }
            } else {
                spdlog::warn("Unknown broker identifier '{}', skipping", broker);
            }
        }

        if (!tws_client) {
            spdlog::warn("No broker connection established. Falling back to mock client.");
            config.tws.use_mock = true;
            tws_client = std::make_unique<tws::TWSClient>(config.tws);
            connected = tws_client->connect();
        }

        if (!connected) {
            spdlog::critical("Unable to initialize any broker client (including mock).");
            if (config.continue_on_error || config.dry_run) {
                spdlog::warn("Proceeding without a live connection. Strategy will operate in analysis mode.");
            } else {
                return EXIT_FAILURE;
            }
        }

        // Order Manager (local implementation)
        auto order_manager = std::make_unique<order::OrderManager>(
            tws_client.get(),
            config.dry_run
        );

        spdlog::info("✓ Order manager initialized");

        // Risk Calculator (local implementation)
        auto risk_calculator = std::make_unique<risk::RiskCalculator>(
            config.risk
        );

        spdlog::info("✓ Risk calculator initialized");

        // Box Spread Strategy (local implementation)
        auto strategy = std::make_unique<strategy::BoxSpreadStrategy>(
            tws_client.get(),
            order_manager.get(),
            config.strategy
        );

        spdlog::info("✓ Strategy initialized");
#endif

        spdlog::info("All components initialized successfully");

        // ====================================================================
        // Main Trading Loop
        // ====================================================================
        spdlog::info("Starting trading loop...");
        spdlog::info("Monitoring symbols: {}", fmt::join(config.strategy.symbols, ", "));

        int loop_count = 0;

        while (g_running) {
            try {
                auto iteration_start = std::chrono::steady_clock::now();

#ifdef USE_BOX_SPREAD_CPP_LIB
                // Process TWS messages via adapter
                if (broker_adapter) {
                    broker_adapter->process_messages();
                }

                // Check if connected
                if (!broker_adapter || !broker_adapter->is_connected()) {
                    // Let the outer loop decide whether to reconnect or shut down based on config
                    spdlog::error("Lost connection to TWS");
                    if (config.tws.auto_reconnect && broker_adapter) {
                        spdlog::info("Attempting to reconnect...");
                        if (broker_adapter->connect()) {
                            spdlog::info("Reconnected successfully");
                        } else {
                            spdlog::error("Reconnection failed");
                            break;
                        }
                    } else {
                        break;
                    }
                }
#else
                // Process TWS messages
                tws_client->process_messages();

                // Check if connected
                if (!tws_client->is_connected()) {
                    // Let the outer loop decide whether to reconnect or shut down based on config
                    spdlog::error("Lost connection to TWS");
                    if (config.tws.auto_reconnect) {
                        spdlog::info("Attempting to reconnect...");
                        if (tws_client->connect()) {
                            spdlog::info("Reconnected successfully");
                        } else {
                            spdlog::error("Reconnection failed");
                            break;
                        }
                    } else {
                        break;
                    }
                }
#endif

                // Update order statuses
                // Keeps local state in sync with TWS fills/acks without issuing new orders
                order_manager->update();

                // Execute strategy (evaluate opportunities)
                // Core decision engine: fetches market data snapshots and decides on box spread legs
                strategy->evaluate_opportunities();

                // Monitor existing positions
                // Handles trailing risk controls and exit criteria on open spreads
                strategy->monitor_positions();

                // Log statistics periodically (every 100 iterations)
                if (++loop_count % 100 == 0) {
                    auto stats = strategy->get_statistics();
                    spdlog::info("Statistics: opportunities={}, trades={}, profit=${:.2f}",
                                stats.total_opportunities_found,
                                stats.total_trades_executed,
                                stats.total_profit);

                    auto positions = strategy->get_active_positions();
                    spdlog::info("Active positions: {}", positions.size());

                    // Log system health check
#ifdef USE_BOX_SPREAD_CPP_LIB
                    // Use adapter's TWSClient for health check
                    auto health = get_system_health(
                        broker_adapter ? broker_adapter->get_tws_client() : nullptr,
                        order_manager.get(),
                        strategy.get()
                    );
#else
                    auto health = get_system_health(
                        tws_client.get(),
                        order_manager.get(),
                        strategy.get()
                    );
#endif
                    log_system_health(health);
                }

                // Sleep to maintain loop timing
                auto iteration_end = std::chrono::steady_clock::now();
                auto iteration_duration = std::chrono::duration_cast<std::chrono::milliseconds>(
                    iteration_end - iteration_start
                ).count();

                int sleep_time = config.loop_delay_ms - static_cast<int>(iteration_duration);
                // Loop cadence is controlled via configuration; avoid busy-waiting when work finishes early
                if (sleep_time > 0) {
                    std::this_thread::sleep_for(std::chrono::milliseconds(sleep_time));
                }

            } catch (const std::exception& e) {
                spdlog::error("Error in trading loop: {}", e.what());

                if (!config.continue_on_error) {
                    spdlog::critical("Stopping due to error (continue_on_error=false)");
                    break;
                }

                // Backoff on errors
                std::this_thread::sleep_for(std::chrono::seconds(5));
            }
        }

        // ====================================================================
        // Cleanup and Shutdown
        // ====================================================================
        spdlog::info("Shutting down gracefully...");

        // Cancel all pending orders
        spdlog::info("Cancelling all pending orders...");
        order_manager->cancel_all_orders();

        // Print final statistics
        auto final_stats = strategy->get_statistics();
        spdlog::info("═══════════════════════════════════════");
        spdlog::info("Final Statistics:");
        spdlog::info("  Total opportunities: {}", final_stats.total_opportunities_found);
        spdlog::info("  Total trades: {}", final_stats.total_trades_executed);
        spdlog::info("  Successful: {}", final_stats.successful_trades);
        spdlog::info("  Failed: {}", final_stats.failed_trades);
        spdlog::info("  Total profit: ${:.2f}", final_stats.total_profit);
        spdlog::info("  Total loss: ${:.2f}", final_stats.total_loss);
        spdlog::info("  Net P&L: ${:.2f}", final_stats.total_profit - final_stats.total_loss);
        if (final_stats.total_trades_executed > 0) {
            spdlog::info("  Win rate: {:.1f}%", final_stats.win_rate * 100.0);
            spdlog::info("  Avg profit/trade: ${:.2f}", final_stats.average_profit_per_trade);
        }
        spdlog::info("═══════════════════════════════════════");

        // Disconnect from TWS
        spdlog::info("Disconnecting from TWS...");
#ifdef USE_BOX_SPREAD_CPP_LIB
        if (broker_adapter) {
            broker_adapter->disconnect();
        }
#else
        tws_client->disconnect();
#endif

        spdlog::info("Shutdown complete");
        return EXIT_SUCCESS;

    } catch (const std::exception& e) {
        spdlog::critical("Fatal error: {}", e.what());
        return EXIT_FAILURE;
    }
}
