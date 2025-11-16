// Abstract session manager to encapsulate provider-specific connection lifecycles (tracked path).
#pragma once

#include <chrono>
#include <string_view>

namespace ib_box_spread
{

class SessionManager
{
public:
  virtual ~SessionManager() = default;

  virtual bool start(std::string_view profile) = 0;
  virtual void stop() = 0;
  virtual bool running() const = 0;
  virtual void set_heartbeat_interval(std::chrono::milliseconds interval) = 0;
};

}  // namespace ib_box_spread


