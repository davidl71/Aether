// tws_positions.cpp - Position tracking, account data, and margin operations
#include "tws_positions.h"
#include "tws_connection.h"
#include "connection_utils.h"
#include <spdlog/spdlog.h>

#include "EClientSocket.h"

#include <algorithm>
#include <cmath>

namespace tws {

PositionHandler::PositionHandler(EClientSocket& client)
    : client_(client) {}

// ============================================================================
// Position operations
// ============================================================================

void PositionHandler::request_positions(PositionCallback callback,
                                        ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    std::lock_guard<std::mutex> lock(position_mutex_);
    if (callback)
    {
      for (const auto& pos : positions_) callback(pos);
    }
    return;
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded: Cannot request positions");
    return;
  }

  position_callback_ = callback;
  conn.record_rate_message();
  client_.reqPositions();
}

std::vector<types::Position> PositionHandler::request_positions_sync(
    int timeout_ms, ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    std::lock_guard<std::mutex> lock(position_mutex_);
    return positions_;
  }

  auto promise = std::make_shared<std::promise<std::vector<types::Position>>>();
  auto future = promise->get_future();

  {
    std::lock_guard<std::mutex> lock(position_mutex_);
    positions_promise_ = promise;
    positions_request_pending_ = true;
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded: Cannot request positions synchronously");
    return {};
  }

  conn.record_rate_message();
  client_.reqPositions();

  if (future.wait_for(std::chrono::milliseconds(timeout_ms)) == std::future_status::ready)
  {
    return future.get();
  }

  spdlog::warn("Positions request timed out after {}ms", timeout_ms);
  {
    std::lock_guard<std::mutex> lock(position_mutex_);
    positions_promise_.reset();
    positions_request_pending_ = false;
  }
  return get_positions();
}

std::vector<types::Position> PositionHandler::get_positions() const
{
  std::lock_guard<std::mutex> lock(position_mutex_);
  return positions_;
}

std::optional<types::Position> PositionHandler::get_position(
    const types::OptionContract& contract) const
{
  std::lock_guard<std::mutex> lock(position_mutex_);
  auto it = std::find_if(
      positions_.begin(), positions_.end(),
      [&contract](const types::Position& pos) {
        return pos.contract.symbol == contract.symbol &&
               pos.contract.expiry == contract.expiry &&
               std::abs(pos.contract.strike - contract.strike) < 0.01 &&
               pos.contract.type == contract.type;
      });
  if (it != positions_.end()) return *it;
  return std::nullopt;
}

// ============================================================================
// Account operations
// ============================================================================

void PositionHandler::request_account_updates(AccountCallback callback,
                                              ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    std::lock_guard<std::mutex> lock(account_mutex_);
    account_callback_ = callback;
    if (account_callback_) account_callback_(account_info_);
    return;
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded: Cannot request account updates");
    return;
  }

  account_callback_ = callback;
  conn.record_rate_message();
  client_.reqAccountUpdates(true, "");
}

std::optional<types::AccountInfo> PositionHandler::request_account_info_sync(
    int timeout_ms, ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    std::lock_guard<std::mutex> lock(account_mutex_);
    return account_info_;
  }

  auto promise = std::make_shared<std::promise<types::AccountInfo>>();
  auto future = promise->get_future();

  {
    std::lock_guard<std::mutex> lock(account_mutex_);
    account_promise_ = promise;
    account_request_pending_ = true;
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded: Cannot request account info synchronously");
    return std::nullopt;
  }

  conn.record_rate_message();
  client_.reqAccountUpdates(true, "");

  auto cleanup = [this]() {
    std::lock_guard<std::mutex> lock(account_mutex_);
    account_promise_.reset();
    account_request_pending_ = false;
  };

  if (future.wait_for(std::chrono::milliseconds(timeout_ms)) == std::future_status::ready)
  {
    auto info = future.get();
    cleanup();
    if (!info.account_id.empty()) return info;
  }
  else
  {
    spdlog::warn("Account info request timed out after {}ms", timeout_ms);
    cleanup();
  }

  return get_account_info();
}

std::optional<types::AccountInfo> PositionHandler::get_account_info() const
{
  std::lock_guard<std::mutex> lock(account_mutex_);
  if (account_info_.account_id.empty()) return std::nullopt;
  return account_info_;
}

// ============================================================================
// Margin operations
// ============================================================================

