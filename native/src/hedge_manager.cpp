// hedge_manager.cpp - Hedging management implementation
#include "hedge_manager.h"
#include "strategies/box_spread/box_spread_strategy.h"
#include <algorithm>
#include <cmath>
#include <spdlog/spdlog.h>

namespace hedge {

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

HedgeManager::HedgeManager() { spdlog::debug("HedgeManager created"); }

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

  // TODO: Fetch current futures price and front-month expiry via TWS API
  // reqMktData; 95.0 is a non-functional placeholder implying ~5% rate.
  future.current_price = 95.0;
  future.expiry = "";

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

  // TODO: Fetch FX spot rate from TWS API reqMktData using the FX pair
  // contract; all rates below are hardcoded stubs.
  if (base_currency == "USD" && hedge_currency == "ILS") {
    return 3.65; // stub
  }

  // Default to 1.0 for same currency or unknown pairs
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
                            const types::BoxSpreadLeg &box_spread) const {

  HedgeEffectiveness effectiveness;
  effectiveness.target_hedge_ratio = hedge.hedge_ratio;
  // TODO: Derive current_hedge_ratio from live position quantities and
  // hedge_drift_bps from realized rate moves since hedge inception.
  effectiveness.current_hedge_ratio = hedge.hedge_ratio;
  effectiveness.hedge_drift_bps = 0.0;
  effectiveness.needs_rebalance = false;
  effectiveness.rebalance_cost = 0.0;

  // Calculate drift
  effectiveness.hedge_drift_bps = std::abs(effectiveness.current_hedge_ratio -
                                           effectiveness.target_hedge_ratio) *
                                  10000.0;

  // Check if rebalancing needed (if enabled in strategy)
  // For now, simple threshold check
  effectiveness.needs_rebalance =
      effectiveness.hedge_drift_bps > 10.0; // 10 bps threshold

  if (effectiveness.needs_rebalance) {
    // Estimate rebalancing cost
    // TODO: Calculate rebalance_cost from live bid-ask spread and round-trip
    // commission for the required number of contracts; $50 is a placeholder.
    effectiveness.rebalance_cost = 50.0;
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
