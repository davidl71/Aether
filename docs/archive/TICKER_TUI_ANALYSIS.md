# Ticker Terminal UI Analysis

**Date**: 2025-11-30
**Status**: Research Complete
**Related Task**: T-5

---

## Executive Summary

Ticker is a Go-based terminal UI application for tracking stocks, cryptocurrencies, and derivatives prices in real-time. This analysis compares ticker's architecture and implementation patterns with our current C++ FTXUI-based TUI to identify potential improvements, architectural insights, and learning opportunities.

**Key Finding**: While ticker serves a different purpose (price tracking vs. trading dashboard), its use of the bubbletea framework and polling-based architecture offers valuable patterns for terminal UI development, particularly around data refresh, configuration management, and position tracking.

---

## Ticker Overview

### Project Details

- **Language**: Go 100%
- **License**: GPL-3.0
- **Stars**: 5.8k+ (popular project)
- **Framework**: bubbletea (terminal UI framework)
- **Purpose**: Real-time stock/crypto price tracking and position monitoring

### Key Features

1. **Live Price Quotes**
   - Real-time stock and cryptocurrency prices
   - Pre-market and post-market quotes
   - Multiple data sources (Yahoo Finance, Coinbase)

2. **Position Tracking**
   - Track stock positions with cost basis
   - Support for multiple cost basis lots
   - Automatic value calculations

3. **Configuration**
   - YAML-based configuration (`.ticker.yaml`)
   - Watchlists and holdings
   - Custom color schemes
   - Currency conversion

4. **Display Options**
   - Summary statistics
   - Tags and fundamentals
   - Holdings display
   - Custom sorting

### Technology Stack

- **bubbletea**: Terminal UI framework (Go)
- **termenv**: Color and styling for terminals
- **term-grid**: Grid layout library
- **Data Sources**: Yahoo Finance (default), Coinbase (WebSocket)

---

## Current Project TUI Architecture

### Technology Stack

- **Language**: C++20
- **Framework**: FTXUI (C++ terminal UI library)
- **Purpose**: Trading dashboard for box spread strategies
- **Build**: CMake, part of main build system

### Key Features

1. **Trading Dashboard**
   - Live account/strategy state
   - Position tracking (box spreads)
   - Order management
   - Alert system

2. **Data Sources**
   - REST endpoints (shared with web)
   - WebSocket for real-time updates
   - Mock TWS service for testing
   - QuestDB for historical data

3. **Configuration**
   - JSON-based configuration (`tui_config.json`)
   - XDG config directory support
   - Environment variable overrides

4. **Layout**
   - Tab-based interface (Dashboard, Positions, Orders, Alerts)
   - GNU top/htop-inspired design
   - Quick action workflows

### Implementation Files

```text
native/src/
  ├── tui_app.cpp          # Main TUI application
  ├── tui_provider.cpp     # Data providers (REST, WebSocket, Mock)
  ├── tui_data.cpp         # Data structures
  ├── tui_config.cpp       # Configuration management
  ├── tui_converter.cpp    # Type conversions
  └── tui_breadcrumb.cpp   # Navigation breadcrumbs

native/include/
  ├── tui_provider.h
  ├── tui_data.h
  ├── tui_config.h
  ├── tui_converter.h
  └── tui_breadcrumb.h
```

---

## Comparison Analysis

### Language & Framework

| Aspect           | Ticker            | IBKR Box Spread TUI |
| ---------------- | ----------------- | ------------------- |
| **Language**     | Go                | C++20               |
| **Framework**    | bubbletea         | FTXUI               |
| **Performance**  | Good (Go runtime) | Excellent (native)  |
| **Memory**       | Garbage collected | Manual management   |
| **Compilation**  | Single binary     | CMake build system  |
| **Dependencies** | Go modules        | CMake dependencies  |

**Verdict**: Our C++20 approach provides better performance and integrates seamlessly with our existing C++ codebase. Go's bubbletea is excellent for standalone tools, but FTXUI fits our native architecture better.

### UI Framework Comparison

#### bubbletea (Ticker)

**Architecture**:

