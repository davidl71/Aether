# Alpaca Source Architecture

**Date**: 2026-03-25  
**Status**: Active guidance for the Rust-first backend  
**Scope**: Alpaca as a market-data source only

## 1. Boundary

Alpaca belongs in the source layer, not in the execution engine layer.
For Aether's current read-only architecture, Alpaca can supply:

- market data
- account/credential identity
- paper/live endpoint selection

It should not be treated as:

- a yield-curve engine
- a synthetic-financing engine
- a generic fallback for unrelated data providers

If Alpaca execution is ever reintroduced, that should be a separate adapter
boundary with explicit broker-engine ownership. The active codebase should not
blur source resolution with order placement.

## 2. Current Rust Shape

The current `api::credentials` boundary already separates:

- paper vs live credentials
- trading base URL vs data base URL
- env overrides vs stored credentials

That matches the data-source-only model:

- paper credentials are the default compatibility path
- live credentials are explicit and separate
- data endpoints are distinct from trading endpoints

## 3. How Alpaca Fits Today

If Alpaca market data is enabled later, it should plug into the shared
`market_data` source model and emit the same normalized quote shapes as other
providers.

That keeps the operator view consistent:

- source name
- source priority
- age / freshness
- stale / degraded state

The yield-curve pipeline should stay separate. Box-spread curves should still
resolve from the active yield sources or synthetic placeholders when live data
is unavailable.

## 4. Future Engine Path

I did not find a strong native Rust Alpaca execution engine in the archived
research. The practical future path is:

1. Keep Alpaca as a thin Rust adapter over official REST and WebSocket APIs.
2. Wire that adapter through `broker_engine` if execution is ever re-enabled.
3. Use simulation/paper-trading support separately from live broker transport.

The promising Rust/native libraries in the repo for later work are:

- `financial_symbols` for OSI symbol parsing and normalization
- `rust_decimal` for currency-safe math
- `yatws` as an adapter-pattern reference for broker connectivity
- `rust-trade` for paper-trading and cache/pipeline patterns
- `matchcore` for deterministic local simulation if a matching engine is needed
- `nautilus-model` / NautilusTrader only as a model/reference layer, not as a
  default dependency

## 5. Active Files

- [agents/backend/crates/credential_store/src/lib.rs](../agents/backend/crates/credential_store/src/lib.rs) (stable import: `api::credentials`)
- [agents/backend/crates/api/src/finance_rates.rs](../agents/backend/crates/api/src/finance_rates.rs)
- [agents/backend/services/backend_service/src/yield_curve_writer.rs](../agents/backend/services/backend_service/src/yield_curve_writer.rs)
- [docs/MARKET_DATA_INTEGRATION.md](./MARKET_DATA_INTEGRATION.md)

## 6. Practical Rule

Use Alpaca as a source of truth for market data and identity, not as a
placeholder for broker execution logic or yield-curve fallback behavior. In
the current read-only product direction, Alpaca should remain source-only.
