// greeks_calculator.cpp - Greeks calculation using QuantLib
#include "greeks_calculator.h"
#include "market_hours.h"
#include <cmath>
#include <unordered_map>
#include <ql/instruments/payoffs.hpp>
#include <ql/instruments/bonds/fixedratebond.hpp>
#include <ql/pricingengines/blackcalculator.hpp>
#include <ql/pricingengines/bond/bondfunctions.hpp>
#include <ql/quantlib.hpp>
#include <spdlog/spdlog.h>

using namespace QuantLib;

namespace risk {

GreeksCalculator::GreeksCalculator() {
  spdlog::debug("GreeksCalculator initialized");
}

std::optional<double> GreeksCalculator::calculate_implied_vol(
    double market_price,
    double underlying_price,
    double strike,
    double risk_free_rate,
    double time_to_expiry,
    types::OptionType option_type) const
{
  if (market_price <= 0.0 || underlying_price <= 0.0 || strike <= 0.0 ||
      time_to_expiry <= 0.0) {
    spdlog::warn("calculate_implied_vol: invalid params price={} S={} K={} T={}",
                 market_price, underlying_price, strike, time_to_expiry);
    return std::nullopt;
  }

  constexpr double kMinVol  = 1e-7;
  constexpr double kMaxVol  = 10.0;
  constexpr double kTol     = 1e-8;
  constexpr int    kMaxIter = 100;

  Option::Type ql_type =
      (option_type == types::OptionType::Call) ? Option::Call : Option::Put;
  ext::shared_ptr<StrikedTypePayoff> payoff(
      new PlainVanillaPayoff(ql_type, strike));

  const double forward  = underlying_price * std::exp(risk_free_rate * time_to_expiry);
  const double discount = std::exp(-risk_free_rate * time_to_expiry);

  double sigma = 0.30; // initial guess: 30% annualised vol

  try {
    for (int i = 0; i < kMaxIter; ++i) {
      const double std_dev = sigma * std::sqrt(time_to_expiry);
      BlackCalculator calc(payoff, forward, std_dev, discount);

      const double diff = calc.value() - market_price;
      if (std::abs(diff) < kTol) {
        return sigma;
      }

      const double vega = calc.vega(time_to_expiry);
      if (std::abs(vega) < 1e-10) {
        spdlog::debug("calculate_implied_vol: vega near-zero at iter {}, price={} S={} K={}",
                      i, market_price, underlying_price, strike);
        break;
      }

      sigma = std::clamp(sigma - diff / vega, kMinVol, kMaxVol);
    }
  } catch (const std::exception& e) {
    spdlog::warn("calculate_implied_vol: QuantLib error: {}", e.what());
    return std::nullopt;
  }

  spdlog::debug("calculate_implied_vol: did not converge price={} S={} K={} T={}",
                market_price, underlying_price, strike, time_to_expiry);
  return std::nullopt;
}

std::optional<Greeks> GreeksCalculator::calculate_option_greeks(
    const types::OptionContract &contract, double underlying_price,
    double option_price, double risk_free_rate,
    double implied_volatility) const {

  if (underlying_price <= 0.0 || option_price <= 0.0 ||
      implied_volatility < 0.0) {
    spdlog::warn(
        "Invalid parameters for Greeks calculation: S={}, price={}, vol={}",
        underlying_price, option_price, implied_volatility);
    return std::nullopt;
  }

  // Calculate DTE
  market_hours::MarketHours market_hours;
  int dte = 0;
  if (!contract.expiry.empty()) {
    // Parse YYYYMMDD expiry
    if (contract.expiry.length() == 8) {
      try {
        int year = std::stoi(contract.expiry.substr(0, 4));
        int month = std::stoi(contract.expiry.substr(4, 2));
        int day = std::stoi(contract.expiry.substr(6, 2));

        std::tm tm_expiry = {};
        tm_expiry.tm_year = year - 1900;
        tm_expiry.tm_mon = month - 1;
        tm_expiry.tm_mday = day;
        tm_expiry.tm_hour = 16;
        tm_expiry.tm_min = 0;
        tm_expiry.tm_sec = 0;

        auto expiry_time =
            std::chrono::system_clock::from_time_t(std::mktime(&tm_expiry));
        auto now = std::chrono::system_clock::now();

        if (expiry_time > now) {
          auto current = now;
          while (current < expiry_time) {
            auto status = market_hours.get_market_status_at(current);
            if (status.current_session != market_hours::MarketSession::Closed) {
              dte++;
            } else if (!status.is_holiday && status.reason != "weekend") {
              dte++;
            }
            current += std::chrono::hours(24);
            if (dte > 365)
              break;
          }
        }
      } catch (const std::exception &e) {
        spdlog::warn("Failed to parse expiry {}: {}", contract.expiry,
                     e.what());
      }
    }
  }

  if (dte <= 0) {
    spdlog::warn("Invalid DTE for Greeks calculation: {}", dte);
    return std::nullopt;
  }

  double time_to_expiry = days_to_years(dte);
  if (time_to_expiry <= 0.0) {
    return std::nullopt;
  }

  try {
    // Set up QuantLib parameters
    Date today = Date::todaysDate();
    Calendar calendar = TARGET();
    DayCounter dayCounter = Actual365Fixed();

    // Create option type
    Option::Type ql_type =
        (contract.type == types::OptionType::Call) ? Option::Call : Option::Put;

    // Create payoff
    ext::shared_ptr<StrikedTypePayoff> payoff(
        new PlainVanillaPayoff(ql_type, contract.strike));

    // Calculate forward price (assuming no dividends for simplicity)
    // Forward = Spot * e^(r*T)
    double forward =
        underlying_price * std::exp(risk_free_rate * time_to_expiry);

    // Standard deviation = volatility * sqrt(time)
    double stdDev = implied_volatility * std::sqrt(time_to_expiry);

    // Discount factor = e^(-r*T)
    double discount = std::exp(-risk_free_rate * time_to_expiry);

    // Create BlackCalculator
    BlackCalculator blackCalc(payoff, forward, stdDev, discount);

    // Calculate Greeks
    Greeks greeks{};
    greeks.delta = blackCalc.delta(underlying_price);
    greeks.gamma = blackCalc.gamma(underlying_price);
    greeks.theta =
        blackCalc.theta(underlying_price, risk_free_rate) / 365.0; // Per day
    greeks.vega = blackCalc.vega(0.01) / 100.0; // Per 1% vol change
    greeks.rho = blackCalc.rho(0.01) / 100.0;   // Per 1% rate change

    return greeks;
  } catch (const std::exception &e) {
    spdlog::warn("QuantLib Greeks calculation failed: {}", e.what());
    return std::nullopt;
  }
}

Greeks GreeksCalculator::calculate_stock_greeks(int quantity) const {
  Greeks greeks{};
  // Stocks have delta = 1.0 per share, other Greeks = 0
  greeks.delta = static_cast<double>(quantity);
  greeks.gamma = 0.0;
  greeks.theta = 0.0;
  greeks.vega = 0.0;
  greeks.rho = 0.0;
  return greeks;
}

Greeks GreeksCalculator::calculate_bond_greeks(const std::string &symbol,
                                               double price, double quantity,
                                               double duration,
                                               double convexity) const {

  Greeks greeks{};

  // Bond Greeks based on duration and convexity
  // Delta ≈ 0 (bonds don't move 1:1 with stock index)
  // Alternative: Can use -Duration × Price for price sensitivity
  greeks.delta = 0.0; // Bonds don't have direct delta like stocks

  // Gamma: Convexity (second-order price sensitivity)
  // Formula: Gamma ≈ Convexity × Price × (Δyield)²
  // For Greeks calculation, use simplified convexity approximation
  greeks.gamma = convexity * price * 0.0001; // Approximate gamma

  // Vega = 0 (bonds don't have implied volatility)
  greeks.vega = 0.0;

  // Theta ≈ 0 (time decay minimal unless approaching maturity)
  greeks.theta = 0.0;

  // Rho: Interest rate sensitivity (dollar duration)
  // Formula: Rho = -Duration × Price
  greeks.rho = -duration * price;

  return greeks;
}

Greeks
GreeksCalculator::calculate_currency_greeks(double position_value_local,
                                            double fx_rate_usd,
                                            const std::string &currency) const {

  Greeks greeks{};

  // Currency Delta: Sensitivity to FX rate changes
  // Formula: Delta_currency = Position_Value_Local
  // This represents sensitivity to FX rate changes
  // For a 1 unit change in FX rate, the USD value changes by
  // position_value_local
  greeks.delta = position_value_local; // Sensitivity to FX rate changes

  // Other Greeks = 0 for currency positions
  greeks.gamma = 0.0;
  greeks.vega = 0.0;
  greeks.theta = 0.0;
  greeks.rho = 0.0;

  return greeks;
}

Greeks GreeksCalculator::aggregate_greeks(
    const std::vector<types::Position> &positions, double underlying_price,
    double risk_free_rate, double implied_volatility) const {

  Greeks aggregate{};

  for (const auto &pos : positions) {
    // Check if option or stock
    bool is_option = (pos.contract.expiry.length() == 8);

    if (is_option) {
      // Calculate option Greeks
      auto greeks_opt = calculate_option_greeks(
          pos.contract, underlying_price, pos.current_price, risk_free_rate,
          implied_volatility);

      if (greeks_opt) {
        // Multiply by quantity and add to aggregate
        aggregate.delta +=
            greeks_opt->delta * static_cast<double>(pos.quantity);
        aggregate.gamma +=
            greeks_opt->gamma * static_cast<double>(pos.quantity);
        aggregate.theta +=
            greeks_opt->theta * static_cast<double>(pos.quantity);
        aggregate.vega += greeks_opt->vega * static_cast<double>(pos.quantity);
        aggregate.rho += greeks_opt->rho * static_cast<double>(pos.quantity);
      }
    } else {
      // Stock position
      auto stock_greeks = calculate_stock_greeks(pos.quantity);
      aggregate.delta += stock_greeks.delta;
      // Other Greeks remain 0 for stocks
    }
  }

  return aggregate;
}

double GreeksCalculator::days_to_years(int days) const {
  return static_cast<double>(days) / 365.0;
}

namespace {

// ETF bond parameters: weighted-average representative bond used to derive
// modified duration and convexity via QuantLib::BondFunctions.
// Each entry encodes the ETF's published weighted-average maturity (years),
// coupon rate, and approximate yield used for the calculation.
struct EtfBondParams
{
  double years_to_maturity; // weighted-average years to final maturity
  double coupon_rate;       // weighted-average annual coupon (decimal)
  double yield;             // approximate current yield used as discount rate
  // Fallback values used when QuantLib construction fails.
  double fallback_duration;
  double fallback_convexity;
};

// Build a QuantLib FixedRateBond proxy and compute modified duration and
// convexity from first principles.  Returns false and leaves duration/convexity
// unchanged if construction or calculation throws.
bool compute_ql_duration_convexity(const EtfBondParams &p,
                                   double &duration_out,
                                   double &convexity_out)
{
  try
  {
    Date today = Date::todaysDate();
    // Settlement is T+1 for Treasuries; keep it simple at T+0.
    Date issue = today;
    // Round maturity to whole months from today.
    auto months = static_cast<Integer>(std::round(p.years_to_maturity * 12.0));
    if (months < 1)
      months = 1;
    Date maturity = today + months * Months;

    Schedule schedule(issue, maturity,
                      Period(Semiannual),
                      UnitedStates(UnitedStates::GovernmentBond),
                      Unadjusted, Unadjusted,
                      DateGeneration::Backward, false);

    FixedRateBond bond(0,                   // settlement days
                       100.0,               // face amount
                       schedule,
                       {p.coupon_rate},
                       ActualActual(ActualActual::ISMA));

    InterestRate yield(p.yield,
                       ActualActual(ActualActual::ISMA),
                       Compounded, Semiannual);

    duration_out  = BondFunctions::duration(bond, yield, Duration::Modified);
    convexity_out = BondFunctions::convexity(bond, yield);
    return true;
  }
  catch (const std::exception &e)
  {
    spdlog::warn("QuantLib ETF duration/convexity calculation failed: {}", e.what());
    return false;
  }
}

// Hardcoded fallback table — values kept in sync with EtfBondParams below.
// These are only used when the QuantLib path throws (e.g., bad date arithmetic
// near holidays).
const std::unordered_map<std::string, EtfBondParams> &etf_bond_params()
{
  static const std::unordered_map<std::string, EtfBondParams> kParams = {
    // TLT: iShares 20+ Year Treasury Bond ETF
    // ~25yr maturity, ~3% coupon, ~4.5% yield → modified duration ~18.5
    {"TLT", {25.0, 0.030, 0.045, 18.5, 350.0}},
    // SHY: iShares 1-3 Year Treasury Bond ETF
    // ~2yr maturity, ~5% coupon, ~4.8% yield → modified duration ~1.9
    {"SHY", { 2.0, 0.050, 0.048,  1.9,   5.0}},
    // IEF: iShares 7-10 Year Treasury Bond ETF
    // ~7yr maturity, ~3.5% coupon, ~4.3% yield → modified duration ~7.5
    {"IEF", { 7.0, 0.035, 0.043,  7.5,  70.0}},
    // BIL: SPDR Bloomberg 1-3 Month T-Bill ETF
    // ~0.25yr maturity, ~5.3% coupon, ~5.3% yield → modified duration ~0.08
    {"BIL", { 0.25, 0.053, 0.053,  0.08,   0.1}},
  };
  return kParams;
}

// Compute both duration and convexity in one QuantLib call; returns false for
// unknown symbols.
bool get_etf_duration_convexity(const std::string &symbol,
                                double &duration_out,
                                double &convexity_out)
{
  const auto &params = etf_bond_params();
  auto it = params.find(symbol);
  if (it == params.end())
    return false;

  const EtfBondParams &p = it->second;
  duration_out  = p.fallback_duration;
  convexity_out = p.fallback_convexity;
  compute_ql_duration_convexity(p, duration_out, convexity_out);
  return true;
}

double get_etf_duration(const std::string &symbol)
{
  double duration = 0.0, convexity = 0.0;
  get_etf_duration_convexity(symbol, duration, convexity);
  return duration;
}

double get_etf_convexity(const std::string &symbol)
{
  double duration = 0.0, convexity = 0.0;
  get_etf_duration_convexity(symbol, duration, convexity);
  return convexity;
}

} // anonymous namespace

} // namespace risk
