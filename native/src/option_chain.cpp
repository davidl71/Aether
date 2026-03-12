// option_chain.cpp - Option chain implementation
#include "option_chain.h"
#include "market_hours.h"
#include <algorithm>
#include <cmath>
#include <iterator>
#include <numeric>
#include <ql/instruments/vanillaoption.hpp>
#include <ql/pricingengines/blackformula.hpp>
#include <ql/pricingengines/vanilla/analyticeuropeanengine.hpp>
#include <ql/processes/blackscholesprocess.hpp>
#include <ql/quantlib.hpp>
#include <ql/termstructures/volatility/equityfx/blackconstantvol.hpp>
#include <ql/termstructures/yield/flatforward.hpp>
#include <spdlog/spdlog.h>

using namespace QuantLib;

// NOTE FOR AUTOMATION AGENTS:
// Option chain utilities live here. They transform raw market data into
// structures
// (`OptionChain`, `ExpiryChain`, `StrikeChain`) consumed by the strategy layer.
// Many quantitative routines remain stubbed; comments mark where
// production-grade Greeks/IV/DTE calculations should be wired in. Keep
// data-shaping logic centralized in this module so trading components can stay
// agnostic to quote representation.

namespace option_chain {

// ============================================================================
// OptionChainEntry Implementation
// ============================================================================

bool OptionChainEntry::is_valid() const {
  return contract.is_valid() && market_data.bid > 0 && market_data.ask > 0 &&
         market_data.ask >= market_data.bid;
}

bool OptionChainEntry::meets_liquidity_requirements(int min_volume,
                                                    int min_oi) const {
  return volume >= min_volume && open_interest >= min_oi;
}

// ============================================================================
// StrikeChain Implementation
// ============================================================================

double StrikeChain::get_call_iv() const {
  if (call.has_value() && call->market_data.implied_volatility.has_value()) {
    return call->market_data.implied_volatility.value();
  }
  return 0.0;
}

double StrikeChain::get_put_iv() const {
  if (put.has_value() && put->market_data.implied_volatility.has_value()) {
    return put->market_data.implied_volatility.value();
  }
  return 0.0;
}

double StrikeChain::get_iv_skew() const { return get_put_iv() - get_call_iv(); }

// ============================================================================
// ExpiryChain Implementation
// ============================================================================

ExpiryChain::ExpiryChain(const std::string &symbol, const std::string &expiry)
    : symbol_(symbol), expiry_(expiry) {}

void ExpiryChain::add_option(const OptionChainEntry &entry) {
  double strike = entry.contract.strike;

  auto &strike_chain = strikes_[strike];
  strike_chain.strike = strike;

  if (entry.contract.type == types::OptionType::Call) {
    strike_chain.call = entry;
  } else {
    strike_chain.put = entry;
  }
}

int ExpiryChain::get_days_to_expiry() const {
  if (expiry_.empty()) {
    return 0;
  }

  // Parse YYYYMMDD expiry string
  if (expiry_.length() != 8) {
    spdlog::warn("Invalid expiry format: {} (expected YYYYMMDD)", expiry_);
    return 0;
  }

  try {
    int year = std::stoi(expiry_.substr(0, 4));
    int month = std::stoi(expiry_.substr(4, 2));
    int day = std::stoi(expiry_.substr(6, 2));

    // Create expiry date
    std::tm tm_expiry = {};
    tm_expiry.tm_year = year - 1900;
    tm_expiry.tm_mon = month - 1;
    tm_expiry.tm_mday = day;
    tm_expiry.tm_hour = 16; // Market close (4:00 PM ET)
    tm_expiry.tm_min = 0;
    tm_expiry.tm_sec = 0;

    auto expiry_time =
        std::chrono::system_clock::from_time_t(std::mktime(&tm_expiry));
    auto now = std::chrono::system_clock::now();

    // Count trading days using MarketHours
    market_hours::MarketHours market_hours;
    int trading_days = 0;
    auto current = now;

    // If expiry is in the past, return 0
    if (expiry_time <= now) {
      return 0;
    }

    // Count trading days (weekdays excluding holidays)
    while (current < expiry_time) {
      auto status = market_hours.get_market_status_at(current);
      // Trading day = weekday and not holiday
      if (status.current_session != market_hours::MarketSession::Closed ||
          (!status.is_holiday && status.reason != "weekend")) {
        trading_days++;
      }
      current += std::chrono::hours(24);

      // Safety limit to prevent infinite loops
      if (trading_days > 365) {
        spdlog::warn("DTE calculation exceeded 365 days for expiry: {}",
                     expiry_);
        break;
      }
    }

    return trading_days;
  } catch (const std::exception &e) {
    spdlog::warn("Failed to calculate DTE for expiry {}: {}", expiry_,
                 e.what());
    return 0;
  }
}

std::vector<double> ExpiryChain::get_strikes() const {
  std::vector<double> strikes;
  for (const auto &[strike, _] : strikes_) {
    strikes.push_back(strike);
  }
  std::sort(strikes.begin(), strikes.end());
  return strikes;
}

std::optional<StrikeChain> ExpiryChain::get_strike_chain(double strike) const {
  auto it = strikes_.find(strike);
  if (it != strikes_.end()) {
    return it->second;
  }
  return std::nullopt;
}

std::optional<OptionChainEntry>
ExpiryChain::get_option(double strike, types::OptionType type) const {

  auto it = strikes_.find(strike);
  if (it != strikes_.end()) {
    if (type == types::OptionType::Call) {
      return it->second.call;
    } else {
      return it->second.put;
    }
  }
  return std::nullopt;
}

std::vector<double> ExpiryChain::get_strikes_in_range(double min_strike,
                                                      double max_strike) const {

  std::vector<double> strikes;
  for (const auto &[strike, _] : strikes_) {
    if (strike >= min_strike && strike <= max_strike) {
      strikes.push_back(strike);
    }
  }
  std::sort(strikes.begin(), strikes.end());
  return strikes;
}

std::optional<double>
ExpiryChain::find_atm_strike(double underlying_price) const {
  if (strikes_.empty()) {
    return std::nullopt;
  }

  double closest_strike = 0;
  double min_diff = std::numeric_limits<double>::max();

  for (const auto &[strike, _] : strikes_) {
    double diff = std::abs(strike - underlying_price);
    if (diff < min_diff) {
      min_diff = diff;
      closest_strike = strike;
    }
  }

  return closest_strike;
}

std::vector<OptionChainEntry> ExpiryChain::get_all_options() const {
  std::vector<OptionChainEntry> options;
  for (const auto &[_, chain] : strikes_) {
    if (chain.call.has_value()) {
      options.push_back(chain.call.value());
    }
    if (chain.put.has_value()) {
      options.push_back(chain.put.value());
    }
  }
  return options;
}

std::vector<OptionChainEntry> ExpiryChain::get_calls() const {
  std::vector<OptionChainEntry> calls;
  for (const auto &[_, chain] : strikes_) {
    if (chain.call.has_value()) {
      calls.push_back(chain.call.value());
    }
  }
  return calls;
}

std::vector<OptionChainEntry> ExpiryChain::get_puts() const {
  std::vector<OptionChainEntry> puts;
  for (const auto &[_, chain] : strikes_) {
    if (chain.put.has_value()) {
      puts.push_back(chain.put.value());
    }
  }
  return puts;
}

std::vector<OptionChainEntry>
ExpiryChain::filter_by_liquidity(int min_volume, int min_open_interest) const {

  auto options = get_all_options();
  std::vector<OptionChainEntry> filtered;
  filtered.reserve(options.size());

  std::copy_if(options.begin(), options.end(), std::back_inserter(filtered),
               [min_volume, min_open_interest](const OptionChainEntry &option) {
                 return option.meets_liquidity_requirements(min_volume,
                                                            min_open_interest);
               });

  return filtered;
}

std::vector<OptionChainEntry>
ExpiryChain::filter_by_moneyness(double min_moneyness,
                                 double max_moneyness) const {

  auto options = get_all_options();
  std::vector<OptionChainEntry> filtered;
  filtered.reserve(options.size());

  std::copy_if(options.begin(), options.end(), std::back_inserter(filtered),
               [min_moneyness, max_moneyness](const OptionChainEntry &option) {
                 return option.moneyness >= min_moneyness &&
                        option.moneyness <= max_moneyness;
               });

  return filtered;
}

// ============================================================================
// OptionChain Implementation
// ============================================================================

OptionChain::OptionChain(const std::string &symbol) : symbol_(symbol) {}

void OptionChain::add_option(const OptionChainEntry &entry) {
  auto it = expiries_.find(entry.contract.expiry);
  if (it == expiries_.end()) {
    auto [iter, inserted] = expiries_.emplace(
        entry.contract.expiry, ExpiryChain(symbol_, entry.contract.expiry));
    iter->second.add_option(entry);
  } else {
    it->second.add_option(entry);
  }
}

std::vector<std::string> OptionChain::get_expiries() const {
  std::vector<std::string> expiries;
  for (const auto &[expiry, _] : expiries_) {
    expiries.push_back(expiry);
  }
  std::sort(expiries.begin(), expiries.end());
  return expiries;
}

std::optional<ExpiryChain>
OptionChain::get_expiry_chain(const std::string &expiry) const {

  auto it = expiries_.find(expiry);
  if (it != expiries_.end()) {
    return it->second;
  }
  return std::nullopt;
}

std::vector<std::string>
OptionChain::get_expiries_in_dte_range(int min_dte, int max_dte) const {

  std::vector<std::string> filtered;
  for (const auto &[expiry, chain] : expiries_) {
    int dte = chain.get_days_to_expiry();
    if (dte >= min_dte && dte <= max_dte) {
      filtered.push_back(expiry);
    }
  }
  std::sort(filtered.begin(), filtered.end());
  return filtered;
}

std::vector<OptionChainEntry> OptionChain::get_all_options() const {
  std::vector<OptionChainEntry> all_options;
  for (const auto &[_, chain] : expiries_) {
    auto options = chain.get_all_options();
    all_options.insert(all_options.end(), options.begin(), options.end());
  }
  return all_options;
}

void OptionChain::set_underlying_price(double price) {
  underlying_price_ = price;
}

int OptionChain::get_total_option_count() const {
  return static_cast<int>(get_all_options().size());
}

int OptionChain::get_expiry_count() const {
  return static_cast<int>(expiries_.size());
}

// ============================================================================
// OptionChainBuilder Implementation (Stubs for Black-Scholes)
// ============================================================================

OptionChain OptionChainBuilder::build_from_market_data(
    const std::string &symbol,
    const std::vector<types::OptionContract> &contracts,
    const std::map<std::string, types::MarketData> &market_data) {

  OptionChain chain(symbol);

  spdlog::debug(
      "Building option chain for {} ({} contracts, {} with market data)",
      symbol, contracts.size(), market_data.size());

  // Build chain entries from contracts and market data
  for (const auto &contract : contracts) {
    std::string contract_key = contract.to_string();
    auto market_data_it = market_data.find(contract_key);

    if (market_data_it == market_data.end()) {
      spdlog::trace("No market data for contract: {}", contract_key);
      continue;
    }

    OptionChainEntry entry;
    entry.contract = contract;
    entry.market_data = market_data_it->second;

    // Set default liquidity scores (can be enhanced with actual volume/OI data)
    entry.volume = 0;
    entry.open_interest = 0;
    entry.liquidity_score = 50.0; // Default score

    chain.add_option(entry);
  }

  spdlog::debug("Built option chain with {} expiries, {} total options",
                chain.get_expiry_count(), chain.get_total_option_count());

  return chain;
}

void OptionChainBuilder::calculate_metrics(OptionChainEntry &entry,
                                           double underlying_price,
                                           double risk_free_rate) {

  entry.moneyness = entry.contract.strike / underlying_price;

  const double mid = entry.market_data.get_mid_price();
  if (entry.contract.type == types::OptionType::Call) {
    entry.intrinsic_value = std::max(underlying_price - entry.contract.strike, 0.0);
  } else {
    entry.intrinsic_value = std::max(entry.contract.strike - underlying_price, 0.0);
  }
  entry.extrinsic_value = std::max(mid - entry.intrinsic_value, 0.0);
}

std::optional<double> OptionChainBuilder::calculate_implied_volatility(
    double option_price, double underlying_price, double strike,
    double time_to_expiry, double risk_free_rate,
    types::OptionType option_type) {

  if (time_to_expiry <= 0.0 || option_price <= 0.0 || underlying_price <= 0.0 ||
      strike <= 0.0) {
    return std::nullopt;
  }

  try {
    // Convert time_to_expiry (years) to QuantLib time
    Time maturity = time_to_expiry;

    // Calculate implied volatility using QuantLib's black formula
    Option::Type ql_type =
        (option_type == types::OptionType::Call) ? Option::Call : Option::Put;

    // Use blackFormulaImpliedStdDev for implied volatility
    // This calculates the implied standard deviation (volatility * sqrt(time))
    double implied_std_dev =
        blackFormulaImpliedStdDev(ql_type, strike, underlying_price,
                                  option_price, risk_free_rate, maturity);

    // Convert to annualized volatility
    double implied_vol = implied_std_dev / std::sqrt(maturity);

    // Validate result
    if (implied_vol > 0.0 &&
        implied_vol < 10.0) { // Reasonable bounds (0-1000%)
      return implied_vol;
    }

    return std::nullopt;
  } catch (const std::exception &e) {
    spdlog::warn("QuantLib implied volatility calculation failed: {}",
                 e.what());
    return std::nullopt;
  }
}

double OptionChainBuilder::calculate_theoretical_price(
    double underlying_price, double strike, double time_to_expiry,
    double volatility, double risk_free_rate, types::OptionType option_type) {

  if (time_to_expiry <= 0.0 || underlying_price <= 0.0 || strike <= 0.0 ||
      volatility < 0.0) {
    return 0.0;
  }

  try {
    // Set up QuantLib date and calendar
    Date today = Date::todaysDate();
    Calendar calendar = TARGET();
    DayCounter dayCounter = Actual365Fixed();

    // Calculate expiration date from time_to_expiry (years)
    int days_to_expiry = static_cast<int>(time_to_expiry * 365.0);
    Date expiration = today + days_to_expiry;

    // Create option type
    Option::Type ql_type =
        (option_type == types::OptionType::Call) ? Option::Call : Option::Put;

    // Create handles for spot, rate, dividend (0), and volatility
    Handle<Quote> spotHandle(
        ext::shared_ptr<Quote>(new SimpleQuote(underlying_price)));
    Handle<YieldTermStructure> rateHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, risk_free_rate, dayCounter)));
    Handle<YieldTermStructure> divHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, 0.0, dayCounter)));
    Handle<BlackVolTermStructure> volHandle(
        ext::shared_ptr<BlackVolTermStructure>(
            new BlackConstantVol(today, calendar, volatility, dayCounter)));

    // Create Black-Scholes-Merton process (spot, dividend, risk-free, vol)
    ext::shared_ptr<BlackScholesMertonProcess> process(
        new BlackScholesMertonProcess(spotHandle, divHandle, rateHandle,
                                      volHandle));

    // Create pricing engine
    ext::shared_ptr<PricingEngine> engine(new AnalyticEuropeanEngine(process));

    // Create payoff and exercise
    ext::shared_ptr<StrikedTypePayoff> payoff(
        new PlainVanillaPayoff(ql_type, strike));
    ext::shared_ptr<Exercise> exercise(new EuropeanExercise(expiration));

    // Create option and set pricing engine
    VanillaOption option(payoff, exercise);
    option.setPricingEngine(engine);

    // Calculate theoretical price
    double theoretical_price = option.NPV();

    return theoretical_price;
  } catch (const std::exception &e) {
    spdlog::warn("QuantLib theoretical price calculation failed: {}", e.what());
    return 0.0;
  }
}

