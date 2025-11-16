// Mock provider adapter for deterministic tests and dev (tracked path).
#pragma once

#include <chrono>
#include <optional>
#include <string>
#include <string_view>
#include <unordered_map>
#include "agg_types_agg.h"
#include "provider_adapter_agg.h"

namespace ib_box_spread
{

class MockAdapter final : public ProviderAdapter
{
public:
  explicit MockAdapter(std::string id = "mock") : id_{std::move(id)} {}

  std::string provider_id() const override
  {
    return id_;
  }

  bool connect(std::string_view /*profile*/) override
  {
    connected_ = true;
    return true;
  }

  void disconnect() override
  {
    connected_ = false;
  }

  bool is_connected() const override
  {
    return connected_;
  }

  bool subscribe_quotes(std::string_view symbol, const QuoteCallback& on_quote) override
  {
    subscribers_[std::string(symbol)] = on_quote;
    return true;
  }

  void unsubscribe_quotes(std::string_view symbol) override
  {
    subscribers_.erase(std::string(symbol));
  }

  std::optional<ErrorEnvelope> place_order(const OrderIntent& /*order*/) override
  {
    return std::nullopt;
  }

  std::optional<ErrorEnvelope> cancel_order(std::string_view /*client_order_id*/) override
  {
    return std::nullopt;
  }

  void publish_quote(const std::string& symbol, double bid, double ask,
                     std::int64_t bid_size = 1, std::int64_t ask_size = 1)
  {
    Quote q;
    q.provider_id = id_;
    q.symbol = symbol;
    q.bid = bid;
    q.ask = ask;
    q.bid_size = bid_size;
    q.ask_size = ask_size;
    q.received_at = std::chrono::steady_clock::now();
    auto it = subscribers_.find(symbol);
    if (it != subscribers_.end())
    {
      it->second(q);
    }
  }

private:
  std::string id_;
  bool connected_ {false};
  std::unordered_map<std::string, QuoteCallback> subscribers_;
};

}  // namespace ib_box_spread


