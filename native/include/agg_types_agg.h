// Aggregation canonical types for multi-provider routing (tracked path).
// C++20, 2-space indent, Allman braces.
#pragma once

#include <chrono>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>

namespace ib_box_spread
{

struct Quote
{
  std::string provider_id;
  std::string symbol;
  double bid {0.0};
  double ask {0.0};
  std::int64_t bid_size {0};
  std::int64_t ask_size {0};
  std::chrono::steady_clock::time_point received_at {};
};

enum class Side
{
  Buy,
  Sell
};

struct OrderIntent
{
  std::string client_order_id;
  std::string symbol;
  Side side {Side::Buy};
  double limit_price {0.0};
  std::int64_t quantity {0};
  bool allow_partial_fills {false};
};

struct FillReport
{
  std::string provider_id;
  std::string client_order_id;
  std::string execution_id;
  std::string symbol;
  double avg_price {0.0};
  std::int64_t filled_qty {0};
  std::int64_t remaining_qty {0};
  std::chrono::steady_clock::time_point filled_at {};
};

enum class ErrorCategory
{
  None,
  Throttled,
  Network,
  ProviderDown,
  InvalidRequest,
  Rejected,
  Unknown
};

struct ErrorEnvelope
{
  ErrorCategory category {ErrorCategory::None};
  std::string code;
  std::string message;
  bool retryable {false};
};

}  // namespace ib_box_spread


