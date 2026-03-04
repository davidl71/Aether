#pragma once

#include "tws_client.h"
#include "connection_utils.h"
#include "tws_error_codes.h"
#include "pcap_capture.h"
#include "rate_limiter.h"

#include <atomic>
#include <chrono>
#include <condition_variable>
#include <memory>
#include <mutex>
#include <string>
#include <thread>
#include <vector>

class EClientSocket;
class EReaderOSSignal;
class EReader;

namespace tws {

class ConnectionHandler
{
public:
  explicit ConnectionHandler(const config::TWSConfig& config,
                             EClientSocket& client,
                             EReaderOSSignal& signal);
  ~ConnectionHandler();

  ConnectionHandler(const ConnectionHandler&) = delete;
  ConnectionHandler& operator=(const ConnectionHandler&) = delete;

  [[nodiscard]] bool connect();
  void disconnect();
  [[nodiscard]] bool is_connected() const;
  [[nodiscard]] ConnectionState get_state() const;
  void process_messages(int timeout_ms);

  void on_connect_ack();
  void on_connection_closed();
  void on_managed_accounts(const std::string& accounts_list);
  void on_next_valid_id(int order_id);
  void on_error(int id, int error_code, const std::string& error_string,
                const std::string& advanced_order_reject_json);
  void on_current_time(long time);

  void record_heartbeat();
  [[nodiscard]] bool is_mock_mode() const { return mock_mode_; }
  [[nodiscard]] int next_order_id() const { return next_order_id_.load(); }
  int claim_order_id() { return next_order_id_++; }
  int claim_request_id() { return next_request_id_++; }
  [[nodiscard]] bool check_rate_limit();
  void record_rate_message();

  void set_error_callback(ErrorCallback callback);
  [[nodiscard]] std::pair<std::string, int> get_last_error() const;

  [[nodiscard]] bool is_market_open() const;
  [[nodiscard]] std::chrono::system_clock::time_point get_server_time() const;

  void enable_rate_limiting();
  void configure_rate_limiter(const RateLimiterConfig& rl_config);
  [[nodiscard]] std::optional<RateLimiterStatus> get_rate_limiter_status() const;
  void cleanup_stale_rate_limiter_requests(std::chrono::seconds max_age);

  [[nodiscard]] bool can_start_market_data(int request_id);
  void start_market_data_tracking(int request_id);
  void end_market_data_tracking(int request_id);

  [[nodiscard]] const config::TWSConfig& config() const { return config_; }
  [[nodiscard]] EClientSocket& client() { return client_; }

private:
  void start_reader_thread();
  void attempt_reconnect_with_backoff();
  void start_health_monitoring();
  void stop_health_monitoring();
  bool wait_for_connection(int timeout_ms);
  bool wait_for_connection_with_progress(int timeout_ms);
  void seed_mock_state();

  config::TWSConfig config_;
  EClientSocket& client_;
  EReaderOSSignal& signal_;

  std::atomic<bool> connected_{false};
  std::atomic<int> next_order_id_{0};
  std::atomic<int> next_request_id_{1000};
  ConnectionState state_{ConnectionState::Disconnected};

  std::atomic<int> last_error_code_{0};
  std::string last_error_message_;
  mutable std::mutex error_mutex_;
  std::chrono::system_clock::time_point last_error_time_{};
  int error_count_last_hour_{0};
  ErrorCallback error_callback_;

  std::atomic<int> reconnect_attempts_{0};
  std::chrono::steady_clock::time_point last_reconnect_attempt_;
  std::mutex reconnect_mutex_;

  std::chrono::steady_clock::time_point last_heartbeat_;
  std::chrono::steady_clock::time_point last_message_time_;
  std::atomic<bool> health_check_enabled_{false};
  std::unique_ptr<std::thread> health_check_thread_;

  struct ConnectionCallbacks
  {
    bool connectAck = false;
    bool managedAccounts = false;
    std::chrono::steady_clock::time_point connectAck_time;
    std::chrono::steady_clock::time_point managedAccounts_time;
  } connection_callbacks_received_;

  std::unique_ptr<std::thread> reader_thread_;

  std::unique_ptr<pcap::PcapCapture> pcap_capture_;
  uint32_t local_ip_{0};
  uint32_t remote_ip_{0};
  uint16_t local_port_{0};
  uint16_t remote_port_{0};

  mutable std::mutex connection_mutex_;
  std::condition_variable connection_cv_;

  std::atomic<long> server_time_epoch_{0};

  RateLimiter rate_limiter_;
  bool mock_mode_;
};

} // namespace tws
