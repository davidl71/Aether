# Crate Boundary Logic

**Date**: 2026-03-24
**Status**: Active guidance for Rust workspace refactors

## Purpose

This document records the logic for crate boundaries in `agents/backend/`.
The goal is not "more crates". The goal is:

- keep broker-specific code out of product logic
- keep reusable domain/data types out of service binaries
- keep service crates thin and orchestration-focused
- split oversized crates and modules only where the seam is real

## Boundary Rules

### 1. Put stable shared language in `common`

`crates/common` should hold low-level shared types and helpers that are used by
multiple backend crates and do not imply ownership by one subsystem.

Good fits:

- snapshot/read-model structs shared across `api`, `nats_adapter`, `tui_service`
- basic parsing/helpers like expiry parsing
- durable message-compatible shapes that are intentionally reused

Bad fits:

- HTTP DTOs tied to one endpoint
- broker traits or broker transport types
- TUI-only view state
- business workflows owned by one subsystem

Rule of thumb: if removing a type from `common` would create duplication across
three or more crates, it likely belongs in `common`. If it mainly exists to
serve one crate's API surface, keep it there.

### 2. Put broker-agnostic read-only broker contracts in `broker_engine`

`crates/broker_engine` owns the broker abstraction boundary:

- `BrokerEngine` trait
- broker-neutral market-data, option-chain, position, account, connection, and event types used by the active read-only stack
- capability flags and request/result types required by multiple adapters

In the current workspace shape, execution-only order placement, BAG/combo
requests, and resolved execution-contract metadata do **not** live here. They
live in `broker_execution_legacy`, which is excluded from the default build.

It should not own:

- `ibapi` details
- REST-specific DTOs
- TUI formatting concerns
- high-level strategy or snapshot projection code

This crate is the seam between product logic and transport implementation. If a
type must survive adapter changes or future broker expansion, default to
`broker_engine` rather than an adapter crate.

### 3. Adapter crates own transport-specific code only

`crates/ib_adapter` should contain:

- connection lifecycle
- transport quirks
- contract translation needed for read-only market-data and account flows
- subscription plumbing
- adapter-specific helpers such as pacing, scanner stubs, or builder bridges

It should not become a second copy of domain logic. If an adapter concern turns
out to be broker-neutral, move the concept up to `broker_engine` or `common`.

Execution-only IBKR order placement and resolved-contract execution plumbing now
belong in `crates/ib_execution_legacy`, not the active `ib_adapter` crate.

### 4. `api` is not the dumping ground

`crates/api` should own:

- API-facing read models and endpoint-adjacent logic
- projection logic from shared state into client-facing DTOs
- calculations that are specifically exposed as API workflows

It should not quietly absorb:

- every runtime state type in the system
- broker transport concerns
- ledger internals
- TUI-only state
- generic market-data provider code

Current problem: `api` has become a mixed crate for DTOs, runtime state,
finance utilities, loans, config-ish behavior, and integration helpers. It is
the main candidate for future segmentation.

### 5. `market_data` owns provider integration and quote selection

`crates/market_data` should own:

- provider clients and provider-specific normalization
- quote aggregation and source-priority selection
- symbol discovery and market data ingestion primitives
- candle-building and synthetic instrument market-data transforms, if they are
  source/data concerns rather than UI concerns

It should not own:

- broker account/position logic
- TUI chart rendering concerns
- broad API DTO projections

### 6. Service crates orchestrate

`services/backend_service`, `services/tui_service`, and
`services/tws_yield_curve_daemon` should compose crates. They should not be the
long-term home for large domain registries or sprawling state models.

Good fits:

- startup wiring
- task orchestration
- config loading
- external interface binding

Bad fits:

- giant business-logic modules
- reusable DTOs used elsewhere
- cross-service domain types

## Current Roles

