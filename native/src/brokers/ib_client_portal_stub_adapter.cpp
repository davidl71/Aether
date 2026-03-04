// ib_client_portal_stub_adapter.cpp - IB Client Portal stub adapter
// implementation
#include "brokers/ib_client_portal_stub_adapter.h"
#include <algorithm>
#include <cmath>
#include <random>
#include <spdlog/spdlog.h>
#include <sstream>

namespace brokers {

// ============================================================================
// Constructor / Destructor
// ============================================================================

IBClientPortalStubAdapter::IBClientPortalStubAdapter(const Config &config)
    : config_(config),
      connection_state_(box_spread::brokers::ConnectionState::Disconnected),
      next_request_id_(1), next_order_id_(2000) {
  spdlog::info("IBClientPortalStubAdapter created (STUB MODE)");
}

IBClientPortalStubAdapter::~IBClientPortalStubAdapter() { disconnect(); }

// ============================================================================
// Connection Management
// ============================================================================

bool IBClientPortalStubAdapter::connect() {
  std::lock_guard<std::mutex> lock(mutex_);

  if (connected_.load()) {
    return true;
  }

  connection_state_ = box_spread::brokers::ConnectionState::Connecting;
  spdlog::info("IBClientPortalStubAdapter: Simulating connection...");

  // Simulate connection delay (OAuth/session setup)
  std::this_thread::sleep_for(std::chrono::milliseconds(150));

  connected_ = true;
  connection_state_ = box_spread::brokers::ConnectionState::Connected;
  spdlog::info("IBClientPortalStubAdapter: Connected (STUB MODE)");
  return true;
}

void IBClientPortalStubAdapter::disconnect() {
  std::lock_guard<std::mutex> lock(mutex_);
  connected_ = false;
  connection_state_ = box_spread::brokers::ConnectionState::Disconnected;
  subscriptions_.clear();
  callbacks_.clear();
  spdlog::info("IBClientPortalStubAdapter: Disconnected (STUB MODE)");
}

bool IBClientPortalStubAdapter::is_connected() const {
  return connected_.load();
}

box_spread::brokers::ConnectionState
IBClientPortalStubAdapter::get_connection_state() const {
  return connection_state_.load();
}

box_spread::brokers::BrokerType
IBClientPortalStubAdapter::get_broker_type() const {
  return box_spread::brokers::BrokerType::IB_CLIENT_PORTAL;
}

box_spread::brokers::BrokerCapabilities
IBClientPortalStubAdapter::get_capabilities() const {
  box_spread::brokers::BrokerCapabilities caps;
  caps.supports_options = true;
  caps.supports_multi_leg_orders = true;
  caps.supports_real_time_data = false;  // Stub doesn't provide real-time
  caps.supports_historical_data = false; // Stub doesn't provide historical
  caps.max_orders_per_second = 50;       // Match IBKR limit
  caps.rate_limit_per_minute = 100;      // Conservative stub limit
  return caps;
}

// ============================================================================
// Market Data
// ============================================================================

int IBClientPortalStubAdapter::request_market_data(
    const box_spread::types::OptionContract &contract,
    std::function<void(const box_spread::types::MarketData &)> callback) {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    spdlog::warn("IBClientPortalStubAdapter: Cannot request market data - not "
                 "connected");
    return -1;
  }

  int request_id = next_request_id_++;
  subscriptions_[request_id] = contract;
  callbacks_[request_id] = callback;

  // Immediately call callback with mock data (stub behavior)
  auto mock_data = generate_mock_market_data(contract);
  callback(mock_data);

  spdlog::debug(
      "IBClientPortalStubAdapter: Market data subscription {} created for {}",
      request_id, contract.symbol);
  return request_id;
}

void IBClientPortalStubAdapter::cancel_market_data(int request_id) {
  std::lock_guard<std::mutex> lock(mutex_);
  subscriptions_.erase(request_id);
  callbacks_.erase(request_id);
  spdlog::debug(
      "IBClientPortalStubAdapter: Market data subscription {} cancelled",
      request_id);
}

std::optional<box_spread::types::MarketData>
IBClientPortalStubAdapter::request_market_data_sync(
    const box_spread::types::OptionContract &contract, int timeout_ms) {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    return std::nullopt;
  }

  return generate_mock_market_data(contract);
}

// ============================================================================
// Options Chain
// ============================================================================

