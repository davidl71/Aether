# TUI Implementation Patterns Research

**Status**: Implementation In Progress  
**Sources**: ticker (Go/bubbletea), tickrs (Rust/tui-rs), longbridge-terminal patterns  
**Applies to**: Aether Rust/Ratatui TUI

---

## Implementation Status

| Task | Pattern | Status | Location |
|------|---------|--------|----------|
| T-UX-001 | AppMode system (Navigation/Edit/View) | ✅ Implemented | `mode.rs`, `app.rs` |
| T-UX-002 | ScrollableTableState | ✅ Implemented | `scrollable_table.rs` |
| T-UX-003 | Numeric right-alignment | 🔄 In Progress | - |
| T-UX-004 | DirtyFlags optimization | ⏳ Planned | - |
| T-UX-005 | Position grouping | ⏳ Planned | - |
| T-UX-006 | Summary statistics | ⏳ Planned | - |
| T-1774469201593212000 | Toast notifications | ✅ Implemented | `ui/feedback.rs` |
| T-1774469228746075000 | Command palette | ✅ Implemented | `discoverability.rs` |
| T-1774528154110853000 | macOS shortcuts | ✅ Implemented | `input.rs` |
| T-NEW | Mouse support | ✅ Implemented | `mouse.rs` |
| T-1774481655277034000 | Alpaca health | ✅ Implemented | `alpaca_health.rs` |

---

## Implemented Patterns

### 1. Toast Notification System ✅

**Location**: `ui/feedback.rs`  
**Usage**: `app.toast_manager.info("Message")`

```rust
pub struct ToastManager {
    toasts: VecDeque<Toast>,
    next_id: u64,
}

pub enum ToastLevel {
    Info, Success, Warning, Error
}
```

**Features**:
- Auto-expiring notifications (4-6 seconds)
- Stacking (max 3 visible)
- Severity levels with colors
- Integrated into status bar

---

### 2. AppMode System ✅

**Location**: `mode.rs`, integrated in `app.rs`  
**Status Bar**: Shows `NAV`/`EDIT`/`VIEW` with colors

```rust
pub enum AppMode {
    Navigation,  // Tab switching, scrolling (green)
    Edit,        // Form input, text editing (yellow)
    View,        // Detail views, read-only (cyan)
}
```

**Mode Transitions**:
- Edit mode: Settings forms, loan forms, search inputs
- View mode: Help overlay, detail popups, log panel
- Navigation mode: Default for tab switching/scrolling

**Key Binding**: `m` to cycle modes

---

### 3. Command Palette ✅

**Location**: `discoverability.rs`  
**Trigger**: `:` key

```rust
pub struct CommandPalette {
    visible: bool,
    search: String,
    selected: usize,
    commands: Vec<Command>,
}
```

**Features**:
- Searchable command list
- Keyboard navigation (up/down)
- Context-aware shortcuts
- Integrated with action system

---

### 4. macOS-Friendly Shortcuts ✅

**Location**: `input.rs`, `input_shell.rs`

**Standard Shortcuts**:
- `d` - Delete in Settings watchlist (in addition to Delete key)
- `Space` - Toggle combo view in Positions

**Cmd (Super) Key Shortcuts**:
- `Cmd+1-0` - Jump to tabs
- `Cmd+Q` - Quit
- `Cmd+W` - Close detail/help/log panel
- `Cmd+P` - Toggle split pane
- `Cmd+R` - Force snapshot

---

### 5. Mouse Support ✅

**Location**: `mouse.rs`

**Features**:
- Mouse wheel scroll in Positions/Orders/Dashboard tabs
- Click on tabs to switch
- Click on content to open detail views
- Automatic mouse capture enable/disable

**Implementation**:
```rust
pub fn handle_mouse_event(app: &App, mouse: MouseEvent, area: Rect) -> Option<Action>
```

---

### 6. ScrollableTableState ✅

**Location**: `scrollable_table.rs`