std::optional<types::BoxSpreadLeg> PositionHandler::query_box_spread_margin(
    types::BoxSpreadLeg spread, double underlying_price,
    double implied_volatility)
{
  static margin::MarginCalculator margin_calc;

  margin::MarginResult result = margin_calc.calculate_margin(
      spread, underlying_price, implied_volatility, true);

  spread.initial_margin = result.initial_margin;
  spread.maintenance_margin = result.maintenance_margin;
  spread.portfolio_margin_benefit = result.portfolio_margin_benefit;
  spread.reg_t_margin = result.reg_t_margin;
  spread.span_margin = result.span_margin;
  spread.uses_portfolio_margin = result.uses_portfolio_margin;
  spread.margin_timestamp = result.calculated_at;

  spdlog::debug("Box spread margin: initial=${:.2f}, maintenance=${:.2f}",
                spread.initial_margin, spread.maintenance_margin);
  return spread;
}

double PositionHandler::get_margin_utilization() const
{
  auto info = get_account_info();
  if (!info || info->initial_margin <= 0.0) return 0.0;
  double total = info->net_liquidation;
  if (total <= 0.0) return 100.0;
  return (info->initial_margin / total) * 100.0;
}

bool PositionHandler::is_margin_call_risk(double buffer_percent) const
{
  auto info = get_account_info();
  if (!info || info->maintenance_margin <= 0.0) return false;

  double margin_used = info->initial_margin;
  double available = info->net_liquidation - margin_used;
  double threshold = info->maintenance_margin * (1.0 + buffer_percent / 100.0);
  return margin_used >= threshold || available < info->maintenance_margin;
}

double PositionHandler::get_remaining_margin_capacity() const
{
  auto info = get_account_info();
  if (!info) return 0.0;
  return std::max(0.0, info->net_liquidation - info->maintenance_margin);
}

// ============================================================================
// Mock state helpers
// ============================================================================

void PositionHandler::seed_mock_state()
{
  {
    std::lock_guard<std::mutex> lock(account_mutex_);
    account_info_.account_id = "DU123456";
    account_info_.net_liquidation = 100000.0;
    account_info_.cash_balance = 50000.0;
    account_info_.buying_power = 200000.0;
    account_info_.maintenance_margin = 25000.0;
    account_info_.initial_margin = 30000.0;
    account_info_.unrealized_pnl = 0.0;
    account_info_.realized_pnl = 0.0;
    account_info_.day_trades_remaining = 3;
    account_info_.gross_position_value = 50000.0;
    account_info_.last_update = std::chrono::system_clock::now();
    account_info_.timestamp = account_info_.last_update;
  }

  {
    std::lock_guard<std::mutex> lock(position_mutex_);
    positions_.clear();
    types::Position pos;
    pos.contract = make_mock_contract("SPY", "20251219", 500.0, types::OptionType::Call);
    pos.quantity = 1;
    pos.avg_price = 2.50;
    pos.current_price = 2.60;
    pos.unrealized_pnl = 10.0;
    pos.entry_time = std::chrono::system_clock::now() - std::chrono::hours(24);
    pos.last_update = std::chrono::system_clock::now();
    positions_.push_back(pos);
  }
}

void PositionHandler::update_mock_positions(const types::OptionContract& contract,
                                            types::OrderAction action,
                                            int quantity, double fill_price)
{
  auto now = std::chrono::system_clock::now();
  int signed_qty = action == types::OrderAction::Buy ? quantity : -quantity;

  {
    std::lock_guard<std::mutex> lock(position_mutex_);
    auto it = std::find_if(
        positions_.begin(), positions_.end(),
        [&contract](const types::Position& pos) {
          return pos.contract.symbol == contract.symbol &&
                 pos.contract.expiry == contract.expiry &&
                 pos.contract.strike == contract.strike &&
                 pos.contract.type == contract.type;
        });

    if (it == positions_.end())
    {
      types::Position pos;
      pos.contract = contract;
      pos.quantity = signed_qty;
      pos.avg_price = fill_price;
      pos.current_price = fill_price;
      pos.unrealized_pnl = 0.0;
      pos.entry_time = now;
      pos.last_update = now;
      positions_.push_back(pos);
    }
    else
    {
      it->quantity += signed_qty;
      if (it->quantity == 0)
        positions_.erase(it);
      else
      {
        it->avg_price = fill_price;
        it->current_price = fill_price;
        it->last_update = now;
      }
    }
  }

  {
    std::lock_guard<std::mutex> lock(account_mutex_);
    double trade_value = fill_price * quantity;
    if (action == types::OrderAction::Buy)
      account_info_.cash_balance -= trade_value;
    else
      account_info_.cash_balance += trade_value;
    account_info_.last_update = now;
    account_info_.timestamp = now;
  }
}

// ============================================================================
// EWrapper callback forwarding
// ============================================================================

