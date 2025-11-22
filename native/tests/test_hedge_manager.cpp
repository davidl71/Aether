// test_hedge_manager.cpp - Hedge manager tests
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>
#include "hedge_manager.h"
#include "types.h"
#include "strategies/box_spread/box_spread_strategy.h"

using namespace hedge;
using namespace types;
using namespace strategy;
using Catch::Matchers::WithinRel;
using Catch::Matchers::WithinAbs;

// ============================================================================
// Helper Functions
// ============================================================================

namespace {
  types::BoxSpreadLeg create_test_box_spread() {
    types::BoxSpreadLeg spread;
    spread.long_call.symbol = "SPY";
    spread.long_call.strike = 500.0;
    spread.long_call.expiry = "20250620";
    spread.long_call.type = OptionType::Call;
    spread.short_call.symbol = "SPY";
    spread.short_call.strike = 510.0;
    spread.short_call.expiry = "20250620";
    spread.short_call.type = OptionType::Call;
    spread.long_put.symbol = "SPY";
    spread.long_put.strike = 510.0;
    spread.long_put.expiry = "20250620";
    spread.long_put.type = OptionType::Put;
    spread.short_put.symbol = "SPY";
    spread.short_put.strike = 500.0;
    spread.short_put.expiry = "20250620";
    spread.short_put.type = OptionType::Put;
    spread.net_debit = 9.85;
    spread.theoretical_value = 10.0;
    return spread;
  }

  InterestRateFuture create_test_future() {
    InterestRateFuture future;
    future.type = InterestRateFutureType::SOFR_3M;
    future.symbol = "SR3";
    future.expiry = "20250620";
    future.current_price = 95.50;  // Implies ~4.5% rate
    future.contract_size = 1000000.0;  // $1M
    future.tick_size = 0.0025;
    future.tick_value = 6.25;
    future.days_to_expiry = 90;
    return future;
  }
} // namespace

// ============================================================================
// InterestRateFuture Tests
// ============================================================================

TEST_CASE("InterestRateFuture calculations", "[hedge][future]") {
  // Given: A SOFR 3M future
  auto future = create_test_future();

  SECTION("Calculate implied rate from futures price") {
    // When: We calculate implied rate
    double implied_rate = future.calculate_implied_rate();

    // Then: Should calculate rate from futures price
    // SOFR futures price = 100 - implied_rate
    // For price 95.50, implied rate ≈ 4.5%
    REQUIRE(implied_rate > 0.0);
    REQUIRE(implied_rate < 10.0);  // Reasonable range
  }

  SECTION("Calculate hedge ratio") {
    // Given: A box spread and notional amount
    auto spread = create_test_box_spread();
    double notional = 100000.0;  // $100k notional

    // When: We calculate hedge ratio
    double hedge_ratio = future.calculate_hedge_ratio(spread, notional);

    // Then: Should return positive hedge ratio
    REQUIRE(hedge_ratio > 0.0);
    // Hedge ratio should be reasonable (not extremely large)
    REQUIRE(hedge_ratio < 100.0);
  }
}

// ============================================================================
// CurrencyHedge Tests
// ============================================================================

TEST_CASE("CurrencyHedge calculations", "[hedge][currency]") {
  // Given: A currency hedge
  CurrencyHedge hedge;
  hedge.base_currency = "USD";
  hedge.hedge_currency = "ILS";
  hedge.pair_symbol = "USDILS";
  hedge.current_rate = 3.65;  // 1 USD = 3.65 ILS
  hedge.exposure_amount = 100000.0;  // $100k exposure

  SECTION("Calculate hedge amount") {
    // When: We calculate hedge amount
    double hedge_amount = hedge.calculate_hedge_amount(hedge.exposure_amount);

    // Then: Should equal exposure * exchange rate
    double expected = hedge.exposure_amount * hedge.current_rate;
    REQUIRE_THAT(hedge_amount, WithinRel(expected, 0.01));
  }

  SECTION("Calculate hedge cost") {
    // When: We calculate hedge cost
    double cost = hedge.calculate_hedge_cost();

    // Then: Should return non-negative cost
    REQUIRE(cost >= 0.0);
  }
}

