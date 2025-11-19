// margin_calculator.cpp - Margin requirement calculations for box spreads
#include "margin_calculator.h"
#include <spdlog/spdlog.h>
#include <algorithm>
#include <cmath>

namespace margin {

// ============================================================================
// MarginResult Implementation
// ============================================================================

double MarginResult::get_effective_margin() const {
  // Return portfolio margin if available and enabled, otherwise initial margin
  if (uses_portfolio_margin && span_margin > 0.0) {
    return span_margin;
  }
  if (initial_margin > 0.0) {
    return initial_margin;
  }
  // Fallback to reg_t_margin if available
  if (reg_t_margin > 0.0) {
    return reg_t_margin;
  }
  return 0.0;
}

bool MarginResult::is_valid() const {
  return initial_margin >= 0.0 && maintenance_margin >= 0.0;
}

// ============================================================================
// MarginCalculator Implementation
// ============================================================================

MarginCalculator::MarginCalculator() {
  spdlog::debug("MarginCalculator initialized");
}

MarginCalculator::~MarginCalculator() {
  spdlog::debug("MarginCalculator destroyed");
}

// ============================================================================
// Box Spread Margin Calculations
// ============================================================================

MarginResult MarginCalculator::calculate_reg_t_margin(
    const types::BoxSpreadLeg& spread,
    double underlying_price) const {

  MarginResult result{};
  result.calculated_at = std::chrono::system_clock::now();
  result.uses_portfolio_margin = false;

  // Reg-T margin for box spreads:
  // Margin on short legs minus long leg premiums (with offsets)
  // Box spreads typically have reduced margin due to offsetting risk

  double short_call_margin = calculate_short_option_margin(
      underlying_price,
      spread.short_call.strike,
      types::OptionType::Call,
      spread.short_call_price,
      0.20,  // Default IV if not available
      spread.get_days_to_expiry()
  );

  double short_put_margin = calculate_short_option_margin(
      underlying_price,
      spread.short_put.strike,
      types::OptionType::Put,
      spread.short_put_price,
      0.20,  // Default IV if not available
      spread.get_days_to_expiry()
  );

  // Long legs: margin is typically 0 (premium paid), but we subtract premium
  double long_call_premium = spread.long_call_price * 100.0;
  double long_put_premium = spread.long_put_price * 100.0;

  // Box spread offset: short margin minus long premiums
  // Minimum margin is typically the net debit
  double net_margin = short_call_margin + short_put_margin
                      - long_call_premium - long_put_premium;

  // Reg-T minimum: typically net debit for box spreads
  result.reg_t_margin = std::max(net_margin, spread.net_debit * 100.0);
  result.initial_margin = result.reg_t_margin;
  result.maintenance_margin = result.initial_margin * 0.75;  // Typically 75% of initial

  spdlog::debug("Reg-T margin for box spread: initial=${:.2f}, maintenance=${:.2f}",
                result.initial_margin, result.maintenance_margin);

  return result;
}

MarginResult MarginCalculator::calculate_portfolio_margin(
    const types::BoxSpreadLeg& spread,
    double underlying_price,
    double implied_volatility) const {

  MarginResult result{};
  result.calculated_at = std::chrono::system_clock::now();
  result.uses_portfolio_margin = true;

  // Portfolio margin uses risk-based approach
  // For box spreads, margin is typically much lower due to offsetting risk

  // Calculate SPAN margin as base
  MarginResult span_result = calculate_span_margin(
      spread, underlying_price, implied_volatility);
  result.span_margin = span_result.span_margin;

  // Portfolio margin benefit: reduction from Reg-T
  MarginResult reg_t_result = calculate_reg_t_margin(spread, underlying_price);
  result.reg_t_margin = reg_t_result.reg_t_margin;

  // Portfolio margin is typically 50-80% of Reg-T for box spreads
  // Box spreads are low risk, so portfolio margin is usually close to net debit
  double portfolio_multiplier = 0.60;  // Conservative estimate
  result.initial_margin = std::max(
      result.span_margin,
      reg_t_result.initial_margin * portfolio_multiplier
  );

  // Ensure portfolio margin is at least net debit
  result.initial_margin = std::max(
      result.initial_margin,
      spread.net_debit * 100.0
  );

  result.maintenance_margin = result.initial_margin * 0.75;
  result.portfolio_margin_benefit = reg_t_result.initial_margin - result.initial_margin;

  spdlog::debug("Portfolio margin for box spread: initial=${:.2f}, benefit=${:.2f}",
                result.initial_margin, result.portfolio_margin_benefit);

  return result;
}

MarginResult MarginCalculator::calculate_span_margin(
    const types::BoxSpreadLeg& spread,
    double underlying_price,
    double implied_volatility) const {

  MarginResult result{};
  result.calculated_at = std::chrono::system_clock::now();
  result.uses_portfolio_margin = true;

  // SPAN margin uses scenario-based risk analysis
  // Calculate risk across multiple price and volatility scenarios

  std::vector<double> scenarios = calculate_span_scenarios(
      underlying_price, implied_volatility, spread.get_days_to_expiry());

  double max_loss = 0.0;

  // For box spreads, risk is limited to net debit
  // SPAN scenarios should confirm this
  for (double scenario_price : scenarios) {
    // Box spread value is fixed (strike width) regardless of price
    // Loss is limited to net debit
    double scenario_loss = spread.net_debit * 100.0;
    max_loss = std::max(max_loss, scenario_loss);
  }

  // SPAN margin = max loss across all scenarios
  // For box spreads, this is typically the net debit
  result.span_margin = std::max(max_loss, spread.net_debit * 100.0);
  result.initial_margin = result.span_margin;
  result.maintenance_margin = result.initial_margin * 0.75;

  spdlog::debug("SPAN margin for box spread: ${:.2f}", result.span_margin);

  return result;
}

MarginResult MarginCalculator::calculate_margin(
    const types::BoxSpreadLeg& spread,
    double underlying_price,
    double implied_volatility,
    bool prefer_portfolio_margin) const {

  if (prefer_portfolio_margin) {
    return calculate_portfolio_margin(spread, underlying_price, implied_volatility);
  } else {
    return calculate_reg_t_margin(spread, underlying_price);
  }
}

// ============================================================================
// Portfolio-Level Margin Calculations
// ============================================================================

double MarginCalculator::calculate_portfolio_margin_benefit(
    const std::vector<types::BoxSpreadLeg>& spreads,
    double underlying_price) const {

  if (spreads.empty()) {
    return 0.0;
  }

  // Calculate total Reg-T margin
  double total_reg_t = 0.0;
  for (const auto& spread : spreads) {
    MarginResult reg_t = calculate_reg_t_margin(spread, underlying_price);
    total_reg_t += reg_t.initial_margin;
  }

  // Calculate portfolio margin
  MarginResult portfolio = calculate_portfolio_margin(
      spreads[0], underlying_price, 0.20);  // Use first spread for IV estimate
  double total_portfolio = portfolio.initial_margin * spreads.size();

  // Benefit is the difference
  return std::max(0.0, total_reg_t - total_portfolio);
}

MarginResult MarginCalculator::calculate_portfolio_margin(
    const std::vector<types::BoxSpreadLeg>& spreads,
    double underlying_price,
    double implied_volatility) const {

  MarginResult result{};
  result.calculated_at = std::chrono::system_clock::now();
  result.uses_portfolio_margin = true;

  if (spreads.empty()) {
    return result;
  }

  // Calculate margin for each spread
  double total_margin = 0.0;
  double total_reg_t = 0.0;

  for (const auto& spread : spreads) {
    MarginResult margin = calculate_portfolio_margin(
        spread, underlying_price, implied_volatility);
    total_margin += margin.initial_margin;
    total_reg_t += margin.reg_t_margin;
  }

  result.initial_margin = total_margin;
  result.maintenance_margin = total_margin * 0.75;
  result.reg_t_margin = total_reg_t;
  result.portfolio_margin_benefit = total_reg_t - total_margin;

  return result;
}

// ============================================================================
// Margin Call Analysis
// ============================================================================

bool MarginCalculator::is_margin_call_risk(
    double current_margin,
    double maintenance_margin,
    double account_value,
    double buffer_percent) const {

  double margin_used = current_margin;
  double available_margin = account_value - margin_used;
  double maintenance_threshold = maintenance_margin * (1.0 + buffer_percent / 100.0);

  return margin_used >= maintenance_threshold || available_margin < maintenance_margin;
}

double MarginCalculator::calculate_margin_utilization(
    double margin_used,
    double available_margin) const {

  if (available_margin <= 0.0) {
    return 100.0;  // Fully utilized
  }

  double total_margin = margin_used + available_margin;
  if (total_margin <= 0.0) {
    return 0.0;
  }

  return (margin_used / total_margin) * 100.0;
}

double MarginCalculator::calculate_remaining_margin_capacity(
    double account_value,
    double initial_margin_used,
    double maintenance_margin_used) const {

  // Remaining capacity is the difference between account value and maintenance margin
  return std::max(0.0, account_value - maintenance_margin_used);
}

// ============================================================================
// Helper Methods
// ============================================================================

double MarginCalculator::calculate_short_option_margin(
    double underlying_price,
    double strike,
    types::OptionType type,
    double option_price,
    double implied_volatility,
    int days_to_expiry) const {

  // Reg-T margin for short options:
  // Call: max(20% of underlying - OTM amount, 10% of underlying) + premium
  // Put: max(20% of underlying - OTM amount, 10% of strike) + premium

  double premium = option_price * 100.0;
  double otm_amount = 0.0;

  if (type == types::OptionType::Call) {
    otm_amount = std::max(0.0, strike - underlying_price);
    double margin = std::max(
        0.20 * underlying_price - otm_amount,
        0.10 * underlying_price
    ) * 100.0;
    return margin + premium;
  } else {  // Put
    otm_amount = std::max(0.0, underlying_price - strike);
    double margin = std::max(
        0.20 * underlying_price - otm_amount,
        0.10 * strike
    ) * 100.0;
    return margin + premium;
  }
}

double MarginCalculator::calculate_long_option_margin(
    double option_price) const {

  // Long options: margin is typically 0 (premium paid)
  // But we return premium as the "margin" (capital required)
  return option_price * 100.0;
}

double MarginCalculator::apply_box_spread_offsets(
    double short_call_margin,
    double short_put_margin,
    double long_call_price,
    double long_put_price,
    double strike_width) const {

  // Box spread offsets: short margins minus long premiums
  double long_premiums = (long_call_price + long_put_price) * 100.0;
  double short_margins = short_call_margin + short_put_margin;

  // Minimum margin is typically the strike width (theoretical value)
  return std::max(short_margins - long_premiums, strike_width * 100.0);
}

std::vector<double> MarginCalculator::calculate_span_scenarios(
    double underlying_price,
    double implied_volatility,
    int days_to_expiry) const {

  // SPAN scenarios: price moves up/down by various percentages
  // Simplified version for box spreads (which are price-neutral)

  std::vector<double> scenarios;

  // Standard SPAN scenarios (simplified)
  scenarios.push_back(underlying_price * 0.70);   // -30%
  scenarios.push_back(underlying_price * 0.85);   // -15%
  scenarios.push_back(underlying_price * 0.95);   // -5%
  scenarios.push_back(underlying_price);           // Current
  scenarios.push_back(underlying_price * 1.05);  // +5%
  scenarios.push_back(underlying_price * 1.15);  // +15%
  scenarios.push_back(underlying_price * 1.30);  // +30%

  return scenarios;
}

} // namespace margin
