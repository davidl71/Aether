#pragma once

#include "tws_client.h"
#include "connection_utils.h"

#include <atomic>
#include <future>
#include <map>
#include <mutex>
#include <set>
#include <string>

class EClientSocket;

namespace platform {
class CacheClient;
}

namespace tws {

class ConnectionHandler;

class MarketDataHandler
{
public:
  explicit MarketDataHandler(EClientSocket& client);

  int request_market_data(const types::OptionContract& contract,
                          MarketDataCallback callback,
                          ConnectionHandler& conn);
  std::optional<types::MarketData> request_market_data_sync(
      const types::OptionContract& contract,
      int timeout_ms, ConnectionHandler& conn);
  void cancel_market_data(int request_id, ConnectionHandler& conn);

  std::vector<types::OptionContract> request_option_chain(
      const std::string& symbol, const std::string& expiry,
      ConnectionHandler& conn);

  /// Optional cache for market data (e.g. Memcached). If set and healthy, sync requests check cache first.
  void set_market_data_cache(platform::CacheClient* cache, int ttl_seconds = 60);

  // EWrapper callback forwarding
  void on_tick_price(int ticker_id, int field, double price);
  void on_tick_size(int ticker_id, int field, int64_t size);
  void on_tick_option_computation(int ticker_id, int field,
                                  double implied_vol, double delta,
                                  double gamma, double vega, double theta);
  void on_security_definition_optional_parameter(
      int req_id, const std::string& exchange,
      int underlying_con_id, const std::string& trading_class,
      const std::string& multiplier,
      const std::set<std::string>& expirations,
      const std::set<double>& strikes);
  void on_security_definition_optional_parameter_end(int req_id);

  [[nodiscard]] std::string get_symbol_for_ticker(int ticker_id) const;

private:
  EClientSocket& client_;
  mutable std::mutex mutex_;
  std::map<int, types::MarketData> market_data_;
  std::map<int, MarketDataCallback> callbacks_;
  std::map<int, std::shared_ptr<std::promise<types::MarketData>>> promises_;
  std::map<int, std::string> ticker_to_symbol_;

  mutable std::mutex option_chain_mutex_;
  std::map<int, std::set<std::string>> option_chain_expirations_;
  std::map<int, std::set<double>> option_chain_strikes_;
  std::map<int, std::string> option_chain_symbols_;
  std::map<int, std::shared_ptr<std::promise<std::vector<types::OptionContract>>>> option_chain_promises_;
  std::atomic<bool> option_chain_complete_{false};

  platform::CacheClient* market_data_cache_{nullptr};
  int market_data_cache_ttl_{60};
};

} // namespace tws
