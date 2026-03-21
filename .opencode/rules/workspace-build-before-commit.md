---
description: Run workspace build before committing; never commit with cross-crate type errors
alwaysApply: false
---

# Workspace Build Before Commit

**Rule:** Before committing, run `cargo build` at the **workspace root** (`agents/backend/`), not just a single crate. Stale incremental caches can hide cross-crate type errors.

## Why

`cargo build -p <crate>` only checks the dependency graph for that crate. If a shared dependency (e.g. `common`, `api`) changes but the dependent crate hasn't been recompiled yet, the single-crate build succeeds using stale artifacts. This hides type mismatches between crates.

**Real example:** After moving TWS position logic, `tws_positions.rs` and `tws_market_data.rs` were left orphaned (not declared as `mod` in `main.rs`). A single-crate build of `backend_service` wouldn't catch orphaned files — only files actually compiled as part of the binary are checked. Use `rg "mod tws_positions" services/backend_service/src/` to verify modules are wired before assuming they're covered by build checks.

## Commands

| Scenario | Command |
|---|---|
| Before commit | `cd agents/backend && cargo build` |
| Quick check (single crate) | `cd agents/backend && cargo build -p <crate>` |
| After changing shared types (`common`, `api`) | `cd agents/backend && cargo build` (always) |

## When to Run

- **Before every commit** — full workspace build
- **After modifying any shared crate** (`common`, `api`, `broker_engine`) — always use workspace build
- **After `cargo clean`** — rebuild workspace, not single crate

## Anti-Pattern

```bash
# ❌ WRONG — single crate, can miss cross-crate errors
cargo build -p backend_service && git commit

# ✅ CORRECT — workspace build catches all type errors
cargo build && git commit
```
