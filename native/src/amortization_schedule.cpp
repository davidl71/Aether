// amortization_schedule.cpp - Loan and bond amortization via QuantLib
#include "amortization_schedule.h"
#include <spdlog/spdlog.h>
#include <ql/quantlib.hpp>
#include <ql/instruments/bonds/fixedratebond.hpp>
#include <ql/pricingengines/bond/bondfunctions.hpp>
#include <cmath>

using namespace QuantLib;

namespace financing {

namespace {

// Build a QuantLib FixedRateBond from the given parameters.
// Returns nullptr on failure.
ext::shared_ptr<FixedRateBond>
build_bond(double face, double coupon, double maturity_years, int frequency)
{
  try {
    Date today = Date::todaysDate();
    auto months = static_cast<Integer>(std::round(maturity_years * 12.0));
    if (months < 1) months = 1;
    Date maturity = today + months * Months;

    Frequency ql_freq = (frequency == 1) ? Annual : Semiannual;
    Period    period(ql_freq);

    Schedule schedule(today, maturity, period,
                      UnitedStates(UnitedStates::GovernmentBond),
                      Unadjusted, Unadjusted,
                      DateGeneration::Backward,
                      /*endOfMonth=*/false);

    return ext::make_shared<FixedRateBond>(
        /*settlementDays=*/0,
        face,
        schedule,
        std::vector<Rate>{coupon},
        ActualActual(ActualActual::ISMA));
  }
  catch (const std::exception& e) {
    spdlog::warn("amortization_schedule build_bond failed: {}", e.what());
    return nullptr;
  }
}

} // namespace

AmortizationSchedule::AmortizationSchedule(double face_amount,
                                           double coupon_rate,
                                           double maturity_years,
                                           int    frequency)
    : face_amount_(face_amount)
    , coupon_rate_(coupon_rate)
    , maturity_years_(maturity_years)
    , frequency_(frequency)
{}

std::vector<CashFlow> AmortizationSchedule::generate() const
{
  auto bond = build_bond(face_amount_, coupon_rate_, maturity_years_, frequency_);
  if (!bond) return {};

  std::vector<CashFlow> schedule;
  schedule.reserve(bond->cashflows().size());

  double remaining_balance = face_amount_;
  int    period            = 0;

  for (const auto& cf : bond->cashflows()) {
    if (cf->hasOccurred()) continue;

    ++period;
    CashFlow entry{};
    entry.period      = period;
    entry.date_serial = static_cast<double>(cf->date().serialNumber());

    // Distinguish coupon flows from the final redemption.
    if (auto coupon = ext::dynamic_pointer_cast<FixedRateCoupon>(cf)) {
      entry.interest = coupon->amount();
      entry.principal = 0.0;
    } else {
      // Redemption / bullet principal at maturity.
      entry.principal = cf->amount();
      entry.interest  = 0.0;
      remaining_balance -= entry.principal;
    }

    entry.balance = remaining_balance;
    schedule.push_back(entry);
  }

  return schedule;
}

double AmortizationSchedule::total_interest() const
{
  auto flows = generate();
  double total = 0.0;
  for (const auto& cf : flows) total += cf.interest;
  return total;
}

double AmortizationSchedule::duration_modified(double yield) const
{
  auto bond = build_bond(face_amount_, coupon_rate_, maturity_years_, frequency_);
  if (!bond) return 0.0;
  try {
    InterestRate y(yield,
                   ActualActual(ActualActual::ISMA),
                   Compounded,
                   (frequency_ == 1) ? Annual : Semiannual);
    return BondFunctions::duration(*bond, y, Duration::Modified);
  }
  catch (const std::exception& e) {
    spdlog::warn("AmortizationSchedule::duration_modified: {}", e.what());
    return 0.0;
  }
}

double AmortizationSchedule::convexity(double yield) const
{
  auto bond = build_bond(face_amount_, coupon_rate_, maturity_years_, frequency_);
  if (!bond) return 0.0;
  try {
    InterestRate y(yield,
                   ActualActual(ActualActual::ISMA),
                   Compounded,
                   (frequency_ == 1) ? Annual : Semiannual);
    return BondFunctions::convexity(*bond, y);
  }
  catch (const std::exception& e) {
    spdlog::warn("AmortizationSchedule::convexity: {}", e.what());
    return 0.0;
  }
}

} // namespace financing