- **Model-View-Update (MVU)** pattern (like Elm)
- **Message-based** state updates
- **Component composition** for complex UIs
- **Built-in update loop** with ticker support

**Example Pattern**:

```go
type model struct {
    quotes []Quote
    err    error
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
    switch msg := msg.(type) {
    case tickMsg:
        return m, fetchQuotes
    case quotesMsg:
        m.quotes = msg.quotes
        return m, nil
    }
    return m, nil
}
```

**Benefits**:

- Functional, predictable state management
- Easy to test (pure functions)
- Built-in refresh/ticker support
- Excellent documentation

#### FTXUI (Our Project)

**Architecture**:

- **Component-based** reactive UI
- **Event-driven** updates
- **Imperative** rendering model
- **Custom refresh** via threads/timers

**Example Pattern**:

```cpp
class TUIApp {
    Snapshot latest_snapshot_;
    std::mutex snapshot_mutex_;

    void UpdateLoop() {
        while (!should_exit_) {
            auto snapshot = provider_->GetSnapshot();
            {
                std::lock_guard<std::mutex> lock(snapshot_mutex_);
                latest_snapshot_ = snapshot;
            }
            std::this_thread::sleep_for(std::chrono::seconds(1));
        }
    }
};
```

**Benefits**:

- Native C++ performance
- Direct integration with existing code
- Full control over rendering
- No language bridge overhead

### Data Refresh Patterns

#### Ticker Approach

**Polling-Based**:

- Configurable `refresh-interval` (default: 5+ seconds)
- Yahoo Finance: Polling with intentional delays
- Coinbase: WebSocket for real-time (spot assets)
- Single-threaded update loop via bubbletea

**Configuration**:

```yaml

# .ticker.yaml

refresh-interval: 5  # seconds
watchlist:
  - AAPL
  - TSLA
```

**Benefits**:

- Simple to implement
- Works with any data source
- Predictable update frequency

**Drawbacks**:

- Higher latency (polling delay)
- More API calls (rate limiting concerns)
- Not truly real-time

#### Our Approach

**Hybrid (REST + WebSocket)**:

- REST polling for initial data
- WebSocket for real-time updates (when available)
- Background threads for data fetching
- Provider abstraction (REST, WebSocket, Mock)

**Architecture**:

```cpp
class Provider {
public:
    virtual void Start() = 0;
    virtual Snapshot GetSnapshot() = 0;
};

class RESTProvider : public Provider {
    // Polls REST endpoints
};

class WebSocketProvider : public Provider {
    // Subscribes to WebSocket for push updates
};
```

**Benefits**:

- Lower latency (WebSocket push)
- Fewer API calls (push vs. poll)
- Real-time updates when available
- Fallback to polling

**Drawbacks**:

- More complex implementation
- Requires WebSocket support
- Thread synchronization needed

### Configuration Management

#### Ticker

**YAML Configuration**:

```yaml

# .ticker.yaml

watchlist:
  - AAPL
  - TSLA
holdings:
  - symbol: AAPL
    quantity: 10
    unit_cost: 150.00
colors:
  text: "#005fff"
  background-tag: "#0087ff"
refresh-interval: 5
currency: USD
```

**Features**:

- YAML format (human-readable)
- Multiple config locations (home, current dir, XDG)
- Groups for organizing watchlists
- Custom color schemes

#### Our Project

**JSON Configuration**:

```json
{
  "backend": "rest",
  "api_url": "http://localhost:8080",
  "refresh_interval_ms": 1000,
  "theme": "default"
}
```

**Features**:

- JSON format (programmatic generation)
- XDG config directory support
- Environment variable overrides
- Type-safe C++ loading

**Comparison**:

- **YAML**: More human-friendly, better for complex nested data
- **JSON**: Better for programmatic generation, type validation
- **Both**: Support multiple config locations

### Position Tracking

#### Ticker

**Multiple Cost Basis Lots**:

```yaml
holdings:
  - symbol: ARKW
    quantity: 50
    unit_cost: 120.00
  - symbol: ARKW
    quantity: 30
    unit_cost: 125.00
```

