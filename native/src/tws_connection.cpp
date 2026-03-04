// tws_connection.cpp - Connection lifecycle, reconnection, health monitoring
#include "tws_connection.h"
#include "market_hours.h"
#include <spdlog/spdlog.h>

#include "EClientSocket.h"
#include "EReader.h"
#include "EReaderOSSignal.h"

#include <algorithm>

namespace tws {

ConnectionHandler::ConnectionHandler(const config::TWSConfig &config,
                                     EClientSocket &client,
                                     EReaderOSSignal &signal)
    : config_(config), client_(client), signal_(signal),
      last_heartbeat_(std::chrono::steady_clock::now()),
      rate_limiter_(RateLimiterConfig{}),
      mock_mode_(should_use_mock_client(config)) {
  if (config_.enable_pcap_capture && !mock_mode_) {
    std::string pcap_file = config_.pcap_output_file;
    if (pcap_file.empty()) {
      pcap_file = pcap::generate_pcap_filename("tws_capture");
    }
    pcap_capture_ = std::make_unique<pcap::PcapCapture>(
        pcap_file, config_.pcap_nanosecond_precision);
    if (pcap_capture_->open()) {
      spdlog::info("PCAP capture enabled: {}", pcap_file);
      local_ip_ = pcap::ip_to_uint32("127.0.0.1");
      remote_ip_ = pcap::ip_to_uint32(config_.host);
      remote_port_ = htons(static_cast<uint16_t>(config_.port));
    } else {
      spdlog::warn("Failed to open PCAP capture file: {}", pcap_file);
      pcap_capture_.reset();
    }
  }

  if (mock_mode_) {
    spdlog::info(
        "TWSClient starting in mock mode (live IBKR connections disabled).");
  }
}

ConnectionHandler::~ConnectionHandler() { disconnect(); }

// ============================================================================
// Connection lifecycle
// ============================================================================

bool ConnectionHandler::connect() {
  if (mock_mode_) {
    state_ = ConnectionState::Connected;
    connected_.store(true);
    next_order_id_ = 1000;
    connection_callbacks_received_.connectAck = true;
    connection_callbacks_received_.managedAccounts = true;
    connection_callbacks_received_.connectAck_time =
        std::chrono::steady_clock::now();
    connection_callbacks_received_.managedAccounts_time =
        connection_callbacks_received_.connectAck_time;
    connection_cv_.notify_all();
    spdlog::info("Mock TWS client connected instantly.");
    return true;
  }

  state_ = ConnectionState::Connecting;

  std::vector<int> port_candidates = get_port_candidates(config_.port);
  std::vector<int> open_ports;
  std::vector<int> closed_ports;
  std::vector<std::thread> check_threads;
  std::mutex ports_mutex;

  std::string port_list_str;
  for (size_t i = 0; i < port_candidates.size(); ++i) {
    if (i > 0)
      port_list_str += ", ";
    port_list_str += std::to_string(port_candidates[i]);
  }
  spdlog::debug("Checking {} ports in parallel: {}", port_candidates.size(),
                port_list_str);

  auto check_start = std::chrono::steady_clock::now();

  for (int port : port_candidates) {
    check_threads.emplace_back([&, port]() {
      bool is_open = is_port_open(config_.host, port, 1000);
      std::lock_guard<std::mutex> lock(ports_mutex);
      if (is_open) {
        open_ports.push_back(port);
        spdlog::info("Port {} is open on {}", port, config_.host);
      } else {
        closed_ports.push_back(port);
      }
    });
  }
  for (auto &thread : check_threads) {
    thread.join();
  }

  auto check_duration = std::chrono::duration_cast<std::chrono::milliseconds>(
                            std::chrono::steady_clock::now() - check_start)
                            .count();

  if (!open_ports.empty()) {
    std::string open_str;
    for (size_t i = 0; i < open_ports.size(); ++i) {
      if (i > 0)
        open_str += ", ";
      open_str += std::to_string(open_ports[i]);
    }
    spdlog::info("Port check complete ({}ms): {} open port(s) found: {}",
                 check_duration, open_ports.size(), open_str);
  } else {
    spdlog::warn("Port check complete ({}ms): No open ports found",
                 check_duration);
  }

  auto is_paper_port = [](int port) -> bool {
    return port == 7497 || port == 4002;
  };
  auto is_live_port = [](int port) -> bool {
    return port == 7496 || port == 4001;
  };
  auto get_port_type_string = [](int port) -> std::string {
    if (port == 7497)
      return "TWS Paper Trading";
    if (port == 7496)
      return "TWS Live Trading";
    if (port == 4002)
      return "IB Gateway Paper Trading";
    if (port == 4001)
      return "IB Gateway Live Trading";
    return "Custom";
  };

  int port_to_use = config_.port;
  bool found_open_port = false;
  bool paper_live_mismatch = false;

  if (std::find(open_ports.begin(), open_ports.end(), config_.port) !=
      open_ports.end()) {
    port_to_use = config_.port;
    found_open_port = true;
  } else if (!open_ports.empty()) {
    bool configured_is_paper = is_paper_port(config_.port);
    bool configured_is_live = is_live_port(config_.port);
    bool has_paper_port = false;
    bool has_live_port = false;
    for (int port : open_ports) {
      if (is_paper_port(port))
        has_paper_port = true;
      if (is_live_port(port))
        has_live_port = true;
    }

    if (configured_is_paper && !has_paper_port && has_live_port) {
      paper_live_mismatch = true;
      spdlog::warn("Paper/Live Trading Mismatch: configured paper ({}), only "
                   "live ports open",
                   config_.port);
    } else if (configured_is_live && !has_live_port && has_paper_port) {
      paper_live_mismatch = true;
      spdlog::warn("Paper/Live Trading Mismatch: configured live ({}), only "
                   "paper ports open",
                   config_.port);
    }

    for (int priority_port : port_candidates) {
      if (std::find(open_ports.begin(), open_ports.end(), priority_port) !=
          open_ports.end()) {
        port_to_use = priority_port;
        found_open_port = true;
        break;
      }
    }
  }

  std::string port_type = get_port_type_string(port_to_use);
  std::string configured_type = get_port_type_string(config_.port);

  if (found_open_port && port_to_use != config_.port) {
    if (paper_live_mismatch) {
      spdlog::warn("Switching from {} (port {}) to {} (port {})",
                   configured_type, config_.port, port_type, port_to_use);
    } else {
      spdlog::warn("Configured port {} is not open, using {} port {} instead",
                   config_.port, port_type, port_to_use);
    }
  } else if (found_open_port) {
    spdlog::info("Using configured port {} ({})", port_to_use, port_type);
  }

  if (!found_open_port) {
    spdlog::error("No open ports found on {}. Checked ports: {}", config_.host,
                  port_list_str);
    spdlog::error(
        "Please ensure TWS or IB Gateway is running and API is enabled");
    state_ = ConnectionState::Error;
    return false;
  }

  spdlog::info("Connecting to {}:{}...", config_.host, port_to_use);

  bool connected = false;
  int successful_port = port_to_use;

  for (int port : open_ports) {
    if (client_.isConnected()) {
      client_.eDisconnect();
      std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }

    spdlog::info("Attempting connection to {}:{}...", config_.host, port);

    connected_ = false;
    state_ = ConnectionState::Connecting;
    last_error_code_ = 0;
    last_error_message_.clear();
    connection_callbacks_received_.connectAck = false;
    connection_callbacks_received_.managedAccounts = false;

    if (pcap_capture_ && pcap_capture_->is_open()) {
      std::string connect_msg =
          "CONNECT_ATTEMPT:host=" + config_.host +
          ",port=" + std::to_string(port) +
          ",client_id=" + std::to_string(config_.client_id);
      std::vector<uint8_t> data(connect_msg.begin(), connect_msg.end());
      pcap_capture_->capture_raw(data, true, 0,
                                 htons(static_cast<uint16_t>(port)));
    }

    bool success =
        client_.eConnect(config_.host.c_str(), port, config_.client_id);

    if (pcap_capture_ && pcap_capture_->is_open()) {
      std::string result_msg = success ? "CONNECT_SUCCESS" : "CONNECT_FAILED";
      std::vector<uint8_t> data(result_msg.begin(), result_msg.end());
      pcap_capture_->capture_raw(data, true, 0,
                                 htons(static_cast<uint16_t>(port)));
    }

    if (!success) {
      spdlog::warn("eConnect() returned false for {}:{}, trying next port...",
                   config_.host, port);
      continue;
    }

    std::this_thread::sleep_for(std::chrono::milliseconds(200));

    {
      std::lock_guard<std::mutex> lock(error_mutex_);
      if (last_error_code_ != 0) {
        spdlog::warn("Error detected after eConnect() (error {}): {}",
                     last_error_code_, last_error_message_);
        if (last_error_code_ == 502 || last_error_code_ == 162 ||
            last_error_code_ == 200) {
          client_.eDisconnect();
          continue;
        }
      }
    }

    if (!client_.isConnected()) {
      spdlog::warn("Socket not connected after eConnect() for {}:{}",
                   config_.host, port);
      client_.eDisconnect();
      continue;
    }

    spdlog::info("Socket connected, starting message reader thread...");
    start_reader_thread();

    spdlog::info("Waiting for TWS acknowledgment (timeout: {}ms)...",
                 config_.connection_timeout_ms);
    bool connection_acknowledged =
        wait_for_connection_with_progress(config_.connection_timeout_ms);

    if (connection_acknowledged) {
      connected = true;
      successful_port = port;
      std::string pt = get_port_type_string(port);
      if (port != config_.port) {
        spdlog::warn(
            "Configured port {} failed, successfully connected to {} port {}",
            config_.port, pt, port);
      } else {
        spdlog::info("Successfully connected to {} port {}", pt, port);
      }
      break;
    } else {
      spdlog::warn("Connection timeout on {}:{}, trying next port...",
                   config_.host, port);
      if (reader_thread_ && reader_thread_->joinable()) {
        connected_ = false;
        signal_.issueSignal();
        reader_thread_->join();
        reader_thread_.reset();
      }
      client_.eDisconnect();
    }
  }

  if (!connected) {
    spdlog::error("Failed to connect to any open port on {}", config_.host);
    state_ = ConnectionState::Error;
    return false;
  }

  port_to_use = successful_port;
  state_ = ConnectionState::Connected;
  spdlog::info("Connected to {}:{}", config_.host, port_to_use);
  return true;
}

void ConnectionHandler::disconnect() {
  if (mock_mode_) {
    connected_ = false;
    state_ = ConnectionState::Disconnected;
    spdlog::info("Mock TWS client disconnected.");
    return;
  }
  if (connected_) {
    spdlog::info("Disconnecting from TWS...");
    stop_health_monitoring();

    if (reader_thread_ && reader_thread_->joinable()) {
      connected_ = false;
      signal_.issueSignal();
      reader_thread_->join();
    }

    client_.eDisconnect();
    state_ = ConnectionState::Disconnected;
    spdlog::info("Disconnected");
  }
}

bool ConnectionHandler::is_connected() const {
  if (mock_mode_)
    return connected_.load();
  return connected_ && client_.isConnected();
}

ConnectionState ConnectionHandler::get_state() const { return state_; }

void ConnectionHandler::process_messages(int timeout_ms) {
  if (mock_mode_) {
    std::this_thread::sleep_for(std::chrono::milliseconds(timeout_ms));
    last_heartbeat_ = std::chrono::steady_clock::now();
    return;
  }
  std::this_thread::sleep_for(std::chrono::milliseconds(timeout_ms));
}

// ============================================================================
// EWrapper callback handlers
// ============================================================================

void ConnectionHandler::on_connect_ack() {
  try {
    spdlog::info("connectAck received - Socket connection established");
    connection_callbacks_received_.connectAck = true;
    connection_callbacks_received_.connectAck_time =
        std::chrono::steady_clock::now();

    if (pcap_capture_ && pcap_capture_->is_open()) {
      std::string event_data = "CONNECTION_ACK";
      std::vector<uint8_t> data(event_data.begin(), event_data.end());
      pcap_capture_->capture_raw(data, false, local_port_, remote_port_);
    }

    reconnect_attempts_ = 0;
    last_message_time_ = std::chrono::steady_clock::now();
    client_.reqIds(-1);
  } catch (const std::exception &e) {
    spdlog::error("Exception in connectAck: {}", e.what());
  }
}

void ConnectionHandler::on_connection_closed() {
  try {
    spdlog::warn("Connection closed by TWS");
    connected_ = false;
    state_ = ConnectionState::Disconnected;

    if (pcap_capture_ && pcap_capture_->is_open()) {
      std::string event_data = "CONNECTION_CLOSED";
      std::vector<uint8_t> data(event_data.begin(), event_data.end());
      pcap_capture_->capture_raw(data, false, local_port_, remote_port_);
      pcap_capture_->flush();
    }

    if (error_callback_) {
      error_callback_(1100, "Connection closed by TWS");
    }

    if (config_.auto_reconnect) {
      attempt_reconnect_with_backoff();
    }
  } catch (const std::exception &e) {
    spdlog::error("Exception in connectionClosed: {}", e.what());
  }
}

void ConnectionHandler::on_managed_accounts(const std::string &accounts_list) {
  try {
    spdlog::info("managedAccounts received: {}", accounts_list);
    connection_callbacks_received_.managedAccounts = true;
    connection_callbacks_received_.managedAccounts_time =
        std::chrono::steady_clock::now();
  } catch (const std::exception &e) {
    spdlog::error("Exception in managedAccounts: {}", e.what());
  }
}

void ConnectionHandler::on_next_valid_id(int order_id) {
  try {
    auto total_elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                             std::chrono::steady_clock::now() -
                             connection_callbacks_received_.connectAck_time)
                             .count();
    spdlog::info("nextValidId received: {} (connection complete in {}ms)",
                 order_id, total_elapsed);

    next_order_id_ = order_id;

    if (pcap_capture_ && pcap_capture_->is_open()) {
      std::string event_data =
          "CONNECTION_COMPLETE:nextValidId=" + std::to_string(order_id);
      std::vector<uint8_t> data(event_data.begin(), event_data.end());
      pcap_capture_->capture_raw(data, false, local_port_, remote_port_);
    }

    std::lock_guard<std::mutex> lock(connection_mutex_);
    connected_ = true;

    bool is_reconnection = (reconnect_attempts_.load() > 0);
    start_health_monitoring();

    client_.reqAllOpenOrders();

    if (is_reconnection) {
      spdlog::info("Synchronizing state after reconnection...");
      client_.reqPositions();
      client_.reqAccountUpdates(true, "");
      reconnect_attempts_ = 0;
    }

    connection_cv_.notify_all();
    spdlog::info("Connection fully established and ready");
  } catch (const std::exception &e) {
    spdlog::error("Exception in nextValidId: {}", e.what());
  }
}

