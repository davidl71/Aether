// tui_app.cpp - Main TUI application using FTXUI
#include "tui_data.h"
#include "tui_provider.h"
#include "tui_config.h"
#include "tui_breadcrumb.h"
#include <ftxui/component/component.hpp>
#include <ftxui/component/screen_interactive.hpp>
#include <ftxui/dom/elements.hpp>
#include <ftxui/screen/color.hpp>
#include <ftxui/screen/screen.hpp>
#include <spdlog/spdlog.h>
#include <iostream>
#include <iomanip>
#include <sstream>
#include <algorithm>
#include <cmath>
#include <thread>
#include <chrono>
#include <cstdlib>
#include <ctime>
#include <atomic>
#include <mutex>
#include <filesystem>
#include <csignal>
#include <fstream>
#include <nlohmann/json.hpp>

using namespace ftxui;

namespace tui {

class TUIApp {
public:
  TUIApp(std::unique_ptr<Provider> provider)
    : provider_(std::move(provider)), should_exit_(false) {
    // Initialize with empty snapshot for immediate rendering
    latest_snapshot_ = Snapshot{};

    // Load configuration
    std::string config_path = GetConfigPath();
    try {
      config_.LoadFromFile(config_path);
    } catch (const std::exception& e) {
      spdlog::warn("Failed to load config: {}, using defaults", e.what());
      config_ = TUIConfig::LoadDefault();
    }
  }

  // Get configuration file path
  static std::string GetConfigPath() {
    const char* home = std::getenv("HOME");
    if (home) {
      std::filesystem::path path(home);
      path /= ".config";
      path /= "ib_box_spread";
      std::filesystem::create_directories(path);
      path /= "tui_config.json";
      return path.string();
    }
    return "tui_config.json";
  }

