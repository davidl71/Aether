// financing_instrument.h - Financing instrument registry (Phase 2)
// See docs/platform/SYNTHETIC_FINANCING_ARCHITECTURE.md
#pragma once

#include "asset_relationship.h"

#include <chrono>
#include <string>
#include <vector>

namespace synthetic {

enum class InstrumentType {
  BOX_SPREAD,        // Options-based synthetic financing
  FUTURES,           // Futures-based financing
  TREASURY_BILL,     // T-bill
  TREASURY_BOND,     // T-bond
  CORPORATE_BOND,    // Corporate bond
  BANK_LOAN,         // Bank financing
  PENSION_LOAN,      // Pension fund loan
  REPO,              // Repurchase agreement
  FX_SWAP,           // FX swap
  MONEY_MARKET_FUND  // Money market fund
};

struct FinancingInstrument {
  InstrumentType type{InstrumentType::BOX_SPREAD};
  std::string asset_id;
  std::string symbol;  // Market symbol
  Currency base_currency;

  // Financing terms
  double annual_rate{0.0};    // Annualized financing cost/return (%)
  double effective_rate{0.0};  // After fees, taxes, etc.
  double all_in_cost{0.0};     // Total cost including all fees

  // Terms
  int days_to_maturity{0};
  std::chrono::system_clock::time_point maturity_date;
  double min_size{0.0};
  double max_size{0.0};

  // Collateral requirements
  double required_collateral_ratio{0.0};  // 0.0-1.0
  std::vector<std::string> acceptable_collateral_types;

  // Liquidity and availability
  double liquidity_score{0.0};  // 0-100
  bool is_available{true};
  std::vector<std::string> available_brokers;

  // Relationship metadata
  std::vector<std::string> can_collateralize;  // What this can be collateral for
  std::vector<std::string> can_finance;        // What this can finance
};

class FinancingInstrumentRegistry {
 public:
  void register_instrument(const FinancingInstrument& instrument);

  [[nodiscard]] std::vector<FinancingInstrument> find_financing_options(
      const Currency& currency,
      double amount,
      int days_needed,
      double min_liquidity = 50.0) const;

  [[nodiscard]] FinancingInstrument get_best_financing(
      const Currency& currency,
      double amount,
      int days_needed,
      const std::string& optimization_criteria = "COST") const;  // "COST", "LIQUIDITY", "FLEXIBILITY"

  [[nodiscard]] std::vector<FinancingInstrument> get_collateral_instruments(
      const std::string& target_asset_id,
      const Currency& currency) const;

 private:
  std::vector<FinancingInstrument> instruments_;
};

}  // namespace synthetic
