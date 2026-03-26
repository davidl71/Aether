# TUI Implementation Patterns Research

**Status**: Synthesis Complete  
**Sources**: ticker (Go/bubbletea), tickrs (Rust/tui-rs), longbridge-terminal patterns  
**Applies to**: Aether Rust/Ratatui TUI

---

## Executive Summary

This document synthesizes research from multiple terminal UI applications to identify implementation patterns applicable to Aether's Rust/Ratatui TUI.

### Key Findings

1. **Mode System**: All professional TUIs use explicit mode systems (Navigation/Edit/View)
2. **Dirty Flag Optimization**: Essential for 60fps rendering with large datasets
3. **ScrollableTableState**: Pattern for table navigation with selection and scrolling
4. **Toast Notifications**: Non-blocking user feedback with auto-dismiss
5. **Right-aligned Numerics**: Financial data presentation standard

---

## Source Analysis

### 1. Ticker (Go/bubbletea) - Archived Analysis

**Repository**: `achannarasappa/ticker`  
**Framework**: bubbletea (Elm architecture for Go)  
**License**: GPL-3.0 (incompatible for direct use)

#### Patterns Identified

| Pattern | Implementation | Applicable to Aether |
|---------|----------------|---------------------|
| **MVU Architecture** | Model-View-Update with message passing | Partial - Ratatui uses event-driven |
| **Polling Refresh** | Configurable intervals (5s default) | Already using NATS push (superior) |
| **Summary Statistics** | Portfolio totals, gain/loss | Not implemented - HIGH PRIORITY |
| **Position Grouping** | By watchlist/group | Not implemented - HIGH PRIORITY |
| **Color Schemes** | User-configurable themes | Not implemented - MEDIUM PRIORITY |
| **Sorting Options** | Configurable sort order | Not implemented - MEDIUM PRIORITY |
| **Export (CSV/JSON)** | `--export` flag | Not implemented - LOW PRIORITY |

#### Key Insight
Tickers uses a polling-based architecture intentionally for simplicity. Aether's NATS-based push architecture is superior for real-time trading data.

---

### 2. Tickrs (Rust/tui-rs)

**Repository**: `tarkah/tickrs`  
**Framework**: tui-rs (predecessor to Ratatui)  
**Stars**: 1.6k  
**License**: MIT (compatible)

#### Architecture

```
tickrs/
├── api/          # Yahoo Finance API client
├── src/
│   ├── main.rs   # Event loop, app state
│   ├── app.rs    # Application state
│   ├── ui.rs     # Rendering
│   ├── widget/   # Custom widgets
│   │   ├── chart.rs      # Candlestick charts
│   │   ├── options.rs    # Options chain display
│   │   └── summary.rs    # Portfolio summary
│   └── task/     # Async task management
│       ├── fetch.rs      # Data fetching
│       └── update.rs     # Update coordination
```

#### Key Patterns

**Async Data Fetching + UI Coordination**:
```rust
// Pattern: Separate async fetch from UI update
pub struct FetchTask {
    symbol: String,
    tx: mpsc::Sender<MarketData>,
}

// In main loop:
// 1. Spawn fetch task
// 2. Receive result via channel
// 3. Update App state
// 4. Trigger redraw
```

**Chart Rendering**:
- Uses tui-rs canvas for candlestick rendering
- Supports multiple timeframes (1D, 1W, 1M, etc.)
- Pre/post market data handling
- Volume overlay

**Keyboard Navigation**:
- Arrow keys for navigation
- Number keys for tab jumping
- `/` for search
- `?` for help overlay
- `q` to quit

**Configuration**:
- YAML config file support
- CLI args override config
- Watchlist persistence

#### Applicability to Aether

| Feature | Tickrs Pattern | Aether Status |
|---------|----------------|---------------|
| Candlestick charts | Canvas-based | Already implemented |
| Timeframe switching | Tab-based | Already implemented |
| Pre/post market | Configurable | Not implemented |
| Volume display | Overlay | Not implemented |
| Options chain | Table view | Partial (scenarios tab) |
| Summary mode | Dedicated view | Not implemented |

---

### 3. Longbridge Terminal Patterns

From task descriptions, these patterns are identified:

#### T-1774285319951400000: Numeric Right-Alignment
**Pattern**: Financial data tables with right-aligned numerics  
**Benefit**: Easy comparison of decimal places  
**Implementation**:
```rust
// Table cells with numeric alignment
Cell::from(format!("{:>10.2}", price))  // Right-align, 2 decimals
Cell::from(format!("{:>8}", quantity))   // Right-align, integer
```

