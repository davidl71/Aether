// tui_data.h - Data structures for TUI display
//
// NOTE: These are display-focused types for the TUI. For trading operations,
// use the native types in types.h. Conversion functions are in tui_converter.h
//
#pragma once

#include <string>
#include <vector>
#include <chrono>
#include <nlohmann/json.hpp>

namespace tui {

// ============================================================================
// TUI Display Types
// ============================================================================
//
// These types are optimized for display in the TUI. They are simpler and
// more focused on presentation than the native trading types in types.h.
//
// When to use:
// - TUI types: For display, JSON serialization, REST API responses
// - Native types (types::*): For trading operations, calculations, TWS integration
//
// Conversion: Use functions in tui_converter.h to convert between them.
//
// ============================================================================

// Candle represents OHLC data for a period.
struct Candle {
  double open = 0.0;
  double high = 0.0;
  double low = 0.0;
  double close = 0.0;
  double volume = 0.0;
  double entry = 0.0;
  std::chrono::system_clock::time_point updated;
};

// OptionStrike represents call/put quotes for a given strike.
struct OptionStrike {
  double strike = 0.0;
  double call_bid = 0.0;
  double call_ask = 0.0;
  double put_bid = 0.0;
  double put_ask = 0.0;
};

// OptionSeries is a set of strikes for a particular expiration.
struct OptionSeries {
  std::string expiration;
  std::vector<OptionStrike> strikes;
};

// SymbolSnapshot describes top-line data for an underlying or combo.
struct SymbolSnapshot {
  std::string symbol;
  double last = 0.0;
  double bid = 0.0;
  double ask = 0.0;
  double spread = 0.0;
  double roi = 0.0;
  int maker_count = 0;
  int taker_count = 0;
  int volume = 0;
  Candle candle;
  double multiplier = 100.0;
  std::vector<OptionSeries> option_chains;
};

// Position snapshot.
struct Position {
  std::string name;
  int quantity = 0;
  double roi = 0.0;
  int maker_count = 0;
  int taker_count = 0;
  double rebate_estimate = 0.0;
  double vega = 0.0;
  double theta = 0.0;
  double fair_diff = 0.0;
  Candle candle;
};

// Order event.
struct Order {
  std::chrono::system_clock::time_point timestamp;
  std::string text;
  std::string severity;  // "info", "warn", "error", "success"
};

// Alert event.
struct Alert {
  std::chrono::system_clock::time_point timestamp;
  std::string text;
  std::string severity;  // "info", "warn", "error", "success"
};

// AccountMetrics summarises account state.
struct AccountMetrics {
  double net_liq = 0.0;
  double buying_power = 0.0;
  double excess_liquidity = 0.0;
  double margin_requirement = 0.0;
  double commissions = 0.0;
  bool portal_ok = false;
  bool tws_ok = false;
  bool orats_ok = false;
  bool questdb_ok = false;
};

// HistoryEntry captures historical box spread observations.
struct HistoryEntry {
  std::chrono::system_clock::time_point date;
  std::string symbol;
  std::string expiration;
  double width = 0.0;
  double net_debit = 0.0;
  double apr = 0.0;
  std::string benchmark;
  double benchmark_rate = 0.0;
  std::string notes;
  double days_to_expiry = 0.0;
  std::string option_style;  // "European" or "American"

  // Buy vs Sell disparity tracking
  double buy_profit = 0.0;
  double sell_profit = 0.0;
  double buy_sell_disparity = 0.0;  // Difference between buy and sell profitability
  double put_call_parity_violation = 0.0;  // Put-call parity violation (bps)
};

// YieldCurvePoint summarises current synthetic funding along the term structure.
struct YieldCurvePoint {
  std::string label;
  std::string expiration;
  double dte = 0.0;
  double net_debit = 0.0;
  double apr = 0.0;
  double benchmark = 0.0;
  double apr_spread = 0.0;
  std::string option_style;  // "European" or "American"

  // Buy vs Sell disparity (intraday differences)
  double buy_profit = 0.0;
  double sell_profit = 0.0;
  double buy_sell_disparity = 0.0;
  double put_call_parity_violation = 0.0;  // Put-call parity violation (bps)
};

// FAQEntry contains frequently asked questions content for offline reference.
struct FAQEntry {
  std::string question;
  std::string answer;
};

// Snapshot is the aggregate data served to the TUI.
struct Snapshot {
  std::chrono::system_clock::time_point generated_at;
  std::string mode;  // "DRY-RUN", "LIVE"
  std::string strategy;  // "RUNNING", "STOPPED"
  std::string account_id;
  AccountMetrics metrics;
  std::vector<SymbolSnapshot> symbols;
  std::vector<Position> positions;
  std::vector<Position> historic;
  std::vector<Order> orders;
  std::vector<Alert> alerts;
  std::vector<HistoryEntry> history;
  std::vector<YieldCurvePoint> yield_curve;
  std::vector<FAQEntry> faqs;
};

// JSON serialization helpers
void to_json(nlohmann::json& j, const Candle& c);
void from_json(const nlohmann::json& j, Candle& c);

void to_json(nlohmann::json& j, const OptionStrike& s);
void from_json(const nlohmann::json& j, OptionStrike& s);

void to_json(nlohmann::json& j, const OptionSeries& s);
void from_json(const nlohmann::json& j, OptionSeries& s);

void to_json(nlohmann::json& j, const SymbolSnapshot& s);
void from_json(const nlohmann::json& j, SymbolSnapshot& s);

void to_json(nlohmann::json& j, const Position& p);
void from_json(const nlohmann::json& j, Position& p);

void to_json(nlohmann::json& j, const Order& o);
void from_json(const nlohmann::json& j, Order& o);

void to_json(nlohmann::json& j, const Alert& a);
void from_json(const nlohmann::json& j, Alert& a);

void to_json(nlohmann::json& j, const AccountMetrics& m);
void from_json(const nlohmann::json& j, AccountMetrics& m);

void to_json(nlohmann::json& j, const HistoryEntry& h);
void from_json(const nlohmann::json& j, HistoryEntry& h);

void to_json(nlohmann::json& j, const YieldCurvePoint& p);
void from_json(const nlohmann::json& j, YieldCurvePoint& p);

void to_json(nlohmann::json& j, const FAQEntry& f);
void from_json(const nlohmann::json& j, FAQEntry& f);

void to_json(nlohmann::json& j, const Snapshot& s);
void from_json(const nlohmann::json& j, Snapshot& s);

} // namespace tui