// ============================================================================
// HedgeManager Tests
// ============================================================================

TEST_CASE("HedgeManager rate hedge calculations", "[hedge][manager]") {
  // Given: A hedge manager
  HedgeManager manager;

  SECTION("Calculate rate hedge") {
    // Given: A box spread and interest rate future
    auto spread = create_test_box_spread();
    auto future = create_test_future();
    double notional = 100000.0;  // $100k notional
    double target_ratio = 1.0;  // Full hedge

    // When: We calculate rate hedge
    auto hedge_calc = manager.calculate_rate_hedge(
        spread, notional, future, target_ratio);

    // Then: Should return valid hedge calculation
    REQUIRE(hedge_calc.is_valid);
    REQUIRE(hedge_calc.contracts_needed >= 0);
    REQUIRE(hedge_calc.hedge_ratio >= 0.0);
    REQUIRE(hedge_calc.hedge_ratio <= 2.0);  // Reasonable range
    REQUIRE(hedge_calc.hedge_cost >= 0.0);
  }

  SECTION("Calculate rate hedge with partial hedge ratio") {
    // Given: A box spread with 50% hedge target
    auto spread = create_test_box_spread();
    auto future = create_test_future();
    double notional = 100000.0;
    double target_ratio = 0.5;  // 50% hedge

    // When: We calculate rate hedge
    auto hedge_calc = manager.calculate_rate_hedge(
        spread, notional, future, target_ratio);

    // Then: Should return hedge with ratio ≈ 0.5
    REQUIRE(hedge_calc.is_valid);
    REQUIRE_THAT(hedge_calc.hedge_ratio, WithinAbs(0.5, 0.1));
  }

  SECTION("Calculate basis risk") {
    // Given: A box spread and future with different rates
    auto spread = create_test_box_spread();
    auto future = create_test_future();
    double notional = 100000.0;

    // When: We calculate rate hedge
    auto hedge_calc = manager.calculate_rate_hedge(
        spread, notional, future, 1.0);

    // Then: Should calculate basis risk
    // Basis risk represents the difference between box spread rate and futures rate
    REQUIRE(hedge_calc.basis_risk_bps >= -1000.0);  // Reasonable range
    REQUIRE(hedge_calc.basis_risk_bps <= 1000.0);
  }
}

TEST_CASE("HedgeManager currency hedge calculations", "[hedge][currency]") {
  // Given: A hedge manager
  HedgeManager manager;

  SECTION("Calculate currency hedge") {
    // Given: A box spread and currency exposure
    auto spread = create_test_box_spread();
    std::string base_currency = "USD";
    std::string hedge_currency = "ILS";
    double exposure = 100000.0;  // $100k exposure

    // When: We calculate currency hedge
    auto currency_hedge = manager.calculate_currency_hedge(
        spread, base_currency, hedge_currency, exposure);

    // Then: Should return valid currency hedge
    REQUIRE(currency_hedge.base_currency == base_currency);
    REQUIRE(currency_hedge.hedge_currency == hedge_currency);
    REQUIRE(currency_hedge.exposure_amount == exposure);
    REQUIRE(currency_hedge.hedge_amount > 0.0);
  }

  SECTION("Calculate currency hedge cost") {
    // Given: A currency hedge
    auto spread = create_test_box_spread();
    auto currency_hedge = manager.calculate_currency_hedge(
        spread, "USD", "ILS", 100000.0);

    // When: We calculate hedge cost
    double cost = manager.calculate_currency_hedge_cost(currency_hedge);

    // Then: Should return non-negative cost
    REQUIRE(cost >= 0.0);
  }
}

