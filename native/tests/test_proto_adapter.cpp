#include "proto_adapter.h"
#include "config_manager.h"
#include "types.h"
#include "messages.pb.h"
#include <catch2/catch_test_macros.hpp>
#include <cmath>

TEST_CASE("Proto adapter round-trip BoxSpreadLeg", "[proto_adapter]") {
  using namespace types;
  BoxSpreadLeg leg;
  leg.long_call.symbol = "SPY";
  leg.long_call.expiry = "20251219";
  leg.long_call.strike = 600.0;
  leg.long_call.type = OptionType::Call;
  leg.long_call.exchange = "SMART";
  leg.short_call.symbol = "SPY";
  leg.short_call.expiry = "20251219";
  leg.short_call.strike = 610.0;
  leg.short_call.type = OptionType::Call;
  leg.long_put.symbol = "SPY";
  leg.long_put.expiry = "20251219";
  leg.long_put.strike = 610.0;
  leg.long_put.type = OptionType::Put;
  leg.short_put.symbol = "SPY";
  leg.short_put.expiry = "20251219";
  leg.short_put.strike = 600.0;
  leg.short_put.type = OptionType::Put;
  leg.net_debit = 9.80;
  leg.theoretical_value = 10.0;
  leg.arbitrage_profit = 0.20;
  leg.roi_percent = 2.04;
  leg.long_call_price = 12.5;
  leg.short_call_price = 3.2;
  leg.long_put_price = 0.5;
  leg.short_put_price = 0.0;
  leg.buy_net_debit = 9.85;
  leg.buy_profit = 0.15;
  leg.buy_implied_rate = 5.2;
  leg.sell_net_credit = 9.75;
  leg.sell_profit = 0.25;
  leg.sell_implied_rate = 5.4;
  leg.buy_sell_disparity = 0.10;
  leg.put_call_parity_violation = 2.5;

  std::string bytes = proto_adapter::box_spread_leg_to_proto_bytes(leg);
  REQUIRE_FALSE(bytes.empty());

  BoxSpreadLeg decoded;
  REQUIRE(proto_adapter::proto_bytes_to_box_spread_leg(bytes, &decoded));

  REQUIRE(decoded.long_call.symbol == leg.long_call.symbol);
  REQUIRE(decoded.long_call.expiry == leg.long_call.expiry);
  REQUIRE(decoded.long_call.strike == leg.long_call.strike);
  REQUIRE(decoded.long_call.type == leg.long_call.type);
  REQUIRE(decoded.short_call.strike == leg.short_call.strike);
  REQUIRE(decoded.long_put.strike == leg.long_put.strike);
  REQUIRE(decoded.short_put.strike == leg.short_put.strike);
  REQUIRE(std::abs(decoded.net_debit - leg.net_debit) < 1e-9);
  REQUIRE(std::abs(decoded.theoretical_value - leg.theoretical_value) < 1e-9);
  REQUIRE(std::abs(decoded.arbitrage_profit - leg.arbitrage_profit) < 1e-9);
  REQUIRE(std::abs(decoded.roi_percent - leg.roi_percent) < 1e-9);
  REQUIRE(std::abs(decoded.buy_net_debit - leg.buy_net_debit) < 1e-9);
  REQUIRE(std::abs(decoded.sell_implied_rate - leg.sell_implied_rate) < 1e-9);
}

TEST_CASE("Proto adapter round-trip StrategyParams", "[proto_adapter]") {
  config::StrategyParams from;
  from.symbols = {"SPX", "XSP", "NDX"};
  from.min_days_to_expiry = 30;
  from.max_days_to_expiry = 90;
  from.min_arbitrage_profit = 0.15;
  from.min_roi_percent = 0.6;
  from.max_position_size = 15000.0;
  from.max_bid_ask_spread = 0.12;
  from.min_volume = 150;
  from.min_open_interest = 600;
  from.benchmark_rate_percent = 5.25;
  from.benchmark_source = "sofr";
  from.treasury_api_url = "https://api.example.com";
  from.min_spread_over_benchmark_bps = 55.0;

  ::ib::platform::v1::StrategyParams pb;
  proto_adapter::to_proto(from, &pb);
  config::StrategyParams to;
  proto_adapter::from_proto(pb, &to);

  REQUIRE(to.symbols == from.symbols);
  REQUIRE(to.min_days_to_expiry == from.min_days_to_expiry);
  REQUIRE(to.max_days_to_expiry == from.max_days_to_expiry);
  REQUIRE(std::abs(to.min_arbitrage_profit - from.min_arbitrage_profit) < 1e-9);
  REQUIRE(std::abs(to.min_roi_percent - from.min_roi_percent) < 1e-9);
  REQUIRE(std::abs(to.max_position_size - from.max_position_size) < 1e-9);
  REQUIRE(std::abs(to.max_bid_ask_spread - from.max_bid_ask_spread) < 1e-9);
  REQUIRE(to.min_volume == from.min_volume);
  REQUIRE(to.min_open_interest == from.min_open_interest);
  REQUIRE(std::abs(to.benchmark_rate_percent - from.benchmark_rate_percent) < 1e-9);
  REQUIRE(to.benchmark_source == from.benchmark_source);
  REQUIRE(to.treasury_api_url == from.treasury_api_url);
  REQUIRE(std::abs(to.min_spread_over_benchmark_bps - from.min_spread_over_benchmark_bps) < 1e-9);
}

TEST_CASE("Proto adapter round-trip RiskConfig", "[proto_adapter]") {
  config::RiskConfig from;
  from.max_total_exposure = 60000.0;
  from.max_positions = 12;
  from.max_loss_per_position = 1200.0;
  from.max_daily_loss = 2500.0;
  from.position_size_percent = 0.12;
  from.enable_stop_loss = false;
  from.stop_loss_percent = 0.25;
  from.risk_free_rate_override = 4.5;

  ::ib::platform::v1::RiskConfig pb;
  proto_adapter::to_proto(from, &pb);
  config::RiskConfig to;
  proto_adapter::from_proto(pb, &to);

  REQUIRE(std::abs(to.max_total_exposure - from.max_total_exposure) < 1e-9);
  REQUIRE(to.max_positions == from.max_positions);
  REQUIRE(std::abs(to.max_loss_per_position - from.max_loss_per_position) < 1e-9);
  REQUIRE(std::abs(to.max_daily_loss - from.max_daily_loss) < 1e-9);
  REQUIRE(std::abs(to.position_size_percent - from.position_size_percent) < 1e-9);
  REQUIRE(to.enable_stop_loss == from.enable_stop_loss);
  REQUIRE(std::abs(to.stop_loss_percent - from.stop_loss_percent) < 1e-9);
  REQUIRE(std::abs(to.risk_free_rate_override - from.risk_free_rate_override) < 1e-9);
}
