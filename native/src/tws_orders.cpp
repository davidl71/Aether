// tws_orders.cpp - Order placement, cancellation, and status tracking
#include "tws_orders.h"
#include "tws_connection.h"
#include "tws_conversions.h"
#include "connection_utils.h"
#include <spdlog/spdlog.h>

#include "EClientSocket.h"
#include "Contract.h"
#include "Order.h"
#include "OrderCancel.h"

#include <thread>

namespace tws {

namespace {

Order create_tws_order(types::OrderAction action, int quantity,
                       double limit_price, types::TimeInForce tif)
{
  Order o;
  o.action = (action == types::OrderAction::Buy) ? "BUY" : "SELL";
  o.totalQuantity = quantity;
  o.orderType = (limit_price > 0) ? "LMT" : "MKT";
  if (limit_price > 0) o.lmtPrice = limit_price;

  switch (tif)
  {
    case types::TimeInForce::Day: o.tif = "DAY"; break;
    case types::TimeInForce::GTC: o.tif = "GTC"; break;
    case types::TimeInForce::IOC: o.tif = "IOC"; break;
    case types::TimeInForce::FOK: o.tif = "FOK"; break;
  }
  return o;
}

} // anonymous namespace

OrderHandler::OrderHandler(EClientSocket& client)
    : client_(client) {}

// ============================================================================
// Public interface
// ============================================================================

int OrderHandler::place_order(const types::OptionContract& contract,
                              types::OrderAction action, int quantity,
                              double limit_price, types::TimeInForce tif,
                              ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    int order_id = conn.claim_order_id();
    types::Order mock_order;
    mock_order.order_id = order_id;
    mock_order.contract = contract;
    mock_order.action = action;
    mock_order.quantity = quantity;
    mock_order.limit_price = limit_price;
    mock_order.tif = tif;
    mock_order.status = types::OrderStatus::Filled;
    mock_order.submitted_time = std::chrono::system_clock::now();
    mock_order.last_update = mock_order.submitted_time;
    mock_order.filled_quantity = quantity;
    mock_order.avg_fill_price = limit_price > 0
        ? limit_price
        : generate_mock_market_data(contract).get_mid_price();
    mock_order.status_message = "Filled via mock TWS client";

    {
      std::lock_guard<std::mutex> lock(mutex_);
      orders_[order_id] = mock_order;
    }

    if (order_status_callback_) order_status_callback_(mock_order);
    return order_id;
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded: Cannot place order for {}", contract.to_string());
    return -1;
  }

  int order_id = conn.claim_order_id();
  Contract tws_contract = convert_to_tws_contract(contract);
  Order tws_order = create_tws_order(action, quantity, limit_price, tif);

  conn.record_rate_message();
  client_.placeOrder(order_id, tws_contract, tws_order);

  spdlog::info("Placed order #{}: {} {} {} @ {}",
               order_id, types::order_action_to_string(action),
               quantity, contract.to_string(),
               limit_price > 0 ? std::to_string(limit_price) : "MKT");

  {
    std::lock_guard<std::mutex> lock(mutex_);
    types::Order our_order;
    our_order.order_id = order_id;
    our_order.contract = contract;
    our_order.action = action;
    our_order.quantity = quantity;
    our_order.limit_price = limit_price;
    our_order.tif = tif;
    our_order.status = types::OrderStatus::Submitted;
    our_order.submitted_time = std::chrono::system_clock::now();
    orders_[order_id] = our_order;
  }

  return order_id;
}

