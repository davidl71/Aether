// risk_calculator_sizing.cpp - Position sizing methods
// Split from risk_calculator.cpp per T-1772887500742229784.
// All methods belong to risk::RiskCalculator (definition-file split pattern).
#include "risk_calculator.h"
#include <algorithm>
#include <cmath>
#include <spdlog/spdlog.h>

namespace risk {

// ============================================================================
// Position Sizing
// ============================================================================

int RiskCalculator::calculate_optimal_position_size(
    const types::BoxSpreadLeg &spread, double account_value,
    double risk_tolerance) const {

  double position_cost = spread.net_debit * 100.0;
  double max_exposure = account_value * risk_tolerance;

  int max_contracts = static_cast<int>(max_exposure / position_cost);

  return std::max(1, max_contracts);
}

/// Calculate optimal position size using Kelly Criterion.
///
/// Algorithm: The Kelly Criterion determines the optimal fraction of capital
/// to risk on a bet to maximize long-term growth.
///
/// Formula: f = (bp - q) / b
///   where:
///     f = fraction of capital to bet
///     b = win/loss ratio (win_amount / loss_amount)
///     p = win probability
///     q = loss probability (1 - p)
///
/// Expected behavior:
/// - Returns 0 if loss_amount == 0 (division by zero protection)
/// - Uses fractional Kelly (50% of full Kelly) for risk management
/// - Clamps result to maximum 25% of account value
/// - Converts to number of contracts (assuming $100 per contract)
///
/// @param win_probability Probability of winning (0.0 to 1.0)
/// @param win_amount Amount won if successful
/// @param loss_amount Amount lost if unsuccessful
/// @param account_value Total account value
/// @return Optimal number of contracts to trade
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
int RiskCalculator::calculate_kelly_position_size(double win_probability,
                                                  double win_amount,
                                                  double loss_amount,
                                                  double account_value) const {

  // Kelly Criterion: f = (bp - q) / b
  // where f = fraction to bet, b = win/loss ratio, p = win probability, q = 1-p

  if (loss_amount == 0)
    return 0;

  double b = win_amount / loss_amount;
  double p = win_probability;
  double q = 1.0 - p;

  double kelly_fraction = (b * p - q) / b;

  // Use fractional Kelly (half Kelly is common)
  kelly_fraction *= 0.5;

  // Clamp to reasonable values
  kelly_fraction = std::max(0.0, std::min(kelly_fraction, 0.25));

  double position_size = account_value * kelly_fraction;

  return static_cast<int>(position_size / 100.0); // Convert to contracts
}

int RiskCalculator::calculate_fixed_fractional_size(double position_cost,
                                                    double account_value,
                                                    double risk_percent) const {

  double risk_amount = account_value * risk_percent;
  int num_contracts = static_cast<int>(risk_amount / position_cost);

  return std::max(1, num_contracts);
}

} // namespace risk