std::vector<box_spread::types::OptionContract>
IBClientPortalStubAdapter::request_option_chain(const std::string &symbol,
                                                const std::string &expiry) {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    return {};
  }

  // Generate mock option chain
  std::vector<box_spread::types::OptionContract> chain;

  // Generate strikes around current price (assume $500 for SPY)
  double base_price = 500.0;
  std::vector<double> strikes = {450.0, 460.0, 470.0, 480.0, 490.0, 500.0,
                                 510.0, 520.0, 530.0, 540.0, 550.0};

  // Use provided expiry or generate a default
  std::string expiry_date = expiry.empty() ? "20250117" : expiry;

  for (double strike : strikes) {
    // Call option
    box_spread::types::OptionContract call;
    call.symbol = symbol;
    call.expiry = expiry_date;
    call.strike = strike;
    call.type = box_spread::types::OptionType::Call;
    call.style = box_spread::types::OptionStyle::American;
    call.exchange = "SMART";
    chain.push_back(call);

    // Put option
    box_spread::types::OptionContract put;
    put.symbol = symbol;
    put.expiry = expiry_date;
    put.strike = strike;
    put.type = box_spread::types::OptionType::Put;
    put.style = box_spread::types::OptionStyle::American;
    put.exchange = "SMART";
    chain.push_back(put);
  }

  spdlog::debug("IBClientPortalStubAdapter: Generated mock option chain with "
                "{} contracts",
                chain.size());
  return chain;
}

// ============================================================================
// Contract Details
// ============================================================================

int IBClientPortalStubAdapter::request_contract_details(
    const box_spread::types::OptionContract &contract,
    std::function<void(long conId)> callback) {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    return -1;
  }

  long conid = generate_mock_contract_id(contract);

  // Call callback immediately (stub behavior)
  callback(conid);

  return next_request_id_++;
}

long IBClientPortalStubAdapter::request_contract_details_sync(
    const box_spread::types::OptionContract &contract, int timeout_ms) {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    return 0;
  }

  return generate_mock_contract_id(contract);
}

// ============================================================================
// Order Management
// ============================================================================

int IBClientPortalStubAdapter::place_order(
    const box_spread::types::OptionContract &contract,
    box_spread::types::OrderAction action, int quantity, double limit_price,
    box_spread::types::TimeInForce tif) {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    spdlog::warn(
        "IBClientPortalStubAdapter: Cannot place order - not connected");
    return -1;
  }

  int order_id = next_order_id_++;

  // Create mock order
  box_spread::types::Order order;
  order.order_id = order_id;
  order.contract = contract;
  order.action = action;
  order.quantity = quantity;
  order.limit_price = limit_price;
  order.tif = tif;
  order.status = box_spread::types::OrderStatus::Submitted;
  order.submitted_time = std::chrono::system_clock::now();
  order.last_update = std::chrono::system_clock::now();
  order.filled_quantity = 0;
  order.avg_fill_price = 0.0;
  order.status_message = "STUB: Order submitted";

  orders_[order_id] = order;

  spdlog::info("IBClientPortalStubAdapter: Mock order {} placed (STUB MODE) - "
               "{} {} {} @ {}",
               order_id,
               action == box_spread::types::OrderAction::Buy ? "BUY" : "SELL",
               quantity, contract.symbol,
               limit_price > 0.0 ? std::to_string(limit_price) : "MARKET");
  return order_id;
}

bool IBClientPortalStubAdapter::cancel_order(int order_id) {
  std::lock_guard<std::mutex> lock(mutex_);

  auto it = orders_.find(order_id);
  if (it != orders_.end()) {
    it->second.status = box_spread::types::OrderStatus::Cancelled;
    it->second.status_message = "STUB: Order cancelled";
    it->second.last_update = std::chrono::system_clock::now();
    spdlog::info(
        "IBClientPortalStubAdapter: Mock order {} cancelled (STUB MODE)",
        order_id);
    return true;
  }

  spdlog::warn("IBClientPortalStubAdapter: Order {} not found for cancellation",
               order_id);
  return false;
}

std::optional<box_spread::types::Order>
IBClientPortalStubAdapter::get_order_status(int order_id) const {
  std::lock_guard<std::mutex> lock(mutex_);

  auto it = orders_.find(order_id);
  if (it != orders_.end()) {
    return it->second;
  }

  return std::nullopt;
}

// ============================================================================
// Multi-Leg Orders (Box Spreads)
// ============================================================================

int IBClientPortalStubAdapter::place_combo_order(
    const std::vector<box_spread::types::OptionContract> &contracts,
    const std::vector<box_spread::types::OrderAction> &actions,
    const std::vector<int> &quantities, const std::vector<long> &contract_ids,
    const std::vector<double> &limit_prices) {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    spdlog::warn(
        "IBClientPortalStubAdapter: Cannot place combo order - not connected");
    return -1;
  }

  if (contracts.size() != actions.size() ||
      contracts.size() != quantities.size()) {
    spdlog::error("IBClientPortalStubAdapter: Invalid combo order - mismatched "
                  "leg counts");
    return -1;
  }

  int order_id = next_order_id_++;

  // Create mock combo order (use first contract for order details)
  box_spread::types::Order order;
  order.order_id = order_id;
  order.contract = contracts[0]; // Use first leg for order contract
  order.action = actions[0];
  order.quantity = quantities[0];
  order.limit_price = limit_prices.empty() ? 0.0 : limit_prices[0];
  order.tif = box_spread::types::TimeInForce::Day;
  order.status = box_spread::types::OrderStatus::Submitted;
  order.submitted_time = std::chrono::system_clock::now();
  order.last_update = std::chrono::system_clock::now();
  order.filled_quantity = 0;
  order.avg_fill_price = 0.0;
  order.status_message = "STUB: Combo order submitted";

  orders_[order_id] = order;

  spdlog::info("IBClientPortalStubAdapter: Mock combo order {} placed (STUB "
               "MODE) - {} legs",
               order_id, contracts.size());
  return order_id;
}

