#pragma once

#include "tws_client.h"
#include "margin_calculator.h"

#include <future>
#include <mutex>
#include <string>
#include <vector>

class EClientSocket;

namespace tws {

class ConnectionHandler;

class PositionHandler
{
public:
  explicit PositionHandler(EClientSocket& client);

  void request_positions(PositionCallback callback, ConnectionHandler& conn);
  std::vector<types::Position> request_positions_sync(int timeout_ms,
                                                      ConnectionHandler& conn);
  [[nodiscard]] std::vector<types::Position> get_positions() const;
  [[nodiscard]] std::optional<types::Position> get_position(
      const types::OptionContract& contract) const;

  void request_account_updates(AccountCallback callback,
                               ConnectionHandler& conn);
  std::optional<types::AccountInfo> request_account_info_sync(
      int timeout_ms, ConnectionHandler& conn);
  [[nodiscard]] std::optional<types::AccountInfo> get_account_info() const;

  std::optional<types::BoxSpreadLeg> query_box_spread_margin(
      types::BoxSpreadLeg spread, double underlying_price,
      double implied_volatility);
  [[nodiscard]] double get_margin_utilization() const;
  [[nodiscard]] bool is_margin_call_risk(double buffer_percent) const;
  [[nodiscard]] double get_remaining_margin_capacity() const;

  void seed_mock_state();
  void update_mock_positions(const types::OptionContract& contract,
                             types::OrderAction action,
                             int quantity, double fill_price);

  // EWrapper callback forwarding
  void on_position(const std::string& account, const std::string& symbol,
                   const std::string& expiry, double strike,
                   const std::string& right, double position,
                   double avg_cost);
  void on_position_end();
  void on_update_account_value(const std::string& key, const std::string& val,
                               const std::string& currency,
                               const std::string& account_name);
  void on_update_account_time(const std::string& timestamp);
  void on_account_download_end(const std::string& account_name);
  void on_update_portfolio(const std::string& symbol, const std::string& expiry,
                           double strike, double position,
                           double market_price, double market_value,
                           double average_cost, double unrealized_pnl,
                           double realized_pnl);

private:
  EClientSocket& client_;

  mutable std::mutex position_mutex_;
  std::vector<types::Position> positions_;
  PositionCallback position_callback_;
  std::shared_ptr<std::promise<std::vector<types::Position>>> positions_promise_;
  std::atomic<bool> positions_request_pending_{false};

  mutable std::mutex account_mutex_;
  types::AccountInfo account_info_;
  AccountCallback account_callback_;
  std::shared_ptr<std::promise<types::AccountInfo>> account_promise_;
  std::atomic<bool> account_request_pending_{false};
};

} // namespace tws
