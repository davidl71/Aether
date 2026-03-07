// convexity_calculator.cpp - Convexity optimization implementation
#include "convexity_calculator.h"
#include <Eigen/Dense>
#include <algorithm>
#include <cmath>
#include <nlopt.hpp>
#include <spdlog/spdlog.h>

namespace convexity {

// ============================================================================
// ConvexityCalculator Implementation
// ============================================================================

ConvexityCalculator::ConvexityCalculator() {
  spdlog::debug("ConvexityCalculator created");
}

ConvexityCalculator::~ConvexityCalculator() {
  spdlog::debug("ConvexityCalculator destroyed");
}

double ConvexityCalculator::calculate_portfolio_convexity(
    double short_term_weight, double short_term_convexity,
    double long_term_weight, double long_term_convexity_value) const {

  // Portfolio convexity = weighted average
  return (short_term_weight * short_term_convexity) +
         (long_term_weight * long_term_convexity_value);
}

double ConvexityCalculator::calculate_current_convexity(
    double short_term_weight, const BondData &short_term_bond,
    double long_term_weight, const BondData &long_term_bond) const {

  return calculate_portfolio_convexity(
      short_term_weight, short_term_bond.convexity, long_term_weight,
      long_term_bond.convexity);
}

double ConvexityCalculator::calculate_weighted_duration(
    double short_weight, double short_duration, double long_weight,
    double long_duration) const {

  return (short_weight * short_duration) + (long_weight * long_duration);
}

bool ConvexityCalculator::should_rebalance(double current_convexity,
                                           double target_convexity,
                                           double threshold_percent) const {

  if (target_convexity == 0.0) {
    return false;
  }

  double deviation_percent =
      std::abs((current_convexity - target_convexity) / target_convexity) *
      100.0;
  return deviation_percent >= threshold_percent;
}

// ============================================================================
// NLopt Objective Function
// ============================================================================

namespace {

// Objective: maximize portfolio convexity (minimize negative convexity)
double convexity_objective(unsigned n, const double *x, double *grad,
                           void *data) {
  // x[0] = short-term weight
  // x[1] = long-term weight (will be constrained to 1.0 - x[0])

  auto *bond_data = static_cast<std::pair<BondData, BondData> *>(data);
  const BondData &short_bond = bond_data->first;
  const BondData &long_bond = bond_data->second;

  // Portfolio convexity = weighted average
  double portfolio_convexity =
      x[0] * short_bond.convexity + x[1] * long_bond.convexity;

  // Gradient (if needed)
  if (grad) {
    grad[0] = short_bond.convexity;
    grad[1] = long_bond.convexity;
  }

  // Minimize negative convexity (maximize convexity)
  return -portfolio_convexity;
}

// Constraint: weights sum to 1.0
double weight_constraint(unsigned n, const double *x, double *grad,
                         void *data) {
  double sum = x[0] + x[1] - 1.0;

  if (grad) {
    grad[0] = 1.0;
    grad[1] = 1.0;
  }

  return sum; // Must equal 0
}

// Constraint: duration target
double duration_constraint(unsigned n, const double *x, double *grad,
                           void *data) {
  auto *constraint_data =
      static_cast<std::pair<std::pair<BondData, BondData>, double> *>(data);
  const BondData &short_bond = constraint_data->first.first;
  const BondData &long_bond = constraint_data->first.second;
  double target_duration = constraint_data->second;

  double portfolio_duration =
      x[0] * short_bond.duration + x[1] * long_bond.duration;
  double diff = portfolio_duration - target_duration;

  if (grad) {
    grad[0] = short_bond.duration;
    grad[1] = long_bond.duration;
  }

  return diff; // Must equal 0
}

} // namespace

OptimizationResult ConvexityCalculator::optimize_barbell_allocation(
    const BondData &short_term_bond, const BondData &long_term_bond,
    double target_duration, double target_convexity) const {

  OptimizationResult result{};
  result.success = false;

  try {
    // Create optimizer (2 variables: short_weight, long_weight)
    nlopt::opt opt(nlopt::LD_SLSQP, 2); // SLSQP for equality constraints

    // Prepare data for objective function
    std::pair<BondData, BondData> bond_data(short_term_bond, long_term_bond);
    opt.set_min_objective(convexity_objective, &bond_data);

    // Set bounds: weights between 0 and 1
    std::vector<double> lb(2, 0.0); // Lower bounds
    std::vector<double> ub(2, 1.0); // Upper bounds
    opt.set_lower_bounds(lb);
    opt.set_upper_bounds(ub);

    // Add constraint: weights sum to 1.0
    opt.add_equality_constraint(weight_constraint, nullptr, 1e-8);

    // Add constraint: duration target
    std::pair<std::pair<BondData, BondData>, double> duration_data(
        std::make_pair(short_term_bond, long_term_bond), target_duration);
    opt.add_equality_constraint(duration_constraint, &duration_data, 1e-8);

    // Set tolerance
    opt.set_xtol_rel(1e-4);
    opt.set_ftol_rel(1e-6);

    // Initial guess: equal weights
    std::vector<double> x(2);
    x[0] = 0.5; // Short-term weight
    x[1] = 0.5; // Long-term weight

    // Optimize
    double minf;
    nlopt::result opt_result = opt.optimize(x, minf);

    if (opt_result < 0) {
      result.error_message =
          "Optimization failed with code: " + std::to_string(opt_result);
      spdlog::warn("Convexity optimization failed: {}", result.error_message);
      return result;
    }

    // Extract results
    result.short_term_weight = x[0];
    result.long_term_weight = x[1];
    result.portfolio_convexity =
        -minf; // Negative because we minimized negative
    result.portfolio_duration = calculate_weighted_duration(
        result.short_term_weight, short_term_bond.duration,
        result.long_term_weight, long_term_bond.duration);
    result.success = true;

    spdlog::debug(
        "Convexity optimization: short_weight={:.4f}, long_weight={:.4f}, "
        "convexity={:.2f}, duration={:.2f}",
        result.short_term_weight, result.long_term_weight,
        result.portfolio_convexity, result.portfolio_duration);

  } catch (const std::exception &e) {
    result.error_message = std::string("Optimization error: ") + e.what();
    spdlog::error("Convexity optimization exception: {}", result.error_message);
  }

  return result;
}

} // namespace convexity
