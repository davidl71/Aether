// alpaca_adapter.h - Alpaca adapter implementing IBroker interface
#pragma once

#include "brokers/broker_interface.h"
#include <string>
#include <memory>
#include <mutex>
#include <map>
#include <thread>
#include <atomic>

namespace brokers {

// Forward declaration
class HttpClient;

/**
 * Alpaca Adapter - Implements IBroker interface using Alpaca REST API
 *
 * This adapter allows the extracted box-spread-cpp library to work with
 * Alpaca Markets API through the broker-agnostic IBroker interface.
 */
class AlpacaAdapter : public box_spread::brokers::IBroker {
public:
    /**
     * Configuration for Alpaca adapter
     */
    struct Config {
        std::string api_key_id;
        std::string api_secret_key;
        std::string base_url;  // paper-api.alpaca.markets or api.alpaca.markets
        bool paper_trading = true;
        int poll_interval_ms = 2000;  // Market data polling interval
    };

    /**
     * Constructor
     * @param config Alpaca configuration
     */
    explicit AlpacaAdapter(const Config& config);

    /**
     * Destructor
     */
    ~AlpacaAdapter() override;

    // Disable copy
    AlpacaAdapter(const AlpacaAdapter&) = delete;
    AlpacaAdapter& operator=(const AlpacaAdapter&) = delete;

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
    // Type conversion helpers
    static box_spread::types::MarketData convert_market_data(const std::string& symbol, double bid, double ask, double last);
    static box_spread::types::Order convert_order(const std::string& order_id, const std::string& symbol, const std::string& status);
    static box_spread::types::Position convert_position(const std::string& symbol, int qty, double avg_price);

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

    // Market data subscriptions
    std::map<int, box_spread::types::OptionContract> subscriptions_;
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
