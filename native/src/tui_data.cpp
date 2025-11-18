// tui_data.cpp - JSON serialization implementations for TUI data structures
#include "tui_data.h"
#include <nlohmann/json.hpp>
#include <chrono>

namespace tui {

// ============================================================================
// Candle JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const Candle& c) {
  j = nlohmann::json{
    {"open", c.open},
    {"high", c.high},
    {"low", c.low},
    {"close", c.close},
    {"volume", c.volume},
    {"entry", c.entry},
    {"updated", std::chrono::duration_cast<std::chrono::seconds>(
      c.updated.time_since_epoch()).count()}
  };
}

void from_json(const nlohmann::json& j, Candle& c) {
  j.at("open").get_to(c.open);
  j.at("high").get_to(c.high);
  j.at("low").get_to(c.low);
  j.at("close").get_to(c.close);
  if (j.contains("volume")) j.at("volume").get_to(c.volume);
  if (j.contains("entry")) j.at("entry").get_to(c.entry);
  if (j.contains("updated")) {
    int64_t seconds = j.at("updated").get<int64_t>();
    c.updated = std::chrono::system_clock::time_point(
      std::chrono::seconds(seconds));
  }
}

// ============================================================================
// OptionStrike JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const OptionStrike& s) {
  j = nlohmann::json{
    {"strike", s.strike},
    {"call_bid", s.call_bid},
    {"call_ask", s.call_ask},
    {"put_bid", s.put_bid},
    {"put_ask", s.put_ask}
  };
}

void from_json(const nlohmann::json& j, OptionStrike& s) {
  j.at("strike").get_to(s.strike);
  j.at("call_bid").get_to(s.call_bid);
  j.at("call_ask").get_to(s.call_ask);
  j.at("put_bid").get_to(s.put_bid);
  j.at("put_ask").get_to(s.put_ask);
}

// ============================================================================
// OptionSeries JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const OptionSeries& s) {
  j = nlohmann::json{
    {"expiration", s.expiration},
    {"strikes", s.strikes}
  };
}

void from_json(const nlohmann::json& j, OptionSeries& s) {
  j.at("expiration").get_to(s.expiration);
  j.at("strikes").get_to(s.strikes);
}

// ============================================================================
// SymbolSnapshot JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const SymbolSnapshot& s) {
  j = nlohmann::json{
    {"symbol", s.symbol},
    {"last", s.last},
    {"bid", s.bid},
    {"ask", s.ask},
    {"spread", s.spread},
    {"roi", s.roi},
    {"maker_count", s.maker_count},
    {"taker_count", s.taker_count},
    {"volume", s.volume},
    {"candle", s.candle},
    {"multiplier", s.multiplier},
    {"option_chains", s.option_chains}
  };
}

void from_json(const nlohmann::json& j, SymbolSnapshot& s) {
  j.at("symbol").get_to(s.symbol);
  if (j.contains("last")) j.at("last").get_to(s.last);
  if (j.contains("bid")) j.at("bid").get_to(s.bid);
  if (j.contains("ask")) j.at("ask").get_to(s.ask);
  if (j.contains("spread")) j.at("spread").get_to(s.spread);
  if (j.contains("roi")) j.at("roi").get_to(s.roi);
  if (j.contains("maker_count")) j.at("maker_count").get_to(s.maker_count);
  if (j.contains("taker_count")) j.at("taker_count").get_to(s.taker_count);
  if (j.contains("volume")) j.at("volume").get_to(s.volume);
  if (j.contains("candle")) j.at("candle").get_to(s.candle);
  if (j.contains("multiplier")) j.at("multiplier").get_to(s.multiplier);
  if (j.contains("option_chains")) j.at("option_chains").get_to(s.option_chains);
}

// ============================================================================
// Position JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const Position& p) {
  j = nlohmann::json{
    {"name", p.name},
    {"quantity", p.quantity},
    {"roi", p.roi},
    {"maker_count", p.maker_count},
    {"taker_count", p.taker_count},
    {"rebate_estimate", p.rebate_estimate},
    {"vega", p.vega},
    {"theta", p.theta},
    {"fair_diff", p.fair_diff},
    {"candle", p.candle}
  };
}

void from_json(const nlohmann::json& j, Position& p) {
  j.at("name").get_to(p.name);
  if (j.contains("quantity")) j.at("quantity").get_to(p.quantity);
  if (j.contains("roi")) j.at("roi").get_to(p.roi);
  if (j.contains("maker_count")) j.at("maker_count").get_to(p.maker_count);
  if (j.contains("taker_count")) j.at("taker_count").get_to(p.taker_count);
  if (j.contains("rebate_estimate")) j.at("rebate_estimate").get_to(p.rebate_estimate);
  if (j.contains("vega")) j.at("vega").get_to(p.vega);
  if (j.contains("theta")) j.at("theta").get_to(p.theta);
  if (j.contains("fair_diff")) j.at("fair_diff").get_to(p.fair_diff);
  if (j.contains("candle")) j.at("candle").get_to(p.candle);
}

