// order_manager.cpp - Order management implementation (stub)
#include "order_manager.h"
#include <spdlog/spdlog.h>
#include <spdlog/fmt/bundled/core.h>
#include <algorithm>
#include <map>
#include <set>

// NOTE FOR AUTOMATION AGENTS:
// The order manager coordinates interaction with IB's TWS API. It validates
// incoming requests, translates them into single- or multi-leg orders, and tracks
// execution statistics. Live builds rely on event callbacks supplied by TWS;
// stubbed sections indicate where polling/callback plumbing should land.

namespace order {

// ============================================================================
// MultiLegOrder Implementation
// ============================================================================

bool MultiLegOrder::is_complete() const {
    return legs_filled == static_cast<int>(legs.size());
}

bool MultiLegOrder::is_partially_filled() const {
    return legs_filled > 0 && legs_filled < static_cast<int>(legs.size());
}

// ============================================================================
// OrderManager::Impl
// ============================================================================

class OrderManager::Impl {
public:
    Impl(tws::TWSClient* client, bool dry_run)
        : client_(client), dry_run_(dry_run), max_order_size_(100) {
        // Initialize stats
        stats_ = OrderStats{};
        stats_.total_orders_placed = 0;
        stats_.total_orders_filled = 0;
        stats_.total_orders_cancelled = 0;
        stats_.total_orders_rejected = 0;
        stats_.executed_trades = 0;
        stats_.total_volume_traded = 0.0;
        stats_.average_fill_time_ms = 0.0;
        stats_.fill_rate = 0.0;
        stats_.efficiency_ratio = 0.0;
    }

    // Raw pointer references to avoid ownership cycles; lifetime managed by caller
    tws::TWSClient* client_;
    bool dry_run_;
    int max_order_size_;
    OrderStats stats_;
    std::map<std::string, MultiLegOrder> multi_leg_orders_;

    // Helper to update efficiency ratio and check for warnings
    void update_efficiency_ratio() {
        if (stats_.total_orders_placed > 0) {
            stats_.efficiency_ratio = static_cast<double>(stats_.executed_trades) /
                                      static_cast<double>(stats_.total_orders_placed);
        } else {
            stats_.efficiency_ratio = 0.0;
        }

        // Warn if efficiency ratio is low and we have enough orders
        if (stats_.total_orders_placed > 20 && stats_.efficiency_ratio < 0.05) {
            double efficiency_percent = stats_.efficiency_ratio * 100.0;
            spdlog::warn("⚠️  Low order efficiency ratio: {:.2f}% ({} executed / {} placed)",
                        efficiency_percent,
                        stats_.executed_trades,
                        stats_.total_orders_placed);
            spdlog::warn("   Consider reviewing order parameters, prices, or market conditions");
        }
    }

