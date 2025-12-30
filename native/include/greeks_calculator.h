// greeks_calculator.h - Greeks calculation using QuantLib
#pragma once

#include "types.h"
#include <optional>

namespace risk {

// ============================================================================
// Greeks Structure
// ============================================================================

struct Greeks {
    double delta;   // Price sensitivity to underlying
    double gamma;   // Delta sensitivity to underlying
    double theta;  // Time decay (per day)
    double vega;   // Volatility sensitivity
    double rho;    // Interest rate sensitivity
};

// ============================================================================
// GreeksCalculator Class
// ============================================================================

class GreeksCalculator {
public:
    GreeksCalculator();
    ~GreeksCalculator() = default;

    // Calculate Greeks for a single option
    // Returns nullopt if calculation fails
    std::optional<Greeks> calculate_option_greeks(
        const types::OptionContract& contract,
        double underlying_price,
        double option_price,
        double risk_free_rate,
        double implied_volatility
    ) const;

    // Calculate Greeks for a stock position (non-option)
    // Stocks have delta=1.0, other Greeks=0.0
    Greeks calculate_stock_greeks(int quantity) const;

    // Aggregate Greeks across multiple positions
    // Multiplies by quantity and sums
    Greeks aggregate_greeks(
        const std::vector<types::Position>& positions,
        double underlying_price,
        double risk_free_rate,
        double implied_volatility
    ) const;

private:
    // Helper: Convert time to expiry from days to years
    double days_to_years(int days) const;
};

} // namespace risk
