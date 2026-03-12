# Research: Rust Core Implementation Patterns

## Overview

Research into Rust trading engines to identify implementable patterns for our Rust core - without splitting languages (unlike NautilusTrader's Python/Rust split).

## Relevant Projects

| Project | Stars | Focus |
|--------|-------|-------|
| **nautilus_trader** | 21.1k | Full platform (Python + Rust) |
| **OrderBook-rs** | 306 | Lock-free order book |
| **matching-core** | 134 | Matching engine core |
| **rust_bt** | 56 | Backtesting engine |

## Key Patterns for Our Use Case

### 1. Box Spread Specific (Your Focus)

What we already have (C++):
- Box spread calculations
- Greeks (delta, gamma, theta, vega)
- Risk calculations
- Option chain data

**What could be added:**
- Event-driven architecture (like NautilusTrader)
- Position tracking with real-time P&L
- Order management with state machine
- Margin requirement calculations

### 2. OrderBook-rs Patterns (Useful?)

**Lock-free architecture:**
- Atomics for concurrent access
- Multiple order types (not needed for box spreads)
- Thread-safe price levels

**Not needed:** We don't do order book matching - we do box spreads which are synthetic financing.

### 3. Event-Driven Architecture

From NautilusTrader:
```rust
pub enum Event {
    OrderSubmitted,
    OrderAccepted,
    OrderFilled,
    OrderCancelled,
    PositionChanged,
}
```

**Could implement:**
- Event sourcing for positions
- State machine for order lifecycle
- Audit trail for compliance

### 4. Backtesting (rust_bt)

```rust
// From rust_bt - deterministic backtesting
pub trait Strategy {
    fn on_bar(&mut self, bar: &Bar);
    fn on_tick(&mut self, tick: &Tick);
}
```

**Could add later:**
- Historical backtesting for box spread strategies
- Walk-forward analysis

## Recommendation: What's Worth Implementing

### High Priority (For Our Use Case)

1. **Event-driven position tracking**
   - Replace mutable state with event sourcing
   - Deterministic replay capability
   
2. **Order state machine**
   - Track order lifecycle (submit → accept → fill → cancel)
   - Handle rejections gracefully

3. **Margin calculator improvements**
   - Real-time margin checking
   - Reg-T vs portfolio margin

4. **Position P&L tracking**
   - Real-time unrealized/realized P&L
   - By strategy, by underlying

### Medium Priority

5. **Backtesting framework**
   - Historical box spread analysis
   - Walk-forward validation

### Low Priority

6. **Lock-free data structures** - Not needed for our scale
7. **Order book** - Not our use case

## Architecture Pattern

```
┌─────────────────────────────────────────────────┐
│                  Rust Core                       │
├─────────────────────────────────────────────────┤
│  Events ──► Position Engine ──► Risk Engine      │
│     │              │                │           │
│     ▼              ▼                ▼           │
│  EventStore    Positions        MarginCalc      │
├─────────────────────────────────────────────────┤
│  IBKR Adapter  │  NATS Publisher  │  TUI View   │
└─────────────────────────────────────────────────┘
```

## Next Steps

- [ ] Implement event-driven position tracking in Rust
- [ ] Add order state machine
- [ ] Enhance margin calculations
- [ ] Add real-time P&L tracking
