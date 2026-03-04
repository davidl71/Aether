#include "convexity_calculator.h"
#include <catch2/catch_test_macros.hpp>
#include <cmath>

using namespace convexity;

TEST_CASE("ConvexityCalculator - Portfolio Convexity Calculation",
          "[convexity]") {
  ConvexityCalculator calc;

  BondData short_bond;
  short_bond.duration = 2.5;
  short_bond.convexity = 5.0;
  short_bond.name = "SHY";

  BondData long_bond;
  long_bond.duration = 25.0;
  long_bond.convexity = 150.0;
  long_bond.name = "TLT";

  // Test equal weights (50/50)
  double convexity = calc.calculate_portfolio_convexity(
      0.5, short_bond.convexity, 0.5, long_bond.convexity);
  double expected = (0.5 * 5.0) + (0.5 * 150.0);
  REQUIRE(std::abs(convexity - expected) < 1e-10);
}

TEST_CASE("ConvexityCalculator - Weighted Duration Calculation",
          "[convexity]") {
  ConvexityCalculator calc;

  BondData short_bond;
  short_bond.duration = 2.5;
  short_bond.convexity = 5.0;

  BondData long_bond;
  long_bond.duration = 25.0;
  long_bond.convexity = 150.0;

  // Test equal weights
  double convexity =
      calc.calculate_current_convexity(0.5, short_bond, 0.5, long_bond);
  REQUIRE(convexity > 0.0);
}

TEST_CASE("ConvexityCalculator - Rebalancing Trigger", "[convexity]") {
  ConvexityCalculator calc;

  // Test: current convexity 10% above target should trigger rebalance
  REQUIRE(calc.should_rebalance(165.0, 150.0, 10.0) == true);

  // Test: current convexity 5% above target should NOT trigger rebalance
  REQUIRE(calc.should_rebalance(157.5, 150.0, 10.0) == false);

  // Test: current convexity 10% below target should trigger rebalance
  REQUIRE(calc.should_rebalance(135.0, 150.0, 10.0) == true);
}

TEST_CASE("ConvexityCalculator - Barbell Optimization", "[convexity]") {
  ConvexityCalculator calc;

  BondData short_bond;
  short_bond.duration = 2.5;
  short_bond.convexity = 5.0;
  short_bond.name = "SHY";

  BondData long_bond;
  long_bond.duration = 25.0;
  long_bond.convexity = 150.0;
  long_bond.name = "TLT";

  // Optimize for target duration of 10 years
  double target_duration = 10.0;
  OptimizationResult result =
      calc.optimize_barbell_allocation(short_bond, long_bond, target_duration);

  REQUIRE(result.success == true);
  REQUIRE(result.short_term_weight >= 0.0);
  REQUIRE(result.short_term_weight <= 1.0);
  REQUIRE(result.long_term_weight >= 0.0);
  REQUIRE(result.long_term_weight <= 1.0);

  // Weights should sum to 1.0 (within tolerance)
  REQUIRE(std::abs(result.short_term_weight + result.long_term_weight - 1.0) <
          1e-6);

  // Portfolio duration should be close to target (within 0.1 years)
  REQUIRE(std::abs(result.portfolio_duration - target_duration) < 0.1);

  // Portfolio convexity should be positive
  REQUIRE(result.portfolio_convexity > 0.0);
}