```rust
pub struct ScrollableTableState {
    offset: usize,
    selected: usize,
    total_rows: usize,
    visible_rows: usize,
}
```

**Features**:
- Scroll up/down with selection tracking
- Page up/down navigation
- Go to top/bottom
- Automatic offset adjustment
- Efficient visible range calculation

**Note**: Module created, full integration into Positions/Orders tabs requires refactoring existing scroll handling.

---

### 7. Alpaca Health Monitoring ✅

**Location**: `alpaca_health.rs`

```rust
pub struct AlpacaHealthMonitor {
    paper_health: AlpacaHealth,
    live_health: AlpacaHealth,
}
```

**Features**:
- Periodic health checks (30s interval)
- Credential validation
- Account info display (equity, buying power)
- Status bar indicators (`A` for paper, `P` for live)
- Color-coded status (green=ok, red=error, gray=not configured)

---

## Remaining Implementation

### T-UX-003: Numeric Right-Alignment

**Pattern**: Financial data tables with right-aligned numerics
**Benefit**: Easy comparison of decimal places

**Implementation Plan**:
```rust
// Table cells with numeric alignment
Cell::from(format!("{:>10.2}", price))  // Right-align, 2 decimals
Cell::from(format!("{:>8}", quantity))   // Right-align, integer
```

**Files to Update**:
- `ui/positions.rs` - Price, quantity, PnL columns
- `ui/orders.rs` - Price, quantity columns
- `ui/scenarios.rs` - Strike, price columns

---

### T-UX-004: DirtyFlags Render Optimization

**Pattern**: Only re-render changed regions
**Benefit**: 60fps with large datasets

**Implementation Plan**:
```rust
pub struct DirtyFlags {
    tabs: bool,
    content: bool,
    status_bar: bool,
    sidebar: bool,
}
```

**Integration**:
- Mark dirty on state changes
- Check flags before rendering each region
- Clear flags after render

---

### T-UX-005: Position Grouping

**Pattern**: Group positions by strategy/expiration
**Benefit**: Better organization for complex portfolios

**Implementation Plan**:
```rust
pub struct PositionGroup {
    key: (account, strategy, expiration),
    positions: Vec<Position>,
    expanded: bool,
    total_value: f64,
}
```

**Features**:
- Expandable/collapsible groups
- Group-level statistics
- Keyboard navigation between groups

---

### T-UX-006: Summary Statistics

**Pattern**: Portfolio-level aggregations
**Benefit**: High-level portfolio overview

**Implementation Plan**:
```rust
pub struct PortfolioSummary {
    total_value: f64,
    total_pnl: f64,
    pnl_percent: f64,
    by_strategy: HashMap<String, StrategySummary>,
}
```

**Display**: Add to Dashboard tab

---

## Architecture Insights

### Strengths of Current Implementation

1. **NATS-based Architecture**: Push model superior to polling
2. **Async Event Loop**: Non-blocking with tokio::select!
3. **Modular Input System**: Separate handlers per tab/mode
4. **Type-Safe Actions**: Action enum prevents invalid states

### Performance Considerations

- Current: Full redraw every tick (250ms)
- Future: DirtyFlags for selective redraw
- Memory: ScrollableTableState for large datasets

### UX Patterns Learned

1. **Discoverability**: Command palette (`:`) more discoverable than `?` help
2. **Mode Indicators**: Visual mode indicator (NAV/EDIT/VIEW) reduces confusion
3. **Context Hints**: Show available keys based on current context
4. **Immediate Feedback**: Toast notifications for async operations

---

## References

- [Ticker Analysis (Archived)](../archive/TICKER_TUI_ANALYSIS.md)
- [Tickrs Repository](https://github.com/tarkah/tickrs)
- [Ratatui Documentation](https://ratatui.rs/)
- [TUI Architecture](../TUI_ARCHITECTURE.md)

---

**Last Updated**: 2026-03-26  
**Next Review**: After T-UX-003 completion