void ConnectionHandler::on_error(
    int id, int error_code, const std::string &error_string,
    const std::string &advanced_order_reject_json) {
  try {
    {
      std::lock_guard<std::mutex> lock(error_mutex_);
      last_error_code_ = error_code;
      last_error_message_ = error_string;
      last_error_time_ = std::chrono::system_clock::now();
      error_count_last_hour_++;
    }

    if (error_code == 502 || (error_code >= 500 && error_code < 600)) {
      spdlog::error("TWS connection error {}: {}", error_code, error_string);
      connected_ = false;
      state_ = ConnectionState::Error;
      connection_cv_.notify_all();
    }

    if (error_code == 162 || error_code == 200) {
      spdlog::error("TWS authentication/authorization error {}: {}", error_code,
                    error_string);
      connected_ = false;
      state_ = ConnectionState::Error;
      connection_cv_.notify_all();
    }

    std::string guidance;
    if (auto it = kIbErrorGuidance.find(error_code);
        it != kIbErrorGuidance.end()) {
      guidance = it->second;
    }

    std::vector<std::string> guidance_notes;

    if (error_code >= 2100 && error_code < 3000) {
      spdlog::info("[IB Info {}] ID: {} | {}", error_code, id, error_string);
      if (!guidance.empty())
        spdlog::info("  -> {}", guidance);
    } else if (error_code >= 1100 && error_code < 2000) {
      spdlog::warn("[IB System {}] ID: {} | {}", error_code, id, error_string);
      if (!guidance.empty())
        spdlog::warn("  -> {}", guidance);

      if (error_code == 1100) {
        connected_ = false;
        state_ = ConnectionState::Error;
        if (config_.auto_reconnect)
          attempt_reconnect_with_backoff();
      } else if (error_code == 1101 || error_code == 1102) {
        connected_ = true;
        state_ = ConnectionState::Connected;
      }
    } else {
      spdlog::error("[IB Error {}] ID: {} | {}", error_code, id, error_string);
      if (!guidance.empty())
        spdlog::error("  -> {}", guidance);
    }

    if (!guidance.empty())
      guidance_notes.emplace_back(guidance);

    for (size_t i = 0; i < kErrorPhraseGuidanceCount; ++i) {
      if (error_string.find(kErrorPhraseGuidance[i].first) !=
          std::string::npos) {
        spdlog::warn("Guidance: {}", kErrorPhraseGuidance[i].second);
        guidance_notes.emplace_back(kErrorPhraseGuidance[i].second);
      }
    }

    if (error_callback_) {
      if (guidance_notes.empty()) {
        error_callback_(error_code, error_string);
      } else {
        std::string enriched = error_string + " | Guidance: ";
        for (size_t i = 0; i < guidance_notes.size(); ++i) {
          if (i > 0)
            enriched += " | ";
          enriched += guidance_notes[i];
        }
        error_callback_(error_code, enriched);
      }
    }
  } catch (const std::exception &e) {
    spdlog::error("Exception in error callback: {}", e.what());
  }
}

