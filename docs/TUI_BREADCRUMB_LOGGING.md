# TUI Breadcrumb Logging Guide

**Purpose**: Comprehensive guide for implementing and using breadcrumb logging in the TUI application for debugging and testing.

**References**:

- [Debug TUI Blog Post](https://www.dantleech.com/blog/2025/05/11/debug-tui/)
- [GUI Testing Tools](https://www.browserstack.com/guide/open-source-gui-testing-tool)

---

## Overview

Breadcrumb logging provides a trail of user interactions and application state changes that helps developers and testers trace the sequence of events leading up to a specific issue or error. This is distinct from traditional navigation breadcrumbs - these are debugging breadcrumbs.

### Key Benefits

1. **Faster Debugging**: Quickly pinpoint the exact sequence of events that led to a bug
2. **Improved Test Reliability**: Detailed logs help differentiate between application bugs and test script issues
3. **Better Collaboration**: Breadcrumbs provide a common language for testers and developers
4. **Reproducibility**: Complete trail of interactions makes it easier to reproduce issues

---

## Architecture

### Components

1. **BreadcrumbLogger**: Main logging class that captures and stores breadcrumb entries
2. **BreadcrumbEntry**: Individual log entry with timestamp, type, component ID, action, and optional state
3. **Integration Points**: Hooks into TUI event handlers, state management, and error handlers

### Breadcrumb Types

- **UserInput**: Keyboard input, mouse clicks (if applicable)
- **Navigation**: Tab switches, menu navigation
- **StateChange**: Application state modifications
- **Error**: Error occurrences
- **ScreenRender**: Screen buffer updates
- **DialogOpen/Close**: Dialog/modal lifecycle
- **Action**: User actions (start strategy, cancel order, etc.)
- **DataUpdate**: Data provider updates
- **ConfigChange**: Configuration modifications

---

## Implementation

### 1. Initialization

Initialize breadcrumb logging at application startup:

```cpp

#include "tui_breadcrumb.h"

// In main() or TUIApp constructor
tui::BreadcrumbLogger::Config breadcrumb_config;
breadcrumb_config.enabled = true;
breadcrumb_config.log_file = "/tmp/ib_box_spread_breadcrumbs.log";
breadcrumb_config.log_to_console = false;
breadcrumb_config.log_to_file = true;
breadcrumb_config.max_entries = 1000;
breadcrumb_config.capture_state_dumps = true;
breadcrumb_config.capture_screen_dumps = false;  // Expensive, enable only when needed

tui::InitializeBreadcrumbLogging(breadcrumb_config);
```

### 2. Logging User Interactions

#### Keyboard Input

```cpp
// In event handler (tui_app.cpp)
component |= CatchEvent(& {
  // Log the input
  std::string event_str;
  if (event == Event::F1) event_str = "F1";
  else if (event == Event::F2) event_str = "F2";
  else if (event == Event::Tab) event_str = "Tab";
  else if (event == Event::Character('q')) event_str = "q";
  // ... etc

  TUI_BREADCRUMB_INPUT("main_screen", event_str,
                       "selected_tab=" + std::to_string(selected_tab));

  // Handle event...
});
```

#### Navigation

```cpp
// When switching tabs
if (event == Event::Tab) {
  int old_tab = selected_tab;
  selected_tab = (selected_tab + 1) % 5;

  TUI_BREADCRUMB_NAVIGATION(
    "tab_" + std::to_string(old_tab),
    "tab_" + std::to_string(selected_tab),
    "keyboard_shortcut=Tab"
  );

  return true;
}
```

### 3. Logging State Changes

```cpp
// When snapshot updates
void UpdateSnapshot(const Snapshot& new_snapshot) {
  std::lock_guard<std::mutex> lock(snapshot_mutex_);

  // Log state change
  nlohmann::json state_json;
  state_json["symbols_count"] = new_snapshot.symbols.size();
  state_json["positions_count"] = new_snapshot.positions.size();
  state_json["orders_count"] = new_snapshot.orders.size();
  state_json["mode"] = new_snapshot.mode;
  state_json["strategy"] = new_snapshot.strategy;

  TUI_BREADCRUMB_STATE_CHANGE(
    "snapshot_manager",
    "snapshot_updated",
    state_json.dump()
  );

  latest_snapshot_ = new_snapshot;
}
```

### 4. Logging Errors

```cpp
// In error handlers
try {
  // ... operation that might fail
} catch (const std::exception& e) {
  TUI_BREADCRUMB_ERROR(
    "order_manager",
    "order_submission_failed",
    "error=" + std::string(e.what()) + " order_id=" + std::to_string(order_id)
  );

  spdlog::error("Order submission failed: {}", e.what());
}
```

### 5. Logging Actions

```cpp
// When user performs actions
if (event == Event::Character('s') || event == Event::Character('S')) {
  TUI_BREADCRUMB_ACTION("start_strategy", "main_screen",
                        "keyboard_shortcut=s");

  // Start strategy...
}

if (event == Event::F9) {
  TUI_BREADCRUMB_ACTION("cancel_orders", "orders_tab",
                        "keyboard_shortcut=F9");

  // Cancel orders...
}
```

### 6. Logging Dialogs

```cpp
// When showing dialogs
void ShowHelp(ScreenInteractive& screen) {
  TUI_BREADCRUMB_DIALOG("help_dialog", true, "triggered_by=F1");

  // Show help...
}

// When closing
if (event == Event::Escape) {
  TUI_BREADCRUMB_DIALOG("help_dialog", false, "triggered_by=Escape");
  // Close dialog...
}
```

### 7. Logging Data Updates

```cpp
// In data provider
void MockProvider::GenerateLoop() {
  while (running_.load()) {
    auto snapshot = GenerateSnapshot();

    TUI_BREADCRUMB_ACTION("data_update", "mock_provider",
                          "symbols=" + std::to_string(snapshot.symbols.size()));

    {
      std::lock_guard<std::mutex> lock(mutex_);
      latest_snapshot_ = snapshot;
    }

    std::this_thread::sleep_for(interval_);
  }
}
```

---

## Component IDs

Use descriptive, unique identifiers for UI components:

### Recommended Naming Convention

- **Screens**: `main_screen`, `setup_screen`, `help_screen`
- **Tabs**: `tab_dashboard`, `tab_positions`, `tab_historic`, `tab_orders`, `tab_alerts`
- **Dialogs**: `help_dialog`, `search_dialog`, `cancel_orders_dialog`, `sort_menu_dialog`
- **Components**: `header_status`, `footer_keys`, `symbol_table`, `position_list`
- **Managers**: `snapshot_manager`, `order_manager`, `config_manager`
- **Providers**: `mock_provider`, `rest_provider`, `ibkr_rest_provider`

### Example Component IDs

```cpp
// In tui_app.cpp
const std::string COMPONENT_MAIN_SCREEN = "main_screen";
const std::string COMPONENT_DASHBOARD_TAB = "tab_dashboard";
const std::string COMPONENT_POSITIONS_TAB = "tab_positions";
const std::string COMPONENT_SETUP_SCREEN = "setup_screen";
const std::string COMPONENT_HELP_DIALOG = "help_dialog";
```

---

## State Dumping

### When to Dump State

1. **On Errors**: Always dump relevant state when errors occur
2. **On State Changes**: Dump state when critical state changes occur
3. **On User Actions**: Dump state before/after user actions that modify state
4. **On Navigation**: Optionally dump state when navigating between major screens

### What to Include

- Current screen/tab
- Selected items
- Form data (if applicable)
- Configuration values
- Connection status
- Data counts (symbols, positions, orders)

### Example State Dump

```cpp
std::string DumpCurrentState() {
  nlohmann::json state;

  state["current_tab"] = selected_tab;
  state["show_setup"] = show_setup_;
  state["snapshot"] = {
    {"symbols_count", latest_snapshot_.symbols.size()},
    {"positions_count", latest_snapshot_.positions.size()},
    {"orders_count", latest_snapshot_.orders.size()},
    {"mode", latest_snapshot_.mode},
    {"strategy", latest_snapshot_.strategy}
  };

  return state.dump();
}

// Use it
TUI_BREADCRUMB_STATE_CHANGE("main_screen", "tab_changed", DumpCurrentState());
```

---

## Retrieving Breadcrumbs

### For Debugging

```cpp
// Get last 100 breadcrumbs
auto breadcrumbs = tui::GetBreadcrumbLogger().GetBreadcrumbs(100);

for (const auto& entry : breadcrumbs) {
  std::cout << entry.ToString() << std::endl;
}
```

### For Testing

```cpp
// Get breadcrumbs since test started
auto test_start = std::chrono::system_clock::now();
// ... run test ...
auto breadcrumbs = tui::GetBreadcrumbLogger().GetBreadcrumbsSince(test_start);

// Assert on breadcrumbs
ASSERT_TRUE(std::any_of(breadcrumbs.begin(), breadcrumbs.end(),
  [](const auto& entry) {
    return entry.component_id == "main_screen" &&
           entry.action == "navigate" &&
           entry.details.find("to=tab_positions") != std::string::npos;
  }));
```

### Filtering by Type

```cpp
// Get all errors
auto errors = tui::GetBreadcrumbLogger().GetBreadcrumbsByType(
  tui::BreadcrumbType::Error, 100);

// Get all user inputs
auto inputs = tui::GetBreadcrumbLogger().GetBreadcrumbsByType(
  tui::BreadcrumbType::UserInput, 100);
```

---

## Integration with Testing

### Test Framework Integration

```cpp
// In test setup
class TUITestFixture {
protected:
  void SetUp() override {
    // Initialize breadcrumb logging for test
    tui::BreadcrumbLogger::Config config;
    config.enabled = true;
    config.log_file = "/tmp/test_breadcrumbs.log";
    config.max_entries = 500;
    tui::InitializeBreadcrumbLogging(config);
  }

  void TearDown() override {
    // Get breadcrumbs for test report
    auto breadcrumbs = tui::GetBreadcrumbLogger().GetBreadcrumbs();
    // Write to test report...
  }
};
```

### Asserting on Breadcrumbs

```cpp
TEST_F(TUITestFixture, TestTabNavigation) {
  // Simulate user input
  SimulateKeyPress(Event::Tab);
  SimulateKeyPress(Event::Tab);

  // Get breadcrumbs
  auto breadcrumbs = tui::GetBreadcrumbLogger().GetBreadcrumbs(10);

  // Assert navigation occurred
  int navigation_count = 0;
  for (const auto& entry : breadcrumbs) {
    if (entry.type == tui::BreadcrumbType::Navigation) {
      navigation_count++;
    }
  }

  ASSERT_GE(navigation_count, 2);
}
```

---

## Configuration

### Environment Variables

```bash

# Enable breadcrumb logging

export TUI_BREADCRUMB_ENABLED=true

# Set log file path

export TUI_BREADCRUMB_LOG_FILE=/tmp/tui_breadcrumbs.log

# Enable console logging (for debugging)

export TUI_BREADCRUMB_CONSOLE=true

# Maximum entries in memory

export TUI_BREADCRUMB_MAX_ENTRIES=1000
```

### Configuration File

```json
{
  "breadcrumb": {
    "enabled": true,
    "log_file": "/tmp/ib_box_spread_breadcrumbs.log",
    "log_to_console": false,
    "log_to_file": true,
    "max_entries": 1000,
    "capture_state_dumps": true,
    "capture_screen_dumps": false,
    "flush_interval_ms": 1000
  }
}
```

---

## Best Practices

### 1. Use Descriptive Component IDs

**Bad**:

```cpp
TUI_BREADCRUMB_INPUT("input1", "key", "");
```

**Good**:

```cpp
TUI_BREADCRUMB_INPUT("search_dialog_input_field", "key=q", "searching_for_symbol");
```

### 2. Include Context in Details

**Bad**:

```cpp
TUI_BREADCRUMB_NAVIGATION("tab1", "tab2", "");
```

**Good**:

```cpp
TUI_BREADCRUMB_NAVIGATION("tab_dashboard", "tab_positions",
                          "triggered_by=Tab keyboard_shortcut");
```

### 3. Dump State on Errors

**Always include state when logging errors**:

```cpp
try {
  // ... operation
} catch (const std::exception& e) {
  std::string state = DumpCurrentState();
  TUI_BREADCRUMB_ERROR("order_manager", "order_failed",
                       "error=" + std::string(e.what()), state);
}
```

### 4. Log Before State Changes

**Log before modifying state**:

```cpp
// Log before change
TUI_BREADCRUMB_STATE_CHANGE("config_manager", "updating_provider_type",
                            "old=" + config_.provider_type);
// Make change
config_.provider_type = new_type;
// Log after change
TUI_BREADCRUMB_STATE_CHANGE("config_manager", "provider_type_updated",
                            "new=" + config_.provider_type);
```

### 5. Use Consistent Action Names

**Use consistent action naming**:

- `input` for user input
- `navigate` for navigation
- `state_change` for state changes
- `error` for errors
- `open`/`close` for dialogs
- Use descriptive names for custom actions: `start_strategy`, `cancel_orders`, etc.

---

## Performance Considerations

### Memory Usage

- Breadcrumbs are stored in memory (configurable max entries)
- State dumps can be large - use judiciously
- Screen dumps are very expensive - only enable when debugging rendering issues

### Recommendations

1. **Default Configuration**:
   - `max_entries = 1000`
   - `capture_state_dumps = true`
   - `capture_screen_dumps = false`

2. **Debugging Configuration**:
   - `max_entries = 5000`
   - `capture_state_dumps = true`
   - `capture_screen_dumps = true` (if debugging rendering)

3. **Production Configuration**:
   - `max_entries = 500`
   - `capture_state_dumps = false` (or minimal)
   - `capture_screen_dumps = false`

---

## Log File Format

### Text Format

```
2025-01-27 14:32:15.123 [0] main_screen input input=F1 |
2025-01-27 14:32:15.234 [1] main_screen navigate to=help_dialog | triggered_by=F1
2025-01-27 14:32:16.456 [2] help_dialog open |
2025-01-27 14:32:18.789 [1] help_dialog navigate to=main_screen | triggered_by=Escape
2025-01-27 14:32:18.890 [3] help_dialog close |
```

### JSON Format

```json
{
  "timestamp": "2025-01-27T14:32:15.123Z",
  "type": 0,
  "component_id": "main_screen",
  "action": "input",
  "details": "input=F1",
  "state_snapshot": ""
}
```

---

## Troubleshooting

### Breadcrumbs Not Appearing

1. Check if breadcrumb logging is enabled:

   ```cpp
   auto& logger = tui::GetBreadcrumbLogger();
   if (!logger.GetConfig().enabled) {
     // Enable it
   }
   ```

2. Check log file permissions
3. Check if logger is initialized

### Too Many Breadcrumbs

1. Reduce `max_entries`
2. Disable `capture_state_dumps` or `capture_screen_dumps`
3. Filter breadcrumbs by type when retrieving

### Performance Issues

1. Disable screen dumps (`capture_screen_dumps = false`)
2. Reduce state dump frequency
3. Reduce `max_entries`
4. Increase `flush_interval` to reduce I/O

---

## Example: Complete Integration

See `native/src/tui_app.cpp` for complete integration examples with:

- Keyboard input logging
- Tab navigation logging
- Dialog open/close logging
- State change logging
- Error logging

---

## References

- [Debug TUI Blog Post](https://www.dantleech.com/blog/2025/05/11/debug-tui/)
- [GUI Testing Tools Guide](https://www.browserstack.com/guide/open-source-gui-testing-tool)
- [TUI Design Documentation](research/architecture/TUI_DESIGN.md)
- [TUI Testing Guide](research/integration/TUI_TESTING.md)
