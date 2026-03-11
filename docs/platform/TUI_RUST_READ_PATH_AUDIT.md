# TUI Read-Path Audit

**Last updated**: 2026-03-11
**Purpose**: classify which Python/Textual TUI read paths are already Rust-backed, which should remain Python-backed, and which are the next migration candidates.

## Current classification

### Already Rust-backed

- shared frontend read models in `python/tui/app.py`
  - unified positions
  - relationships
  - cash-flow timeline
  - opportunity simulation
- snapshot path when using the Rust API base

### Should remain Python-backed for now

- `python/integration/risk_free_rate_service.py`
  - benchmark and treasury-rate logic remains Python-specific
- `python/services/health_dashboard.py`
  - still the active aggregated health source for the TUI
- broker and bank integration services
  - IB
  - Alpaca
  - Tastytrade
  - Discount Bank

### Remaining local/non-Rust read path

- `python/tui/components/loan_entry.py`
  - loans tab reads local `config/loans.json`
  - this is the main remaining TUI durable-state path outside Rust

### TUI Python-backed paths that still justify gateway convenience

- `rest_ib`
- `rest_alpaca`
- `rest_tastytrade`

These remain valid because they expose specialist Python-backed broker snapshots.
They should not expand into a general business-API proxy surface, a collection layer, or shared frontend read-model ownership.

## Decision

- Do **not** migrate risk-free-rate or health-dashboard in the next slice.
- The **next Rust migration candidate for the TUI is loans**, not benchmarks or health.
- Keep the Go gateway only as a convenience entrypoint for these still-separate specialist services.
- Keep Python read paths limited to explicit specialist services; do not regrow Python into a general frontend backend.

## Recommended next migration

1. Define a Rust-owned loan read/write contract.
2. Make the TUI loans tab consume that contract.
3. Preserve local JSON import/export only as a manual import path.

## Non-goals

- No change to the active Python/Textual TUI runtime.
- No change to broker/bank integration ownership in this slice.
