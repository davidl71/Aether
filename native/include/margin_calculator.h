// margin_calculator.h - Margin requirement calculations for box spreads
#pragma once

#include "types.h"
#include <vector>
#include <optional>
#include <chrono>

namespace margin {

// ============================================================================
// Margin Calculation Results
// ============================================================================

struct MarginResult {
  double initial_margin;              // Initial margin required
  double maintenance_margin;         // Maintenance margin required
  double portfolio_margin_benefit;    // Margin reduction from portfolio margining
  double reg_t_margin;                // Reg-T margin if applicable
  double span_margin;                 // SPAN margin if applicable
  bool uses_portfolio_margin;         // Whether portfolio margin is used
  std::chrono::system_clock::time_point calculated_at;

  // Helper methods
  double get_effective_margin() const;
  bool is_valid() const;
};

// ============================================================================
// Margin Calculator Class
// ============================================================================

class MarginCalculator {
public:
  MarginCalculator();
  ~MarginCalculator();

  // ========================================================================
  // Box Spread Margin Calculations
  // ========================================================================

  // Calculate Reg-T margin for box spread
  // Reg-T: Margin on short legs minus long legs (with offsets)
  MarginResult calculate_reg_t_margin(
      const types::BoxSpreadLeg& spread,
      double underlying_price
  ) const;

  // Calculate portfolio margin (simplified SPAN-like)
  // Portfolio margin considers offsetting positions and risk scenarios
  MarginResult calculate_portfolio_margin(
      const types::BoxSpreadLeg& spread,
      double underlying_price,
      double implied_volatility
  ) const;

  // Calculate SPAN margin (Standard Portfolio Analysis of Risk)
  // SPAN uses scenario-based risk analysis
  MarginResult calculate_span_margin(
      const types::BoxSpreadLeg& spread,
      double underlying_price,
      double implied_volatility
  ) const;

  // Calculate margin for box spread (auto-selects best method)
  MarginResult calculate_margin(
      const types::BoxSpreadLeg& spread,
      double underlying_price,
      double implied_volatility = 0.20,
      bool prefer_portfolio_margin = true
  ) const;

  // ========================================================================
  // Portfolio-Level Margin Calculations
  // ========================================================================

  // Calculate portfolio margin benefit from multiple box spreads
  double calculate_portfolio_margin_benefit(
      const std::vector<types::BoxSpreadLeg>& spreads,
      double underlying_price
  ) const;

  // Calculate total margin for portfolio of box spreads
  MarginResult calculate_portfolio_margin(
      const std::vector<types::BoxSpreadLeg>& spreads,
      double underlying_price,
      double implied_volatility = 0.20
  ) const;

  // ========================================================================
  // Margin Call Analysis
  // ========================================================================

  // Check if position is at risk of margin call
  bool is_margin_call_risk(
      double current_margin,
      double maintenance_margin,
      double account_value,
      double buffer_percent = 10.0  // 10% buffer above maintenance
  ) const;

  // Calculate margin utilization percentage
  double calculate_margin_utilization(
      double margin_used,
      double available_margin
  ) const;

  // Calculate remaining margin capacity
  double calculate_remaining_margin_capacity(
      double account_value,
      double initial_margin_used,
      double maintenance_margin_used
  ) const;

private:
  // Helper: Calculate margin for short option leg
  double calculate_short_option_margin(
      double underlying_price,
      double strike,
      types::OptionType type,
      double option_price,
      double implied_volatility,
      int days_to_expiry
  ) const;

  // Helper: Calculate margin for long option leg (typically 0 for long options)
  double calculate_long_option_margin(
      double option_price
  ) const;

  // Helper: Apply box spread margin offsets (short legs offset long legs)
  double apply_box_spread_offsets(
      double short_call_margin,
      double short_put_margin,
      double long_call_price,
      double long_put_price,
      double strike_width
  ) const;

  // Helper: Calculate SPAN risk array scenarios
  std::vector<double> calculate_span_scenarios(
      double underlying_price,
      double implied_volatility,
      int days_to_expiry
  ) const;
};

} // namespace margin