    // Optional callbacks invoked by the full implementation when fills/status change
    OrderUpdateCallback order_update_callback_;
    FillCallback fill_callback_;
};

// ============================================================================
// OrderManager Implementation
// ============================================================================

OrderManager::OrderManager(tws::TWSClient* client, bool dry_run)
    : pimpl_(std::make_unique<Impl>(client, dry_run)) {

    spdlog::debug("OrderManager created (dry_run={})", dry_run);
}

OrderManager::~OrderManager() {
    spdlog::debug("OrderManager destroyed");
}

ExecutionResult OrderManager::place_order(
    const types::OptionContract& contract,
    types::OrderAction action,
    int quantity,
    double limit_price,
    types::TimeInForce tif) {

    ExecutionResult result;
    result.execution_time = std::chrono::system_clock::now();

    // Validate order
    std::string error_msg;
    if (!validate_order(contract, action, quantity, limit_price, error_msg)) {
        result.success = false;
        result.error_message = error_msg;
        return result;
    }

    if (pimpl_->dry_run_) {
        spdlog::info("[DRY RUN] Would place order: {} {} {} @ {}",
                    types::order_action_to_string(action),
                    quantity, contract.to_string(),
                    limit_price > 0 ? std::to_string(limit_price) : "MKT");

        result.success = true;
        result.order_ids.push_back(999);  // Dummy order ID
        pimpl_->stats_.total_orders_placed++;  // Track stats in dry-run too
        return result;
    }

    // Place order through TWS
    int order_id = pimpl_->client_->place_order(
        contract, action, quantity, limit_price, tif
    );

    result.success = true;
    result.order_ids.push_back(order_id);

    pimpl_->stats_.total_orders_placed++;

    return result;
}

bool OrderManager::cancel_order(int order_id) {
    spdlog::info("Cancelling order #{}", order_id);

    if (pimpl_->dry_run_) {
        spdlog::info("[DRY RUN] Would cancel order #{}", order_id);
        return true;
    }

    pimpl_->client_->cancel_order(order_id);
    pimpl_->stats_.total_orders_cancelled++;

    return true;
}

void OrderManager::cancel_all_orders() {
    spdlog::info("Cancelling all orders");

    if (pimpl_->dry_run_) {
        spdlog::info("[DRY RUN] Would cancel all orders");
        return;
    }

    pimpl_->client_->cancel_all_orders();
}

std::optional<types::Order> OrderManager::get_order_status(int order_id) const {
    return pimpl_->client_->get_order(order_id);
}

ExecutionResult OrderManager::place_box_spread(
    const types::BoxSpreadLeg& spread,
    const std::string& strategy_id) {

    ExecutionResult result;
    result.execution_time = std::chrono::system_clock::now();

    spdlog::info("Placing box spread: {} {:.0f}/{:.0f} {}",
                spread.long_call.symbol,
                spread.long_call.strike,
                spread.short_call.strike,
                spread.long_call.expiry);

    if (pimpl_->dry_run_) {
        spdlog::info("[DRY RUN] Would place box spread (4 legs)");
        result.success = true;
        result.order_ids = {1001, 1002, 1003, 1004};
        pimpl_->stats_.total_orders_placed += 4;
        return result;
    }

    // Try combo order first (atomic execution) if contract IDs are available
    // For now, we'll use individual orders as fallback since contract IDs require reqContractDetails
    // TODO: Add contract details lookup to get conIds for combo orders
    bool use_combo_order = false;  // Set to true when contract IDs are available

    if (use_combo_order) {
        // Prepare combo order data
        std::vector<types::OptionContract> contracts = {
            spread.long_call,
            spread.short_call,
            spread.long_put,
            spread.short_put
        };
        std::vector<types::OrderAction> actions = {
            types::OrderAction::Buy,
            types::OrderAction::Sell,
            types::OrderAction::Buy,
            types::OrderAction::Sell
        };
        std::vector<int> quantities = {1, 1, 1, 1};
        std::vector<double> limit_prices = {
            spread.long_call_price,
            spread.short_call_price,
            spread.long_put_price,
            spread.short_put_price
        };

        // TODO: Get contract IDs (conId) from reqContractDetails
        // For now, use placeholder - combo order will fail without real IDs
        std::vector<long> contract_ids = {0, 0, 0, 0};  // Placeholder

        int combo_order_id = pimpl_->client_->place_combo_order(
            contracts, actions, quantities, contract_ids, limit_prices
        );

        if (combo_order_id > 0) {
            spdlog::info("✓ Box spread placed as combo order #{} (atomic execution)", combo_order_id);
            result.success = true;
            result.order_ids = {combo_order_id};
            result.total_cost = spread.net_debit * 100.0;
            pimpl_->stats_.total_orders_placed += 1;  // Single combo order counts as 1
            pimpl_->update_efficiency_ratio();
            return result;
        } else {
            spdlog::warn("Combo order failed, falling back to individual orders");
        }
    }

    // Fallback: Place all 4 legs as individual orders (with rollback on failure)
    std::vector<int> order_ids;
    std::vector<int> successful_order_ids;

    // Leg 1: Long call
    int id1 = pimpl_->client_->place_order(
        spread.long_call,
        types::OrderAction::Buy,
        1,
        spread.long_call_price
    );
    order_ids.push_back(id1);
    successful_order_ids.push_back(id1);

    // Leg 2: Short call
    int id2 = pimpl_->client_->place_order(
        spread.short_call,
        types::OrderAction::Sell,
        1,
        spread.short_call_price
    );
    order_ids.push_back(id2);
    successful_order_ids.push_back(id2);

    // Leg 3: Long put
    int id3 = pimpl_->client_->place_order(
        spread.long_put,
        types::OrderAction::Buy,
        1,
        spread.long_put_price
    );
    order_ids.push_back(id3);
    successful_order_ids.push_back(id3);

    // Leg 4: Short put
    int id4 = pimpl_->client_->place_order(
        spread.short_put,
        types::OrderAction::Sell,
        1,
        spread.short_put_price
    );
    order_ids.push_back(id4);
    successful_order_ids.push_back(id4);

    // Monitor order statuses for a short period to detect immediate rejections
    // In a real implementation, this would use callbacks or polling
    // For now, we'll check immediately and implement rollback if any fail

    // Check if any orders were immediately rejected
    bool has_failures = false;
    for (int order_id : order_ids) {
        auto order_status = pimpl_->client_->get_order(order_id);
        if (order_status.has_value()) {
            if (order_status->status == types::OrderStatus::Rejected ||
                order_status->status == types::OrderStatus::Error) {
                spdlog::warn("Order #{} was rejected: {}", order_id, order_status->status_message);
                has_failures = true;
                // Remove from successful list
                successful_order_ids.erase(
                    std::remove(successful_order_ids.begin(), successful_order_ids.end(), order_id),
                    successful_order_ids.end()
                );
            }
        }
    }

    // If any order failed, cancel all remaining orders (rollback)
    if (has_failures) {
        spdlog::warn("Box spread execution failed - rolling back remaining orders");
        for (int order_id : successful_order_ids) {
            spdlog::info("Cancelling order #{} due to rollback", order_id);
            pimpl_->client_->cancel_order(order_id);
        }

        result.success = false;
        result.error_message = "One or more legs failed - all orders cancelled";
        result.order_ids = order_ids;  // Include all IDs for tracking
        pimpl_->stats_.total_orders_cancelled += static_cast<int>(successful_order_ids.size());
        return result;
    }

    // All orders placed successfully
    result.success = true;
    result.order_ids = order_ids;
    result.total_cost = spread.net_debit * 100.0;

    pimpl_->stats_.total_orders_placed += 4;
    pimpl_->update_efficiency_ratio();

    // Store multi-leg order for tracking
    if (!strategy_id.empty()) {
        MultiLegOrder multi_leg;
        multi_leg.strategy_id = strategy_id;
        multi_leg.status = types::OrderStatus::Submitted;
        multi_leg.created_time = std::chrono::system_clock::now();
        multi_leg.legs_filled = 0;
        multi_leg.total_cost = result.total_cost;

        // Get order objects for tracking
        for (int order_id : order_ids) {
            auto order_opt = pimpl_->client_->get_order(order_id);
            if (order_opt.has_value()) {
                multi_leg.legs.push_back(order_opt.value());
            }
        }

        pimpl_->multi_leg_orders_[strategy_id] = multi_leg;
    }

    spdlog::info("✓ Box spread orders placed: {}", fmt::join(order_ids, ", "));
    return result;
}

ExecutionResult OrderManager::close_box_spread(const std::string& strategy_id) {
    ExecutionResult result;
    result.execution_time = std::chrono::system_clock::now();

    spdlog::info("Closing box spread position: {}", strategy_id);

    if (pimpl_->dry_run_) {
        spdlog::info("[DRY RUN] Would close box spread");
        result.success = true;
        return result;
    }

    // NOTE: Full implementation would reverse all legs

    result.success = true;
    return result;
}

std::optional<MultiLegOrder> OrderManager::get_multi_leg_order(
    const std::string& strategy_id) const {

    auto it = pimpl_->multi_leg_orders_.find(strategy_id);
    if (it != pimpl_->multi_leg_orders_.end()) {
        return it->second;
    }

    return std::nullopt;
}

void OrderManager::update() {
    // Check all active orders for status changes
    auto active_orders = get_active_orders();
    for (const auto& order : active_orders) {
        // Track fills
        if (order.status == types::OrderStatus::Filled) {
            track_order_fill(order.order_id);
        }
    }
}

void OrderManager::track_order_fill(int order_id) {
    // Check if we've already tracked this fill
    // Static variable intentionally persists across calls to track fills
    // cppcheck-suppress[staticVariable]
    static std::set<int> tracked_fills;

    if (tracked_fills.count(order_id) > 0) {
        return;  // Already tracked
    }

    auto order_opt = pimpl_->client_->get_order(order_id);
    if (order_opt.has_value() && order_opt->status == types::OrderStatus::Filled) {
        pimpl_->stats_.total_orders_filled++;
        pimpl_->stats_.executed_trades++;
        tracked_fills.insert(order_id);
        pimpl_->update_efficiency_ratio();

        double efficiency_percent = pimpl_->stats_.efficiency_ratio * 100.0;
        spdlog::debug("Tracked order fill: #{} (executed_trades: {}, efficiency: {:.2f}%)",
                     order_id,
                     pimpl_->stats_.executed_trades,
                     efficiency_percent);
    }
}

std::vector<types::Order> OrderManager::get_active_orders() const {
    return pimpl_->client_->get_active_orders();
}

std::vector<types::Order> OrderManager::get_strategy_orders(
    const std::string& strategy_id) const {

    std::vector<types::Order> orders;

    auto it = pimpl_->multi_leg_orders_.find(strategy_id);
    if (it != pimpl_->multi_leg_orders_.end()) {
        orders = it->second.legs;
    }

    return orders;
}

bool OrderManager::are_all_legs_filled(const std::string& strategy_id) const {
    auto multi_leg = get_multi_leg_order(strategy_id);
    if (multi_leg.has_value()) {
        return multi_leg->is_complete();
    }
    return false;
}

ExecutionResult OrderManager::execute_ioc(
    const types::OptionContract& contract,
    types::OrderAction action,
    int quantity,
    double limit_price) {

    return place_order(contract, action, quantity, limit_price,
                      types::TimeInForce::IOC);
}

ExecutionResult OrderManager::execute_fok(
    const types::OptionContract& contract,
    types::OrderAction action,
    int quantity,
    double limit_price) {

    return place_order(contract, action, quantity, limit_price,
                      types::TimeInForce::FOK);
}

ExecutionResult OrderManager::execute_twap(
    const types::OptionContract& contract,
    types::OrderAction action,
    int quantity,
    int duration_seconds) {

    // NOTE: TWAP implementation would split order over time
    ExecutionResult result;
    result.success = false;
    result.error_message = "TWAP not implemented";
    return result;
}

std::optional<double> OrderManager::get_best_price(
    const types::OptionContract& contract,
    types::OrderAction action) const {

    // NOTE: Would query current market data
    return std::nullopt;
}

double OrderManager::estimate_fill_probability(
    const types::OptionContract& contract,
    types::OrderAction action,
    double limit_price) const {

    // NOTE: Would analyze order book and historical fills
    return 0.5;  // Stub
}

void OrderManager::set_max_order_size(int max_contracts) {
    pimpl_->max_order_size_ = max_contracts;
    spdlog::info("Max order size set to {}", max_contracts);
}

void OrderManager::set_max_orders_per_second(int max_rate) {
    spdlog::info("Max orders/second set to {}", max_rate);
    // NOTE: Would implement rate limiting
}

void OrderManager::set_dry_run(bool enabled) {
    pimpl_->dry_run_ = enabled;
    spdlog::info("Dry run mode: {}", enabled ? "enabled" : "disabled");
}

bool OrderManager::is_dry_run() const {
    return pimpl_->dry_run_;
}

bool OrderManager::validate_order(
    const types::OptionContract& contract,
    types::OrderAction action,
    int quantity,
    double limit_price,
    std::string& error_message) const {

    if (!contract.is_valid()) {
        error_message = "Invalid contract";
        return false;
    }

    if (quantity <= 0) {
        error_message = "Quantity must be positive";
        return false;
    }

    if (exceeds_limits(quantity)) {
        error_message = "Order size exceeds limits";
        return false;
    }

    if (limit_price < 0) {
        error_message = "Limit price cannot be negative";
        return false;
    }

    return true;
}

bool OrderManager::exceeds_limits(int quantity) const {
    return quantity > pimpl_->max_order_size_;
}

OrderManager::OrderStats OrderManager::get_statistics() const {
    // Update efficiency ratio before returning
    pimpl_->update_efficiency_ratio();
    return pimpl_->stats_;
}

void OrderManager::reset_statistics() {
    pimpl_->stats_ = OrderStats();
    spdlog::info("Order statistics reset");
}

void OrderManager::set_order_update_callback(OrderUpdateCallback callback) {
    pimpl_->order_update_callback_ = callback;
}

void OrderManager::set_fill_callback(FillCallback callback) {
    pimpl_->fill_callback_ = callback;
}

// ============================================================================
// OrderValidator Implementation
// ============================================================================

bool OrderValidator::validate_contract(
    const types::OptionContract& contract,
    std::string& error) {

    if (contract.symbol.empty()) {
        error = "Symbol cannot be empty";
        return false;
    }

    if (contract.strike <= 0) {
        error = "Strike must be positive";
        return false;
    }

    if (contract.expiry.empty()) {
        error = "Expiry cannot be empty";
        return false;
    }

    return true;
}

bool OrderValidator::validate_quantity(int quantity, std::string& error) {
    if (quantity <= 0) {
        error = "Quantity must be positive";
        return false;
    }

    if (quantity > 1000) {
        error = "Quantity exceeds maximum";
        return false;
    }

    return true;
}

bool OrderValidator::validate_price(double price, std::string& error) {
    if (price < 0) {
        error = "Price cannot be negative";
        return false;
    }

    return true;
}

bool OrderValidator::validate_action(
    types::OrderAction action,
    std::string& error) {

    // Always valid for now
    return true;
}

bool OrderValidator::validate(
    const types::OptionContract& contract,
    types::OrderAction action,
    int quantity,
    double limit_price,
    std::vector<std::string>& errors) {

    bool valid = true;
    std::string error;

    if (!validate_contract(contract, error)) {
        errors.push_back(error);
        valid = false;
    }

    if (!validate_quantity(quantity, error)) {
        errors.push_back(error);
        valid = false;
    }

    if (!validate_price(limit_price, error)) {
        errors.push_back(error);
        valid = false;
    }

    if (!validate_action(action, error)) {
        errors.push_back(error);
        valid = false;
    }

    return valid;
}

// ============================================================================
// OrderBuilder Implementation
// ============================================================================

OrderBuilder::OrderBuilder() {
    order_.order_id = 0;
    order_.quantity = 0;
    order_.limit_price = 0.0;
    order_.tif = types::TimeInForce::Day;
    order_.status = types::OrderStatus::Pending;
}

OrderBuilder& OrderBuilder::contract(const types::OptionContract& c) {
    order_.contract = c;
    return *this;
}

OrderBuilder& OrderBuilder::action(types::OrderAction a) {
    order_.action = a;
    return *this;
}

OrderBuilder& OrderBuilder::quantity(int q) {
    order_.quantity = q;
    return *this;
}

OrderBuilder& OrderBuilder::limit_price(double price) {
    order_.limit_price = price;
    return *this;
}

OrderBuilder& OrderBuilder::time_in_force(types::TimeInForce tif) {
    order_.tif = tif;
    return *this;
}

types::Order OrderBuilder::build() const {
    return order_;
}

// ============================================================================
// Helper Functions
// ============================================================================

double calculate_order_cost(
    const types::Order& order,
    double commission_per_contract) {

    double contract_cost = order.limit_price * 100.0 * static_cast<double>(order.quantity);
    double commission = commission_per_contract * static_cast<double>(order.quantity);

    return contract_cost + commission;
}

double estimate_slippage(
    const types::OptionContract& contract,
    types::OrderAction action,
    int quantity,
    double bid,
    double ask) {

    double spread = ask - bid;

    // Estimate slippage as a fraction of the spread
    return spread * 0.5;  // Simplified estimate
}

std::string format_order(const types::Order& order) {
    return "Order #" + std::to_string(order.order_id) + ": " +
           types::order_action_to_string(order.action) + " " +
           std::to_string(order.quantity) + " " +
           order.contract.to_string() + " @ " +
           (order.limit_price > 0 ? std::to_string(order.limit_price) : "MKT");
}

} // namespace order
