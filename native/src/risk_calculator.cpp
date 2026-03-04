// risk_calculator.cpp - Risk management implementation
#include "risk_calculator.h"
#include "greeks_calculator.h"
#include "option_chain.h"
#include <Eigen/Dense>
#include <algorithm>
#include <cmath>
#include <iterator>
#include <numeric>
#include <spdlog/spdlog.h>

// NOTE FOR AUTOMATION AGENTS:
// Risk assessment logic is centralized here. The calculator exposes per-trade,
// per-position, and portfolio-level metrics that downstream components consume
// to enforce guardrails. Many quantitative formulas are currently placeholders
// to keep the integration surface stable—prefer implementing richer analytics
// in this module rather than duplicating math in strategy/order layers.

namespace risk {

namespace {

double total_abs_exposure(const std::vector<types::Position> &positions) {
  return std::accumulate(positions.begin(), positions.end(), 0.0,
                         [](double total, const types::Position &pos) {
                           return total + std::abs(pos.get_market_value());
                         });
}

} // namespace

// ============================================================================
// RiskCalculator::Impl
// ============================================================================

class RiskCalculator::Impl {
public:
  explicit Impl(const config::RiskConfig &config) : config_(config) {}

  // Copy of the active risk configuration; caller updates via `update_config`
  config::RiskConfig config_;
};

// ============================================================================
// RiskCalculator Implementation
// ============================================================================

RiskCalculator::RiskCalculator(const config::RiskConfig &config)
    : pimpl_(std::make_unique<Impl>(config)) {

  spdlog::debug("RiskCalculator created");
}

RiskCalculator::~RiskCalculator() { spdlog::debug("RiskCalculator destroyed"); }

PositionRisk
RiskCalculator::calculate_box_spread_risk(const types::BoxSpreadLeg &spread,
                                          double underlying_price,
                                          double implied_volatility) const {

  PositionRisk risk{};

  // Box spreads have defined risk
  risk.position_size = spread.net_debit * 100.0; // Per contract
  risk.max_loss = spread.net_debit * 100.0;
  risk.max_gain = (spread.get_strike_width() - spread.net_debit) * 100.0;
  risk.expected_value = risk.max_gain; // Box spreads converge to max value

  // Box spreads are delta-neutral
  risk.delta = 0.0;
  risk.gamma = 0.0;
  risk.theta = 0.0; // Minimal time decay
  risk.vega = 0.0;  // IV-neutral

  risk.leverage = 1.0;              // No leverage
  risk.probability_of_profit = 1.0; // Guaranteed if held to expiry

  if (risk.max_loss > 0) {
    risk.risk_reward_ratio = risk.max_gain / risk.max_loss;
  }

  // Initialize margin fields (will be calculated by margin calculator)
  risk.initial_margin = 0.0;
  risk.maintenance_margin = 0.0;
  risk.margin_utilization = 0.0;
  risk.margin_call_risk = false;
  risk.margin_timestamp = std::chrono::system_clock::now();

  spdlog::debug("Box spread risk: max_loss=${:.2f}, max_gain=${:.2f}",
                risk.max_loss, risk.max_gain);

  return risk;
}

PositionRisk
RiskCalculator::calculate_position_risk(const types::Position &position,
                                        double underlying_price,
                                        double implied_volatility) const {

  PositionRisk risk{};

  // NOTE: Full implementation would calculate Greeks and risk metrics

  risk.position_size = std::abs(position.get_market_value());
  risk.max_loss = risk.position_size;       // Simplified
  risk.max_gain = risk.position_size * 2.0; // Simplified

  return risk;
}

double
RiskCalculator::calculate_max_loss(const types::Position &position) const {
  // For long options: max loss is premium paid
  // For short options: max loss is theoretically unlimited (but managed)

  if (position.is_long()) {
    return position.get_cost_basis();
  } else {
    // Short positions - use a reasonable estimate
    return position.contract.strike * 100.0 *
           static_cast<double>(std::abs(position.quantity));
  }
}

double
RiskCalculator::calculate_max_gain(const types::Position &position) const {
  // Simplified calculation
  if (position.is_long()) {
    return position.contract.strike * 100.0 *
           static_cast<double>(position.quantity);
  } else {
    return position.get_cost_basis();
  }
}

PortfolioRisk RiskCalculator::calculate_portfolio_risk(
    const std::vector<types::Position> &positions,
    const types::AccountInfo &account) const {

  PortfolioRisk portfolio_risk{};

  // Calculate total exposure
  for (const auto &pos : positions) {
    portfolio_risk.total_exposure += std::abs(pos.get_market_value());
  }

  // Calculate aggregate Greeks
  auto greeks = calculate_aggregate_greeks(positions);
  portfolio_risk.total_delta = greeks.delta;
  portfolio_risk.total_gamma = greeks.gamma;
  portfolio_risk.total_theta = greeks.theta;
  portfolio_risk.total_vega = greeks.vega;

  // Simple VaR calculation (stub)
  portfolio_risk.var_95 = portfolio_risk.total_exposure * 0.05;
  portfolio_risk.var_99 = portfolio_risk.total_exposure * 0.10;

  spdlog::debug("Portfolio risk: exposure=${:.2f}, delta={:.2f}",
                portfolio_risk.total_exposure, portfolio_risk.total_delta);

  return portfolio_risk;
}

types::RiskMetrics RiskCalculator::calculate_aggregate_greeks(
    const std::vector<types::Position> &positions) const {

  types::RiskMetrics metrics{};

  if (positions.empty()) {
    return metrics;
  }

  // Calculate Greeks using GreeksCalculator
  GreeksCalculator greeks_calc;

  // Derive underlying price from positions.  For option positions the
  // strike price is the best proxy for the underlying level; when
  // current_price is available on a position we also weight by market
  // value to select the most representative underlying.
  double underlying_price = 100.0;
  double best_mv = 0.0;
  for (const auto &pos : positions) {
    double mv = std::abs(pos.get_market_value());
    if (mv > best_mv) {
      best_mv = mv;
      if (pos.contract.strike > 0.0) {
        underlying_price = pos.contract.strike;
      } else if (pos.current_price > 0.0) {
        underlying_price = pos.current_price * 100.0;
      }
    }
  }

  // Risk-free rate derived from box spread pricing when available.
  // The implied financing rate of a zero-arbitrage box equals the
  // risk-free rate.  Default to Fed Funds effective ≈ 4.5% (2025).
  double risk_free_rate = pimpl_->config_.risk_free_rate_override > 0.0
                              ? pimpl_->config_.risk_free_rate_override
                              : 0.045;

  // Implied volatility: aggregate from market data snapshots attached
  // to each position.  Weighted average by absolute market value.
  double implied_volatility = 0.20;
  double iv_weight_sum = 0.0;
  double iv_weighted = 0.0;
  for (const auto &pos : positions) {
    if (pos.current_price > 0.0 && pos.avg_price > 0.0) {
      double mv = std::abs(pos.get_market_value());
      double price_vol =
          std::abs(pos.current_price - pos.avg_price) / pos.avg_price;
      double annualized_vol = price_vol * std::sqrt(252.0);
      iv_weighted += annualized_vol * mv;
      iv_weight_sum += mv;
    }
  }
  if (iv_weight_sum > 0.0) {
    implied_volatility = std::clamp(iv_weighted / iv_weight_sum, 0.05, 1.5);
  }

  auto aggregate = greeks_calc.aggregate_greeks(
      positions, underlying_price, risk_free_rate, implied_volatility);

  metrics.delta = aggregate.delta;
  metrics.gamma = aggregate.gamma;
  metrics.theta = aggregate.theta;
  metrics.vega = aggregate.vega;
  metrics.rho = aggregate.rho;

  spdlog::debug("Aggregate Greeks: delta={:.4f}, gamma={:.4f}, theta={:.4f}, "
                "vega={:.4f}, rho={:.4f}",
                metrics.delta, metrics.gamma, metrics.theta, metrics.vega,
                metrics.rho);

  return metrics;
}

double RiskCalculator::calculate_correlation_risk(
    const std::vector<types::Position> &positions) const {

  if (positions.size() < 2) {
    return 0.0; // Need at least 2 positions for correlation
  }

  // Use Eigen MatrixXd for correlation matrix
  // For now, we'll use a simplified approach based on underlying symbols
  // In a full implementation, we'd use historical returns to calculate
  // correlation

  size_t n = positions.size();
  Eigen::MatrixXd correlation_matrix = Eigen::MatrixXd::Identity(n, n);

  // Calculate correlation based on underlying symbols
  // Same underlying = 1.0, different = calculate from historical returns if
  // available For now, use simplified approach with same-underlying detection

  // Step 1: Same underlying detection (correlation = 1.0)
  for (size_t i = 0; i < n; ++i) {
    for (size_t j = i + 1; j < n; ++j) {
      const auto &pos1 = positions[i];
      const auto &pos2 = positions[j];

      // Check if same underlying
      if (pos1.contract.symbol == pos2.contract.symbol) {
        correlation_matrix(i, j) = 1.0;
        correlation_matrix(j, i) = 1.0;
      } else {
        // Different underlyings - calculate from historical returns
        // For now, use simplified correlation based on price movements
        // In production, would fetch historical data from TWS API

        // Simplified: Use price correlation if both have current prices
        if (pos1.current_price > 0.0 && pos2.current_price > 0.0 &&
            pos1.avg_price > 0.0 && pos2.avg_price > 0.0) {
          // Calculate returns from entry to current
          double ret1 = (pos1.current_price - pos1.avg_price) / pos1.avg_price;
          double ret2 = (pos2.current_price - pos2.avg_price) / pos2.avg_price;

          // Simple correlation estimate (would need more data points in
          // production) For now, use sign correlation: if both moved same
          // direction, positive correlation
          if ((ret1 > 0 && ret2 > 0) || (ret1 < 0 && ret2 < 0)) {
            correlation_matrix(i, j) = 0.7; // Positive correlation
          } else {
            correlation_matrix(i, j) = 0.3; // Lower correlation
          }
        } else {
          // Fallback: Use default correlation for different underlyings
          correlation_matrix(i, j) = 0.5;
        }
        correlation_matrix(j, i) = correlation_matrix(i, j); // Symmetric
      }
    }
  }

  // Historical price returns for a full correlation matrix require async
  // TWS reqHistoricalData calls that belong in a data-pipeline layer, not
  // in the synchronous risk check path.  The sign-correlation estimator
  // above is used when live prices are available; callers that need a
  // richer matrix should pre-populate Position::historical_returns and
  // call calculate_correlation_from_returns() (below) instead.

  // Calculate portfolio variance using Eigen: w^T * C * w
  // where w = position weights, C = correlation matrix
  Eigen::VectorXd weights(n);
  double total_value = 0.0;

  // Calculate total portfolio value
  for (size_t i = 0; i < n; ++i) {
    total_value += std::abs(positions[i].get_market_value());
  }

  if (total_value == 0.0) {
    return 0.0;
  }

  // Calculate normalized weights
  for (size_t i = 0; i < n; ++i) {
    weights(i) = std::abs(positions[i].get_market_value()) / total_value;
  }

  // Portfolio variance = w^T * C * w
  Eigen::VectorXd temp = correlation_matrix * weights;
  double portfolio_variance = weights.transpose() * temp;

  // Correlation risk metric: normalized portfolio variance
  // Higher variance = higher correlation risk
  double correlation_risk = std::sqrt(portfolio_variance);

  spdlog::debug(
      "Correlation risk: portfolio_variance={:.6f}, correlation_risk={:.6f}",
      portfolio_variance, correlation_risk);

  return correlation_risk;
}

Eigen::MatrixXd RiskCalculator::calculate_correlation_from_returns(
    const std::vector<std::vector<double>> &returns_matrix) const {

  if (returns_matrix.empty()) {
    return Eigen::MatrixXd::Identity(1, 1);
  }

  size_t n_assets = returns_matrix.size();
  size_t n_obs = returns_matrix[0].size();

  if (n_obs < 2) {
    return Eigen::MatrixXd::Identity(n_assets, n_assets);
  }

  // Compute means
  std::vector<double> means(n_assets, 0.0);
  for (size_t i = 0; i < n_assets; ++i) {
    means[i] = std::accumulate(returns_matrix[i].begin(),
                               returns_matrix[i].end(), 0.0) /
               n_obs;
  }

  // Compute standard deviations and covariance matrix
  Eigen::MatrixXd cov(n_assets, n_assets);
  std::vector<double> stds(n_assets, 0.0);

  for (size_t i = 0; i < n_assets; ++i) {
    double var = 0.0;
    for (size_t t = 0; t < n_obs; ++t) {
      double d = returns_matrix[i][t] - means[i];
      var += d * d;
    }
    stds[i] = std::sqrt(var / (n_obs - 1));
  }

  for (size_t i = 0; i < n_assets; ++i) {
    for (size_t j = i; j < n_assets; ++j) {
      double covar = 0.0;
      for (size_t t = 0; t < n_obs; ++t) {
        covar += (returns_matrix[i][t] - means[i]) *
                 (returns_matrix[j][t] - means[j]);
      }
      covar /= (n_obs - 1);

      double corr = (stds[i] > 1e-12 && stds[j] > 1e-12)
                        ? covar / (stds[i] * stds[j])
                        : (i == j ? 1.0 : 0.0);

      cov(i, j) = std::clamp(corr, -1.0, 1.0);
      cov(j, i) = cov(i, j);
    }
  }

  spdlog::debug("Computed {0}x{0} correlation matrix from {1} observations",
                n_assets, n_obs);
  return cov;
}

bool RiskCalculator::is_within_limits(
    const types::Position &position,
    const std::vector<types::Position> &existing_positions) const {

  double total_exposure = total_abs_exposure(existing_positions) +
                          std::abs(position.get_market_value());

  return total_exposure <= pimpl_->config_.max_total_exposure;
}

bool RiskCalculator::is_box_spread_within_limits(
    const types::BoxSpreadLeg &spread,
    const std::vector<types::Position> &existing_positions) const {

  double position_cost = spread.net_debit * 100.0;

  double total_exposure = total_abs_exposure(existing_positions);

  return (total_exposure + position_cost) <=
             pimpl_->config_.max_total_exposure &&
         static_cast<int>(existing_positions.size()) <
             pimpl_->config_.max_positions;
}

double RiskCalculator::calculate_remaining_capacity(
    const std::vector<types::Position> &positions, double account_value) const {

  double total_exposure = total_abs_exposure(positions);

  double max_allowed =
      std::min(pimpl_->config_.max_total_exposure,
               account_value * pimpl_->config_.position_size_percent);

  return max_allowed - total_exposure;
}

bool RiskCalculator::would_exceed_limits(
    const types::Position &new_position,
    const std::vector<types::Position> &existing_positions,
    double account_value) const {

  return !is_within_limits(new_position, existing_positions);
}

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

// ============================================================================
// VaR Calculations
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
/// Expected behavior:
/// - Returns 0.0 if returns vector is empty
/// - Sorts returns in ascending order
/// - Selects the return at the (1 - confidence_level) percentile
/// - Returns negative of selected return (VaR is typically positive)
/// - For 95% confidence, selects 5th percentile (worst 5% of returns)
///
/// @param returns Vector of historical returns
/// @param confidence_level Confidence level (e.g., 0.95 for 95%)
/// @return VaR value (positive number representing potential loss)
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
/// Algorithm: Parametric VaR assumes returns follow a normal distribution.
/// Uses the mean, standard deviation, and z-score to estimate potential loss.
///
/// Formula: VaR = position_value * z_score * volatility * sqrt(time_horizon /
/// 252)
///
/// Z-scores:
/// - 95% confidence: z = 1.645
/// - 99% confidence: z = 2.326
///
/// Expected behavior:
/// - Scales volatility by square root of time (square root of time rule)
/// - Assumes 252 trading days per year
/// - Returns positive value representing potential loss
/// - Higher confidence level = higher VaR (more conservative)
///
/// @param expected_return Expected return (mean)
/// @param volatility Standard deviation of returns
/// @param position_value Current value of position
/// @param confidence_level Confidence level (0.95 or 0.99)
/// @param time_horizon_days Time horizon in days (default: 1)
/// @return VaR value (positive number representing potential loss)
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double RiskCalculator::calculate_var_parametric(double expected_return,
                                                double volatility,
                                                double position_value,
                                                double confidence_level,
                                                int time_horizon_days) const {

  // VaR = position_value * z_score * volatility * sqrt(time_horizon)

  // Z-scores for common confidence levels
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

// ============================================================================
// Greeks Calculations
// ============================================================================

double RiskCalculator::calculate_box_spread_delta(
    const types::BoxSpreadLeg &spread) const {

  // Box spreads are delta-neutral
  return 0.0;
}

types::RiskMetrics
RiskCalculator::calculate_box_spread_greeks(const types::BoxSpreadLeg &spread,
                                            double underlying_price,
                                            double volatility) const {

  types::RiskMetrics metrics;

  // Box spreads are Greek-neutral
  metrics.delta = 0.0;
  metrics.gamma = 0.0;
  metrics.theta = 0.0;
  metrics.vega = 0.0;
  metrics.rho = 0.0;

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
/// Algorithm: Sharpe ratio measures excess return per unit of risk
/// (volatility). Higher Sharpe ratio indicates better risk-adjusted
/// performance.
///
/// Formula: Sharpe = (mean_return - risk_free_rate) / standard_deviation
///
/// Expected behavior:
/// - Returns 0.0 if returns vector is empty
/// - Returns 0.0 if standard deviation is zero (division by zero protection)
/// - Positive value: Returns exceed risk-free rate
/// - Negative value: Returns below risk-free rate
/// - Higher value: Better risk-adjusted performance
///
/// Interpretation:
/// - Sharpe > 1: Good
/// - Sharpe > 2: Very good
/// - Sharpe > 3: Excellent
///
/// @param returns Vector of historical returns
/// @param risk_free_rate Risk-free rate (e.g., T-bill rate)
/// @return Sharpe ratio
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
/// Algorithm: Sortino ratio is similar to Sharpe ratio but only considers
/// downside volatility (negative returns). This makes it more appropriate
/// for strategies where upside volatility is desirable.
///
/// Formula: Sortino = (mean_return - risk_free_rate) / downside_deviation
///   where downside_deviation = std_dev of negative returns only
///
/// Expected behavior:
/// - Returns 0.0 if returns vector is empty
/// - Returns infinity if no negative returns (perfect downside protection)
/// - Returns 0.0 if downside deviation is zero
/// - Higher value: Better downside risk-adjusted performance
///
/// Interpretation:
/// - Sortino > Sharpe: Strategy has favorable upside volatility
/// - Sortino < Sharpe: Strategy has unfavorable upside volatility
///
/// @param returns Vector of historical returns
/// @param risk_free_rate Risk-free rate (e.g., T-bill rate)
/// @return Sortino ratio (or infinity if no downside risk)
///
/// @see ALGORITHMS_AND_BEHAVIOR.md for detailed algorithm documentation
double
RiskCalculator::calculate_sortino_ratio(const std::vector<double> &returns,
                                        double risk_free_rate) const {

  if (returns.empty())
    return 0.0;

  double mean_return = calculate_mean(returns);

  // Calculate downside deviation (only negative returns)
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

  // Calculate excess returns
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

  // NOTE: Would calculate days in drawdown
  return 0; // Stub
}

// ============================================================================
// Configuration
// ============================================================================

void RiskCalculator::update_config(const config::RiskConfig &config) {
  pimpl_->config_ = config;
  spdlog::info("Risk configuration updated");
}

config::RiskConfig RiskCalculator::get_config() const {
  return pimpl_->config_;
}

// ============================================================================
// RiskMonitor Implementation
// ============================================================================

RiskMonitor::RiskMonitor(const config::RiskConfig &config) : config_(config) {}

std::vector<RiskAlert>
RiskMonitor::check_risks(const std::vector<types::Position> &positions,
                         const types::AccountInfo &account) const {

  std::vector<RiskAlert> alerts;

  // Check exposure
  double total_exposure = total_abs_exposure(positions);

  if (total_exposure > config_.max_total_exposure * 0.9) {
    RiskAlert alert;
    alert.level = RiskAlertLevel::Warning;
    alert.category = "EXPOSURE";
    alert.message = "Approaching maximum exposure limit";
    alert.timestamp = std::chrono::system_clock::now();
    alerts.push_back(alert);
  }

  return alerts;
}

std::vector<RiskAlert>
RiskMonitor::check_position_risks(const types::Position &position) const {

  return {}; // Stub
}

std::vector<RiskAlert>
RiskMonitor::check_portfolio_risks(const PortfolioRisk &portfolio_risk,
                                   const types::AccountInfo &account) const {

  return {}; // Stub
}

// ============================================================================
// Helper Functions
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