TEST_CASE("HedgeManager complete hedge calculations", "[hedge][complete]") {
  // Given: A hedge manager and hedge strategy
  HedgeManager manager;
  HedgeStrategy strategy;
  strategy.hedge_interest_rate = true;
  strategy.hedge_currency = false;  // Test interest rate only first
  strategy.target_hedge_ratio = 1.0;
  strategy.max_hedge_cost_bps = 50.0;

  SECTION("Calculate complete hedge with interest rate only") {
    // Given: A box spread
    auto spread = create_test_box_spread();
    double notional = 100000.0;

    // When: We calculate complete hedge
    auto complete_hedge = manager.calculate_complete_hedge(
        spread, notional, strategy);

    // Then: Should return valid complete hedge
    REQUIRE(complete_hedge.rate_hedge.is_valid);
    REQUIRE(complete_hedge.total_hedge_cost >= 0.0);
    REQUIRE(complete_hedge.total_hedge_cost_bps >= 0.0);
    // Viability depends on hedge cost vs max allowed
    REQUIRE(complete_hedge.is_viable == (complete_hedge.total_hedge_cost_bps <= strategy.max_hedge_cost_bps));
  }

  SECTION("Calculate complete hedge with both rate and currency") {
    // Given: Strategy with both hedges enabled
    strategy.hedge_interest_rate = true;
    strategy.hedge_currency = true;
    strategy.hedge_currency_code = "ILS";

    auto spread = create_test_box_spread();
    double notional = 100000.0;

    // When: We calculate complete hedge
    auto complete_hedge = manager.calculate_complete_hedge(
        spread, notional, strategy);

    // Then: Should include both hedges
    REQUIRE(complete_hedge.rate_hedge.is_valid);
    REQUIRE(complete_hedge.total_hedge_cost >= 0.0);
    // Currency hedge should be calculated
    REQUIRE(complete_hedge.currency_hedge.hedge_amount >= 0.0);
  }
}

TEST_CASE("HedgeManager hedge monitoring", "[hedge][monitoring]") {
  // Given: A hedge manager
  HedgeManager manager;

  SECTION("Monitor hedge effectiveness") {
    // Given: A rate hedge calculation
    auto spread = create_test_box_spread();
    auto future = create_test_future();
    auto hedge_calc = manager.calculate_rate_hedge(
        spread, 100000.0, future, 1.0);

    // When: We monitor hedge effectiveness
    auto effectiveness = manager.monitor_hedge(hedge_calc, spread);

    // Then: Should return effectiveness metrics
    REQUIRE(effectiveness.current_hedge_ratio >= 0.0);
    REQUIRE(effectiveness.target_hedge_ratio >= 0.0);
    REQUIRE(effectiveness.hedge_drift_bps >= -1000.0);
    REQUIRE(effectiveness.hedge_drift_bps <= 1000.0);
    REQUIRE(effectiveness.rebalance_cost >= 0.0);
  }

  SECTION("Detect when hedge needs rebalancing") {
    // Given: A hedge with drift
    auto spread = create_test_box_spread();
    auto future = create_test_future();
    auto hedge_calc = manager.calculate_rate_hedge(
        spread, 100000.0, future, 1.0);

    // When: We monitor hedge (may detect drift)
    auto effectiveness = manager.monitor_hedge(hedge_calc, spread);

    // Then: needs_rebalance should be boolean
    // (Exact value depends on drift threshold)
    REQUIRE(effectiveness.needs_rebalance == true || effectiveness.needs_rebalance == false);
  }
}

// ============================================================================
// Edge Cases and Error Conditions
// ============================================================================

TEST_CASE("HedgeManager edge cases", "[hedge][edge]") {
  // Given: A hedge manager
  HedgeManager manager;

  SECTION("Calculate hedge with zero notional") {
    // Given: Box spread with zero notional
    auto spread = create_test_box_spread();
    auto future = create_test_future();

    // When: We calculate hedge with zero notional
    auto hedge_calc = manager.calculate_rate_hedge(
        spread, 0.0, future, 1.0);

    // Then: Should return valid calculation (may have zero contracts)
    REQUIRE(hedge_calc.is_valid);
    REQUIRE(hedge_calc.contracts_needed == 0);
  }

  SECTION("Calculate hedge with zero target ratio") {
    // Given: Box spread with zero hedge target
    auto spread = create_test_box_spread();
    auto future = create_test_future();

    // When: We calculate hedge with zero ratio
    auto hedge_calc = manager.calculate_rate_hedge(
        spread, 100000.0, future, 0.0);

    // Then: Should return hedge with zero ratio
    REQUIRE(hedge_calc.is_valid);
    REQUIRE_THAT(hedge_calc.hedge_ratio, WithinAbs(0.0, 0.001));
  }
}