double OptionChainBuilder::calculate_delta(double underlying_price,
                                           double strike, double time_to_expiry,
                                           double volatility,
                                           double risk_free_rate,
                                           types::OptionType option_type) {

  if (time_to_expiry <= 0.0 || underlying_price <= 0.0 || strike <= 0.0 ||
      volatility < 0.0) {
    return 0.0;
  }

  try {
    // Set up QuantLib date and calendar
    Date today = Date::todaysDate();
    Calendar calendar = TARGET();
    DayCounter dayCounter = Actual365Fixed();

    // Calculate expiration date
    int days_to_expiry = static_cast<int>(time_to_expiry * 365.0);
    Date expiration = today + days_to_expiry;

    // Create option type
    Option::Type ql_type =
        (option_type == types::OptionType::Call) ? Option::Call : Option::Put;

    // Create handles
    Handle<Quote> spotHandle(
        ext::shared_ptr<Quote>(new SimpleQuote(underlying_price)));
    Handle<YieldTermStructure> rateHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, risk_free_rate, dayCounter)));
    Handle<YieldTermStructure> divHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, 0.0, dayCounter)));
    Handle<BlackVolTermStructure> volHandle(
        ext::shared_ptr<BlackVolTermStructure>(
            new BlackConstantVol(today, calendar, volatility, dayCounter)));

    // Create process and engine
    ext::shared_ptr<BlackScholesMertonProcess> process(
        new BlackScholesMertonProcess(spotHandle, divHandle, rateHandle,
                                      volHandle));
    ext::shared_ptr<PricingEngine> engine(new AnalyticEuropeanEngine(process));

    // Create option
    ext::shared_ptr<StrikedTypePayoff> payoff(
        new PlainVanillaPayoff(ql_type, strike));
    ext::shared_ptr<Exercise> exercise(new EuropeanExercise(expiration));
    VanillaOption option(payoff, exercise);
    option.setPricingEngine(engine);

    // Calculate delta
    return option.delta();
  } catch (const std::exception &e) {
    spdlog::warn("QuantLib delta calculation failed: {}", e.what());
    return 0.0;
  }
}