  void Run() {
    if (!provider_) {
      spdlog::error("No data provider available");
      return;
    }

    // Start provider in background
    provider_->Start();

    // CRITICAL: Fetch initial snapshot immediately and verify it has data
    // This prevents blank screen on startup
    Snapshot initial_snapshot;
    int retry_count = 0;
    const int max_retries = 10;

    while (retry_count < max_retries) {
      initial_snapshot = provider_->GetSnapshot();
      if (initial_snapshot.generated_at.time_since_epoch().count() > 0) {
        break;  // Got valid snapshot
      }
      retry_count++;
      std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }

    {
      std::lock_guard<std::mutex> lock(snapshot_mutex_);
      latest_snapshot_ = initial_snapshot;
    }

    // Debug: verify snapshot has data
    if (initial_snapshot.generated_at.time_since_epoch().count() > 0) {
      spdlog::info("Initial snapshot loaded: {} symbols, {} positions, mode={}",
                   initial_snapshot.symbols.size(),
                   initial_snapshot.positions.size(),
                   initial_snapshot.mode);
    } else {
      spdlog::error("Initial snapshot is empty after {} retries - UI will show blank screen!",
                   max_retries);
      spdlog::error("Provider GetSnapshot() returned empty snapshot");
    }

    // Initialize screen with proper terminal size detection
    auto screen = ScreenInteractive::Fullscreen();
    screen_ptr_ = &screen;  // Store pointer for signal handlers

    // Set up signal handlers for Ctrl+C and Ctrl+Z
    // Use static pointer for signal handler access (signal handlers have restrictions)
    static ScreenInteractive* g_screen = &screen;
    static std::atomic<bool> g_interrupted{false};

    // Ctrl+C (SIGINT) - ensure clean exit
    // FTXUI should handle this, but we ensure it works
    std::signal(SIGINT, [](int sig) {
      g_interrupted.store(true);
      if (g_screen) {
        // Call exit closure from signal handler (should be safe)
        try {
          g_screen->ExitLoopClosure()();
        } catch (...) {
          // If that fails, just exit
          std::_Exit(0);
        }
      }
    });

    // Ctrl+Z (SIGTSTP) - allow normal suspend/resume
    // Restore default handler to allow normal suspend behavior
    std::signal(SIGTSTP, [](int sig) {
      // Restore default handler and raise signal for normal suspend
      std::signal(SIGTSTP, SIG_DFL);
      std::raise(SIGTSTP);
      // When resumed, restore our handler (if needed)
      std::signal(SIGTSTP, [](int) { /* default suspend */ });
    });

    // SIGWINCH - terminal window resize
    // FTXUI should handle this automatically, but we ensure it triggers a redraw
    std::signal(SIGWINCH, [](int sig) {
      if (g_screen) {
        // Trigger a redraw by posting a custom event
        // This ensures the UI re-renders with new terminal dimensions
        g_screen->PostEvent(Event::Custom);
      }
    });

    // Debug: Log terminal size if available
    #ifdef __APPLE__
    // On macOS, ensure we detect terminal size properly
    // FTXUI should handle this automatically, but log for debugging
    spdlog::debug("Screen initialized - FTXUI will detect terminal size");
    #endif

    // Build UI components - these render immediately with current snapshot
    int selected_tab = 0;
    auto tabs = RenderTabs(&selected_tab);
    auto dashboard = RenderDashboard();
    auto positions = RenderPositions();
    auto historic = RenderHistoric();
    auto orders = RenderOrders();
    auto alerts = RenderAlerts();

    // Multiscreen: Create 2-pane split layout (Dashboard | Positions)
    // Using ResizableSplitLeft for side-by-side panes
    // ResizableSplitLeft requires: (main_component, back_component, split_size_pointer)
    auto split_panes = ResizableSplitLeft(dashboard, positions, &split_size_);

    // Main component - renders immediately, updates asynchronously
    // Use a simple Renderer that returns Elements directly (not Container components)
    // This ensures proper terminal size detection and full-screen rendering
    auto component = Renderer([&] {
      // Show help modal if requested (highest priority)
      if (show_help_) {
        return RenderHelpModal();
      }

      // Show setup screen if requested
      if (show_setup_) {
        return RenderSetupScreen(selected_tab);
      }

      // Get snapshot atomically (non-blocking)
      Snapshot snap = GetSnapshot();

      // Always render something - never return empty
      // This ensures the screen is never blank
      auto header = RenderHeader(snap);

      // Multiscreen: Render tabs indicator (showing which panes are active)
      // Left pane: Dashboard, Right pane: Positions
      Elements tab_elements;
      tab_elements.push_back(text("Dashboard") | bold | color(Color::Cyan1) | bgcolor(Color::Blue1));
      tab_elements.push_back(text("  |  ") | dim);
      tab_elements.push_back(text("Current Positions") | bold | color(Color::Cyan1) | bgcolor(Color::Blue1));
      tab_elements.push_back(text("  (Historic | Orders | Alerts - use Tab to switch)") | dim);
      auto tabs_rendered = hbox(tab_elements);

      // Multiscreen: Render split panes (Dashboard | Positions)
      // The split_panes component is rendered automatically by the Renderer wrapper
      // We render both panes side-by-side manually for now
      auto dashboard_content = dashboard->Render();
      auto positions_content = positions->Render();
      Element content = hbox({
        dashboard_content | flex,
        separator(),
        positions_content | flex,
      });

      auto footer = RenderFooter(selected_tab);

      // Ensure the entire layout fills the screen
      // Use flex on content to fill available vertical space
      return vbox({
        header,
        separator(),
        tabs_rendered,
        separator(),
        content | flex,  // This expands to fill available space
        separator(),
        footer,
      });
    });

    // htop-like keyboard shortcuts
    component |= CatchEvent([&](Event event) {
      // Handle tab navigation with arrow keys
      if (!show_setup_) {
        if (event == Event::ArrowLeft || event == Event::Character('h')) {
          selected_tab = (selected_tab - 1 + 5) % 5;
          screen.PostEvent(Event::Custom);
          return true;
        }
        if (event == Event::ArrowRight || event == Event::Character('l')) {
          selected_tab = (selected_tab + 1) % 5;
          screen.PostEvent(Event::Custom);
          return true;
        }

        // Handle row navigation with up/down arrows (htop-style)
        if (event == Event::ArrowUp || event == Event::Character('k')) {
          auto snapshot = GetSnapshot();
          switch (selected_tab) {
            case 0:  // Dashboard
              if (!snapshot.symbols.empty()) {
                selected_dashboard_row_ = (selected_dashboard_row_ - 1 + snapshot.symbols.size()) % snapshot.symbols.size();
                screen.PostEvent(Event::Custom);
              }
              return true;
            case 1:  // Positions
              if (!snapshot.positions.empty()) {
                selected_positions_row_ = (selected_positions_row_ - 1 + snapshot.positions.size()) % snapshot.positions.size();
                screen.PostEvent(Event::Custom);
              }
              return true;
            case 3:  // Orders
              if (!snapshot.orders.empty()) {
                selected_orders_row_ = (selected_orders_row_ - 1 + snapshot.orders.size()) % snapshot.orders.size();
                screen.PostEvent(Event::Custom);
              }
              return true;
            case 4:  // Alerts
              if (!snapshot.alerts.empty()) {
                selected_alerts_row_ = (selected_alerts_row_ - 1 + snapshot.alerts.size()) % snapshot.alerts.size();
                screen.PostEvent(Event::Custom);
              }
              return true;
          }
        }
        if (event == Event::ArrowDown || event == Event::Character('j')) {
          auto snapshot = GetSnapshot();
          switch (selected_tab) {
            case 0:  // Dashboard
              if (!snapshot.symbols.empty()) {
                selected_dashboard_row_ = (selected_dashboard_row_ + 1) % snapshot.symbols.size();
                screen.PostEvent(Event::Custom);
              }
              return true;
            case 1:  // Positions
              if (!snapshot.positions.empty()) {
                selected_positions_row_ = (selected_positions_row_ + 1) % snapshot.positions.size();
                screen.PostEvent(Event::Custom);
              }
              return true;
            case 3:  // Orders
              if (!snapshot.orders.empty()) {
                selected_orders_row_ = (selected_orders_row_ + 1) % snapshot.orders.size();
                screen.PostEvent(Event::Custom);
              }
              return true;
            case 4:  // Alerts
              if (!snapshot.alerts.empty()) {
                selected_alerts_row_ = (selected_alerts_row_ + 1) % snapshot.alerts.size();
                screen.PostEvent(Event::Custom);
              }
              return true;
          }
        }
      }

      // Handle setup screen navigation
      if (show_setup_) {
        if (event == Event::F10) {
          // Save and exit setup
          SaveSetupConfig();
          show_setup_ = false;
          return true;
        }
        if (event == Event::Escape) {
          // Cancel setup
          show_setup_ = false;
          return true;
        }
        // Navigation within setup
        if (event == Event::ArrowUp) {
          setup_provider_selected_ = (setup_provider_selected_ - 1 + 5) % 5;
          return true;
        }
        if (event == Event::ArrowDown) {
          setup_provider_selected_ = (setup_provider_selected_ + 1) % 5;
          return true;
        }
        // Let other events through for future input fields
        return false;
      }

      // Main screen shortcuts
      // F-keys (htop style)
      if (event == Event::F1) {
        // Show help
        show_help_ = true;
        return true;
      }
      if (event == Event::F2) {
        // Setup/Configuration (htop-style)
        show_setup_ = true;
        return true;
      }
      if (event == Event::F3 || event == Event::Character('/')) {
        // Search/Filter
        ShowSearch(screen);
        return true;
      }
      if (event == Event::F4) {
        // Filter by...
        return true;
      }
      if (event == Event::F5) {
        // Tree view toggle in htop, we use for refresh
        screen.PostEvent(Event::Custom);  // Force refresh
        return true;
      }
      if (event == Event::F6) {
        // Sort by...
        ShowSortMenu(screen, selected_tab);
        return true;
      }
      if (event == Event::F9) {
        // Kill/Cancel (htop: kill process, us: cancel orders)
        ShowCancelOrders(screen);
        return true;
      }
      if (event == Event::F10) {
        // F10: Quit (unless in setup, where it saves)
        screen.ExitLoopClosure()();
        return true;
      }
      if (event == Event::Character('q') || event == Event::Character('Q')) {
        // Q: Quit
        screen.ExitLoopClosure()();
        return true;
      }

      // Tab navigation (htop uses F5/F6 for tree, we use Tab)
      if (event == Event::Tab) {
        selected_tab = (selected_tab + 1) % 5;
        return true;
      }
      if (event == Event::TabReverse) {
        selected_tab = (selected_tab - 1 + 5) % 5;
        return true;
      }

      // Arrow key navigation (htop-style)
      if (event == Event::ArrowUp || event == Event::ArrowDown) {
        // Navigate within current view
        // TODO: Implement row selection
        return false;  // Let component handle it
      }
      if (event == Event::PageUp || event == Event::PageDown) {
        // Scroll by page
        // TODO: Implement page scrolling
        return false;
      }
      if (event == Event::Home || event == Event::End) {
        // Jump to top/bottom
        // TODO: Implement jump to top/bottom
        return false;
      }
      if (event == Event::Return) {
        // Enter: Open details (htop: expand/collapse tree)
        // TODO: Implement detail view
        return false;
      }
      if (event == Event::Escape) {
        // Esc: Close dialog/cancel
        if (show_help_) {
          show_help_ = false;
          return true;
        }
        if (show_setup_) {
          show_setup_ = false;
          return true;
        }
        return false;
      }

      // Trading shortcuts (additional to htop)
      if (event == Event::Character('s') || event == Event::Character('S')) {
        // Start strategy
        return true;
      }
      if (event == Event::Character('t') || event == Event::Character('T')) {
        // Stop strategy
        return true;
      }
      if (event == Event::Character('k') || event == Event::Character('K')) {
        // Cancel orders (same as F9)
        ShowCancelOrders(screen);
        return true;
      }
      if (event == Event::Character('d') || event == Event::Character('D')) {
        // Toggle dry-run
        return true;
      }
      if (event == Event::Character('b') || event == Event::Character('B')) {
        // Buy combo
        return true;
      }
      if (event == Event::Character('h') || event == Event::Character('H')) {
        // 'h' also shows help (htop convention)
        show_help_ = true;
        return true;
      }
      if (event == Event::Character('?')) {
        // '?' also shows help (common TUI convention)
        show_help_ = true;
        return true;
      }

      return false;
    });

    // Background update thread - updates snapshot without blocking UI
    std::atomic<bool> running{true};
    std::thread update_thread([&]() {
      while (running.load() && provider_->IsRunning()) {
        // Fetch new snapshot (non-blocking)
        auto new_snapshot = provider_->GetSnapshot();

        // Update atomically
        {
          std::lock_guard<std::mutex> lock(snapshot_mutex_);
          latest_snapshot_ = new_snapshot;
        }

        // Trigger UI refresh (non-blocking event)
        screen.PostEvent(Event::Custom);

        // Update every 500ms for responsiveness
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
      }
    });

    // Force immediate render with initial data
    // Post multiple events to ensure UI updates
    screen.PostEvent(Event::Custom);
    screen.PostEvent(Event::Custom);

    spdlog::info("TUI screen loop starting - UI should be visible now");
    spdlog::info("Press Ctrl+C to exit, Ctrl+Z to suspend");

    // Run main loop - this handles input immediately
    // FTXUI's Loop() will handle Ctrl+C automatically
    try {
      screen.Loop(component);
    } catch (...) {
      spdlog::info("Screen loop interrupted");
    }

    // Cleanup
    running = false;
    screen_ptr_ = nullptr;  // Clear pointer
    if (update_thread.joinable()) {
      update_thread.join();
    }
    provider_->Stop();
  }

private:
  std::unique_ptr<Provider> provider_;
  Snapshot latest_snapshot_;
  std::mutex snapshot_mutex_;
  TUIConfig config_;
  bool show_setup_ = false;
  bool setup_saved_ = false;
  bool show_help_ = false;  // Help modal state
  std::atomic<bool> should_exit_{false};
  ScreenInteractive* screen_ptr_ = nullptr;  // For signal handlers to access

  // Row selection state for keyboard navigation
  int selected_dashboard_row_ = 0;
  int selected_positions_row_ = 0;
  int selected_orders_row_ = 0;
  int selected_alerts_row_ = 0;

  // Multiscreen: Split pane size (percentage or pixels)
  int split_size_ = 50;  // Default 50% split

