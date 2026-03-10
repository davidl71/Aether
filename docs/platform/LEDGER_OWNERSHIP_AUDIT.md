# Ledger Ownership Audit

**Last updated**: 2026-03-11
**Purpose**: document current durable-state ownership and the remaining dual-writer risk around loans and ledger-adjacent data.

## Current ownership

- **Rust SQLite ledger** is the target durable system of record for backend transactional state.
- **Go** does not write ledger state; it owns collection, `LIVE_STATE`, and QuestDB fanout.
- **Python** no longer owns frontend read-model HTTP APIs, but it still owns one local durable workflow:
  - `python/tui/components/loan_entry.py`
  - this reads and writes `config/loans.json`
- **Native C++** still has a matching local loan persistence path:
  - `native/src/loan_manager.cpp`
  - this also reads and writes the same `config/loans.json` contract

## What is actually dual-writer today

The current dual-writer issue is not Rust and Python both writing the same SQLite database. It is:

- **Python/Textual TUI loan editor** writing `config/loans.json`
- **native C++ `LoanManager`** writing `config/loans.json`

That file is a local/manual durable store that sits outside the Rust ledger path.

## Current writer classification

### Active durable writers

- `native/src/loan_manager.cpp`
  - writes `config/loans.json`
- `python/tui/components/loan_entry.py`
  - writes `config/loans.json`
- Rust ledger crates
  - own SQLite-backed ledger/database state

### Read-only or non-ledger components

- TUI tabs that only consume snapshot/read-model data
- web frontend read-model consumers
- Go collector/gateway components

## Decision

- **Rust remains the intended single durable backend owner.**
- **Python loan CRUD is a temporary local/manual workflow, not the long-term ledger owner.**
- **C++ loan JSON persistence is compatibility legacy and should not expand.**

## Migration order

1. Keep Python loan editing explicitly documented as a local file-based workflow.
2. Define a Rust-owned loan API or import path for the same data model.
3. Switch TUI loan CRUD from local JSON writes to the Rust-owned contract.
4. Remove Python JSON writer behavior.
5. Remove or archive native C++ JSON loan persistence once Rust is the only durable owner.

## Defaults for future work

- Do not add new Python or C++ writers to `config/loans.json`.
- Do not introduce a second durable database for loans.
- Treat `config/loans.json` as a transitional/manual-import format until Rust owns loan CRUD.
