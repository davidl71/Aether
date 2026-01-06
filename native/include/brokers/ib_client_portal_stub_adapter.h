// ib_client_portal_stub_adapter.h - IB Client Portal stub adapter implementing IBroker interface with mock data
#pragma once

#include <box_spread/brokers/broker_interface.h>
#include <string>
#include <mutex>
#include <map>
#include <atomic>
#include <chrono>

namespace brokers {

/**
 * IB Client Portal Stub Adapter - Implements IBroker interface with mock data
 *
 * This stub adapter provides mock implementations for all IBroker methods,
 * allowing BrokerManager/Factory to be developed and tested without requiring
 * real IB Client Portal API integration. Returns realistic mock data that matches
 * the expected data structures.
 */
class IBClientPortalStubAdapter : public box_spread::brokers::IBroker {
public:
    /**
     * Configuration for IB Client Portal stub adapter
     */
    struct Config {
        std::string base_url = "https://localhost:5000/v1/api";
        std::string account_id = "STUB_ACCOUNT";
        bool use_oauth = false;
        std::string client_id = "STUB_CLIENT_ID";
        std::string client_secret = "STUB_SECRET";
        std::string username = "stub_user";
        std::string password = "stub_pass";
        int poll_interval_ms = 2000;
    };

    /**
     * Constructor
     * @param config IB Client Portal stub configuration
     */
    explicit IBClientPortalStubAdapter(const Config& config);

    /**
     * Destructor
     */
    ~IBClientPortalStubAdapter() override;

    // Disable copy
    IBClientPortalStubAdapter(const IBClientPortalStubAdapter&) = delete;
    IBClientPortalStubAdapter& operator=(const IBClientPortalStubAdapter&) = delete;

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
    // Mock data generation helpers
    box_spread::types::MarketData generate_mock_market_data(
        const box_spread::types::OptionContract& contract
    ) const;

    box_spread::types::AccountInfo generate_mock_account_info() const;

    long generate_mock_contract_id(const box_spread::types::OptionContract& contract) const;

    Config config_;
    mutable std::mutex mutex_;
    std::atomic<bool> connected_{false};
    std::atomic<box_spread::brokers::ConnectionState> connection_state_{
        box_spread::brokers::ConnectionState::Disconnected
    };

    // Market data subscriptions (stub - stores subscriptions but doesn't poll)
    std::map<int, box_spread::types::OptionContract> subscriptions_;
    std::map<int, std::function<void(const box_spread::types::MarketData&)>> callbacks_;
    int next_request_id_{1};
    int next_order_id_{2000};

    // Contract ID cache (stub - generates deterministic IDs)
    std::map<std::string, long> contract_id_cache_;

    // Order tracking (stub - stores mock orders)
    std::map<int, box_spread::types::Order> orders_;

    // Error callback
    std::function<void(int code, const std::string& msg)> error_callback_;
};

} // namespace brokers

