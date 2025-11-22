// box_spread_strategy.cpp - Box spread synthetic financing strategy implementation
#include "box_spread_strategy.h"
#include "config_manager.h"
#include <spdlog/spdlog.h>
#include <algorithm>
#include <numeric>
#include <cmath>

// NOTE FOR AUTOMATION AGENTS:
// This module encapsulates the decision logic around identifying, validating, and
// executing box spread synthetic financing opportunities (extracting risk-free rates
// for lending/borrowing). It depends on three collaborators:
//  * `tws::TWSClient` for live market data/event processing
//  * `order::OrderManager` for coordinated multi-leg order placement
//  * `config::StrategyParams` describing guardrails (liquidity, ROI, exposure)
// Production deployments are expected to replace several stubbed sections with live
// integrations; comments flag those extension points to avoid duplicating effort in
// this file.

namespace strategy {

// ============================================================================
// BoxSpreadOpportunity Implementation
// ============================================================================

bool BoxSpreadOpportunity::is_actionable() const {
    return confidence_score >= 50.0 &&
           expected_profit > 0 &&
           execution_probability >= 0.7;
}

// ============================================================================
// BoxSpreadStrategy::Impl
// ============================================================================

class BoxSpreadStrategy::Impl {
public:
    Impl(tws::TWSClient* client,
         order::OrderManager* order_mgr,
         const config::StrategyParams& params)
        : client_(client), order_mgr_(order_mgr), params_(params) {

        stats_.start_time = std::chrono::system_clock::now();
    }

    // Raw pointers mirror the lifetime managed by the caller (TUI entry point)
    tws::TWSClient* client_;
    order::OrderManager* order_mgr_;

    // Mutable strategy parameters and runtime statistics
    config::StrategyParams params_;
    StrategyStats stats_;

