// test_amortization_schedule.cpp - Tests for AmortizationSchedule
#include "amortization_schedule.h"
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>

using namespace financing;
using Catch::Matchers::WithinAbs;
using Catch::Matchers::WithinRel;

TEST_CASE("AmortizationSchedule generate() returns non-empty schedule", "[amortization]")
{
  // 2-year annual-pay 5% bond, face = 1000
  AmortizationSchedule sched(1000.0, 0.05, 2.0, /*frequency=*/1);
  auto flows = sched.generate();

  REQUIRE_FALSE(flows.empty());
}

TEST_CASE("AmortizationSchedule 2yr annual 5% bond cash flows", "[amortization]")
{
  // Annual-pay bullet bond: 2 coupon payments + 1 principal redemption.
  AmortizationSchedule sched(1000.0, 0.05, 2.0, /*frequency=*/1);
  auto flows = sched.generate();

  SECTION("Exactly 3 cash flows (2 coupons + 1 redemption)") {
    REQUIRE(flows.size() == 3);
  }

  SECTION("Coupon flows carry interest = face * coupon_rate") {
    // Two annual coupon payments of 1000 * 5% = 50 each.
    double coupon_total = 0.0;
    for (const auto& cf : flows) coupon_total += cf.interest;
    REQUIRE_THAT(coupon_total, WithinAbs(100.0, 1.0)); // 2 × 50
  }

  SECTION("Redemption flow carries principal = face amount") {
    double principal_total = 0.0;
    for (const auto& cf : flows) principal_total += cf.principal;
    REQUIRE_THAT(principal_total, WithinAbs(1000.0, 1e-6));
  }

  SECTION("Period numbers are sequential starting at 1") {
    REQUIRE(flows[0].period == 1);
    REQUIRE(flows[1].period == 2);
    REQUIRE(flows[2].period == 3);
  }
}

TEST_CASE("AmortizationSchedule total_interest()", "[amortization]")
{
  // 2yr annual 5% on 1000: total interest = 2 × 50 = 100
  AmortizationSchedule sched(1000.0, 0.05, 2.0, 1);
  REQUIRE_THAT(sched.total_interest(), WithinAbs(100.0, 1.0));
}

TEST_CASE("AmortizationSchedule duration_modified() is in expected range", "[amortization]")
{
  // 2yr annual-pay 5% par bond (yield = coupon): modified duration ≈ 1.86yr.
  AmortizationSchedule sched(1000.0, 0.05, 2.0, 1);
  double d = sched.duration_modified(0.05);
  REQUIRE(d > 1.5);
  REQUIRE(d < 2.1);
}

TEST_CASE("AmortizationSchedule convexity() is positive", "[amortization]")
{
  AmortizationSchedule sched(1000.0, 0.05, 2.0, 1);
  double c = sched.convexity(0.05);
  REQUIRE(c > 0.0);
}

TEST_CASE("AmortizationSchedule semiannual bond has more cash flows", "[amortization]")
{
  // 2yr semiannual: 4 coupon payments + 1 redemption = 5 flows.
  AmortizationSchedule sched(1000.0, 0.05, 2.0, /*frequency=*/2);
  auto flows = sched.generate();
  REQUIRE(flows.size() == 5);
}

TEST_CASE("AmortizationSchedule accessors", "[amortization]")
{
  AmortizationSchedule sched(5000.0, 0.04, 3.0, 2);
  REQUIRE_THAT(sched.face_amount(),    WithinAbs(5000.0, 1e-9));
  REQUIRE_THAT(sched.coupon_rate(),    WithinAbs(0.04,   1e-9));
  REQUIRE_THAT(sched.maturity_years(), WithinAbs(3.0,    1e-9));
  REQUIRE(sched.frequency() == 2);
}
