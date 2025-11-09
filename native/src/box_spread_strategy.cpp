// box_spread_strategy.cpp - Box spread strategy implementation (stub)
#include "box_spread_strategy.h"
#include <spdlog/spdlog.h>
#include <algorithm>
#include <numeric>

// NOTE FOR AUTOMATION AGENTS:
// This module encapsulates the decision logic around identifying, validating, and
// executing box spread arbitrage opportunities. It depends on three collaborators:
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

    // Calculate net debit and theoretical value
    spread.net_debit = BoxSpreadCalculator::calculate_net_debit(spread);
    spread.theoretical_value = BoxSpreadCalculator::calculate_theoretical_value(spread);
    spread.arbitrage_profit = BoxSpreadCalculator::calculate_max_profit(spread);
    spread.roi_percent = BoxSpreadCalculator::calculate_roi(spread);

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

bool BoxSpreadStrategy::is_profitable(const types::BoxSpreadLeg& spread) const {
    double profit = calculate_arbitrage_profit(spread);
    double roi = calculate_roi(spread);

    return profit >= pimpl_->params_.min_arbitrage_profit &&
           roi >= pimpl_->params_.min_roi_percent;
}

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

void BoxSpreadStrategy::update_parameters(const config::StrategyParams& params) {
    pimpl_->params_ = params;
    spdlog::info("Strategy parameters updated");
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

double BoxSpreadCalculator::calculate_theoretical_value(
    const types::BoxSpreadLeg& spread) {

    return spread.get_strike_width();
}

double BoxSpreadCalculator::calculate_net_debit(
    const types::BoxSpreadLeg& spread) {

    return spread.long_call_price - spread.short_call_price +
           spread.long_put_price - spread.short_put_price;
}

double BoxSpreadCalculator::calculate_max_profit(
    const types::BoxSpreadLeg& spread) {

    return spread.theoretical_value - spread.net_debit;
}

double BoxSpreadCalculator::calculate_max_loss(
    const types::BoxSpreadLeg& spread) {

    // Box spreads have limited risk
    return spread.net_debit < 0 ? -spread.net_debit : 0;
}

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

double BoxSpreadCalculator::calculate_total_cost(
    const types::BoxSpreadLeg& spread,
    double per_contract_fee) {

    return spread.net_debit + calculate_commission(spread, per_contract_fee);
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
