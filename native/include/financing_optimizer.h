// financing_optimizer.h - NLopt multi-instrument financing cost minimization (Phase 4)
#pragma once

#include <array>
#include <string>

namespace financing {

// Number of instrument slots: box spread, T-bill, bank loan, pension loan, FX swap
constexpr size_t kNumInstruments = 5;

enum class InstrumentSlot : size_t {
  BoxSpread = 0,
  TBill = 1,
  BankLoan = 2,
  PensionLoan = 3,
  FxSwap = 4
};

struct FinancingOptimizerInput {
  // Effective annual rate per instrument (decimal, e.g. 0.05 = 5%)
  std::array<double, kNumInstruments> effective_rates{};

  // Optional min/max weight per instrument (0 = no min; 0 or 1 = no max)
  std::array<double, kNumInstruments> min_weight{};
  std::array<double, kNumInstruments> max_weight{};  // 0 or 1 = no max (1.0)
};

struct FinancingOptimizerResult {
  std::array<double, kNumInstruments> weights{};
  double effective_cost{0.0};  // Minimized blended rate (decimal)
  bool success{false};
  std::string error_message;
};

class FinancingOptimizer {
public:
  FinancingOptimizer() = default;
  ~FinancingOptimizer() = default;

  // Minimize effective cost across instruments; weights sum to 1.0
  [[nodiscard]] FinancingOptimizerResult optimize(
      const FinancingOptimizerInput& input) const;
};

}  // namespace financing
