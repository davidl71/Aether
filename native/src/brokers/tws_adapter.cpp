// tws_adapter.cpp - TWS adapter implementation
#include "brokers/tws_adapter.h"
#include <algorithm>
#include <spdlog/spdlog.h>

namespace brokers {

// ============================================================================
// Constructor / Destructor
// ============================================================================

TWSAdapter::TWSAdapter(const config::TWSConfig &config)
    : tws_client_(std::make_unique<tws::TWSClient>(config)) {
  spdlog::info("TWSAdapter created");
}

TWSAdapter::~TWSAdapter() {
  if (tws_client_ && tws_client_->is_connected()) {
    tws_client_->disconnect();
  }
}

// ============================================================================
// Type Conversion Helpers
// ============================================================================

types::OptionContract TWSAdapter::convert_contract(
    const box_spread::types::OptionContract &contract) {
  types::OptionContract result;
  result.symbol = contract.symbol;
  result.expiry = contract.expiry;
  result.strike = contract.strike;
  result.type = static_cast<types::OptionType>(static_cast<int>(contract.type));
  result.style =
      static_cast<types::OptionStyle>(static_cast<int>(contract.style));
  result.exchange = contract.exchange;
  result.local_symbol = contract.local_symbol;
  return result;
}

box_spread::types::OptionContract
TWSAdapter::convert_contract_back(const types::OptionContract &contract) {
  box_spread::types::OptionContract result;
  result.symbol = contract.symbol;
  result.expiry = contract.expiry;
  result.strike = contract.strike;
  result.type = static_cast<box_spread::types::OptionType>(
      static_cast<int>(contract.type));
  result.style = static_cast<box_spread::types::OptionStyle>(
      static_cast<int>(contract.style));
  result.exchange = contract.exchange;
  result.local_symbol = contract.local_symbol;
  return result;
}

types::OrderAction
TWSAdapter::convert_action(box_spread::types::OrderAction action) {
  return static_cast<types::OrderAction>(static_cast<int>(action));
}

box_spread::types::OrderAction
TWSAdapter::convert_action_back(types::OrderAction action) {
  return static_cast<box_spread::types::OrderAction>(static_cast<int>(action));
}

types::TimeInForce TWSAdapter::convert_tif(box_spread::types::TimeInForce tif) {
  return static_cast<types::TimeInForce>(static_cast<int>(tif));
}

box_spread::types::TimeInForce
TWSAdapter::convert_tif_back(types::TimeInForce tif) {
  return static_cast<box_spread::types::TimeInForce>(static_cast<int>(tif));
}

types::MarketData
TWSAdapter::convert_market_data(const box_spread::types::MarketData &data) {
  types::MarketData result;
  result.symbol = data.symbol;
  result.timestamp = data.timestamp;
  result.bid = data.bid;
  result.ask = data.ask;
  result.last = data.last;
  result.bid_size = data.bid_size;
  result.ask_size = data.ask_size;
  result.last_size = data.last_size;
  result.volume = data.volume;
  result.high = data.high;
  result.low = data.low;
  result.close = data.close;
  result.open = data.open;
  result.implied_volatility = data.implied_volatility;
  result.delta = data.delta;
  result.gamma = data.gamma;
  result.theta = data.theta;
  result.vega = data.vega;
  return result;
}

box_spread::types::MarketData
TWSAdapter::convert_market_data_back(const types::MarketData &data) {
  box_spread::types::MarketData result;
  result.symbol = data.symbol;
  result.timestamp = data.timestamp;
  result.bid = data.bid;
  result.ask = data.ask;
  result.last = data.last;
  result.bid_size = data.bid_size;
  result.ask_size = data.ask_size;
  result.last_size = data.last_size;
  result.volume = data.volume;
  result.high = data.high;
  result.low = data.low;
  result.close = data.close;
  result.open = data.open;
  result.implied_volatility = data.implied_volatility;
  result.delta = data.delta;
  result.gamma = data.gamma;
  result.theta = data.theta;
  result.vega = data.vega;
  return result;
}

types::Order TWSAdapter::convert_order(const box_spread::types::Order &order) {
  types::Order result;
  result.order_id = order.order_id;
  result.contract = convert_contract(order.contract);
  result.action = convert_action(order.action);
  result.quantity = order.quantity;
  result.limit_price = order.limit_price;
  result.tif = convert_tif(order.tif);
  result.status =
      static_cast<types::OrderStatus>(static_cast<int>(order.status));
  result.submitted_time = order.submitted_time;
  result.last_update = order.last_update;
  result.filled_quantity = order.filled_quantity;
  result.avg_fill_price = order.avg_fill_price;
  result.status_message = order.status_message;
  return result;
}

box_spread::types::Order
TWSAdapter::convert_order_back(const types::Order &order) {
  box_spread::types::Order result;
  result.order_id = order.order_id;
  result.contract = convert_contract_back(order.contract);
  result.action = convert_action_back(order.action);
  result.quantity = order.quantity;
  result.limit_price = order.limit_price;
  result.tif = convert_tif_back(order.tif);
  result.status = static_cast<box_spread::types::OrderStatus>(
      static_cast<int>(order.status));
  result.submitted_time = order.submitted_time;
  result.last_update = order.last_update;
  result.filled_quantity = order.filled_quantity;
  result.avg_fill_price = order.avg_fill_price;
  result.status_message = order.status_message;
  return result;
}

types::Position
TWSAdapter::convert_position(const box_spread::types::Position &position) {
  types::Position result;
  result.contract = convert_contract(position.contract);
  result.quantity = position.quantity;
  result.avg_price = position.avg_price;
  result.current_price = position.current_price;
  result.unrealized_pnl = position.unrealized_pnl;
  result.entry_time = position.entry_time;
  result.last_update = position.last_update;
  return result;
}

box_spread::types::Position
TWSAdapter::convert_position_back(const types::Position &position) {
  box_spread::types::Position result;
  result.contract = convert_contract_back(position.contract);
  result.quantity = position.quantity;
  result.avg_price = position.avg_price;
  result.current_price = position.current_price;
  result.unrealized_pnl = position.unrealized_pnl;
  result.entry_time = position.entry_time;
  result.last_update = position.last_update;
  return result;
}

types::AccountInfo
TWSAdapter::convert_account_info(const box_spread::types::AccountInfo &info) {
  types::AccountInfo result;
  result.account_id = info.account_id;
  result.net_liquidation = info.net_liquidation;
  result.cash_balance = info.cash_balance;
  result.buying_power = info.buying_power;
  result.maintenance_margin = info.maintenance_margin;
  result.initial_margin = info.initial_margin;
  result.unrealized_pnl = info.unrealized_pnl;
  result.realized_pnl = info.realized_pnl;
  result.day_trades_remaining = info.day_trades_remaining;
  result.gross_position_value = info.gross_position_value;
  result.last_update = info.last_update;
  result.timestamp = info.timestamp;
  return result;
}

box_spread::types::AccountInfo
TWSAdapter::convert_account_info_back(const types::AccountInfo &info) {
  box_spread::types::AccountInfo result;
  result.account_id = info.account_id;
  result.net_liquidation = info.net_liquidation;
  result.cash_balance = info.cash_balance;
  result.buying_power = info.buying_power;
  result.maintenance_margin = info.maintenance_margin;
  result.initial_margin = info.initial_margin;
  result.unrealized_pnl = info.unrealized_pnl;
  result.realized_pnl = info.realized_pnl;
  result.day_trades_remaining = info.day_trades_remaining;
  result.gross_position_value = info.gross_position_value;
  result.last_update = info.last_update;
  result.timestamp = info.timestamp;
  return result;
}

tws::ConnectionState TWSAdapter::convert_connection_state(
    box_spread::brokers::ConnectionState state) {
  return static_cast<tws::ConnectionState>(static_cast<int>(state));
}

box_spread::brokers::ConnectionState
TWSAdapter::convert_connection_state_back(tws::ConnectionState state) {
  return static_cast<box_spread::brokers::ConnectionState>(
      static_cast<int>(state));
}

// ============================================================================
// Connection Management
// ============================================================================

bool TWSAdapter::connect() {
  std::lock_guard<std::mutex> lock(mutex_);
  return tws_client_->connect();
}

void TWSAdapter::disconnect() {
  std::lock_guard<std::mutex> lock(mutex_);
  tws_client_->disconnect();
}

bool TWSAdapter::is_connected() const {
  std::lock_guard<std::mutex> lock(mutex_);
  return tws_client_->is_connected();
}

box_spread::brokers::ConnectionState TWSAdapter::get_connection_state() const {
  std::lock_guard<std::mutex> lock(mutex_);
  return convert_connection_state_back(tws_client_->get_connection_state());
}

box_spread::brokers::BrokerType TWSAdapter::get_broker_type() const {
  return box_spread::brokers::BrokerType::TWS;
}

box_spread::brokers::BrokerCapabilities TWSAdapter::get_capabilities() const {
  box_spread::brokers::BrokerCapabilities caps;
  caps.supports_options = true;
  caps.supports_multi_leg_orders = true;
  caps.supports_real_time_data = true;
  caps.supports_historical_data = true;
  caps.max_orders_per_second = 50;  // IBKR limit
  caps.rate_limit_per_minute = 100; // Conservative limit
  return caps;
}

// ============================================================================
// Market Data
// ============================================================================

int TWSAdapter::request_market_data(
    const box_spread::types::OptionContract &contract,
    std::function<void(const box_spread::types::MarketData &)> callback) {
  std::lock_guard<std::mutex> lock(mutex_);
  types::OptionContract native_contract = convert_contract(contract);

  return tws_client_->request_market_data(
      native_contract, [callback](const types::MarketData &data) {
        callback(convert_market_data_back(data));
      });
}

void TWSAdapter::cancel_market_data(int request_id) {
  std::lock_guard<std::mutex> lock(mutex_);
  tws_client_->cancel_market_data(request_id);
}

std::optional<box_spread::types::MarketData>
TWSAdapter::request_market_data_sync(
    const box_spread::types::OptionContract &contract, int timeout_ms) {
  std::lock_guard<std::mutex> lock(mutex_);
  types::OptionContract native_contract = convert_contract(contract);

  auto native_data =
      tws_client_->request_market_data_sync(native_contract, timeout_ms);
  if (native_data) {
    return convert_market_data_back(*native_data);
  }
  return std::nullopt;
}

// ============================================================================
// Options Chain
// ============================================================================

std::vector<box_spread::types::OptionContract>
TWSAdapter::request_option_chain(const std::string &symbol,
                                 const std::string &expiry) {
  std::lock_guard<std::mutex> lock(mutex_);
  auto native_chain = tws_client_->request_option_chain(symbol, expiry);

  std::vector<box_spread::types::OptionContract> result;
  result.reserve(native_chain.size());
  for (const auto &contract : native_chain) {
    result.push_back(convert_contract_back(contract));
  }
  return result;
}

// ============================================================================
// Contract Details
// ============================================================================

int TWSAdapter::request_contract_details(
    const box_spread::types::OptionContract &contract,
    std::function<void(long conId)> callback) {
  std::lock_guard<std::mutex> lock(mutex_);
  types::OptionContract native_contract = convert_contract(contract);
  return tws_client_->request_contract_details(native_contract, callback);
}

long TWSAdapter::request_contract_details_sync(
    const box_spread::types::OptionContract &contract, int timeout_ms) {
  std::lock_guard<std::mutex> lock(mutex_);
  types::OptionContract native_contract = convert_contract(contract);
  return tws_client_->request_contract_details_sync(native_contract,
                                                    timeout_ms);
}

// ============================================================================
// Order Management
// ============================================================================

int TWSAdapter::place_order(const box_spread::types::OptionContract &contract,
                            box_spread::types::OrderAction action, int quantity,
                            double limit_price,
                            box_spread::types::TimeInForce tif) {
  std::lock_guard<std::mutex> lock(mutex_);
  types::OptionContract native_contract = convert_contract(contract);
  return tws_client_->place_order(native_contract, convert_action(action),
                                  quantity, limit_price, convert_tif(tif));
}

bool TWSAdapter::cancel_order(int order_id) {
  std::lock_guard<std::mutex> lock(mutex_);
  try {
    tws_client_->cancel_order(order_id);
    return true;
  } catch (...) {
    return false;
  }
}

std::optional<box_spread::types::Order>
TWSAdapter::get_order_status(int order_id) const {
  std::lock_guard<std::mutex> lock(mutex_);
  auto native_order = tws_client_->get_order(order_id);
  if (native_order) {
    return convert_order_back(*native_order);
  }
  return std::nullopt;
}

// ============================================================================
// Multi-Leg Orders (Box Spreads)
// ============================================================================

int TWSAdapter::place_combo_order(
    const std::vector<box_spread::types::OptionContract> &contracts,
    const std::vector<box_spread::types::OrderAction> &actions,
    const std::vector<int> &quantities, const std::vector<long> &contract_ids,
    const std::vector<double> &limit_prices) {
  std::lock_guard<std::mutex> lock(mutex_);

  // Convert contracts and actions
  std::vector<types::OptionContract> native_contracts;
  native_contracts.reserve(contracts.size());
  for (const auto &contract : contracts) {
    native_contracts.push_back(convert_contract(contract));
  }

  std::vector<types::OrderAction> native_actions;
  native_actions.reserve(actions.size());
  for (const auto &action : actions) {
    native_actions.push_back(convert_action(action));
  }

  // Use Day as default TIF for combo orders
  return tws_client_->place_combo_order(native_contracts, native_actions,
                                        quantities, contract_ids, limit_prices,
                                        types::TimeInForce::Day);
}

// ============================================================================
// Positions
// ============================================================================

std::vector<box_spread::types::Position> TWSAdapter::get_positions() {
  std::lock_guard<std::mutex> lock(mutex_);
  auto native_positions = tws_client_->get_positions();

  std::vector<box_spread::types::Position> result;
  result.reserve(native_positions.size());
  for (const auto &position : native_positions) {
    result.push_back(convert_position_back(position));
  }
  return result;
}

std::optional<box_spread::types::Position>
TWSAdapter::get_position(const box_spread::types::OptionContract &contract) {
  std::lock_guard<std::mutex> lock(mutex_);
  types::OptionContract native_contract = convert_contract(contract);
  auto native_position = tws_client_->get_position(native_contract);
  if (native_position) {
    return convert_position_back(*native_position);
  }
  return std::nullopt;
}

// ============================================================================
// Account Information
// ============================================================================

std::optional<box_spread::types::AccountInfo> TWSAdapter::get_account_info() {
  std::lock_guard<std::mutex> lock(mutex_);
  auto native_info = tws_client_->get_account_info();
  if (native_info) {
    return convert_account_info_back(*native_info);
  }
  return std::nullopt;
}

double TWSAdapter::get_buying_power() {
  std::lock_guard<std::mutex> lock(mutex_);
  auto account_info = tws_client_->get_account_info();
  if (account_info) {
    return account_info->buying_power;
  }
  return 0.0;
}

double TWSAdapter::get_net_liquidation_value() {
  std::lock_guard<std::mutex> lock(mutex_);
  auto account_info = tws_client_->get_account_info();
  if (account_info) {
    return account_info->net_liquidation;
  }
  return 0.0;
}

// ============================================================================
// Error Handling
// ============================================================================

void TWSAdapter::set_error_callback(
    std::function<void(int code, const std::string &msg)> callback) {
  std::lock_guard<std::mutex> lock(mutex_);
  tws_client_->set_error_callback(callback);
}

// ============================================================================
// Additional Methods
// ============================================================================

void TWSAdapter::process_messages(int timeout_ms) {
  std::lock_guard<std::mutex> lock(mutex_);
  tws_client_->process_messages(timeout_ms);
}

} // namespace brokers