double OptionChainBuilder::calculate_gamma(double underlying_price,
                                           double strike, double time_to_expiry,
                                           double volatility,
                                           double risk_free_rate) {

  if (time_to_expiry <= 0.0 || underlying_price <= 0.0 || strike <= 0.0 ||
      volatility < 0.0) {
    return 0.0;
  }

  try {
    // Set up QuantLib (gamma is same for calls and puts)
    Date today = Date::todaysDate();
    Calendar calendar = TARGET();
    DayCounter dayCounter = Actual365Fixed();

    int days_to_expiry = static_cast<int>(time_to_expiry * 365.0);
    Date expiration = today + days_to_expiry;

    // Create handles
    Handle<Quote> spotHandle(
        ext::shared_ptr<Quote>(new SimpleQuote(underlying_price)));
    Handle<YieldTermStructure> rateHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, risk_free_rate, dayCounter)));
    Handle<YieldTermStructure> divHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, 0.0, dayCounter)));
    Handle<BlackVolTermStructure> volHandle(
        ext::shared_ptr<BlackVolTermStructure>(
            new BlackConstantVol(today, calendar, volatility, dayCounter)));

    // Create process and engine
    ext::shared_ptr<BlackScholesMertonProcess> process(
        new BlackScholesMertonProcess(spotHandle, divHandle, rateHandle,
                                      volHandle));
    ext::shared_ptr<PricingEngine> engine(new AnalyticEuropeanEngine(process));

    // Create option (use Call for gamma - same for both)
    ext::shared_ptr<StrikedTypePayoff> payoff(
        new PlainVanillaPayoff(Option::Call, strike));
    ext::shared_ptr<Exercise> exercise(new EuropeanExercise(expiration));
    VanillaOption option(payoff, exercise);
    option.setPricingEngine(engine);

    // Calculate gamma
    return option.gamma();
  } catch (const std::exception &e) {
    spdlog::warn("QuantLib gamma calculation failed: {}", e.what());
    return 0.0;
  }
}

