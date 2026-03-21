---
description: New position/order/account types must have From conversions to existing types; no orphaned duplicate types
alwaysApply: false
---

# No New Duplicate Types

**Rule:** Before adding a new `*Position*`, `*Order*`, `*Account*`, or `*Metrics*` struct, check if an existing type can be reused. If a new type is needed, add at minimum one `From` conversion to or from an existing type. No orphaned types without conversions.

## Existing Position Types (do not add new ones without justification)

| Type | Location | Purpose |
|------|----------|---------|
| `broker_engine::domain::Position` | `broker_engine/src/domain.rs` | Broker-agnostic position with contract, qty, avg_price, mktval |
| `common::PositionSnapshot` | `common/src/snapshot.rs` | NATS/proto snapshot (id, symbol, qty, cost_basis, mark, unrealized_pnl, account_id, source) |
| `api::IbPositionDto` | `api/src/ib_positions.rs` | Client Portal REST normalized position |
| `api::RuntimePositionDto` | `api/src/runtime_state.rs` | Enriched runtime view (+ position_type, strategy, apr_pct, market_value) |
| `api::RuntimePositionState` | `api/src/runtime_state.rs` | Intermediate state type for position lifecycle |

## Existing Account/Metrics Types

| Type | Location | Purpose |
|------|----------|---------|
| `broker_engine::domain::AccountInfo` | `broker_engine/src/domain.rs` | Raw broker account data (net_liquidation, cash_balance, buying_power, etc.) |
| `common::Metrics` | `common/src/snapshot.rs` | API snapshot metrics (net_liq, buying_power, excess_liquidity, margin_requirement, commissions, *_ok flags) |

## Pattern: Adding a New Snapshot Type

If you need a new snapshot field (e.g. `OpenOrderSnapshot`):

1. **Check if `common::OrderSnapshot` or existing type can be extended**
2. **If new type is needed**, place it in `common/src/snapshot.rs` if used by 2+ crates
3. **Add `From<Existing> for New` and/or `From<New> for Existing`** conversions
4. **Document why** the new type is necessary in the commit message

## Anti-Patterns to Flag

- `fn fetch_positions() -> Vec<SomeNewPositionType>` without `From<SomeNewPositionType> for PositionSnapshot>`
- Adding `position_type`, `strategy`, `apr_pct` fields to a **new** struct when they should be added to `RuntimePositionDto` instead
- Creating a second `AccountInfo`-like type in `api` when `common::Metrics` could be extended
- `IbPositionDto` → `PositionSnapshot` conversion missing (tracked in T-1774099649464475000)

## Enforcement

When adding new structs, grep for existing similar types first:

```bash
# Check for existing position types before adding new ones
rg "pub struct.*Position" agents/backend/crates --type rust

# Check for existing account/metrics types
rg "pub struct.*Account|pub struct.*Metrics" agents/backend/crates --type rust
```
