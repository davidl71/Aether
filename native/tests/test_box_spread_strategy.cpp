// test_box_spread_strategy.cpp - Box spread strategy tests
#include "strategies/box_spread/box_spread_strategy.h"
#include "types.h"
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>

using namespace strategy;
using namespace types;

TEST_CASE("BoxSpreadValidator validates structure", "[strategy]") {
  BoxSpreadLeg spread;

  spread.long_call.symbol = "SPY";
  spread.long_call.strike = 500.0;
  spread.long_call.expiry = "20271219";
  spread.long_call.type = OptionType::Call;
  spread.long_call.exchange = "SMART";

  spread.short_call.symbol = "SPY";
  spread.short_call.strike = 510.0;
  spread.short_call.expiry = "20271219";
  spread.short_call.type = OptionType::Call;
  spread.short_call.exchange = "SMART";

  spread.long_put.symbol = "SPY";
  spread.long_put.strike = 510.0;
  spread.long_put.expiry = "20271219";
  spread.long_put.type = OptionType::Put;
  spread.long_put.exchange = "SMART";

  spread.short_put.symbol = "SPY";
  spread.short_put.strike = 500.0;
  spread.short_put.expiry = "20271219";
  spread.short_put.type = OptionType::Put;
  spread.short_put.exchange = "SMART";

  SECTION("Valid box spread structure") {
    REQUIRE(BoxSpreadValidator::validate_structure(spread));
  }

  SECTION("Valid strikes") {
    REQUIRE(BoxSpreadValidator::validate_strikes(spread));
  }

  SECTION("Valid expiries") {
    REQUIRE(BoxSpreadValidator::validate_expiries(spread));
  }

  SECTION("Valid symbols") {
    REQUIRE(BoxSpreadValidator::validate_symbols(spread));
  }
}

TEST_CASE("BoxSpreadValidator detects invalid configurations", "[strategy]") {
  BoxSpreadLeg spread;

  spread.long_call.symbol = "SPY";
  spread.long_call.strike = 500.0;
  spread.long_call.expiry = "20271219";
  spread.long_call.type = OptionType::Call;
  spread.long_call.exchange = "SMART";

  spread.short_call.symbol = "SPY";
  spread.short_call.strike = 510.0;
  spread.short_call.expiry = "20271219";
  spread.short_call.type = OptionType::Call;
  spread.short_call.exchange = "SMART";

  spread.long_put.symbol = "SPY";
  spread.long_put.strike = 510.0;
  spread.long_put.expiry = "20271219";
  spread.long_put.type = OptionType::Put;
  spread.long_put.exchange = "SMART";

  spread.short_put.symbol = "SPY";
  spread.short_put.strike = 500.0;
  spread.short_put.expiry = "20271219";
  spread.short_put.type = OptionType::Put;
  spread.short_put.exchange = "SMART";

  SECTION("Mismatched expiries fail validation") {
    spread.short_call.expiry = "20271226";
    REQUIRE_FALSE(BoxSpreadValidator::validate_expiries(spread));
  }

  SECTION("Mismatched symbols fail validation") {
    spread.long_put.symbol = "QQQ";
    REQUIRE_FALSE(BoxSpreadValidator::validate_symbols(spread));
  }

  SECTION("Invalid strikes fail validation") {
    spread.long_call.strike = 510.0; // Same as short call
    REQUIRE_FALSE(BoxSpreadValidator::validate_strikes(spread));
  }
}

