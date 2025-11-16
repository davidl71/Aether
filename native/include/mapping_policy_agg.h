// MappingPolicy centralizes provider request/response normalization (tracked path).
#pragma once

#include <string>
#include <string_view>
#include "agg_types_agg.h"

namespace ib_box_spread
{

class MappingPolicy
{
public:
  static std::string normalize_symbol(std::string_view symbol)
  {
    return std::string(symbol);
  }

  static ErrorEnvelope make_error(std::string code, std::string message, bool retryable)
  {
    return ErrorEnvelope{
      ErrorCategory::Unknown,
      std::move(code),
      std::move(message),
      retryable
    };
  }
};

}  // namespace ib_box_spread
