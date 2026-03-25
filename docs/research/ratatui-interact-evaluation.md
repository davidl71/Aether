# ratatui-interact Evaluation for Aether TUI

**Date**: March 2026  
**Status**: Evaluation in Progress  
**Task**: T-1774479830564215000

---

## Executive Summary

ratatui-interact provides 30+ interactive TUI components with focus management and mouse support. This evaluation assesses whether adopting it would improve Aether's TUI architecture.

**Preliminary Verdict**: **Selective Adoption Recommended**

- ✅ Use: FocusManager for tab navigation
- ✅ Use: LogViewer for consolidated Logs/Alerts tab
- ✅ Use: Toast widget (already implemented custom, but ratatui-interact's is more feature-rich)
- ⚠️ Evaluate: TreeView for account hierarchy vs tui-tree-widget
- ❌ Avoid: Full component migration (too disruptive at current stage)

---

## Current Aether TUI Architecture

### Focus Management (Current)

From `app.rs` and `workspace.rs`:

```rust
// Current: Manual focus tracking in App
pub struct App {
    pub active_tab: Tab,
    pub secondary_focus: SecondaryFocus,
    pub settings_health_focus: SettingsHealthFocus,
    // ... more focus state
}

// Tab switching
pub enum Tab {
    Dashboard, Positions, Charts, Orders, Alerts,
    Yield, Loans, DiscountBank, Scenarios, Logs, Settings,
}

// Settings section focus
pub enum SettingsSection {
    Health,
    BackendConfig,
    SymbolConfig,
}
```

**Issues with Current Approach**:
1. **Scattered focus state** - Spread across App, workspace, and input handlers
2. **Manual tab order** - No centralized focus navigation
3. **No mouse support** - Keyboard-only navigation
4. **Inconsistent patterns** - Each tab implements focus differently

### Input Handling (Current)

From `input.rs`:

```rust
pub enum Action {
    Quit, ShowHelp, ToggleLogPanel,
    TabNext, TabPrev, JumpToTab(u8),
    // ... 100+ actions
}
```

**Issues**:
1. **Action explosion** - 100+ actions in single enum
2. **No context awareness** - Actions don't know their focus context
3. **Manual dispatch** - Each handler manually checks focus state

---

## ratatui-interact Components Analysis

### 1. FocusManager

**Features**:
- Tab/Shift+Tab navigation
- Generic over focusable element types
- Integration with ClickRegion for mouse support

```rust
// ratatui-interact pattern
#[derive(Clone, Eq, PartialEq, Hash)]
enum Element {
    NameInput,
    EnableCheckbox,
    SubmitButton,
}

let mut focus = FocusManager::new();
focus.register(Element::NameInput);
focus.register(Element::EnableCheckbox);
focus.register(Element::SubmitButton);

// In event loop
Event::Key(key) if is_tab(key) => focus.next(),
Event::Key(key) if is_shift_tab(key) => focus.prev(),
```

**Aether Application**:

```rust
// Proposed: Unified focus management
#[derive(Clone, Eq, PartialEq, Hash)]
enum TuiFocus {
    // Tab level
    TabBar,
    
    // Dashboard
    DashboardPositions,
    DashboardChart,
    
    // Positions
    PositionsList,
    PositionsDetail,
    
    // Yield
    YieldSymbolSelector,
    YieldCurveTable,
    
    // Settings
    SettingsSectionList,
    SettingsEditor,
}

impl App {
    pub focus: FocusManager<TuiFocus>,
}
```

**Benefits**:
- ✅ Centralized focus tracking
- ✅ Consistent Tab/Shift+Tab behavior
- ✅ Type-safe focus elements
- ✅ Easier to add mouse support later

### 2. LogViewer

**Features**:
- Scrollable log viewer
- Line numbers
- Search functionality
- Log-level coloring

**Current Aether Logs Tab**:

From audit (TUI_AUDIT_2026_03_24.md):
- Custom log widget implementation
- No Home/End shortcuts
- Paragraph `.scroll()` can go past last alert
- No integrated search

**Comparison**:

| Feature | Current | ratatui-interact LogViewer |
|---------|---------|---------------------------|
| Scrollback | Basic | Full with line numbers |
| Search | None | Built-in |
| Log levels | Manual styling | Automatic coloring |
| Integration | Custom | Standardized |
| Maintenance | Ongoing | Community maintained |

**Recommendation**: Replace custom log widget with LogViewer

### 3. Toast Notifications

**Current**: Custom implementation added 2026-03-24:

```rust
pub enum ToastLevel { Info, Warning, Error }
pub toast_queue: VecDeque<ToastMessage>,
const TOAST_TTL_SECS: u64 = 3;
```

**ratatui-interact Toast**:
- More styling options
- Auto-expiry
- Multiple toast types
- Better integration with focus system

**Verdict**: Current implementation is fine. Could migrate to ratatui-interact Toast if more features needed.

### 4. TreeView

**Features**:
- Collapsible tree view
- Selection
- Customizable rendering

**Comparison with tui-tree-widget**:

| Feature | ratatui-interact TreeView | tui-tree-widget |
|---------|--------------------------|-----------------|
| Downloads | Lower (part of ratatui-interact) | 548K+ (standalone) |
| Focus integration | Native | Manual |
| Mouse support | Built-in | Manual |
| Performance | Unknown | Proven at scale |
| Community | Smaller | Larger |

**Recommendation**: Evaluate tui-tree-widget first (T-1774479849756339000)

### 5. Input Widgets

**ratatui-interact provides**:
- Input (text)
- TextArea (multi-line)
- CheckBox
- Select (dropdown)
- Button

**Current Aether**:
- Custom input handling in loans/settings
- No reusable input components

**Verdict**: Could use for standardization, but current approach works. Not urgent.

---

## Integration Strategy

### Phase 1: FocusManager (High Priority)

**Scope**: Replace manual focus tracking with FocusManager

**Files to Modify**:
- `app.rs` - Add FocusManager field
- `workspace.rs` - Remove secondary focus tracking
- `input.rs` - Simplify action dispatch
- All `input_*.rs` - Use focus context

**Migration Plan**:

```rust
// Step 1: Add to App
pub struct App {
    pub focus: FocusManager<TuiFocus>,
    // ... remove secondary_focus, settings_health_focus, etc.
}

// Step 2: Define focus hierarchy
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TuiFocus {
    // Global
    TabBar,
    
    // Per-tab focus
    Dashboard,
    PositionsList,
    PositionsDetail,
    // ... etc
}

// Step 3: Update input handlers
fn handle_key(app: &mut App, key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Tab => {
            app.focus.next();
            None
        }
        KeyCode::BackTab => {
            app.focus.prev();
            None
        }
        _ => dispatch_by_focus(app, key)
    }
}
```

**Effort**: 2-3 days
**Risk**: Medium (touch focus system)
**Benefit**: High (simplifies architecture)

### Phase 2: LogViewer (Medium Priority)

**Scope**: Replace custom log/alerts with LogViewer

**Deliverables**:
1. Prototype LogViewer integration
2. Compare performance with current
3. Migrate if acceptable

**Effort**: 1-2 days
**Risk**: Low (isolated component)
**Benefit**: Medium (better UX)

### Phase 3: Evaluation (Low Priority)

- TreeView vs tui-tree-widget
- Input widgets standardization
- Other components as needed

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking focus behavior | High | Extensive testing, feature flags |
| Performance regression | Medium | Benchmark before/after |
| Learning curve | Low | Good documentation |
| Dependency maintenance | Low | Active project |

---

## Recommendation

**Proceed with Phase 1 (FocusManager) and Phase 2 (LogViewer)**

1. **Immediate**: Create prototype branch with FocusManager
2. **Test**: Verify all tab navigation still works
3. **Iterate**: Fix issues, refine focus hierarchy
4. **Merge**: When confident
5. **Follow-up**: Evaluate LogViewer migration

---

## Related Tasks

- T-1774479830564215000: This evaluation
- T-1774479836566898000: tui-logger evaluation (alternative to LogViewer)
- T-1774479849756339000: tui-tree-widget evaluation
- T-1774459957300872000: Current focus model work
- T-1774354599853391000: UX foundation

---

## Links

- ratatui-interact: https://github.com/Brainwires/ratatui-interact
- Current TUI audit: docs/TUI_AUDIT_2026_03_24.md