int OrderHandler::place_combo_order(
    const std::vector<types::OptionContract>& contracts,
    const std::vector<types::OrderAction>& actions,
    const std::vector<int>& quantities,
    const std::vector<long>& contract_ids,
    const std::vector<double>& limit_prices,
    types::TimeInForce tif,
    ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    if (contracts.empty())
    {
      spdlog::error("Combo order: Cannot place empty combo");
      return -1;
    }
    int order_id = conn.claim_order_id();
    types::Order mock_order;
    mock_order.order_id = order_id;
    mock_order.contract = contracts.front();
    mock_order.action = actions.empty() ? types::OrderAction::Buy : actions.front();
    mock_order.quantity = 1;
    mock_order.limit_price = limit_prices.empty() ? 0.0 : limit_prices.front();
    mock_order.tif = tif;
    mock_order.status = types::OrderStatus::Filled;
    mock_order.submitted_time = std::chrono::system_clock::now();
    mock_order.last_update = mock_order.submitted_time;
    mock_order.filled_quantity = 1;
    mock_order.avg_fill_price = mock_order.limit_price;
    mock_order.status_message = "Mock combo order filled";

    {
      std::lock_guard<std::mutex> lock(mutex_);
      orders_[order_id] = mock_order;
    }

    if (order_status_callback_) order_status_callback_(mock_order);
    return order_id;
  }

  if (contracts.size() != actions.size() || contracts.size() != quantities.size() ||
      contracts.size() != contract_ids.size() || contracts.size() != limit_prices.size())
  {
    spdlog::error("Combo order: Mismatched vector sizes");
    return -1;
  }

  if (contracts.empty())
  {
    spdlog::error("Combo order: Cannot place empty combo");
    return -1;
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded: Cannot place combo order");
    return -1;
  }

  int order_id = conn.claim_order_id();

  Contract combo_contract;
  combo_contract.secType = "BAG";
  combo_contract.symbol = contracts[0].symbol;
  combo_contract.currency = "USD";
  combo_contract.exchange = "SMART";

  combo_contract.comboLegs = std::make_shared<Contract::ComboLegList>();
  for (size_t i = 0; i < contracts.size(); ++i)
  {
    auto leg = std::make_shared<ComboLeg>();
    leg->conId = contract_ids[i];
    leg->ratio = quantities[i];
    leg->action = (actions[i] == types::OrderAction::Buy) ? "BUY" : "SELL";
    leg->exchange = contracts[i].exchange.empty() ? "SMART" : contracts[i].exchange;
    leg->openClose = 0;
    leg->shortSaleSlot = 0;
    leg->exemptCode = -1;
    combo_contract.comboLegs->push_back(leg);
  }

  Order combo_order = create_tws_order(actions[0], 1, 0.0, tif);
  combo_order.orderType = "LMT";
  combo_order.allOrNone = true;

  combo_order.orderComboLegs = std::make_shared<Order::OrderComboLegList>();
  double total_limit = 0.0;
  for (size_t i = 0; i < limit_prices.size(); ++i)
  {
    auto order_leg = std::make_shared<OrderComboLeg>();
    order_leg->price = limit_prices[i];
    combo_order.orderComboLegs->push_back(order_leg);

    if (actions[i] == types::OrderAction::Buy)
      total_limit += limit_prices[i] * quantities[i];
    else
      total_limit -= limit_prices[i] * quantities[i];
  }
  combo_order.lmtPrice = total_limit;

  conn.record_rate_message();
  client_.placeOrder(order_id, combo_contract, combo_order);

  spdlog::info("Placed combo order #{}: {} legs, total limit: {:.2f}",
               order_id, contracts.size(), total_limit);

  {
    std::lock_guard<std::mutex> lock(mutex_);
    types::Order our_order;
    our_order.order_id = order_id;
    our_order.contract = contracts[0];
    our_order.action = actions[0];
    our_order.quantity = 1;
    our_order.limit_price = total_limit;
    our_order.tif = tif;
    our_order.status = types::OrderStatus::Submitted;
    our_order.submitted_time = std::chrono::system_clock::now();
    orders_[order_id] = our_order;
  }

  return order_id;
}

void OrderHandler::cancel_order(int order_id, ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    std::lock_guard<std::mutex> lock(mutex_);
    auto it = orders_.find(order_id);
    if (it != orders_.end())
    {
      it->second.status = types::OrderStatus::Cancelled;
      it->second.last_update = std::chrono::system_clock::now();
      it->second.status_message = "Cancelled via mock TWS client";
    }
    return;
  }

  OrderCancel orderCancel;
  client_.cancelOrder(order_id, orderCancel);
  conn.record_rate_message();

  spdlog::info("Cancelling order #{}", order_id);

  std::lock_guard<std::mutex> lock(mutex_);
  if (orders_.count(order_id))
  {
    orders_[order_id].status = types::OrderStatus::Cancelled;
    orders_[order_id].last_update = std::chrono::system_clock::now();
  }
}

