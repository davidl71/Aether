// Aggregator for best-quote selection and simple routing metadata (tracked path).
#pragma once

#include <algorithm>
#include <chrono>
#include <functional>
#include <mutex>
#include <optional>
#include <string>
#include <string_view>
#include <unordered_map>
#include <unordered_set>
#include <vector>
#include "agg_types_agg.h"
#include "provider_adapter_agg.h"

namespace ib_box_spread
{

class Aggregator
{
public:
  explicit Aggregator(std::chrono::milliseconds freshness_budget_ms = std::chrono::milliseconds(500))
    : freshness_budget_{freshness_budget_ms}
  {
  }

  void register_provider(ProviderAdapter* adapter)
  {
    std::scoped_lock lock(mutex_);
    providers_.push_back(adapter);
  }

  bool subscribe_symbol(std::string symbol)
  {
    std::scoped_lock lock(mutex_);
    for (auto* p : providers_)
    {
      if (!p->is_connected())
      {
        continue;
      }
      p->subscribe_quotes(symbol, [this](const Quote& q) { on_quote_(q); });
    }
    symbols_.insert(symbol);
    return true;
  }

  std::optional<Quote> best_quote(std::string_view symbol) const
  {
    std::scoped_lock lock(mutex_);
    const auto it = latest_quotes_.find(std::string(symbol));
    if (it == latest_quotes_.end())
    {
      return std::nullopt;
    }
    const auto& per_provider = it->second;

    const auto now = std::chrono::steady_clock::now();
    std::optional<Quote> best {};
    for (const auto& [_, q] : per_provider)
    {
      if (now - q.received_at > freshness_budget_)
      {
        continue;
      }
      if (!best.has_value())
      {
        best = q;
        continue;
      }
      const double best_spread = best->ask - best->bid;
      const double q_spread = q.ask - q.bid;
      const bool tighter = q_spread < best_spread;
      const bool better_ask = q.ask < best->ask;
      const bool better_bid = q.bid > best->bid;
      if (tighter || better_ask || better_bid)
      {
        best = q;
      }
    }
    return best;
  }

private:
  void on_quote_(const Quote& q) const
  {
    auto& per_provider = latest_quotes_[q.symbol];
    per_provider[q.provider_id] = q;
  }

  std::chrono::milliseconds freshness_budget_;
  mutable std::vector<ProviderAdapter*> providers_;
  mutable std::unordered_map<std::string, std::unordered_map<std::string, Quote>> latest_quotes_;
  mutable std::unordered_set<std::string> symbols_;
  mutable std::mutex mutex_;
};

}  // namespace ib_box_spread