  // Thread-safe snapshot getter
  Snapshot GetSnapshot() {
    std::lock_guard<std::mutex> lock(snapshot_mutex_);
    return latest_snapshot_;
  }

  // Load scenarios from box_spread_sample.json file
  void LoadScenarios(Snapshot& snapshot) {
    try {
      // Try multiple possible paths
      std::vector<std::string> candidate_paths = {
        "web/public/data/box_spread_sample.json",
        "../web/public/data/box_spread_sample.json",
        "../../web/public/data/box_spread_sample.json",
        "./data/box_spread_sample.json",
        std::string(std::getenv("HOME") ? std::getenv("HOME") : ".") + "/.config/ib_box_spread/box_spread_sample.json"
      };

      std::ifstream file;
      bool found = false;
      for (const auto& path : candidate_paths) {
        file.open(path);
        if (file.good()) {
          found = true;
          break;
        }
      }

      if (!found || !file.good()) {
        // No scenario file found - that's okay, scenarios are optional
        return;
      }

      nlohmann::json j;
      file >> j;
      file.close();

      // Parse scenario data
      if (j.contains("scenarios") && j["scenarios"].is_array()) {
        snapshot.scenarios.clear();
        for (const auto& scenario_json : j["scenarios"]) {
          BoxSpreadScenario scenario;
          from_json(scenario_json, scenario);
          // Set default option_style if not present
          if (scenario.option_style.empty()) {
            scenario.option_style = "European";  // Default to European
          }
          snapshot.scenarios.push_back(scenario);
        }
      }

      if (j.contains("underlying")) {
        snapshot.scenario_underlying = j["underlying"].get<std::string>();
      }

      if (j.contains("as_of")) {
        std::string as_of_str = j["as_of"].get<std::string>();
        // Parse ISO 8601 timestamp
        std::tm tm = {};
        std::istringstream ss(as_of_str);
        ss >> std::get_time(&tm, "%Y-%m-%dT%H:%M:%S");
        if (ss.fail()) {
          // Try alternative format
          ss.clear();
          ss.str(as_of_str);
          ss >> std::get_time(&tm, "%Y-%m-%d %H:%M:%S");
        }
        if (!ss.fail()) {
          snapshot.scenario_as_of = std::chrono::system_clock::from_time_t(std::mktime(&tm));
        } else {
          snapshot.scenario_as_of = std::chrono::system_clock::now();
        }
      } else {
        snapshot.scenario_as_of = std::chrono::system_clock::now();
      }

      // Calculate summary
      CalculateScenarioSummary(snapshot);
    } catch (const std::exception& e) {
      spdlog::debug("Failed to load scenarios: {}", e.what());
      // Don't fail if scenarios can't be loaded - they're optional
    }
  }

  // Calculate scenario summary statistics
  void CalculateScenarioSummary(Snapshot& snapshot) {
    auto& summary = snapshot.scenario_summary;
    summary.total_scenarios = static_cast<int>(snapshot.scenarios.size());

    if (snapshot.scenarios.empty()) {
      summary.avg_apr = 0.0;
      summary.probable_count = 0;
      summary.max_apr_scenario = BoxSpreadScenario{};
      return;
    }

    // Calculate average APR
    double total_apr = 0.0;
    int probable_count = 0;
    double max_apr = -1.0;
    BoxSpreadScenario max_scenario;

    for (const auto& scenario : snapshot.scenarios) {
      total_apr += scenario.annualized_return;
      if (scenario.fill_probability > 0.0) {
        probable_count++;
      }
      if (scenario.annualized_return > max_apr) {
        max_apr = scenario.annualized_return;
        max_scenario = scenario;
      }
    }

    summary.avg_apr = total_apr / static_cast<double>(summary.total_scenarios);
    summary.probable_count = probable_count;
    summary.max_apr_scenario = max_scenario;
  }

  // Render scenario summary section
  Element RenderScenarioSummary(const Snapshot& snapshot) {
    const auto& summary = snapshot.scenario_summary;

    if (summary.total_scenarios == 0) {
      return vbox({
        text("Box Spread Scenarios") | bold | color(Color::Cyan1),
        separator(),
        text("No scenarios available") | dim | center,
      }) | border;
    }

    // Format APR as percentage
    auto format_apr = [](double apr) -> std::string {
      std::ostringstream oss;
      oss << std::fixed << std::setprecision(2) << apr << "%";
      return oss.str();
    };

    Elements summary_elements;
    summary_elements.push_back(
      hbox({
        text("Total Scenarios:") | dim,
        text(" " + std::to_string(summary.total_scenarios)) | color(Color::Cyan1) | bold,
      })
    );
    summary_elements.push_back(
      hbox({
        text("Average APR:") | dim,
        text(" " + format_apr(summary.avg_apr)) | color(Color::Green1) | bold,
      })
    );
    summary_elements.push_back(
      hbox({
        text("Probable (fill_prob > 0):") | dim,
        text(" " + std::to_string(summary.probable_count)) | color(Color::Yellow1) | bold,
      })
    );

    if (summary.max_apr_scenario.width > 0.0) {
      std::ostringstream max_apr_oss;
      max_apr_oss << format_apr(summary.max_apr_scenario.annualized_return)
                  << " (" << snapshot.scenario_underlying
                  << " " << std::fixed << std::setprecision(2) << summary.max_apr_scenario.width << "pts)";
      summary_elements.push_back(
        hbox({
          text("Max APR:") | dim,
          text(" " + max_apr_oss.str()) | color(Color::Green1) | bold,
        })
      );
    }

    return vbox({
      text("Box Spread Scenarios") | bold | color(Color::Cyan1),
      separator(),
      vbox(summary_elements),
    }) | border;
  }

  // Render scenario table
  Element RenderScenarioTable(const Snapshot& snapshot) {
    if (snapshot.scenarios.empty()) {
      return text("No scenarios to display") | dim | center;
    }

    // Helper to format numbers
    auto format_num = [](double val, int width, int precision = 2) -> std::string {
      if (val == 0.0 && width < 8) return std::string(width, ' ') + "0";
      std::ostringstream oss;
      oss << std::fixed << std::setprecision(precision) << std::setw(width) << val;
      return oss.str();
    };

    auto format_str = [](const std::string& str, int width) -> std::string {
      std::string result = str;
      if (result.length() > width) {
        result = result.substr(0, width - 1) + "…";
      }
      return result + std::string(width - result.length(), ' ');
    };

    // Table header
    Elements table_rows;
    table_rows.push_back(hbox({
      text(format_str("Width", 10)) | bold | color(Color::Cyan1),
      text(format_str("Style", 10)) | bold | dim,
      text(format_str("Net Debit", 12)) | bold | dim,
      text(format_str("Profit", 10)) | bold | dim,
      text(format_str("ROI%", 10)) | bold | dim,
      text(format_str("APR%", 10)) | bold | dim,
      text(format_str("Fill Prob", 12)) | bold | dim,
    }));

    // Table rows
    for (const auto& scenario : snapshot.scenarios) {
      // Calculate profit (simplified: use mid_price - synthetic_bid as approximation)
      double profit = scenario.mid_price - scenario.synthetic_bid;
      double roi = (profit / scenario.synthetic_bid) * 100.0;

      // Color code APR (green=high, yellow=medium, red=low)
      Element apr_text = text(format_num(scenario.annualized_return, 10, 2) + "%");
      if (scenario.annualized_return > 10.0) {
        apr_text = apr_text | color(Color::Green1) | bold;
      } else if (scenario.annualized_return > 5.0) {
        apr_text = apr_text | color(Color::Yellow1);
      } else {
        apr_text = apr_text | color(Color::Red1);
      }

      // Color code fill probability
      Element prob_text = text(format_num(scenario.fill_probability, 12, 1) + "%");
      if (scenario.fill_probability > 50.0) {
        prob_text = prob_text | color(Color::Green1);
      } else if (scenario.fill_probability > 25.0) {
        prob_text = prob_text | color(Color::Yellow1);
      } else {
        prob_text = prob_text | dim;
      }

      table_rows.push_back(hbox({
        text(format_num(scenario.width, 10, 2)),
        text(format_str(scenario.option_style.empty() ? "European" : scenario.option_style, 10)),
        text(format_num(scenario.synthetic_bid, 12, 2)),
        text(format_num(profit, 10, 2)),
        text(format_num(roi, 10, 2)),
        apr_text,
        prob_text,
      }));
    }

    return vbox(table_rows);
  }

