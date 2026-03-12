# TUI Read-Path Audit

**Last updated**: 2026-03-12
**Purpose**: Record which read paths are Rust-backed and which open items remain.
The Python/Textual TUI (`python/tui/`) has been archived. Active frontend is the
Ratatui TUI (`agents/backend/crates/tui/`).

## Migration status — all complete

| Area | Status | Notes |
|------|--------|-------|
| Unified positions, relationships, cash-flow, opportunity simulation | **Rust ✅** | Served from Rust API |
| Snapshot path | **Rust ✅** | `snapshot.{backend_id}` via NATS |
| Benchmark / treasury-rate routes | **Rust ✅** | Rust-owned end to end; no Python fallback |
| Loan CRUD | **Rust ✅** | `LoanRepository` (SQLite); `/api/v1/loans*`; legacy JSON seed-on-empty |
| Discount Bank | **Rust ✅** | `/api/balance`, `/api/transactions`, `/api/bank-accounts` via `DiscountBankParser` |
| IB positions | **Rust ✅** | `/api/v1/ib/positions` |

## Specialist presets

- `rest_ib` — points at Rust-owned IB routes. Keep scope narrow (not a general proxy).
- Alpaca and Tastytrade — retired from active runtime surface.

## Open items

None. All read paths have migrated to Rust. Python layer is archived.

## Non-goals

- No attempt to revive the legacy Python/Textual TUI.
- No change to broker/bank integration ownership.
