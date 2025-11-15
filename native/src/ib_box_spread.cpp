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
#include "box_spread_strategy.h"
#include "order_manager.h"
#include "risk_calculator.h"
#include "version.h"

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
        spdlog::info("║    Automated Options Arbitrage Trading System             ║");
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
    CLI::App app{"IBKR Box Spread Generator - Automated options arbitrage"};
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

        // TWS Client
        spdlog::info("Connecting to TWS on {}:{}",
                     config.tws.host, config.tws.port);

        auto tws_client = std::make_unique<tws::TWSClient>(config.tws);

        if (!tws_client->connect()) {
            spdlog::critical("Failed to connect to TWS");
            spdlog::error("Make sure TWS or IB Gateway is running");
            spdlog::error("Check that the port {} is correct", config.tws.port);
            spdlog::error("Port 7497 = Paper Trading, Port 7496 = Live Trading");
            return EXIT_FAILURE;
        }

        spdlog::info("✓ Connected to TWS");

        // Order Manager
        auto order_manager = std::make_unique<order::OrderManager>(
            tws_client.get(),
            config.dry_run
        );

        spdlog::info("✓ Order manager initialized");

        // Risk Calculator
        auto risk_calculator = std::make_unique<risk::RiskCalculator>(
            config.risk
        );

        spdlog::info("✓ Risk calculator initialized");

        // Box Spread Strategy
        auto strategy = std::make_unique<strategy::BoxSpreadStrategy>(
            tws_client.get(),
            order_manager.get(),
            config.strategy
        );

        spdlog::info("✓ Strategy initialized");
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
        tws_client->disconnect();

        spdlog::info("Shutdown complete");
        return EXIT_SUCCESS;

    } catch (const std::exception& e) {
        spdlog::critical("Fatal error: {}", e.what());
        return EXIT_FAILURE;
    }
}
