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

using namespace ftxui;

namespace tui {

class TUIApp {
public:
  TUIApp(std::unique_ptr<Provider> provider)
    : provider_(std::move(provider)) {
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

    auto screen = ScreenInteractive::Fullscreen();

    // Build UI components - these render immediately with current snapshot
    int selected_tab = 0;
    auto tabs = RenderTabs(&selected_tab);
    auto dashboard = RenderDashboard();
    auto positions = RenderPositions();
    auto historic = RenderHistoric();
    auto orders = RenderOrders();
    auto alerts = RenderAlerts();

    // Tab container
    auto tab_container = Container::Tab({
      dashboard,
      positions,
      historic,
      orders,
      alerts,
    }, &selected_tab);

    // Main layout
    auto layout = Container::Vertical({
      tabs,
      tab_container,
    });

    // Main component - renders immediately, updates asynchronously
    auto component = Renderer(layout, [&] {
      // Show setup screen if requested
      if (show_setup_) {
        return RenderSetupScreen(selected_tab);
      }

      // Get snapshot atomically (non-blocking)
      Snapshot snap = GetSnapshot();

      return vbox({
        RenderHeader(snap),
        separator(),
        tabs->Render(),
        separator(),
        tab_container->Render() | flex,
        separator(),
        RenderFooter(selected_tab),  // htop-style footer
      });
    });

    // htop-like keyboard shortcuts
    component |= CatchEvent([&](Event event) {
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
        ShowHelp(screen);
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
        // TODO: Implement dialog closing
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
        ShowHelp(screen);
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

    // Start with immediate render
    screen.PostEvent(Event::Custom);

    // Run main loop - this handles input immediately
    screen.Loop(component);

    // Cleanup
    running = false;
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

  // Thread-safe snapshot getter
  Snapshot GetSnapshot() {
    std::lock_guard<std::mutex> lock(snapshot_mutex_);
    return latest_snapshot_;
  }

  Element RenderFooter(int selected_tab) {
    // htop-style footer showing available keys
    std::vector<std::string> footer_keys;

    // htop-style footer: show relevant keys for current tab
    switch (selected_tab) {
      case 0:  // Dashboard
        footer_keys = {"F1", "F2", "F3", "F4", "F5", "F6", "F10"};
        break;
      case 1:  // Positions
        footer_keys = {"F1", "F2", "F6", "F9", "Enter", "F10"};
        break;
      case 2:  // Historic
        footer_keys = {"F1", "F2", "F6", "Enter", "F10"};
        break;
      case 3:  // Orders
        footer_keys = {"F1", "F2", "F9", "Enter", "F10"};
        break;
      case 4:  // Alerts
        footer_keys = {"F1", "F2", "Enter", "F10"};
        break;
      default:
        footer_keys = {"F1", "F10"};
    }

    Elements footer_elements;
    for (size_t i = 0; i < footer_keys.size(); ++i) {
      if (i > 0) footer_elements.push_back(text("  "));
      footer_elements.push_back(text(footer_keys[i]) | bold | color(Color::Yellow1));
      footer_elements.push_back(text("="));

      std::string action;
      if (footer_keys[i] == "F1") action = "Help";
      else if (footer_keys[i] == "F2") action = "Setup";
      else if (footer_keys[i] == "F3") action = "Search";
      else if (footer_keys[i] == "F4") action = "Filter";
      else if (footer_keys[i] == "F5") action = "Tree";  // htop: Tree, we: Refresh
      else if (footer_keys[i] == "F6") action = "SortBy";
      else if (footer_keys[i] == "F9") action = "Kill";  // htop: Kill, we: Cancel
      else if (footer_keys[i] == "F10") action = "Quit";
      else if (footer_keys[i] == "Enter") action = "Details";

      footer_elements.push_back(text(action) | dim);
    }

    return hbox(footer_elements) | center;
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

    // htop-style header: color-coded status
    Element mode_text = text(mode);
    if (mode == "DRY-RUN") {
      mode_text = mode_text | color(Color::Yellow1);
    } else {
      mode_text = mode_text | color(Color::Red1) | bold;
    }

    Element strategy_text = text(strategy);
    if (strategy == "RUNNING") {
      strategy_text = strategy_text | color(Color::Green1);
    } else {
      strategy_text = strategy_text | color(Color::Red1);
    }

    // Status indicators (htop-style: green=OK, red=ERROR)
    auto status_color = [](bool ok) {
      return ok ? color(Color::Green1) : color(Color::Red1);
    };

    return vbox({
      hbox({
        text("IB Box Spread Terminal") | bold | color(Color::Cyan1),
        text("    Time: ") | dim,
        text(time_str) | color(Color::White),
        text("    Source: ") | dim,
        text("Mock") | color(Color::Yellow1),
      }),
      hbox({
        text("Mode: "),
        mode_text,
        text("   Strategy: "),
        strategy_text,
        text("   Account: ") | dim,
        text(account) | color(Color::Cyan1),
      }),
      hbox({
        text("TWS: ") | status_color(snapshot.metrics.tws_ok),
        text(snapshot.metrics.tws_ok ? "OK" : "ERROR") | status_color(snapshot.metrics.tws_ok),
        text("   ORATS: ") | status_color(snapshot.metrics.orats_ok),
        text(snapshot.metrics.orats_ok ? "Enabled" : "Disabled") | status_color(snapshot.metrics.orats_ok),
        text("   Portal: ") | status_color(snapshot.metrics.portal_ok),
        text(snapshot.metrics.portal_ok ? "OK" : "ERROR") | status_color(snapshot.metrics.portal_ok),
        text("   QuestDB: ") | status_color(snapshot.metrics.questdb_ok),
        text(snapshot.metrics.questdb_ok ? "OK" : "ERROR") | status_color(snapshot.metrics.questdb_ok),
        text("   NetLiq: ") | dim,
        text("$" + std::to_string(static_cast<int>(snapshot.metrics.net_liq))) |
          color(Color::Green1),
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

      // htop-style: color code ROI values
      Elements table_rows;
      table_rows.push_back(hbox({
        text("Symbol") | bold,
        text("  Last") | bold,
        text("  Bid") | bold,
        text("  Ask") | bold,
        text("  Spread") | bold,
        text("  ROI%") | bold,
      }));

      for (const auto& sym : snapshot.symbols) {
        std::ostringstream oss;
        oss << std::fixed << std::setprecision(2);

        // Color code ROI (green=positive, red=negative, yellow=neutral)
        Element roi_text = text("  " + (sym.roi > 0 ? std::to_string(sym.roi) : "--"));
        if (sym.roi > 5.0) {
          roi_text = roi_text | color(Color::Green1) | bold;
        } else if (sym.roi > 0) {
          roi_text = roi_text | color(Color::Green1);
        } else if (sym.roi < 0) {
          roi_text = roi_text | color(Color::Red1);
        } else {
          roi_text = roi_text | dim;
        }

        table_rows.push_back(hbox({
          text(sym.symbol) | color(Color::Cyan1),
          text("  " + (sym.last > 0 ? std::to_string(sym.last) : "--")),
          text("  " + (sym.bid > 0 ? std::to_string(sym.bid) : "--")),
          text("  " + (sym.ask > 0 ? std::to_string(sym.ask) : "--")),
          text("  " + (sym.spread > 0 ? std::to_string(sym.spread) : "--")),
          roi_text,
        }));
      }

      return vbox({
        text("Dashboard") | bold | color(Color::Cyan1),
        separator(),
        vbox(table_rows),
        separator(),
        hbox({
          text("Positions: " + std::to_string(snapshot.positions.size())) |
            (snapshot.positions.empty() ? dim : color(Color::Green1)),
          text("  Orders: " + std::to_string(snapshot.orders.size())) |
            (snapshot.orders.empty() ? dim : color(Color::Yellow1)),
          text("  Alerts: " + std::to_string(snapshot.alerts.size())) |
            (snapshot.alerts.empty() ? dim : color(Color::Cyan1)),
        }),
      }) | border;
    });
  }

  Component RenderPositions() {
    return Renderer([&] {
      auto snapshot = GetSnapshot();  // Non-blocking atomic read

      // Always render immediately, show placeholder if empty
      if (snapshot.positions.empty()) {
        return vbox({
          text("Current Positions") | bold,
          separator(),
          text("No current positions") | dim,
        }) | border;
      }

      // htop-style: color-coded table with selectable rows
      Elements table_rows;
      table_rows.push_back(hbox({
        text("Name") | bold,
        text("  Qty") | bold,
        text("  ROI%") | bold,
        text("  Mk/Tk") | bold,
        text("  Rebate") | bold,
        text("  Vega") | bold,
        text("  Theta") | bold,
      }));

      for (const auto& pos : snapshot.positions) {
        // Color code ROI
        Element roi_text = text("  " + std::to_string(pos.roi));
        if (pos.roi > 5.0) {
          roi_text = roi_text | color(Color::Green1) | bold;
        } else if (pos.roi > 0) {
          roi_text = roi_text | color(Color::Green1);
        } else {
          roi_text = roi_text | color(Color::Red1);
        }

        table_rows.push_back(hbox({
          text(pos.name) | color(Color::Cyan1),
          text("  " + std::to_string(pos.quantity)),
          roi_text,
          text("  " + std::to_string(pos.maker_count) + "/" + std::to_string(pos.taker_count)) |
            color(Color::Magenta1),
          text("  " + std::to_string(pos.rebate_estimate)),
          text("  " + std::to_string(pos.vega)),
          text("  " + std::to_string(pos.theta)),
        }));
      }

      return vbox({
        text("Current Positions") | bold | color(Color::Cyan1),
        separator(),
        vbox(table_rows),
      }) | border;
    });
  }

  Component RenderHistoric() {
    return Renderer([&] {
      auto snapshot = GetSnapshot();  // Non-blocking atomic read

      // Always render immediately
      if (snapshot.historic.empty()) {
        return vbox({
          text("Historic Positions") | bold,
          separator(),
          text("No historic positions") | dim,
        }) | border;
      }

      return text("Historic Positions: " + std::to_string(snapshot.historic.size())) | border;
    });
  }

  Component RenderOrders() {
    return Renderer([&] {
      auto snapshot = GetSnapshot();  // Non-blocking atomic read

      // Always render immediately
      if (snapshot.orders.empty()) {
        return vbox({
          text("Recent Orders") | bold,
          separator(),
          text("No recent orders") | dim,
        }) | border;
      }

      Elements order_list;
      for (const auto& order : snapshot.orders) {
        auto time_t = std::chrono::system_clock::to_time_t(order.timestamp);
        std::tm tm_buf;
        localtime_r(&time_t, &tm_buf);
        std::ostringstream oss;
        oss << std::setfill('0') << std::setw(2) << tm_buf.tm_hour << ":"
            << std::setw(2) << tm_buf.tm_min << ":"
            << std::setw(2) << tm_buf.tm_sec;
        order_list.push_back(text(oss.str() + " " + order.text));
      }

      return vbox({
        text("Recent Orders"),
        separator(),
        vbox(order_list),
      }) | border;
    });
  }

  Component RenderAlerts() {
    return Renderer([&] {
      auto snapshot = GetSnapshot();  // Non-blocking atomic read

      // Always render immediately
      if (snapshot.alerts.empty()) {
        return vbox({
          text("Alerts") | bold,
          separator(),
          text("No alerts") | dim,
        }) | border;
      }

      Elements alert_list;
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
        if (alert.severity == "error" || alert.severity == "critical") {
          alert_text = alert_text | color(Color::Red1);
        } else if (alert.severity == "warn" || alert.severity == "warning") {
          alert_text = alert_text | color(Color::Yellow1);
        } else if (alert.severity == "success") {
          alert_text = alert_text | color(Color::Green1);
        } else {
          alert_text = alert_text | color(Color::Cyan1);
        }
        alert_list.push_back(alert_text);
      }

      return vbox({
        text("Alerts"),
        separator(),
        vbox(alert_list),
      }) | border;
    });
  }

  // htop-style helper functions
  void ShowHelp(ScreenInteractive& screen) {
    // Help modal matching htop's help screen
    auto help_text = vbox({
      text("htop - Help") | bold | center | color(Color::Cyan1),
      separator(),
      text("F1  Help              Show this help screen"),
      text("F2  Setup             Configure display options"),
      text("F3  Search            Search for a symbol/process"),
      text("F4  Filter            Filter processes by criteria"),
      text("F5  Tree              Toggle tree view (refresh for us)"),
      text("F6  SortBy            Select sort column"),
      text("F7  Nice-             Decrease priority (not applicable)"),
      text("F8  Nice+             Increase priority (not applicable)"),
      text("F9  Kill              Cancel orders (kill process in htop)"),
      text("F10 Quit              Exit htop"),
      separator(),
      text("Navigation (htop-style):"),
      text("  ↑/↓                 Move selection up/down"),
      text("  PgUp/PgDn           Scroll one page up/down"),
      text("  Home/End            Jump to top/bottom"),
      text("  Tab/Shift+Tab       Switch tabs (our addition)"),
      text("  Enter               Open details/expand"),
      text("  Esc                 Close dialog/cancel"),
      separator(),
      text("Trading Shortcuts (additional):"),
      text("  S                   Start/resume strategy"),
      text("  T                   Stop strategy"),
      text("  K                   Cancel orders (same as F9)"),
      text("  D                   Toggle dry-run mode"),
      text("  B                   Buy combo order"),
      text("  H                   Show help (same as F1)"),
      separator(),
      text("Press F1, H, or Esc to close") | dim | center,
    }) | border;

    // For now, log help (proper modal implementation would use FTXUI Modal)
    spdlog::info("Help screen displayed (F1/H/Esc to close)");
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
  } else {
    spdlog::warn("Unknown backend: {}, falling back to mock", backend);
    provider = std::make_unique<MockProvider>();
  }

  TUIApp app(std::move(provider));
  app.Run();

  return 0;
}
