// amortization_schedule.h - Loan and bond amortization via QuantLib
#pragma once

#include <vector>

namespace QuantLib {
  class Date;
  class Schedule;
  class DayCounter;
}

namespace financing {

// A single cash flow entry in an amortization schedule.
struct CashFlow {
  int    period;       // Period number (1-based)
  double date_serial;  // QuantLib serial date (for ordering)
  double principal;    // Principal portion of this payment
  double interest;     // Interest/coupon portion of this payment
  double balance;      // Remaining principal after this payment
};

// Generates a full amortization schedule for a fixed-rate bullet bond or loan
// using QuantLib::FixedRateBond and BondFunctions.
//
// For standard bullet bonds (no scheduled amortization) the only principal
// payment appears at maturity; all intermediate payments are coupon-only.
class AmortizationSchedule {
public:
  // Construct from bond parameters.
  // face_amount   : notional / face value
  // coupon_rate   : annual coupon as a decimal (e.g. 0.05 for 5%)
  // maturity_years: years to maturity from today (rounded to whole months)
  // frequency     : coupon periods per year (2 = semiannual, 1 = annual)
  AmortizationSchedule(double face_amount,
                       double coupon_rate,
                       double maturity_years,
                       int    frequency = 2);

  // Generate the full schedule of cash flows.
  std::vector<CashFlow> generate() const;

  // Total interest paid over the life of the bond.
  double total_interest() const;

  // Modified duration at the given yield (decimal).
  // Returns 0.0 on error.
  double duration_modified(double yield) const;

  // Convexity at the given yield (decimal).
  // Returns 0.0 on error.
  double convexity(double yield) const;

  double face_amount()    const { return face_amount_; }
  double coupon_rate()    const { return coupon_rate_; }
  double maturity_years() const { return maturity_years_; }
  int    frequency()      const { return frequency_; }

private:
  double face_amount_;
  double coupon_rate_;
  double maturity_years_;
  int    frequency_;
};

} // namespace financing