**Features**:

- Multiple lots per symbol
- Automatic value calculation
- Weight calculation
- Cost basis tracking

#### Our Project

**Box Spread Positions**:

```cpp
struct Position {
    std::string symbol;
    int quantity;
    double cost_basis;
    double current_value;
    double roi;
    // Box spread specific fields
};
```

**Features**:

- Box spread-specific tracking
- ROI calculations
- Maker/taker counts
- Rebate estimates

**Comparison**:

- **Ticker**: Generic position tracking (stocks/crypto)
- **Our Project**: Trading-specific (box spreads, options)
- **Both**: Cost basis and value tracking

### Data Source Architecture

#### Ticker

**Multi-Source Support**:

- **Yahoo Finance** (default): Polling-based, 5+ second delay
- **Coinbase** (`.CB` suffix): WebSocket for spot, polling for derivatives
- **Shorthand symbols** (`.X`): Ticker-specific symbols

**Symbol Format**:

```
AAPL          # Yahoo Finance (default)
SOL1-USD      # Yahoo Finance crypto
STRK.CB       # Coinbase
SOL.X         # Ticker shorthand
```

**Architecture**:

- Source determined by symbol suffix
- Unified interface for all sources
- Automatic fallback handling

#### Our Project

**Provider Abstraction**:

```cpp
class Provider {
public:
    virtual void Start() = 0;
    virtual Snapshot GetSnapshot() = 0;
};

// Implementations:
// - RESTProvider: Polls REST API
// - WebSocketProvider: WebSocket push
// - MockProvider: Test data
```

**Data Sources**:

- REST API (backend service)
- WebSocket (real-time updates)
- Mock (testing)
- QuestDB (historical)

**Comparison**:

- **Ticker**: Multiple external APIs (Yahoo, Coinbase)
- **Our Project**: Internal backend services
- **Both**: Abstraction layer for data sources

---

## Architectural Insights & Learning Opportunities

### 1. Refresh Interval Configuration

**Ticker Pattern**:

```yaml
refresh-interval: 5  # seconds
```

**Our Current Approach**:

```json
{
  "refresh_interval_ms": 1000
}
```

**Recommendation**: Consider adding a `refresh-interval` field to our config with human-readable units (seconds) that converts to milliseconds internally.

### 2. Group/Organization Support

**Ticker Feature**:

```yaml
groups:
  - name: Tech Stocks
    watchlist:
      - AAPL
      - MSFT
  - name: Crypto
    watchlist:
      - BTC-USD
```

**Potential Application**: We could add group support for organizing positions:

- By strategy
- By expiration date
- By symbol type

### 3. Color Scheme Customization

**Ticker Pattern**:

```yaml
colors:
  text: "#005fff"
  text-light: "#0087ff"
  background-tag: "#0087ff"
```

**Our Current Approach**: Hardcoded color scheme in FTXUI.

**Recommendation**: Add configurable color schemes to match user preferences and terminal capabilities.

### 4. Summary Statistics

**Ticker Feature**: `--show-summary` displays:

- Total portfolio value
- Total gain/loss
- Percentage change

**Our Current Approach**: Dashboard shows similar metrics but could benefit from ticker's layout.

**Recommendation**: Study ticker's summary layout for inspiration on displaying aggregate statistics.

### 5. Sorting Options

**Ticker Feature**:

```yaml
sort: value  # or: alpha, user, default
```

**Our Current Approach**: Fixed sorting (by symbol or value).

**Recommendation**: Add configurable sorting options similar to ticker.

### 6. Currency Conversion

**Ticker Feature**:

```yaml
currency: EUR  # Convert all values to EUR
currency-summary-only: true  # Only convert summary
```

**Our Current Approach**: USD-only (implicit).

**Recommendation**: Consider adding currency conversion for international users.

### 7. Print/Export Functionality

**Ticker Feature**:

```bash
ticker print --format=csv
ticker print --format=json
```

**Our Current Approach**: No export functionality.

**Recommendation**: Add `--export` flag to export positions/orders to CSV/JSON.

---

## Integration Opportunities