  Element RenderFooter(int selected_tab) {
    // htop-style footer: compact, color-coded keys
    std::vector<std::pair<std::string, std::string>> footer_keys;

    // htop-style footer: show relevant keys for current tab
    switch (selected_tab) {
      case 0:  // Dashboard
        footer_keys = {
          {"F1", "Help"}, {"F2", "Setup"}, {"F3", "Search"}, {"F4", "Filter"},
          {"F5", "Refresh"}, {"F6", "Sort"}, {"F10", "Quit"}
        };
        break;
      case 1:  // Positions
        footer_keys = {
          {"F1", "Help"}, {"F2", "Setup"}, {"F6", "Sort"}, {"F9", "Cancel"},
          {"Enter", "Details"}, {"F10", "Quit"}
        };
        break;
      case 2:  // Historic
        footer_keys = {
          {"F1", "Help"}, {"F2", "Setup"}, {"F6", "Sort"}, {"Enter", "Details"},
          {"F10", "Quit"}
        };
        break;
      case 3:  // Orders
        footer_keys = {
          {"F1", "Help"}, {"F2", "Setup"}, {"F9", "Cancel"}, {"Enter", "Details"},
          {"F10", "Quit"}
        };
        break;
      case 4:  // Alerts
        footer_keys = {
          {"F1", "Help"}, {"F2", "Setup"}, {"Enter", "Details"}, {"F10", "Quit"}
        };
        break;
      default:
        footer_keys = {{"F1", "Help"}, {"F10", "Quit"}};
    }

    // htop-style: compact footer with colored keys
    Elements footer_elements;
    for (size_t i = 0; i < footer_keys.size(); ++i) {
      if (i > 0) footer_elements.push_back(text("  ") | dim);
      // Key in yellow (htop style)
      footer_elements.push_back(text(footer_keys[i].first) | bold | color(Color::Yellow1));
      footer_elements.push_back(text("=") | dim);
      // Action in dim white
      footer_elements.push_back(text(footer_keys[i].second) | dim);
    }

    return hbox(footer_elements) | center;
  }

  // Helper to create htop-style visual bar
  Element CreateBar(double value, double max_value, int width, Color bar_color) {
    int filled = static_cast<int>((value / max_value) * width);
    filled = std::max(0, std::min(filled, width));
    // Use ASCII characters for compatibility
    std::string bar_str = std::string(filled, '#') + std::string(width - filled, '-');
    return text(bar_str) | color(bar_color);
  }

  Element RenderHeader(const Snapshot& snapshot) {
    std::string time_str = "--:--:--";
    if (snapshot.generated_at.time_since_epoch().count() > 0) {
      auto time_t = std::chrono::system_clock::to_time_t(snapshot.generated_at);
      std::tm tm_buf;
      localtime_r(&time_t, &tm_buf);
      std::ostringstream oss;
      oss << std::setfill('0') << std::setw(2) << tm_buf.tm_hour << ":"
          << std::setw(2) << tm_buf.tm_min << ":"
          << std::setw(2) << tm_buf.tm_sec;
      time_str = oss.str();
    }

    std::string mode = snapshot.mode.empty() ? "DRY-RUN" : snapshot.mode;
    std::string strategy = snapshot.strategy.empty() ? "STOPPED" : snapshot.strategy;
    std::string account = snapshot.account_id.empty() ? "--" : snapshot.account_id;

    // htop-style header: compact, color-coded, with visual indicators
    Element mode_text = text(mode);
    if (mode == "DRY-RUN") {
      mode_text = mode_text | color(Color::Yellow1) | bold;
    } else {
      mode_text = mode_text | color(Color::Red1) | bold;
    }

    Element strategy_text = text(strategy);
    if (strategy == "RUNNING") {
      strategy_text = strategy_text | color(Color::Green1) | bold;
    } else {
      strategy_text = strategy_text | color(Color::Red1) | bold;
    }

    // Status indicators (htop-style: green=OK, red=ERROR, yellow=WARN)
    auto status_indicator = [](bool ok) {
      return ok ? text("●") | color(Color::Green1) : text("●") | color(Color::Red1);
    };

    // Calculate metrics for visual bars (htop-style)
    int total_services = 4;
    int active_services = (snapshot.metrics.tws_ok ? 1 : 0) +
                          (snapshot.metrics.orats_ok ? 1 : 0) +
                          (snapshot.metrics.portal_ok ? 1 : 0) +
                          (snapshot.metrics.questdb_ok ? 1 : 0);
    double service_health = total_services > 0 ? (100.0 * active_services / total_services) : 0.0;

    // Format NetLiq (htop-style: compact numbers)
    std::string netliq_str = "$" + std::to_string(static_cast<int>(snapshot.metrics.net_liq));
    if (snapshot.metrics.net_liq >= 1000000) {
      netliq_str = "$" + std::to_string(static_cast<int>(snapshot.metrics.net_liq / 1000000)) + "M";
    } else if (snapshot.metrics.net_liq >= 1000) {
      netliq_str = "$" + std::to_string(static_cast<int>(snapshot.metrics.net_liq / 1000)) + "K";
    }

    // htop-style compact header (3 lines max)
    return vbox({
      // Line 1: Title, time, mode, strategy
      hbox({
        text("IB Box Spread") | bold | color(Color::Cyan1),
        text("  ") | dim,
        text(time_str) | color(Color::White) | bold,
        text("  ") | dim,
        text("Mode:") | dim,
        mode_text,
        text("  Strategy:") | dim,
        strategy_text,
        text("  Acc:") | dim,
        text(account) | color(Color::Cyan1),
      }),
      // Line 2: Service status with visual indicators
      hbox({
        status_indicator(snapshot.metrics.tws_ok),
        text("TWS") | dim,
        text("  "),
        status_indicator(snapshot.metrics.orats_ok),
        text("ORATS") | dim,
        text("  "),
        status_indicator(snapshot.metrics.portal_ok),
        text("Portal") | dim,
        text("  "),
        status_indicator(snapshot.metrics.questdb_ok),
        text("QuestDB") | dim,
        text("  "),
        text("NetLiq:") | dim,
        text(netliq_str) | color(Color::Green1) | bold,
      }),
      // Line 3: Health bar (htop-style)
      hbox({
        text("Health:") | dim,
        CreateBar(service_health, 100.0, 20, Color::Green1),
        text(" " + std::to_string(active_services) + "/" + std::to_string(total_services)) | dim,
      }),
    }) | border;
  }

  Component RenderTabs(int* selected) {
    std::vector<std::string> tab_names = {
      "Dashboard", "Current Positions", "Historic Positions",
      "Orders", "Alerts"
    };

    auto tabs = Menu(&tab_names, selected);
    return tabs;
  }

