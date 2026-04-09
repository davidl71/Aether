# TUI Architecture

**Last updated:** 2026-04-05
**Service:** `agents/backend/services/tui_service`
**Framework:** ratatui + crossterm + tokio

---

## Overview

The TUI is a full-screen operator console for monitoring and controlling the financing platform. It renders live tabs for Dashboard, Positions, Orders, Alerts, Logs, Settings, and related instruments, and receives real-time data from the Rust backend via NATS JetStream snapshots.

Large-terminal operational workspaces are composed in `ui/mod.rs`. Shared workspace membership, focus cycling, and hint metadata now live in `workspace.rs`, while Settings-specific key handling lives in `input_settings.rs` and the Settings tab is split across `settings_*` section modules so composed layouts do not duplicate shell logic.

**Pane & focus:** See **[`TUI_PANE_MODEL.md`](./TUI_PANE_MODEL.md)** for the workspace / overlay / `InputMode` / `ScrollableTableState` model and the migration plan toward a central input router.

**Planning backlog:** **[`TUI_IMPLEMENTATION_BACKLOG.md`](./TUI_IMPLEMENTATION_BACKLOG.md)** — ScrollableTableState / toasts / numeric columns / pane regression checklist / layout helper map (task IDs inline).

---

## Current Architecture

### Main Loop

```
main.rs
├── Terminal setup (crossterm alternate screen)
├── NATS client connect
├── Spawn background tasks:
│   ├── snapshot_subscriber() → watch::Sender<Option<TuiSnapshot>>
│   ├── config_watcher()      → watch::Sender<TuiConfig>
│   └── health_checker()      → watch::Sender<BackendHealthState>
└── tokio::select! loop
    ├── crossterm::EventStream  → keyboard input
    ├── tick timer (250ms)      → app.tick() + terminal.draw()
    ├── config_rx               → config hot-reload
    ├── snapshot_rx             → data update
    └── health_rx               → backend connectivity
```

### App State

```rust
pub struct App {
    pub active_tab: Tab,
    pub snapshot: Option<TuiSnapshot>,
    pub show_help: bool,
    pub detail_popup: Option<DetailPopupContent>,
    pub split_pane: bool,
    pub positions_scroll: usize,
    pub orders_scroll: usize,
    pub roi_history: HashMap<String, VecDeque<f64>>,
    // ... channels
    snapshot_rx: watch::Receiver<Option<TuiSnapshot>>,
    config_rx: watch::Receiver<TuiConfig>,
    health_rx: watch::Receiver<BackendHealthState>,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
}

pub enum Tab {
    Dashboard, Positions, Charts, Orders, Alerts,
    Yield, Loans, DiscountBank, Scenarios, Logs, Settings,
}
```

`app.tick()` polls `has_changed()` on all `watch::Receiver`s — no wasted work between ticks.

### File Structure

```
tui_service/src/
├── main.rs               # entry point, main loop, terminal lifecycle
├── app.rs                # App struct, Tab enum, tick(), event dispatch
├── workspace.rs          # shared workspace/focus model and SettingsSection
├── events.rs             # AppEvent, ConnectionState, StrategyCommand
├── models.rs             # TuiSnapshot, display DTOs
├── config.rs             # TuiConfig (loaded from TOML)
├── config_watcher.rs     # hot-reload watcher
├── input_settings.rs     # Settings-only key handling and action application
├── theme_palette.rs      # UiPalette from TuiTheme (borders, shortcuts, pilot styling)
├── input/router.rs       # Global key routing (e.g. theme cycle before command palette)
├── nats.rs               # NATS snapshot subscriber
├── circuit_breaker.rs    # reconnect backoff
├── expiry_buckets.rs     # options expiry bucketing
└── ui/
    ├── mod.rs            # render dispatcher, status bar, hint bar, overlays
    ├── dashboard.rs      # Dashboard tab
    ├── positions.rs      # Positions tab
    ├── orders.rs         # Orders tab
    ├── alerts.rs         # Alerts tab + reusable alert view builder
    ├── logs.rs           # Logs tab + reusable logger widget builder
    ├── settings.rs       # Settings tab + reusable section renderers
    ├── settings_config.rs
    ├── settings_health.rs
    ├── settings_hint.rs
    ├── settings_sources.rs
    ├── settings_symbols.rs
    ├── yield_curve.rs    # Yield tab
    ├── loans.rs          # Loans tab
    └── discount_bank.rs  # Discount bank tab
```

---

## Theme and palette

