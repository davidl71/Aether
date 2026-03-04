// proto_adapter.cpp - Convert between native types and protobuf DTOs.
#include "proto_adapter.h"
#include "messages.pb.h"
#include <google/protobuf/stubs/common.h>

namespace proto_adapter {

namespace {
::ib::platform::v1::OptionTypeEnum option_type_to_proto(types::OptionType t) {
  return t == types::OptionType::Call
             ? ::ib::platform::v1::OPTION_TYPE_CALL
             : ::ib::platform::v1::OPTION_TYPE_PUT;
}
types::OptionType option_type_from_proto(::ib::platform::v1::OptionTypeEnum t) {
  return t == ::ib::platform::v1::OPTION_TYPE_PUT ? types::OptionType::Put
                                                   : types::OptionType::Call;
}
}  // namespace

void to_proto(const types::OptionContract& from,
              ::ib::platform::v1::OptionContract* out) {
  if (!out) return;
  out->set_symbol(from.symbol);
  out->set_expiry(from.expiry);
  out->set_strike(from.strike);
  out->set_option_type(option_type_to_proto(from.type));
  out->set_exchange(from.exchange);
  out->set_local_symbol(from.local_symbol);
}

void from_proto(const ::ib::platform::v1::OptionContract& from,
               types::OptionContract* out) {
  if (!out) return;
  out->symbol = from.symbol();
  out->expiry = from.expiry();
  out->strike = from.strike();
  out->type = option_type_from_proto(from.option_type());
  out->exchange = from.exchange();
  out->local_symbol = from.local_symbol();
}

void to_proto(const types::BoxSpreadLeg& from,
              ::ib::platform::v1::BoxSpreadLeg* out) {
  if (!out) return;
  to_proto(from.long_call, out->mutable_long_call());
  to_proto(from.short_call, out->mutable_short_call());
  to_proto(from.long_put, out->mutable_long_put());
  to_proto(from.short_put, out->mutable_short_put());
  out->set_net_debit(from.net_debit);
  out->set_theoretical_value(from.theoretical_value);
  out->set_arbitrage_profit(from.arbitrage_profit);
  out->set_roi_percent(from.roi_percent);
  out->set_long_call_price(from.long_call_price);
  out->set_short_call_price(from.short_call_price);
  out->set_long_put_price(from.long_put_price);
  out->set_short_put_price(from.short_put_price);
  out->set_long_call_bid_ask_spread(from.long_call_bid_ask_spread);
  out->set_short_call_bid_ask_spread(from.short_call_bid_ask_spread);
  out->set_long_put_bid_ask_spread(from.long_put_bid_ask_spread);
  out->set_short_put_bid_ask_spread(from.short_put_bid_ask_spread);
  out->set_buy_net_debit(from.buy_net_debit);
  out->set_buy_profit(from.buy_profit);
  out->set_buy_implied_rate(from.buy_implied_rate);
  out->set_sell_net_credit(from.sell_net_credit);
  out->set_sell_profit(from.sell_profit);
  out->set_sell_implied_rate(from.sell_implied_rate);
  out->set_buy_sell_disparity(from.buy_sell_disparity);
  out->set_put_call_parity_violation(from.put_call_parity_violation);
}

void from_proto(const ::ib::platform::v1::BoxSpreadLeg& from,
               types::BoxSpreadLeg* out) {
  if (!out) return;
  from_proto(from.long_call(), &out->long_call);
  from_proto(from.short_call(), &out->short_call);
  from_proto(from.long_put(), &out->long_put);
  from_proto(from.short_put(), &out->short_put);
  out->net_debit = from.net_debit();
  out->theoretical_value = from.theoretical_value();
  out->arbitrage_profit = from.arbitrage_profit();
  out->roi_percent = from.roi_percent();
  out->long_call_price = from.long_call_price();
  out->short_call_price = from.short_call_price();
  out->long_put_price = from.long_put_price();
  out->short_put_price = from.short_put_price();
  out->long_call_bid_ask_spread = from.long_call_bid_ask_spread();
  out->short_call_bid_ask_spread = from.short_call_bid_ask_spread();
  out->long_put_bid_ask_spread = from.long_put_bid_ask_spread();
  out->short_put_bid_ask_spread = from.short_put_bid_ask_spread();
  out->buy_net_debit = from.buy_net_debit();
  out->buy_profit = from.buy_profit();
  out->buy_implied_rate = from.buy_implied_rate();
  out->sell_net_credit = from.sell_net_credit();
  out->sell_profit = from.sell_profit();
  out->sell_implied_rate = from.sell_implied_rate();
  out->buy_sell_disparity = from.buy_sell_disparity();
  out->put_call_parity_violation = from.put_call_parity_violation();
}

std::string box_spread_leg_to_proto_bytes(const types::BoxSpreadLeg& leg) {
  ::ib::platform::v1::BoxSpreadLeg msg;
  to_proto(leg, &msg);
  std::string bytes;
  if (!msg.SerializeToString(&bytes)) return {};
  return bytes;
}

bool proto_bytes_to_box_spread_leg(const std::string& bytes,
                                   types::BoxSpreadLeg* out) {
  if (!out) return false;
  ::ib::platform::v1::BoxSpreadLeg msg;
  if (!msg.ParseFromString(bytes)) return false;
  from_proto(msg, out);
  return true;
}

}  // namespace proto_adapter