void ConnectionHandler::on_current_time(long time) {
  server_time_epoch_.store(time);
  spdlog::debug("TWS server time: {}", time);
}

// ============================================================================
// Rate limiting
// ============================================================================

bool ConnectionHandler::check_rate_limit() {
  return rate_limiter_.check_message_rate();
}

void ConnectionHandler::record_rate_message() {
  rate_limiter_.record_message();
}

bool ConnectionHandler::can_start_market_data(int request_id) {
  return rate_limiter_.can_start_market_data(request_id);
}

void ConnectionHandler::start_market_data_tracking(int request_id) {
  rate_limiter_.start_market_data(request_id);
}

void ConnectionHandler::end_market_data_tracking(int request_id) {
  rate_limiter_.end_market_data(request_id);
}

void ConnectionHandler::enable_rate_limiting() {
  RateLimiterConfig rl_config;
  rl_config.enabled = true;
  rate_limiter_.configure(rl_config);
}

void ConnectionHandler::configure_rate_limiter(
    const RateLimiterConfig &rl_config) {
  rate_limiter_.configure(rl_config);
}

std::optional<RateLimiterStatus>
ConnectionHandler::get_rate_limiter_status() const {
  if (!rate_limiter_.is_enabled())
    return std::nullopt;
  return rate_limiter_.get_status();
}

