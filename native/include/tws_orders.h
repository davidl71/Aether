#pragma once

#include "tws_client.h"
#include "connection_utils.h"

#include <map>
#include <mutex>
#include <string>
#include <vector>

class EClientSocket;

namespace tws {

class ConnectionHandler;

class OrderHandler
{
public:
  explicit OrderHandler(EClientSocket& client);

  int place_order(const types::OptionContract& contract,
                  types::OrderAction action, int quantity,
                  double limit_price, types::TimeInForce tif,
                  ConnectionHandler& conn);
  int place_combo_order(const std::vector<types::OptionContract>& contracts,
                        const std::vector<types::OrderAction>& actions,
                        const std::vector<int>& quantities,
                        const std::vector<long>& contract_ids,
                        const std::vector<double>& limit_prices,
                        types::TimeInForce tif,
                        ConnectionHandler& conn);
  void cancel_order(int order_id, ConnectionHandler& conn);
  void cancel_all_orders(ConnectionHandler& conn);
  [[nodiscard]] std::optional<types::Order> get_order(int order_id) const;
  [[nodiscard]] std::vector<types::Order> get_active_orders() const;

  void set_order_status_callback(OrderStatusCallback callback);

  // EWrapper callback forwarding
  void on_order_status(int order_id, const std::string& status,
                       double filled, double remaining,
                       double avg_fill_price);
  void on_open_order(int order_id, const std::string& status);
  void on_open_order_end();
  void on_exec_details(int req_id, int order_id, double shares,
                       double price);
  void on_exec_details_end(int req_id);

private:
  EClientSocket& client_;
  mutable std::mutex mutex_;
  std::map<int, types::Order> orders_;
  OrderStatusCallback order_status_callback_;
};

} // namespace tws
