# Rust IBAPI Spike

**Purpose**: Minimal proof of concept for talking to TWS/IB Gateway directly from the Rust backend workspace using the community `ibapi` crate.

## Scope

This spike is intentionally narrow:

- connect to TWS/IB Gateway
- request server time
- subscribe to positions
- exit after the initial `PositionEnd` marker

It is **not** wired into the production backend service path. The goal is to measure integration friction and prove that the Rust workspace can establish a real IB transport path without going through the existing Client Portal HTTP flow.

## Binary

The spike binary lives at:

- `agents/backend/services/backend_service/src/bin/ib_probe.rs`

Run it from the Rust workspace root:

```bash
cd agents/backend
cargo run -p backend_service --bin ib_probe
```

## Environment

Supported environment variables:

- `IB_GATEWAY_ADDR` or `IBAPI_ADDR`
  Default: `127.0.0.1:7497`
- `IB_CLIENT_ID`
  Default: `7101`
- `IB_PROBE_TIMEOUT_SECS`
  Default: `15`

Examples:

```bash
cd agents/backend
IB_GATEWAY_ADDR=127.0.0.1:4002 IB_CLIENT_ID=42 cargo run -p backend_service --bin ib_probe
```

## Expected Output

On success the probe logs:

- connection metadata (`server_version`, `next_order_id`, connection time)
- one `server_time` result
- zero or more `Position` updates
- a final `PositionEnd` marker for the initial snapshot

## Why This Spike Matters

The active Rust backend currently gets IB positions through the Client Portal HTTP path in `api::ib_positions`. This spike checks whether a direct TWS/Gateway transport path is viable enough to justify future Rust-owned IB transport work.

## Interpretation

Good spike outcome:

- connects reliably to local Gateway/TWS
- server time request succeeds
- positions stream reaches `PositionEnd` without transport instability

Bad spike outcome:

- handshake or runtime errors are frequent
- account/session requirements are materially harder than the current Client Portal flow
- position stream behavior is too awkward to fit the current Rust service model

## Current Decision

Do **not** adopt direct Rust-owned TWS/Gateway transport as the primary backend path yet.

For now:

- keep the existing Rust production path on the Client Portal HTTP integration in `api::ib_positions`
- keep `ib_probe` as an isolated technical spike and operator verification tool
- re-open the architectural decision only after live verification against a real Gateway/TWS session

## Why

Current evidence supports a limited, conservative decision:

- the `ibapi` integration is compile-verified inside the Rust workspace
- the probe shows the crate surface is usable for connection, `server_time`, and `positions`
- but there is still no live-run evidence in this repository that the direct transport is operationally better than the current Client Portal path

The current Client Portal path already has several advantages for the active Rust backend:

- it is already integrated into the API surface
- it matches the current REST-centric runtime model
- it avoids immediately owning long-lived socket/session behavior inside the Rust backend
- it already provides a working positions path for the current product surface

Direct TWS/Gateway transport may still become worthwhile later, but only if live verification shows a clear benefit such as:

- materially better data access or latency
- access to flows that Client Portal cannot support cleanly
- acceptable operational reliability for local Gateway/TWS sessions
- a clear fit for the Rust service lifecycle and health model

## Adopt-Later Criteria

Move beyond the spike only if the follow-up live verification shows that:

1. `ib_probe` connects reliably across normal local development runs.
2. `server_time()` and `positions()` reach a stable `PositionEnd` without frequent session/operator friction.
3. The extra transport ownership cost is justified by capability or reliability gains over Client Portal.
4. We are ready to introduce a real transport abstraction instead of embedding `ibapi` calls directly into the current API handlers.

Until then, the correct posture is:

- **Client Portal remains the primary Rust-owned IB path**
- **`ibapi` remains an exploratory spike**