  Component RenderDashboard() {
    return Renderer([&] {
      auto snapshot = GetSnapshot();  // Non-blocking atomic read

      // Always render immediately, even with empty data
      if (snapshot.symbols.empty()) {
      return vbox({
        text("Dashboard") | bold | color(Color::Cyan1),
        separator(),
        text("No symbols tracked yet. Press F3 to search or 'A' to add one.") | dim,
        separator(),
        hbox({
          text("Positions: 0") | dim,
          text("  Orders: 0") | dim,
          text("  Alerts: 0") | dim,
        }),
      }) | border;
      }

      // Symbol table
      std::vector<std::vector<std::string>> table_data;
      table_data.push_back({"Symbol", "Last", "Bid", "Ask", "Spread", "ROI%"});

      for (const auto& sym : snapshot.symbols) {
        std::ostringstream oss;
        oss << std::fixed << std::setprecision(2);
        table_data.push_back({
          sym.symbol,
          sym.last > 0 ? std::to_string(sym.last) : "--",
          sym.bid > 0 ? std::to_string(sym.bid) : "--",
          sym.ask > 0 ? std::to_string(sym.ask) : "--",
          sym.spread > 0 ? std::to_string(sym.spread) : "--",
          sym.roi > 0 ? std::to_string(sym.roi) : "--",
        });
      }

      // Helper to format numbers with fixed width
      auto format_num = [](double val, int width, int precision = 2) -> std::string {
        if (val <= 0) return std::string(width, ' ') + "--";
        std::ostringstream oss;
        oss << std::fixed << std::setprecision(precision) << std::setw(width) << val;
        return oss.str();
      };

      auto format_str = [](const std::string& str, int width) -> std::string {
        std::string result = str;
        if (result.length() > width) {
          result = result.substr(0, width - 1) + "…";
        }
        return result + std::string(width - result.length(), ' ');
      };

      // htop-style: aligned table header
      Elements table_rows;
      table_rows.push_back(hbox({
        text(format_str("Symbol", 10)) | bold | color(Color::Cyan1),
        text(format_str("Last", 10)) | bold | dim,
        text(format_str("Bid", 10)) | bold | dim,
        text(format_str("Ask", 10)) | bold | dim,
        text(format_str("Spread", 10)) | bold | dim,
        text(format_str("ROI%", 10)) | bold | dim,
      }));

      int row_idx = 0;
      for (const auto& sym : snapshot.symbols) {
        // Color code ROI (green=positive, red=negative, yellow=neutral)
        Element roi_text = text(format_num(sym.roi, 10));
        if (sym.roi > 5.0) {
          roi_text = roi_text | color(Color::Green1) | bold;
        } else if (sym.roi > 0) {
          roi_text = roi_text | color(Color::Green1);
        } else if (sym.roi < 0) {
          roi_text = roi_text | color(Color::Red1);
        } else {
          roi_text = roi_text | dim;
        }

        // Highlight selected row (htop-style)
        bool is_selected = (row_idx == selected_dashboard_row_);
        Color symbol_color = Color::Cyan1;
        if (is_selected) {
          symbol_color = Color::Black;
        }

        Elements row_elements;
        if (is_selected) {
          row_elements.push_back(text(format_str(sym.symbol, 10)) | color(symbol_color) | bgcolor(Color::White) | bold);
          row_elements.push_back(text(format_num(sym.last, 10)) | bgcolor(Color::White) | bold);
          row_elements.push_back(text(format_num(sym.bid, 10)) | bgcolor(Color::White) | bold);
          row_elements.push_back(text(format_num(sym.ask, 10)) | bgcolor(Color::White) | bold);
          row_elements.push_back(text(format_num(sym.spread, 10)) | bgcolor(Color::White) | bold);
          row_elements.push_back(roi_text | bgcolor(Color::White) | bold);
        } else {
          row_elements.push_back(text(format_str(sym.symbol, 10)) | color(symbol_color));
          row_elements.push_back(text(format_num(sym.last, 10)));
          row_elements.push_back(text(format_num(sym.bid, 10)));
          row_elements.push_back(text(format_num(sym.ask, 10)));
          row_elements.push_back(text(format_num(sym.spread, 10)));
          row_elements.push_back(roi_text);
        }
        table_rows.push_back(hbox(row_elements));
        row_idx++;
      }

      // Clamp selected row to valid range
      if (selected_dashboard_row_ >= static_cast<int>(snapshot.symbols.size())) {
        selected_dashboard_row_ = std::max(0, static_cast<int>(snapshot.symbols.size()) - 1);
      }

      // Load scenarios if not already loaded
      Snapshot mutable_snapshot = snapshot;
      if (mutable_snapshot.scenarios.empty() && mutable_snapshot.scenario_summary.total_scenarios == 0) {
        LoadScenarios(mutable_snapshot);
        // Update snapshot with loaded scenarios
        {
          std::lock_guard<std::mutex> lock(snapshot_mutex_);
          latest_snapshot_.scenarios = mutable_snapshot.scenarios;
          latest_snapshot_.scenario_summary = mutable_snapshot.scenario_summary;
          latest_snapshot_.scenario_underlying = mutable_snapshot.scenario_underlying;
          latest_snapshot_.scenario_as_of = mutable_snapshot.scenario_as_of;
        }
        snapshot = mutable_snapshot;
      }

      // Build dashboard with symbols table and scenario explorer
      Elements dashboard_sections;

      // Symbol table section
      dashboard_sections.push_back(
        vbox({
          text("Symbols") | bold | color(Color::Cyan1),
          separator(),
          vbox(table_rows) | flex,
        }) | border
      );

      // Scenario explorer section (if scenarios available)
      if (!snapshot.scenarios.empty()) {
        dashboard_sections.push_back(separator());
        dashboard_sections.push_back(RenderScenarioSummary(snapshot));
        dashboard_sections.push_back(separator());
        dashboard_sections.push_back(
          vbox({
            text("Scenario Table") | bold | color(Color::Cyan1),
            separator(),
            RenderScenarioTable(snapshot) | flex,
          }) | border
        );
      }

      // htop-style: compact summary at bottom
      return vbox({
        vbox(dashboard_sections) | flex,
        separator(),
        hbox({
          text("Positions:") | dim,
          text(" " + std::to_string(snapshot.positions.size())) |
            (snapshot.positions.empty() ? dim : color(Color::Green1) | bold),
          text("  Orders:") | dim,
          text(" " + std::to_string(snapshot.orders.size())) |
            (snapshot.orders.empty() ? dim : color(Color::Yellow1) | bold),
          text("  Alerts:") | dim,
          text(" " + std::to_string(snapshot.alerts.size())) |
            (snapshot.alerts.empty() ? dim : color(Color::Cyan1) | bold),
          text("  Scenarios:") | dim,
          text(" " + std::to_string(snapshot.scenarios.size())) |
            (snapshot.scenarios.empty() ? dim : color(Color::Magenta1) | bold),
        }),
      });
    });
  }

  Component RenderPositions() {
    return Renderer([&] {
      auto snapshot = GetSnapshot();  // Non-blocking atomic read

      // Always render immediately, show placeholder if empty
      if (snapshot.positions.empty()) {
        return vbox({
          text("No current positions") | dim | center,
        });
      }

      // Helper to format numbers with fixed width
      auto format_num = [](double val, int width, int precision = 2) -> std::string {
        if (val == 0.0 && width < 8) return std::string(width, ' ') + "0";
        std::ostringstream oss;
        oss << std::fixed << std::setprecision(precision) << std::setw(width) << val;
        return oss.str();
      };

      auto format_str = [](const std::string& str, int width) -> std::string {
        std::string result = str;
        if (result.length() > width) {
          result = result.substr(0, width - 1) + "…";
        }
        return result + std::string(width - result.length(), ' ');
      };

      // htop-style: aligned color-coded table
      Elements table_rows;
      table_rows.push_back(hbox({
        text(format_str("Name", 20)) | bold | color(Color::Cyan1),
        text(format_str("Qty", 8)) | bold | dim,
        text(format_str("ROI%", 10)) | bold | dim,
        text(format_str("Mk/Tk", 10)) | bold | dim,
        text(format_str("Rebate", 10)) | bold | dim,
        text(format_str("Vega", 10)) | bold | dim,
        text(format_str("Theta", 10)) | bold | dim,
      }));

      int row_idx = 0;
      for (const auto& pos : snapshot.positions) {
        // Color code ROI
        Element roi_text = text(format_num(pos.roi, 10));
        if (pos.roi > 5.0) {
          roi_text = roi_text | color(Color::Green1) | bold;
        } else if (pos.roi > 0) {
          roi_text = roi_text | color(Color::Green1);
        } else {
          roi_text = roi_text | color(Color::Red1);
        }

        // Highlight selected row (htop-style)
        bool is_selected = (row_idx == selected_positions_row_);
        Color name_color = Color::Cyan1;
        if (is_selected) {
          name_color = Color::Black;
        }

        Elements row_elements;
        if (is_selected) {
          row_elements.push_back(text(format_str(pos.name, 20)) | color(name_color) | bgcolor(Color::White) | bold);
          row_elements.push_back(text(format_num(static_cast<double>(pos.quantity), 8, 0)) | bgcolor(Color::White) | bold);
          row_elements.push_back(roi_text | bgcolor(Color::White) | bold);
          row_elements.push_back(text(format_str(std::to_string(pos.maker_count) + "/" + std::to_string(pos.taker_count), 10)) |
              color(Color::Magenta1) | bgcolor(Color::White) | bold);
          row_elements.push_back(text(format_num(pos.rebate_estimate, 10)) | bgcolor(Color::White) | bold);
          row_elements.push_back(text(format_num(pos.vega, 10)) | bgcolor(Color::White) | bold);
          row_elements.push_back(text(format_num(pos.theta, 10)) | bgcolor(Color::White) | bold);
        } else {
          row_elements.push_back(text(format_str(pos.name, 20)) | color(name_color));
          row_elements.push_back(text(format_num(static_cast<double>(pos.quantity), 8, 0)));
          row_elements.push_back(roi_text);
          row_elements.push_back(text(format_str(std::to_string(pos.maker_count) + "/" + std::to_string(pos.taker_count), 10)) |
              color(Color::Magenta1));
          row_elements.push_back(text(format_num(pos.rebate_estimate, 10)));
          row_elements.push_back(text(format_num(pos.vega, 10)));
          row_elements.push_back(text(format_num(pos.theta, 10)));
        }
        table_rows.push_back(hbox(row_elements));
        row_idx++;
      }

      // Clamp selected row to valid range
      if (selected_positions_row_ >= static_cast<int>(snapshot.positions.size())) {
        selected_positions_row_ = std::max(0, static_cast<int>(snapshot.positions.size()) - 1);
      }

      return vbox(table_rows);
    });
  }