void PositionHandler::on_position(const std::string& account,
                                  const std::string& symbol,
                                  const std::string& expiry,
                                  double strike, const std::string& right,
                                  double position, double avg_cost)
{
  try
  {
    spdlog::debug("Position: {} {} @ {} (account={})", position, symbol, avg_cost, account);

    std::lock_guard<std::mutex> lock(position_mutex_);

    types::Position pos;
    pos.contract.symbol = symbol;
    pos.contract.expiry = expiry;
    pos.contract.strike = strike;
    pos.contract.type = (right == "C") ? types::OptionType::Call : types::OptionType::Put;
    pos.quantity = static_cast<int>(position);
    pos.avg_price = avg_cost;
    pos.current_price = avg_cost;

    auto existing = std::find_if(
        positions_.begin(), positions_.end(),
        [&pos](const types::Position& candidate) {
          return candidate.contract.symbol == pos.contract.symbol &&
                 candidate.contract.expiry == pos.contract.expiry &&
                 candidate.contract.strike == pos.contract.strike &&
                 candidate.contract.type == pos.contract.type;
        });

    if (existing != positions_.end())
      *existing = pos;
    else if (position != 0)
      positions_.push_back(pos);
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in position: {}", e.what());
  }
}

void PositionHandler::on_position_end()
{
  try
  {
    spdlog::debug("Position updates complete");
    if (positions_request_pending_.load())
    {
      std::lock_guard<std::mutex> lock(position_mutex_);
      if (positions_promise_)
      {
        positions_promise_->set_value(positions_);
        positions_promise_.reset();
        positions_request_pending_ = false;
      }
    }
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in positionEnd: {}", e.what());
  }
}

void PositionHandler::on_update_account_value(const std::string& key,
                                              const std::string& val,
                                              const std::string& currency,
                                              const std::string& account_name)
{
  try
  {
    spdlog::trace("Account update: {}={} ({}, {})", key, val, currency, account_name);
    std::lock_guard<std::mutex> lock(account_mutex_);

    try
    {
      if (key == "NetLiquidation" && currency == "USD")
        account_info_.net_liquidation = std::stod(val);
      else if (key == "TotalCashBalance" && currency == "USD")
        account_info_.cash_balance = std::stod(val);
      else if (key == "BuyingPower")
        account_info_.buying_power = std::stod(val);
      else if (key == "GrossPositionValue" && currency == "USD")
        account_info_.gross_position_value = std::stod(val);
      else if (key == "UnrealizedPnL" && currency == "USD")
        account_info_.unrealized_pnl = std::stod(val);
      else if (key == "RealizedPnL" && currency == "USD")
        account_info_.realized_pnl = std::stod(val);
    }
    catch (const std::exception& e)
    {
      spdlog::warn("Failed to parse account value: {}={} ({})", key, val, e.what());
    }

    account_info_.account_id = account_name;
    account_info_.timestamp = std::chrono::system_clock::now();
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in updateAccountValue: {}", e.what());
  }
}

void PositionHandler::on_update_account_time(const std::string& timestamp)
{
  spdlog::trace("Account time: {}", timestamp);
}

void PositionHandler::on_account_download_end(const std::string& account_name)
{
  spdlog::debug("Account download complete: {}", account_name);
  if (account_request_pending_.load())
  {
    std::lock_guard<std::mutex> lock(account_mutex_);
    if (account_promise_)
    {
      account_promise_->set_value(account_info_);
      account_promise_.reset();
      account_request_pending_ = false;
    }
  }
}

void PositionHandler::on_update_portfolio(const std::string& symbol,
                                          const std::string& expiry,
                                          double strike, double position,
                                          double market_price, double /*market_value*/,
                                          double average_cost,
                                          double /*unrealized_pnl*/,
                                          double /*realized_pnl*/)
{
  try
  {
    std::lock_guard<std::mutex> lock(position_mutex_);
    auto it = std::find_if(
        positions_.begin(), positions_.end(),
        [&](const types::Position& candidate) {
          return candidate.contract.symbol == symbol &&
                 candidate.contract.expiry == expiry &&
                 std::abs(candidate.contract.strike - strike) < 0.01;
        });

    if (it != positions_.end())
    {
      it->current_price = market_price;
      it->quantity = static_cast<int>(position);
      it->avg_price = average_cost;
    }
    else if (position != 0)
    {
      types::Position pos;
      pos.contract.symbol = symbol;
      pos.contract.expiry = expiry;
      pos.contract.strike = strike;
      pos.quantity = static_cast<int>(position);
      pos.avg_price = average_cost;
      pos.current_price = market_price;
      positions_.push_back(pos);
    }
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in updatePortfolio: {}", e.what());
  }
}

} // namespace tws