void ConnectionHandler::cleanup_stale_rate_limiter_requests(
    std::chrono::seconds max_age) {
  rate_limiter_.cleanup_stale_requests(max_age);
}

// ============================================================================
// Callbacks and queries
// ============================================================================

void ConnectionHandler::set_error_callback(ErrorCallback callback) {
  error_callback_ = callback;
}

std::pair<std::string, int> ConnectionHandler::get_last_error() const {
  std::lock_guard<std::mutex> lock(error_mutex_);
  auto now = std::chrono::system_clock::now();
  int recent_errors = error_count_last_hour_;
  if (last_error_time_ != std::chrono::system_clock::time_point{} &&
      (now - last_error_time_) > std::chrono::hours(1)) {
    recent_errors = 0;
  }
  return {last_error_message_, recent_errors};
}

void ConnectionHandler::record_heartbeat() {
  last_heartbeat_ = std::chrono::steady_clock::now();
}

bool ConnectionHandler::is_market_open() const {
  static market_hours::MarketHours market_hours;
  auto status = market_hours.get_market_status();
  return status.is_open &&
         status.current_session == market_hours::MarketSession::Regular;
}

std::chrono::system_clock::time_point
ConnectionHandler::get_server_time() const {
  if (server_time_epoch_.load() > 0) {
    return std::chrono::system_clock::from_time_t(server_time_epoch_.load());
  }
  return std::chrono::system_clock::now();
}

