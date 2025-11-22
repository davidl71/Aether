# TUI Multiscreen Support Research

**Date:** 2025-11-18
**Purpose:** Research multiscreen TUI patterns for displaying all tabs simultaneously, similar to btop, zellij, ATAC, lazygit, lazydocker, k9s

---

## Executive Summary

**Key Finding:** FTXUI (our current TUI library) has **native multiscreen support** via `ResizableSplit` components. We can implement multiscreen layouts without switching libraries.

**Recommendation:** Replace single-screen tab switching with split-pane layouts using FTXUI's `ResizableSplitLeft/Right/Top/Bottom()` components, allowing multiple tabs to be visible simultaneously.

---

## Current Implementation Analysis

### Architecture Overview

**Library:** FTXUI (C++)
**Location:** `native/src/tui_app.cpp`
**Current Pattern:** Single-screen with tab switching

**Current Implementation:**

```cpp
// Current: Only one tab visible at a time
int selected_tab = 0;
auto tab_container = Container::Tab({
  dashboard,
  positions,
  historic,
  orders,
  alerts,
}, &selected_tab);
```

**Tabs Available:**

1. Dashboard - Symbol table with metrics
2. Current Positions - Open box spreads
3. Historic Positions - Recently closed spreads
4. Orders - Order timeline
5. Alerts - Alert feed

**Layout Structure:**

```
┌─────────────────────────────────┐
│ Header (status, time, metrics)   │
├─────────────────────────────────┤
│ Tabs (Dashboard | Positions...) │
├─────────────────────────────────┤
│ Content (single tab visible)    │
├─────────────────────────────────┤
│ Footer (keyboard shortcuts)     │
└─────────────────────────────────┘
```

**Limitations:**

- Only one tab visible at a time
- No split-screen or pane support
- Users must switch tabs to see different information
- Inefficient use of terminal space on large screens

---

## Reference Application Patterns

### 1. Zellij (Terminal Multiplexer)

**Pattern:** Multiple panes with resizable splits

**Key Features:**

- Multiple panes displayed simultaneously
- Resizable splits (keyboard + mouse)
- Tab management within panes
- Session persistence

**Layout Example:**

```
┌─────────────┬─────────────┐
│  Pane 1     │  Pane 2     │
│  (Tab A)    │  (Tab B)    │
├─────────────┼─────────────┤
│  Pane 3     │  Pane 4     │
│  (Tab C)    │  (Tab D)    │
└─────────────┴─────────────┘
```

**Relevance:** Perfect reference for multiscreen TUI architecture

---

### 2. Lazygit (Git TUI)

**Pattern:** Side-by-side panes with multiple views

**Key Features:**

- Multiple panes showing different Git views
- Tab navigation within panes
- Efficient terminal space usage
- Keyboard-driven navigation

**Layout Example:**

```
┌─────────────┬─────────────┐
│  Files      │  Diff View   │
│  (staged)   │  (changes)   │
├─────────────┴─────────────┤
│  Commit History           │
│  (log)                    │
└───────────────────────────┘
```

**Relevance:** Real-world example of multiscreen TUI with multiple data sources

---

### 3. K9s (Kubernetes TUI)

**Pattern:** Multiple resource views in split panes

**Key Features:**

- Multiple panes for different Kubernetes resources
- Split views for detailed information
- Efficient navigation patterns
- Real-time updates across panes

**Relevance:** Complex multiscreen layout patterns

---

### 4. Btop (Resource Monitor)

**Pattern:** Multiple sections displayed simultaneously

**Key Features:**

- CPU, memory, disk, network displayed at once
- Multiple sections in single view
- Real-time updates across all sections
- Efficient information density

**Relevance:** Pattern for displaying multiple data sources simultaneously

---

### 5. Lazydocker (Docker TUI)

**Pattern:** Multi-pane interface for container management

**Key Features:**

- Containers, images, logs in separate panes
- Side-by-side views
- Efficient terminal space usage

**Relevance:** Similar use case (monitoring multiple data sources)

---

