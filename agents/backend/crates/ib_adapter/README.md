# ib_adapter

Async Rust wrapper for Interactive Brokers TWS/Gateway. Uses the [ibapi](https://docs.rs/ibapi) crate for socket communication.

## Scope

`ib_adapter` is the active **read-only** IBKR/TWS adapter. It owns:

- connection lifecycle
- market-data subscriptions
- option-chain lookup for analytics
- account and position fetches

It does **not** own active order placement anymore. Execution-only order
placement, BAG/combo submission, cancellation, and resolved contract metadata
for order flows now live in `ib_execution_legacy`.

## Legacy order placement readiness gate

For explicit legacy execution workflows, the session must be **ready** before
placing any order:

1. **Connected** — TWS/Gateway connection established (`ConnectionState::Connected`).
2. **nextValidId received** — The API has sent the next valid order ID. The underlying `ibapi::Client` exposes this (e.g. after `next_order_id()` is valid); do not submit orders until the client has received `nextValidId` from the server.

**Rationale:** TWS requires that order IDs are only used after the server has confirmed the session with `nextValidId`. Submitting before that can cause rejections or duplicate IDs.

**Actionable:** Any future opt-in execution path should check that the adapter is
connected and that a valid next order ID is available before submitting. See
[ACTIONABLE_ITEMS_PATTERNS §1 — Gate actions on readiness](../../../../docs/platform/ACTIONABLE_ITEMS_PATTERNS.md#1-gate-actions-on-readiness) and
[§2 — Order ID lifecycle](../../../../docs/platform/ACTIONABLE_ITEMS_PATTERNS.md#2-order-id-lifecycle-never-reuse).

## Event channels

`IbAdapter` holds the active market-data sender/broadcast path for streaming
quote events. Historical references to `position_tx` and `order_tx` are now
legacy design notes rather than active exported channels.

| Channel | Type | Intent |
|---------|------|--------|
| `market_data_tx` | `Sender<MarketDataEvent>` | Forward TWS tick updates (bid/ask/last/volume) to a consumer task. |

**Current state:** `request_market_data` forwards tick-derived quote events to
`market_data_tx`. The remaining integration gap is on the consumer side: the
active backend startup path does not yet attach a receiver/bridge cleanly enough
to make broker events flow through the shared aggregator and NATS path.

The active product direction is to keep TWS integration on `IbAdapter` for
read-only flows and finish the service-side market-data wiring instead of
reintroducing another parallel path.

## Scanner (stub)

The `scanner` module provides a **stub** for TWS scanner-based discovery (e.g. hot by volume, high option volume P/C ratio). Flow: `reqScannerParameters` → `reqScannerSubscription` → `scannerData` / `scannerDataEnd` → `cancelScannerSubscription`. TWS message IDs (protobuf, min server 210): `REQ_SCANNER_PARAMETERS`, `REQ_SCANNER_SUBSCRIPTION`, `CANCEL_SCANNER_SUBSCRIPTION`. See `src/scanner.rs` and [TWS_API_LEARNINGS_FROM_SIBLING_REPO.md §6](../../../../docs/platform/TWS_API_LEARNINGS_FROM_SIBLING_REPO.md#6-scanner-subscription). Full implementation optional.
