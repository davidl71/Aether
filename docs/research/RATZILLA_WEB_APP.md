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
