# broker_engine Architecture

**Date**: 2026-03-24
**Status**: Active
**Owner**: backend / ib_adapter team

## 1. Overview

`broker_engine` defines the broker-neutral boundary for Aether's active
read-only Rust backend. The current supported TWS implementation is
`ib_adapter`.

```text
backend_service / api handlers / services
                |
                v
          broker_engine
   (active read-only trait + types)
                |
                v
            ib_adapter
      (active ibapi-backed TWS)

legacy execution:
  broker_execution_legacy
                |
                v
        ib_execution_legacy
```

The purpose of this boundary is to keep broker transport details out of product
logic, not to maintain multiple partially integrated TWS adapters.

## 2. What broker_engine Owns

- `BrokerEngine` trait
- broker-neutral market-data, account, position, option-chain, connection, and event types
- broker-neutral error and connection-state types
- request/result types used across backend code

It should not own:

- `ibapi` handles
- execution-only order/BAG request types
- service wiring
- HTTP DTOs
- TUI state

## 3. What ib_adapter Owns

- TWS connection lifecycle
- contract translation
- market-data subscription plumbing
- IBKR-specific helpers such as pacing and scanner support

It should not own:

- API projection logic
- TUI rendering concerns
- generic snapshot/read-model types

Execution-only IBKR order placement, BAG/combo submission, cancellation, and
resolved execution-contract lookup now live in `ib_execution_legacy`, which is
excluded from the default workspace build.

## 4. Why This Boundary Exists

Historically, backend code reached directly into broker transport concerns.
`broker_engine` fixes that by giving the rest of the system one stable seam.

Current result:

- backend code depends on `BrokerEngine`, not direct `ibapi` calls
- `ib_adapter` is the only supported TWS backend path
- execution-era broker APIs are isolated from the default build
- tests and future mock engines can target the trait instead of the transport

## 5. Current Workspace Shape

```text
agents/backend/
  crates/
    broker_engine/
    broker_execution_legacy/
    ib_adapter/
    ib_execution_legacy/
    common/
    market_data/
    risk/
    quant/
    ledger/
    nats_adapter/
    api/
    tws_yield_curve/
  services/
    backend_service/
    tui_service/
    tws_yield_curve_daemon/
```

`backend_service` creates `IbAdapter` and passes it around as `Arc<dyn
BrokerEngine>`.

## 6. Design Notes

### Zero broker transport leakage

`broker_engine` should remain free of `ibapi` details. If a type or helper is
transport-specific, it belongs in `ib_adapter`.

### Event types belong in the domain boundary

`MarketDataEvent`, `PositionEvent`, and `OrderStatusEvent` are outputs of broker
work. They belong in `broker_engine` so downstream code can depend on one shape.

### Execution types are opt-in, not default

Order placement methods, BAG/combo requests, and fully resolved execution
contracts are intentionally not part of the active `BrokerEngine` boundary
anymore. They remain available only through the legacy crates so default builds
and routine read-only work do not pull execution paths back into the active
surface.

### Trait object over generics

The runtime constructs one engine instance and shares it across long-lived
tasks. `Arc<dyn BrokerEngine>` keeps signatures cleaner than propagating type
parameters through the service layer.

## 7. Current Gaps

- The market-data path still needs a clean consumer bridge from `IbAdapter`
  events into the shared aggregator and NATS publication flow.
- `broker_engine` is the correct crate boundary, but some large modules around
  it still need segmentation, especially in `api`.
- Remaining legacy docs and comments still need cleanup so they stop presenting
  `ib_adapter` as an active order-placement surface.

## 8. Related Docs

- [CRATE_BOUNDARIES.md](./CRATE_BOUNDARIES.md)
- [TWS_BACKEND_PROVIDER_DECISION.md](./TWS_BACKEND_PROVIDER_DECISION.md)
- [BACKEND_TYPE_COMPARISON.md](./BACKEND_TYPE_COMPARISON.md)