double OptionChainBuilder::calculate_theta(double underlying_price,
                                           double strike, double time_to_expiry,
                                           double volatility,
                                           double risk_free_rate,
                                           types::OptionType option_type) {

  if (time_to_expiry <= 0.0 || underlying_price <= 0.0 || strike <= 0.0 ||
      volatility < 0.0) {
    return 0.0;
  }

  try {
    // Set up QuantLib
    Date today = Date::todaysDate();
    Calendar calendar = TARGET();
    DayCounter dayCounter = Actual365Fixed();

    int days_to_expiry = static_cast<int>(time_to_expiry * 365.0);
    Date expiration = today + days_to_expiry;

    Option::Type ql_type =
        (option_type == types::OptionType::Call) ? Option::Call : Option::Put;

    // Create handles
    Handle<Quote> spotHandle(
        ext::shared_ptr<Quote>(new SimpleQuote(underlying_price)));
    Handle<YieldTermStructure> rateHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, risk_free_rate, dayCounter)));
    Handle<YieldTermStructure> divHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, 0.0, dayCounter)));
    Handle<BlackVolTermStructure> volHandle(
        ext::shared_ptr<BlackVolTermStructure>(
            new BlackConstantVol(today, calendar, volatility, dayCounter)));

    // Create process and engine
    ext::shared_ptr<BlackScholesMertonProcess> process(
        new BlackScholesMertonProcess(spotHandle, divHandle, rateHandle,
                                      volHandle));
    ext::shared_ptr<PricingEngine> engine(new AnalyticEuropeanEngine(process));

    // Create option
    ext::shared_ptr<StrikedTypePayoff> payoff(
        new PlainVanillaPayoff(ql_type, strike));
    ext::shared_ptr<Exercise> exercise(new EuropeanExercise(expiration));
    VanillaOption option(payoff, exercise);
    option.setPricingEngine(engine);

    // Calculate theta (per day)
    return option.thetaPerDay();
  } catch (const std::exception &e) {
    spdlog::warn("QuantLib theta calculation failed: {}", e.what());
    return 0.0;
  }
}

