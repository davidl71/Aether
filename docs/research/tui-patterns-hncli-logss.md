# TUI Patterns Research: hncli & logss

**Sources:** 
- [pierreyoda/hncli](https://github.com/pierreyoda/hncli) - Hacker News TUI reader
- [todoesverso/logss](https://github.com/todoesverso/logss) - Log stream splitter/viewer

**Date:** 2026-03-26  
**Status:** Completed

## Overview

Analysis of two production Ratatui TUI applications to extract reusable patterns for Aether's TUI UX foundation work.

---

## hncli (Hacker News CLI)

### Architecture

```
src/
├── main.rs      # Entry point: terminal init, event loop
├── api.rs       # HackerNews API client (async)
├── app.rs       # Application state & business logic
├── config.rs    # Persistent settings
├── errors.rs    # Error types
└── ui/          # UI layer (components)
    ├── mod.rs
    ├── components/  # Reusable UI components
    └── screens/     # Screen-specific rendering
```

### Key Patterns

#### 1. Component-Based Architecture

```rust
// Trait-based component system
pub trait Component {
    fn init(&mut self) -> Result<()>;
    fn handle_events(&mut self, event: Option<Event>) -> Action;
    fn handle_key_events(&mut self, key: KeyEvent) -> Action;
    fn update(&mut self, action: Action) -> Action;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
```

**Benefits:**
- Each component encapsulates state, events, and rendering
- Fine-grained event handling
- Composition over inheritance
- Testable in isolation

#### 2. Action-Driven State Updates

```rust
pub enum Action {
    Quit,
    Tick,
    Navigate(NavigationAction),
    Update(UpdateAction),
    Noop,
}
```

**Flow:**
1. Event → `handle_events()` → Action
2. Action → `update()` → State mutation
3. State → `render()` → Frame

#### 3. Contextual Help System

- Help bar at bottom of screen (can be disabled)
- Context-sensitive: shows relevant keys per screen
- Global help screen (triggered by 'h')
- Persistent settings for user preferences

#### 4. Theming System

- Easily modifiable themes
- Stored in OS-appropriate config folder
- Runtime theme switching support

#### 5. Navigation History

- Remembers scroll position per story
- Returns to last-read comment on revisit
- State persistence across sessions

---

## logss (Log Stream Splitter)

### Key Features

| Feature | Pattern |
|---------|---------|
| **Multi-pane layout** | Horizontal/Vertical split containers |
| **Runtime config changes** | Add/remove containers on the fly |
| **Zoom mode** | Full-screen a single container |
| **Pause/Resume** | Freeze stream without stopping input |
| **Search/Filter** | Regex matching with auto-color assignment |
| **Output redirection** | Matched lines → files |
| **Shell triggers** | Execute commands on pattern match |
| **BarChart popup** | Live statistics overlay |
| **Line wrapping toggle** | Horizontal scroll vs wrap |
| **Show/Hide containers** | Toggle visibility per pane |

### Key Patterns

#### 1. Dynamic Container Management

```rust
// Containers can be added/removed at runtime
pub struct Container {
    id: u32,
    pattern: Regex,
    lines: Vec<Line>,
    visible: bool,
    color: Color,
}

impl Container {
    fn toggle_visibility(&mut self);
    fn clear(&mut self);
    fn push_line(&mut self, line: String);
}
```

**Relevance to Aether:** Similar to workspace tab management with dynamic pane addition/removal.

#### 2. Stream Processing with Backpressure

```rust
// Configurable render speed prevents UI freezing
pub struct Config {
    render_ms: u64,      // Throttle rendering
    threads_per_container: usize,
    exit_on_empty: bool,
}
```

**Relevance to Aether:** NATS message throttling, slow consumer handling.

#### 3. View Modes

```rust
pub enum ViewMode {
    Vertical,    // Side-by-side panes
    Horizontal,  // Stacked panes
    Single,      // Zoomed into one container
    Consolidated, // Merged view with highlights
}
```

**Relevance to Aether:** Workspace layout switching (dashboard vs positions vs composed).

#### 4. Shell Command Integration

```rust
pub struct Trigger {
    command: String,      // Template: "echo {line} >> /tmp/log"
    timeout_secs: u64,
    max_threads: usize,
}
```

**Relevance to Aether:** Operator-defined actions on alerts/signals.

#### 5. Configuration File Support

```yaml
# ~/.config/logss/config.yaml
containers:
  - pattern: "ERROR"
    color: red
    trigger: "notify-send 'Error found'"
  - pattern: "WARN"
    color: yellow
view_mode: vertical
render_speed: 100
```

---

## Patterns Applicable to Aether

### Immediate Implementations

| Pattern | Aether Use Case | Priority |
|---------|-----------------|----------|
| Contextual help bar | Operator guidance per tab | High |
| Global help screen ('h') | Shortcut reference | High |
| Zoom mode | Focus on single position/symbol | Medium |
| Pause/Resume | Freeze market data updates | Medium |
| View mode switching | Layout presets (dash/positions) | High |
| Container show/hide | Toggle sidebar panels | Medium |

### Architecture Insights

#### 1. Component Pattern Over Elm

Both hncli and logss use component traits rather than Elm Architecture.

**Comparison:**

| Aspect | Component (hncli/logss) | Elm (TEA) |
|--------|------------------------|-----------|
| State | Distributed per component | Centralized |
| Events | Component handles own | Routed through central update |
| Reusability | High (self-contained) | Medium (needs model slice) |
| Complexity | Lower (local reasoning) | Higher (message routing) |

**Recommendation:** Aether's current App struct pattern is closer to Elm. Consider component extraction for:
- Tab bar
- Status bar
- Position table
- Symbol watchlist

#### 2. Configuration Persistence

**Pattern:** OS-appropriate config folders

```rust
// Cross-platform config location
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aether")
        .join("config.toml")
}
```

**Aether application:**
- TUI theme preferences
- Column visibility
- Default workspace layout
- Watchlist symbols

#### 3. Action Enum for Decoupling

**Pattern:** Event → Action → Update → Render

```rust
pub enum Action {
    // Navigation
    NextTab,
    PrevTab,
    SelectPane(PaneId),
    
    // Data
    RefreshData,
    TogglePause,
    
    // UI
    ToggleHelp,
    ZoomPane(Option<PaneId>),
    
    // System
    Quit,
    Noop,
}
```

**Benefits:**
- Testable state transitions
- Undo/redo support
- Event logging/telemetry
- Time-travel debugging

#### 4. Container/Pane Abstraction

**logss pattern:**

```rust
pub struct Pane {
    id: PaneId,
    title: String,
    content: Box<dyn Widget>,
    visible: bool,
    zoomed: bool,
    constraints: Constraint,
}

impl Pane {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, event: Event) -> Action;
}
```

**Aether application:** Dynamic workspace composition with draggable/resizable panes.

---

## UI/UX Patterns

### 1. Help Integration

**hncli approach:**
```rust
// Bottom status bar with contextual help
fn render_help_bar(frame: &mut Frame, area: Rect, ctx: &Context) {
    let help_text = match ctx.current_screen {
        Screen::Home => "q:quit | j/k:nav | o:open | h:help",
        Screen::Story => "q:quit | ←:back | c:comments | h:help",
    };
    // Render at bottom
}
```

**Aether enhancement:** Dynamic help based on current mode (Navigation vs Edit vs View).

### 2. Zoom/Focus Mode

**logss approach:**
```rust
pub enum ViewState {
    Normal { visible_panes: Vec<PaneId> },
    Zoomed { focused_pane: PaneId },
}
```

**Aether application:** Focus on single position with full Greeks/history visible.

### 3. Pause Indication

**logss visual cue:**
```rust
fn render_pause_indicator(frame: &mut Frame, area: Rect, paused: bool) {
    if paused {
        let block = Block::default()
            .title(" PAUSED ")
            .border_style(Style::default().fg(Color::Yellow));
        // Render overlay
    }
}
```

**Aether application:** Freeze data updates, show stale data indicator.

---

## Code Structure Recommendations

### Current Aether Structure

```
src/
├── app.rs           # Large App struct (state + update + partial render)
├── input.rs         # Event handling
├── ui/mod.rs        # Rendering functions
└── workspace.rs     # Layout management
```

### Proposed Evolution

```
src/
├── main.rs
├── app.rs           # Reduced: state + action routing
├── action.rs        # Action enum definitions
├── components/      # Reusable UI components
│   ├── mod.rs
│   ├── tab_bar.rs
│   ├── status_bar.rs
│   ├── help_bar.rs
│   └── data_table.rs
├── panes/           # Pane implementations
│   ├── mod.rs
│   ├── positions.rs
│   ├── market_data.rs
│   └── alerts.rs
├── input/           # Input handling
│   ├── mod.rs
│   ├── navigation.rs
│   ├── edit.rs
│   └── view.rs
└── config.rs        # Persistent settings
```

---

## Implementation Priority

### Phase 1: UX Foundation (Current Sprint)

1. **Contextual help bar** - Bottom of screen, per-tab shortcuts
2. **Global help popup** - 'h' key, searchable shortcut reference
3. **Pause indicator** - Visual stale data warning

### Phase 2: Mode System

1. **Navigation mode** - Arrow keys move between panes/tabs
2. **Edit mode** - Form input with validation
3. **View mode** - Deep inspection, zoomed single item

### Phase 3: Workspace Evolution

1. **Pane abstraction** - Container pattern from logss
2. **Dynamic layout** - Runtime add/remove panes
3. **View presets** - Dashboard, Positions, Operations modes

---

## References

- [hncli Repository](https://github.com/pierreyoda/hncli)
- [hncli Architecture Blog Post](https://www.newstackwhodis.com/blog/hncli-2-architecture)
- [logss Repository](https://github.com/todoesverso/logss)
- [Ratatui Component Architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/)
- [Ratatui Application Patterns](https://deepwiki.com/ratatui/ratatui/4.2-application-structure-patterns)

---

## Related Tasks

- T-1774354599853391000: TUI UX foundation: discoverability, modes, terminal feedback primitives
- T-1774469201593212000: Implement terminal feedback primitives (toasts, status bar, errors, progress)
- T-1774469215296702000: Implement mode system (Navigation, Edit, View modes)
- T-1774469228746075000: Implement discoverability layer (shortcuts help, command palette, hints)
