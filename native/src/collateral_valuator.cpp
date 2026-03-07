// collateral_valuator.cpp - Collateral valuation implementation (Phase 2)
#include "collateral_valuator.h"

namespace synthetic {

CollateralValuation CollateralValuator::value_for_margin(
    const CollateralPosition& position,
    const std::string& margin_requirement_type,
    const std::string& broker) const {
  (void)margin_requirement_type;
  (void)broker;
  double haircut = get_haircut(
      position.asset_id.empty() ? "CASH" : "STOCK",
      position.days_to_maturity,
      position.credit_rating,
      broker);
  CollateralValuation v;
  v.gross_value = position.market_value;
  v.haircut_percent = haircut;
  v.net_collateral_value = position.market_value * (1.0 - haircut / 100.0);
  v.margin_credit = v.net_collateral_value;
  v.currency = position.currency;
  v.valuation_method = "MARKET";
  return v;
}

CollateralValuation CollateralValuator::value_portfolio(
    const std::vector<CollateralPosition>& positions,
    const std::string& margin_requirement_type) const {
  CollateralValuation acc;
  acc.currency = "USD";
  acc.valuation_method = "MARKET";
  for (const auto& p : positions) {
    auto v = value_for_margin(p, margin_requirement_type, "");
    acc.gross_value += v.gross_value;
    acc.haircut_percent = (acc.haircut_percent + v.haircut_percent) / 2.0;  // simplistic
    acc.net_collateral_value += v.net_collateral_value;
    acc.margin_credit += v.margin_credit;
  }
  return acc;
}

CollateralValuation CollateralValuator::value_cross_currency(
    const CollateralPosition& position,
    const Currency& target_currency,
    double fx_rate,
    double fx_haircut) const {
  CollateralValuation v;
  v.currency = target_currency;
  v.gross_value = position.market_value * fx_rate;
  v.haircut_percent = fx_haircut * 100.0;
  v.net_collateral_value = v.gross_value * (1.0 - fx_haircut);
  v.margin_credit = v.net_collateral_value;
  v.valuation_method = "MARKET";
  return v;
}

double CollateralValuator::get_haircut(
    const std::string& asset_type,
    int days_to_maturity,
    double credit_rating,
    const std::string& broker) const {
  (void)broker;
  (void)credit_rating;
  if (asset_type == "T-BILL" || asset_type == "TREASURY_BILL") {
    if (days_to_maturity <= 90) return 0.0;
    if (days_to_maturity <= 365) return 0.5;
    return 1.0;
  }
  if (asset_type == "BOND" || asset_type == "TREASURY_BOND" || asset_type == "CORPORATE_BOND")
    return 2.0;
  if (asset_type == "STOCK" || asset_type == "ETF") return 5.0;
  if (asset_type == "CASH") return 0.0;
  return 10.0;  // default conservative
}

}  // namespace synthetic