| Crate | Intended boundary |
|------|-------------------|
| `common` | Shared snapshots, events, helpers, small common utilities |
| `broker_engine` | Active read-only broker trait and broker-neutral market/account/position domain types |
| `broker_execution_legacy` | Opt-in execution-only broker traits and BAG/order domain types |
| `ib_adapter` | Active read-only IBKR/TWS socket implementation via `ibapi` |
| `ib_execution_legacy` | Opt-in IBKR order-placement and cancellation implementation |
| `market_data` | Provider clients, quote aggregation, source prioritization |
| `quant` | Financial calculations and pricing math |
| `risk` | Risk checks built on shared and quant primitives |
| `ledger` | Durable accounting and transaction persistence |
| `discount_bank_parser` | Bank import/parsing boundary |
| `api` | Client-facing read models, projections, and API workflows |
| `nats_adapter` | Proto conversion and messaging integration |
| service crates | Composition and process entry points |

## What Does Not Belong Where

### Do not put these in `common`

- endpoint request/response structs that only serve one route
- adapter-specific request builders
- TUI tab or popup state
- one-off admin or migration helpers

### Do not put these in `broker_engine`

- concrete `ibapi` handles
- HTTP clients
- NATS publishing details
- large projection/read-model DTOs for one UI

### Do not put these in service crates

- reusable shared state DTOs
- generic market-data aggregation logic
- broker-neutral domain types

## Segmentation Opportunities

These are real seams, not arbitrary package churn.

### `api` crate

High-value split candidates:

- `runtime_state.rs` into `runtime_state/{positions,orders,decisions,market,metrics}.rs`
- `loans.rs` into narrower loan-domain and loan-DTO modules
- move any cross-crate reusable runtime structs down to `common` if they are no
  longer API-specific

Why:

- `api` currently has high dependency fan-in and too many ownership concerns
- large files make it the most likely source of accidental coupling

### `market_data` crate

High-value split candidates:

- provider modules by vendor: `fmp`, `yahoo`, `polygon`, `tase`
- aggregator submodule separated from provider clients
- synthetic/candle generation isolated from raw provider fetchers

Why:

- this crate is doing both ingestion and normalization and is growing toward a
  platform boundary of its own

### `broker_engine` crate

Module split candidate inside the crate:

- `domain.rs` into `contracts.rs`, `positions.rs`, `account.rs`,
  `events.rs`, `config.rs`

Why:

- the crate boundary is correct
- the module boundary inside the crate is too flat
- the active crate should stay visibly read-only after the execution split

### `quant` crate

Module split candidate:

- move exported result/domain structs into a `types.rs` or narrower submodules
- keep `lib.rs` as a barrel and high-level entry surface

Why:

- the crate boundary is correct
- the current file shape hides the distinction between math code and data types

### `tui_service`

Module split candidates:

- `app.rs` into app shell, tab state, overlays/forms, and command handling
- `input.rs` into keymap, mode transitions, and action dispatch

Why:

- TUI complexity is real, but it should remain inside the service crate because
  most of this state is UI-local rather than cross-crate domain state

## Include Graph Heuristics

When deciding whether to create or move a crate/module, prefer these tests:

1. Dependency direction
   A lower-level crate must not depend on a higher-level workflow crate.

2. Reuse count
   If two or more active crates need the same type or helper, duplication is a
   stronger smell than another module.

3. Runtime ownership
   If the type exists only because one binary renders or serves it, keep it
   close to that binary unless another consumer appears.

4. Transport neutrality
   If a concept must survive adapter replacement, it does not belong in an
   adapter crate.

5. Compile-cost leverage
   Split crates when the seam reduces churn and rebuild scope, not just to make
   the dependency graph prettier.

## Builder Guidance

Per repository guidance, new or refactored event/data structs with 5+ fields
should use `derive_builder`. This matters most at crate boundaries, where added
fields otherwise create cascading constructor breakage.

Priority targets for that pattern are shared boundary structs in:

- `common`
- `broker_engine`
- cross-crate DTOs that are still evolving

## Immediate Recommendations

1. Keep the current top-level crate map. The biggest problem is oversized
   modules, not missing crates.
2. Treat `api` as the first segmentation target.
3. Keep `broker_engine` as the broker-neutral seam and do not leak `ibapi`
   details upward.
4. Keep TUI state in `tui_service` unless another client actually needs it.
5. Move only stable shared types into `common`; do not turn it into a misc
   bucket.