// ============================================================================
// Order JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const Order& o) {
  j = nlohmann::json{
    {"timestamp", std::chrono::duration_cast<std::chrono::seconds>(
      o.timestamp.time_since_epoch()).count()},
    {"text", o.text},
    {"severity", o.severity}
  };
}

void from_json(const nlohmann::json& j, Order& o) {
  if (j.contains("timestamp")) {
    int64_t seconds = j.at("timestamp").get<int64_t>();
    o.timestamp = std::chrono::system_clock::time_point(
      std::chrono::seconds(seconds));
  }
  j.at("text").get_to(o.text);
  if (j.contains("severity")) j.at("severity").get_to(o.severity);
}

// ============================================================================
// Alert JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const Alert& a) {
  j = nlohmann::json{
    {"timestamp", std::chrono::duration_cast<std::chrono::seconds>(
      a.timestamp.time_since_epoch()).count()},
    {"text", a.text},
    {"severity", a.severity}
  };
}

void from_json(const nlohmann::json& j, Alert& a) {
  if (j.contains("timestamp")) {
    int64_t seconds = j.at("timestamp").get<int64_t>();
    a.timestamp = std::chrono::system_clock::time_point(
      std::chrono::seconds(seconds));
  }
  j.at("text").get_to(a.text);
  if (j.contains("severity")) j.at("severity").get_to(a.severity);
}

// ============================================================================
// AccountMetrics JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const AccountMetrics& m) {
  j = nlohmann::json{
    {"net_liq", m.net_liq},
    {"buying_power", m.buying_power},
    {"excess_liquidity", m.excess_liquidity},
    {"margin_requirement", m.margin_requirement},
    {"commissions", m.commissions},
    {"portal_ok", m.portal_ok},
    {"tws_ok", m.tws_ok},
    {"orats_ok", m.orats_ok},
    {"questdb_ok", m.questdb_ok}
  };
}

void from_json(const nlohmann::json& j, AccountMetrics& m) {
  if (j.contains("net_liq")) j.at("net_liq").get_to(m.net_liq);
  if (j.contains("buying_power")) j.at("buying_power").get_to(m.buying_power);
  if (j.contains("excess_liquidity")) j.at("excess_liquidity").get_to(m.excess_liquidity);
  if (j.contains("margin_requirement")) j.at("margin_requirement").get_to(m.margin_requirement);
  if (j.contains("commissions")) j.at("commissions").get_to(m.commissions);
  if (j.contains("portal_ok")) j.at("portal_ok").get_to(m.portal_ok);
  if (j.contains("tws_ok")) j.at("tws_ok").get_to(m.tws_ok);
  if (j.contains("orats_ok")) j.at("orats_ok").get_to(m.orats_ok);
  if (j.contains("questdb_ok")) j.at("questdb_ok").get_to(m.questdb_ok);
}

// ============================================================================
// HistoryEntry JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const HistoryEntry& h) {
  j = nlohmann::json{
    {"date", std::chrono::duration_cast<std::chrono::seconds>(
      h.date.time_since_epoch()).count()},
    {"symbol", h.symbol},
    {"expiration", h.expiration},
    {"width", h.width},
    {"net_debit", h.net_debit},
    {"apr", h.apr},
    {"benchmark", h.benchmark},
    {"benchmark_rate", h.benchmark_rate},
    {"notes", h.notes},
    {"days_to_expiry", h.days_to_expiry},
    {"option_style", h.option_style},
    {"buy_profit", h.buy_profit},
    {"sell_profit", h.sell_profit},
    {"buy_sell_disparity", h.buy_sell_disparity},
    {"put_call_parity_violation", h.put_call_parity_violation}
  };
}

void from_json(const nlohmann::json& j, HistoryEntry& h) {
  if (j.contains("date")) {
    int64_t seconds = j.at("date").get<int64_t>();
    h.date = std::chrono::system_clock::time_point(std::chrono::seconds(seconds));
  }
  if (j.contains("symbol")) j.at("symbol").get_to(h.symbol);
  if (j.contains("expiration")) j.at("expiration").get_to(h.expiration);
  if (j.contains("width")) j.at("width").get_to(h.width);
  if (j.contains("net_debit")) j.at("net_debit").get_to(h.net_debit);
  if (j.contains("apr")) j.at("apr").get_to(h.apr);
  if (j.contains("benchmark")) j.at("benchmark").get_to(h.benchmark);
  if (j.contains("benchmark_rate")) j.at("benchmark_rate").get_to(h.benchmark_rate);
  if (j.contains("notes")) j.at("notes").get_to(h.notes);
  if (j.contains("days_to_expiry")) j.at("days_to_expiry").get_to(h.days_to_expiry);
  if (j.contains("option_style")) j.at("option_style").get_to(h.option_style);
  if (j.contains("buy_profit")) j.at("buy_profit").get_to(h.buy_profit);
  if (j.contains("sell_profit")) j.at("sell_profit").get_to(h.sell_profit);
  if (j.contains("buy_sell_disparity")) j.at("buy_sell_disparity").get_to(h.buy_sell_disparity);
  if (j.contains("put_call_parity_violation")) j.at("put_call_parity_violation").get_to(h.put_call_parity_violation);
}

