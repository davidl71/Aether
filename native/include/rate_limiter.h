// rate_limiter.h - Rate limiter for TWS API compliance
#pragma once

#include <chrono>
#include <mutex>
#include <deque>
#include <string>
#include <unordered_set>
#include <unordered_map>
#include <atomic>

namespace tws {

// ============================================================================
// Rate Limiter Configuration
// ============================================================================

struct RateLimiterConfig {
  bool enabled = false;
  int max_messages_per_second = 50;      // IBKR allows up to 50 msg/sec
  int max_historical_requests = 50;      // Max simultaneous historical requests
  int max_market_data_lines = 100;       // Max market data subscriptions
};

// ============================================================================
// Rate Limiter Status
// ============================================================================

struct RateLimiterStatus {
  int active_historical_requests = 0;
  int active_market_data_lines = 0;
  int messages_in_last_second = 0;
  bool is_rate_limited = false;
};

// ============================================================================
// Rate Limiter Class
// ============================================================================

/**
 * Rate limiter for TWS API compliance
 *
 * Enforces IBKR rate limits:
 * - Maximum messages per second (default: 50)
 * - Maximum simultaneous historical data requests (default: 50)
 * - Maximum market data lines (default: 100)
 *
 * Based on yatws implementation patterns.
 */
class RateLimiter {
public:
  explicit RateLimiter(const RateLimiterConfig& config = RateLimiterConfig{});

  // Configuration
  void configure(const RateLimiterConfig& config);
  void enable();
  void disable();
  bool is_enabled() const;

  // Message rate limiting
  bool check_message_rate();  // Returns true if allowed, false if rate limited
  void record_message();       // Record a message was sent

  // Historical data request limiting
  bool can_start_historical_request(int request_id);  // Check if can start request
  void start_historical_request(int request_id);     // Mark request as started
  void end_historical_request(int request_id);        // Mark request as ended

  // Market data line limiting
  bool can_start_market_data(int request_id);        // Check if can start subscription
  void start_market_data(int request_id);            // Mark subscription as started
  void end_market_data(int request_id);              // Mark subscription as ended

  // Status
  RateLimiterStatus get_status() const;

  // Cleanup
  void cleanup_stale_requests(std::chrono::seconds max_age);

  // Persistence — save/restore historical request timestamps across restarts.
  // Only entries within the last 10 minutes are persisted (IB pace-API window).
  // save_state() writes atomically (tmp file + rename). load_state() silently
  // ignores a missing or corrupt file.
  void save_state(const std::string &path) const;
  void load_state(const std::string &path);

  // Configure a path for automatic state saves on each historical request.
  // Call once after construction to enable auto-persistence.
  void set_state_path(const std::string &path);

private:
  mutable std::mutex mutex_;
  RateLimiterConfig config_;
  std::atomic<bool> enabled_;

  // Message rate tracking
  std::deque<std::chrono::steady_clock::time_point> message_timestamps_;

  // Historical request tracking
  std::unordered_set<int> active_historical_requests_;
  std::unordered_map<int, std::chrono::steady_clock::time_point> historical_request_times_;

  // Market data line tracking
  std::unordered_set<int> active_market_data_lines_;
  std::unordered_map<int, std::chrono::steady_clock::time_point> market_data_times_;

  // Auto-save path (empty = disabled)
  std::string state_path_;

  // Helper methods
  void cleanup_old_message_timestamps();
  int count_messages_in_last_second() const;
};

} // namespace tws
