// asset_relationship.h - Asset relationship graph for synthetic financing (Phase 2)
// See docs/platform/SYNTHETIC_FINANCING_ARCHITECTURE.md
#pragma once

#include <chrono>
#include <string>
#include <vector>

namespace synthetic {

// Currency as ISO code (e.g. "USD", "ILS"); aligns with hedge_manager and multi-currency use.
using Currency = std::string;

enum class RelationshipType {
  COLLATERAL,      // Asset can be used as collateral
  MARGIN,          // Asset can satisfy margin requirements
  FINANCING,       // Asset provides financing
  HEDGE,           // Asset hedges another position
  ARBITRAGE,       // Arbitrage relationship
  CROSS_CURRENCY,  // Cross-currency relationship
  REGULATORY       // Regulatory relationship (haircuts, offsets)
};

struct AssetRelationship {
  std::string source_asset_id;  // e.g. "T-BILL-3M-USD"
  std::string target_asset_id;  // e.g. "SPX-OPTIONS"
  RelationshipType type{RelationshipType::COLLATERAL};

  // Relationship parameters
  double collateral_value_ratio{1.0};  // 0.0-1.0 (haircut applied)
  double margin_credit_ratio{1.0};     // Margin reduction percentage
  Currency base_currency;
  Currency target_currency;

  // Constraints
  double min_amount{0.0};
  double max_amount{0.0};  // 0 = no limit
  int min_days_to_maturity{0};
  int max_days_to_maturity{0};  // 0 = no limit

  // Broker-specific
  std::vector<std::string> applicable_brokers;  // IBKR, Alpaca, etc.
  std::string regulatory_regime;                  // "REG-T", "PORTFOLIO", "SPAN"

  // Validity
  std::chrono::system_clock::time_point valid_from;
  std::chrono::system_clock::time_point valid_until;
  bool is_active{true};
};

class AssetRelationshipGraph {
 public:
  void add_relationship(const AssetRelationship& rel);

  [[nodiscard]] std::vector<AssetRelationship> get_collateral_for(
      const std::string& target_asset_id,
      const Currency& currency) const;

  [[nodiscard]] std::vector<AssetRelationship> get_financing_options(
      const Currency& currency,
      double amount,
      int days_needed) const;

  [[nodiscard]] std::vector<AssetRelationship> find_collateral_chain(
      const std::string& target_asset_id,
      double required_margin,
      const Currency& currency) const;

  [[nodiscard]] std::vector<AssetRelationship> get_cross_currency_paths(
      const Currency& from,
      const Currency& to) const;

 private:
  std::vector<AssetRelationship> relationships_;
  static bool is_relationship_valid_at(const AssetRelationship& rel,
                                       std::chrono::system_clock::time_point at);
};

}  // namespace synthetic
