# ratatui-interact Evaluation for Aether TUI

**Date**: March 2026  
**Status**: Complete (research + recommendation)  
**Tasks**: T-1774479830564215000 (original), **T-1774862718741621000** (research + recommendation; implementation out of scope per task)

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

## Interaction approaches (2–4 viable patterns)

| Approach | Mechanism | Fits Aether today? |
|----------|-----------|-------------------|
| **A. Central `App` + `Action` enum** | `crossterm` events → `shell_key_action` / tab handlers → `apply_shell_action`; workspace Tab cycles via `workspace_focus_target` (`input_shell.rs`) | **Default** — already ships; explicit and grep-friendly. |
| **B. ratatui-interact `FocusManager`** | Register focus IDs; Tab/BackTab call `next()`/`prev()`; dispatch keys by focused id | **Incremental** — reduces ad-hoc order for complex panes; migration cost across `input_*.rs`. |
| **C. Dedicated input crate (e.g. `tui-input`)** | Widget-local input state for text fields | **Pockets** — useful for loans/settings forms; not a global navigation replacement. |
| **D. Mouse-first / `ClickRegion` stacks** | Layer hit-testing on top of ratatui | **Optional** — operator console is keyboard-first; defer until product asks for mouse parity. |

**Recommendation (T-1774862718741621000):** Keep **A** as the backbone. Prototype **B** only for one vertical (e.g. Settings subsection list + editor) if Tab order bugs or focus drift become painful. Use **C** when adding a new multi-field form rather than rewriting global navigation.

---

## Current Aether TUI Architecture

### Focus Management (Current)

From `app.rs` and `workspace.rs`:

```rust
// Current: Manual focus tracking in App
pub struct App {
    pub active_tab: Tab,
    pub settings_section: SettingsSection,
    pub settings_health_focus: SettingsHealthFocus,
    // ... table scroll state, overlays, etc.
}

// Tab switching (includes Logs, Settings, …)
pub enum Tab {
    Dashboard, Positions, Charts, Orders, Alerts,
    Yield, Loans, DiscountBank, Scenarios, Logs, Settings,
}

// Settings section focus (see workspace.rs)
pub enum SettingsSection {
    Health,
    Config,
    Symbols,
    Sources,
    Alpaca,
}
```

**Tradeoffs**:
1. **Explicit state** — Easy to reason about; `focus_label()` reflects Settings nesting (`app.rs`).
2. **Workspace-aware Tab** — When `visible_workspace_spec()` is `Some`, Tab cycles **workspace tabs** only (`input_shell.rs`).
3. **No global FocusManager** — Order is encoded in `Tab::next` / workspace tab slices / settings enums.
4. **Mouse** — Not a primary target; keyboard and command palette (`:`) cover operators.

### Input Handling (Current)

From `input.rs` + `input_shell.rs`:

```rust
pub enum Action {
    Quit, ShowHelp, ToggleLogPanel,
    TabNext, TabPrev, WorkspaceFocusNext, WorkspaceFocusPrev,
    JumpToTab(u8),
    // ... many more
}
```

**Notes**:
1. Large `Action` enum is the **explicit command set** for the terminal UI (acceptable for a dense operator console).
2. **Context** enters via `InputMode` (`app.rs`) and `active_tab` before dispatch to `input_tabs`, `input_loans`, etc.
3. **Resize / key repeat** — Handled by ratatui loop + crossterm; no extra abstraction required unless profiling shows issues.

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

- ratatui-interact (repo): https://github.com/Brainwires/ratatui-interact
- ratatui-interact (crates.io): https://crates.io/crates/ratatui-interact
- ratatui (repo): https://github.com/ratatui/ratatui
- tui-tree-widget (crates.io): https://crates.io/crates/tui-tree-widget
- tui-input (crates.io): https://crates.io/crates/tui-input
- Current TUI audit: docs/TUI_AUDIT_2026_03_24.md
