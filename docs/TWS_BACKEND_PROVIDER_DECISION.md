# TWS Backend Provider Decision

**Date**: 2026-03-24
**Decision**: `ib_adapter` is the only supported TWS backend path
**Status**: Active guidance

## Decision

Use `agents/backend/crates/ib_adapter` as the single supported TWS backend
provider for Aether.

## Why `ib_adapter`

### 1. It is the provider the running backend actually instantiates

`backend_service` currently creates `IbAdapter` directly in
`agents/backend/services/backend_service/src/main.rs`.

### 2. The active config surface is built around the `tws`/`ibapi` path

`agents/backend/config/default.toml` exposes `"tws"` as the broker-backed market
data mode.

### 3. The service dependency graph already assumes `ib_adapter`

`backend_service` depends on `ib_adapter` directly, which makes `ib_adapter`
the maintained execution path today.

### 4. The current repo problem is integration coherence

Aether needs one provider that matches the actual runtime, docs, and operator
workflows. Keeping one supported path reduces ambiguity and maintenance cost.

## Comparison Summary

| Criterion | `ib_adapter` |
|----------|---------------|
| Active backend startup path | Yes |
| Present in `backend_service` deps | Yes |
| Active config path | Yes |
| Generic single-order flow | Yes |
| Box spread BAG support | Yes |
| Runtime comments/docs aligned today | Yes |

## What This Does Not Mean

This decision means:

- `ib_adapter` is the supported path for current backend, TUI, and CLI work
- deprecated alternative adapters should not remain in active docs or workspace

## Consequences

### Active guidance

- new backend/TUI/CLI work should target `ib_adapter`
- active docs should describe only the supported runtime path
- any future replacement should be a clean explicit cut unless a concrete
  compatibility requirement proves otherwise

## Archive Docs Worth Promoting Later

These archived docs contain material that could be moved to active docs and
rustified, but should be curated rather than copied:

- `docs/archive/platform/TWS_RECONNECT_BACKOFF.md`
  Good candidate for an active runbook because the reconnect pattern is still
  relevant to Rust services.
- `docs/archive/platform/TWS_YIELD_CURVE_KV_WRITER.md`
  Good candidate for an active operations doc because the daemon/backend/TUI
  yield-curve flow is current.
- `docs/archive/TWS_CONNECTION_TEST.md`
  Good candidate for a Rust-first connectivity runbook if rewritten around
  `cargo run -p backend_service`, `cargo run -p tui_service`, and the current
  broker config.

These should stay archived for now because they are still too research-heavy or
no longer match the active runtime direction:

- `docs/archive/RUST_IBAPI_SPIKE.md`
- `docs/archive/RESEARCH_RUST_TRADING_FRAMEWORKS.md`
