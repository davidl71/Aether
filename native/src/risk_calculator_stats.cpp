// risk_calculator_stats.cpp - Greeks stubs, risk-adjusted returns, drawdown,
// and free helper functions (calculate_mean, calculate_std_dev, etc.)
// Split from risk_calculator.cpp per T-1772887500742229784.
// All RiskCalculator:: methods belong to risk::RiskCalculator (definition-file
// split pattern). Free functions (calculate_mean, etc.) are in namespace risk.
#include "risk_calculator.h"
#include <algorithm>
#include <cmath>
#include <limits>
#include <numeric>
#include <vector>
#include <spdlog/spdlog.h>

namespace risk {

// ============================================================================
// Greeks Calculations (box spread stubs — box spreads are Greek-neutral)
// ============================================================================

double RiskCalculator::calculate_box_spread_delta(
    const types::BoxSpreadLeg &spread) const {
  return 0.0; // Box spreads are delta-neutral
}

types::RiskMetrics
RiskCalculator::calculate_box_spread_greeks(const types::BoxSpreadLeg &spread,
                                            double underlying_price,
                                            double volatility) const {
  types::RiskMetrics metrics;
  metrics.delta = 0.0;
  metrics.gamma = 0.0;
  metrics.theta = 0.0;
  metrics.vega  = 0.0;
  metrics.rho   = 0.0;

  metrics.max_loss = spread.net_debit * 100.0;
  metrics.max_gain = (spread.get_strike_width() - spread.net_debit) * 100.0;
  metrics.probability_of_profit = 1.0; // If held to expiry

  return metrics;
}

// ============================================================================
// Risk-Adjusted Returns
// ============================================================================

/// Calculate Sharpe ratio (risk-adjusted return measure).
///
/// Formula: Sharpe = (mean_return - risk_free_rate) / standard_deviation
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double
RiskCalculator::calculate_sharpe_ratio(const std::vector<double> &returns,
                                       double risk_free_rate) const {

  if (returns.empty())
    return 0.0;

  double mean_return = calculate_mean(returns);
  double std_dev = calculate_standard_deviation(returns);

  if (std_dev == 0)
    return 0.0;

  return (mean_return - risk_free_rate) / std_dev;
}

/// Calculate Sortino ratio (downside risk-adjusted return measure).
///
/// Formula: Sortino = (mean_return - risk_free_rate) / downside_deviation
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double
RiskCalculator::calculate_sortino_ratio(const std::vector<double> &returns,
                                        double risk_free_rate) const {

  if (returns.empty())
    return 0.0;

  double mean_return = calculate_mean(returns);

  std::vector<double> downside_returns;
  downside_returns.reserve(returns.size());
  std::copy_if(returns.begin(), returns.end(),
               std::back_inserter(downside_returns),
               [](double r) { return r < 0.0; });

  if (downside_returns.empty())
    return std::numeric_limits<double>::infinity();

  double downside_dev = calculate_standard_deviation(downside_returns);

  if (downside_dev == 0)
    return 0.0;

  return (mean_return - risk_free_rate) / downside_dev;
}

double RiskCalculator::calculate_calmar_ratio(double annualized_return,
                                              double max_drawdown) const {
  if (max_drawdown == 0)
    return 0.0;

  return annualized_return / max_drawdown;
}

double RiskCalculator::calculate_information_ratio(
    const std::vector<double> &returns,
    const std::vector<double> &benchmark_returns) const {

  if (returns.size() != benchmark_returns.size() || returns.empty()) {
    return 0.0;
  }

  std::vector<double> excess_returns;
  excess_returns.reserve(returns.size());
  std::transform(returns.begin(), returns.end(), benchmark_returns.begin(),
                 std::back_inserter(excess_returns),
                 [](double r, double benchmark) { return r - benchmark; });

  double mean_excess = calculate_mean(excess_returns);
  double tracking_error = calculate_standard_deviation(excess_returns);

  if (tracking_error == 0)
    return 0.0;

  return mean_excess / tracking_error;
}

// ============================================================================
// Drawdown Analysis
// ============================================================================

double RiskCalculator::calculate_max_drawdown(
    const std::vector<double> &equity_curve) const {

  if (equity_curve.empty())
    return 0.0;

  double max_drawdown = 0.0;
  double peak = equity_curve[0];

  for (double value : equity_curve) {
    if (value > peak) {
      peak = value;
    }
    double drawdown = (peak - value) / peak;
    max_drawdown = std::max(max_drawdown, drawdown);
  }

  return max_drawdown;
}

double RiskCalculator::calculate_current_drawdown(
    const std::vector<double> &equity_curve) const {

  if (equity_curve.empty())
    return 0.0;

  double peak = *std::max_element(equity_curve.begin(), equity_curve.end());
  double current = equity_curve.back();

  return (peak - current) / peak;
}

int RiskCalculator::calculate_drawdown_duration(
    const std::vector<double> &equity_curve) const {

  return 0; // Stub
}

// ============================================================================
// Free Helper Functions (declared in risk_calculator.h)
// ============================================================================

double calculate_standard_deviation(const std::vector<double> &values) {
  if (values.empty())
    return 0.0;

  double mean = calculate_mean(values);
  double sum_squared_diff = 0.0;

  for (double value : values) {
    double diff = value - mean;
    sum_squared_diff += diff * diff;
  }

  return std::sqrt(sum_squared_diff / static_cast<double>(values.size()));
}

double calculate_mean(const std::vector<double> &values) {
  if (values.empty())
    return 0.0;

  double sum = std::accumulate(values.begin(), values.end(), 0.0);
  return sum / static_cast<double>(values.size());
}

double calculate_percentile(const std::vector<double> &values,
                            double percentile) {
  if (values.empty())
    return 0.0;

  std::vector<double> sorted = values;
  std::sort(sorted.begin(), sorted.end());

  size_t index = static_cast<size_t>(percentile * sorted.size());
  index = std::min(index, sorted.size() - 1);

  return sorted[index];
}

double calculate_correlation(const std::vector<double> &x,
                             const std::vector<double> &y) {
  if (x.size() != y.size() || x.empty())
    return 0.0;

  double mean_x = calculate_mean(x);
  double mean_y = calculate_mean(y);

  double sum_xy = 0.0;
  double sum_x2 = 0.0;
  double sum_y2 = 0.0;

  for (size_t i = 0; i < x.size(); ++i) {
    double dx = x[i] - mean_x;
    double dy = y[i] - mean_y;
    sum_xy += dx * dy;
    sum_x2 += dx * dx;
    sum_y2 += dy * dy;
  }

  double denom = std::sqrt(sum_x2 * sum_y2);
  if (denom == 0)
    return 0.0;

  return sum_xy / denom;
}

double calculate_beta(const std::vector<double> &asset_returns,
                      const std::vector<double> &market_returns) {
  double correlation = calculate_correlation(asset_returns, market_returns);
  double asset_std = calculate_standard_deviation(asset_returns);
  double market_std = calculate_standard_deviation(market_returns);

  if (market_std == 0)
    return 0.0;

  return correlation * (asset_std / market_std);
}

double annualize_return(double period_return, int periods_per_year) {
  return period_return * static_cast<double>(periods_per_year);
}

double annualize_volatility(double period_volatility, int periods_per_year) {
  return period_volatility * std::sqrt(static_cast<double>(periods_per_year));
}

} // namespace risk
