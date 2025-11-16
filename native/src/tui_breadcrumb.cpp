// tui_breadcrumb.cpp - Breadcrumb logging implementation
#include "tui_breadcrumb.h"
#include <spdlog/sinks/rotating_file_sink.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <algorithm>
#include <fstream>
#include <nlohmann/json.hpp>

namespace tui {

// ============================================================================
// BreadcrumbEntry Implementation
// ============================================================================

std::string BreadcrumbEntry::ToString() const {
  std::ostringstream oss;
  auto time_t = std::chrono::system_clock::to_time_t(timestamp);
  std::tm tm_buf;
  localtime_r(&time_t, &tm_buf);

  oss << std::put_time(&tm_buf, "%Y-%m-%d %H:%M:%S");
  auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
      timestamp.time_since_epoch()) % 1000;
  oss << "." << std::setfill('0') << std::setw(3) << ms.count();

  oss << " [" << static_cast<int>(type) << "]";
  oss << " " << component_id;
  oss << " " << action;
  if (!details.empty()) {
    oss << " | " << details;
  }
  if (!state_snapshot.empty()) {
    oss << " | State: " << state_snapshot.substr(0, 100);
    if (state_snapshot.length() > 100) oss << "...";
  }

  return oss.str();
}

std::string BreadcrumbEntry::ToJSON() const {
  nlohmann::json j;

  auto time_t = std::chrono::system_clock::to_time_t(timestamp);
  auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
      timestamp.time_since_epoch()) % 1000;

  std::ostringstream oss;
  oss << std::put_time(std::localtime(&time_t), "%Y-%m-%dT%H:%M:%S");
  oss << "." << std::setfill('0') << std::setw(3) << ms.count();
  oss << "Z";

  j["timestamp"] = oss.str();
  j["type"] = static_cast<int>(type);
  j["component_id"] = component_id;
  j["action"] = action;
  j["details"] = details;
  if (!state_snapshot.empty()) {
    j["state_snapshot"] = state_snapshot;
  }

  return j.dump();
}

// ============================================================================
// BreadcrumbLogger Implementation
// ============================================================================

BreadcrumbLogger::BreadcrumbLogger(const Config& config)
  : config_(config), last_flush_(std::chrono::system_clock::now()) {

  if (!config_.enabled) {
    return;
  }

  // Setup logger
  std::vector<spdlog::sink_ptr> sinks;

  if (config_.log_to_file && !config_.log_file.empty()) {
    auto max_size = 10 * 1024 * 1024;  // 10MB
    auto file_sink = std::make_shared<spdlog::sinks::rotating_file_sink_mt>(
        config_.log_file, max_size, 5);
    sinks.push_back(file_sink);
  }

  if (config_.log_to_console) {
    auto console_sink = std::make_shared<spdlog::sinks::stdout_color_sink_mt>();
    sinks.push_back(console_sink);
  }

  if (!sinks.empty()) {
    logger_ = std::make_shared<spdlog::logger>("breadcrumb", sinks.begin(), sinks.end());
    logger_->set_level(spdlog::level::debug);
    logger_->set_pattern("[%Y-%m-%d %H:%M:%S.%e] [breadcrumb] %v");
  }
}

BreadcrumbLogger::~BreadcrumbLogger() {
  Flush();
}

void BreadcrumbLogger::LogInput(const std::string& component_id,
                                 const std::string& input,
                                 const std::string& details) {
  Log(BreadcrumbType::UserInput, component_id, "input",
      "input=" + input + (details.empty() ? "" : " | " + details));
}

void BreadcrumbLogger::LogNavigation(const std::string& from,
                                      const std::string& to,
                                      const std::string& details) {
  Log(BreadcrumbType::Navigation, from, "navigate",
      "to=" + to + (details.empty() ? "" : " | " + details));
}

void BreadcrumbLogger::LogStateChange(const std::string& component_id,
                                       const std::string& change,
                                       const std::string& state_dump) {
  Log(BreadcrumbType::StateChange, component_id, "state_change", change, state_dump);
}

