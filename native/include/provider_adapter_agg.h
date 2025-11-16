// ProviderAdapter abstraction for liquidity providers (tracked path).
#pragma once

#include <functional>
#include <optional>
#include <string>
#include <string_view>
#include "agg_types_agg.h"

namespace ib_box_spread
{

class ProviderAdapter
{
public:
  virtual ~ProviderAdapter() = default;

  virtual std::string provider_id() const = 0;

  virtual bool connect(std::string_view profile) = 0;
  virtual void disconnect() = 0;
  virtual bool is_connected() const = 0;

  using QuoteCallback = std::function<void(const Quote&)>;

  virtual bool subscribe_quotes(std::string_view symbol, const QuoteCallback& on_quote) = 0;
  virtual void unsubscribe_quotes(std::string_view symbol) = 0;

  virtual std::optional<ErrorEnvelope> place_order(const OrderIntent& order) = 0;
  virtual std::optional<ErrorEnvelope> cancel_order(std::string_view client_order_id) = 0;
};

}  // namespace ib_box_spread


