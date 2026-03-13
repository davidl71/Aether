// yield_curve_fitting.cpp - Nelson-Siegel yield curve fitting via QuantLib
#include "yield_curve_fitting.h"
#include <spdlog/spdlog.h>
#include <ql/quantlib.hpp>
#include <ql/termstructures/yield/fittedbonddiscountcurve.hpp>
#include <ql/termstructures/yield/nelsonsiegelfitting.hpp>
#include <ql/instruments/bonds/zerocouponbond.hpp>
#include <ql/pricingengines/bond/discountingbondengine.hpp>

using namespace QuantLib;

namespace yield_curve {

std::shared_ptr<FittedBondDiscountCurve>
NelsonSiegelFitter::fit(const std::vector<types::YieldCurvePoint>& points)
{
  curve_ = nullptr;

  if (points.size() < 3) {
    spdlog::warn("NelsonSiegelFitter::fit: need at least 3 yield curve points, got {}",
                 points.size());
    return nullptr;
  }

  try {
    Date today = Date::todaysDate();
    Settings::instance().evaluationDate() = today;
    DayCounter dc = ActualActual(ActualActual::ISDA);
    Calendar cal  = UnitedStates(UnitedStates::GovernmentBond);

    // Build zero-coupon bond helpers from the yield curve points.
    // Each YieldCurvePoint provides:
    //   days_to_expiry  → maturity
    //   implied_rate    → yield in percent (convert to decimal)
    std::vector<ext::shared_ptr<BondHelper>> helpers;
    helpers.reserve(points.size());

    for (const auto& pt : points) {
      if (pt.days_to_expiry <= 0 || pt.implied_rate <= 0.0) continue;

      Date maturity = today + pt.days_to_expiry * Days;
      // Price a zero-coupon bond at par discounted by the implied rate.
      double t       = dc.yearFraction(today, maturity);
      double rate    = pt.implied_rate / 100.0;
      double price   = 100.0 * std::exp(-rate * t);

      auto quote = ext::make_shared<SimpleQuote>(price);
      auto bond  = ext::make_shared<ZeroCouponBond>(
          /*settlementDays=*/0,
          cal,
          100.0,
          maturity,
          Unadjusted,
          100.0,
          today);

      helpers.push_back(ext::make_shared<BondHelper>(
          Handle<Quote>(quote), bond));
    }

    if (helpers.size() < 3) {
      spdlog::warn("NelsonSiegelFitter::fit: fewer than 3 valid helpers after filtering");
      return nullptr;
    }

    NelsonSiegelFitting fitting_method;
    auto curve = ext::make_shared<FittedBondDiscountCurve>(
        today,
        helpers,
        dc,
        fitting_method);
    curve->enableExtrapolation();

    // Force evaluation to detect any fitting error early.
    curve->discount(1.0);

    curve_ = std::shared_ptr<FittedBondDiscountCurve>(
        curve.get(), [curve](FittedBondDiscountCurve*) mutable { curve.reset(); });

    return curve_;
  }
  catch (const std::exception& e) {
    spdlog::warn("NelsonSiegelFitter::fit: QuantLib error: {}", e.what());
    curve_ = nullptr;
    return nullptr;
  }
}

double NelsonSiegelFitter::discount_factor(double maturity_years) const
{
  if (!curve_ || maturity_years <= 0.0) return 1.0;
  try {
    Date today    = curve_->referenceDate();
    auto months   = static_cast<Integer>(std::round(maturity_years * 12.0));
    Date maturity = today + std::max(months, Integer(1)) * Months;
    return curve_->discount(maturity);
  }
  catch (const std::exception& e) {
    spdlog::warn("NelsonSiegelFitter::discount_factor: {}", e.what());
    return 1.0;
  }
}

double NelsonSiegelFitter::zero_rate(double maturity_years) const
{
  if (!curve_ || maturity_years <= 0.0) return 0.0;
  try {
    DayCounter dc = ActualActual(ActualActual::ISDA);
    return curve_->zeroRate(maturity_years, dc, Continuous).rate();
  }
  catch (const std::exception& e) {
    spdlog::warn("NelsonSiegelFitter::zero_rate: {}", e.what());
    return 0.0;
  }
}

} // namespace yield_curve
