// wasm_types.h - WASM-compatible type definitions
// These types are designed for efficient serialization between JavaScript and WASM

#pragma once

#include <cstdint>
#include <cstddef>

namespace wasm {

// Box spread input (from JavaScript)
// All fields are POD types for easy WASM binding
struct BoxSpreadInput {
  // Long call leg
  double long_call_strike;
  double long_call_bid;
  double long_call_ask;

  // Short call leg
  double short_call_strike;
  double short_call_bid;
  double short_call_ask;

  // Long put leg
  double long_put_strike;
  double long_put_bid;
  double long_put_ask;

  // Short put leg
  double short_put_strike;
  double short_put_bid;
  double short_put_ask;

  // Market context
  double underlying_price;
  double risk_free_rate;
  double days_to_expiry;
  double volatility;  // Optional, for Greeks calculations
};

// Box spread calculation result (to JavaScript)
struct BoxSpreadResult {
  double net_debit;
  double arbitrage_profit;
  double roi;
  double apr;  // Annualized percentage return
  double confidence_score;  // 0-100
  bool is_profitable;
  double risk_score;  // 0-100, higher = riskier

  // Greeks (if volatility provided)
  double delta;
  double gamma;
  double theta;
  double vega;
};

// Risk calculation input
struct RiskInput {
  double position_size;
  double volatility;
  double time_horizon_days;
  double confidence_level;  // e.g., 0.95 for 95% VaR
};

// Risk calculation result
struct RiskResult {
  double var;  // Value at Risk
  double expected_loss;
  double max_loss;
  double position_size_limit;  // Recommended position size
};

} // namespace wasm
