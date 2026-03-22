# Bevy ECS for TUI State Management Research

**Source:** longbridge-terminal (`app.rs`)  
**Date:** 2026-03-22  
**Status:** Todo

## Overview

longbridge-terminal uses Bevy ECS (v0.11) for TUI state management. Bevy is primarily a game engine but its ECS pattern applies well to complex UI state.

## longbridge-terminal Pattern

```rust
// app.rs uses Bevy ECS
use bevy_app::{App, AppExit, Schedules};
use bevy_ecs::prelude::*;

struct AppState { /* ... */ }

// Systems process entities and update UI
fn handle_input_system(/* ... */) { /* ... */ }
fn update_ui_system(/* ... */) { /* ... */ }

// Main loop
let mut app = App::new();
app.add_systems(Update, (handle_input_system, update_ui_system));
```

## Key Benefits

1. **Decoupled Systems** - Each system handles one concern
2. **Entity-Component Model** - Stocks, positions, orders as entities with components
3. **Parallel Execution** - Systems can run in parallel when dependencies allow
4. **Event System** - Bevy's event bus for pub/sub

## Aether Current State

- Ratatui-only in `tui_service`
- Manual state management with channels
- No ECS pattern

## Evaluation Criteria

- [ ] Complexity overhead vs benefits
- [ ] Integration with existing ratatui rendering
- [ ] Learning curve for team
- [ ] Performance implications

## Related Tasks

- T-1774192022865695000: Research Bevy ECS for TUI state management
