// rate_limiter.cpp - Rate limiter implementation for TWS API compliance
#include "rate_limiter.h"
#include <algorithm>
#include <chrono>
#include <spdlog/spdlog.h>
#include <unordered_map>

namespace tws {

// ============================================================================
// RateLimiter Implementation
// ============================================================================

RateLimiter::RateLimiter(const RateLimiterConfig &config)
    : config_(config), enabled_(config.enabled) {}

void RateLimiter::configure(const RateLimiterConfig &config) {
  std::lock_guard<std::mutex> lock(mutex_);
  config_ = config;
  enabled_ = config.enabled;

  spdlog::info("Rate limiter configured: enabled={}, max_msg/sec={}, "
               "max_historical={}, max_market_data={}",
               config.enabled, config.max_messages_per_second,
               config.max_historical_requests, config.max_market_data_lines);
}

void RateLimiter::enable() {
  enabled_ = true;
  spdlog::info("Rate limiter enabled");
}

void RateLimiter::disable() {
  enabled_ = false;
  spdlog::info("Rate limiter disabled");
}

bool RateLimiter::is_enabled() const { return enabled_.load(); }

bool RateLimiter::check_message_rate() {
  if (!enabled_.load()) {
    return true; // Not enabled, allow all messages
  }

  std::lock_guard<std::mutex> lock(mutex_);
  cleanup_old_message_timestamps();

  int messages_in_last_second = count_messages_in_last_second();

  if (messages_in_last_second >= config_.max_messages_per_second) {
    spdlog::warn("Rate limit exceeded: {} messages in last second (limit: {})",
                 messages_in_last_second, config_.max_messages_per_second);
    return false;
  }

  return true;
}

void RateLimiter::record_message() {
  if (!enabled_.load()) {
    return; // Not enabled, don't track
  }

  std::lock_guard<std::mutex> lock(mutex_);
  auto now = std::chrono::steady_clock::now();
  message_timestamps_.push_back(now);

  // Clean up old timestamps periodically (keep last 2 seconds worth)
  if (message_timestamps_.size() >
      static_cast<size_t>(config_.max_messages_per_second * 2)) {
    cleanup_old_message_timestamps();
  }
}

bool RateLimiter::can_start_historical_request(int request_id) {
  if (!enabled_.load()) {
    return true; // Not enabled, allow all requests
  }

  std::lock_guard<std::mutex> lock(mutex_);

  // Check if already active
  if (active_historical_requests_.count(request_id) > 0) {
    return false; // Already active, reject duplicate
  }

  // Check limit
  if (static_cast<int>(active_historical_requests_.size()) >=
      config_.max_historical_requests) {
    spdlog::warn("Historical request limit exceeded: {} active (limit: {})",
                 active_historical_requests_.size(),
                 config_.max_historical_requests);
    return false;
  }

  return true;
}

void RateLimiter::start_historical_request(int request_id) {
  if (!enabled_.load()) {
    return; // Not enabled, don't track
  }

  std::lock_guard<std::mutex> lock(mutex_);
  auto now = std::chrono::steady_clock::now();
  active_historical_requests_.insert(request_id);
  historical_request_times_[request_id] = now;

  spdlog::debug("Started historical request #{} (active: {}/{})", request_id,
                active_historical_requests_.size(),
                config_.max_historical_requests);
}

void RateLimiter::end_historical_request(int request_id) {
  if (!enabled_.load()) {
    return; // Not enabled, don't track
  }

  std::lock_guard<std::mutex> lock(mutex_);
  active_historical_requests_.erase(request_id);
  historical_request_times_.erase(request_id);

  spdlog::debug("Ended historical request #{} (active: {}/{})", request_id,
                active_historical_requests_.size(),
                config_.max_historical_requests);
}

bool RateLimiter::can_start_market_data(int request_id) {
  if (!enabled_.load()) {
    return true; // Not enabled, allow all subscriptions
  }

  std::lock_guard<std::mutex> lock(mutex_);

  // Check if already active
  if (active_market_data_lines_.count(request_id) > 0) {
    return false; // Already active, reject duplicate
  }

  // Check limit
  if (static_cast<int>(active_market_data_lines_.size()) >=
      config_.max_market_data_lines) {
    spdlog::warn("Market data line limit exceeded: {} active (limit: {})",
                 active_market_data_lines_.size(),
                 config_.max_market_data_lines);
    return false;
  }

  return true;
}

void RateLimiter::start_market_data(int request_id) {
  if (!enabled_.load()) {
    return; // Not enabled, don't track
  }

  std::lock_guard<std::mutex> lock(mutex_);
  auto now = std::chrono::steady_clock::now();
  active_market_data_lines_.insert(request_id);
  market_data_times_[request_id] = now;

  spdlog::debug("Started market data subscription #{} (active: {}/{})",
                request_id, active_market_data_lines_.size(),
                config_.max_market_data_lines);
}

void RateLimiter::end_market_data(int request_id) {
  if (!enabled_.load()) {
    return; // Not enabled, don't track
  }

  std::lock_guard<std::mutex> lock(mutex_);
  active_market_data_lines_.erase(request_id);
  market_data_times_.erase(request_id);

  spdlog::debug("Ended market data subscription #{} (active: {}/{})",
                request_id, active_market_data_lines_.size(),
                config_.max_market_data_lines);
}

RateLimiterStatus RateLimiter::get_status() const {
  std::lock_guard<std::mutex> lock(mutex_);

  RateLimiterStatus status;
  status.active_historical_requests =
      static_cast<int>(active_historical_requests_.size());
  status.active_market_data_lines =
      static_cast<int>(active_market_data_lines_.size());
  status.messages_in_last_second = count_messages_in_last_second();
  status.is_rate_limited =
      (status.messages_in_last_second >= config_.max_messages_per_second);

  return status;
}

void RateLimiter::cleanup_stale_requests(std::chrono::seconds max_age) {
  if (!enabled_.load()) {
    return; // Not enabled, nothing to clean
  }

  std::lock_guard<std::mutex> lock(mutex_);
  auto now = std::chrono::steady_clock::now();
  auto threshold = now - max_age;

  // Clean up stale historical requests
  std::vector<int> stale_historical;
  for (const auto &[request_id, start_time] : historical_request_times_) {
    if (start_time < threshold) {
      stale_historical.push_back(request_id);
    }
  }
  for (int request_id : stale_historical) {
    active_historical_requests_.erase(request_id);
    historical_request_times_.erase(request_id);
    spdlog::warn("Cleaned up stale historical request #{} (older than {}s)",
                 request_id, max_age.count());
  }

  // Clean up stale market data lines
  std::vector<int> stale_market_data;
  for (const auto &[request_id, start_time] : market_data_times_) {
    if (start_time < threshold) {
      stale_market_data.push_back(request_id);
    }
  }
  for (int request_id : stale_market_data) {
    active_market_data_lines_.erase(request_id);
    market_data_times_.erase(request_id);
    spdlog::warn(
        "Cleaned up stale market data subscription #{} (older than {}s)",
        request_id, max_age.count());
  }

  if (!stale_historical.empty() || !stale_market_data.empty()) {
    spdlog::info("Cleaned up {} stale historical requests and {} stale market "
                 "data lines",
                 stale_historical.size(), stale_market_data.size());
  }
}

// ============================================================================
// Helper Methods
// ============================================================================

void RateLimiter::cleanup_old_message_timestamps() {
  auto now = std::chrono::steady_clock::now();
  auto one_second_ago = now - std::chrono::seconds(1);

  // Remove timestamps older than 1 second
  message_timestamps_.erase(
      std::remove_if(message_timestamps_.begin(), message_timestamps_.end(),
                     [one_second_ago](const auto &timestamp) {
                       return timestamp < one_second_ago;
                     }),
      message_timestamps_.end());
}

int RateLimiter::count_messages_in_last_second() const {
  auto now = std::chrono::steady_clock::now();
  auto one_second_ago = now - std::chrono::seconds(1);

  // Count timestamps in the last second
  return static_cast<int>(
      std::count_if(message_timestamps_.begin(), message_timestamps_.end(),
                    [one_second_ago](const auto &timestamp) {
                      return timestamp >= one_second_ago;
                    }));
}

} // namespace tws