// ============================================================================
// YieldCurvePoint JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const YieldCurvePoint& p) {
  j = nlohmann::json{
    {"label", p.label},
    {"expiration", p.expiration},
    {"dte", p.dte},
    {"net_debit", p.net_debit},
    {"apr", p.apr},
    {"benchmark", p.benchmark},
    {"apr_spread", p.apr_spread},
    {"option_style", p.option_style},
    {"buy_profit", p.buy_profit},
    {"sell_profit", p.sell_profit},
    {"buy_sell_disparity", p.buy_sell_disparity},
    {"put_call_parity_violation", p.put_call_parity_violation}
  };
}

void from_json(const nlohmann::json& j, YieldCurvePoint& p) {
  if (j.contains("label")) j.at("label").get_to(p.label);
  if (j.contains("expiration")) j.at("expiration").get_to(p.expiration);
  if (j.contains("dte")) j.at("dte").get_to(p.dte);
  if (j.contains("net_debit")) j.at("net_debit").get_to(p.net_debit);
  if (j.contains("apr")) j.at("apr").get_to(p.apr);
  if (j.contains("benchmark")) j.at("benchmark").get_to(p.benchmark);
  if (j.contains("apr_spread")) j.at("apr_spread").get_to(p.apr_spread);
  if (j.contains("option_style")) j.at("option_style").get_to(p.option_style);
  if (j.contains("buy_profit")) j.at("buy_profit").get_to(p.buy_profit);
  if (j.contains("sell_profit")) j.at("sell_profit").get_to(p.sell_profit);
  if (j.contains("buy_sell_disparity")) j.at("buy_sell_disparity").get_to(p.buy_sell_disparity);
  if (j.contains("put_call_parity_violation")) j.at("put_call_parity_violation").get_to(p.put_call_parity_violation);
}

// ============================================================================
// FAQEntry JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const FAQEntry& f) {
  j = nlohmann::json{
    {"question", f.question},
    {"answer", f.answer}
  };
}

void from_json(const nlohmann::json& j, FAQEntry& f) {
  j.at("question").get_to(f.question);
  j.at("answer").get_to(f.answer);
}

// ============================================================================
// Snapshot JSON Serialization
// ============================================================================

// ============================================================================
// BoxSpreadScenario JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const BoxSpreadScenario& s) {
  j = nlohmann::json{
    {"width", s.width},
    {"put_bid", s.put_bid},
    {"call_ask", s.call_ask},
    {"synthetic_bid", s.synthetic_bid},
    {"synthetic_ask", s.synthetic_ask},
    {"mid_price", s.mid_price},
    {"annualized_return", s.annualized_return},
    {"fill_probability", s.fill_probability},
    {"option_style", s.option_style}
  };
  if (s.buy_profit != 0.0) j["buy_profit"] = s.buy_profit;
  if (s.sell_profit != 0.0) j["sell_profit"] = s.sell_profit;
  if (s.buy_implied_rate != 0.0) j["buy_implied_rate"] = s.buy_implied_rate;
  if (s.sell_implied_rate != 0.0) j["sell_implied_rate"] = s.sell_implied_rate;
  if (s.buy_sell_disparity != 0.0) j["buy_sell_disparity"] = s.buy_sell_disparity;
  if (s.put_call_parity_violation != 0.0) j["put_call_parity_violation"] = s.put_call_parity_violation;
  if (!s.expiration_date.empty()) j["expiration_date"] = s.expiration_date;
  if (s.days_to_expiry != 0.0) j["days_to_expiry"] = s.days_to_expiry;
}