    // Local cache of open positions; populated by order callbacks in the full build
    std::vector<types::Position> positions_;
};

// ============================================================================
// BoxSpreadStrategy Implementation
// ============================================================================

BoxSpreadStrategy::BoxSpreadStrategy(
    tws::TWSClient* client,
    order::OrderManager* order_mgr,
    const config::StrategyParams& params)
    : pimpl_(std::make_unique<Impl>(client, order_mgr, params)) {

    spdlog::debug("BoxSpreadStrategy created");
}

BoxSpreadStrategy::~BoxSpreadStrategy() {
    spdlog::debug("BoxSpreadStrategy destroyed");
}

void BoxSpreadStrategy::evaluate_opportunities() {
    for (const auto& symbol : pimpl_->params_.symbols) {
        evaluate_symbol(symbol);
    }
}

void BoxSpreadStrategy::evaluate_symbol(const std::string& symbol) {
    spdlog::trace("Evaluating opportunities for {}", symbol);

    // NOTE: Full implementation would:
    // 1. Request option chain from TWS
    // 2. Find valid box spread combinations
    // 3. Calculate profitability
    // 4. Check risk limits
    // 5. Execute if profitable

    auto opportunities = find_box_spreads(symbol);
    pimpl_->stats_.total_opportunities_found += static_cast<int>(opportunities.size());

    if (!opportunities.empty()) {
        spdlog::info("Found {} box spread opportunities for {}",
                    opportunities.size(), symbol);

        // Execute the best opportunity
        for (const auto& opp : opportunities) {
            if (opp.is_actionable() && can_take_new_position()) {
                if (execute_box_spread(opp)) {
                    break;  // One trade per iteration
                }
            }
        }
    }
}

std::vector<BoxSpreadOpportunity> BoxSpreadStrategy::find_box_spreads(
    const std::string& symbol) {

    // NOTE: Full implementation would scan option chains
    // Stub keeps interface intact while allowing offline tests to progress
    std::vector<BoxSpreadOpportunity> opportunities;

    return opportunities;
}

std::vector<BoxSpreadOpportunity> BoxSpreadStrategy::find_box_spreads_in_chain(
    const option_chain::OptionChain& chain,
    double underlying_price) {

    std::vector<BoxSpreadOpportunity> opportunities;

    // Get all expiries in the DTE range
    int min_dte = pimpl_->params_.min_days_to_expiry;
    int max_dte = pimpl_->params_.max_days_to_expiry;
    auto expiries = chain.get_expiries_in_dte_range(min_dte, max_dte);

    spdlog::trace("Scanning {} expiries for box spreads", expiries.size());

    // For each expiry, find all strike combinations
    for (const auto& expiry : expiries) {
        auto expiry_chain_opt = chain.get_expiry_chain(expiry);
        if (!expiry_chain_opt.has_value()) {
            continue;
        }

        const auto& expiry_chain = expiry_chain_opt.value();
        auto strikes = expiry_chain.get_strikes();

        // Generate all strike pairs (K1, K2) where K1 < K2
        for (size_t i = 0; i < strikes.size(); ++i) {
            double strike_low = strikes[i];

            for (size_t j = i + 1; j < strikes.size(); ++j) {
                double strike_high = strikes[j];

                // Get all 4 legs for this strike pair
                auto long_call_opt = expiry_chain.get_option(strike_low, types::OptionType::Call);
                auto short_call_opt = expiry_chain.get_option(strike_high, types::OptionType::Call);
                auto long_put_opt = expiry_chain.get_option(strike_high, types::OptionType::Put);
                auto short_put_opt = expiry_chain.get_option(strike_low, types::OptionType::Put);

                // All 4 legs must exist
                if (!long_call_opt.has_value() || !short_call_opt.has_value() ||
                    !long_put_opt.has_value() || !short_put_opt.has_value()) {
                    continue;
                }

                // Check liquidity requirements
                int min_volume = pimpl_->params_.min_volume;
                int min_oi = pimpl_->params_.min_open_interest;
                if (!long_call_opt->meets_liquidity_requirements(min_volume, min_oi) ||
                    !short_call_opt->meets_liquidity_requirements(min_volume, min_oi) ||
                    !long_put_opt->meets_liquidity_requirements(min_volume, min_oi) ||
                    !short_put_opt->meets_liquidity_requirements(min_volume, min_oi)) {
                    continue;
                }

                // Evaluate this box spread combination
                auto opportunity_opt = evaluate_box_spread(
                    long_call_opt->contract,
                    short_call_opt->contract,
                    long_put_opt->contract,
                    short_put_opt->contract,
                    chain
                );

                if (opportunity_opt.has_value()) {
                    opportunities.push_back(opportunity_opt.value());
                }
            }
        }
    }

    // Sort by profitability (highest first)
    std::sort(opportunities.begin(), opportunities.end(),
        [](const BoxSpreadOpportunity& a, const BoxSpreadOpportunity& b) {
            return a.expected_profit > b.expected_profit;
        });

    spdlog::trace("Found {} box spread opportunities", opportunities.size());
    return opportunities;
}

std::optional<BoxSpreadOpportunity> BoxSpreadStrategy::evaluate_box_spread(
    const types::OptionContract& long_call,
    const types::OptionContract& short_call,
    const types::OptionContract& long_put,
    const types::OptionContract& short_put,
    const option_chain::OptionChain& chain) {

    // Get market data for all 4 legs
    auto expiry_chain_opt = chain.get_expiry_chain(long_call.expiry);
    if (!expiry_chain_opt.has_value()) {
        return std::nullopt;
    }

    const auto& expiry_chain = expiry_chain_opt.value();

    auto long_call_entry = expiry_chain.get_option(long_call.strike, types::OptionType::Call);
    auto short_call_entry = expiry_chain.get_option(short_call.strike, types::OptionType::Call);
    auto long_put_entry = expiry_chain.get_option(long_put.strike, types::OptionType::Put);
    auto short_put_entry = expiry_chain.get_option(short_put.strike, types::OptionType::Put);

    if (!long_call_entry.has_value() || !short_call_entry.has_value() ||
        !long_put_entry.has_value() || !short_put_entry.has_value()) {
        return std::nullopt;
    }

    // Check market data quality - all must have valid bid/ask
    if (!long_call_entry->is_valid() || !short_call_entry->is_valid() ||
        !long_put_entry->is_valid() || !short_put_entry->is_valid()) {
        return std::nullopt;
    }

    // Check bid/ask spread thresholds
    double max_spread = pimpl_->params_.max_bid_ask_spread;
    if (long_call_entry->market_data.get_spread() > max_spread ||
        short_call_entry->market_data.get_spread() > max_spread ||
        long_put_entry->market_data.get_spread() > max_spread ||
        short_put_entry->market_data.get_spread() > max_spread) {
        return std::nullopt;
    }

    // Build box spread leg
    types::BoxSpreadLeg spread;
    spread.long_call = long_call;
    spread.short_call = short_call;
    spread.long_put = long_put;
    spread.short_put = short_put;

    // Use mid prices for calculations
    spread.long_call_price = long_call_entry->market_data.get_mid_price();
    spread.short_call_price = short_call_entry->market_data.get_mid_price();
    spread.long_put_price = long_put_entry->market_data.get_mid_price();
    spread.short_put_price = short_put_entry->market_data.get_mid_price();

    // Calculate bid/ask spreads
    spread.long_call_bid_ask_spread = long_call_entry->market_data.get_spread();
    spread.short_call_bid_ask_spread = short_call_entry->market_data.get_spread();
    spread.long_put_bid_ask_spread = long_put_entry->market_data.get_spread();
    spread.short_put_bid_ask_spread = short_put_entry->market_data.get_spread();

    // Calculate net debit and theoretical value (using mid prices)
    spread.net_debit = BoxSpreadCalculator::calculate_net_debit(spread);
    spread.theoretical_value = BoxSpreadCalculator::calculate_theoretical_value(spread);
    spread.arbitrage_profit = BoxSpreadCalculator::calculate_max_profit(spread);
    spread.roi_percent = BoxSpreadCalculator::calculate_roi(spread);

    // Calculate BUY vs SELL disparity (using bid/ask prices)
    // This reveals intraday differences due to bid-ask spreads, put-call parity violations, etc.
    // BUY: Long legs use ASK, short legs use BID
    spread.buy_net_debit = BoxSpreadCalculator::calculate_buy_net_debit(
        spread,
        long_call_entry->market_data.ask,    // Long call: ASK
        short_call_entry->market_data.bid,   // Short call: BID
        long_put_entry->market_data.ask,     // Long put: ASK
        short_put_entry->market_data.bid     // Short put: BID
    );
    spread.buy_profit = spread.theoretical_value - spread.buy_net_debit;
    // Calculate implied rate for buying (using buy_net_debit as net cost)
    int days_to_expiry = spread.get_days_to_expiry();
    double strike_width = spread.get_strike_width();
    if (days_to_expiry > 0 && spread.buy_net_debit > 0) {
        // Implied rate when buying (borrowing scenario: pay now, receive at expiry)
        spread.buy_implied_rate = ((spread.buy_net_debit - strike_width) / strike_width) * (365.0 / days_to_expiry) * 100.0;
    } else {
        spread.buy_implied_rate = 0.0;
    }

    // SELL: Long legs use BID, short legs use ASK
    spread.sell_net_credit = BoxSpreadCalculator::calculate_sell_net_credit(
        spread,
        long_call_entry->market_data.bid,    // Long call: BID
        short_call_entry->market_data.ask,   // Short call: ASK
        long_put_entry->market_data.bid,     // Long put: BID
        short_put_entry->market_data.ask     // Short put: ASK
    );
    spread.sell_profit = spread.sell_net_credit - spread.theoretical_value;
    // Calculate implied rate for selling (using sell_net_credit as net received)
    if (days_to_expiry > 0 && spread.sell_net_credit > 0) {
        // Implied rate when selling (lending scenario: receive now, pay at expiry)
        spread.sell_implied_rate = ((strike_width - spread.sell_net_credit) / spread.sell_net_credit) * (365.0 / days_to_expiry) * 100.0;
    } else {
        spread.sell_implied_rate = 0.0;
    }

    // Calculate disparity and put-call parity violation
    spread.buy_sell_disparity = BoxSpreadCalculator::calculate_buy_sell_disparity(
        spread.buy_profit, spread.sell_profit
    );
    spread.put_call_parity_violation = BoxSpreadCalculator::calculate_put_call_parity_violation(
        spread, spread.buy_implied_rate, spread.sell_implied_rate
    );

    // Validate the spread
    std::vector<std::string> validation_errors;
    if (!BoxSpreadValidator::validate(spread, validation_errors)) {
        spdlog::trace("Box spread validation failed: {}", validation_errors[0]);
        return std::nullopt;
    }

    // Check profitability
    if (!is_profitable(spread)) {
        return std::nullopt;
    }

    // Build opportunity
    BoxSpreadOpportunity opportunity;
    opportunity.spread = spread;
    opportunity.expected_profit = spread.arbitrage_profit;
    opportunity.confidence_score = calculate_confidence_score(spread, chain);
    opportunity.risk_adjusted_return = spread.roi_percent;
    opportunity.discovered_time = std::chrono::system_clock::now();

    // Calculate liquidity score (average of all legs)
    opportunity.liquidity_score = (
        long_call_entry->liquidity_score +
        short_call_entry->liquidity_score +
        long_put_entry->liquidity_score +
        short_put_entry->liquidity_score
    ) / 4.0;

    // Estimate execution probability based on liquidity and spreads
    double avg_spread_pct = (
        long_call_entry->market_data.get_spread_percent() +
        short_call_entry->market_data.get_spread_percent() +
        long_put_entry->market_data.get_spread_percent() +
        short_put_entry->market_data.get_spread_percent()
    ) / 4.0;

    // Higher liquidity and lower spreads = higher execution probability
    opportunity.execution_probability = std::min(1.0,
        (opportunity.liquidity_score / 100.0) * (1.0 - std::min(avg_spread_pct / 10.0, 1.0))
    );

    return opportunity;
}

/// Check if a box spread meets profitability criteria.
///
/// Algorithm: A box spread is considered profitable if it meets both:
///   1. Minimum arbitrage profit threshold (absolute dollar amount)
///   2. Minimum ROI threshold (percentage return)
///
/// Expected behavior:
/// - Returns true if both thresholds are met
/// - Returns false if either threshold is not met
/// - Thresholds are configurable via StrategyParams
///
/// @param spread The box spread leg structure to evaluate
/// @return true if profitable, false otherwise
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
bool BoxSpreadStrategy::is_profitable(const types::BoxSpreadLeg& spread) const {
    double profit = calculate_arbitrage_profit(spread);
    double roi = calculate_roi(spread);

    return profit >= pimpl_->params_.min_arbitrage_profit &&
           roi >= pimpl_->params_.min_roi_percent;
}

/// Calculate arbitrage profit for a box spread (legacy function name).
///
/// NOTE: This function is used for synthetic financing rate extraction, not arbitrage.
/// The "profit" represents the basis for calculating implied interest rates.
///
/// Algorithm: Profit is the difference between theoretical value and net debit.
/// This is a wrapper around BoxSpreadCalculator::calculate_max_profit().
///
/// Formula: arbitrage_profit = theoretical_value - net_debit
///
/// For synthetic financing: This value is used to calculate implied interest rate:
///   implied_rate = ((net_debit - strike_width) / strike_width) × (365 / days_to_expiry) × 100%
///
/// @param spread The box spread leg structure
/// @return Profit basis (theoretical_value - net_debit) used for rate calculation
///
/// @see BoxSpreadCalculator::calculate_max_profit() for implementation details
/// @see RISK_FREE_RATE_METHODOLOGY.md for synthetic financing rate extraction
/// @todo Refactor to calculate_implied_interest_rate() for clarity
double BoxSpreadStrategy::calculate_arbitrage_profit(
    const types::BoxSpreadLeg& spread) const {

    return spread.theoretical_value - spread.net_debit;
}

double BoxSpreadStrategy::calculate_roi(const types::BoxSpreadLeg& spread) const {
    if (spread.net_debit > 0) {
        return (spread.arbitrage_profit / spread.net_debit) * 100.0;
    }
    return 0.0;
}

double BoxSpreadStrategy::calculate_confidence_score(
    const types::BoxSpreadLeg& spread,
    const option_chain::OptionChain& chain) const {

    // NOTE: Full implementation would consider:
    // - Liquidity scores
    // - Bid-ask spreads
    // - Volume and open interest
    // - Market conditions

    return 50.0;  // Stub
}

bool BoxSpreadStrategy::execute_box_spread(const BoxSpreadOpportunity& opportunity) {
    spdlog::info("Executing box spread: profit=${:.2f}, ROI={:.2f}%",
                opportunity.expected_profit,
                opportunity.spread.roi_percent);

    if (!within_risk_limits(opportunity.spread)) {
        spdlog::warn("Box spread exceeds risk limits, skipping");
        return false;
    }

    auto result = pimpl_->order_mgr_->place_box_spread(opportunity.spread);

    if (result.success) {
        pimpl_->stats_.total_trades_executed++;
        pimpl_->stats_.successful_trades++;
        spdlog::info("✓ Box spread executed successfully");
        return true;
    } else {
        pimpl_->stats_.failed_trades++;
        spdlog::error("✗ Box spread execution failed: {}", result.error_message);
        return false;
    }
}

bool BoxSpreadStrategy::close_box_spread(const std::string& position_id) {
    spdlog::info("Closing box spread position: {}", position_id);

    auto result = pimpl_->order_mgr_->close_box_spread(position_id);
    return result.success;
}

bool BoxSpreadStrategy::within_risk_limits(const types::BoxSpreadLeg& spread) const {
    double current_exposure = get_current_exposure();
    double position_cost = spread.net_debit * 100.0;  // Per contract

    // Check if adding this position would exceed limits
    return (current_exposure + position_cost) <= get_max_exposure();
}

bool BoxSpreadStrategy::can_take_new_position() const {
    int current_positions = get_position_count();
    // Max positions check would come from risk config
    return current_positions < 10;  // Stub
}

double BoxSpreadStrategy::get_current_exposure() const {
    return std::accumulate(
        pimpl_->positions_.begin(),
        pimpl_->positions_.end(),
        0.0,
        [](double total, const types::Position& pos) {
            return total + pos.get_market_value();
        }
    );
}

double BoxSpreadStrategy::get_max_exposure() const {
    return 50000.0;  // Would come from config
}

std::vector<types::Position> BoxSpreadStrategy::get_active_positions() const {
    return pimpl_->positions_;
}

void BoxSpreadStrategy::monitor_positions() {
    // NOTE: Full implementation would:
    // 1. Check P&L on positions
    // 2. Check for stop-loss triggers
    // 3. Check for profit targets
    // 4. Close positions near expiry
}

int BoxSpreadStrategy::get_position_count() const {
    return static_cast<int>(pimpl_->positions_.size());
}

// ============================================================================
// Lending/Borrowing Opportunities
// ============================================================================

std::vector<BoxSpreadOpportunity> BoxSpreadStrategy::find_lending_opportunities(
    const std::string& symbol,
    double benchmark_rate_percent,
    double min_spread_bps) {

    spdlog::info("Scanning for lending opportunities for {} vs benchmark rate {:.2f}% (min spread: {:.1f} bps)",
                 symbol, benchmark_rate_percent, min_spread_bps);

    // Find all box spreads for the symbol
    std::vector<BoxSpreadOpportunity> all_opportunities = find_box_spreads(symbol);

    // Filter and rank by rate competitiveness
    std::vector<BoxSpreadOpportunity> lending_opportunities;
    for (const auto& opp : all_opportunities) {
        if (beats_benchmark(opp.spread, benchmark_rate_percent, min_spread_bps)) {
            lending_opportunities.push_back(opp);
        }
    }

    // Sort by rate competitiveness (better rates first)
    std::sort(lending_opportunities.begin(), lending_opportunities.end(),
        [this, benchmark_rate_percent](const BoxSpreadOpportunity& a, const BoxSpreadOpportunity& b) {
            double spread_a = BoxSpreadCalculator::compare_to_benchmark(a.spread, benchmark_rate_percent);
            double spread_b = BoxSpreadCalculator::compare_to_benchmark(b.spread, benchmark_rate_percent);
            return spread_a > spread_b;  // Higher spread = better
        });

    spdlog::info("Found {} lending opportunities beating benchmark", lending_opportunities.size());
    return lending_opportunities;
}

bool BoxSpreadStrategy::beats_benchmark(
    const types::BoxSpreadLeg& spread,
    double benchmark_rate_percent,
    double min_spread_bps) const {

    double spread_bps = BoxSpreadCalculator::compare_to_benchmark(spread, benchmark_rate_percent);
    return spread_bps >= min_spread_bps;
}

// ============================================================================
// Intraday Position Improvement
// ============================================================================

std::optional<BoxSpreadStrategy::PositionImprovement> BoxSpreadStrategy::evaluate_position_improvement(
    const std::string& position_id) {

    // Find position
    auto pos_it = std::find_if(pimpl_->positions_.begin(), pimpl_->positions_.end(),
        [&position_id](const types::Position& pos) {
            // NOTE: Position ID matching would need to be implemented
            // For now, using symbol matching as placeholder
            return position_id == pos.contract.symbol;
        });

    if (pos_it == pimpl_->positions_.end()) {
        spdlog::warn("Position {} not found for improvement evaluation", position_id);
        return std::nullopt;
    }

    // NOTE: This is a stub implementation
    // Full implementation would:
    // 1. Get current market data for the position
    // 2. Calculate current implied rate
    // 3. Compare to entry implied rate
    // 4. Scan for better opportunities on different expirations
    // 5. Evaluate early close value vs holding to expiry

    PositionImprovement improvement;
    improvement.position_id = position_id;
    improvement.current_implied_rate = 0.0;  // Would calculate from current market data
    improvement.entry_implied_rate = 0.0;     // Would retrieve from position history
    improvement.improvement_bps = 0.0;
    improvement.has_improvement_opportunity = false;
    improvement.improvement_action = "none";
    improvement.early_close_beneficial = false;

    // TODO: Implement full evaluation logic
    spdlog::debug("Evaluating position improvement for {}", position_id);

    return improvement;
}

bool BoxSpreadStrategy::roll_box_spread(
    const std::string& position_id,
    const BoxSpreadOpportunity& new_opportunity) {

    spdlog::info("Rolling box spread position {} to new opportunity", position_id);

    // Close existing position
    if (!close_box_spread(position_id)) {
        spdlog::error("Failed to close existing position {} for roll", position_id);
        return false;
    }

    // Open new position
    if (!execute_box_spread(new_opportunity)) {
        spdlog::error("Failed to open new position for roll");
        return false;
    }

    spdlog::info("Successfully rolled position {} to new expiration", position_id);
    return true;
}

double BoxSpreadStrategy::calculate_early_close_value(
    const types::BoxSpreadLeg& spread) const {

    // Calculate current mark-to-market value
    // For early close, we would reverse the box spread (sell what we bought, buy what we sold)
    // Early close value = -current_net_debit (opposite of entry cost)

    // NOTE: This requires current market data for all legs
    // Stub implementation returns theoretical early close value

    double current_market_value = spread.theoretical_value;  // Would use current market prices
    double early_close_value = current_market_value - spread.net_debit;

    return early_close_value;
}

void BoxSpreadStrategy::monitor_positions_with_improvements(
    double improvement_threshold_bps) {

    spdlog::debug("Monitoring positions for improvement opportunities (threshold: {:.1f} bps)",
                  improvement_threshold_bps);

    for (const auto& position : pimpl_->positions_) {
        // NOTE: Position ID would be stored in Position structure
        std::string position_id = position.contract.symbol;  // Placeholder

        auto improvement_opt = evaluate_position_improvement(position_id);
        if (!improvement_opt.has_value()) {
            continue;
        }

        PositionImprovement improvement = improvement_opt.value();

        if (!improvement.has_improvement_opportunity) {
            continue;
        }

        if (improvement.improvement_bps < improvement_threshold_bps) {
            spdlog::trace("Position {} has improvement opportunity ({:.1f} bps) but below threshold",
                         position_id, improvement.improvement_bps);
            continue;
        }

        // Log improvement opportunity
        spdlog::info("Position {} improvement opportunity: {} (improvement: {:.1f} bps)",
                    position_id, improvement.improvement_action, improvement.improvement_bps);

        // Execute improvement action
        if (improvement.improvement_action == "roll" && improvement.roll_opportunity.has_value()) {
            if (improvement.roll_benefit_bps > improvement_threshold_bps) {
                spdlog::info("Executing roll for position {} (benefit: {:.1f} bps)",
                            position_id, improvement.roll_benefit_bps);
                roll_box_spread(position_id, improvement.roll_opportunity.value());
            }
        } else if (improvement.improvement_action == "close_early" && improvement.early_close_beneficial) {
            spdlog::info("Executing early close for position {} (value: ${:.2f} vs hold: ${:.2f})",
                        position_id, improvement.early_close_value, improvement.hold_to_expiry_value);
            close_box_spread(position_id);
        }
    }
}

void BoxSpreadStrategy::update_parameters(const config::StrategyParams& params) {
    pimpl_->params_ = params;
    spdlog::info("Strategy parameters updated");
}

// ============================================================================
// Yield Curve Analysis Implementation
// ============================================================================

types::YieldCurve BoxSpreadStrategy::build_yield_curve(
    const std::string& symbol,
    double strike_width,
    double benchmark_rate_percent,
    int min_dte,
    int max_dte) {

    spdlog::info("Building yield curve for {} with strike width {:.2f} (DTE: {}-{})",
                 symbol, strike_width, min_dte, max_dte);

    types::YieldCurve curve;
    curve.symbol = symbol;
    curve.strike_width = strike_width;
    curve.benchmark_rate = benchmark_rate_percent;
    curve.generated_time = std::chrono::system_clock::now();

    // Find all box spreads for the symbol
    std::vector<BoxSpreadOpportunity> all_opportunities = find_box_spreads(symbol);

    // Filter by strike width and DTE range
    for (const auto& opp : all_opportunities) {
        double opp_strike_width = opp.spread.get_strike_width();
        int dte = opp.spread.get_days_to_expiry();

        // Check if strike width matches (within tolerance) and DTE is in range
        const double strike_tolerance = 0.01;  // $0.01 tolerance
        if (std::abs(opp_strike_width - strike_width) <= strike_tolerance &&
            dte >= min_dte && dte <= max_dte) {

            // Calculate implied rate
            double implied_rate = BoxSpreadCalculator::calculate_implied_interest_rate(opp.spread);
            double effective_rate = BoxSpreadCalculator::calculate_effective_interest_rate(opp.spread);
            double spread_bps = BoxSpreadCalculator::compare_to_benchmark(opp.spread, benchmark_rate_percent);

            // Create yield curve point
            types::YieldCurvePoint point;
            point.symbol = symbol;
            point.days_to_expiry = dte;
            point.expiry_date = opp.spread.long_call.expiry;  // All legs have same expiry
            point.strike_width = opp_strike_width;
            point.implied_rate = implied_rate;
            point.effective_rate = effective_rate;
            point.net_debit = opp.spread.net_debit;
            point.spread_bps = spread_bps;
            point.liquidity_score = opp.liquidity_score;
            point.spread = opp.spread;
            point.timestamp = std::chrono::system_clock::now();

            curve.points.push_back(point);
        }
    }

    // Sort by days to expiry
    curve.sort_by_dte();

    spdlog::info("Built yield curve with {} data points", curve.points.size());
    return curve;
}

std::vector<types::YieldCurve> BoxSpreadStrategy::build_yield_curves_multi_symbol(
    const std::vector<std::string>& symbols,
    double strike_width,
    double benchmark_rate_percent,
    int min_dte,
    int max_dte) {

    spdlog::info("Building yield curves for {} symbols with strike width {:.2f}",
                 symbols.size(), strike_width);

    std::vector<types::YieldCurve> curves;
    curves.reserve(symbols.size());

    for (const auto& symbol : symbols) {
        types::YieldCurve curve = build_yield_curve(
            symbol, strike_width, benchmark_rate_percent, min_dte, max_dte);
        curves.push_back(curve);
    }

    spdlog::info("Built {} yield curves", curves.size());
    return curves;
}

BoxSpreadStrategy::YieldCurveComparison BoxSpreadStrategy::compare_yield_curves(
    const std::vector<types::YieldCurve>& curves) {

    YieldCurveComparison comparison;
    comparison.generated_time = std::chrono::system_clock::now();

    if (curves.empty()) {
        return comparison;
    }

    // Get strike width from first curve (all should be same)
    comparison.strike_width = curves[0].strike_width;

    // Collect all symbols
    for (const auto& curve : curves) {
        comparison.symbols.push_back(curve.symbol);
        comparison.rates_by_symbol[curve.symbol] = {};
        comparison.spreads_by_symbol[curve.symbol] = {};
    }

    // Find common DTE points across all curves
    if (curves.empty()) {
        return comparison;
    }

    // Start with DTE points from first curve
    std::vector<int> candidate_dte_points;
    for (const auto& point : curves[0].points) {
        candidate_dte_points.push_back(point.days_to_expiry);
    }

    // Filter to only DTE points that exist in all curves
    for (int dte : candidate_dte_points) {
        bool all_have_dte = true;
        for (const auto& curve : curves) {
            bool has_dte = false;
            for (const auto& point : curve.points) {
                if (point.days_to_expiry == dte) {
                    has_dte = true;
                    break;
                }
            }
            if (!has_dte) {
                all_have_dte = false;
                break;
            }
        }

        if (all_have_dte) {
            comparison.common_dte_points.push_back(dte);
        }
    }

    // Populate rates and spreads for each symbol at each common DTE
    for (const auto& curve : curves) {
        for (int dte : comparison.common_dte_points) {
            // Find point with matching DTE
            for (const auto& point : curve.points) {
                if (point.days_to_expiry == dte) {
                    comparison.rates_by_symbol[curve.symbol].push_back(point.implied_rate);
                    comparison.spreads_by_symbol[curve.symbol].push_back(point.spread_bps);
                    break;
                }
            }
        }
    }

    spdlog::info("Comparison created with {} common DTE points across {} symbols",
                 comparison.common_dte_points.size(), comparison.symbols.size());

    return comparison;
}

// ============================================================================
// Yield Curve Helper Methods
// ============================================================================

} // namespace strategy

