// box_spread_bag.cpp - Box Spread Bag implementation
#include "strategies/box_spread/box_spread_bag.h"
#include "option_chain.h"
#include "strategies/box_spread/box_spread_strategy.h"
#include <spdlog/spdlog.h>
#include <algorithm>
#include <cmath>
#include <sstream>
#include <iomanip>

namespace types {

// ============================================================================
// BoxSpreadBag Helper Methods
// ============================================================================

bool BoxSpreadBag::is_valid() const {
    return !complex_symbol.empty() &&
           !cboe_symbol.empty() &&
           spread.is_valid() &&
           days_to_expiry > 0;
}

void BoxSpreadBag::update_candle(double price, double volume) {
    auto now = std::chrono::system_clock::now();

    if (candle.period_start.time_since_epoch().count() == 0) {
        // Initialize new candle
        candle.open = price;
        candle.high = price;
        candle.low = price;
        candle.close = price;
        candle.entry = position.entry_price > 0 ? position.entry_price : price;
        candle.volume = volume;
        candle.period_start = now;
        candle.period_end = now;
        candle.updated = now;
    } else {
        // Update existing candle
        candle.high = std::max(candle.high, price);
        candle.low = std::min(candle.low, price);
        candle.close = price;
        candle.volume += volume;
        candle.period_end = now;
        candle.updated = now;
    }
}

void BoxSpreadBag::add_candle_to_history() {
    if (candle.period_start.time_since_epoch().count() > 0) {
        candle_history.push_back(candle);

        // Keep only recent history (e.g., last 100 candles)
        const size_t max_history = 100;
        if (candle_history.size() > max_history) {
            candle_history.erase(candle_history.begin());
        }
    }
}

void BoxSpreadBag::reset_candle() {
    add_candle_to_history();

    // Reset current candle
    candle.open = 0.0;
    candle.high = 0.0;
    candle.low = 0.0;
    candle.close = 0.0;
    candle.volume = 0.0;
    candle.period_start = std::chrono::system_clock::time_point{};
    candle.period_end = std::chrono::system_clock::time_point{};
}

double BoxSpreadBag::get_current_pnl() const {
    if (position.quantity == 0) return 0.0;

    double current_value = market_data.get_mid_price() * position.quantity;
    double cost_basis = position.cost_basis;
    return current_value - cost_basis;
}

double BoxSpreadBag::get_pnl_per_contract() const {
    if (position.quantity == 0) return 0.0;
    return get_current_pnl() / std::abs(position.quantity);
}

std::string BoxSpreadBag::generate_cboe_symbol(
    const std::string& underlying,
    const std::string& expiry,
    double strike_low,
    double strike_high) {

    // Format: "SPX 25JAN24 4500/4600 BOX"
    std::ostringstream oss;
    oss << underlying << " " << expiry << " "
        << static_cast<int>(strike_low) << "/" << static_cast<int>(strike_high)
        << " BOX";
    return oss.str();
}

// ============================================================================
// BoxSpreadBagManager Implementation
// ============================================================================

BoxSpreadBag BoxSpreadBagManager::create_bag_from_spread(
    const types::BoxSpreadLeg& spread,
    const std::string& underlying_symbol) {

    BoxSpreadBag bag;
    bag.spread = spread;
    bag.complex_symbol = underlying_symbol + " BOX";

    // Generate Cboe-style symbol
    bag.cboe_symbol = BoxSpreadBag::generate_cboe_symbol(
        underlying_symbol,
        spread.long_call.expiry,
        spread.long_call.strike,
        spread.short_call.strike
    );

    // Calculate basic metrics
    bag.theoretical_value = spread.theoretical_value;
    bag.net_debit = spread.net_debit;
    bag.days_to_expiry = spread.get_days_to_expiry();

    // Calculate implied rates using BoxSpreadCalculator
    bag.implied_rate = strategy::BoxSpreadCalculator::calculate_implied_interest_rate(spread);

    // Initialize market data (will be updated with real prices)
    bag.market_data.bid = 0.0;
    bag.market_data.ask = 0.0;
    bag.market_data.last = bag.net_debit;
    bag.market_data.mid = bag.net_debit;
    bag.market_data.spread = 0.0;

    // Initialize position
    bag.position.quantity = 0;
    bag.position.entry_price = bag.net_debit;
    bag.position.current_price = bag.net_debit;
    bag.position.cost_basis = 0.0;
    bag.position.unrealized_pnl = 0.0;

    // Initialize candle
    bag.candle.entry = bag.net_debit;
    bag.candle.open = bag.net_debit;
    bag.candle.high = bag.net_debit;
    bag.candle.low = bag.net_debit;
    bag.candle.close = bag.net_debit;

    // Timestamps
    auto now = std::chrono::system_clock::now();
    bag.created_at = now;
    bag.last_updated = now;
    bag.market_data.timestamp = now;
    bag.greeks.calculated_at = now;

    spdlog::debug("Created box spread bag: {}", bag.cboe_symbol);

    return bag;
}

BoxSpreadBag::BagGreeks BoxSpreadBagManager::calculate_bag_greeks(
    const types::BoxSpreadLeg& spread,
    double underlying_price,
    double time_to_expiry,
    double volatility,
    double risk_free_rate) {

    BagGreeks greeks;
    greeks.calculated_at = std::chrono::system_clock::now();

    // Convert days to expiry to years
    double time_to_expiry_years = time_to_expiry / 365.0;
    if (time_to_expiry_years <= 0) {
        time_to_expiry_years = 0.001;  // Minimum to avoid division by zero
    }

    // Aggregate Greeks from individual legs
    // Box spread = Long Call(K1) - Short Call(K2) + Long Put(K2) - Short Put(K1)

    double strike_low = spread.long_call.strike;
    double strike_high = spread.short_call.strike;

    // Calculate Greeks for each leg
    double long_call_delta = option_chain::OptionChainBuilder::calculate_delta(
        underlying_price, strike_low, time_to_expiry_years,
        volatility, risk_free_rate, OptionType::Call);

    double short_call_delta = option_chain::OptionChainBuilder::calculate_delta(
        underlying_price, strike_high, time_to_expiry_years,
        volatility, risk_free_rate, OptionType::Call);

    double long_put_delta = option_chain::OptionChainBuilder::calculate_delta(
        underlying_price, strike_high, time_to_expiry_years,
        volatility, risk_free_rate, OptionType::Put);

    double short_put_delta = option_chain::OptionChainBuilder::calculate_delta(
        underlying_price, strike_low, time_to_expiry_years,
        volatility, risk_free_rate, OptionType::Put);

    // Aggregate delta (should be ~0 for perfect box spread)
    // Long Call + Short Put = synthetic long
    // Short Call + Long Put = synthetic short
    // Net should be near zero
    greeks.delta = long_call_delta - short_call_delta + long_put_delta - short_put_delta;

    // Gamma (same for all options at same strike)
    double gamma_low = option_chain::OptionChainBuilder::calculate_gamma(
        underlying_price, strike_low, time_to_expiry_years,
        volatility, risk_free_rate);

    double gamma_high = option_chain::OptionChainBuilder::calculate_gamma(
        underlying_price, strike_high, time_to_expiry_years,
        volatility, risk_free_rate);

    // Aggregate gamma (should be ~0)
    greeks.gamma = gamma_low - gamma_high + gamma_high - gamma_low;  // Should cancel

    // Theta (time decay)
    double long_call_theta = option_chain::OptionChainBuilder::calculate_theta(
        underlying_price, strike_low, time_to_expiry_years,
        volatility, risk_free_rate, types::OptionType::Call);

    double short_call_theta = option_chain::OptionChainBuilder::calculate_theta(
        underlying_price, strike_high, time_to_expiry_years,
        volatility, risk_free_rate, types::OptionType::Call);

    double long_put_theta = option_chain::OptionChainBuilder::calculate_theta(
        underlying_price, strike_high, time_to_expiry_years,
        volatility, risk_free_rate, types::OptionType::Put);

    double short_put_theta = option_chain::OptionChainBuilder::calculate_theta(
        underlying_price, strike_low, time_to_expiry_years,
        volatility, risk_free_rate, types::OptionType::Put);

    // Aggregate theta (net time decay)
    greeks.theta = long_call_theta - short_call_theta + long_put_theta - short_put_theta;

    // Vega (IV sensitivity - same for all options)
    double vega_low = option_chain::OptionChainBuilder::calculate_vega(
        underlying_price, strike_low, time_to_expiry_years,
        volatility, risk_free_rate);

    double vega_high = option_chain::OptionChainBuilder::calculate_vega(
        underlying_price, strike_high, time_to_expiry_years,
        volatility, risk_free_rate);

    // Aggregate vega (should be ~0)
    greeks.vega = vega_low - vega_high + vega_high - vega_low;  // Should cancel

    // Rho (rate sensitivity)
    // Box spreads have some rho exposure due to financing costs
    // For now, approximate based on time to expiry
    greeks.rho = (spread.get_strike_width() - spread.net_debit) * time_to_expiry_years * 0.01;

    spdlog::debug("Bag Greeks: delta={:.4f}, gamma={:.4f}, theta={:.4f}, vega={:.4f}, rho={:.4f}",
                 greeks.delta, greeks.gamma, greeks.theta, greeks.vega, greeks.rho);

    return greeks;
}

void BoxSpreadBagManager::update_bag_market_data(
    BoxSpreadBag& bag,
    double bid,
    double ask,
    double last,
    int bid_size,
    int ask_size) {

    bag.market_data.bid = bid;
    bag.market_data.ask = ask;
    bag.market_data.last = last > 0 ? last : bag.market_data.get_mid_price();
    bag.market_data.bid_size = bid_size;
    bag.market_data.ask_size = ask_size;
    bag.market_data.timestamp = std::chrono::system_clock::now();

    // Update candle with mid price
    double mid_price = bag.market_data.get_mid_price();
    bag.update_candle(mid_price, 1.0);  // Assume 1 contract volume

    // Update position if we have one
    if (bag.position.quantity != 0) {
        bag.position.current_price = mid_price;
        bag.position.unrealized_pnl = bag.get_current_pnl();
    }

    bag.last_updated = std::chrono::system_clock::now();
}

void BoxSpreadBagManager::update_bag_greeks(
    BoxSpreadBag& bag,
    double underlying_price,
    double volatility,
    double risk_free_rate) {

    double time_to_expiry = static_cast<double>(bag.days_to_expiry) / 365.0;
    bag.greeks = calculate_bag_greeks(
        bag.spread,
        underlying_price,
        time_to_expiry,
        volatility,
        risk_free_rate
    );
}

void BoxSpreadBagManager::update_bag_candle(
    BoxSpreadBag& bag,
    double price,
    double volume) {

    bag.update_candle(price, volume);
    bag.last_updated = std::chrono::system_clock::now();
}

} // namespace types