#### T-1774285317440018000: Toast Notifications
**Pattern**: Non-blocking transient notifications  
**Benefit**: User feedback without modal interruption  
**Status**: ✅ IMPLEMENTED in `ui/feedback.rs`

#### T-1774285315059562000: ScrollableTableState
**Pattern**: Table state with selection + scroll position  
**Benefit**: Large table navigation  
**Implementation**:
```rust
pub struct ScrollableTableState {
    offset: usize,      // Scroll offset
    selected: usize,    // Selected row
    items_count: usize,
}
```

#### T-1774285312494816000: DirtyFlags Render Optimization
**Pattern**: Only re-render changed regions  
**Benefit**: 60fps with large datasets  
**Implementation**:
```rust
pub struct DirtyFlags {
    tabs: bool,
    content: bool,
    status_bar: bool,
    // Only redraw flagged regions
}
```

---

## Mode System Design

### Current State

Aether has an `InputMode` enum but it's reactive (responding to state), not proactive (controlling available actions).

```rust
// Current (reactive)
pub enum InputMode {
    Normal,
    Help,           // Shows help overlay
    DetailPopup,    // Shows detail view
    SettingsEditConfig,
    LoanForm,
    ChartSearch,
    // ...
}
```

### Recommended Mode System

Implement explicit modes that control available key bindings:

```rust
pub enum AppMode {
    Navigation,     // Tab switching, scrolling, selection
    Edit,           // Form input, text editing
    View,           // Detail views, charts, read-only
}

pub enum NavigationMode {
    Normal,         // Standard navigation
    Search,         // Search/filter active
}
```

### Mode Transition Map

| From Mode | Key/Event | To Mode | Action |
|-----------|-----------|---------|--------|
| Navigation | Enter on item | View | Open detail |
| Navigation | Tab | Navigation | Next tab |
| Navigation | i | Edit | Enter insert mode |
| Edit | Esc | Navigation | Cancel/Close |
| Edit | Enter | Navigation | Save/Confirm |
| View | Esc | Navigation | Close view |
| View | Arrow keys | View | Scroll detail |

### Implementation Plan

1. Add `AppMode` enum alongside existing `InputMode`
2. Create mode-specific key binding maps
3. Add mode indicator to status bar
4. Update help overlay to show mode-specific shortcuts
5. Implement mode transitions in input handlers

---

## Implementation Tasks

### Immediate (High Priority)

1. **T-UX-001**: Implement AppMode system (Navigation/Edit/View)
   - Add AppMode enum
   - Create mode transition logic
   - Add mode indicator to status bar
   - Update key binding handlers

2. **T-UX-002**: Add ScrollableTableState for large tables
   - Positions tab
   - Orders tab
   - Scenarios tab

3. **T-UX-003**: Numeric right-alignment in tables
   - Update positions table
   - Update orders table
   - Update scenarios table

### Short-term (Medium Priority)

4. **T-UX-004**: DirtyFlags render optimization
   - Implement DirtyFlags struct
   - Update render loop
   - Benchmark performance

5. **T-UX-005**: Position grouping by strategy/expiration
   - Group positions in positions tab
   - Expandable/collapsible groups
   - Group-level statistics

6. **T-UX-006**: Summary statistics display
   - Portfolio totals in dashboard
   - Gain/loss aggregations
   - Per-strategy summaries

### Long-term (Low Priority)

7. **T-UX-007**: Configurable color schemes
   - Theme configuration
   - Color palette management
   - Terminal capability detection

8. **T-UX-008**: Export functionality (CSV/JSON)
   - Export positions
   - Export orders
   - Export scenarios

9. **T-UX-009**: Sorting options for tables
   - Clickable column headers
   - Configurable default sort
   - Multi-column sort

---

## Key Insights

1. **Architecture**: Aether's NATS-based push model is superior to polling used by ticker/tickrs
2. **Framework**: Ratatui is the right choice (actively maintained, feature-rich)
3. **Mode System**: Essential for discoverability and power-user workflows
4. **Performance**: DirtyFlags + ScrollableTableState needed for large datasets
5. **Presentation**: Right-aligned numerics are standard for financial UIs

---

## References

- [Ticker Analysis (Archived)](../archive/TICKER_TUI_ANALYSIS.md)
- [Tickrs Repository](https://github.com/tarkah/tickrs)
- [Ratatui Documentation](https://ratatui.rs/)
- [TUI Architecture](../TUI_ARCHITECTURE.md)

---

**Last Updated**: 2026-03-26  
**Next Review**: After mode system implementation
