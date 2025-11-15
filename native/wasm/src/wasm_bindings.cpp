// wasm_bindings.cpp - Emscripten bindings for JavaScript interop
// This file provides the bridge between C++ calculation logic and JavaScript

#include <emscripten/bind.h>
#include "wasm_types.h"
#include "../include/box_spread_strategy.h"
#include "../include/risk_calculator.h"
#include "../include/types.h"
#include <memory>
#include <cmath>

using namespace emscripten;

// Forward declarations
namespace {
  // Convert WASM input to native types
  types::BoxSpreadLeg convert_to_native(const wasm::BoxSpreadInput& input) {
    types::BoxSpreadLeg spread;

    // Long call
    spread.long_call.strike = input.long_call_strike;
    spread.long_call_price = (input.long_call_bid + input.long_call_ask) / 2.0;

    // Short call
    spread.short_call.strike = input.short_call_strike;
    spread.short_call_price = (input.short_call_bid + input.short_call_ask) / 2.0;

    // Long put
    spread.long_put.strike = input.long_put_strike;
    spread.long_put_price = (input.long_put_bid + input.long_put_ask) / 2.0;

    // Short put
    spread.short_put.strike = input.short_put_strike;
    spread.short_put_price = (input.short_put_bid + input.short_put_ask) / 2.0;

    // Calculate net debit
    spread.net_debit = spread.long_call_price
                     - spread.short_call_price
                     + spread.long_put_price
                     - spread.short_put_price;

    // Calculate theoretical value (strike width)
    double strike_width = std::abs(input.short_call_strike - input.long_call_strike);
    spread.theoretical_value = strike_width;
    spread.arbitrage_profit = spread.theoretical_value - spread.net_debit;

    // Calculate ROI
    if (spread.net_debit > 0) {
      spread.roi_percent = (spread.arbitrage_profit / spread.net_debit) * 100.0;
    }

    return spread;
  }

  // Convert native result to WASM result
  wasm::BoxSpreadResult convert_to_wasm(
    const types::BoxSpreadLeg& spread,
    double apr,
    double confidence_score,
    bool is_profitable
  ) {
    wasm::BoxSpreadResult result;
    result.net_debit = spread.net_debit;
    result.arbitrage_profit = spread.arbitrage_profit;
    result.roi = spread.roi_percent;
    result.apr = apr;
    result.confidence_score = confidence_score;
    result.is_profitable = is_profitable;
    result.risk_score = 0.0;  // TODO: Calculate from risk calculator
    result.delta = 0.0;  // TODO: Calculate Greeks
    result.gamma = 0.0;
    result.theta = 0.0;
    result.vega = 0.0;
    return result;
  }
}

// Main box spread calculation function
wasm::BoxSpreadResult calculate_box_spread(const wasm::BoxSpreadInput& input) {
  // Convert WASM input to native types
  types::BoxSpreadLeg spread = convert_to_native(input);

  // TODO: Create strategy instance (needs refactoring to avoid TWS dependency)
  // For now, use simple calculations
  double apr = 0.0;
  if (input.days_to_expiry > 0 && spread.net_debit > 0) {
    double annualized_profit = spread.arbitrage_profit * (365.0 / input.days_to_expiry);
    apr = (annualized_profit / spread.net_debit) * 100.0;
  }

  // Simple confidence score based on spread tightness
  double confidence_score = 50.0;  // Placeholder
  if (spread.arbitrage_profit > 0) {
    confidence_score = std::min(100.0, 50.0 + (spread.arbitrage_profit * 10.0));
  }

  bool is_profitable = spread.arbitrage_profit > 0 && spread.roi_percent > 0;

  return convert_to_wasm(spread, apr, confidence_score, is_profitable);
}

