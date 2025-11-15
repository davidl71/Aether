// tui_converter.cpp - Conversion implementations
#include "tui_converter.h"
#include "types.h"
#include <sstream>
#include <iomanip>

namespace tui {

Candle ConvertCandle(const types::MarketData& market_data) {
  Candle candle;
  candle.open = market_data.open;
  candle.high = market_data.high;
  candle.low = market_data.low;
  candle.close = market_data.close;
  candle.volume = market_data.volume;
  candle.entry = market_data.close;  // Use close as entry if not available
  candle.updated = market_data.timestamp;
  return candle;
}

Position ConvertPosition(const types::Position& pos, const std::string& name) {
  Position tui_pos;

  // Generate name from contract if not provided
  if (name.empty()) {
    std::ostringstream oss;
    oss << pos.contract.symbol << " "
        << types::option_type_to_string(pos.contract.type) << " "
        << pos.contract.strike << " " << pos.contract.expiry;
    tui_pos.name = oss.str();
  } else {
    tui_pos.name = name;
  }

  tui_pos.quantity = pos.quantity;

  // Calculate ROI from P&L and cost basis
  double cost_basis = pos.get_cost_basis();
  if (cost_basis > 0) {
    tui_pos.roi = (pos.unrealized_pnl / cost_basis) * 100.0;
  }

  // These would need to come from additional data sources
  // For now, set defaults
  tui_pos.maker_count = 0;
  tui_pos.taker_count = 0;
  tui_pos.rebate_estimate = 0.0;

  // Greeks from market data if available (would need to be passed separately)
  tui_pos.vega = 0.0;
  tui_pos.theta = 0.0;
  tui_pos.fair_diff = 0.0;

  // Convert market data to candle if we have it
  // This would require market data to be passed separately
  tui_pos.candle.updated = pos.last_update;

  return tui_pos;
}

Order ConvertOrderEvent(const types::Order& order) {
  Order tui_order;
  tui_order.timestamp = order.submitted_time;
  tui_order.text = FormatOrderText(order);

  // Map order status to severity
  switch (order.status) {
    case types::OrderStatus::Filled:
      tui_order.severity = "success";
      break;
    case types::OrderStatus::Rejected:
    case types::OrderStatus::Error:
      tui_order.severity = "error";
      break;
    case types::OrderStatus::PartiallyFilled:
      tui_order.severity = "warn";
      break;
    default:
      tui_order.severity = "info";
      break;
  }

  return tui_order;
}

AccountMetrics ConvertAccountMetrics(const types::AccountInfo& account) {
  AccountMetrics metrics;
  metrics.net_liq = account.net_liquidation;
  metrics.buying_power = account.buying_power;
  metrics.excess_liquidity = account.cash_balance;  // Approximate
  metrics.margin_requirement = account.maintenance_margin;
  metrics.commissions = 0.0;  // Would need separate tracking

  // Connection status would need to come from TWS client
  metrics.portal_ok = true;  // Would need actual status
  metrics.tws_ok = true;     // Would need actual status
  metrics.orats_ok = false;  // Would need actual status
  metrics.questdb_ok = false; // Would need actual status

  return metrics;
}

std::vector<Position> ConvertPositions(const std::vector<types::Position>& positions) {
  std::vector<Position> tui_positions;
  tui_positions.reserve(positions.size());

  for (const auto& pos : positions) {
    tui_positions.push_back(ConvertPosition(pos));
  }

  return tui_positions;
}

std::vector<Order> ConvertOrderEvents(const std::vector<types::Order>& orders) {
  std::vector<Order> tui_orders;
  tui_orders.reserve(orders.size());

  for (const auto& order : orders) {
    tui_orders.push_back(ConvertOrderEvent(order));
  }

  return tui_orders;
}

std::string FormatOrderText(const types::Order& order) {
  std::ostringstream oss;

  oss << order.contract.symbol << " ";
  oss << types::order_action_to_string(order.action) << " ";
  oss << order.quantity << " @ ";

  if (order.limit_price > 0) {
    oss << std::fixed << std::setprecision(2) << order.limit_price;
  } else {
    oss << "MARKET";
  }

  oss << " - " << types::order_status_to_string(order.status);

  if (order.filled_quantity > 0) {
    oss << " (" << order.filled_quantity << "/" << order.quantity << " filled)";
  }

  if (!order.status_message.empty()) {
    oss << " - " << order.status_message;
  }

  return oss.str();
}

} // namespace tui