void OrderHandler::cancel_all_orders(ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    std::lock_guard<std::mutex> lock(mutex_);
    for (auto& [id, order] : orders_)
    {
      order.status = types::OrderStatus::Cancelled;
      order.last_update = std::chrono::system_clock::now();
      order.status_message = "Cancelled via mock TWS client";
    }
    return;
  }

  spdlog::info("Cancelling all orders");
  OrderCancel orderCancel;
  client_.reqGlobalCancel(orderCancel);

  std::lock_guard<std::mutex> lock(mutex_);
  for (auto& [id, order] : orders_)
  {
    if (order.is_active())
    {
      order.status = types::OrderStatus::Cancelled;
      order.last_update = std::chrono::system_clock::now();
    }
  }
}

std::optional<types::Order> OrderHandler::get_order(int order_id) const
{
  std::lock_guard<std::mutex> lock(mutex_);
  auto it = orders_.find(order_id);
  if (it != orders_.end()) return it->second;
  return std::nullopt;
}

std::vector<types::Order> OrderHandler::get_active_orders() const
{
  std::lock_guard<std::mutex> lock(mutex_);
  std::vector<types::Order> active;
  for (const auto& [id, order] : orders_)
  {
    if (order.is_active()) active.push_back(order);
  }
  return active;
}

void OrderHandler::set_order_status_callback(OrderStatusCallback callback)
{
  order_status_callback_ = callback;
}

// ============================================================================
// EWrapper callback forwarding
// ============================================================================

void OrderHandler::on_order_status(int order_id, const std::string& status,
                                   double filled, double remaining,
                                   double avg_fill_price)
{
  try
  {
    spdlog::info("Order #{} status: {}, filled={}, remaining={}, avgPrice={}",
                 order_id, status, filled, remaining, avg_fill_price);

    std::lock_guard<std::mutex> lock(mutex_);
    if (orders_.count(order_id))
    {
      auto& order = orders_[order_id];

      if (status == "PreSubmitted" || status == "Submitted")
        order.status = types::OrderStatus::Submitted;
      else if (status == "Filled")
        order.status = types::OrderStatus::Filled;
      else if (status == "Cancelled")
        order.status = types::OrderStatus::Cancelled;
      else if (status == "Inactive" || status == "ApiCancelled")
        order.status = types::OrderStatus::Rejected;
      else if (filled > 0 && remaining > 0)
        order.status = types::OrderStatus::PartiallyFilled;

      order.filled_quantity = static_cast<int>(filled);
      order.avg_fill_price = avg_fill_price;
      order.last_update = std::chrono::system_clock::now();

      if (order_status_callback_) order_status_callback_(order);
    }
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in orderStatus(orderId={}): {}", order_id, e.what());
  }
}

void OrderHandler::on_open_order(int order_id, const std::string& status)
{
  try
  {
    std::lock_guard<std::mutex> lock(mutex_);
    if (orders_.count(order_id))
    {
      auto& order = orders_[order_id];
      if (status == "PreSubmitted" || status == "Submitted")
        order.status = types::OrderStatus::Submitted;
      else if (status == "Filled")
        order.status = types::OrderStatus::Filled;
      else if (status == "Cancelled")
        order.status = types::OrderStatus::Cancelled;
      else if (status == "PartiallyFilled")
        order.status = types::OrderStatus::PartiallyFilled;

      order.last_update = std::chrono::system_clock::now();
      if (order_status_callback_) order_status_callback_(order);
    }
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in openOrder(orderId={}): {}", order_id, e.what());
  }
}

void OrderHandler::on_open_order_end()
{
  spdlog::debug("Open orders sync complete");
}

void OrderHandler::on_exec_details(int /*req_id*/, int order_id,
                                   double shares, double price)
{
  try
  {
    spdlog::info("Execution: order={}, shares={}, price={}", order_id, shares, price);

    std::lock_guard<std::mutex> lock(mutex_);
    if (orders_.count(order_id))
    {
      auto& order = orders_[order_id];
      order.filled_quantity += static_cast<int>(shares);
      order.avg_fill_price = price;
      order.last_update = std::chrono::system_clock::now();
    }
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in execDetails: {}", e.what());
  }
}

void OrderHandler::on_exec_details_end(int req_id)
{
  spdlog::debug("Execution details end for reqId={}", req_id);
}

} // namespace tws
