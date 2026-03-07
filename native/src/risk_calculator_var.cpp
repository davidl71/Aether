// risk_calculator_var.cpp - VaR and scenario analysis methods
// Split from risk_calculator.cpp per T-1772887500742229784.
// All methods belong to risk::RiskCalculator (definition-file split pattern).
#include "risk_calculator.h"
#include <algorithm>
#include <cmath>
#include <numeric>
#include <string>
#include <vector>
#include <spdlog/spdlog.h>

namespace risk {

// ============================================================================
// Value at Risk (VaR) Calculations
// ============================================================================

/// Calculate Value at Risk (VaR) using historical simulation method.
///
/// Algorithm: Historical simulation uses past returns to estimate future risk.
/// The method sorts historical returns and selects the percentile corresponding
/// to the confidence level.
///
/// Formula: VaR = -sorted_returns[percentile_index]
///   where percentile_index = (1 - confidence_level) * returns.size()
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double
RiskCalculator::calculate_var_historical(const std::vector<double> &returns,
                                         double confidence_level) const {

  if (returns.empty())
    return 0.0;

  std::vector<double> sorted_returns = returns;
  std::sort(sorted_returns.begin(), sorted_returns.end());

  size_t index =
      static_cast<size_t>((1.0 - confidence_level) * sorted_returns.size());
  index = std::min(index, sorted_returns.size() - 1);

  return -sorted_returns[index];
}

/// Calculate Value at Risk (VaR) using parametric (variance-covariance) method.
///
/// Formula: VaR = position_value * z_score * volatility * sqrt(time_horizon / 252)
///
/// Z-scores: 95% → 1.645, 99% → 2.326
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double RiskCalculator::calculate_var_parametric(double expected_return,
                                                double volatility,
                                                double position_value,
                                                double confidence_level,
                                                int time_horizon_days) const {

  double z_score = 1.645; // 95%
  if (confidence_level >= 0.99) {
    z_score = 2.326; // 99%
  }

  double time_factor =
      std::sqrt(static_cast<double>(time_horizon_days) / 252.0);

  return position_value * z_score * volatility * time_factor;
}

double RiskCalculator::calculate_var_monte_carlo(
    const types::Position &position, double underlying_price, double volatility,
    int simulations, double confidence_level) const {

  // NOTE: Full Monte Carlo simulation would go here
  return 0.0; // Stub
}

double
RiskCalculator::calculate_expected_shortfall(const std::vector<double> &returns,
                                             double confidence_level) const {

  if (returns.empty())
    return 0.0;

  std::vector<double> sorted_returns = returns;
  std::sort(sorted_returns.begin(), sorted_returns.end());

  size_t cutoff =
      static_cast<size_t>((1.0 - confidence_level) * sorted_returns.size());
  cutoff = std::max(size_t(1), cutoff);

  const auto cutoff_it =
      std::next(sorted_returns.begin(),
                static_cast<std::vector<double>::difference_type>(cutoff));
  double sum = std::accumulate(sorted_returns.begin(), cutoff_it, 0.0);

  return -sum / static_cast<double>(cutoff);
}

// ============================================================================
// Scenario Analysis
// ============================================================================

std::vector<RiskCalculator::ScenarioResult>
RiskCalculator::run_scenario_analysis(
    const types::Position &position, double current_price,
    const std::vector<double> &price_scenarios) const {

  std::vector<ScenarioResult> results;

  for (double scenario_price : price_scenarios) {
    ScenarioResult result;
    result.scenario_name = "Price: $" + std::to_string(scenario_price);
    result.price_change_percent =
        ((scenario_price - current_price) / current_price) * 100.0;

    // NOTE: Would calculate P&L for this scenario
    result.position_pnl = 0.0; // Stub

    results.push_back(result);
  }

  return results;
}

std::vector<RiskCalculator::ScenarioResult>
RiskCalculator::stress_test_portfolio(
    const std::vector<types::Position> &positions,
    const std::vector<double> &market_scenarios) const {

  // NOTE: Would run stress tests across portfolio
  return {}; // Stub
}

} // namespace risk