namespace types {

bool YieldCurvePoint::is_valid() const {
    return !symbol.empty() &&
           days_to_expiry > 0 &&
           strike_width > 0 &&
           !expiry_date.empty();
}

bool YieldCurve::is_valid() const {
    return !symbol.empty() &&
           strike_width > 0 &&
           !points.empty();
}

void YieldCurve::sort_by_dte() {
    std::sort(points.begin(), points.end(),
        [](const YieldCurvePoint& a, const YieldCurvePoint& b) {
            return a.days_to_expiry < b.days_to_expiry;
        });
}

} // namespace types

namespace strategy {

bool BoxSpreadStrategy::YieldCurveComparison::is_valid() const {
    return strike_width > 0 &&
           !symbols.empty() &&
           !common_dte_points.empty();
}

config::StrategyParams BoxSpreadStrategy::get_parameters() const {
    return pimpl_->params_;
}

BoxSpreadStrategy::StrategyStats BoxSpreadStrategy::get_statistics() const {
    return pimpl_->stats_;
}

void BoxSpreadStrategy::reset_statistics() {
    pimpl_->stats_ = StrategyStats();
    pimpl_->stats_.start_time = std::chrono::system_clock::now();
    spdlog::info("Statistics reset");
}

// ============================================================================
// BoxSpreadValidator Implementation
// ============================================================================

bool BoxSpreadValidator::validate_structure(const types::BoxSpreadLeg& spread) {
    return spread.is_valid();
}

bool BoxSpreadValidator::validate_strikes(const types::BoxSpreadLeg& spread) {
    return spread.long_call.strike < spread.short_call.strike &&
           spread.short_put.strike == spread.long_call.strike &&
           spread.long_put.strike == spread.short_call.strike;
}

bool BoxSpreadValidator::validate_expiries(const types::BoxSpreadLeg& spread) {
    return spread.long_call.expiry == spread.short_call.expiry &&
           spread.long_call.expiry == spread.long_put.expiry &&
           spread.long_call.expiry == spread.short_put.expiry;
}

bool BoxSpreadValidator::validate_symbols(const types::BoxSpreadLeg& spread) {
    return spread.long_call.symbol == spread.short_call.symbol &&
           spread.long_call.symbol == spread.long_put.symbol &&
           spread.long_call.symbol == spread.short_put.symbol;
}

bool BoxSpreadValidator::validate_pricing(const types::BoxSpreadLeg& spread) {
    return spread.net_debit > 0 &&
           spread.theoretical_value > 0 &&
           spread.net_debit < spread.theoretical_value;
}

bool BoxSpreadValidator::validate(
    const types::BoxSpreadLeg& spread,
    std::vector<std::string>& errors) {

    bool valid = true;

    if (!validate_structure(spread)) {
        errors.push_back("Invalid spread structure");
        valid = false;
    }

    if (!validate_strikes(spread)) {
        errors.push_back("Invalid strike configuration");
        valid = false;
    }

    if (!validate_expiries(spread)) {
        errors.push_back("Expiries do not match");
        valid = false;
    }

    if (!validate_symbols(spread)) {
        errors.push_back("Symbols do not match");
        valid = false;
    }

    if (!validate_pricing(spread)) {
        errors.push_back("Invalid pricing");
        valid = false;
    }

    // Additional validation: Strike width must equal theoretical value
    double strike_width = spread.get_strike_width();
    const double tolerance = 0.01;  // Allow small floating point differences
    if (std::abs(spread.theoretical_value - strike_width) > tolerance) {
        errors.push_back("Theoretical value must equal strike width (theoretical=" +
                        std::to_string(spread.theoretical_value) +
                        ", strike_width=" + std::to_string(strike_width) + ")");
        valid = false;
    }

    // Validate bid/ask spreads are reasonable
    const double max_spread_threshold = 0.50;  // $0.50 max spread per leg
    if (spread.long_call_bid_ask_spread > max_spread_threshold) {
        errors.push_back("Long call bid/ask spread too wide: " +
                        std::to_string(spread.long_call_bid_ask_spread));
        valid = false;
    }
    if (spread.short_call_bid_ask_spread > max_spread_threshold) {
        errors.push_back("Short call bid/ask spread too wide: " +
                        std::to_string(spread.short_call_bid_ask_spread));
        valid = false;
    }
    if (spread.long_put_bid_ask_spread > max_spread_threshold) {
        errors.push_back("Long put bid/ask spread too wide: " +
                        std::to_string(spread.long_put_bid_ask_spread));
        valid = false;
    }
    if (spread.short_put_bid_ask_spread > max_spread_threshold) {
        errors.push_back("Short put bid/ask spread too wide: " +
                        std::to_string(spread.short_put_bid_ask_spread));
        valid = false;
    }

    // Validate prices are positive
    if (spread.long_call_price <= 0 || spread.short_call_price <= 0 ||
        spread.long_put_price <= 0 || spread.short_put_price <= 0) {
        errors.push_back("All option prices must be positive");
        valid = false;
    }

    return valid;
}

// ============================================================================
// BoxSpreadCalculator Implementation
// ============================================================================

/// Calculate theoretical value of a box spread.
///
/// Algorithm: The theoretical value of a box spread at expiration is always
/// equal to the strike width (K2 - K1), regardless of the underlying price.
///
/// Mathematical proof:
/// - If S > K2: Long call = (S - K1), Short call = -(S - K2), Long put = 0, Short put = 0
///   Net: (S - K1) - (S - K2) = K2 - K1
/// - If K1 < S <= K2: Long call = (S - K1), Short call = 0, Long put = (K2 - S), Short put = 0
///   Net: (S - K1) + (K2 - S) = K2 - K1
/// - If S <= K1: Long call = 0, Short call = 0, Long put = (K2 - S), Short put = -(K1 - S)
///   Net: (K2 - S) - (K1 - S) = K2 - K1
///
/// @param spread The box spread leg structure
/// @return Theoretical value (always equals strike width)
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double BoxSpreadCalculator::calculate_theoretical_value(
    const types::BoxSpreadLeg& spread) {

    return spread.get_strike_width();
}

/// Calculate net debit (cost to enter) a box spread.
///
/// Algorithm: Net debit is the total cost to enter the spread, calculated as:
///   net_debit = long_call_price - short_call_price + long_put_price - short_put_price
///
/// Explanation:
/// - Long positions (long_call, long_put) cost money (positive contribution)
/// - Short positions (short_call, short_put) generate income (negative contribution)
/// - Net debit is the sum of all costs minus all income
///
/// Expected behavior:
/// - Positive value: We pay to enter (net debit)
/// - Negative value: We receive to enter (net credit)
/// - Zero: Costless entry (rare, but possible)
///
/// @param spread The box spread leg structure with prices for each leg
/// @return Net debit (positive = we pay, negative = we receive)
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double BoxSpreadCalculator::calculate_net_debit(
    const types::BoxSpreadLeg& spread) {

    return spread.long_call_price - spread.short_call_price +
           spread.long_put_price - spread.short_put_price;
}

/// Calculate maximum arbitrage profit from a box spread.
///
/// Algorithm: Arbitrage profit is the difference between what we receive at
/// expiration (theoretical value) and what we pay to enter (net debit).
///
/// Formula: arbitrage_profit = theoretical_value - net_debit
///
/// Expected behavior:
/// - Positive profit: Arbitrage opportunity exists
/// - Zero profit: Break-even (no arbitrage)
/// - Negative profit: Loss (not an arbitrage opportunity)
///
/// Note: This is the maximum profit because box spreads always reach their
/// theoretical value at expiration, regardless of underlying price movement.
///
/// @param spread The box spread leg structure
/// @return Arbitrage profit (theoretical_value - net_debit)
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double BoxSpreadCalculator::calculate_max_profit(
    const types::BoxSpreadLeg& spread) {

    return spread.theoretical_value - spread.net_debit;
}

double BoxSpreadCalculator::calculate_max_loss(
    const types::BoxSpreadLeg& spread) {

    // Box spreads have limited risk
    return spread.net_debit < 0 ? -spread.net_debit : 0;
}

/// Calculate return on investment (ROI) percentage for a box spread.
///
/// Algorithm: ROI measures profitability relative to capital deployed.
///
/// Formula: roi_percent = (arbitrage_profit / net_debit) * 100.0
///
/// Expected behavior:
/// - Positive ROI: Profitable opportunity (arbitrage_profit > 0)
/// - Zero ROI: Break-even (arbitrage_profit = 0)
/// - Negative ROI: Loss (arbitrage_profit < 0)
/// - Returns 0.0 if net_debit <= 0 (division by zero protection)
///
/// @param spread The box spread leg structure
/// @return ROI as a percentage (0.0 if net_debit <= 0)
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double BoxSpreadCalculator::calculate_roi(const types::BoxSpreadLeg& spread) {
    if (spread.net_debit > 0) {
        return (calculate_max_profit(spread) / spread.net_debit) * 100.0;
    }
    return 0.0;
}

double BoxSpreadCalculator::calculate_breakeven(
    const types::BoxSpreadLeg& spread) {

    // Box spreads always reach max profit at expiry if held
    return spread.long_call.strike;
}

double BoxSpreadCalculator::calculate_commission(
    const types::BoxSpreadLeg& spread,
    double per_contract_fee) {

    return 4.0 * per_contract_fee;  // 4 legs
}

double BoxSpreadCalculator::calculate_commission_ibkr_pro(
    const types::BoxSpreadLeg& spread,
    const config::CommissionConfig& commission_config) {

    // Get effective rate based on tier
    double per_contract_rate = commission_config.get_effective_rate();

    // Calculate base commission (4 legs)
    double base_commission = 4.0 * per_contract_rate;

    // Apply minimum order fee if applicable
    // Note: Minimum fee is waived if monthly commissions > $30
    // For now, we'll assume minimum applies unless told otherwise
    if (base_commission < commission_config.minimum_order_fee) {
        base_commission = commission_config.minimum_order_fee;
    }

    return base_commission;
}

double BoxSpreadCalculator::calculate_total_cost(
    const types::BoxSpreadLeg& spread,
    double per_contract_fee) {

    return spread.net_debit + calculate_commission(spread, per_contract_fee);
}

double BoxSpreadCalculator::calculate_total_cost_ibkr_pro(
    const types::BoxSpreadLeg& spread,
    const config::CommissionConfig& commission_config) {

    return spread.net_debit + calculate_commission_ibkr_pro(spread, commission_config);
}

// ============================================================================
// Lending/Borrowing Rate Calculations
// ============================================================================

/// Calculate implied annual interest rate from a box spread.
///
/// Algorithm: Box spreads can be used as synthetic lending/borrowing instruments.
/// The implied rate represents the annualized interest rate embedded in the spread.
///
/// For Borrowing (net_debit > 0):
///   - We pay net_debit upfront
///   - We receive strike_width at expiration
///   - Formula: rate = ((net_debit - strike_width) / strike_width) * (365 / days_to_expiry) * 100
///
/// For Lending (net_debit < 0, i.e., net_credit > 0):
///   - We receive net_credit upfront
///   - We pay strike_width at expiration
///   - Formula: rate = ((strike_width - net_credit) / net_credit) * (365 / days_to_expiry) * 100
///
/// Expected behavior:
/// - Positive rate: Lending opportunity (we receive upfront, pay at expiry)
/// - Negative rate: Borrowing opportunity (we pay upfront, receive at expiry)
/// - Returns 0.0 if days_to_expiry <= 0 (division by zero protection)
/// - Returns 0.0 if net_debit == 0 (no rate embedded)
///
/// @param spread The box spread leg structure
/// @return Implied annual interest rate as a percentage (0.0 if invalid)
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double BoxSpreadCalculator::calculate_implied_interest_rate(
    const types::BoxSpreadLeg& spread) {

    double strike_width = spread.get_strike_width();
    int days_to_expiry = spread.get_days_to_expiry();

    if (days_to_expiry <= 0) {
        return 0.0;
    }

    double net_cost = spread.net_debit;
    double implied_rate = 0.0;

    if (net_cost > 0) {
        // Net debit (borrowing scenario): paying upfront, receiving strike width at expiry
        // Rate = ((net_debit - strike_width) / strike_width) * (365 / days_to_expiry) * 100
        implied_rate = ((net_cost - strike_width) / strike_width) * (365.0 / days_to_expiry) * 100.0;
    } else if (net_cost < 0) {
        // Net credit (lending scenario): receiving upfront, paying strike width at expiry
        // Rate = ((strike_width - net_credit) / net_credit) * (365 / days_to_expiry) * 100
        double net_credit = -net_cost;
        implied_rate = ((strike_width - net_credit) / net_credit) * (365.0 / days_to_expiry) * 100.0;
    }
    // If net_cost == 0, implied_rate remains 0.0

    return implied_rate;
}

double BoxSpreadCalculator::calculate_effective_interest_rate(
    const types::BoxSpreadLeg& spread,
    double per_contract_fee) {

    double strike_width = spread.get_strike_width();
    int days_to_expiry = spread.get_days_to_expiry();

    if (days_to_expiry <= 0) {
        return 0.0;
    }

    // Include transaction costs in rate calculation
    double commission = calculate_commission(spread, per_contract_fee);
    double total_cost = spread.net_debit + commission;
    double effective_rate = 0.0;

    if (total_cost > 0) {
        // Net debit after costs (borrowing)
        effective_rate = ((total_cost - strike_width) / strike_width) * (365.0 / days_to_expiry) * 100.0;
    } else if (total_cost < 0) {
        // Net credit after costs (lending)
        double net_credit_after_costs = -total_cost;
        effective_rate = ((strike_width - net_credit_after_costs) / net_credit_after_costs) * (365.0 / days_to_expiry) * 100.0;
    }

    return effective_rate;
}

// Overload using IBKR Pro commission config
double BoxSpreadCalculator::calculate_effective_interest_rate(
    const types::BoxSpreadLeg& spread,
    const config::CommissionConfig& commission_config) {

    double strike_width = spread.get_strike_width();
    int days_to_expiry = spread.get_days_to_expiry();

    if (days_to_expiry <= 0) {
        return 0.0;
    }

    // Include IBKR Pro transaction costs in rate calculation
    double commission = calculate_commission_ibkr_pro(spread, commission_config);
    double total_cost = spread.net_debit + commission;
    double effective_rate = 0.0;

    if (total_cost > 0) {
        // Net debit after costs (borrowing)
        effective_rate = ((total_cost - strike_width) / strike_width) * (365.0 / days_to_expiry) * 100.0;
    } else if (total_cost < 0) {
        // Net credit after costs (lending)
        double net_credit_after_costs = -total_cost;
        effective_rate = ((strike_width - net_credit_after_costs) / net_credit_after_costs) * (365.0 / days_to_expiry) * 100.0;
    }

    return effective_rate;
}

double BoxSpreadCalculator::compare_to_benchmark(
    const types::BoxSpreadLeg& spread,
    double benchmark_rate_percent,
    double per_contract_fee) {

    double effective_rate = calculate_effective_interest_rate(spread, per_contract_fee);

    // Return spread in basis points (1 basis point = 0.01%)
    // Positive = box spread beats benchmark (lower rate for borrowing, higher rate for lending)
    // Negative = benchmark beats box spread

    double spread_bps = (effective_rate - benchmark_rate_percent) * 100.0;

    // For borrowing: negative spread_bps is good (we're paying less)
    // For lending: positive spread_bps is good (we're earning more)
    // This calculation gives positive when box spread is better

    if (spread.net_debit > 0) {
        // Borrowing: invert so negative spread means better (lower rate)
        return -spread_bps;
    } else {
        // Lending: positive spread means better (higher rate)
        return spread_bps;
    }
}

// ============================================================================
// Buy vs Sell Calculations (Bid-Ask Disparity Analysis)
// ============================================================================

/// Calculate net debit for BUYING a box spread.
/// Uses ASK prices for long legs (what we pay) and BID prices for short legs (what we receive).
/// This represents the actual cost to buy the box spread at market prices.
double BoxSpreadCalculator::calculate_buy_net_debit(
    const types::BoxSpreadLeg& spread,
    double long_call_ask,
    double short_call_bid,
    double long_put_ask,
    double short_put_bid) {

    // BUY: Pay ASK for long positions, receive BID for short positions
    return long_call_ask - short_call_bid + long_put_ask - short_put_bid;
}

/// Calculate net credit for SELLING a box spread.
/// Uses BID prices for long legs (what we receive) and ASK prices for short legs (what we pay).
/// This represents the actual credit received from selling the box spread at market prices.
double BoxSpreadCalculator::calculate_sell_net_credit(
    const types::BoxSpreadLeg& spread,
    double long_call_bid,
    double short_call_ask,
    double long_put_bid,
    double short_put_ask) {

    // SELL: Receive BID for long positions, pay ASK for short positions
    return long_call_bid - short_call_ask + long_put_bid - short_put_ask;
}

/// Calculate the disparity between buying and selling profitability.
/// This reveals intraday differences due to:
/// - Bid-ask spread widths
/// - Put-call parity violations
/// - Liquidity imbalances
/// - Market maker inventory effects
/// - Order flow imbalances
double BoxSpreadCalculator::calculate_buy_sell_disparity(
    double buy_profit,
    double sell_profit) {

    // Disparity is the difference in profitability between buying and selling
    // Positive = buying is more profitable, Negative = selling is more profitable
    return buy_profit - sell_profit;
}

/// Calculate put-call parity violation.
/// Compares implied interest rates from call side vs put side.
/// Violations can occur due to:
/// - Dividend expectations
/// - Early exercise risk (American options)
/// - Interest rate changes
/// - Market microstructure effects
double BoxSpreadCalculator::calculate_put_call_parity_violation(
    const types::BoxSpreadLeg& spread,
    double call_implied_rate,
    double put_implied_rate) {

    // Put-call parity violation in basis points
    // Positive = call side implies higher rate, Negative = put side implies higher rate
    return (call_implied_rate - put_implied_rate) * 100.0;  // Convert % to bps
}

// ============================================================================
// Helper Functions
// ============================================================================

std::vector<BoxSpreadOpportunity> sort_opportunities_by_profit(
    std::vector<BoxSpreadOpportunity> opportunities) {

    std::sort(opportunities.begin(), opportunities.end(),
        [](const BoxSpreadOpportunity& a, const BoxSpreadOpportunity& b) {
            return a.expected_profit > b.expected_profit;
        });

    return opportunities;
}

std::vector<BoxSpreadOpportunity> sort_opportunities_by_confidence(
    std::vector<BoxSpreadOpportunity> opportunities) {

    std::sort(opportunities.begin(), opportunities.end(),
        [](const BoxSpreadOpportunity& a, const BoxSpreadOpportunity& b) {
            return a.confidence_score > b.confidence_score;
        });

    return opportunities;
}

std::vector<BoxSpreadOpportunity> filter_by_min_profit(
    const std::vector<BoxSpreadOpportunity>& opportunities,
    double min_profit) {

    std::vector<BoxSpreadOpportunity> filtered;
    std::copy_if(opportunities.begin(), opportunities.end(),
                 std::back_inserter(filtered),
                 [min_profit](const BoxSpreadOpportunity& opp) {
                     return opp.expected_profit >= min_profit;
                 });

    return filtered;
}

std::vector<BoxSpreadOpportunity> filter_by_min_roi(
    const std::vector<BoxSpreadOpportunity>& opportunities,
    double min_roi_percent) {

    std::vector<BoxSpreadOpportunity> filtered;
    std::copy_if(opportunities.begin(), opportunities.end(),
                 std::back_inserter(filtered),
                 [min_roi_percent](const BoxSpreadOpportunity& opp) {
                     return opp.spread.roi_percent >= min_roi_percent;
                 });

    return filtered;
}

} // namespace strategy
