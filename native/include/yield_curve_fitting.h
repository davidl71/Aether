// yield_curve_fitting.h - Nelson-Siegel yield curve fitting via QuantLib
#pragma once

#include "types.h"
#include <memory>
#include <vector>

namespace QuantLib {
  class FittedBondDiscountCurve;
}

namespace yield_curve {

// Fits a Nelson-Siegel smooth yield curve through a set of YieldCurvePoints
// (box-spread implied rates at various expiries) and exposes discount factors
// and zero rates for arbitrary maturities.
//
// Requires at least 3 data points for a well-determined fit.
// Returns nullptr from fit() when fewer than 3 points are provided.
class NelsonSiegelFitter {
public:
  NelsonSiegelFitter() = default;

  // Fit Nelson-Siegel curve to the provided yield curve points.
  // Returns a shared_ptr to the fitted QuantLib curve, or nullptr if the
  // fit fails or fewer than 3 data points are provided.
  std::shared_ptr<QuantLib::FittedBondDiscountCurve>
  fit(const std::vector<types::YieldCurvePoint>& points);

  // Evaluate the discount factor P(0, t) at a given maturity in years.
  // Requires a prior successful call to fit().  Returns 1.0 on error.
  double discount_factor(double maturity_years) const;

  // Evaluate the continuously compounded zero rate at a given maturity.
  // Requires a prior successful call to fit().  Returns 0.0 on error.
  double zero_rate(double maturity_years) const;

  bool has_curve() const { return curve_ != nullptr; }

private:
  std::shared_ptr<QuantLib::FittedBondDiscountCurve> curve_;
};

} // namespace yield_curve