void BreadcrumbLogger::LogError(const std::string& component_id,
                                 const std::string& error,
                                 const std::string& details) {
  Log(BreadcrumbType::Error, component_id, "error",
      "error=" + error + (details.empty() ? "" : " | " + details));
}

void BreadcrumbLogger::LogScreenRender(const std::string& screen_id,
                                       const std::string& screen_dump) {
  if (config_.capture_screen_dumps) {
    Log(BreadcrumbType::ScreenRender, screen_id, "render", "", screen_dump);
  } else {
    Log(BreadcrumbType::ScreenRender, screen_id, "render");
  }
}

void BreadcrumbLogger::LogDialog(const std::string& dialog_id, bool opened,
                                  const std::string& details) {
  Log(opened ? BreadcrumbType::DialogOpen : BreadcrumbType::DialogClose,
      dialog_id, opened ? "open" : "close", details);
}

void BreadcrumbLogger::LogAction(const std::string& action,
                                  const std::string& component_id,
                                  const std::string& details) {
  Log(BreadcrumbType::Action, component_id, action, details);
}

void BreadcrumbLogger::LogDataUpdate(const std::string& source,
                                      const std::string& update_type,
                                      const std::string& details) {
  Log(BreadcrumbType::DataUpdate, source, update_type, details);
}

void BreadcrumbLogger::LogConfigChange(const std::string& config_key,
                                       const std::string& old_value,
                                       const std::string& new_value) {
  Log(BreadcrumbType::ConfigChange, "config", "change",
      "key=" + config_key + " old=" + old_value + " new=" + new_value);
}

void BreadcrumbLogger::Log(BreadcrumbType type, const std::string& component_id,
                           const std::string& action, const std::string& details,
                           const std::string& state_snapshot) {
  if (!config_.enabled) {
    return;
  }

  BreadcrumbEntry entry;
  entry.timestamp = std::chrono::system_clock::now();
  entry.type = type;
  entry.component_id = component_id;
  entry.action = action;
  entry.details = details;
  entry.state_snapshot = config_.capture_state_dumps ? state_snapshot : "";

  AddEntry(entry);
  FlushIfNeeded();
}

std::string BreadcrumbLogger::DumpState(const std::string& component_id,
                                         const std::string& state_data) {
  if (!config_.capture_state_dumps) {
    return "";
  }

  std::ostringstream oss;
  oss << "Component: " << component_id << "\n";
  oss << "State: " << state_data;
  return oss.str();
}

std::string BreadcrumbLogger::DumpScreen(const std::string& screen_buffer) {
  if (!config_.capture_screen_dumps) {
    return "";
  }
  return screen_buffer;
}

std::vector<BreadcrumbEntry> BreadcrumbLogger::GetBreadcrumbs(size_t max_count) const {
  std::lock_guard<std::mutex> lock(mutex_);

  size_t start = breadcrumbs_.size() > max_count
                 ? breadcrumbs_.size() - max_count
                 : 0;

  return std::vector<BreadcrumbEntry>(
      breadcrumbs_.begin() + start, breadcrumbs_.end());
}

std::vector<BreadcrumbEntry> BreadcrumbLogger::GetBreadcrumbsSince(
    std::chrono::system_clock::time_point since, size_t max_count) const {
  std::lock_guard<std::mutex> lock(mutex_);

  std::vector<BreadcrumbEntry> result;
  auto it = breadcrumbs_.rbegin();
  size_t count = 0;

  while (it != breadcrumbs_.rend() && count < max_count) {
    if (it->timestamp >= since) {
      result.push_back(*it);
      count++;
    } else {
      break;
    }
    ++it;
  }

  std::reverse(result.begin(), result.end());
  return result;
}

