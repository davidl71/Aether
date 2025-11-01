// box_spread_strategy.h - Box spread arbitrage strategy
#pragma once

#include "types.h"
#include "config_manager.h"
#include "option_chain.h"
#include "tws_client.h"
#include "order_manager.h"
#include <vector>
#include <memory>
#include <optional>

namespace strategy {

// ============================================================================
// Box Spread Opportunity
// ============================================================================

struct BoxSpreadOpportunity {
    types::BoxSpreadLeg spread;
    double confidence_score;      // 0-100, based on liquidity, spread, etc.
    double expected_profit;
    double risk_adjusted_return;
    std::chrono::system_clock::time_point discovered_time;

    // Ranking metrics
    double liquidity_score;
    double execution_probability;

    bool is_actionable() const;
};

// ============================================================================
// Box Spread Strategy Class
// ============================================================================

class BoxSpreadStrategy {
public:
    BoxSpreadStrategy(
        tws::TWSClient* client,
        order::OrderManager* order_mgr,
        const config::StrategyParams& params
    );

    ~BoxSpreadStrategy();

    // Disable copy
    BoxSpreadStrategy(const BoxSpreadStrategy&) = delete;
    BoxSpreadStrategy& operator=(const BoxSpreadStrategy&) = delete;

    // ========================================================================
    // Main Strategy Loop
    // ========================================================================

    // Evaluate opportunities for all symbols
    void evaluate_opportunities();

    // Evaluate opportunities for specific symbol
    void evaluate_symbol(const std::string& symbol);

    // ========================================================================
    // Box Spread Detection
    // ========================================================================

    // Find all valid box spreads for a symbol
    std::vector<BoxSpreadOpportunity> find_box_spreads(
        const std::string& symbol
    );

    // Find box spreads in option chain
    std::vector<BoxSpreadOpportunity> find_box_spreads_in_chain(
        const option_chain::OptionChain& chain,
        double underlying_price
    );

    // Check if a specific combination is a valid box spread
    std::optional<BoxSpreadOpportunity> evaluate_box_spread(
        const types::OptionContract& long_call,
        const types::OptionContract& short_call,
        const types::OptionContract& long_put,
        const types::OptionContract& short_put,
        const option_chain::OptionChain& chain
    );

    // ========================================================================
    // Profitability Analysis
    // ========================================================================

    // Check if box spread meets profit criteria
    bool is_profitable(const types::BoxSpreadLeg& spread) const;

    // Calculate arbitrage profit
    double calculate_arbitrage_profit(const types::BoxSpreadLeg& spread) const;

    // Calculate ROI
    double calculate_roi(const types::BoxSpreadLeg& spread) const;

    // Calculate confidence score
    double calculate_confidence_score(
        const types::BoxSpreadLeg& spread,
        const option_chain::OptionChain& chain
    ) const;

    // ========================================================================
    // Execution
    // ========================================================================

    // Execute a box spread trade
    bool execute_box_spread(const BoxSpreadOpportunity& opportunity);

    // Close an existing box spread position
    bool close_box_spread(const std::string& position_id);

    // ========================================================================
    // Risk Management
    // ========================================================================

    // Check if spread is within risk limits
    bool within_risk_limits(const types::BoxSpreadLeg& spread) const;

    // Check if we can take a new position
    bool can_take_new_position() const;

    // Get current exposure
    double get_current_exposure() const;

    // Get maximum allowable exposure
    double get_max_exposure() const;

    // ========================================================================
    // Position Management
    // ========================================================================

    // Get all active box spread positions
    std::vector<types::Position> get_active_positions() const;

    // Monitor existing positions
    void monitor_positions();

    // Get position count
    int get_position_count() const;

    // ========================================================================
    // Configuration
    // ========================================================================

    // Update strategy parameters
    void update_parameters(const config::StrategyParams& params);

    // Get current parameters
    config::StrategyParams get_parameters() const;

    // ========================================================================
    // Statistics
    // ========================================================================

    struct StrategyStats {
        int total_opportunities_found;
        int total_trades_executed;
        int successful_trades;
        int failed_trades;
        double total_profit;
        double total_loss;
        double win_rate;
        double average_profit_per_trade;
        double sharpe_ratio;
        std::chrono::system_clock::time_point start_time;
    };

    StrategyStats get_statistics() const;
    void reset_statistics();

private:
    class Impl;
    std::unique_ptr<Impl> pimpl_;
};

// ============================================================================
// Box Spread Validator
// ============================================================================

class BoxSpreadValidator {
public:
    // Validate box spread structure
    static bool validate_structure(const types::BoxSpreadLeg& spread);

    // Validate strikes are consistent
    static bool validate_strikes(const types::BoxSpreadLeg& spread);

    // Validate expiries match
    static bool validate_expiries(const types::BoxSpreadLeg& spread);

    // Validate underlying symbols match
    static bool validate_symbols(const types::BoxSpreadLeg& spread);

    // Validate pricing makes sense
    static bool validate_pricing(const types::BoxSpreadLeg& spread);

    // Comprehensive validation
    static bool validate(
        const types::BoxSpreadLeg& spread,
        std::vector<std::string>& errors
    );
};

// ============================================================================
// Box Spread Calculator
// ============================================================================

class BoxSpreadCalculator {
public:
    // Calculate theoretical value of box spread
    static double calculate_theoretical_value(
        const types::BoxSpreadLeg& spread
    );

    // Calculate net debit/credit
    static double calculate_net_debit(const types::BoxSpreadLeg& spread);

    // Calculate maximum profit
    static double calculate_max_profit(const types::BoxSpreadLeg& spread);

    // Calculate maximum loss
    static double calculate_max_loss(const types::BoxSpreadLeg& spread);

    // Calculate ROI
    static double calculate_roi(const types::BoxSpreadLeg& spread);

    // Calculate break-even
    static double calculate_breakeven(const types::BoxSpreadLeg& spread);

    // Calculate commission costs
    static double calculate_commission(
        const types::BoxSpreadLeg& spread,
        double per_contract_fee = 0.65
    );

    // Calculate total cost including commission
    static double calculate_total_cost(
        const types::BoxSpreadLeg& spread,
        double per_contract_fee = 0.65
    );
};

// ============================================================================
// Helper Functions
// ============================================================================

// Sort opportunities by profitability
std::vector<BoxSpreadOpportunity> sort_opportunities_by_profit(
    std::vector<BoxSpreadOpportunity> opportunities
);

// Sort opportunities by confidence
std::vector<BoxSpreadOpportunity> sort_opportunities_by_confidence(
    std::vector<BoxSpreadOpportunity> opportunities
);

// Filter opportunities by minimum profit
std::vector<BoxSpreadOpportunity> filter_by_min_profit(
    const std::vector<BoxSpreadOpportunity>& opportunities,
    double min_profit
);

// Filter opportunities by minimum ROI
std::vector<BoxSpreadOpportunity> filter_by_min_roi(
    const std::vector<BoxSpreadOpportunity>& opportunities,
    double min_roi_percent
);

} // namespace strategy