// ============================================================================
// Private helpers
// ============================================================================

void ConnectionHandler::start_reader_thread() {
  if (mock_mode_)
    return;
  spdlog::debug("Starting EReader thread...");
  auto reader = std::make_unique<EReader>(&client_, &signal_);
  reader->start();

  reader_thread_ =
      std::make_unique<std::thread>([this, r = std::move(reader)]() mutable {
        int message_count = 0;
        auto thread_start = std::chrono::steady_clock::now();

        while (client_.isConnected() || connected_.load()) {
          signal_.waitForSignal();
          if (!client_.isConnected() && !connected_.load())
            break;

          try {
            r->processMsgs();
            message_count++;
            last_message_time_ = std::chrono::steady_clock::now();
          } catch (const std::exception &e) {
            spdlog::error("Error processing messages: {}", e.what());
            if (!client_.isConnected())
              break;
          } catch (...) {
            spdlog::error("Unknown exception in EReader thread");
            if (!client_.isConnected())
              break;
          }
        }

        auto uptime = std::chrono::duration_cast<std::chrono::seconds>(
                          std::chrono::steady_clock::now() - thread_start)
                          .count();
        spdlog::debug("EReader thread stopped after {}s ({} messages)", uptime,
                      message_count);
      });
}

void ConnectionHandler::attempt_reconnect_with_backoff() {
  std::lock_guard<std::mutex> lock(reconnect_mutex_);

  int attempts = reconnect_attempts_.load();
  const int max_retries =
      config_.max_reconnect_attempts > 0 ? config_.max_reconnect_attempts : 10;

  if (attempts >= max_retries) {
    spdlog::error("Max reconnection attempts ({}) reached.", max_retries);
    state_ = ConnectionState::Error;
    return;
  }

  int delay_ms = std::min(1000 * (1 << attempts), 30000);
  reconnect_attempts_++;
  last_reconnect_attempt_ = std::chrono::steady_clock::now();

  spdlog::info("Attempting reconnection (attempt {}/{}, delay: {}ms)...",
               attempts + 1, max_retries, delay_ms);

  std::this_thread::sleep_for(std::chrono::milliseconds(delay_ms));

  std::thread([this]() {
    if (connect()) {
      spdlog::info("Reconnected successfully after {} attempts",
                   reconnect_attempts_.load());
    }
  }).detach();
}

