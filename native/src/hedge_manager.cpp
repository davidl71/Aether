// hedge_manager.cpp - Hedging management implementation
#include "hedge_manager.h"
#include "tws_client.h"
#include "strategies/box_spread/box_spread_strategy.h"
#include <algorithm>
#include <chrono>
#include <cmath>
#include <ctime>
#include <spdlog/spdlog.h>

namespace hedge {

namespace {

// Returns the YYYYMMDD string for the nearest IMM quarterly date (15th of
// Mar/Jun/Sep/Dec) that is at least `dte_days` calendar days from today.
std::string next_imm_expiry(int dte_days) {
  auto now = std::chrono::system_clock::now();
  std::time_t t = std::chrono::system_clock::to_time_t(now);
  std::tm tm_now = *std::gmtime(&t);

  // Target date: today + dte_days
  t += static_cast<std::time_t>(dte_days) * 86400;
  std::tm tm_target = *std::gmtime(&t);
  int year  = tm_target.tm_year + 1900;
  int month = tm_target.tm_mon + 1; // 1-based

  // Snap to the next IMM quarter month (3, 6, 9, 12)
  static const int imm_months[] = {3, 6, 9, 12};
  for (int m : imm_months) {
    if (m >= month) {
      char buf[9];
      std::snprintf(buf, sizeof(buf), "%04d%02d15", year, m);
      return buf;
    }
  }
  // Rolled past December — use March of next year
  char buf[9];
  std::snprintf(buf, sizeof(buf), "%04d%02d15", year + 1, 3);
  return buf;
}

} // anonymous namespace

// ============================================================================
// InterestRateFuture Implementation
// ============================================================================

double InterestRateFuture::calculate_implied_rate() const {
  // For SOFR/Eurodollar futures, price = 100 - (implied rate * 100)
  // For example, price 95.5 = 4.5% implied rate
  // Implied rate = (100 - price) / 100.0
  return (100.0 - current_price) / 100.0;
}

double
InterestRateFuture::calculate_hedge_ratio(const types::BoxSpreadLeg &box_spread,
                                          double box_spread_notional) const {

  // Calculate box spread implied rate
  double box_spread_rate =
      strategy::BoxSpreadCalculator::calculate_implied_interest_rate(
          box_spread);

  // Calculate futures implied rate
  double futures_rate = calculate_implied_rate();

  // Hedge ratio based on notional and rates
  // If box spread rate = 5% and futures rate = 4.5%, we need to adjust
  double rate_difference = box_spread_rate - futures_rate;

  // For full hedge, ratio = 1.0
  // Adjust based on rate difference
  double hedge_ratio = 1.0 - (rate_difference / box_spread_rate);

  return std::max(0.0, std::min(2.0, hedge_ratio)); // Clamp between 0 and 2
}

// ============================================================================
// CurrencyHedge Implementation
// ============================================================================

double CurrencyHedge::calculate_hedge_amount(double exposure_usd) const {
  // Convert exposure in base currency to hedge currency
  // If base is USD and hedge is ILS, multiply by exchange rate
  double hedge_amount = exposure_usd * current_rate;

  // Apply hedge ratio if different from 1.0
  double ratio = hedge_amount / exposure_amount;
  if (exposure_amount > 0) {
    hedge_amount = exposure_usd * (hedge_amount / exposure_amount);
  }

  return hedge_amount;
}

double CurrencyHedge::calculate_hedge_cost() const {
  // Cost of currency hedge (futures/forwards spread)
  // Typically small spread (few basis points)
  // For now, estimate as 0.1% of hedge amount
  double cost_pct = 0.001; // 10 basis points
  return hedge_amount * cost_pct;
}

// ============================================================================
// HedgeManager Implementation
// ============================================================================

HedgeManager::HedgeManager(tws::TWSClient *client)
    : client_(client) {
  spdlog::debug("HedgeManager created (client={})", client ? "set" : "null");
}

HedgeManager::~HedgeManager() { spdlog::debug("HedgeManager destroyed"); }

std::optional<InterestRateFuture>
HedgeManager::find_rate_future_hedge(const types::BoxSpreadLeg &box_spread,
                                     InterestRateFutureType preferred_type) {

  int box_spread_dte = box_spread.get_days_to_expiry();

  // Find suitable futures contract matching DTE
  InterestRateFuture future;
  future.type = preferred_type;
  future.days_to_expiry = box_spread_dte;

  // Set contract specifications based on type
  switch (preferred_type) {
  case InterestRateFutureType::SOFR_3M:
    future.symbol = "SR3";
    future.contract_size = 1000000.0; // $1M
    future.tick_size = 0.0025;        // 0.25 basis points
    future.tick_value = 6.25;         // $6.25 per tick
    break;

  case InterestRateFutureType::SOFR_1M:
    future.symbol = "SR1";
    future.contract_size = 1000000.0;
    future.tick_size = 0.0025;
    future.tick_value = 2.08; // ~$2.08 per tick (1/3 of 3M)
    break;

  case InterestRateFutureType::Eurodollar:
    future.symbol = "ED";
    future.contract_size = 1000000.0;
    future.tick_size = 0.0025;
    future.tick_value = 6.25;
    break;

  case InterestRateFutureType::Fed_Funds:
    future.symbol = "ZQ";
    future.contract_size = 5000000.0; // $5M
    future.tick_size = 0.0025;
    future.tick_value = 31.25; // $31.25 per tick
    break;

  default:
    future.symbol = "SR3"; // Default to SOFR 3M
    future.contract_size = 1000000.0;
    future.tick_size = 0.0025;
    future.tick_value = 6.25;
    break;
  }

  // Fetch current futures price and expiry via TWS API when available.
  // Falls back to 95.0 (≈5% implied rate) when client is null or call fails.
  future.current_price = 95.0;
  future.expiry = next_imm_expiry(box_spread_dte);
  if (client_) {
    types::OptionContract proxy;
    proxy.symbol   = future.symbol;
    proxy.strike   = 100.0; // dummy non-zero so is_valid() passes
    proxy.expiry   = future.expiry;
    proxy.exchange = "GLOBEX";
    proxy.type     = types::OptionType::Call;
    auto md = client_->request_market_data_sync(proxy, 5000);
    if (md) {
      double price = (md->last > 0.0) ? md->last : md->get_mid_price();
      if (price > 0.0) {
        future.current_price = price;
        spdlog::debug("Futures {} price from market data: {:.4f}",
                      future.symbol, future.current_price);
      }
    }
  }

  spdlog::debug("Found rate future hedge: {} (DTE: {}, Price: {:.2f})",
                future.symbol, future.days_to_expiry, future.current_price);

  return future;
}

HedgeManager::RateHedgeCalculation HedgeManager::calculate_rate_hedge(
    const types::BoxSpreadLeg &box_spread, double box_spread_notional,
    const InterestRateFuture &future, double target_hedge_ratio) {

  RateHedgeCalculation calc;
  calc.future = future;
  calc.hedge_ratio = target_hedge_ratio;
  calc.is_valid = false;

  // Calculate box spread implied rate
  double box_spread_rate =
      strategy::BoxSpreadCalculator::calculate_implied_interest_rate(
          box_spread);

  // Calculate futures implied rate
  double futures_rate = future.calculate_implied_rate();

  // Calculate basis risk (rate difference)
  calc.basis_risk_bps =
      (box_spread_rate - futures_rate) * 10000.0; // Convert to basis points

  // Calculate number of contracts needed
  // Contract size is $1M for SOFR/ED, adjust for hedge ratio
  double contracts_raw =
      (box_spread_notional * target_hedge_ratio) / future.contract_size;
  calc.contracts_needed = static_cast<int>(std::ceil(contracts_raw));

  // Calculate hedge cost (commission + bid-ask spread)
  // Estimate bid-ask spread as 1 tick
  double spread_cost = future.tick_value; // Cost of spread
  double commission =
      2.0 * 2.00; // Round-trip commission (futures ~$2/contract)
  calc.hedge_cost = (spread_cost + commission) * calc.contracts_needed;

  // Hedge is viable if basis risk and cost are acceptable
  calc.is_valid =
      (std::abs(calc.basis_risk_bps) < 50.0) && // Basis risk < 50 bps
      (calc.hedge_cost <
       box_spread_notional * 0.001); // Cost < 0.1% of notional

  spdlog::debug("Rate hedge calculation: {} contracts, basis risk: {:.1f} bps, "
                "cost: ${:.2f}",
                calc.contracts_needed, calc.basis_risk_bps, calc.hedge_cost);

  return calc;
}

CurrencyHedge HedgeManager::calculate_currency_hedge(
    const types::BoxSpreadLeg &box_spread, const std::string &base_currency,
    const std::string &hedge_currency, double exposure_amount) {

  CurrencyHedge hedge;
  hedge.base_currency = base_currency;
  hedge.hedge_currency = hedge_currency;
  hedge.pair_symbol = base_currency + hedge_currency;
  hedge.exposure_amount = exposure_amount;

  // Get exchange rate (would fetch from TWS API in full implementation)
  hedge.current_rate = get_exchange_rate(base_currency, hedge_currency);
  hedge.hedge_rate = hedge.current_rate;

  // Calculate hedge amount
  hedge.hedge_amount = hedge.calculate_hedge_amount(exposure_amount);

  spdlog::debug("Currency hedge: {} {} -> {} {} (rate: {:.4f})",
                exposure_amount, base_currency, hedge.hedge_amount,
                hedge_currency, hedge.current_rate);

  return hedge;
}

double
HedgeManager::get_exchange_rate(const std::string &base_currency,
                                const std::string &hedge_currency) const {

  // Fetch FX spot rate via TWS market data when client is available.
  if (client_) {
    types::OptionContract proxy;
    proxy.symbol       = base_currency + hedge_currency;
    proxy.strike       = 1.0; // dummy non-zero so is_valid() passes
    proxy.exchange     = "IDEALPRO";
    proxy.type         = types::OptionType::Call;
    proxy.local_symbol = base_currency + "." + hedge_currency;
    auto md = client_->request_market_data_sync(proxy, 5000);
    if (md) {
      double rate = md->get_mid_price();
      if (rate > 0.0) {
        spdlog::debug("FX {}/{} rate from market data: {:.4f}",
                      base_currency, hedge_currency, rate);
        return rate;
      }
    }
  }

  // Hardcoded fallback stubs when client unavailable or data absent.
  if (base_currency == "USD" && hedge_currency == "ILS") {
    return 3.65;
  }

  if (base_currency == hedge_currency) {
    return 1.0;
  }

  spdlog::warn("Exchange rate not available for {}/{} (using 1.0)",
               base_currency, hedge_currency);
  return 1.0;
}

double
HedgeManager::calculate_currency_hedge_cost(const CurrencyHedge &hedge) const {
  return hedge.calculate_hedge_cost();
}

HedgeManager::CompleteHedge
HedgeManager::calculate_complete_hedge(const types::BoxSpreadLeg &box_spread,
                                       double box_spread_notional,
                                       const HedgeStrategy &strategy) {

  CompleteHedge complete;
  complete.is_viable = false;
  complete.total_hedge_cost = 0.0;
  complete.total_hedge_cost_bps = 0.0;

  // Calculate interest rate hedge if enabled
  if (strategy.hedge_interest_rate) {
    auto future_opt =
        find_rate_future_hedge(box_spread, strategy.rate_future_type);
    if (future_opt.has_value()) {
      complete.rate_hedge =
          calculate_rate_hedge(box_spread, box_spread_notional,
                               future_opt.value(), strategy.target_hedge_ratio);
      complete.total_hedge_cost += complete.rate_hedge.hedge_cost;
    }
  }

  // Calculate currency hedge if enabled
  if (strategy.hedge_currency && !strategy.hedge_currency_code.empty()) {
    complete.currency_hedge = calculate_currency_hedge(
        box_spread, "USD", strategy.hedge_currency_code, box_spread_notional);
    double currency_cost =
        calculate_currency_hedge_cost(complete.currency_hedge);
    complete.total_hedge_cost += currency_cost;
  }

  // Calculate total cost as basis points
  if (box_spread_notional > 0) {
    complete.total_hedge_cost_bps =
        (complete.total_hedge_cost / box_spread_notional) * 10000.0;
  }

  // Check if hedge is viable (cost within threshold)
  complete.is_viable =
      complete.total_hedge_cost_bps <= strategy.max_hedge_cost_bps;

  spdlog::info(
      "Complete hedge calculated: cost: ${:.2f} ({:.1f} bps), viable: {}",
      complete.total_hedge_cost, complete.total_hedge_cost_bps,
      complete.is_viable);

  return complete;
}

HedgeManager::HedgeEffectiveness
HedgeManager::monitor_hedge(const RateHedgeCalculation &hedge,
                            const types::BoxSpreadLeg &box_spread,
                            int actual_contracts) const {

  HedgeEffectiveness effectiveness;
  effectiveness.target_hedge_ratio = hedge.hedge_ratio;
  effectiveness.needs_rebalance = false;
  effectiveness.rebalance_cost = 0.0;

  // Derive current hedge ratio from actual filled contracts when known.
  // When actual_contracts < 0 (unknown), assume target was fully achieved.
  if (actual_contracts >= 0 && hedge.contracts_needed > 0) {
    effectiveness.current_hedge_ratio =
        static_cast<double>(actual_contracts) /
        static_cast<double>(hedge.contracts_needed) * hedge.hedge_ratio;
  } else {
    effectiveness.current_hedge_ratio = hedge.hedge_ratio;
  }

  // Drift = deviation from target in basis points.
  effectiveness.hedge_drift_bps =
      std::abs(effectiveness.current_hedge_ratio -
               effectiveness.target_hedge_ratio) *
      10000.0;

  effectiveness.needs_rebalance =
      effectiveness.hedge_drift_bps > 10.0; // 10 bps threshold

  if (effectiveness.needs_rebalance) {
    // Rebalance cost = (1-tick spread + $2/side commission) × contracts to trade.
    // tick_value is set per contract type in find_rate_future_hedge.
    int contracts_to_trade = (actual_contracts >= 0)
                                 ? std::abs(actual_contracts -
                                            hedge.contracts_needed)
                                 : hedge.contracts_needed;
    effectiveness.rebalance_cost =
        (hedge.future.tick_value + 4.0) * std::max(1, contracts_to_trade);
  }

  return effectiveness;
}

double HedgeManager::calculate_basis_risk(double box_spread_rate,
                                          double futures_implied_rate) const {

  return (box_spread_rate - futures_implied_rate) * 10000.0; // Basis points
}

double HedgeManager::convert_dte_to_futures_contract(
    int box_spread_dte, InterestRateFutureType future_type) const {

  // Convert box spread DTE to appropriate futures contract
  // This is a simplified mapping - in reality would match to nearest quarterly
  // expiry
  return static_cast<double>(box_spread_dte);
}

} // namespace hedge
