# TUI Architecture

**Last updated:** 2026-03-19
**Service:** `agents/backend/services/tui_service`
**Framework:** ratatui + crossterm + tokio

---

## Overview

The TUI is a full-screen terminal dashboard for monitoring and controlling the box spread platform. It renders 5 live tabs (Dashboard, Positions, Orders, Alerts, Logs) and receives real-time data from the Rust backend via NATS JetStream snapshots.

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
    Dashboard, Positions, Orders, Alerts,
    Yield, Loans, Scenarios, Logs, Settings,  // ← 4 not yet live in ui.rs
}
```

`app.tick()` polls `has_changed()` on all `watch::Receiver`s — no wasted work between ticks.

### File Structure

```
tui_service/src/
├── main.rs               # entry point, main loop, terminal lifecycle
├── app.rs                # App struct, Tab enum, tick(), event dispatch
├── events.rs             # AppEvent, ConnectionState, StrategyCommand
├── models.rs             # TuiSnapshot, display DTOs
├── config.rs             # TuiConfig (loaded from TOML)
├── config_watcher.rs     # hot-reload watcher
├── nats.rs               # NATS snapshot subscriber
├── circuit_breaker.rs    # reconnect backoff
├── expiry_buckets.rs     # options expiry bucketing
└── ui/
    ├── mod.rs            # render dispatcher, status bar, hint bar, overlays
    ├── dashboard.rs      # Dashboard tab
    ├── positions.rs      # Positions tab
    ├── orders.rs         # Orders tab
    ├── alerts.rs         # Alerts tab
    └── logs.rs           # Logs tab (tui-logger)
```

**Dead code note:** `ui/mod.rs` references Yield/Loans/Scenarios/Settings tabs that exist in `ui/mod.rs` (old) but are not rendered by the live `ui.rs`. See T-1773939577056591000 for resolution.

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

### Phase 2 — Code Organization (low priority)

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