void ConnectionHandler::start_health_monitoring() {
  if (health_check_enabled_.load())
    return;

  health_check_enabled_ = true;
  last_message_time_ = std::chrono::steady_clock::now();

  health_check_thread_ = std::make_unique<std::thread>([this]() {
    const auto health_check_interval = std::chrono::seconds(30);
    const auto stale_threshold = std::chrono::minutes(2);

    while (health_check_enabled_.load() && connected_.load()) {
      std::this_thread::sleep_for(health_check_interval);
      if (!connected_.load())
        break;

      auto now = std::chrono::steady_clock::now();
      auto time_since_last = now - last_message_time_;

      if (time_since_last > stale_threshold) {
        spdlog::warn(
            "Connection appears stale ({}s since last message)",
            std::chrono::duration_cast<std::chrono::seconds>(time_since_last)
                .count());

        if (!client_.isConnected()) {
          connected_ = false;
          state_ = ConnectionState::Error;
          on_connection_closed();
          break;
        } else {
          client_.reqCurrentTime();
          rate_limiter_.record_message();
        }
      }
      last_heartbeat_ = now;
    }
  });

  spdlog::debug("Connection health monitoring started");
}

void ConnectionHandler::stop_health_monitoring() {
  if (!health_check_enabled_.load())
    return;
  health_check_enabled_ = false;
  if (health_check_thread_ && health_check_thread_->joinable()) {
    health_check_thread_->join();
    health_check_thread_.reset();
  }
}