// ============================================================================
// Positions
// ============================================================================

std::vector<box_spread::types::Position>
IBClientPortalStubAdapter::get_positions() {
  std::lock_guard<std::mutex> lock(mutex_);

  // Stub returns empty positions (no mock positions by default)
  return {};
}

std::optional<box_spread::types::Position>
IBClientPortalStubAdapter::get_position(
    const box_spread::types::OptionContract &contract) {
  std::lock_guard<std::mutex> lock(mutex_);

  // Stub returns no position
  return std::nullopt;
}

// ============================================================================
// Account Information
// ============================================================================

std::optional<box_spread::types::AccountInfo>
IBClientPortalStubAdapter::get_account_info() {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    return std::nullopt;
  }

  return generate_mock_account_info();
}

double IBClientPortalStubAdapter::get_buying_power() {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    return 0.0;
  }

  auto account_info = generate_mock_account_info();
  return account_info.buying_power;
}

double IBClientPortalStubAdapter::get_net_liquidation_value() {
  std::lock_guard<std::mutex> lock(mutex_);

  if (!connected_.load()) {
    return 0.0;
  }

  auto account_info = generate_mock_account_info();
  return account_info.net_liquidation;
}

// ============================================================================
// Error Handling
// ============================================================================

void IBClientPortalStubAdapter::set_error_callback(
    std::function<void(int code, const std::string &msg)> callback) {
  std::lock_guard<std::mutex> lock(mutex_);
  error_callback_ = callback;
}

// ============================================================================
// Mock Data Generation Helpers
// ============================================================================

box_spread::types::MarketData
IBClientPortalStubAdapter::generate_mock_market_data(
    const box_spread::types::OptionContract &contract) const {
  box_spread::types::MarketData data;
  data.symbol = contract.symbol;
  data.timestamp = std::chrono::system_clock::now();

  // Generate realistic mock prices based on strike
  double base_price = contract.strike > 0 ? contract.strike / 10.0 : 100.0;

  // Add some variation based on symbol hash
  std::hash<std::string> hasher;
  size_t symbol_hash = hasher(contract.symbol);
  double variation =
      static_cast<double>(symbol_hash % 100) / 100.0 - 0.5; // -0.5 to 0.5
  base_price += variation * 10.0;

  data.bid = std::max(0.1, base_price - 0.1);
  data.ask = data.bid + 0.2;
  data.last = (data.bid + data.ask) / 2.0;
  data.bid_size = 10;
  data.ask_size = 10;
  data.last_size = 1;
  data.volume = 5000;
  data.high = data.last + 0.5;
  data.low = data.last - 0.5;
  data.open = data.last - 0.2;
  data.close = data.last - 0.1;

  // Option-specific Greeks
  data.implied_volatility = 0.25;
  data.delta =
      contract.type == box_spread::types::OptionType::Call ? 0.4 : -0.4;
  data.gamma = 0.02;
  data.theta = -0.01;
  data.vega = 0.15;

  return data;
}

box_spread::types::AccountInfo
IBClientPortalStubAdapter::generate_mock_account_info() const {
  box_spread::types::AccountInfo info;
  info.account_id = config_.account_id;
  info.net_liquidation = 100000.0; // $100k mock account
  info.cash_balance = 50000.0;     // $50k cash
  info.buying_power = 200000.0;    // $200k buying power (2x margin)
  info.maintenance_margin = 10000.0;
  info.initial_margin = 20000.0;
  info.unrealized_pnl = 0.0;
  info.realized_pnl = 0.0;
  info.day_trades_remaining = 3;
  info.gross_position_value = 50000.0;
  return info;
}

long IBClientPortalStubAdapter::generate_mock_contract_id(
    const box_spread::types::OptionContract &contract) const {
  // Generate deterministic contract ID from contract details
  std::string key =
      contract.symbol + contract.expiry +
      (contract.type == box_spread::types::OptionType::Call ? "C" : "P") +
      std::to_string(static_cast<int>(contract.strike * 1000));

  auto it = contract_id_cache_.find(key);
  if (it != contract_id_cache_.end()) {
    return it->second;
  }

  // Generate new ID (deterministic hash, different range from Alpaca)
  std::hash<std::string> hasher;
  long conid =
      20000000 +
      static_cast<long>(hasher(key) % 80000000); // 20M-100M range (IBKR format)
  contract_id_cache_[key] = conid;
  return conid;
}

} // namespace brokers