TEST_CASE("BoxSpreadCalculator calculations", "[strategy][calculator]") {
  // Given: A box spread with strike width $10 (K1=500, K2=510) and known option
  // prices
  BoxSpreadLeg spread;

  spread.long_call.strike = 500.0;  // K1
  spread.short_call.strike = 510.0; // K2
  spread.long_put.strike = 510.0;   // K2
  spread.short_put.strike = 500.0;  // K1

  spread.long_call_price = 2.50;
  spread.short_call_price = 1.00;
  spread.long_put_price = 2.00;
  spread.short_put_price = 0.75;

  SECTION("Calculate theoretical value") {
    // When: We calculate the theoretical value
    double theoretical_value =
        BoxSpreadCalculator::calculate_theoretical_value(spread);

    // Then: Should equal strike width (K2 - K1 = 510 - 500 = 10.0)
    // This is a fundamental property of box spreads - they always equal strike
    // width at expiration
    REQUIRE_THAT(theoretical_value, Catch::Matchers::WithinRel(10.0, 0.001));
  }

  SECTION("Calculate net debit") {
    // When: We calculate the net debit
    double net_debit = BoxSpreadCalculator::calculate_net_debit(spread);

    // Then: Should equal sum of long positions minus sum of short positions
    // Formula: (2.50 - 1.00) + (2.00 - 0.75) = 1.50 + 1.25 = 2.75
    REQUIRE_THAT(net_debit, Catch::Matchers::WithinRel(2.75, 0.001));
  }

  SECTION("Calculate max profit") {
    // Given: Net debit and theoretical value are known
    spread.net_debit = 2.75;
    spread.theoretical_value = 10.0;

    // When: We calculate the maximum profit
    double max_profit = BoxSpreadCalculator::calculate_max_profit(spread);

    // Then: Should equal theoretical_value - net_debit = 10.0 - 2.75 = 7.25
    // This represents the arbitrage profit if held to expiration
    REQUIRE_THAT(max_profit, Catch::Matchers::WithinRel(7.25, 0.001));
  }

  SECTION("Calculate ROI") {
    // Given: Net debit and theoretical value are known
    spread.net_debit = 2.75;
    spread.theoretical_value = 10.0;

    // When: We calculate the ROI percentage
    double roi = BoxSpreadCalculator::calculate_roi(spread);

    // Then: Should equal (profit / net_debit) * 100 = (7.25 / 2.75) * 100 ≈
    // 263.6% This measures return relative to capital deployed
    REQUIRE(roi > 260.0);
    REQUIRE(roi < 265.0);
  }

  SECTION("Calculate commission") {
    // When: We calculate commission with $0.65 per contract fee
    double commission = BoxSpreadCalculator::calculate_commission(spread, 0.65);

    // Then: Should equal 4 legs * $0.65 = $2.60
    // Box spreads have 4 legs, so commission is 4x the per-contract fee
    REQUIRE_THAT(commission, Catch::Matchers::WithinRel(2.60, 0.001));
  }
}

TEST_CASE("BoxSpreadOpportunity actionable check", "[strategy]") {
  BoxSpreadOpportunity opp;

  SECTION("High confidence opportunity is actionable") {
    opp.confidence_score = 75.0;
    opp.expected_profit = 100.0;
    opp.execution_probability = 0.8;

    REQUIRE(opp.is_actionable());
  }

  SECTION("Low confidence opportunity is not actionable") {
    opp.confidence_score = 30.0;
    opp.expected_profit = 100.0;
    opp.execution_probability = 0.8;

    REQUIRE_FALSE(opp.is_actionable());
  }

  SECTION("Negative profit is not actionable") {
    opp.confidence_score = 75.0;
    opp.expected_profit = -10.0;
    opp.execution_probability = 0.8;

    REQUIRE_FALSE(opp.is_actionable());
  }

  SECTION("Low execution probability is not actionable") {
    opp.confidence_score = 75.0;
    opp.expected_profit = 100.0;
    opp.execution_probability = 0.5;

    REQUIRE_FALSE(opp.is_actionable());
  }
}

TEST_CASE("Helper functions for filtering opportunities", "[strategy]") {
  std::vector<BoxSpreadOpportunity> opportunities;

  for (int i = 0; i < 5; ++i) {
    BoxSpreadOpportunity opp;
    opp.expected_profit = 10.0 * static_cast<double>(i);
    opp.spread.roi_percent = 1.0 * static_cast<double>(i);
    opp.confidence_score = 50.0 + 10.0 * static_cast<double>(i);
    opportunities.push_back(opp);
  }

  SECTION("Filter by minimum profit") {
    auto filtered = filter_by_min_profit(opportunities, 15.0);
    REQUIRE(filtered.size() == 3); // 20, 30, 40
  }

  SECTION("Filter by minimum ROI") {
    auto filtered = filter_by_min_roi(opportunities, 2.0);
    REQUIRE(filtered.size() == 3); // 2%, 3%, 4%
  }

  SECTION("Sort by profit (descending)") {
    auto sorted = sort_opportunities_by_profit(opportunities);
    REQUIRE(sorted[0].expected_profit == 40.0);
    REQUIRE(sorted[4].expected_profit == 0.0);
  }

  SECTION("Sort by confidence (descending)") {
    auto sorted = sort_opportunities_by_confidence(opportunities);
    REQUIRE(sorted[0].confidence_score == 90.0);
    REQUIRE(sorted[4].confidence_score == 50.0);
  }
}