  Component RenderHistoric() {
    return Renderer([&] {
      auto snapshot = GetSnapshot();  // Non-blocking atomic read

      // Always render immediately
      if (snapshot.historic.empty()) {
        return vbox({
          text("No historic positions") | dim | center,
        });
      }

      // htop-style: simple count display
      return vbox({
        text("Historic Positions: " + std::to_string(snapshot.historic.size())) | dim | center,
      });
    });
  }

  Component RenderOrders() {
    return Renderer([&] {
      auto snapshot = GetSnapshot();  // Non-blocking atomic read

      // Always render immediately
      if (snapshot.orders.empty()) {
        return vbox({
          text("No recent orders") | dim | center,
        });
      }

      // htop-style: compact order list with selection
      Elements order_list;
      int row_idx = 0;
      for (const auto& order : snapshot.orders) {
        auto time_t = std::chrono::system_clock::to_time_t(order.timestamp);
        std::tm tm_buf;
        localtime_r(&time_t, &tm_buf);
        std::ostringstream oss;
        oss << std::setfill('0') << std::setw(2) << tm_buf.tm_hour << ":"
            << std::setw(2) << tm_buf.tm_min << ":"
            << std::setw(2) << tm_buf.tm_sec;

        // Color code by severity (htop-style)
        Element order_text = text(oss.str() + " " + order.text);
        if (order.severity == "success") {
          order_text = order_text | color(Color::Green1);
        } else if (order.severity == "error") {
          order_text = order_text | color(Color::Red1) | dim;
        } else if (order.severity == "warn" || order.severity == "warning") {
          order_text = order_text | color(Color::Yellow1);
        } else {
          order_text = order_text | color(Color::Cyan1);
        }

        // Highlight selected row (htop-style)
        bool is_selected = (row_idx == selected_orders_row_);
        if (is_selected) {
          order_text = order_text | bgcolor(Color::White) | color(Color::Black) | bold;
        }
        order_list.push_back(order_text);
        row_idx++;
      }

      // Clamp selected row to valid range
      if (selected_orders_row_ >= static_cast<int>(snapshot.orders.size())) {
        selected_orders_row_ = std::max(0, static_cast<int>(snapshot.orders.size()) - 1);
      }

      return vbox(order_list);
    });
  }

  Component RenderAlerts() {
    return Renderer([&] {
      auto snapshot = GetSnapshot();  // Non-blocking atomic read

      // Always render immediately
      if (snapshot.alerts.empty()) {
        return vbox({
          text("No alerts") | dim | center,
        });
      }

      // htop-style: compact alert list with color coding and selection
      Elements alert_list;
      int row_idx = 0;
      for (const auto& alert : snapshot.alerts) {
        auto time_t = std::chrono::system_clock::to_time_t(alert.timestamp);
        std::tm tm_buf;
        localtime_r(&time_t, &tm_buf);
        std::ostringstream oss;
        oss << std::setfill('0') << std::setw(2) << tm_buf.tm_hour << ":"
            << std::setw(2) << tm_buf.tm_min << ":"
            << std::setw(2) << tm_buf.tm_sec;

        // Color code by severity (htop-style)
        Element alert_text = text(oss.str() + " " + alert.text);
        if (alert.severity == "error") {
          alert_text = alert_text | color(Color::Red1) | bold;
        } else if (alert.severity == "warn" || alert.severity == "warning") {
          alert_text = alert_text | color(Color::Yellow1);
        } else if (alert.severity == "success") {
          alert_text = alert_text | color(Color::Green1);
        } else {
          alert_text = alert_text | color(Color::Cyan1);
        }

        // Highlight selected row (htop-style)
        bool is_selected = (row_idx == selected_alerts_row_);
        if (is_selected) {
          alert_text = alert_text | bgcolor(Color::White) | color(Color::Black) | bold;
        }
        alert_list.push_back(alert_text);
        row_idx++;
      }

      // Clamp selected row to valid range
      if (selected_alerts_row_ >= static_cast<int>(snapshot.alerts.size())) {
        selected_alerts_row_ = std::max(0, static_cast<int>(snapshot.alerts.size()) - 1);
      }

      return vbox(alert_list);
    });
  }

  // Help modal renderer
  Element RenderHelpModal() {
    // Help modal matching htop's help screen
    auto help_content = vbox({
      text("IB Box Spread TUI - Help") | bold | center | color(Color::Cyan1),
      separator(),
      text("Function Keys:"),
      text("  F1  Help              Show this help screen"),
      text("  F2  Setup             Configure display options"),
      text("  F3  Search            Search for a symbol/process"),
      text("  F4  Filter            Filter processes by criteria"),
      text("  F5  Refresh           Force refresh data"),
      text("  F6  Sort              Select sort column"),
      text("  F9  Cancel            Cancel orders"),
      text("  F10 Quit              Exit TUI"),
      separator(),
      text("Navigation:"),
      text("  ↑/↓ (k/j)            Move selection up/down"),
      text("  ←/→ (h/l)            Switch tabs"),
      text("  Tab                  Next tab"),
      text("  Shift+Tab            Previous tab"),
      text("  PgUp/PgDn            Scroll one page up/down"),
      text("  Home/End             Jump to top/bottom"),
      text("  Enter                Open details/expand"),
      text("  Esc                  Close dialog/cancel"),
      separator(),
      text("Trading Shortcuts:"),
      text("  S                    Start/resume strategy"),
      text("  T                    Stop strategy"),
      text("  K                    Cancel orders (same as F9)"),
      text("  D                    Toggle dry-run mode"),
      text("  B                    Buy combo order"),
      separator(),
      text("Help & Search:"),
      text("  ?                    Show help (this screen)"),
      text("  H                    Show help (same as F1/?)"),
      text("  /                    Search/filter"),
      separator(),
      text("Multiscreen Layout:"),
      text("  Dashboard | Positions displayed simultaneously"),
      text("  Mouse: Drag split divider to resize panes"),
      separator(),
      text("Press F1, H, ?, or Esc to close") | dim | center,
    }) | border;

    // Center the modal on screen
    return help_content | center | bgcolor(Color::Black);
  }

  // Legacy function kept for compatibility (now just sets flag)
  void ShowHelp(ScreenInteractive& screen) {
    show_help_ = true;
  }

  void ShowSearch(ScreenInteractive& screen) {
    // Search dialog - placeholder
    spdlog::info("Search: Type to filter (not fully implemented)");
  }

  void ShowSortMenu(ScreenInteractive& screen, int tab) {
    // Sort menu - placeholder
    spdlog::info("Sort: Select sort column for tab {}", tab);
  }

  void ShowCancelOrders(ScreenInteractive& screen) {
    // Cancel orders confirmation - placeholder
    spdlog::info("Cancel Orders: Are you sure? (not fully implemented)");
  }

