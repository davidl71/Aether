# TUI workspace layout specs: Market and Operations

**Date:** 2026-03-30  
**Tasks:** T-1774864455252255000 (Market), T-1774864479163446000 (Operations)  
**Scope:** Design notes only (no implementation change implied).  
**Implementation references:** `agents/backend/services/tui_service/src/workspace.rs`, `ui/mod.rs`, `input_shell.rs`, `app.rs`.

---

## Shared behavior

- **Workspace detection:** `App::visible_workspace()` picks `Market`, `Operations`, or `Credit` when the terminal is at least `WorkspaceSpec::min_width` × `min_height` and `active_tab` is one of that workspace’s tabs (`workspace.rs`).
- **Tab / BackTab in workspace mode:** When `visible_workspace_spec()` is `Some`, `shell_key_action` maps **Tab** / **Shift+Tab** to `WorkspaceFocusNext` / `WorkspaceFocusPrev`, which cycles **only** among `spec.tabs` (`input_shell.rs` → `workspace_focus_target`).
- **Plain Tab when not in a tile workspace:** Falls through to global tab next/prev across the full tab bar.
- **Focus label:** Banner shows `app.focus_label()` — tab title, or for **Settings** the nested `SettingsSection` / `SettingsHealthFocus` title (`app.rs`).

---

## Market workspace

**Tabs (order):** Dashboard → Positions → Charts → Orders → Yield (`MARKET_WORKSPACE_TABS`).  
**Minimum size:** 170×22 (`workspace.rs`).

### Target composed layout (panes)

Rendered by `render_market_workspace` (`ui/mod.rs`):

| Region | Split | Pane | Responsibility |
|--------|-------|------|----------------|
| Top row | 50% vertical split of body | 52% / 48% horizontal | **Dashboard** (left), **Positions** (right) |
| Bottom row | remaining 50% | 48% / 52% horizontal | **Orders** (left), **Yield curve** (right) |

**Minimum widths (design intent):** Treat **~170 columns** as the floor for all four panes to remain usable; below that the UI falls back out of `Market` workspace to single-tab navigation.

### Focus order (Tab / BackTab)

Cycles: **Dashboard → Positions → Charts → Orders → Yield** (order of `MARKET_WORKSPACE_TABS`).

**Implementation nuance:** While `active_tab == Tab::Charts`, `render_main` still calls `render_market_workspace` (four quadrants: Dashboard, Positions, Orders, Yield). There is **no** full-screen charts pane in tile mode—the Charts tab advances focus in the cycle and drives which tab owns input (e.g. chart search / keys) while the same four panes stay on screen. Full-screen charts appear only in **non-tile** mode via `render_tab_panel`.

### Scroll ownership

- **Per-tab state** in `App` (e.g. `dashboard_table`, `positions_table`, orders table state, yield UI state). Only the **active** tab’s primary widget receives arrow/Page key scrolling from input dispatch (existing `input_*.rs` behavior).
- **Design expectation:** When focus is **Dashboard**, vertical scroll applies to the dashboard table; when **Positions**, to positions; **Orders** → orders table; **Yield** → yield panel. For **Charts** in tile mode, chart-specific keys apply while the quadrant layout remains unchanged visually.

---

## Operations workspace

**Tabs (order):** Alerts → Logs → Settings (`OPERATIONS_WORKSPACE_TABS`).  
**Minimum size:** 170×20 (`workspace.rs`).

### Target composed layout (panes)

Rendered by `render_operations_workspace` (`ui/mod.rs`):

| Column | Share | Content |
|--------|-------|---------|
| Left | 38% width | **Alerts** (top 42% height), **Logs** (bottom 58%) |
| Right | 62% width | **Settings** (full height; internal sections via `settings_layout`) |

### Focus order (Tab / BackTab)

Cycles: **Alerts → Logs → Settings**.

### Scroll ownership

- **Alerts:** `alerts_scroll` (`ScrollableTableState`) when Alerts tab is active.
- **Logs:** Logger widget state (e.g. scroll position / filter) when Logs tab is active.
- **Settings:** Section list vs editor vs health sub-focus (`SettingsSection`, `SettingsHealthFocus`) — already modeled as `secondary_focus()`; arrow keys navigate within the active settings subsection.

**Design expectation:** Only one pane’s scroll pipeline is active at a time, keyed off **`active_tab`**, not mouse hover (keyboard-first).

---

## Implementation follow-ups (separate tasks)

- **Per-pane scroll independence** (if product wants scroll without changing tab): would require a **secondary focus dimension** for “which physical pane is focused” inside Operations/Market tiles—not in this design note’s scope.
