// tui_breadcrumb.h - Breadcrumb logging for TUI debugging and testing
//
// Breadcrumb logging provides a trail of user interactions and application
// state changes to help debug issues and test the TUI application.
//
// Based on best practices from:
// - https://www.dantleech.com/blog/2025/05/11/debug-tui/
// - GUI testing patterns adapted for TUI
//
#pragma once

#include <string>
#include <vector>
#include <chrono>
#include <mutex>
#include <memory>
#include <sstream>
#include <iomanip>
#include <spdlog/spdlog.h>
#include <spdlog/logger.h>

namespace tui {

// ============================================================================
// Breadcrumb Entry Types
// ============================================================================

enum class BreadcrumbType {
  UserInput,        // Keyboard input, mouse click (if applicable)
  Navigation,       // Tab switch, menu navigation
  StateChange,      // Application state modification
  Error,            // Error occurrence
  ScreenRender,     // Screen buffer update
  DialogOpen,       // Dialog/modal opened
  DialogClose,      // Dialog/modal closed
  Action,           // User action (start strategy, cancel order, etc.)
  DataUpdate,       // Data provider update
  ConfigChange,     // Configuration modification
};

// ============================================================================
// Breadcrumb Entry
// ============================================================================

struct BreadcrumbEntry {
  std::chrono::system_clock::time_point timestamp;
  BreadcrumbType type;
  std::string component_id;      // Unique identifier for UI component
  std::string action;            // Action performed
  std::string details;           // Additional context
  std::string state_snapshot;     // JSON or text dump of relevant state

  // Convert to log string
  std::string ToString() const;

  // Convert to JSON for structured logging
  std::string ToJSON() const;
};

// ============================================================================
// Breadcrumb Logger
// ============================================================================

class BreadcrumbLogger {
public:
  // Configuration
  struct Config {
    bool enabled = true;
    size_t max_entries = 1000;           // Maximum entries in memory
    std::string log_file;                // Optional file for breadcrumb log
    bool log_to_console = false;        // Log to console
    bool log_to_file = true;             // Log to file
    bool capture_state_dumps = true;     // Include state snapshots
    bool capture_screen_dumps = false;   // Include screen buffer dumps (expensive)
    std::chrono::milliseconds flush_interval{1000};  // Flush interval
  };

  BreadcrumbLogger(const Config& config = Config{.enabled = true, .max_entries = 1000, .log_to_console = false, .log_to_file = true, .capture_state_dumps = true, .capture_screen_dumps = false, .flush_interval = std::chrono::milliseconds{1000}});
  ~BreadcrumbLogger();

  // Logging methods
  void LogInput(const std::string& component_id, const std::string& input,
                const std::string& details = "");
  void LogNavigation(const std::string& from, const std::string& to,
                     const std::string& details = "");
  void LogStateChange(const std::string& component_id, const std::string& change,
                      const std::string& state_dump = "");
  void LogError(const std::string& component_id, const std::string& error,
                const std::string& details = "");
  void LogScreenRender(const std::string& screen_id, const std::string& screen_dump = "");
  void LogDialog(const std::string& dialog_id, bool opened, const std::string& details = "");
  void LogAction(const std::string& action, const std::string& component_id,
                 const std::string& details = "");
  void LogDataUpdate(const std::string& source, const std::string& update_type,
                     const std::string& details = "");
  void LogConfigChange(const std::string& config_key, const std::string& old_value,
                       const std::string& new_value);

  // Generic log method
  void Log(BreadcrumbType type, const std::string& component_id,
           const std::string& action, const std::string& details = "",
           const std::string& state_snapshot = "");

  // State dumping
  std::string DumpState(const std::string& component_id, const std::string& state_data);
  std::string DumpScreen(const std::string& screen_buffer);

  // Retrieval
  std::vector<BreadcrumbEntry> GetBreadcrumbs(size_t max_count = 100) const;
  std::vector<BreadcrumbEntry> GetBreadcrumbsSince(
      std::chrono::system_clock::time_point since,
      size_t max_count = 100) const;
  std::vector<BreadcrumbEntry> GetBreadcrumbsByType(
      BreadcrumbType type, size_t max_count = 100) const;

  // Clear breadcrumbs
  void Clear();

  // Configuration
  void SetConfig(const Config& config);
  const Config& GetConfig() const { return config_; }

  // Flush logs to file
  void Flush();

private:
  Config config_;
  mutable std::mutex mutex_;
  std::vector<BreadcrumbEntry> breadcrumbs_;
  std::shared_ptr<spdlog::logger> logger_;
  std::chrono::system_clock::time_point last_flush_;

  void AddEntry(BreadcrumbEntry entry);
  void RotateIfNeeded();
  void FlushIfNeeded();
  std::string FormatTimestamp(const std::chrono::system_clock::time_point& tp) const;
  std::string BreadcrumbTypeToString(BreadcrumbType type) const;
};

// ============================================================================
// Global Breadcrumb Logger Instance
// ============================================================================

// Get the global breadcrumb logger instance
BreadcrumbLogger& GetBreadcrumbLogger();

// Initialize breadcrumb logging (call once at startup)
void InitializeBreadcrumbLogging(const BreadcrumbLogger::Config& config = BreadcrumbLogger::Config{});

// ============================================================================
// Convenience Macros
// ============================================================================

#define TUI_BREADCRUMB_INPUT(component_id, input, details) \
  tui::GetBreadcrumbLogger().LogInput(component_id, input, details)

#define TUI_BREADCRUMB_NAVIGATION(from, to, details) \
  tui::GetBreadcrumbLogger().LogNavigation(from, to, details)

#define TUI_BREADCRUMB_STATE_CHANGE(component_id, change, state_dump) \
  tui::GetBreadcrumbLogger().LogStateChange(component_id, change, state_dump)

#define TUI_BREADCRUMB_ERROR(component_id, error, details) \
  tui::GetBreadcrumbLogger().LogError(component_id, error, details)

#define TUI_BREADCRUMB_ACTION(action, component_id, details) \
  tui::GetBreadcrumbLogger().LogAction(action, component_id, details)

#define TUI_BREADCRUMB_DIALOG(dialog_id, opened, details) \
  tui::GetBreadcrumbLogger().LogDialog(dialog_id, opened, details)

} // namespace tui
