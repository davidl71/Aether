# Ledger Ownership Audit

**Last updated**: 2026-03-11
**Purpose**: document current durable-state ownership and the remaining migration work around loans and ledger-adjacent data.

## Current ownership

- **Rust SQLite ledger** is the target durable system of record for backend transactional state.
- **Go** does not write ledger state; it owns collection, `LIVE_STATE`, and QuestDB fanout.
- **Rust backend** now owns active loan CRUD through the backend API/store.
- **Python** TUI loan UI is now a client of the Rust loan API by default.
- **Native C++** retains loan calculation/data structures, but its JSON persistence path is retired.

## What changed

- The prior dual-writer issue around `config/loans.json` has been narrowed:
  - Python TUI now prefers Rust `/api/v1/loans`
  - Rust owns the active loan store path
  - native C++ `LoanManager` no longer persists to JSON

`config/loans.json` remains only as a transitional legacy seed/import format.

## Current writer classification

### Active durable writers

- `agents/backend/crates/api/src/loans.rs`
  - owns backend loan store and REST CRUD
- Rust ledger crates
  - own SQLite-backed ledger/database state

### Read-only or non-ledger components

- TUI tabs that only consume snapshot/read-model data
- web frontend read-model consumers
- Go collector/gateway components

## Decision

- **Rust remains the intended single durable backend owner.**
- **Python loan CRUD has been moved behind the Rust API.**
- **C++ loan JSON persistence is retired.**

## Migration order

1. Keep `config/loans.json` only as a legacy import/seed format.
2. Move backend loan persistence from transitional JSON storage to final Rust-owned durable storage.
3. Remove remaining legacy references that imply Python/C++ are active loan writers.

## Defaults for future work

- Do not add new Python or C++ writers for loan state.
- Do not introduce a second durable database for loans.
- Treat `config/loans.json` as a transitional/manual-import format only.
