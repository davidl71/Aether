// tws_client.h - TWS API Client wrapper for IBKR Box Spread Generator
#pragma once

#include "types.h"
#include "config_manager.h"
#include "rate_limiter.h"
#include <memory>
#include <functional>
#include <map>
#include <vector>
#include <future>
#include <chrono>
#include <optional>

namespace tws {

// ============================================================================
// TWS Client Callbacks
// ============================================================================

using MarketDataCallback = std::function<void(const types::MarketData&)>;
using OrderStatusCallback = std::function<void(const types::Order&)>;
using PositionCallback = std::function<void(const types::Position&)>;
using AccountCallback = std::function<void(const types::AccountInfo&)>;
using ErrorCallback = std::function<void(int code, const std::string& msg)>;

// ============================================================================
// Connection State
// ============================================================================

enum class ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error
};

// ============================================================================
// TWS Client Class
// ============================================================================

class TWSClient {
public:
    // Constructor
    explicit TWSClient(const config::TWSConfig& config);

    // Destructor
    ~TWSClient();

    // Disable copy (manage connection state)
    TWSClient(const TWSClient&) = delete;
    TWSClient& operator=(const TWSClient&) = delete;

    // Connection management
    // Return value should be checked to verify connection succeeded
    [[nodiscard]] bool connect();
    void disconnect();
    // Pure query function - no side effects
    bool is_connected() const __attribute__((pure));
    // Pure query function - no side effects
    ConnectionState get_connection_state() const __attribute__((pure));

    // Process incoming messages (call in main loop)
    void process_messages(int timeout_ms = 100);

    // ========================================================================
    // Market Data Operations
    // ========================================================================

    // Request market data for an option contract (async - callback-based)
    int request_market_data(const types::OptionContract& contract,
                           MarketDataCallback callback);

    // Request market data synchronously (blocks until data received or timeout)
    std::optional<types::MarketData> request_market_data_sync(
        const types::OptionContract& contract,
        int timeout_ms = 5000
    );

    // Cancel market data subscription
    void cancel_market_data(int request_id);

    // Request option chain for underlying symbol
    std::vector<types::OptionContract> request_option_chain(
        const std::string& symbol,
        const std::string& expiry = ""  // Empty for all expiries
    );

    // ========================================================================
    // Order Operations
    // ========================================================================

    // Place a new order
    int place_order(const types::OptionContract& contract,
                    types::OrderAction action,
                    int quantity,
                    double limit_price = 0.0,  // 0 for market order
                    types::TimeInForce tif = types::TimeInForce::Day);

    // Place a combo order (BAG secType) for atomic multi-leg execution
    // Returns order ID, or -1 if failed
    // Note: Requires contract IDs (conId) for each leg - use reqContractDetails first if needed
    int place_combo_order(
        const std::vector<types::OptionContract>& contracts,
        const std::vector<types::OrderAction>& actions,
        const std::vector<int>& quantities,
        const std::vector<long>& contract_ids,  // TWS contract IDs (conId) for each leg
        const std::vector<double>& limit_prices,  // Limit price for each leg (0 for market)
        types::TimeInForce tif = types::TimeInForce::Day
    );

    // Cancel an order
    void cancel_order(int order_id);

    // Cancel all orders
    void cancel_all_orders();

    // Get order status
    // Pure query function - no side effects
    std::optional<types::Order> get_order(int order_id) const __attribute__((pure));

    // Get all active orders
    // Pure query function - no side effects
    std::vector<types::Order> get_active_orders() const __attribute__((pure));

    // ========================================================================
    // Position Operations
    // ========================================================================

    // Request current positions (async - callback-based)
    void request_positions(PositionCallback callback);

    // Request positions synchronously (blocks until positions received or timeout)
    std::vector<types::Position> request_positions_sync(int timeout_ms = 5000);

    // Get all positions (from cache - may be stale)
    // Pure query function - no side effects
    std::vector<types::Position> get_positions() const __attribute__((pure));

    // Get position for specific contract
    // Pure query function - no side effects
    std::optional<types::Position> get_position(
        const types::OptionContract& contract
    ) const __attribute__((pure));

    // ========================================================================
    // Account Operations
    // ========================================================================

    // Request account updates (async - callback-based)
    void request_account_updates(AccountCallback callback);

    // Request account updates synchronously (blocks until account info received or timeout)
    std::optional<types::AccountInfo> request_account_info_sync(int timeout_ms = 5000);

    // Get current account information (from cache - may be stale)
    // Pure query function - no side effects
    std::optional<types::AccountInfo> get_account_info() const __attribute__((pure));

    // ========================================================================
    // Callbacks
    // ========================================================================

    void set_order_status_callback(OrderStatusCallback callback);
    void set_error_callback(ErrorCallback callback);

    // ========================================================================
    // Helper Methods
    // ========================================================================

    // Get next valid order ID
    // Pure query function - no side effects
    int get_next_order_id() const __attribute__((pure));

    // Check if market is open
    // Pure query function - no side effects
    bool is_market_open() const __attribute__((pure));

    // Get server time
    // Pure query function - no side effects
    std::chrono::system_clock::time_point get_server_time() const __attribute__((pure));

    // ========================================================================
    // Rate Limiting (IBKR Compliance)
    // ========================================================================

    // Enable rate limiting with default settings
    void enable_rate_limiting();

    // Configure rate limiter
    void configure_rate_limiter(const RateLimiterConfig& config);

    // Get rate limiter status
    // Pure query function - no side effects
    std::optional<RateLimiterStatus> get_rate_limiter_status() const __attribute__((pure));

    // Cleanup stale requests (for long-running applications)
    void cleanup_stale_rate_limiter_requests(std::chrono::seconds max_age);

private:
    class Impl;
    std::unique_ptr<Impl> pimpl_;
};

// ============================================================================
// Helper Functions
// ============================================================================

// Validate symbol format
bool is_valid_symbol(const std::string& symbol);

// Parse option symbol (e.g., "SPY250620C00500000" -> components)
std::optional<types::OptionContract> parse_option_symbol(
    const std::string& option_symbol
);

// Format option symbol from components
std::string format_option_symbol(const types::OptionContract& contract);

// Calculate DTE (days to expiry)
int calculate_dte(const std::string& expiry);

} // namespace tws
