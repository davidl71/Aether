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
    // Return value should be checked to verify execution succeeded
    [[nodiscard]] bool execute_box_spread(const BoxSpreadOpportunity& opportunity);

    // Close an existing box spread position
    // Return value should be checked to verify close succeeded
    [[nodiscard]] bool close_box_spread(const std::string& position_id);

    // ========================================================================
    // Risk Management
    // ========================================================================

    // Check if spread is within risk limits
    // Pure calculation function - no side effects
    bool within_risk_limits(const types::BoxSpreadLeg& spread) const __attribute__((pure));

    // Check if we can take a new position
    // Pure query function - no side effects
    bool can_take_new_position() const __attribute__((pure));

    // Get current exposure
    // Pure query function - no side effects
    double get_current_exposure() const __attribute__((pure));

    // Get maximum allowable exposure
    // Pure query function - no side effects
    double get_max_exposure() const __attribute__((pure));

    // ========================================================================
    // Position Management
    // ========================================================================

    // Get all active box spread positions
    // Pure query function - no side effects
    std::vector<types::Position> get_active_positions() const __attribute__((pure));

    // Monitor existing positions
    void monitor_positions();

    // Get position count
    // Pure query function - no side effects
    int get_position_count() const __attribute__((pure));

    // ========================================================================
    // Lending/Borrowing Opportunities
    // ========================================================================

    // Find box spread opportunities for lending/borrowing (rate-based)
    std::vector<BoxSpreadOpportunity> find_lending_opportunities(
        const std::string& symbol,
        double benchmark_rate_percent,
        double min_spread_bps = 50.0  // Minimum spread over benchmark in basis points
    );

    // Evaluate if box spread beats benchmark rate
    // Pure calculation function - no side effects
    bool beats_benchmark(
        const types::BoxSpreadLeg& spread,
        double benchmark_rate_percent,
        double min_spread_bps = 50.0
    ) const __attribute__((pure));

    // ========================================================================
    // Intraday Position Improvement
    // ========================================================================

    // Position improvement evaluation result
    struct PositionImprovement {
        std::string position_id;
        double current_implied_rate;
        double entry_implied_rate;
        double improvement_bps;  // Rate improvement in basis points
        bool has_improvement_opportunity;
        std::string improvement_action;  // "roll", "close_early", "partial_adjust"

        // Rolling opportunity details
        std::optional<BoxSpreadOpportunity> roll_opportunity;
        double roll_benefit_bps;  // Net benefit after transaction costs

        // Early close opportunity details
        double early_close_value;
        double hold_to_expiry_value;
        bool early_close_beneficial;
    };

    // Evaluate position for improvement opportunities
    std::optional<PositionImprovement> evaluate_position_improvement(
        const std::string& position_id
    );

    // Roll position to new expiration with better rate
    // Return value should be checked to verify roll succeeded
    [[nodiscard]] bool roll_box_spread(
        const std::string& position_id,
        const BoxSpreadOpportunity& new_opportunity
    );

    // Calculate value of early close vs holding to expiry
    double calculate_early_close_value(
        const types::BoxSpreadLeg& spread
    ) const;

    // Enhanced position monitoring with improvement detection
    void monitor_positions_with_improvements(
        double improvement_threshold_bps = 25.0  // Minimum improvement to trigger action
    );

    // ========================================================================
    // Yield Curve Analysis
    // ========================================================================

    // Build yield curve for a symbol with specified strike width
    // Scans all available expirations and calculates implied rates
    types::YieldCurve build_yield_curve(
        const std::string& symbol,
        double strike_width,
        double benchmark_rate_percent = 5.0,
        int min_dte = 7,    // Minimum days to expiry
        int max_dte = 180   // Maximum days to expiry
    );

    // Build yield curve for multiple symbols with same strike width
    // Allows comparison across ES, XSP, SPX, nanos, etc.
    std::vector<types::YieldCurve> build_yield_curves_multi_symbol(
        const std::vector<std::string>& symbols,
        double strike_width,
        double benchmark_rate_percent = 5.0,
        int min_dte = 7,
        int max_dte = 180
    );

    // Compare yield curves across symbols
    // Returns comparison data showing relative rates at each expiration
    struct YieldCurveComparison {
        double strike_width;
        std::vector<std::string> symbols;
        std::vector<int> common_dte_points;  // Days to expiry where all symbols have data
        std::map<std::string, std::vector<double>> rates_by_symbol;  // rates by symbol at each DTE
        std::map<std::string, std::vector<double>> spreads_by_symbol;  // spreads by symbol at each DTE
        std::chrono::system_clock::time_point generated_time;

        // Helper methods
        bool is_valid() const;
    };

    YieldCurveComparison compare_yield_curves(
        const std::vector<types::YieldCurve>& curves
    );

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

    // Buy vs Sell calculations (using bid/ask prices instead of mid)
    // These reveal intraday disparities due to bid-ask spreads, put-call parity violations, etc.

    // Calculate net debit for BUYING box spread (using ASK for long legs, BID for short legs)
    static double calculate_buy_net_debit(
        const types::BoxSpreadLeg& spread,
        double long_call_ask,
        double short_call_bid,
        double long_put_ask,
        double short_put_bid
    );

    // Calculate net credit for SELLING box spread (using BID for long legs, ASK for short legs)
    static double calculate_sell_net_credit(
        const types::BoxSpreadLeg& spread,
        double long_call_bid,
        double short_call_ask,
        double long_put_bid,
        double short_put_ask
    );

    // Calculate buy vs sell disparity (difference in profitability)
    static double calculate_buy_sell_disparity(
        double buy_profit,
        double sell_profit
    );

    // Calculate put-call parity violation (bps)
    // Compares implied rates from call side vs put side to detect violations
    static double calculate_put_call_parity_violation(
        const types::BoxSpreadLeg& spread,
        double call_implied_rate,
        double put_implied_rate
    );

    // Calculate commission using IBKR Pro config (with volume tiers)
    static double calculate_commission_ibkr_pro(
        const types::BoxSpreadLeg& spread,
        const config::CommissionConfig& commission_config
    );

    // Calculate total cost including commission
    static double calculate_total_cost(
        const types::BoxSpreadLeg& spread,
        double per_contract_fee = 0.65
    );

    // Calculate total cost including IBKR Pro commission
    static double calculate_total_cost_ibkr_pro(
        const types::BoxSpreadLeg& spread,
        const config::CommissionConfig& commission_config
    );

    // ========================================================================
    // Lending/Borrowing Rate Calculations
    // ========================================================================

    // Calculate implied annual interest rate (for lending/borrowing)
    // Returns annualized percentage rate
    // For lending (net credit > 0): rate = ((strike_width - net_credit) / net_credit) * (365 / dte) * 100
    // For borrowing (net debit > 0): rate = ((net_debit - strike_width) / strike_width) * (365 / dte) * 100
    static double calculate_implied_interest_rate(
        const types::BoxSpreadLeg& spread
    );

    // Calculate effective rate after transaction costs
    static double calculate_effective_interest_rate(
        const types::BoxSpreadLeg& spread,
        double per_contract_fee = 0.65
    );

    // Calculate effective rate using IBKR Pro commission config
    static double calculate_effective_interest_rate(
        const types::BoxSpreadLeg& spread,
        const config::CommissionConfig& commission_config
    );

    // Compare implied rate to benchmark rate
    // Returns spread in basis points (positive = box spread beats benchmark)
    static double compare_to_benchmark(
        const types::BoxSpreadLeg& spread,
        double benchmark_rate_percent,
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