bool ConnectionHandler::wait_for_connection(int timeout_ms) {
  std::unique_lock<std::mutex> lock(connection_mutex_);
  return connection_cv_.wait_for(lock, std::chrono::milliseconds(timeout_ms),
                                 [this] { return connected_.load(); });
}

bool ConnectionHandler::wait_for_connection_with_progress(int timeout_ms) {
  if (mock_mode_)
    return connected_.load();

  std::unique_lock<std::mutex> lock(connection_mutex_);
  auto start = std::chrono::steady_clock::now();
  auto last_log = start;
  const auto log_interval = std::chrono::seconds(2);

  while (true) {
    auto now = std::chrono::steady_clock::now();
    auto elapsed =
        std::chrono::duration_cast<std::chrono::milliseconds>(now - start)
            .count();

    if (elapsed >= timeout_ms) {
      spdlog::warn("Connection timeout after {}ms", elapsed);
      return false;
    }

    {
      std::lock_guard<std::mutex> error_lock(error_mutex_);
      if (last_error_code_ == 502 || last_error_code_ == 162 ||
          last_error_code_ == 200 ||
          (last_error_code_ >= 500 && last_error_code_ < 600)) {
        spdlog::warn("Connection error {} during wait: {}", last_error_code_,
                     last_error_message_);
        return false;
      }
    }

    if (connected_.load()) {
      spdlog::info("Connection acknowledged after {}ms", elapsed);
      return true;
    }

    if ((now - last_log) >= log_interval) {
      int progress = 0;
      if (connection_callbacks_received_.connectAck)
        progress += 33;
      if (connection_callbacks_received_.managedAccounts)
        progress += 33;
      spdlog::info("Connection progress: {}% ({}ms remaining)", progress,
                   timeout_ms - elapsed);
      last_log = now;
    }

    auto wait_time = std::min(std::chrono::milliseconds(timeout_ms - elapsed),
                              std::chrono::milliseconds(500));

    if (connection_cv_.wait_for(lock, wait_time,
                                [this] { return connected_.load(); })) {
      auto total = std::chrono::duration_cast<std::chrono::milliseconds>(
                       std::chrono::steady_clock::now() - start)
                       .count();
      spdlog::info("Connection acknowledged after {}ms", total);
      return true;
    }
  }
}

} // namespace tws
