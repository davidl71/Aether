// ib_client_portal_adapter.h - IB Client Portal adapter implementing IBroker interface
#pragma once

#include "brokers/broker_interface.h"
#include <string>
#include <memory>
#include <mutex>
#include <map>
#include <thread>
#include <atomic>
#include <chrono>

namespace brokers {

// Forward declaration
class HttpClient;

/**
 * IB Client Portal Adapter - Implements IBroker interface using IB Client Portal REST API
 *
 * This adapter allows the extracted box-spread-cpp library to work with
 * Interactive Brokers Client Portal Web API through the broker-agnostic IBroker interface.
 */
class IBClientPortalAdapter : public box_spread::brokers::IBroker {
public:
    /**
     * Configuration for IB Client Portal adapter
     */
    struct Config {
        std::string base_url;  // https://localhost:5001/v1/api
        std::string account_id;
        bool use_oauth = false;
        std::string client_id;  // For OAuth
        std::string client_secret;  // For OAuth
        std::string username;  // For session-based auth
        std::string password;  // For session-based auth
        int poll_interval_ms = 2000;  // Market data polling interval
    };

    /**
     * Constructor
     * @param config IB Client Portal configuration
     */
    explicit IBClientPortalAdapter(const Config& config);

    /**
     * Destructor
     */
    ~IBClientPortalAdapter() override;

    // Disable copy
    IBClientPortalAdapter(const IBClientPortalAdapter&) = delete;
    IBClientPortalAdapter& operator=(const IBClientPortalAdapter&) = delete;

    // ========================================================================
    // Connection Management
    // ========================================================================

    [[nodiscard]] bool connect() override;
    void disconnect() override;
    [[nodiscard]] bool is_connected() const override;
    [[nodiscard]] box_spread::brokers::ConnectionState get_connection_state() const override;
    [[nodiscard]] box_spread::brokers::BrokerType get_broker_type() const override;
    [[nodiscard]] box_spread::brokers::BrokerCapabilities get_capabilities() const override;

    // ========================================================================
    // Market Data
    // ========================================================================

    int request_market_data(
        const box_spread::types::OptionContract& contract,
        std::function<void(const box_spread::types::MarketData&)> callback
    ) override;

    void cancel_market_data(int request_id) override;

    [[nodiscard]] std::optional<box_spread::types::MarketData> request_market_data_sync(
        const box_spread::types::OptionContract& contract,
        int timeout_ms = 5000
    ) override;

    // ========================================================================
    // Options Chain
    // ========================================================================

    [[nodiscard]] std::vector<box_spread::types::OptionContract> request_option_chain(
        const std::string& symbol,
        const std::string& expiry = ""
    ) override;

    // ========================================================================
    // Contract Details
    // ========================================================================

    int request_contract_details(
        const box_spread::types::OptionContract& contract,
        std::function<void(long conId)> callback
    ) override;

    [[nodiscard]] long request_contract_details_sync(
        const box_spread::types::OptionContract& contract,
        int timeout_ms = 5000
    ) override;

    // ========================================================================
    // Order Management
    // ========================================================================

    int place_order(
        const box_spread::types::OptionContract& contract,
        box_spread::types::OrderAction action,
        int quantity,
        double limit_price = 0.0,
        box_spread::types::TimeInForce tif = box_spread::types::TimeInForce::Day
    ) override;

    bool cancel_order(int order_id) override;

    [[nodiscard]] std::optional<box_spread::types::Order> get_order_status(int order_id) const override;

    // ========================================================================
    // Multi-Leg Orders (Box Spreads)
    // ========================================================================

    int place_combo_order(
        const std::vector<box_spread::types::OptionContract>& contracts,
        const std::vector<box_spread::types::OrderAction>& actions,
        const std::vector<int>& quantities,
        const std::vector<long>& contract_ids,
        const std::vector<double>& limit_prices
    ) override;

    // ========================================================================
    // Positions
    // ========================================================================

    [[nodiscard]] std::vector<box_spread::types::Position> get_positions() override;

    [[nodiscard]] std::optional<box_spread::types::Position> get_position(
        const box_spread::types::OptionContract& contract
    ) override;

    // ========================================================================
    // Account Information
    // ========================================================================

    [[nodiscard]] std::optional<box_spread::types::AccountInfo> get_account_info() override;

    [[nodiscard]] double get_buying_power() override;

    [[nodiscard]] double get_net_liquidation_value() override;

    // ========================================================================
    // Error Handling
    // ========================================================================

    void set_error_callback(
        std::function<void(int code, const std::string& msg)> callback
    ) override;

private:
    // Session management
    bool authenticate();
    bool refresh_session();
    bool is_session_valid() const;
    std::string get_session_token() const;

    // Type conversion helpers
    static box_spread::types::MarketData convert_market_data(long conid, double bid, double ask, double last);
    static box_spread::types::Order convert_order(const std::string& order_id, const std::string& status);
    static box_spread::types::Position convert_position(long conid, int qty, double avg_price);

    // HTTP client helpers
    std::string make_request(const std::string& method, const std::string& endpoint, const std::string& body = "");
    std::string get_auth_headers() const;

    // Contract ID lookup
    long lookup_contract_id(const box_spread::types::OptionContract& contract);

    // Market data polling
    void start_market_data_polling();
    void stop_market_data_polling();
    void poll_market_data_loop();

    Config config_;
    std::unique_ptr<HttpClient> http_client_;
    mutable std::mutex mutex_;
    std::atomic<bool> connected_{false};
    std::atomic<box_spread::brokers::ConnectionState> connection_state_{box_spread::brokers::ConnectionState::Disconnected};

    // Session management
    std::string session_token_;
    std::chrono::system_clock::time_point token_expiry_;

    // Market data subscriptions
    std::map<int, long> subscriptions_;  // request_id -> conid
    std::map<int, std::function<void(const box_spread::types::MarketData&)>> callbacks_;
    int next_request_id_{1};
    std::atomic<bool> polling_active_{false};
    std::thread polling_thread_;

    // Contract ID cache
    std::map<std::string, long> contract_id_cache_;

    // Error callback
    std::function<void(int code, const std::string& msg)> error_callback_;
};

} // namespace brokers
