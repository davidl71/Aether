// financing_instrument_registry.cpp - Financing instrument registry (Phase 2)
#include "financing_instrument.h"

#include <algorithm>
#include <cmath>

namespace synthetic {

void FinancingInstrumentRegistry::register_instrument(const FinancingInstrument& instrument) {
  instruments_.push_back(instrument);
}

std::vector<FinancingInstrument> FinancingInstrumentRegistry::find_financing_options(
    const Currency& currency,
    double amount,
    int days_needed,
    double min_liquidity) const {
  std::vector<FinancingInstrument> out;
  for (const auto& inst : instruments_) {
    if (!inst.is_available) continue;
    if (inst.base_currency != currency) continue;
    if (inst.liquidity_score < min_liquidity) continue;
    if (inst.min_size > 0.0 && amount < inst.min_size) continue;
    if (inst.max_size > 0.0 && amount > inst.max_size) continue;
    if (inst.days_to_maturity > 0 && days_needed > inst.days_to_maturity) continue;
    out.push_back(inst);
  }
  return out;
}

FinancingInstrument FinancingInstrumentRegistry::get_best_financing(
    const Currency& currency,
    double amount,
    int days_needed,
    const std::string& optimization_criteria) const {
  auto options = find_financing_options(currency, amount, days_needed);
  if (options.empty()) return {};

  if (optimization_criteria == "LIQUIDITY") {
    return *std::max_element(
        options.begin(), options.end(),
        [](const FinancingInstrument& a, const FinancingInstrument& b) {
          return a.liquidity_score < b.liquidity_score;
        });
  }
  if (optimization_criteria == "FLEXIBILITY") {
    return *std::max_element(
        options.begin(), options.end(),
        [](const FinancingInstrument& a, const FinancingInstrument& b) {
          return (a.max_size - a.min_size) < (b.max_size - b.min_size);
        });
  }
  // COST (default): lowest effective rate
  return *std::min_element(
      options.begin(), options.end(),
      [](const FinancingInstrument& a, const FinancingInstrument& b) {
        return (std::isnan(a.effective_rate) ? a.annual_rate : a.effective_rate) <
               (std::isnan(b.effective_rate) ? b.annual_rate : b.effective_rate);
      });
}

std::vector<FinancingInstrument> FinancingInstrumentRegistry::get_collateral_instruments(
    const std::string& target_asset_id,
    const Currency& currency) const {
  std::vector<FinancingInstrument> out;
  for (const auto& inst : instruments_) {
    if (!inst.is_available) continue;
    if (inst.base_currency != currency) continue;
    for (const auto& can : inst.can_collateralize) {
      if (can == target_asset_id) {
        out.push_back(inst);
        break;
      }
    }
  }
  return out;
}

}  // namespace synthetic