- **Startup:** `TUI_THEME` selects the base palette (`default` or `high_contrast`); see `main.rs` module docs and Settings.
- **`UiPalette`** (`theme_palette.rs`) maps `TuiTheme` to Ratatui `Color` pairs so borders, status/hint styling, and pilot surfaces (for example Orders rollups) stay consistent when the theme changes.
- **Runtime cycle:** **Ctrl+T** cycles the in-process theme (macOS terminals may surface the same action as **⌘⇧T**). Routed in `input/router.rs`, applied via `Action::ThemeCycle` in `input_shell.rs`, and listed in the command palette (`discoverability.rs`, action `theme_cycle`). `dirty_flags` marks layout dirty so the next tick redraws with the new palette.

---

## Help overlay

The help overlay is a **centered, modal reference** for global and per-tab key bindings. It is the **authoritative in-app list** of what the TUI documents to the operator; this section describes behavior and where to edit it.

### Opening and closing

| Action | Keys |
|--------|------|
| Open | `?` (from `input_shell::global_key_action`), **Command palette → Show Help**, or macOS **⌘/** (maps to help, not chart search) |
| Close | **Any key** while the overlay is visible (`App::handle_key` in `InputMode::Help` clears `show_help` and returns) |

State flag: `App::show_help`. Rendering runs before other overlays when true (`ui/mod.rs` `render`).

### What it shows (summary)

Content is hard-coded in **`render_help_overlay`** in `ui/mod.rs` (not generated from `discoverability.rs`). Lines cover, in order:

- Global: quit, help, command palette (`:`), Tab / Shift-Tab (tabs vs workspace panes)
- Status bar modes (NAV / EDIT / VIEW), toasts
- macOS chords: ⌘, Settings; ⌘/ help; ⌘⇧P palette; ⌘0–⌘9; ⌘p split; ⌘r refresh; ⌘w close
- Digit jumps `1`–`9`, `0` → tab mnemonic map (Dash … Settings)
- Utilities: `M` mode cycle, **Ctrl+T** / **⌘⇧T** theme cycle, `` ` `` log panel, `g` tree panel, `p` split, `f` FMP vs refresh, `Esc` dismiss
- Composed workspaces (Market / Operations / Credit): Tab cycles **inner** panes when the workspace hint is active
- Per-area bindings: Dashboard/Positions/Alerts; Orders; Discount Bank; Loans (list, form, bulk path); Charts + Yield (shared line); Scenarios + Logs; Settings (including `0` / ⌘0 jump); exploration-mode letters **S T K O** / **X** (Orders)

The block uses `centered_rect(86, 34, …)` (percent of terminal). Footer: *Press any key to close*.

### Hint bar and discoverability (related, not the same)

| Surface | Role |
|---------|------|
| **Help overlay** | Full static cheat sheet; one place for all tabs |
| **Hint bar** (`render_hint_bar` in `ui/mod.rs`) | Short, **tab- and mode-aware** strip (e.g. Charts search shows Enter/Esc; Loans shows form vs import hints) |
| **Command palette** (`discoverability.rs`) | Searchable actions + key hints; includes **Show Help** |
| **`context_hints_for`** (`discoverability.rs`) | Optional extra hints for mode+tab (used if wired to UI); keep roughly aligned with overlay when you change bindings |

When you add or change a key binding, update **`render_help_overlay`** first, then the hint bar and/or `context_hints_for` if the shortcut is tab-specific.

---

## Data Flow

```
IBKR TWS
    └── backend_service (Axum :8080)
            └── NATS JetStream (LIVE_STATE KV)
                    └── snapshot_subscriber() [tokio::spawn]
                            └── watch::Sender<Option<TuiSnapshot>>
                                    └── app.tick() → has_changed()
                                            └── terminal.draw() [250ms tick]
```

---

## Planned Improvements

Based on research of [longbridge-terminal](../longbridge-terminal/) (T-1773941611822304000).

### Phase 1 — Dirty Flags (T-1773952110044212000, medium)

Eliminate unnecessary redraws. Currently the full frame is redrawn every 250ms tick regardless of data changes.

**New file:** `src/render/dirty_flags.rs`

```rust
use bitflags::bitflags;

bitflags! {
    pub struct DirtyFlags: u32 {
        const SNAPSHOT  = 0b0000_0001;
        const POSITIONS = 0b0000_0010;
        const ORDERS    = 0b0000_0100;
        const ALERTS    = 0b0000_1000;
        const CONFIG    = 0b0001_0000;
        const ALL       = 0xFFFF_FFFF;
    }
}

pub struct RenderState {
    dirty: DirtyFlags,
    last_render: std::time::Instant,
}

impl RenderState {
    pub fn mark_dirty(&mut self, flags: DirtyFlags) { self.dirty |= flags; }
    pub fn is_dirty(&self) -> bool { !self.dirty.is_empty() }
    pub fn clear(&mut self) { self.dirty = DirtyFlags::empty(); }
}
```

**Integration in `app.tick()`:**
```rust
if self.snapshot_rx.has_changed().unwrap_or(false) {
    self.snapshot = self.snapshot_rx.borrow_and_update().clone();
    self.render_state.mark_dirty(DirtyFlags::SNAPSHOT | DirtyFlags::POSITIONS | DirtyFlags::ORDERS);
}
```

**Integration in main loop:**
```rust
if app.render_state.is_dirty() || force_render {
    terminal.draw(|f| ui::render(f, &app))?;
    app.render_state.clear();
}
```

Expected: 40–60% reduction in render calls (Longbridge measured 70–80%).

### Phase 2 — Code Organization

The workspace/settings split described above is already implemented. Keep future key handling and pane-specific behavior in the dedicated modules instead of reintroducing tab-local shell logic.

**`src/input.rs`** (T-1773952110096607000) — extract keyboard handling from main loop:
```rust
pub fn handle_global_keys(key: KeyCode, app: &mut App) -> bool;   // tab/help/esc/quit
pub fn handle_tab_input(key: KeyCode, app: &mut App);              // dispatch per active_tab
fn handle_positions_keys(key: KeyCode, app: &mut App);
fn handle_orders_keys(key: KeyCode, app: &mut App);
// ...
```

**`src/ui/table_state.rs`** (T-1773952110143329000) — replace manual `usize` scroll fields:
```rust
pub struct ScrollableTableState {
    selected: usize,
    scroll: usize,
}
impl ScrollableTableState {
    pub fn move_up(&mut self);
    pub fn move_down(&mut self, max: usize);
    pub fn adjust_scroll(&mut self, visible_height: usize);
    pub fn selected(&self) -> usize;
    pub fn scroll(&self) -> usize;
}
```

**`src/ui/toast.rs`** (T-1773952110237059000) — transient notifications:
```rust
pub enum ToastLevel { Info, Success, Warning, Error }
pub struct Toast { message: String, created_at: Instant, ttl: Duration, level: ToastLevel }
// App gains: toast_queue: VecDeque<Toast>
// tick() expires old toasts; render shows latest in hint bar area
```

### Phase 3 — Table Polish (T-1773952110189533000, low)

- **Right-align numerics:** `format!("{:>10.2}", p.mark)` for mark, P&L, quantity, cost columns
- **Conditional row styling:**
  ```rust
  let style = if idx == app.positions_scroll { Style::default().bg(Color::DarkGray) }
              else if p.unrealized_pnl < 0.0  { Style::default().fg(Color::Red) }
              else                             { Style::default() };
  Row::new([...]).style(style)
  ```
- **Scrollbar widget** for visual scroll position feedback

---

## Comparison: Our TUI vs Longbridge Terminal

| Aspect | tui_service | Longbridge Terminal | Decision |
|--------|-------------|---------------------|----------|
| State management | `App` struct (owned) | Bevy ECS | Keep ours — simpler for linear tab bar |
| Tab routing | `Tab` enum + `next()`/`prev()` | `AppState` + OnEnter/OnExit handlers | Keep ours |
| Async data feed | `watch::Receiver` channels | `DashMap` + `mpsc` CommandQueue | Keep ours — more idiomatic |
| Render optimization | Full redraw every 250ms | `DirtyFlags` bitflags | **Adopt dirty flags** |
| Input handling | Inline in main loop | `handle_global_keys()` + per-state | **Adopt separation** |
| Table scroll state | Manual `usize` fields | `Select<T>` generic widget | **Adopt `ScrollableTableState`** |
| Transient notifications | `last_strategy_result` field | Toast pattern | **Adopt Toast** |
| Numeric alignment | Left (default) | Right-aligned | **Adopt right-alignment** |
| Row styling | Minimal (header bold only) | selected/up/down conditional | **Adopt conditional styles** |
| Global state | Owned by `App` | `OnceLock` + `DashMap` statics | Keep ours — safer, no hidden globals |

---

## References

- Longbridge Terminal source: `/longbridge-terminal/src/`
- Render optimization doc: `/longbridge-terminal/docs/render_optimization.md`
- rust-trade caching: `/rust-trade/trading-common/src/data/cache.rs`
- Task T-1773941611822304000 — Longbridge Terminal research
- Task T-1773939577056591000 — ui.rs vs ui/mod.rs dead code resolution
- `docs/ARCHIVED_INTEGRATION_PATTERNS.md` (consolidated from research docs)
