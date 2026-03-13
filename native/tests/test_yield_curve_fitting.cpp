// test_yield_curve_fitting.cpp - Tests for NelsonSiegelFitter
#include "yield_curve_fitting.h"
#include "types.h"
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>

using namespace yield_curve;
using Catch::Matchers::WithinAbs;

namespace {

// Build a synthetic YieldCurvePoint with the specified DTE and implied rate (%).
types::YieldCurvePoint make_point(int dte, double implied_rate_pct) {
  types::YieldCurvePoint pt{};
  pt.days_to_expiry = dte;
  pt.implied_rate   = implied_rate_pct;
  pt.effective_rate = implied_rate_pct;
  return pt;
}

// Four plausible box-spread implied rate observations.
std::vector<types::YieldCurvePoint> make_sample_points() {
  return {
    make_point( 30, 4.90),
    make_point( 90, 5.00),
    make_point(180, 5.10),
    make_point(360, 5.20),
  };
}

} // namespace

TEST_CASE("NelsonSiegelFitter rejects fewer than 3 data points", "[yield_curve]")
{
  NelsonSiegelFitter fitter;

  SECTION("Empty input returns nullptr") {
    REQUIRE(fitter.fit({}) == nullptr);
    REQUIRE_FALSE(fitter.has_curve());
  }

  SECTION("One point returns nullptr") {
    REQUIRE(fitter.fit({make_point(90, 5.0)}) == nullptr);
  }

  SECTION("Two points return nullptr") {
    auto pts = std::vector<types::YieldCurvePoint>{
        make_point(90, 5.0), make_point(180, 5.1)};
    REQUIRE(fitter.fit(pts) == nullptr);
  }
}

TEST_CASE("NelsonSiegelFitter fits 4-point curve and returns non-null", "[yield_curve]")
{
  NelsonSiegelFitter fitter;
  auto pts = make_sample_points();

  auto curve = fitter.fit(pts);
  REQUIRE(curve != nullptr);
  REQUIRE(fitter.has_curve());
}

TEST_CASE("NelsonSiegelFitter discount_factor sanity range", "[yield_curve]")
{
  NelsonSiegelFitter fitter;
  fitter.fit(make_sample_points());

  SECTION("6-month discount factor is in (0.97, 1.0)") {
    double df = fitter.discount_factor(0.5);
    REQUIRE(df > 0.97);
    REQUIRE(df < 1.0);
  }

  SECTION("1-year discount factor is in (0.94, 1.0)") {
    double df = fitter.discount_factor(1.0);
    REQUIRE(df > 0.94);
    REQUIRE(df < 1.0);
  }

  SECTION("Discount factors are monotonically decreasing with maturity") {
    double df_6m  = fitter.discount_factor(0.5);
    double df_1y  = fitter.discount_factor(1.0);
    REQUIRE(df_1y < df_6m);
  }
}

TEST_CASE("NelsonSiegelFitter zero_rate plausible range", "[yield_curve]")
{
  NelsonSiegelFitter fitter;
  fitter.fit(make_sample_points());

  SECTION("Zero rate at 1yr is near fitted implied rates (4-6%)") {
    double z = fitter.zero_rate(1.0);
    REQUIRE(z > 0.03);
    REQUIRE(z < 0.07);
  }
}

TEST_CASE("NelsonSiegelFitter without prior fit returns safe defaults", "[yield_curve]")
{
  NelsonSiegelFitter fitter;  // no fit() called

  SECTION("discount_factor returns 1.0") {
    REQUIRE_THAT(fitter.discount_factor(1.0), WithinAbs(1.0, 1e-9));
  }

  SECTION("zero_rate returns 0.0") {
    REQUIRE_THAT(fitter.zero_rate(1.0), WithinAbs(0.0, 1e-9));
  }

  SECTION("has_curve returns false") {
    REQUIRE_FALSE(fitter.has_curve());
  }
}