  // Setup screen state (stored as member variables for persistence)
  int setup_provider_selected_ = 0;
  std::string setup_rest_endpoint_;
  std::string setup_update_interval_;
  std::string setup_refresh_rate_;
  std::string setup_rest_timeout_;
  std::string setup_nautilus_endpoint_;
  bool setup_show_colors_ = true;
  bool setup_show_footer_ = true;
  bool setup_rest_verify_ssl_ = false;

  // IBKR/TWS settings
  std::string setup_ibkr_host_;
  std::string setup_ibkr_port_;
  std::string setup_ibkr_client_id_;
  bool setup_ibkr_paper_trading_ = true;
  std::string setup_ibkr_account_id_;
  std::string setup_ibkr_connection_timeout_;
  bool setup_ibkr_auto_reconnect_ = true;
  std::string setup_ibkr_max_reconnect_;
  bool setup_ibkr_use_mock_ = false;

  // TA-Lib settings
  bool setup_use_ta_lib_ = false;

  Element RenderSetupScreen(int& selected_tab) {
    // Initialize setup state from config if first time
    static bool setup_initialized = false;
    if (!setup_initialized) {
      setup_provider_selected_ = 0;
      if (config_.provider_type == "mock") setup_provider_selected_ = 0;
      else if (config_.provider_type == "rest") setup_provider_selected_ = 1;
      else if (config_.provider_type == "ibkr_rest") setup_provider_selected_ = 2;
      else if (config_.provider_type == "livevol") setup_provider_selected_ = 3;
      else if (config_.provider_type == "nautilus") setup_provider_selected_ = 4;

      setup_rest_endpoint_ = config_.rest_endpoint;
      setup_update_interval_ = std::to_string(config_.update_interval.count());
      setup_refresh_rate_ = std::to_string(config_.refresh_rate_ms);
      setup_rest_timeout_ = std::to_string(config_.rest_timeout_ms);
      setup_nautilus_endpoint_ = config_.nautilus_endpoint;
      setup_show_colors_ = config_.show_colors;
      setup_show_footer_ = config_.show_footer;
      setup_rest_verify_ssl_ = config_.rest_verify_ssl;

      // IBKR/TWS settings
      setup_ibkr_host_ = config_.ibkr_host;
      setup_ibkr_port_ = std::to_string(config_.ibkr_port);
      setup_ibkr_client_id_ = std::to_string(config_.ibkr_client_id);
      setup_ibkr_paper_trading_ = config_.ibkr_paper_trading;
      setup_ibkr_account_id_ = config_.ibkr_account_id;
      setup_ibkr_connection_timeout_ = std::to_string(config_.ibkr_connection_timeout_ms);
      setup_ibkr_auto_reconnect_ = config_.ibkr_auto_reconnect;
      setup_ibkr_max_reconnect_ = std::to_string(config_.ibkr_max_reconnect_attempts);
      setup_ibkr_use_mock_ = config_.ibkr_use_mock;

      // TA-Lib settings
      setup_use_ta_lib_ = config_.use_ta_lib;

      setup_initialized = true;
    }

    std::vector<std::string> provider_options = {"Mock", "REST API", "IBKR REST API", "LiveVol API", "Nautilus Trader"};

    return vbox({
      text("Setup - Datafeed Configuration") | bold | center | color(Color::Cyan1),
      separator(),
      text("Data Provider Type: " + provider_options[setup_provider_selected_]) |
        (setup_provider_selected_ == 0 ? color(Color::Yellow1) :
         setup_provider_selected_ == 1 ? color(Color::Green1) :
         setup_provider_selected_ == 2 ? color(Color::Cyan1) :
         setup_provider_selected_ == 3 ? color(Color::Blue1) : color(Color::Magenta1)),
      text("  (Use ↑/↓ to change, Enter to select)") | dim,
      separator(),
      text("REST API Settings:"),
      text("  Endpoint: " + setup_rest_endpoint_) | dim,
      text("  Update Interval: " + setup_update_interval_ + " ms") | dim,
      text("  Timeout: " + setup_rest_timeout_ + " ms") | dim,
      text("  Verify SSL: " + std::string(setup_rest_verify_ssl_ ? "Yes" : "No")) | dim,
      separator(),
      text("IBKR Client Portal REST API Settings:") | bold | color(Color::Cyan1),
      text("  Base URL: " + config_.ibkr_rest_base_url) | dim,
      text("  Account ID: " + (config_.ibkr_rest_account_id.empty() ? "Auto-detect" : config_.ibkr_rest_account_id)) | dim,
      text("  Verify SSL: " + std::string(config_.ibkr_rest_verify_ssl ? "Yes" : "No")) | dim,
      text("  Timeout: " + std::to_string(config_.ibkr_rest_timeout_ms) + " ms") | dim,
      text("  Note: Requires Client Portal Gateway running") | dim,
      separator(),
      text("LiveVol API Settings:") | bold | color(Color::Blue1),
      text("  Base URL: " + config_.livevol_base_url) | dim,
      text("  Client ID: " + std::string(config_.livevol_client_id.empty() ? "Not set" : "***")) | dim,
      text("  Real-time Data: " + std::string(config_.livevol_use_real_time ? "Yes" : "No")) | dim,
      text("  Timeout: " + std::to_string(config_.livevol_timeout_ms) + " ms") | dim,
      text("  Note: Requires OAuth 2.0 credentials") | dim,
      separator(),
      text("Nautilus Trader Settings:"),
      text("  Endpoint: " + setup_nautilus_endpoint_) | dim,
      separator(),
      text("IBKR/TWS Socket Connection Settings:") | bold | color(Color::Cyan1),
      text("  Host: " + setup_ibkr_host_) | dim,
      text("  Port: " + setup_ibkr_port_ +
            (setup_ibkr_paper_trading_ ? " (Paper Trading)" : " (Live Trading)")) |
        (setup_ibkr_paper_trading_ ? color(Color::Yellow1) : color(Color::Red1)),
      text("  Client ID: " + setup_ibkr_client_id_) | dim,
      text("  Account ID: " + (setup_ibkr_account_id_.empty() ? "Auto-detect" : setup_ibkr_account_id_)) | dim,
      text("  Connection Timeout: " + setup_ibkr_connection_timeout_ + " ms") | dim,
      text("  Auto Reconnect: " + std::string(setup_ibkr_auto_reconnect_ ? "Yes" : "No")) | dim,
      text("  Max Reconnect Attempts: " + setup_ibkr_max_reconnect_) | dim,
      text("  Use Mock TWS: " + std::string(setup_ibkr_use_mock_ ? "Yes" : "No")) | dim,
      separator(),
      text("TA-Lib Technical Analysis:") | bold | color(Color::Magenta1),
      text("  Enabled: " + std::string(setup_use_ta_lib_ ? "Yes" : "No")) |
        (setup_use_ta_lib_ ? color(Color::Green1) : color(Color::Red1)),
      text("  Indicators: RSI, MACD, BBANDS (when enabled)") | dim,
      separator(),
      text("Display Settings:"),
      text("  Show Colors: " + std::string(setup_show_colors_ ? "Yes" : "No")) | dim,
      text("  Show Footer: " + std::string(setup_show_footer_ ? "Yes" : "No")) | dim,
      text("  Refresh Rate: " + setup_refresh_rate_ + " ms") | dim,
      separator(),
      text("Note: Full interactive setup requires form components") | dim | center,
      text("Current values shown above. Press F10 to save, Esc to cancel.") | dim | center,
      separator(),
      text("F10=Save  Esc=Cancel") | dim | center,
    }) | border;
  }

