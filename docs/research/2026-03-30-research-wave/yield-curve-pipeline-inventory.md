# Yield curve pipeline — inventory (2026-03-30)

End-to-end map of how box-spread yield curves flow into the TUI and CLI. **Provider “interface”** is implicit: each path produces `Vec<serde_json::Value>` opportunities → encoded to KV / `CurveResponse`, not a single Rust trait.

## Data plane (NATS JetStream KV)

| Artifact | Location |
|----------|----------|
| KV bucket | Default `LIVE_STATE` (`NATS_KV_BUCKET`) |
| Keys | `yield_curve.{symbol}` |
| Value | Protobuf `YieldCurve` bytes and/or JSON opportunities (writers annotate `data_source`) |

## Writers (who fills KV)

1. **`tws_yield_curve_daemon`** (`agents/backend/services/tws_yield_curve_daemon/`)  
   - Standalone process: TWS → `tws_yield_curve` → encode → `yield_curve.{symbol}`.  
   - Health: `tws_yield_curve_daemon` via `nats_adapter::spawn_health_publisher`.

2. **`yield_curve_writer`** (`agents/backend/services/backend_service/src/yield_curve_writer.rs`)  
   - Spawned from `backend_service` when `yield_curve.enabled` in config.  
   - Source selection (see module doc): `YIELD_CURVE_SOURCE=tws` → `tws_yield_curve::fetch_yield_curve_from_tws`; optional `YIELD_CURVE_SOURCE_URL`; else Yahoo option chains + **synthetic** placeholder curve (`DTE_DAYS`, `BASE_RATE`, etc.).  
   - Same key prefix `yield_curve.{symbol}`.

## Core library

- **`tws_yield_curve`** (`agents/backend/crates/tws_yield_curve/`) — IBKR/TWS option chain + quotes; **no NATS**. Used by daemon, `yield_curve_writer` (tws branch), `finance_rates` handlers, and CLI `run_yield_curve`.

## Readers / request plane

| Component | Role |
|-----------|------|
| **`finance_rates` handlers** (`backend_service/src/handlers/finance_rates/`) | Subscribes `api.finance_rates.yield_curve`, `api.yield_curve.refresh`. Serves curves: KV first (`load_yield_curve_from_kv`), else live `fetch_yield_curve_from_tws` when appropriate. |
| **Topics** (`nats_adapter::topics::api`) | `api.finance_rates.yield_curve`, `api.yield_curve.refresh` (control; triggers writer cycle). |
| **TUI** (`tui_service/src/nats.rs`) | KV watcher `yield_curve.*`; decode via `api::yield_curve_proto::curve_response_from_proto_bytes`. Refresh publishes `{}` to `api.yield_curve.refresh`. |
| **TUI state** (`app.rs`) | `yield_curves_all: HashMap<String, CurveResponse>`, `sync_yield_curve_from_cache()`, tab `ui/yield_curve.rs`. |
| **CLI** (`agents/backend/bin/cli`) | `yield-curve` command: synthetic/local/TWS, optional publish to `yield_curve.direct.{symbol}` or request/response via `api.yield_curve.refresh`. |

## Mock / deterministic provider (gap vs backlog)

There is **no** dedicated “MockYieldCurve” type in-tree today. Closest deterministic paths:

- **Synthetic** points inside `yield_curve_writer` (placeholder curve when Yahoo/TWS unavailable).  
- **CLI** `source=synthetic` / `local` for one-off curves.

Backlog task **“Yield curve: add Mock provider end-to-end”** should introduce an explicit mock (config-driven or test-only) that writes the same KV shape without TWS/Yahoo, for baseline tests.

## R sidecar (deferred, T-1774201865476785000)

- **`analytics/r/yield_curve/`** — optional **plumber** API (`POST /estimate`): sparse zero pillars → smoothed zero/forward grid (base R spline). Optional CRAN **`termstrc`** / **`YieldCurve`** in a follow-up. No NATS/KV write; Rust would call HTTP off the hot path if integrated later.

## File index (quick navigation)

- `crates/tws_yield_curve/src/lib.rs` — TWS fetch  
- `services/tws_yield_curve_daemon/src/main.rs` — daemon loop  
- `services/backend_service/src/yield_curve_writer.rs` — embedded writer + synthetic/Yahoo/TWS  
- `services/backend_service/src/handlers/finance_rates/mod.rs` — NATS RPC + refresh  
- `services/tui_service/src/nats.rs` — KV watch + refresh publish  
- `services/tui_service/src/app.rs` — cache + UI sync  
