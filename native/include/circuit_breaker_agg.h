// Simple circuit breaker to guard provider calls (tracked path).
#pragma once

#include <chrono>
#include <cstdint>
#include <optional>

namespace ib_box_spread
{

class CircuitBreaker
{
public:
  enum class State
  {
    Closed,
    Open,
    HalfOpen
  };

  explicit CircuitBreaker(std::int32_t failure_threshold = 5,
                          std::chrono::milliseconds open_duration = std::chrono::seconds(5))
    : failure_threshold_{failure_threshold},
      open_duration_{open_duration}
  {
  }

  void on_success()
  {
    failure_count_ = 0;
    if (state_ != State::Closed)
    {
      state_ = State::Closed;
    }
  }

  void on_failure()
  {
    ++failure_count_;
    if (state_ == State::Closed && failure_count_ >= failure_threshold_)
    {
      state_ = State::Open;
      opened_at_ = std::chrono::steady_clock::now();
    }
  }

  State state() const
  {
    if (state_ == State::Open)
    {
      const auto now = std::chrono::steady_clock::now();
      if (opened_at_.has_value() && now - *opened_at_ >= open_duration_)
      {
        return State::HalfOpen;
      }
    }
    return state_;
  }

  void transition_half_open()
  {
    state_ = State::HalfOpen;
  }

  void transition_open()
  {
    state_ = State::Open;
    opened_at_ = std::chrono::steady_clock::now();
  }

private:
  std::int32_t failure_threshold_;
  std::chrono::milliseconds open_duration_;
  std::int32_t failure_count_ {0};
  State state_ {State::Closed};
  std::optional<std::chrono::steady_clock::time_point> opened_at_ {};
};

}  // namespace ib_box_spread
