// tws_adapter.h - TWS adapter implementing IBroker interface
#pragma once

#include "brokers/broker_interface.h"
#include "tws_client.h"
#include "config_manager.h"
#include <memory>
#include <mutex>

namespace brokers {

/**
 * TWS Adapter - Bridges TWSClient to IBroker interface
 *
 * This adapter allows the extracted box-spread-cpp library to work with
 * Interactive Brokers TWS API through the broker-agnostic IBroker interface.
 */
class TWSAdapter : public box_spread::brokers::IBroker {
public:
    /**
     * Constructor
     * @param config TWS configuration
     */
    explicit TWSAdapter(const config::TWSConfig& config);

    /**
     * Destructor
     */
    ~TWSAdapter() override;

    // Disable copy
    TWSAdapter(const TWSAdapter&) = delete;
    TWSAdapter& operator=(const TWSAdapter&) = delete;

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

    // ========================================================================
    // Additional Methods (TWS-specific)
    // ========================================================================

    /**
     * Process incoming messages (call in main loop)
     * @param timeout_ms Timeout in milliseconds
     */
    void process_messages(int timeout_ms = 100);

    /**
     * Get underlying TWSClient (for advanced usage)
     * @return Pointer to TWSClient
     */
    tws::TWSClient* get_tws_client() { return tws_client_.get(); }
    const tws::TWSClient* get_tws_client() const { return tws_client_.get(); }

private:
    // Type conversion helpers
    static types::OptionContract convert_contract(const box_spread::types::OptionContract& contract);
    static box_spread::types::OptionContract convert_contract_back(const types::OptionContract& contract);
    static types::OrderAction convert_action(box_spread::types::OrderAction action);
    static box_spread::types::OrderAction convert_action_back(types::OrderAction action);
    static types::TimeInForce convert_tif(box_spread::types::TimeInForce tif);
    static box_spread::types::TimeInForce convert_tif_back(types::TimeInForce tif);
    static types::MarketData convert_market_data(const box_spread::types::MarketData& data);
    static box_spread::types::MarketData convert_market_data_back(const types::MarketData& data);
    static types::Order convert_order(const box_spread::types::Order& order);
    static box_spread::types::Order convert_order_back(const types::Order& order);
    static types::Position convert_position(const box_spread::types::Position& position);
    static box_spread::types::Position convert_position_back(const types::Position& position);
    static types::AccountInfo convert_account_info(const box_spread::types::AccountInfo& info);
    static box_spread::types::AccountInfo convert_account_info_back(const types::AccountInfo& info);
    static tws::ConnectionState convert_connection_state(box_spread::brokers::ConnectionState state);
    static box_spread::brokers::ConnectionState convert_connection_state_back(tws::ConnectionState state);

    std::unique_ptr<tws::TWSClient> tws_client_;
    mutable std::mutex mutex_;  // For thread-safe access
};

} // namespace brokers
