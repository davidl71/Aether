#include "proto_adapter.h"
#include "types.h"
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
