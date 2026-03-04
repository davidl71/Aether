// mock_data_generator.cpp - Mock data generator implementation
#include "mock_data_generator.h"
#include "strategies/box_spread/box_spread_bag.h"
#include <algorithm>
#include <cmath>
#include <iomanip>
#include <spdlog/spdlog.h>
#include <sstream>

namespace mock {

// ============================================================================
// MockBoxSpreadBagGenerator Implementation
// ============================================================================

MockBoxSpreadBagGenerator::MockBoxSpreadBagGenerator()
    : rng_(std::random_device{}()), price_dist_(0.01, 1000.0),
      volume_dist_(1.0, 1000.0),
      price_change_dist_(0.0, 0.02) { // 2% std dev for price changes
}

types::BoxSpreadBag MockBoxSpreadBagGenerator::generate_bag(
    const std::string &underlying_symbol, const std::string &expiry,
    double strike_width, int days_to_expiry) {

  // Generate realistic box spread leg
  types::BoxSpreadLeg spread;

  // Determine strikes based on a hypothetical underlying price
  double underlying_price = 4500.0; // Default for SPX
  if (underlying_symbol == "ES" ||
      underlying_symbol.find("ES") != std::string::npos) {
    underlying_price = 4500.0; // E-mini S&P 500
  } else if (underlying_symbol == "XSP") {
    underlying_price = 450.0;           // Mini SPX (1/10th)
    strike_width = strike_width / 10.0; // Adjust for mini
  }

  double strike_low = underlying_price - strike_width / 2.0;
  double strike_high = underlying_price + strike_width / 2.0;

  // Create option contracts
  // Determine option style: SPX is European, others are typically American
  types::OptionStyle style =
      (underlying_symbol == "SPX" || underlying_symbol == "XSP")
          ? types::OptionStyle::European
          : types::OptionStyle::American;

  spread.long_call = types::OptionContract{underlying_symbol,
                                           expiry,
                                           strike_low,
                                           types::OptionType::Call,
                                           style,
                                           "CBOE",
                                           ""};
  spread.short_call = types::OptionContract{underlying_symbol,
                                            expiry,
                                            strike_high,
                                            types::OptionType::Call,
                                            style,
                                            "CBOE",
                                            ""};
  spread.long_put = types::OptionContract{underlying_symbol,
                                          expiry,
                                          strike_high,
                                          types::OptionType::Put,
                                          style,
                                          "CBOE",
                                          ""};
  spread.short_put = types::OptionContract{underlying_symbol,
                                           expiry,
                                           strike_low,
                                           types::OptionType::Put,
                                           style,
                                           "CBOE",
                                           ""};

  // Generate realistic net debit based on implied rate
  double implied_rate_percent = 5.0 + (rng_() % 200) / 10.0; // 5.0% - 25.0%
  spread.net_debit = generate_realistic_net_debit(strike_width, days_to_expiry,
                                                  implied_rate_percent);
  spread.theoretical_value = strike_width;
  spread.arbitrage_profit = spread.theoretical_value - spread.net_debit;
  spread.roi_percent =
      spread.net_debit > 0
          ? (spread.arbitrage_profit / spread.net_debit) * 100.0
          : 0.0;

  // Generate realistic option prices
  double time_to_expiry = days_to_expiry / 365.0;
  double volatility = 0.15 + (rng_() % 100) / 1000.0; // 15% - 25% IV

  // Generate prices (simplified - would use Black-Scholes in full
  // implementation)
  spread.long_call_price = add_realistic_noise(strike_width * 0.45, 2.0);
  spread.short_call_price = add_realistic_noise(strike_width * 0.35, 2.0);
  spread.long_put_price = add_realistic_noise(strike_width * 0.35, 2.0);
  spread.short_put_price = add_realistic_noise(strike_width * 0.25, 2.0);

  // Adjust to match net debit
  double current_net = spread.long_call_price - spread.short_call_price +
                       spread.long_put_price - spread.short_put_price;
  double adjustment = (spread.net_debit - current_net) / 4.0;
  spread.long_call_price += adjustment;
  spread.short_call_price -= adjustment;
  spread.long_put_price += adjustment;
  spread.short_put_price -= adjustment;

  // Generate bid-ask spreads
  spread.long_call_bid_ask_spread =
      0.05 + (rng_() % 50) / 1000.0; // $0.05 - $0.10
  spread.short_call_bid_ask_spread = 0.05 + (rng_() % 50) / 1000.0;
  spread.long_put_bid_ask_spread = 0.05 + (rng_() % 50) / 1000.0;
  spread.short_put_bid_ask_spread = 0.05 + (rng_() % 50) / 1000.0;

  // Generate expiry if not provided
  std::string actual_expiry =
      expiry.empty() ? generate_expiry_date(days_to_expiry) : expiry;

  // Create bag from spread
  types::BoxSpreadBag bag = types::BoxSpreadBagManager::create_bag_from_spread(
      spread, underlying_symbol);

  // Update with mock market data
  update_bag_market_data(bag, underlying_price);

  // Update Greeks
  update_greeks_for_bag(bag, underlying_price);

  spdlog::debug("Generated mock bag: {} (DTE: {}, Width: {:.2f})",
                bag.cboe_symbol, days_to_expiry, strike_width);

  return bag;
}

std::vector<types::BoxSpreadBag>
MockBoxSpreadBagGenerator::generate_bags(size_t count,
                                         const std::string &underlying_symbol) {

  std::vector<types::BoxSpreadBag> bags;
  bags.reserve(count);

  for (size_t i = 0; i < count; ++i) {
    int dte = 7 + (rng_() % 173);                       // 7-180 days
    double strike_width = 50.0 + (rng_() % 450) / 10.0; // $5 - $50

    bags.push_back(generate_bag(underlying_symbol, "", strike_width, dte));
  }

  return bags;
}

void MockBoxSpreadBagGenerator::simulate_price_movement(
    types::BoxSpreadBag &bag, double underlying_price, int num_updates,
    double time_step_minutes) {

  double current_price = bag.market_data.get_mid_price();

  for (int i = 0; i < num_updates; ++i) {
    // Simulate price movement (random walk with slight mean reversion to
    // theoretical)
    double theoretical = bag.theoretical_value;
    double mean_reversion = (theoretical - current_price) * 0.1;
    double random_change = price_change_dist_(rng_) * current_price;

    current_price += mean_reversion + random_change;
    current_price = std::max(0.01, current_price); // Prevent negative prices

    // Update bid/ask around mid price with spread
    double spread_amount = current_price * 0.001; // 0.1% spread
    double bid = current_price - spread_amount / 2.0;
    double ask = current_price + spread_amount / 2.0;

    types::BoxSpreadBagManager::update_bag_market_data(bag, bid, ask,
                                                       current_price, 10, 10);

    // Update Greeks periodically
    if (i % 5 == 0) {
      update_greeks_for_bag(bag, underlying_price);
    }
  }
}

void MockBoxSpreadBagGenerator::generate_candle_history(
    types::BoxSpreadBag &bag, int num_candles, double base_price) {

  if (base_price <= 0) {
    base_price = bag.market_data.get_mid_price();
  }

  double current_price = base_price;

  for (int i = 0; i < num_candles; ++i) {
    types::BoxSpreadBag::BagCandle candle;

    // Simulate candle with realistic price movement
    double open = current_price;
    double change = price_change_dist_(rng_) * open;
    double high = open + std::abs(change) * (0.5 + (rng_() % 50) / 100.0);
    double low = open - std::abs(change) * (0.5 + (rng_() % 50) / 100.0);
    double close = open + change;

    candle.open = std::max(0.01, open);
    candle.high = std::max({candle.open, high, close});
    candle.low = std::min({candle.open, low, close});
    candle.close = std::max(0.01, close);
    candle.volume = volume_dist_(rng_);
    candle.entry =
        bag.position.entry_price > 0 ? bag.position.entry_price : base_price;

    auto now = std::chrono::system_clock::now();
    auto candle_time = now - std::chrono::minutes(num_candles - i);
    candle.period_start = candle_time;
    candle.period_end = candle_time + std::chrono::minutes(5);
    candle.updated = candle.period_end;

    bag.candle_history.push_back(candle);
    current_price = close;
  }
}

void MockBoxSpreadBagGenerator::update_bag_market_data(
    types::BoxSpreadBag &bag, double underlying_price) {

  // Generate realistic bid/ask around theoretical value
  double theoretical = bag.theoretical_value;
  double mid_price = bag.net_debit;

  // Add small random variation
  mid_price += add_realistic_noise(mid_price, 0.5);
  mid_price = std::max(0.01, mid_price);

  // Generate bid/ask with spread
  double spread_pct = 0.001 + (rng_() % 50) / 100000.0; // 0.1% - 0.6%
  double spread_amount = mid_price * spread_pct;
  double bid = mid_price - spread_amount / 2.0;
  double ask = mid_price + spread_amount / 2.0;

  int bid_size = 5 + (rng_() % 50);
  int ask_size = 5 + (rng_() % 50);

  types::BoxSpreadBagManager::update_bag_market_data(bag, bid, ask, mid_price,
                                                     bid_size, ask_size);

  // Update liquidity score
  bag.liquidity_score = 50.0 + (rng_() % 50); // 50-100
  bag.execution_probability = bag.liquidity_score / 100.0;
}

std::vector<types::BoxSpreadBag>
MockBoxSpreadBagGenerator::generate_multi_symbol_bags(
    const std::vector<std::string> &symbols, double strike_width,
    int days_to_expiry) {

  std::vector<types::BoxSpreadBag> bags;
  bags.reserve(symbols.size());

  for (const auto &symbol : symbols) {
    bags.push_back(generate_bag(symbol, "", strike_width, days_to_expiry));
  }

  return bags;
}

std::vector<types::BoxSpreadBag>
MockBoxSpreadBagGenerator::generate_yield_curve_bags(
    const std::string &symbol, double strike_width,
    const std::vector<int> &days_to_expiry_list, double underlying_price) {

  std::vector<types::BoxSpreadBag> bags;
  bags.reserve(days_to_expiry_list.size());

  for (int dte : days_to_expiry_list) {
    bags.push_back(generate_bag(symbol, "", strike_width, dte));
  }

  return bags;
}

// ============================================================================
// Helper Methods
// ============================================================================

double MockBoxSpreadBagGenerator::generate_realistic_net_debit(
    double strike_width, int days_to_expiry, double implied_rate_percent) {

  // Calculate net debit from implied rate
  // For borrowing: net_debit = strike_width * (1 + rate * dte / 365)
  double time_to_expiry = days_to_expiry / 365.0;
  double rate_decimal = implied_rate_percent / 100.0;

  double net_debit = strike_width * (1.0 + rate_decimal * time_to_expiry);

  // Add small variation for realism
  return add_realistic_noise(net_debit, 0.5);
}

std::string
MockBoxSpreadBagGenerator::generate_expiry_date(int days_to_expiry) {
  auto now = std::chrono::system_clock::now();
  auto expiry_time = now + std::chrono::days(days_to_expiry);
  auto time_t = std::chrono::system_clock::to_time_t(expiry_time);

  std::tm tm = *std::localtime(&time_t);

  std::ostringstream oss;
  oss << std::setfill('0') << std::setw(2) << tm.tm_mday << std::setw(2)
      << (tm.tm_mon + 1) << std::setw(2) << (tm.tm_year % 100);

  return oss.str();
}

double MockBoxSpreadBagGenerator::add_realistic_noise(double value,
                                                      double std_dev_percent) {

  std::normal_distribution<double> noise_dist(0.0,
                                              value * std_dev_percent / 100.0);
  return value + noise_dist(rng_);
}

void MockBoxSpreadBagGenerator::update_greeks_for_bag(types::BoxSpreadBag &bag,
                                                      double underlying_price) {

  double volatility = 0.20;
  double risk_free_rate = 0.05;

  types::BoxSpreadBagManager::update_bag_greeks(bag, underlying_price,
                                                volatility, risk_free_rate);
}

// ============================================================================
// CboeSymbolFormatter Implementation
// ============================================================================

std::string CboeSymbolFormatter::format_complex_symbol(
    const std::string &underlying, const std::string &expiry, double strike_low,
    double strike_high) {

  return types::BoxSpreadBag::generate_cboe_symbol(underlying, expiry,
                                                   strike_low, strike_high);
}

bool CboeSymbolFormatter::parse_complex_symbol(const std::string &cboe_symbol,
                                               std::string &underlying,
                                               std::string &expiry,
                                               double &strike_low,
                                               double &strike_high) {

  // Parse format: "SPX 25JAN24 4500/4600 BOX"
  // Simplified parser - would need more robust implementation
  std::istringstream iss(cboe_symbol);
  iss >> underlying >> expiry >> strike_low;

  std::string separator;
  iss >> separator; // "/"
  iss >> strike_high;

  // Skip "BOX"
  std::string box;
  iss >> box;

  return !iss.fail() && box == "BOX";
}

} // namespace mock
