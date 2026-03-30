# Serde and chrono usage — domain & quant models (audit)

**Scope:** `broker_engine`, `quant`, `risk`, `strategy`, and shared `common` snapshot types.  
**Date:** 2026-03-30 (Todo2 T-1774379493374560000).

## Summary

| Crate | Serde | Chrono | Notes |
|-------|--------|--------|--------|
| **quant** | Optional feature `serde`; all public derives gated with `#[cfg_attr(feature = "serde", ...)]` | **Not used** — dates use **`time`** (`Date`, etc.) | `api` enables `quant/serde` for JSON surfaces. |
| **risk** | Same optional `serde` pattern as quant | **Not used** — uses **`time`** | `api` enables `risk/serde`; `backend_service` depends on `risk` **without** `serde` (smaller graph where types are not serialized). |
| **broker_engine** | **Always** depends on workspace `serde`; only **`OptionContract`** derives `Serialize`/`Deserialize` | None in `domain.rs` | Narrow intentional wire/config surface; `Position`, `AccountInfo`, `MarketData`, events have **no** serde. |
| **common** | `snapshot` and related wire types use serde | `DateTime<Utc>` on snapshot fields | Canonical place for timestamped API/snapshot types. |
| **strategy** | No derives on `StrategySignal` / `Decision` | `StrategySignal.timestamp` is `chrono::DateTime<Utc>` | In-process model today; no extra serialization dependency on strategy types. |

## Dependency layering (intent)

- **Calculation crates** (`quant`, `risk`): keep **optional** serde so library users and `backend_service` can avoid JSON derives when not needed.
- **API boundary** (`api`): turns on `serde` for types that appear in REST/JSON responses.
- **Broker domain** (`broker_engine`): serde is **required** at crate level because `OptionContract` is the single stable serialized shape for adapters; making it optional would ripple through `ib_adapter`, legacy execution crates, etc. **Deferred** unless a second “no-serde” consumer needs `broker_engine` alone.

## Follow-ups (not done here)

- If `StrategySignal` must cross the API, add `Serialize`/`Deserialize` behind a `strategy` crate feature (and align timestamp format with `common` snapshot conventions).
- Consider optional `serde` on `broker_engine` only if a concrete consumer needs `broker_engine` without JSON (measure compile time / dependency wins first).

## Verification

```bash
cd agents/backend && cargo check -p quant && cargo check -p quant --no-default-features
cd agents/backend && cargo check -p risk && cargo check -p broker_engine && cargo check -p strategy
```

(`quant` has no default features besides implied; `cargo check -p quant` verifies default path; `--features serde` is exercised transitively via `api`.)
