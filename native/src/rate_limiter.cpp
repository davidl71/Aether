// rate_limiter.cpp - Rate limiter implementation for TWS API compliance
#include "rate_limiter.h"
#include <algorithm>
#include <chrono>
#include <filesystem>
#include <fstream>
#include <nlohmann/json.hpp>
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

void RateLimiter::set_state_path(const std::string &path) {
  std::lock_guard<std::mutex> lock(mutex_);
  state_path_ = path;
}

void RateLimiter::start_historical_request(int request_id) {
  if (!enabled_.load()) {
    return; // Not enabled, don't track
  }

  {
    std::lock_guard<std::mutex> lock(mutex_);
    auto now = std::chrono::steady_clock::now();
    active_historical_requests_.insert(request_id);
    historical_request_times_[request_id] = now;

    spdlog::debug("Started historical request #{} (active: {}/{})", request_id,
                  active_historical_requests_.size(),
                  config_.max_historical_requests);
  }

  // Persist after recording so a crash immediately after doesn't lose this entry.
  if (!state_path_.empty()) {
    save_state(state_path_);
  }
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
// Persistence
// ============================================================================

void RateLimiter::save_state(const std::string &path) const {
  static constexpr auto kWindow = std::chrono::minutes(10);

  std::lock_guard<std::mutex> lock(mutex_);

  auto now_steady = std::chrono::steady_clock::now();
  auto now_system = std::chrono::system_clock::now();

  nlohmann::json j;
  j["historical_request_timestamps_ms"] = nlohmann::json::array();

  for (const auto &[id, tp] : historical_request_times_) {
    if (now_steady - tp < kWindow) {
      // Convert steady_clock time_point to a system_clock wall timestamp.
      auto age = now_steady - tp;
      auto wall_tp = now_system - std::chrono::duration_cast<std::chrono::system_clock::duration>(age);
      auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
                    wall_tp.time_since_epoch())
                    .count();
      j["historical_request_timestamps_ms"].push_back(ms);
    }
  }

  // Write atomically: tmp file then rename.
  std::string tmp = path + ".tmp";
  try {
    {
      std::ofstream ofs(tmp);
      ofs << j.dump();
    }
    std::filesystem::rename(tmp, path);
    spdlog::debug("RateLimiter: saved {} historical timestamps to {}",
                  j["historical_request_timestamps_ms"].size(), path);
  }
  catch (const std::exception &e) {
    spdlog::warn("RateLimiter: failed to save state to {}: {}", path, e.what());
    std::filesystem::remove(tmp);
  }
}

void RateLimiter::load_state(const std::string &path) {
  static constexpr auto kWindow = std::chrono::minutes(10);

  std::ifstream ifs(path);
  if (!ifs.is_open()) {
    return; // No prior state file — fresh start is fine.
  }

  try {
    nlohmann::json j;
    ifs >> j;

    auto now_steady = std::chrono::steady_clock::now();
    auto now_system = std::chrono::system_clock::now();

    int restored = 0;
    // Use negative IDs to avoid colliding with live request IDs (which start at 1000).
    int synthetic_id = -1;

    std::lock_guard<std::mutex> lock(mutex_);
    for (auto ms_val : j.at("historical_request_timestamps_ms")) {
      auto ms = std::chrono::milliseconds(ms_val.get<long long>());
      auto wall_tp = std::chrono::system_clock::time_point(
          std::chrono::duration_cast<std::chrono::system_clock::duration>(ms));

      if (wall_tp > now_system) {
        continue; // Clock skew or corrupt entry — skip.
      }

      auto age = std::chrono::duration_cast<std::chrono::steady_clock::duration>(
          now_system - wall_tp);

      if (age < kWindow) {
        historical_request_times_[synthetic_id] = now_steady - age;
        // Do NOT add to active_historical_requests_ — these are completed
        // requests that only count toward the pace quota, not the concurrency limit.
        --synthetic_id;
        ++restored;
      }
    }

    if (restored > 0) {
      spdlog::info("RateLimiter: restored {} historical request timestamps "
                   "from {} (within 10-min pace window)",
                   restored, path);
    }
  }
  catch (const std::exception &e) {
    spdlog::warn("RateLimiter: ignoring corrupt state file {}: {}", path, e.what());
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
