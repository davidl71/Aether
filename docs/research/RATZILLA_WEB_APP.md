# Research: Ratzilla as Future Web App

## Overview

Evaluate using Ratzilla (Ratatui + WebAssembly) to replace the retired React web app.

## Background

- **React web app** - Retired, deleted from repo
- **Rust TUI** - Active, uses Ratatui (`tui_service`)
- **Ratzilla** - Run Ratatui TUIs in browser via WASM

## Ratzilla

**What it does:**
- Compile Ratatui TUI to WebAssembly
- Runs in browser with same Rust code as native
- Powered by Ratatui (your TUI already uses it)

**Requirements:**
- `wasm32-unknown-unknown` target
- `trunk` build tool
- Same Ratatui widgets work in browser

## Pros

1. **Single codebase** - Native TUI + Web TUI from same code
2. **Pure Rust** - No JavaScript/React needed
3. **Lightweight** - Minimal WASM bundle
4. **Consistent** - Same UI everywhere

## Cons

1. **Browser limitations** - No true terminal features (256-color, true color support varies)
2. **Input handling** - Keyboard/mouse differs from native
3. **Initial load** - WASM download + compile time
4. **No real terminal** - Emulated terminal in DOM

## Comparison

| Aspect | React Web (retired) | Ratzilla |
|--------|---------------------|----------|
| Codebase | JS/TS + React | Pure Rust |
| Rendering | DOM | Canvas/DOM emulated |
| Development | Hot reload | Trunk watch |
| Performance | Good | Good |
| Native feel | Good | Moderate |

## Recommendation

**Keep as option for future** - Not urgent:
- Your native Rust TUI works well
- Ratzilla adds complexity
- Browser TUI experience differs from native
- Can add later when/if web UI needed again

## Tasks

- [ ] Revisit if web UI needed again
- [ ] Evaluate Ratatui v2 features (better browser support)
- [ ] Consider minimal landing page instead of full TUI in browser

---

# Research: Finance TUI Apps Built with Ratatui

## GitHub Projects Found

| Project | Stars | Description |
|---------|-------|-------------|
| **budget_tracker_tui** | 174 | Budget tracker - income/expenses visualization |
| **rust-tui-dashboard** | 0 | US + TW stocks with Yahoo Finance API |
| **arbitrage-scouter** | 1 | Triangular arbitrage detector (async) |
| **EnvelopeCLI** | 2 | Zero-based budgeting, bank import, encryption |

## Lessons from rust-tui-dashboard

**Features (F1-F8):**
- F1: Dashboard View - Real-time tracking of holdings, daily P&L, best/worst performers
- F2: Transaction Management - Create, view, delete buy/sell via TUI
- F3: Multi-Portfolio Support - Switch between portfolios
- F4: Rebalancing Engine - Compare current vs target allocations
- F5: Watchlist Tracking - Track symbols alongside holdings
- F6: Search & Filtering - Filter transaction history by ticker
- F7: Config Hot-Reload - Auto-detect config.json changes
- F8: Performance - Local price caching (30s TTL), Circuit Breaker pattern

**Key takeways:**
- Config hot-reload is valuable
- Price caching with TTL prevents rate limit issues
- Multi-portfolio support is a good pattern

## Lessons from arbitrage-scouter

**Architecture:**
- Multi-task concurrency (WebSocket, arbitrage detection, TUI rendering)
- Shared state with `Arc<RwLock<T>>` - read-optimized locking
- tokio::select! for task coordination
- Graceful shutdown propagation

**Design Decisions:**
- Use `RwLock` (not Mutex) for read-heavy workloads
- Use `anyhow::Result` for application errors
- tokio-tungstenite for WebSocket (native async/await)
- Ratatui for flicker-free rendering

**Key takeways:**
- `Arc<RwLock>` pattern is battle-tested for async Rust
- Circuit breaker pattern prevents cascade failures from API rate limits

## Recommendation

These projects validate that Ratatui is production-ready for finance apps. Key patterns to adopt:
1. Config hot-reload (F7 from rust-tui-dashboard)
2. Price caching with TTL (F8)
3. Circuit breaker pattern for external APIs
4. `Arc<RwLock<T>>` for shared state in async context

---

# Research: Yahoo Finance as Data Source

## Crate

**yahoo-finance** (crates.io: 0.3.0, last updated 2020)
- GitHub: github.com/fbriden/yahoo-finance-rs

## Features

- Historical OHLCV pricing
- Real-time streaming quotes (via WebSocket)
- Symbol profile info

## Usage

```rust
use yahoo_finance::history;
let data = history::retrieve("AAPL").unwrap();
```

## Pros

- Free (public data)
- No API key needed
- Good for testing/demo

## Cons

- Rate limited by Yahoo
- Delayed data (15 min)
- Not reliable for production trading
- Crate outdated (last update 2020)

## Comparison with IBKR

| Aspect | Yahoo Finance | IBKR API |
|--------|--------------|----------|
| Cost | Free | Free |
| API Key | Not needed | Not needed |
| Data | Delayed (15 min) | Real-time |
| Reliability | Medium | High |

## Recommendation

**Use as fallback, not primary:**
- Primary: IBKR API
- Fallback: Yahoo Finance (demo, testing, when IBKR unavailable)

Add price caching with TTL to handle rate limits.

---

# Research: Lessons from Rust Trading Platform Rebuild

## Source

Blog post: "I spent 2 years rebuilding my algorithmic trading platform in Rust. I have no regrets."
- Author: Austin Starks (NexusTrade)
- Original platform: NextTrade (TypeScript) → Rust

## Why Rust (vs TypeScript/Go/C++)

| Previous | Issue |
|----------|-------|
| TypeScript | Too slow, crashed during genetic optimizations (days to run) |
| Go | Would always wonder "could it be faster?" |
| C++ | Would crash constantly from segfaults |

## Results

- **Performance gain**: Genetic optimizations went from **days → seconds**
- **Scalability**: Platform now handles complex strategies
- **Maintenance**: Easier than expected after learning curve

## Key Takeaways

### Why Rust was worth it:
- Enums + pattern matching: "descended from the heavens"
- No NullPointerExceptions - type safety
- Lightning fast performance
- Easier maintenance than expected

### Tips for Rust:
1. **Use enums correctly** - Rust enums can hold data, unlike other languages
2. **Don't over-abstract** - Helper functions are harder in Rust  
3. **Use AI help** - LLMs help once you understand basics
4. **Use clone()** - Performance impact minimal, makes code work

## Validation

This validates our C++ → Rust direction:
- Trading logic performance gains are significant
- Type safety prevents runtime crashes
- Maintainability improves after learning curve
