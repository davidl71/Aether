// mock_data_generator.h - Mock data generator for box spread bags
#pragma once

#include "strategies/box_spread/box_spread_bag.h"
#include "types.h"
#include <vector>
#include <random>
#include <chrono>

namespace mock {

// ============================================================================
// Mock Data Generator for Box Spread Bags
// ============================================================================

class MockBoxSpreadBagGenerator {
public:
    MockBoxSpreadBagGenerator();

    // Generate a single mock box spread bag
    types::BoxSpreadBag generate_bag(
        const std::string& underlying_symbol = "SPX",
        const std::string& expiry = "",
        double strike_width = 100.0,
        int days_to_expiry = 30
    );

    // Generate multiple mock bags with varying parameters
    std::vector<types::BoxSpreadBag> generate_bags(
        size_t count,
        const std::string& underlying_symbol = "SPX"
    );

    // Generate realistic price movement for a bag
    void simulate_price_movement(
        types::BoxSpreadBag& bag,
        double underlying_price,
        int num_updates = 10,
        double time_step_minutes = 5.0
    );

    // Generate candle history for a bag
    void generate_candle_history(
        types::BoxSpreadBag& bag,
        int num_candles = 50,
        double base_price = 0.0
    );

    // Update bag with realistic market data
    void update_bag_market_data(
        types::BoxSpreadBag& bag,
        double underlying_price = 4500.0
    );

    // Generate bags for different underlyings (SPX, ES, XSP, nanos)
    std::vector<types::BoxSpreadBag> generate_multi_symbol_bags(
        const std::vector<std::string>& symbols,
        double strike_width = 100.0,
        int days_to_expiry = 30
    );

    // Generate bags with same strike width across different expirations (yield curve)
    std::vector<types::BoxSpreadBag> generate_yield_curve_bags(
        const std::string& symbol,
        double strike_width,
        const std::vector<int>& days_to_expiry_list,
        double underlying_price = 4500.0
    );

private:
    std::mt19937 rng_;
    std::uniform_real_distribution<double> price_dist_;
    std::uniform_real_distribution<double> volume_dist_;
    std::normal_distribution<double> price_change_dist_;

    // Helper methods
    double generate_realistic_net_debit(
        double strike_width,
        int days_to_expiry,
        double implied_rate_percent = 5.0
    );

    std::string generate_expiry_date(int days_to_expiry);

    double add_realistic_noise(double value, double std_dev_percent = 0.5);

    void update_greeks_for_bag(
        types::BoxSpreadBag& bag,
        double underlying_price
    );
};

// ============================================================================
// Cboe Complex Symbol Format Helper
// ============================================================================

class CboeSymbolFormatter {
public:
    // Format box spread as Cboe complex symbol
    static std::string format_complex_symbol(
        const std::string& underlying,
        const std::string& expiry,
        double strike_low,
        double strike_high
    );

    // Parse Cboe complex symbol (if needed)
    static bool parse_complex_symbol(
        const std::string& cboe_symbol,
        std::string& underlying,
        std::string& expiry,
        double& strike_low,
        double& strike_high
    );

private:
    static std::string format_expiry_cboe(const std::string& ibkr_expiry);
};

} // namespace mock
