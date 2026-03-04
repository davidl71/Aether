// Minimal types implementation for proto_adapter_tests (no TWS dependency).
#include "types.h"
#include <string>

namespace types {

std::string OptionContract::to_string() const {
  return symbol + " " + expiry + " " + std::to_string(strike);
}

bool OptionContract::is_valid() const {
  return !symbol.empty() && !expiry.empty() && strike > 0 && !exchange.empty();
}

bool BoxSpreadLeg::is_valid() const {
  return long_call.is_valid() && short_call.is_valid() &&
         long_put.is_valid() && short_put.is_valid();
}

double BoxSpreadLeg::get_strike_width() const {
  return short_call.strike - long_call.strike;
}

int BoxSpreadLeg::get_days_to_expiry() const { return 0; }

double BoxSpreadLeg::get_effective_margin() const {
  if (uses_portfolio_margin && span_margin > 0.0) return span_margin;
  if (initial_margin > 0.0) return initial_margin;
  return net_debit * 100.0;
}

}  // namespace types
