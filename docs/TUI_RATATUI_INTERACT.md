# Ratatui interaction: research note (Aether TUI)

**Task:** T-1774862718741621000 — identify viable approaches for “interact” (focus, keys, mouse, components) aligned with the current `tui_service` architecture.

**References (ecosystem):**

- [ratatui-interact on crates.io](https://crates.io/crates/ratatui-interact) — interactive widgets, focus, click regions.
- [ratatui_interact on docs.rs](https://docs.rs/ratatui-interact) — `FocusManager`, components, events traits.

---

## 1. Existing local patterns (codebase survey)

| Concern | Where | Pattern |
|--------|--------|---------|
| Async event loop | `services/tui_service/src/main.rs` | `EventStream` + `tokio::select!` multiplexes keys, mouse, tick, and NATS/async channels; keys call `app.handle_key`, mouse calls `mouse::handle_mouse_event` → `app.handle_action`. |
| Key → action | `input.rs`, `input_*.rs` | Large `Action` enum; mode-specific dispatch (`input_tabs`, `input_shell`, `input_loans`, …). |
| Focus / modality | `app.rs` | `InputMode` (Normal, LoanForm, ChartSearch, OrdersFilter, …) drives `handle_key` branches; `app_mode_for_input_mode` maps to `mode::AppMode` (Navigation / Edit / View). |
| Mouse | `mouse.rs` | Manual `Layout::split` mirrors `ui` geometry (tabs, market 2×2); returns `Option<Action>`. Duplication risk when layout drifts from render. |
| Render | `ui/mod.rs` | Single `render(f, app)` tree; `needs_redraw` + dirty regions in `app.rs` / `dirty_flags.rs`. |
| Future direction | `main.rs` (`run_loop` comment) | Explicit TODO: per-component routing could adopt ratatui async-template / component model later. |

**Optional crate:** `ratatui-interact = { version = "0.5.1", optional = true }` in `tui_service/Cargo.toml`. The `tui-interact` feature wires `mod tui_interact` (shim re-exports), `mod field_list_focus` (shared `FocusManager` helper), and `mod chart_search_interact` (type alias) in `main.rs`.

---

## 1b. Implemented overlays (shared `FieldListFocus`)

With `--features tui-interact`:

- **Build / run:** `cargo build -p tui_service --features tui-interact` / `cargo run -p tui_service --features tui-interact`
- **Shared helper:** `field_list_focus.rs` — `FieldListFocus` wraps `FocusManager` for a **field** vs **list** pair (`FieldListRegion::Field` / `List`): `on_open` / `on_close`, `tab_next` / `tab_prev`, `allows_field_edit`.
- **Charts tab — symbol search** (`/`): **Tab** / **Shift+Tab** move focus between the **query** field and the **results** list. On **list** focus, typed characters and Backspace are `NoOp` so **Up/Down** move the selection without mutating the query. `App::chart_search_interact` is a type alias to `FieldListFocus`. Routing: `input_tabs.rs`, `input_views.rs`.
- **Orders tab — filter** (`/`): same pattern between the **filter line** and the **orders table**. Routing: `input_tabs.rs` (`orders_key_action`), `input_views.rs` (`orders_filter_interact`).
- **Command palette** (`:` / **⌘⇧P**): **Tab** / **Shift+Tab** between **search** and **command list**; on list focus, typing does not edit the search buffer; **Up/Down** still move the highlighted command. Routing: `input.rs` (`handle_command_palette_input`), `input_shell.rs` (`command_palette_interact`).
- **Loans tab — bulk import path** (`b` / `i`): **Tab** / **Shift+Tab** between the **path field** and the **loans list**; on list focus, **↑↓** / **PgUp/PgDn** scroll the list (`LoansScroll*`). Routing: `input_tabs.rs` (`loans_key_action`), `input_loans.rs` (`loan_import_interact`).

Default builds (no feature) keep prior behavior (e.g. Tab in chart search falls through to global tab switching; no sub-focus in orders filter, palette, or loan import).

---

## 2. Viable approaches (2–4)

### A. Stay on the current pipeline (manual)

Keep `Action` + `InputMode` + manual mouse hit-testing. Improve by centralizing layout rects (single source of truth for render + mouse), tests for hit regions.

- **Pros:** No new abstraction; matches exploration-mode complexity today.  
- **Cons:** Focus/tab order and new interactive surfaces stay ad hoc; easy to desync mouse vs draw.

### B. Incremental `ratatui-interact` (recommended direction)

Keep `run_loop` and `App` ownership. Behind `--features tui-interact`, introduce `FocusManager` / click registry **only** for bounded surfaces (e.g. loan form, settings sub-form, orders filter) while global chrome (tabs, workspace switch) stays on `Action`.

- **Pros:** Aligns with upstream interactive primitives; reduces custom tab-order code where forms grow.  
- **Cons:** Two systems coexist until pilots expand; need clear boundaries (which `InputMode`s use RI).

### C. Full component architecture (ratatui async-template style)

Restructure around nested components that own state + event handling.

- **Pros:** Scales to many panes/dialogs; matches `run_loop` TODO.  
- **Cons:** High churn; fights current monolithic `App` + `ui::render`; not minimal.

### D. Mouse-only `ratatui-interact`

Use `ClickRegionRegistry` / mouse helpers from the crate; keep keyboard path on `input.rs`.

- **Pros:** Addresses drift between `mouse.rs` and `ui` without touching keymaps.  
- **Cons:** Half-integration; focus story still split.

---

## 3. Recommendation

**Adopt B (incremental `ratatui-interact`)** with these rules:

1. **Shim wired:** `#[cfg(feature = "tui-interact")] mod tui_interact`, `mod field_list_focus`, and `mod chart_search_interact` (type alias) in `main.rs` (not a default feature).  
2. **Pilots (done):** chart search, orders filter, command palette, loans import path — all use `FieldListFocus` / `FocusManager` behind `tui-interact`; **loan form** (`LoanForm`) remains on plain Tab field navigation unless extended later.  
3. **Mouse:** optionally register the same rects used for rendering for the pilot pane to retire duplicate `Layout` in `mouse.rs` for that path only.  
4. **Do not** adopt full component tree (C) until pane EPIC (e.g. central router) lands.  
5. **Resize / repeat / focus churn:** keep handling `KeyEventKind::Press` only (already in `run_loop`); on resize, re-register or invalidate RI regions from the same layout pass as `ui::render`.

---

## 4. Follow-up work

1. **Status bar hints (done):** With `--features tui-interact`, [`FocusContext`](../agents/backend/services/tui_service/src/focus_context.rs) carries `field_list_subfocus`; [`context_hints_for`](../agents/backend/services/tui_service/src/discoverability.rs) shows **Tab** → list/field and **↑↓** on list where applicable (chart search, orders filter, palette, loan import).  
2. Optional `app_tests` key sequences for Tab order when the feature is enabled in CI.  
3. Optional: extend **loan form** (`LoanForm`) with `FieldListFocus` if a list-adjacent sub-pane is added.

---

## 5. Testing notes

- **Existing:** `app_tests.rs` uses `TestBackend`, `Terminal`, buffer snapshots — good pattern for focus/order regressions.  
- **Manual:** resize terminal during pilot; rapid key repeat on Tab; switch tab while in Edit mode (ensure focus reset or global precedence matches `InputMode`).

---

## 6. “Interact” clarified from codebase

In Aether TUI today, *interact* means: **modal focus** (`InputMode` / `AppMode`), **keyboard** (`Action` dispatch), **mouse** (tabs, scroll, some clicks), and **no** separate per-widget ratatui state machine yet — all merged in `App` + `mouse.rs`.