void from_json(const nlohmann::json& j, BoxSpreadScenario& s) {
  j.at("width").get_to(s.width);
  j.at("put_bid").get_to(s.put_bid);
  j.at("call_ask").get_to(s.call_ask);
  j.at("synthetic_bid").get_to(s.synthetic_bid);
  j.at("synthetic_ask").get_to(s.synthetic_ask);
  j.at("mid_price").get_to(s.mid_price);
  j.at("annualized_return").get_to(s.annualized_return);
  j.at("fill_probability").get_to(s.fill_probability);
  if (j.contains("option_style")) j.at("option_style").get_to(s.option_style);
  if (j.contains("buy_profit")) j.at("buy_profit").get_to(s.buy_profit);
  if (j.contains("sell_profit")) j.at("sell_profit").get_to(s.sell_profit);
  if (j.contains("buy_implied_rate")) j.at("buy_implied_rate").get_to(s.buy_implied_rate);
  if (j.contains("sell_implied_rate")) j.at("sell_implied_rate").get_to(s.sell_implied_rate);
  if (j.contains("buy_sell_disparity")) j.at("buy_sell_disparity").get_to(s.buy_sell_disparity);
  if (j.contains("put_call_parity_violation")) j.at("put_call_parity_violation").get_to(s.put_call_parity_violation);
  if (j.contains("expiration_date")) j.at("expiration_date").get_to(s.expiration_date);
  if (j.contains("days_to_expiry")) j.at("days_to_expiry").get_to(s.days_to_expiry);
}

// ============================================================================
// BoxSpreadSummary JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const BoxSpreadSummary& s) {
  j = nlohmann::json{
    {"total_scenarios", s.total_scenarios},
    {"avg_apr", s.avg_apr},
    {"probable_count", s.probable_count}
  };
  if (s.max_apr_scenario.width > 0.0) {
    j["max_apr_scenario"] = s.max_apr_scenario;
  }
}

void from_json(const nlohmann::json& j, BoxSpreadSummary& s) {
  j.at("total_scenarios").get_to(s.total_scenarios);
  j.at("avg_apr").get_to(s.avg_apr);
  j.at("probable_count").get_to(s.probable_count);
  if (j.contains("max_apr_scenario")) {
    j.at("max_apr_scenario").get_to(s.max_apr_scenario);
  }
}

// ============================================================================
// Snapshot JSON Serialization
// ============================================================================

void to_json(nlohmann::json& j, const Snapshot& s) {
  j = nlohmann::json{
    {"generated_at", std::chrono::duration_cast<std::chrono::seconds>(
      s.generated_at.time_since_epoch()).count()},
    {"mode", s.mode},
    {"strategy", s.strategy},
    {"account_id", s.account_id},
    {"metrics", s.metrics},
    {"symbols", s.symbols},
    {"positions", s.positions},
    {"historic", s.historic},
    {"orders", s.orders},
    {"alerts", s.alerts},
    {"history", s.history},
    {"yield_curve", s.yield_curve},
    {"faqs", s.faqs}
  };
  if (!s.scenarios.empty()) {
    j["scenarios"] = s.scenarios;
    j["scenario_summary"] = s.scenario_summary;
    if (!s.scenario_underlying.empty()) {
      j["scenario_underlying"] = s.scenario_underlying;
    }
    if (s.scenario_as_of.time_since_epoch().count() > 0) {
      j["scenario_as_of"] = std::chrono::duration_cast<std::chrono::seconds>(
        s.scenario_as_of.time_since_epoch()).count();
    }
  }
}

void from_json(const nlohmann::json& j, Snapshot& s) {
  if (j.contains("generated_at")) {
    int64_t seconds = j.at("generated_at").get<int64_t>();
    s.generated_at = std::chrono::system_clock::time_point(
      std::chrono::seconds(seconds));
  }
  if (j.contains("mode")) j.at("mode").get_to(s.mode);
  if (j.contains("strategy")) j.at("strategy").get_to(s.strategy);
  if (j.contains("account_id")) j.at("account_id").get_to(s.account_id);
  if (j.contains("metrics")) j.at("metrics").get_to(s.metrics);
  if (j.contains("symbols")) j.at("symbols").get_to(s.symbols);
  if (j.contains("positions")) j.at("positions").get_to(s.positions);
  if (j.contains("historic")) j.at("historic").get_to(s.historic);
  if (j.contains("orders")) j.at("orders").get_to(s.orders);
  if (j.contains("alerts")) j.at("alerts").get_to(s.alerts);
  if (j.contains("history")) j.at("history").get_to(s.history);
  if (j.contains("yield_curve")) j.at("yield_curve").get_to(s.yield_curve);
  if (j.contains("faqs")) j.at("faqs").get_to(s.faqs);
  if (j.contains("scenarios")) j.at("scenarios").get_to(s.scenarios);
  if (j.contains("scenario_summary")) j.at("scenario_summary").get_to(s.scenario_summary);
  if (j.contains("scenario_underlying")) j.at("scenario_underlying").get_to(s.scenario_underlying);
  if (j.contains("scenario_as_of")) {
    int64_t scenario_seconds = j.at("scenario_as_of").get<int64_t>();
    s.scenario_as_of = std::chrono::system_clock::time_point(std::chrono::seconds(scenario_seconds));
  }
}

} // namespace tui