## FTXUI Multiscreen Capabilities

### ResizableSplit Components

**Source:** [FTXUI Component Documentation](https://github.com/arthursonzogni/ftxui/blob/main/doc/module-component.md)

**Available Components:**

- `ResizableSplitLeft()` - Split with divider on left
- `ResizableSplitRight()` - Split with divider on right
- `ResizableSplitTop()` - Split with divider on top
- `ResizableSplitBottom()` - Split with divider on bottom

**Features:**

- Mouse-controllable split position
- Keyboard navigation support
- Automatic resize handling
- Nested splits supported

**Example Usage:**

```cpp
#include <ftxui/component/component.hpp>

using namespace ftxui;

// Create two panes side-by-side
auto left_pane = RenderDashboard();
auto right_pane = RenderPositions();

// Create resizable split
auto split = ResizableSplitLeft(left_pane, right_pane);
```

---

### Container Components

**Available Containers:**

- `Container::Horizontal()` - Horizontal layout
- `Container::Vertical()` - Vertical layout
- `Container::Tab()` - Tab switching (current implementation)

**Layout Elements:**

- `hbox()` - Horizontal box
- `vbox()` - Vertical box
- `gridbox()` - Grid layout
- `flex` decorator - Flexible sizing

**Example:**

```cpp
// Horizontal layout with flexible elements
hbox({
  text("left") | border,
  text("middle") | border | flex,
  text("right") | border | flex,
});
```

---

## Recommended Implementation Approach

### Phase 1: Basic Split Layout

**Goal:** Display 2 tabs simultaneously in split panes

**Implementation:**

1. Replace `Container::Tab()` with `ResizableSplitLeft()` or `ResizableSplitRight()`
2. Display Dashboard in left pane, Positions in right pane
3. Add pane focus management (similar to tab selection)
4. Extend keyboard shortcuts for pane navigation

**Layout:**

```
┌─────────────────┬─────────────────┐
│  Dashboard      │  Positions      │
│  (symbols)      │  (open spreads) │
└─────────────────┴─────────────────┘
```

---

### Phase 2: Multi-Pane Layout

**Goal:** Display 4 tabs simultaneously in 2x2 grid

**Implementation:**

1. Use nested splits (horizontal + vertical)
2. Create 2x2 grid layout
3. Display Dashboard, Positions, Orders, Alerts
4. Add pane focus cycling

**Layout:**

```
┌─────────────┬─────────────┐
│  Dashboard  │  Positions  │
├─────────────┼─────────────┤
│  Orders     │  Alerts     │
└─────────────┴─────────────┘
```

---

### Phase 3: Advanced Features

**Goal:** User-configurable layouts

**Features:**

- Save/restore layout preferences
- Keyboard shortcuts for layout presets
- Dynamic pane creation/destruction
- Tab assignment to panes

---

## Technical Implementation Details

### Integration Points

**File:** `native/src/tui_app.cpp`

**Key Changes Required:**

1. **Replace Tab Container (Lines 163-169):**

```cpp
// OLD: Single tab container
auto tab_container = Container::Tab({...}, &selected_tab);

// NEW: Split panes
auto left_pane = RenderDashboard();
auto right_pane = RenderPositions();
auto split = ResizableSplitLeft(left_pane, right_pane);
```

1. **Modify Main Renderer (Lines 174-227):**

```cpp
// OLD: Single content area
Element content = RenderDashboard()->Render();

// NEW: Split layout
Element content = split->Render();
```

1. **Extend Event Handling (Lines 230-444):**

```cpp
// Add pane focus management
int focused_pane = 0;  // 0 = left, 1 = right
// Add keyboard shortcuts for pane navigation
```

1. **Reuse Existing Renderers:**

- No changes needed to individual tab renderers
- They can be used directly in panes

---

### State Management

**Current State:**

- `selected_tab` - Which tab is visible
- `selected_dashboard_row_` - Row selection within tab
- `selected_positions_row_` - Row selection within tab
- etc.

**New State Needed:**

- `focused_pane` - Which pane has focus
- `pane_layout` - Layout configuration (2-pane, 4-pane, etc.)
- `pane_tabs` - Which tab is displayed in each pane

---

### Data Flow

**Current Flow:**

```
Provider → Snapshot (atomic) → Renderers → FTXUI Screen
```

**Multiscreen Flow (No Changes Needed):**

```
Provider → Snapshot (atomic) → Multiple Renderers → Split Layout → FTXUI Screen
```

**Note:** The atomic snapshot system is perfect for multiscreen - multiple panes can safely read the same snapshot simultaneously.

---

## Comparison with Alternatives

### FTXUI (Current - Recommended)

**Pros:**

- ✅ Already integrated
- ✅ Native multiscreen support
- ✅ C++ native performance
- ✅ Cross-platform
- ✅ Active maintenance

**Cons:**

- ⚠️ Requires code changes (but minimal)

**Verdict:** **Best choice** - native support, already integrated

---

### Ratatui (Rust)

**Pros:**

- ✅ Excellent multiscreen support
- ✅ Modern API

**Cons:**

- ❌ Different language (Rust vs C++)
- ❌ Would require complete rewrite
- ❌ Not applicable to our C++ codebase

**Verdict:** Not applicable

---

### tview (Go)

**Pros:**

- ✅ Good multiscreen support
- ✅ Rich widgets

**Cons:**

- ❌ Different language (Go vs C++)
- ❌ Would require complete rewrite
- ❌ Not applicable to our C++ codebase

**Verdict:** Not applicable

---

## Implementation Roadmap

### Step 1: Research & Design ✅

- [x] Research multiscreen patterns
- [x] Analyze current implementation
- [x] Document findings

### Step 2: Basic Split Layout

- [ ] Implement 2-pane split (Dashboard + Positions)
- [ ] Add pane focus management
- [ ] Test keyboard navigation
- [ ] Verify data updates work correctly

### Step 3: Multi-Pane Layout

- [ ] Implement 2x2 grid layout
- [ ] Add pane focus cycling
- [ ] Test all tabs in panes
- [ ] Verify performance

### Step 4: Advanced Features

- [ ] Add layout configuration
- [ ] Implement save/restore preferences
- [ ] Add keyboard shortcuts for layouts
- [ ] Documentation

---

## References

### FTXUI Documentation

- [FTXUI Component Documentation](https://github.com/arthursonzogni/ftxui/blob/main/doc/module-component.md)
- [FTXUI Layout Examples](https://github.com/arthursonzogni/ftxui/blob/main/examples/dom/CMakeLists.txt)
- [FTXUI ResizableSplit API](https://github.com/arthursonzogni/ftxui/blob/main/doc/module-component.md)

### Reference Applications

- [Zellij Terminal Multiplexer](https://www.ssp.sh/brain/tuis/)
- [Lazygit Multi-Pane Interface](https://learn.omacom.io/books/2/pages/59)
- [K9s Kubernetes TUI](https://www.ssp.sh/brain/tuis/)
- [Btop Resource Monitor](https://learn.omacom.io/books/2/pages/59)

### TUI Libraries

- [Rust TUI Tabs Widget](https://docs.rs/tui/latest/tui/widgets/struct.Tabs.html)
- [FTXUI GitHub Repository](https://github.com/arthursonzogni/ftxui)

---

## Conclusion

**FTXUI has native multiscreen support** via `ResizableSplit` components. We can implement multiscreen layouts similar to zellij, lazygit, k9s, and btop without switching libraries or major architectural changes.

**Recommended Next Steps:**

1. Implement basic 2-pane split layout
2. Test with Dashboard + Positions
3. Extend to 2x2 grid layout
4. Add advanced features (layout config, save/restore)

**Estimated Effort:**

- Basic 2-pane: 2-4 hours
- 2x2 grid: 4-6 hours
- Advanced features: 6-8 hours
- **Total: 12-18 hours**

---

**Document Status:** ✅ Research Complete
**Next Action:** Implementation planning and design document