double OptionChainBuilder::calculate_vega(double underlying_price,
                                          double strike, double time_to_expiry,
                                          double volatility,
                                          double risk_free_rate) {

  if (time_to_expiry <= 0.0 || underlying_price <= 0.0 || strike <= 0.0 ||
      volatility < 0.0) {
    return 0.0;
  }

  try {
    // Set up QuantLib (vega is same for calls and puts)
    Date today = Date::todaysDate();
    Calendar calendar = TARGET();
    DayCounter dayCounter = Actual365Fixed();

    int days_to_expiry = static_cast<int>(time_to_expiry * 365.0);
    Date expiration = today + days_to_expiry;

    // Create handles
    Handle<Quote> spotHandle(
        ext::shared_ptr<Quote>(new SimpleQuote(underlying_price)));
    Handle<YieldTermStructure> rateHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, risk_free_rate, dayCounter)));
    Handle<YieldTermStructure> divHandle(ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, 0.0, dayCounter)));
    Handle<BlackVolTermStructure> volHandle(
        ext::shared_ptr<BlackVolTermStructure>(
            new BlackConstantVol(today, calendar, volatility, dayCounter)));

    // Create process and engine
    ext::shared_ptr<BlackScholesMertonProcess> process(
        new BlackScholesMertonProcess(spotHandle, divHandle, rateHandle,
                                      volHandle));
    ext::shared_ptr<PricingEngine> engine(new AnalyticEuropeanEngine(process));

    // Create option (use Call for vega - same for both)
    ext::shared_ptr<StrikedTypePayoff> payoff(
        new PlainVanillaPayoff(Option::Call, strike));
    ext::shared_ptr<Exercise> exercise(new EuropeanExercise(expiration));
    VanillaOption option(payoff, exercise);
    option.setPricingEngine(engine);

    // Calculate vega
    return option.vega();
  } catch (const std::exception &e) {
    spdlog::warn("QuantLib vega calculation failed: {}", e.what());
    return 0.0;
  }
}

double OptionChainBuilder::standard_normal_cdf(double x) {
  return 0.5 * std::erfc(-x / std::sqrt(2.0));
}

double OptionChainBuilder::standard_normal_pdf(double x) {
  return std::exp(-0.5 * x * x) / std::sqrt(2.0 * M_PI);
}

std::pair<double, double>
OptionChainBuilder::calculate_d1_d2(double underlying_price, double strike,
                                    double time_to_expiry, double volatility,
                                    double risk_free_rate) {

  double d1 =
      (std::log(underlying_price / strike) +
       (risk_free_rate + 0.5 * volatility * volatility) * time_to_expiry) /
      (volatility * std::sqrt(time_to_expiry));

  double d2 = d1 - volatility * std::sqrt(time_to_expiry);

  return {d1, d2};
}

} // namespace option_chain
