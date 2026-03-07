// collateral_valuator.h - Collateral valuation for synthetic financing (Phase 2)
// See docs/platform/SYNTHETIC_FINANCING_ARCHITECTURE.md
#pragma once

#include "asset_relationship.h"

#include <string>
#include <vector>

namespace synthetic {

struct CollateralPosition {
  std::string asset_id;
  double quantity{0.0};
  double market_value{0.0};
  Currency currency;
  int days_to_maturity{0};   // For bonds/T-bills
  double credit_rating{0.0};  // For bonds
};

struct CollateralValuation {
  double gross_value{0.0};            // Market value
  double haircut_percent{0.0};        // Applied haircut
  double net_collateral_value{0.0};   // After haircut
  double margin_credit{0.0};          // Credit toward margin
  Currency currency;
  std::string valuation_method;  // "MARKET", "STRESS", "REGULATORY"
};

class CollateralValuator {
 public:
  [[nodiscard]] CollateralValuation value_for_margin(
      const CollateralPosition& position,
      const std::string& margin_requirement_type,  // "OPTIONS", "FUTURES", etc.
      const std::string& broker) const;

  [[nodiscard]] CollateralValuation value_portfolio(
      const std::vector<CollateralPosition>& positions,
      const std::string& margin_requirement_type) const;

  [[nodiscard]] CollateralValuation value_cross_currency(
      const CollateralPosition& position,
      const Currency& target_currency,
      double fx_rate,
      double fx_haircut = 0.02) const;

  [[nodiscard]] double get_haircut(
      const std::string& asset_type,  // "T-BILL", "BOND", "STOCK", "ETF"
      int days_to_maturity,
      double credit_rating,
      const std::string& broker) const;
};

}  // namespace synthetic