  void SaveSetupConfig() {
    // Save current setup state to config
    TUIConfig edit_config = config_;

    std::vector<std::string> provider_options = {"Mock", "REST API", "IBKR REST API", "LiveVol API", "Nautilus Trader"};
    edit_config.provider_type = provider_options[setup_provider_selected_];
    std::transform(edit_config.provider_type.begin(), edit_config.provider_type.end(),
                  edit_config.provider_type.begin(), ::tolower);
    if (edit_config.provider_type == "rest api") edit_config.provider_type = "rest";
    if (edit_config.provider_type == "ibkr rest api") edit_config.provider_type = "ibkr_rest";
    if (edit_config.provider_type == "livevol api") edit_config.provider_type = "livevol";
    if (edit_config.provider_type == "nautilus trader") edit_config.provider_type = "nautilus";

    edit_config.rest_endpoint = setup_rest_endpoint_;
    try {
      edit_config.update_interval = std::chrono::milliseconds(std::stoi(setup_update_interval_));
    } catch (...) {
      spdlog::warn("Invalid update interval, using default");
    }
    try {
      edit_config.refresh_rate_ms = std::stoi(setup_refresh_rate_);
    } catch (...) {
      spdlog::warn("Invalid refresh rate, using default");
    }
    try {
      edit_config.rest_timeout_ms = std::stoi(setup_rest_timeout_);
    } catch (...) {
      spdlog::warn("Invalid REST timeout, using default");
    }
    edit_config.nautilus_endpoint = setup_nautilus_endpoint_;
    edit_config.show_colors = setup_show_colors_;
    edit_config.show_footer = setup_show_footer_;
    edit_config.rest_verify_ssl = setup_rest_verify_ssl_;

    // IBKR/TWS settings
    edit_config.ibkr_host = setup_ibkr_host_;
    try {
      edit_config.ibkr_port = std::stoi(setup_ibkr_port_);
      // Auto-update port based on paper trading setting
      if (setup_ibkr_paper_trading_) {
        edit_config.ibkr_port = 7497;  // Paper trading
      } else {
        edit_config.ibkr_port = 7496;  // Live trading
      }
    } catch (...) {
      spdlog::warn("Invalid IBKR port, using default");
    }
    try {
      edit_config.ibkr_client_id = std::stoi(setup_ibkr_client_id_);
    } catch (...) {
      spdlog::warn("Invalid IBKR client ID, using default");
    }
    edit_config.ibkr_paper_trading = setup_ibkr_paper_trading_;
    edit_config.ibkr_account_id = setup_ibkr_account_id_;
    try {
      edit_config.ibkr_connection_timeout_ms = std::stoi(setup_ibkr_connection_timeout_);
    } catch (...) {
      spdlog::warn("Invalid IBKR connection timeout, using default");
    }
    edit_config.ibkr_auto_reconnect = setup_ibkr_auto_reconnect_;
    try {
      edit_config.ibkr_max_reconnect_attempts = std::stoi(setup_ibkr_max_reconnect_);
    } catch (...) {
      spdlog::warn("Invalid IBKR max reconnect attempts, using default");
    }
    edit_config.ibkr_use_mock = setup_ibkr_use_mock_;

    // TA-Lib settings
    edit_config.use_ta_lib = setup_use_ta_lib_;

    try {
      edit_config.SaveToFile(GetConfigPath());
      config_ = edit_config;
      setup_saved_ = true;
      spdlog::info("Configuration saved to {}", GetConfigPath());
      spdlog::info("Note: Restart TUI to apply provider and IBKR connection changes");
    } catch (const std::exception& e) {
      spdlog::error("Failed to save config: {}", e.what());
    }
  }
};

} // namespace tui

int main(int argc, char* argv[]) {
  using namespace tui;

  spdlog::set_level(spdlog::level::info);

  // Initialize breadcrumb logging
  BreadcrumbLogger::Config breadcrumb_config;
  breadcrumb_config.enabled = true;
  const char* breadcrumb_file = std::getenv("TUI_BREADCRUMB_LOG_FILE");
  breadcrumb_config.log_file = breadcrumb_file ? breadcrumb_file : "/tmp/ib_box_spread_tui_breadcrumbs.log";
  breadcrumb_config.log_to_console = std::getenv("TUI_BREADCRUMB_CONSOLE") != nullptr;
  breadcrumb_config.log_to_file = true;
  breadcrumb_config.max_entries = 1000;
  breadcrumb_config.capture_state_dumps = true;
  breadcrumb_config.capture_screen_dumps = false;  // Expensive, enable only when debugging rendering
  InitializeBreadcrumbLogging(breadcrumb_config);

  GetBreadcrumbLogger().LogAction("application_start", "main", "TUI application starting");

  // Load configuration first
  TUIConfig config;
  try {
    config.LoadFromFile(TUIApp::GetConfigPath());
  } catch (const std::exception& e) {
    spdlog::warn("Failed to load config: {}, using defaults", e.what());
    config = TUIConfig::LoadDefault();
  }

  // Determine provider type (config takes precedence, then environment, then default)
  std::string backend = config.provider_type;
  if (backend.empty()) {
    if (const char* env = std::getenv("TUI_BACKEND")) {
      backend = env;
    } else {
      backend = "mock";
    }
  }

  std::unique_ptr<Provider> provider;

  if (backend == "mock" || backend.empty()) {
    provider = std::make_unique<MockProvider>();
    spdlog::info("Using Mock data provider");
  } else if (backend == "rest") {
    std::string endpoint = config.rest_endpoint;
    if (endpoint.empty()) {
      endpoint = "http://localhost:8080/api/snapshot";
    }
    if (const char* env = std::getenv("TUI_API_URL")) {
      endpoint = env;  // Environment variable overrides config
    }
    provider = std::make_unique<RestProvider>(endpoint, config.update_interval);
    spdlog::info("Using REST provider: {} (interval: {}ms)", endpoint, config.update_interval.count());
  } else if (backend == "ibkr_rest") {
    std::string base_url = config.ibkr_rest_base_url;
    if (base_url.empty()) {
      base_url = "https://localhost:5000/v1/portal";
    }
    provider = std::make_unique<IBKRRestProvider>(
        base_url,
        config.ibkr_rest_account_id,
        config.ibkr_rest_verify_ssl,
        config.update_interval
    );
    spdlog::info("Using IBKR Client Portal REST API: {} (interval: {}ms)", base_url, config.update_interval.count());
    spdlog::info("Note: Ensure Client Portal Gateway is running");
  } else if (backend == "livevol") {
    std::string base_url = config.livevol_base_url;
    if (base_url.empty()) {
      base_url = "https://api.livevol.com/v1";
    }
    provider = std::make_unique<LiveVolProvider>(
        base_url,
        config.livevol_client_id,
        config.livevol_client_secret,
        config.livevol_use_real_time,
        config.update_interval
    );
    spdlog::info("Using LiveVol API: {} (interval: {}ms, real-time: {})",
                 base_url, config.update_interval.count(), config.livevol_use_real_time);
    spdlog::info("Note: Requires OAuth 2.0 credentials (client_id, client_secret)");
  } else if (backend == "nautilus") {
    // TODO: Implement Nautilus provider
    spdlog::warn("Nautilus provider not yet implemented, falling back to mock");
    provider = std::make_unique<MockProvider>();
  } else if (backend == "file") {
    std::string file_path = std::getenv("TUI_SNAPSHOT_FILE")
      ? std::getenv("TUI_SNAPSHOT_FILE")
      : std::string("web/public/data/snapshot.json");
    auto interval = config.update_interval.count() > 0 ? config.update_interval : std::chrono::milliseconds(1000);
    provider = std::make_unique<FileProvider>(file_path, interval);
    spdlog::info("Using File provider: {} (interval: {}ms)", file_path, interval.count());
  } else if (backend == "websocket") {
    std::string ws_url = std::getenv("TUI_WS_URL")
      ? std::getenv("TUI_WS_URL")
      : std::string("ws://localhost:8000/ws");
    std::string fallback_file = std::getenv("TUI_SNAPSHOT_FILE")
      ? std::getenv("TUI_SNAPSHOT_FILE")
      : std::string("web/public/data/snapshot.json");
    auto reconnect_interval = config.update_interval.count() > 0 ? config.update_interval : std::chrono::milliseconds(3000);
    provider = std::make_unique<WebSocketProvider>(ws_url, fallback_file, reconnect_interval);
    spdlog::info("Using WebSocket provider: {} (fallback: {})", ws_url, fallback_file);
    spdlog::info("Note: WebSocket connection not yet implemented, will use file polling fallback");
  } else {
    spdlog::warn("Unknown backend: {}, falling back to mock", backend);
    provider = std::make_unique<MockProvider>();
  }

  TUIApp app(std::move(provider));
  app.Run();

  return 0;
}