### Option 1: Adopt UI Patterns (Recommended)

**Action**: Study ticker's UI patterns and apply similar approaches to our FTXUI implementation.

**Specific Patterns**:

1. **Summary Statistics Layout**: Improve dashboard summary display
2. **Group Organization**: Add position grouping by strategy/expiration
3. **Color Customization**: Add configurable color schemes
4. **Sorting Options**: Add user-configurable sorting

**Benefits**:

- Keep our C++20 performance
- Improve UX with proven patterns
- No external dependencies
- Maintain our architecture

**Effort**: Low-Medium (1-2 weeks)

### Option 2: Learn from bubbletea Architecture

**Action**: Study bubbletea's MVU pattern and apply similar state management to our FTXUI code.

**Benefits**:

- More predictable state updates
- Easier testing (pure functions)
- Better separation of concerns

**Challenges**:

- FTXUI uses different patterns (component-based)
- Would require significant refactoring
- May not fit our current architecture

**Effort**: High (4-6 weeks)

### Option 3: Direct Integration (Not Recommended)

**Action**: Use ticker as a subprocess or library for price tracking.

**Why Not Recommended**:

- Different purpose (price tracking vs. trading dashboard)
- Language mismatch (Go vs C++)
- License incompatibility (GPL-3.0 vs our license)
- Would add unnecessary complexity

---

## Recommendations

### Short-Term (1-3 months)

1. **Add Color Scheme Configuration**
   - Support custom colors in `tui_config.json`
   - Fallback to defaults if not specified
   - Test with different terminal capabilities

2. **Improve Summary Statistics Display**
   - Study ticker's summary layout
   - Add aggregate metrics (total value, gain/loss)
   - Better visual hierarchy

3. **Add Sorting Options**
   - Configurable sort order (symbol, value, ROI)
   - User preference in config file
   - Keyboard shortcuts for quick sorting

### Medium-Term (3-6 months)

1. **Group/Organization Support**
   - Group positions by strategy
   - Group by expiration date
   - Tab-based navigation between groups

2. **Export Functionality**
   - `--export` flag for CSV/JSON
   - Print positions/orders to file
   - Integration with reporting tools

3. **Currency Conversion**
   - Support for non-USD currencies
   - Configurable conversion rates
   - Display in local currency

### Long-Term (6+ months)

1. **Enhanced Refresh Strategy**
   - Adaptive refresh intervals
   - Smart polling (reduce when idle)
   - Better WebSocket integration

2. **Multiple Cost Basis Lots**
   - Track multiple entries per position
   - Average cost calculation
   - FIFO/LIFO support

---

## Key Takeaways

1. **Different Purposes**: Ticker is for price tracking, our TUI is for trading - complementary but different
2. **Framework Comparison**: bubbletea (Go) vs FTXUI (C++) - both excellent, FTXUI fits our stack better
3. **UI Patterns are Valuable**: Ticker's layout and organization patterns can improve our UX
4. **Configuration**: YAML vs JSON - both valid, JSON fits our programmatic needs
5. **Refresh Strategies**: Polling (ticker) vs WebSocket (ours) - ours is better for real-time trading

---

## References

- **Ticker GitHub**: <https://github.com/achannarasappa/ticker>
- **bubbletea Framework**: <https://github.com/charmbracelet/bubbletea>
- **FTXUI Framework**: <https://github.com/ArthurSonzogni/FTXUI>
- **GPL-3.0 License**: Note - incompatible with our license if direct integration

---

## Related Documentation

- [../TUI_ARCHITECTURE.md](../TUI_ARCHITECTURE.md) - Active Rust TUI architecture
- [platform/TUI_LEGACY_DESIGN_LEARNINGS.md](platform/TUI_LEGACY_DESIGN_LEARNINGS.md) - Consolidated archive TUI learnings
- TUI testing — historical Python/Textual guide (archived; doc removed)
- [API Documentation Index](API_DOCUMENTATION_INDEX.md) - Complete API reference

---

**Last Updated**: 2025-11-30
**Next Review**: When implementing color scheme configuration or group support
