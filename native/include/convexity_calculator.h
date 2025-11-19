// convexity_calculator.h - Convexity optimization for barbell strategy
#pragma once

#include <vector>
#include <optional>

namespace convexity {

// ============================================================================
// Bond Convexity Data
// ============================================================================

struct BondData {
    double duration;      // Duration in years
    double convexity;     // Convexity value
    std::string name;    // Bond/ETF name (e.g., "SHY", "TLT")
};

// ============================================================================
// Optimization Result
// ============================================================================

struct OptimizationResult {
    double short_term_weight;    // Weight for short-term bonds (0.0 to 1.0)
    double long_term_weight;     // Weight for long-term bonds (0.0 to 1.0)
    double portfolio_convexity;  // Resulting portfolio convexity
    double portfolio_duration;    // Resulting portfolio duration
    bool success;                 // Whether optimization succeeded
    std::string error_message;   // Error message if failed
};

// ============================================================================
// Convexity Calculator Class
// ============================================================================

class ConvexityCalculator {
public:
    ConvexityCalculator();
    ~ConvexityCalculator();

    // Calculate portfolio convexity from weights
    // Pure calculation function - no side effects
    double calculate_portfolio_convexity(
        double short_term_weight,
        double long_term_convexity,
        double long_term_weight,
        double long_term_convexity_value
    ) const __attribute__((pure));

    // Optimize barbell allocation to maximize convexity
    // Constrained optimization: weights sum to 1.0, duration target
    OptimizationResult optimize_barbell_allocation(
        const BondData& short_term_bond,
        const BondData& long_term_bond,
        double target_duration,
        double target_convexity = 150.0
    ) const;

    // Calculate current portfolio convexity
    // Pure calculation function - no side effects
    double calculate_current_convexity(
        double short_term_weight,
        const BondData& short_term_bond,
        double long_term_weight,
        const BondData& long_term_bond
    ) const __attribute__((pure));

    // Check if rebalancing is needed (convexity drift > threshold)
    // Pure calculation function - no side effects
    bool should_rebalance(
        double current_convexity,
        double target_convexity,
        double threshold_percent = 10.0
    ) const __attribute__((pure));

private:
    // Helper: Calculate weighted average duration
    double calculate_weighted_duration(
        double short_weight,
        double short_duration,
        double long_weight,
        double long_duration
    ) const;
};

} // namespace convexity
