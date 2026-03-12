# TUI Read-Path Audit

**Last updated**: 2026-03-11
**Purpose**: classify which legacy Python/Textual TUI read paths were already Rust-backed, which specialist paths remain intentionally Python-backed, and which migration candidates still matter for the current Rust TUI era.

## Current classification

### Already Rust-backed

- shared frontend read models now owned by the Rust TUI/backend path
  - unified positions
  - relationships
  - cash-flow timeline
  - opportunity simulation
- snapshot path when using the Rust API base

### Should remain Python-backed for now

- Benchmark and treasury-rate routes are now Rust-owned end to end.
  - The TUI should use the shared Rust origin; there is no separate benchmark service fallback path anymore.
  - still the active aggregated health source for the TUI
- broker and bank integration services
  - Discount Bank

### Remaining local/non-Rust read path

- local/manual loan JSON workflows are legacy fallback only
  - runtime loan CRUD goes through the Rust backend API

### TUI specialist presets that remain valid

- `rest_ib`

`rest_ib` now points at Rust-owned IB routes.
It should not expand into a general business-API proxy surface, a collection layer, or shared frontend read-model ownership.

Alpaca and Tastytrade are retired from the active runtime surface for now and are not active TUI specialist paths.

## Decision

- Public benchmark/risk-free-rate ownership and implementation have already moved to Rust.
- The **next Rust migration candidate for the TUI is Discount Bank or deeper Python read-model reduction**.
- Keep Python read paths limited to explicit specialist services; do not regrow Python into a general frontend backend.

## Recommended next migration

1. Finish hardening the Rust-owned loan read/write contract.
2. Keep the TUI loans tab on that Rust contract.
3. Preserve local JSON import/export only as a manual import path.

## Non-goals

- No attempt to revive the legacy Python/Textual TUI runtime.
- No change to broker/bank integration ownership in this slice.
