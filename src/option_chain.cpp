// option_chain.cpp - Option chain implementation (stub)
#include "option_chain.h"
#include <spdlog/spdlog.h>
#include <cmath>
#include <algorithm>
#include <numeric>

namespace option_chain {

// ============================================================================
// OptionChainEntry Implementation
// ============================================================================

bool OptionChainEntry::is_valid() const {
    return contract.is_valid() &&
           market_data.bid > 0 &&
           market_data.ask > 0 &&
           market_data.ask >= market_data.bid;
}

bool OptionChainEntry::meets_liquidity_requirements(int min_volume, int min_oi) const {
    return volume >= min_volume && open_interest >= min_oi;
}

// ============================================================================
// StrikeChain Implementation
// ============================================================================

double StrikeChain::get_call_iv() const {
    if (call.has_value() && call->market_data.implied_volatility.has_value()) {
        return call->market_data.implied_volatility.value();
    }
    return 0.0;
}

double StrikeChain::get_put_iv() const {
    if (put.has_value() && put->market_data.implied_volatility.has_value()) {
        return put->market_data.implied_volatility.value();
    }
    return 0.0;
}

double StrikeChain::get_iv_skew() const {
    return get_put_iv() - get_call_iv();
}

// ============================================================================
// ExpiryChain Implementation
// ============================================================================

ExpiryChain::ExpiryChain(const std::string& symbol, const std::string& expiry)
    : symbol_(symbol), expiry_(expiry) {
}

void ExpiryChain::add_option(const OptionChainEntry& entry) {
    double strike = entry.contract.strike;

    auto& strike_chain = strikes_[strike];
    strike_chain.strike = strike;

    if (entry.contract.type == types::OptionType::Call) {
        strike_chain.call = entry;
    } else {
        strike_chain.put = entry;
    }
}

int ExpiryChain::get_days_to_expiry() const {
    // NOTE: Proper DTE calculation would go here
    return 30;  // Stub
}

std::vector<double> ExpiryChain::get_strikes() const {
    std::vector<double> strikes;
    for (const auto& [strike, _] : strikes_) {
        strikes.push_back(strike);
    }
    std::sort(strikes.begin(), strikes.end());
    return strikes;
}

std::optional<StrikeChain> ExpiryChain::get_strike_chain(double strike) const {
    auto it = strikes_.find(strike);
    if (it != strikes_.end()) {
        return it->second;
    }
    return std::nullopt;
}

std::optional<OptionChainEntry> ExpiryChain::get_option(
    double strike,
    types::OptionType type) const {

    auto it = strikes_.find(strike);
    if (it != strikes_.end()) {
        if (type == types::OptionType::Call) {
            return it->second.call;
        } else {
            return it->second.put;
        }
    }
    return std::nullopt;
}

std::vector<double> ExpiryChain::get_strikes_in_range(
    double min_strike,
    double max_strike) const {

    std::vector<double> strikes;
    for (const auto& [strike, _] : strikes_) {
        if (strike >= min_strike && strike <= max_strike) {
            strikes.push_back(strike);
        }
    }
    std::sort(strikes.begin(), strikes.end());
    return strikes;
}

std::optional<double> ExpiryChain::find_atm_strike(double underlying_price) const {
    if (strikes_.empty()) {
        return std::nullopt;
    }

    double closest_strike = 0;
    double min_diff = std::numeric_limits<double>::max();

    for (const auto& [strike, _] : strikes_) {
        double diff = std::abs(strike - underlying_price);
        if (diff < min_diff) {
            min_diff = diff;
            closest_strike = strike;
        }
    }

    return closest_strike;
}

std::vector<OptionChainEntry> ExpiryChain::get_all_options() const {
    std::vector<OptionChainEntry> options;
    for (const auto& [_, chain] : strikes_) {
        if (chain.call.has_value()) {
            options.push_back(chain.call.value());
        }
        if (chain.put.has_value()) {
            options.push_back(chain.put.value());
        }
    }
    return options;
}

std::vector<OptionChainEntry> ExpiryChain::get_calls() const {
    std::vector<OptionChainEntry> calls;
    for (const auto& [_, chain] : strikes_) {
        if (chain.call.has_value()) {
            calls.push_back(chain.call.value());
        }
    }
    return calls;
}

std::vector<OptionChainEntry> ExpiryChain::get_puts() const {
    std::vector<OptionChainEntry> puts;
    for (const auto& [_, chain] : strikes_) {
        if (chain.put.has_value()) {
            puts.push_back(chain.put.value());
        }
    }
    return puts;
}

std::vector<OptionChainEntry> ExpiryChain::filter_by_liquidity(
    int min_volume,
    int min_open_interest) const {

    std::vector<OptionChainEntry> filtered;
    for (const auto& option : get_all_options()) {
        if (option.meets_liquidity_requirements(min_volume, min_open_interest)) {
            filtered.push_back(option);
        }
    }
    return filtered;
}

std::vector<OptionChainEntry> ExpiryChain::filter_by_moneyness(
    double min_moneyness,
    double max_moneyness) const {

    std::vector<OptionChainEntry> filtered;
    for (const auto& option : get_all_options()) {
        if (option.moneyness >= min_moneyness && option.moneyness <= max_moneyness) {
            filtered.push_back(option);
        }
    }
    return filtered;
}

// ============================================================================
// OptionChain Implementation
// ============================================================================

OptionChain::OptionChain(const std::string& symbol)
    : symbol_(symbol) {
}

void OptionChain::add_option(const OptionChainEntry& entry) {
    auto it = expiries_.find(entry.contract.expiry);
    if (it == expiries_.end()) {
        auto [iter, inserted] = expiries_.emplace(
            entry.contract.expiry,
            ExpiryChain(symbol_, entry.contract.expiry)
        );
        iter->second.add_option(entry);
    } else {
        it->second.add_option(entry);
    }
}

std::vector<std::string> OptionChain::get_expiries() const {
    std::vector<std::string> expiries;
    for (const auto& [expiry, _] : expiries_) {
        expiries.push_back(expiry);
    }
    std::sort(expiries.begin(), expiries.end());
    return expiries;
}

std::optional<ExpiryChain> OptionChain::get_expiry_chain(
    const std::string& expiry) const {

    auto it = expiries_.find(expiry);
    if (it != expiries_.end()) {
        return it->second;
    }
    return std::nullopt;
}

std::vector<std::string> OptionChain::get_expiries_in_dte_range(
    int min_dte,
    int max_dte) const {

    std::vector<std::string> filtered;
    for (const auto& [expiry, chain] : expiries_) {
        int dte = chain.get_days_to_expiry();
        if (dte >= min_dte && dte <= max_dte) {
            filtered.push_back(expiry);
        }
    }
    std::sort(filtered.begin(), filtered.end());
    return filtered;
}

std::vector<OptionChainEntry> OptionChain::get_all_options() const {
    std::vector<OptionChainEntry> all_options;
    for (const auto& [_, chain] : expiries_) {
        auto options = chain.get_all_options();
        all_options.insert(all_options.end(), options.begin(), options.end());
    }
    return all_options;
}

void OptionChain::set_underlying_price(double price) {
    underlying_price_ = price;
}

int OptionChain::get_total_option_count() const {
    return static_cast<int>(get_all_options().size());
}

int OptionChain::get_expiry_count() const {
    return static_cast<int>(expiries_.size());
}

// ============================================================================
// OptionChainBuilder Implementation (Stubs for Black-Scholes)
// ============================================================================

OptionChain OptionChainBuilder::build_from_market_data(
    const std::string& symbol,
    const std::vector<types::OptionContract>& contracts,
    const std::map<std::string, types::MarketData>& market_data) {

    OptionChain chain(symbol);

    // NOTE: Full implementation would build the chain from market data
    spdlog::debug("Building option chain for {} ({} contracts)",
                  symbol, contracts.size());

    return chain;
}

void OptionChainBuilder::calculate_metrics(
    OptionChainEntry& entry,
    double underlying_price,
    double risk_free_rate) {

    // NOTE: Full Black-Scholes calculations would go here
    entry.moneyness = entry.contract.strike / underlying_price;
    entry.intrinsic_value = 0.0;  // Stub
    entry.extrinsic_value = entry.market_data.get_mid_price();
}

std::optional<double> OptionChainBuilder::calculate_implied_volatility(
    double option_price,
    double underlying_price,
    double strike,
    double time_to_expiry,
    double risk_free_rate,
    types::OptionType option_type) {

    // NOTE: Newton-Raphson IV calculation would go here
    return std::nullopt;  // Stub
}

double OptionChainBuilder::calculate_theoretical_price(
    double underlying_price,
    double strike,
    double time_to_expiry,
    double volatility,
    double risk_free_rate,
    types::OptionType option_type) {

    // NOTE: Black-Scholes formula would go here
    return 0.0;  // Stub
}

double OptionChainBuilder::calculate_delta(
    double underlying_price,
    double strike,
    double time_to_expiry,
    double volatility,
    double risk_free_rate,
    types::OptionType option_type) {

    // NOTE: Delta calculation would go here
    return 0.0;  // Stub
}

double OptionChainBuilder::calculate_gamma(
    double underlying_price,
    double strike,
    double time_to_expiry,
    double volatility,
    double risk_free_rate) {

    // NOTE: Gamma calculation would go here
    return 0.0;  // Stub
}

double OptionChainBuilder::calculate_theta(
    double underlying_price,
    double strike,
    double time_to_expiry,
    double volatility,
    double risk_free_rate,
    types::OptionType option_type) {

    // NOTE: Theta calculation would go here
    return 0.0;  // Stub
}

double OptionChainBuilder::calculate_vega(
    double underlying_price,
    double strike,
    double time_to_expiry,
    double volatility,
    double risk_free_rate) {

    // NOTE: Vega calculation would go here
    return 0.0;  // Stub
}

double OptionChainBuilder::standard_normal_cdf(double x) {
    return 0.5 * std::erfc(-x / std::sqrt(2.0));
}

double OptionChainBuilder::standard_normal_pdf(double x) {
    return std::exp(-0.5 * x * x) / std::sqrt(2.0 * M_PI);
}

std::pair<double, double> OptionChainBuilder::calculate_d1_d2(
    double underlying_price,
    double strike,
    double time_to_expiry,
    double volatility,
    double risk_free_rate) {

    double d1 = (std::log(underlying_price / strike) +
                 (risk_free_rate + 0.5 * volatility * volatility) * time_to_expiry) /
                (volatility * std::sqrt(time_to_expiry));

    double d2 = d1 - volatility * std::sqrt(time_to_expiry);

    return {d1, d2};
}

} // namespace option_chain
