// order_manager.h - Order management and execution
#pragma once

#include "types.h"
#include "tws_client.h"
#include <memory>
#include <vector>
#include <optional>
#include <functional>

namespace order {

// ============================================================================
// Order Execution Result
// ============================================================================

struct ExecutionResult {
    bool success;
    std::vector<int> order_ids;  // IDs of orders placed
    std::string error_message;
    std::chrono::system_clock::time_point execution_time;

    double total_cost;
    double average_fill_price;
    int total_quantity_filled;
};

// ============================================================================
// Multi-Leg Order
// ============================================================================

struct MultiLegOrder {
    std::string strategy_id;      // Unique ID for the strategy
    std::vector<types::Order> legs;
    types::OrderStatus status;
    std::chrono::system_clock::time_point created_time;

    // Execution tracking
    int legs_filled;
    double total_cost;
    bool is_complete() const;
    bool is_partially_filled() const;
};

// ============================================================================
// Order Manager Class
// ============================================================================

class OrderManager {
public:
    OrderManager(tws::TWSClient* client, bool dry_run = false);
    ~OrderManager();

    // Disable copy
    OrderManager(const OrderManager&) = delete;
    OrderManager& operator=(const OrderManager&) = delete;

    // ========================================================================
    // Single Order Operations
    // ========================================================================

    // Place a single order
    ExecutionResult place_order(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        double limit_price = 0.0,
        types::TimeInForce tif = types::TimeInForce::Day
    );

    // Cancel an order
    bool cancel_order(int order_id);

    // Cancel all orders
    void cancel_all_orders();

    // Get order status
    std::optional<types::Order> get_order_status(int order_id) const;

    // ========================================================================
    // Multi-Leg Order Operations (Box Spreads)
    // ========================================================================

    // Place a box spread (4-leg order)
    ExecutionResult place_box_spread(
        const types::BoxSpreadLeg& spread,
        const std::string& strategy_id = ""
    );

    // Close a box spread position
    ExecutionResult close_box_spread(const std::string& strategy_id);

    // Get multi-leg order status
    std::optional<MultiLegOrder> get_multi_leg_order(
        const std::string& strategy_id
    ) const;

    // ========================================================================
    // Order Monitoring
    // ========================================================================

    // Update all order statuses
    void update();

    // Get all active orders
    std::vector<types::Order> get_active_orders() const;

    // Get all orders for a strategy
    std::vector<types::Order> get_strategy_orders(
        const std::string& strategy_id
    ) const;

    // Check if all legs of a multi-leg order are filled
    bool are_all_legs_filled(const std::string& strategy_id) const;

    // ========================================================================
    // Order Execution Strategies
    // ========================================================================

    // Execute with immediate-or-cancel
    ExecutionResult execute_ioc(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        double limit_price
    );

    // Execute with fill-or-kill
    ExecutionResult execute_fok(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        double limit_price
    );

    // Execute with TWAP (Time-Weighted Average Price)
    ExecutionResult execute_twap(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        int duration_seconds
    );

    // ========================================================================
    // Smart Order Routing
    // ========================================================================

    // Get best price for a contract
    std::optional<double> get_best_price(
        const types::OptionContract& contract,
        types::OrderAction action
    ) const;

    // Estimate fill probability
    double estimate_fill_probability(
        const types::OptionContract& contract,
        types::OrderAction action,
        double limit_price
    ) const;

    // ========================================================================
    // Risk Controls
    // ========================================================================

    // Set maximum order size
    void set_max_order_size(int max_contracts);

    // Set maximum orders per second (rate limiting)
    void set_max_orders_per_second(int max_rate);

    // Enable/disable dry run mode
    void set_dry_run(bool enabled);
    bool is_dry_run() const;

    // ========================================================================
    // Order Validation
    // ========================================================================

    // Validate order before submission
    bool validate_order(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        double limit_price,
        std::string& error_message
    ) const;

    // Check if order exceeds limits
    bool exceeds_limits(int quantity) const;

    // ========================================================================
    // Statistics and Reporting
    // ========================================================================

    struct OrderStats {
        int total_orders_placed;
        int total_orders_filled;
        int total_orders_cancelled;
        int total_orders_rejected;
        double total_volume_traded;
        double average_fill_time_ms;
        double fill_rate;  // Percentage of orders filled
    };

    OrderStats get_statistics() const;
    void reset_statistics();

    // ========================================================================
    // Callbacks
    // ========================================================================

    using OrderUpdateCallback = std::function<void(const types::Order&)>;
    using FillCallback = std::function<void(const types::Order&)>;

    void set_order_update_callback(OrderUpdateCallback callback);
    void set_fill_callback(FillCallback callback);

private:
    class Impl;
    std::unique_ptr<Impl> pimpl_;
};

// ============================================================================
// Order Validator
// ============================================================================

class OrderValidator {
public:
    // Validate contract
    static bool validate_contract(
        const types::OptionContract& contract,
        std::string& error
    );

    // Validate quantity
    static bool validate_quantity(int quantity, std::string& error);

    // Validate price
    static bool validate_price(double price, std::string& error);

    // Validate action
    static bool validate_action(
        types::OrderAction action,
        std::string& error
    );

    // Comprehensive validation
    static bool validate(
        const types::OptionContract& contract,
        types::OrderAction action,
        int quantity,
        double limit_price,
        std::vector<std::string>& errors
    );
};

// ============================================================================
// Order Builder
// ============================================================================

class OrderBuilder {
public:
    OrderBuilder();

    // Set contract
    OrderBuilder& contract(const types::OptionContract& c);

    // Set action
    OrderBuilder& action(types::OrderAction a);

    // Set quantity
    OrderBuilder& quantity(int q);

    // Set limit price
    OrderBuilder& limit_price(double price);

    // Set time in force
    OrderBuilder& time_in_force(types::TimeInForce tif);

    // Build the order
    types::Order build() const;

private:
    types::Order order_;
};

// ============================================================================
// Helper Functions
// ============================================================================

// Calculate total cost of an order
double calculate_order_cost(
    const types::Order& order,
    double commission_per_contract = 0.65
);

// Calculate expected slippage
double estimate_slippage(
    const types::OptionContract& contract,
    types::OrderAction action,
    int quantity,
    double bid,
    double ask
);

// Format order for logging
std::string format_order(const types::Order& order);

} // namespace order