// Risk calculation function
wasm::RiskResult calculate_risk(const wasm::RiskInput& input) {
  wasm::RiskResult result;

  // TODO: Use risk calculator (needs refactoring)
  // Placeholder calculation
  result.var = input.position_size * input.volatility * std::sqrt(input.time_horizon_days / 365.0);
  result.expected_loss = result.var * 0.5;
  result.max_loss = input.position_size;
  result.position_size_limit = input.position_size * 0.8;  // Conservative limit

  return result;
}

// Export bindings to JavaScript
EMSCRIPTEN_BINDINGS(box_spread_wasm) {
  // BoxSpreadInput type
  value_object<wasm::BoxSpreadInput>("BoxSpreadInput")
    .field("longCallStrike", &wasm::BoxSpreadInput::long_call_strike)
    .field("longCallBid", &wasm::BoxSpreadInput::long_call_bid)
    .field("longCallAsk", &wasm::BoxSpreadInput::long_call_ask)
    .field("shortCallStrike", &wasm::BoxSpreadInput::short_call_strike)
    .field("shortCallBid", &wasm::BoxSpreadInput::short_call_bid)
    .field("shortCallAsk", &wasm::BoxSpreadInput::short_call_ask)
    .field("longPutStrike", &wasm::BoxSpreadInput::long_put_strike)
    .field("longPutBid", &wasm::BoxSpreadInput::long_put_bid)
    .field("longPutAsk", &wasm::BoxSpreadInput::long_put_ask)
    .field("shortPutStrike", &wasm::BoxSpreadInput::short_put_strike)
    .field("shortPutBid", &wasm::BoxSpreadInput::short_put_bid)
    .field("shortPutAsk", &wasm::BoxSpreadInput::short_put_ask)
    .field("underlyingPrice", &wasm::BoxSpreadInput::underlying_price)
    .field("riskFreeRate", &wasm::BoxSpreadInput::risk_free_rate)
    .field("daysToExpiry", &wasm::BoxSpreadInput::days_to_expiry)
    .field("volatility", &wasm::BoxSpreadInput::volatility)
    ;

  register_vector<wasm::BoxSpreadInput>("BoxSpreadInputVector");

  // BoxSpreadResult type
  value_object<wasm::BoxSpreadResult>("BoxSpreadResult")
    .field("netDebit", &wasm::BoxSpreadResult::net_debit)
    .field("arbitrageProfit", &wasm::BoxSpreadResult::arbitrage_profit)
    .field("roi", &wasm::BoxSpreadResult::roi)
    .field("apr", &wasm::BoxSpreadResult::apr)
    .field("confidenceScore", &wasm::BoxSpreadResult::confidence_score)
    .field("isProfitable", &wasm::BoxSpreadResult::is_profitable)
    .field("riskScore", &wasm::BoxSpreadResult::risk_score)
    .field("delta", &wasm::BoxSpreadResult::delta)
    .field("gamma", &wasm::BoxSpreadResult::gamma)
    .field("theta", &wasm::BoxSpreadResult::theta)
    .field("vega", &wasm::BoxSpreadResult::vega)
    ;

  register_vector<wasm::BoxSpreadResult>("BoxSpreadResultVector");

  // RiskInput type
  value_object<wasm::RiskInput>("RiskInput")
    .field("positionSize", &wasm::RiskInput::position_size)
    .field("volatility", &wasm::RiskInput::volatility)
    .field("timeHorizonDays", &wasm::RiskInput::time_horizon_days)
    .field("confidenceLevel", &wasm::RiskInput::confidence_level)
    ;

  // RiskResult type
  value_object<wasm::RiskResult>("RiskResult")
    .field("var", &wasm::RiskResult::var)
    .field("expectedLoss", &wasm::RiskResult::expected_loss)
    .field("maxLoss", &wasm::RiskResult::max_loss)
    .field("positionSizeLimit", &wasm::RiskResult::position_size_limit)
    ;

  // Export functions
  function("calculateBoxSpread", &calculate_box_spread);
  function("calculateRisk", &calculate_risk);
}