std::vector<BreadcrumbEntry> BreadcrumbLogger::GetBreadcrumbsByType(
    BreadcrumbType type, size_t max_count) const {
  std::lock_guard<std::mutex> lock(mutex_);

  std::vector<BreadcrumbEntry> result;
  auto it = breadcrumbs_.rbegin();
  size_t count = 0;

  while (it != breadcrumbs_.rend() && count < max_count) {
    if (it->type == type) {
      result.push_back(*it);
      count++;
    }
    ++it;
  }

  std::reverse(result.begin(), result.end());
  return result;
}

void BreadcrumbLogger::Clear() {
  std::lock_guard<std::mutex> lock(mutex_);
  breadcrumbs_.clear();
}

void BreadcrumbLogger::SetConfig(const Config& config) {
  std::lock_guard<std::mutex> lock(mutex_);
  config_ = config;
}

void BreadcrumbLogger::Flush() {
  if (!logger_) {
    return;
  }

  std::lock_guard<std::mutex> lock(mutex_);

  for (const auto& entry : breadcrumbs_) {
    logger_->debug("{}", entry.ToString());
  }

  logger_->flush();
  last_flush_ = std::chrono::system_clock::now();
}

void BreadcrumbLogger::AddEntry(BreadcrumbEntry entry) {
  std::lock_guard<std::mutex> lock(mutex_);

  breadcrumbs_.push_back(entry);
  RotateIfNeeded();

  if (logger_) {
    logger_->debug("{}", entry.ToString());
  }
}

void BreadcrumbLogger::RotateIfNeeded() {
  if (breadcrumbs_.size() > config_.max_entries) {
    // Remove oldest entries
    size_t to_remove = breadcrumbs_.size() - config_.max_entries;
    breadcrumbs_.erase(breadcrumbs_.begin(),
                       breadcrumbs_.begin() + to_remove);
  }
}

void BreadcrumbLogger::FlushIfNeeded() {
  auto now = std::chrono::system_clock::now();
  auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
      now - last_flush_);

  if (elapsed >= config_.flush_interval && logger_) {
    logger_->flush();
    last_flush_ = now;
  }
}

std::string BreadcrumbLogger::FormatTimestamp(
    const std::chrono::system_clock::time_point& tp) const {
  auto time_t = std::chrono::system_clock::to_time_t(tp);
  std::tm tm_buf;
  localtime_r(&time_t, &tm_buf);

  std::ostringstream oss;
  oss << std::put_time(&tm_buf, "%Y-%m-%d %H:%M:%S");
  return oss.str();
}

std::string BreadcrumbLogger::BreadcrumbTypeToString(BreadcrumbType type) const {
  switch (type) {
    case BreadcrumbType::UserInput: return "UserInput";
    case BreadcrumbType::Navigation: return "Navigation";
    case BreadcrumbType::StateChange: return "StateChange";
    case BreadcrumbType::Error: return "Error";
    case BreadcrumbType::ScreenRender: return "ScreenRender";
    case BreadcrumbType::DialogOpen: return "DialogOpen";
    case BreadcrumbType::DialogClose: return "DialogClose";
    case BreadcrumbType::Action: return "Action";
    case BreadcrumbType::DataUpdate: return "DataUpdate";
    case BreadcrumbType::ConfigChange: return "ConfigChange";
    default: return "Unknown";
  }
}

// ============================================================================
// Global Instance
// ============================================================================

namespace {
  std::unique_ptr<BreadcrumbLogger> g_breadcrumb_logger;
  std::mutex g_breadcrumb_mutex;
}

BreadcrumbLogger& GetBreadcrumbLogger() {
  std::lock_guard<std::mutex> lock(g_breadcrumb_mutex);
  if (!g_breadcrumb_logger) {
    g_breadcrumb_logger = std::make_unique<BreadcrumbLogger>();
  }
  return *g_breadcrumb_logger;
}

void InitializeBreadcrumbLogging(const BreadcrumbLogger::Config& config) {
  std::lock_guard<std::mutex> lock(g_breadcrumb_mutex);
  g_breadcrumb_logger = std::make_unique<BreadcrumbLogger>(config);
}

} // namespace tui
