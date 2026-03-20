---
description: Snapshot/domain types shared across crates must live in common crate; no duplicate struct definitions
alwaysApply: false
---

# Shared Types Pattern

**Rule:** Types used by **multiple backend crates** (`api`, `nats_adapter`, `broker_engine`, `market_data`, `strategy`) must live in `crates/common/` — not duplicated. Each crate imports from common. No independent struct copies across crates.

## Rationale

`nats_adapter` depends on `api` (`api/Cargo.toml:25`), so `nats_adapter` cannot depend on `api` back — creating pressure to copy types. The solution: extract shared types into `common`, which both can depend on.

## Layer Architecture

```
crates/common/          ← shared types only (no business logic)
├── src/
│   └── lib.rs
│   └── snapshot.rs     ← all cross-crate snapshot/event types
│   └── mod.rs

crates/api/            ← depends on common
├── src/state.rs       ← re-exports from common
│                       ← SystemSnapshot, SymbolSnapshot (api-only)

crates/nats_adapter/   ← depends on common (not api)
├── src/conversions.rs ← From<common::T> for pb::T
│                       ← imports from common, NOT local copies

crates/broker_engine/ ← domain types (broker-agnostic)
crates/market_data/    ← market data primitives
crates/strategy/       ← signal/decision types
```

## Shared Types (in `crates/common/`)

| Type | Fields | Used By |
|------|--------|---------|
| `PositionSnapshot` | id, symbol, quantity, cost_basis, mark, unrealized_pnl | api, nats_adapter |
| `HistoricPosition` | id, symbol, quantity, realized_pnl, closed_at | api, nats_adapter |
| `OrderSnapshot` | id, symbol, side, quantity, status, submitted_at | api, nats_adapter |
| `CandleSnapshot` | open, high, low, close, volume, entry, updated | api |
| `RiskStatus` | allowed, reason, updated_at | api, nats_adapter |
| `Metrics` | net_liq, buying_power, excess_liquidity, margin_requirement, commissions, *_ok flags | api, nats_adapter |
| `Alert` | level, message, timestamp | api, nats_adapter |
| `StrategyDecisionSnapshot` | symbol, quantity, side, mark, created_at | api, nats_adapter |

## Api-Only Types (NOT in common)

- `SystemSnapshot` — aggregates all above + api-specific fields
- `SymbolSnapshot` — runtime market state (bid, ask, spread, roi, candle, etc.)

## Proto Types

- `prost`-generated types in `nats_adapter::proto::v1` — always via `build.rs` from `proto/messages.proto`
- `From<pb::X> for common::X` and `From<common::X> for pb::X` go in `nats_adapter/src/conversions.rs`

## Enforcement

### When Adding a New Cross-Crate Type

1. **Check if it belongs in `common`** — if used by 2+ crates, it goes in `common`
2. **Do NOT copy to multiple crates** — one definition, multiple imports
3. **Add `common` to `Cargo.toml`** if not already:
   ```toml
   common = { path = "../common" }
   ```

### When Duplication is Found

Flag and track for refactor via T-1773992521958000 (move types to common). Until then:
- Prefer `common` types at new call sites
- Do NOT add new duplicate definitions

### Anti-Patterns to Flag

- `nats_adapter/src/conversions.rs` defining structs that mirror `api::state::*` → T-1773992551800128000
- `api/src/state.rs` defining structs already in `nats_adapter` → T-1773992581023776000
- New event/snapshot type added to only one crate but should be shared → add to common

## Related Tasks

- T-1773992521958807000 — Move snapshot types to common (foundational)
- T-1773992551800128000 — nats_adapter imports from common
- T-1773992581023776000 — api::state re-exports from common
