# Bevy ECS for TUI State Management Research

**Source:** longbridge-terminal (`src/app.rs`)  
**Date:** 2026-03-22  
**Status:** Completed

## Overview

longbridge-terminal uses Bevy ECS (v0.11) for TUI state management. Bevy is a game engine that provides Entity-Component-System architecture.

## Implementation Pattern

### App Setup

```rust
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

let mut app = bevy_app::App::new();
app.add_state::<AppState>()
    .add_event::<systems::Key>()
    .add_event::<systems::TuiEvent>()
    .init_resource::<Terminal>()
    .add_systems(Update, systems::render_watchlist.run_if(in_state(AppState::Watchlist)))
    .add_systems(OnEnter(AppState::Stock), systems::enter_stock);
```

### Key Concepts

| Concept | Purpose |
|---------|---------|
| **States** | App states (Watchlist, Stock, Portfolio) with transitions |
| **Resources** | Global singletons (Terminal, User, Watchlist) |
| **Systems** | Functions that run each frame, query entities |
| **Events** | Input events (Key press), TuiEvents |

### System Example

```rust
fn render_watchlist(
    terminal: Query<&mut Terminal>,
    stocks: Res<STOCKS>,
) {
    // Render logic
}
```

## Evaluation for Aether

| Aspect | Assessment |
|--------|------------|
| **Complexity** | High - new paradigm to learn |
| **Decoupling** | Excellent - systems are independent |
| **Testability** | Good - systems can be tested in isolation |
| **Integration** | Hard - ratatui rendering would need wrapping |

### Pros for Aether
- Clean separation of concerns
- Automatic state transition handling
- Parallel system execution
- Good for complex UI with many components

### Cons for Aether
- Significant learning curve
- Overkill for simpler TUI
- Current ratatui approach works well
- ECS concepts may confuse team

## Recommendation

**Not Recommended** - Aether's TUI is well-structured with ratatui. Bevy ECS would add significant complexity without proportional benefit. The longbridge-terminal use case benefits from ECS because:
1. Many entity types (stocks, watchlists, accounts)
2. Complex state transitions
3. Multiple concurrent views

Aether's TUI has simpler state requirements.

## Related Tasks

- T-1774192022865695000: Research Bevy ECS for TUI (Done)
