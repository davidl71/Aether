// financing_optimizer.cpp - NLopt multi-instrument financing cost minimization (Phase 4)
#include "financing_optimizer.h"
#include <nlopt.hpp>
#include <spdlog/spdlog.h>
#include <stdexcept>

namespace financing {

namespace {

// Objective: minimize weighted sum of effective rates (blended cost)
double effective_cost_objective(unsigned n, const double* x, double* grad,
                               void* data) {
  const auto* rates = static_cast<const double*>(data);
  double cost = 0.0;
  for (unsigned i = 0; i < n; ++i) {
    cost += x[i] * rates[i];
  }
  if (grad) {
    for (unsigned i = 0; i < n; ++i) {
      grad[i] = rates[i];
    }
  }
  return cost;
}

// Constraint: weights sum to 1.0
double sum_weights_constraint(unsigned n, const double* x, double* grad,
                              void* /*data*/) {
  double sum = 0.0;
  for (unsigned i = 0; i < n; ++i) {
    sum += x[i];
  }
  sum -= 1.0;
  if (grad) {
    for (unsigned i = 0; i < n; ++i) {
      grad[i] = 1.0;
    }
  }
  return sum;
}

}  // namespace

FinancingOptimizerResult FinancingOptimizer::optimize(
    const FinancingOptimizerInput& input) const {
  FinancingOptimizerResult result{};
  result.success = false;

  constexpr unsigned n = static_cast<unsigned>(kNumInstruments);

  try {
    nlopt::opt opt(nlopt::LD_SLSQP, n);

    opt.set_min_objective(effective_cost_objective,
                          const_cast<double*>(input.effective_rates.data()));

    std::vector<double> lb(n, 0.0);
    std::vector<double> ub(n, 1.0);
    for (size_t i = 0; i < kNumInstruments; ++i) {
      if (input.min_weight[i] > 0.0) {
        lb[i] = input.min_weight[i];
      }
      // max_weight 0 or 1 means no upper bound (use 1.0); otherwise clamp
      if (input.max_weight[i] > 0.0 && input.max_weight[i] < 1.0) {
        ub[i] = input.max_weight[i];
      }
    }
    opt.set_lower_bounds(lb);
    opt.set_upper_bounds(ub);

    opt.add_equality_constraint(sum_weights_constraint, nullptr, 1e-8);

    opt.set_xtol_rel(input.xtol_rel);
    opt.set_ftol_rel(input.ftol_rel);

    std::vector<double> x(n, 1.0 / static_cast<double>(n));

    double minf = 0.0;
    nlopt::result opt_result = opt.optimize(x, minf);

    if (opt_result < 0) {
      // Greedy fallback: satisfy min_weight constraints first, then allocate
      // the remainder to the cheapest instrument.
      spdlog::warn("FinancingOptimizer: NLopt failed (code {}); using greedy fallback",
                   opt_result);
      std::fill(result.weights.begin(), result.weights.end(), 0.0);
      double min_sum = 0.0;
      for (size_t i = 0; i < kNumInstruments; ++i) {
        result.weights[i] = lb[i];
        min_sum += lb[i];
      }
      double remaining = 1.0 - min_sum;
      if (remaining > 0.0) {
        size_t best = 0;
        for (size_t i = 1; i < kNumInstruments; ++i) {
          if (input.effective_rates[i] < input.effective_rates[best]) {
            best = i;
          }
        }
        result.weights[best] += remaining;
      }
      result.effective_cost = 0.0;
      for (size_t i = 0; i < kNumInstruments; ++i) {
        result.effective_cost += result.weights[i] * input.effective_rates[i];
      }
      result.success = true;
      result.error_message = "NLopt failed (code " + std::to_string(opt_result) +
                             "); greedy fallback applied";
      return result;
    }

    for (size_t i = 0; i < kNumInstruments; ++i) {
      result.weights[i] = x[i];
    }
    result.effective_cost = minf;
    result.success = true;

    spdlog::debug(
        "FinancingOptimizer: effective_cost={:.4f}, weights=[{:.3f},{:.3f},"
        "{:.3f},{:.3f},{:.3f}]",
        result.effective_cost, result.weights[0], result.weights[1],
        result.weights[2], result.weights[3], result.weights[4]);

  } catch (const std::exception& e) {
    result.error_message = std::string("FinancingOptimizer error: ") + e.what();
    spdlog::error("{}", result.error_message);
  }

  return result;
}

}  // namespace financing
