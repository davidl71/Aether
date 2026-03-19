# ib_adapter

Async Rust wrapper for Interactive Brokers TWS/Gateway. Uses the [ibapi](https://docs.rs/ibapi) crate for socket communication.

## Order placement readiness gate

**Before placing any order** (e.g. `place_order`, `place_bag_order` or equivalent), the session must be **ready**:

1. **Connected** — TWS/Gateway connection established (`ConnectionState::Connected`).
2. **nextValidId received** — The API has sent the next valid order ID. The underlying `ibapi::Client` exposes this (e.g. after `next_order_id()` is valid); do not submit orders until the client has received `nextValidId` from the server.

**Rationale:** TWS requires that order IDs are only used after the server has confirmed the session with `nextValidId`. Submitting before that can cause rejections or duplicate IDs.

**Actionable:** In any path that calls order placement (strategy, TUI, CLI), check that the adapter is connected and that a valid next order ID is available before submitting. See [ACTIONABLE_ITEMS_PATTERNS §1 — Gate actions on readiness](../../../../docs/platform/ACTIONABLE_ITEMS_PATTERNS.md#1-gate-actions-on-readiness) and [§2 — Order ID lifecycle](../../../../docs/platform/ACTIONABLE_ITEMS_PATTERNS.md#2-order-id-lifecycle-never-reuse).

## Event channels (reserved for future wiring)

`IbAdapter` holds three MPSC senders for streaming events; **none are currently wired** to producers or consumers:

| Channel        | Type                    | Intent |
|----------------|-------------------------|--------|
| `market_data_tx` | `Sender<MarketDataEvent>`  | Forward TWS tick updates (bid/ask/last/volume) to a consumer task. |
| `position_tx`    | `Sender<PositionEvent>`    | Stream position updates as they arrive (e.g. after `request_positions` or position subscription). |
| `order_tx`       | `Sender<OrderStatusEvent>` | Stream order status updates (filled, cancelled, etc.). |

**Current state:** Channels are created in `IbAdapter::new` and the receivers are dropped, so nothing is sent. `request_market_data` does not yet forward ticks to `market_data_tx`; `request_positions` returns a snapshot and does not stream to `position_tx`; order placement does not push to `order_tx`.

**Future wiring:** Callers can clone the senders via `market_data_tx()`, `position_tx()`, and `order_tx()` and pass the receiver to a backend or TUI task. When wiring: (1) start a task that `recv()`s on the receiver and applies events to shared state or NATS; (2) from TWS callbacks or subscription streams, call `send()` on the cloned sender. See `backend_service::tws_market_data` and `tws_positions` for current TWS integration that uses `ibapi::Client` directly (not `IbAdapter`); a future refactor could route through `IbAdapter` and these channels.

## Scanner (stub)

The `scanner` module provides a **stub** for TWS scanner-based discovery (e.g. hot by volume, high option volume P/C ratio). Flow: `reqScannerParameters` → `reqScannerSubscription` → `scannerData` / `scannerDataEnd` → `cancelScannerSubscription`. TWS message IDs (protobuf, min server 210): `REQ_SCANNER_PARAMETERS`, `REQ_SCANNER_SUBSCRIPTION`, `CANCEL_SCANNER_SUBSCRIPTION`. See `src/scanner.rs` and [TWS_API_LEARNINGS_FROM_SIBLING_REPO.md §6](../../../../docs/platform/TWS_API_LEARNINGS_FROM_SIBLING_REPO.md#6-scanner-subscription). Full implementation optional.
