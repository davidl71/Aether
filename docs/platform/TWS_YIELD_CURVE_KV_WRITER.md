# TWS yield curve and KV writer

How TWS yield-curve data reaches NATS KV and what may need fixing.

## Current data paths

| Path | Component | Data source | Writes KV? |
|------|-----------|-------------|------------|
| **CLI** | `cargo run -p cli -- yield-curve --symbol SPX --source tws` | `tws_yield_curve::fetch_yield_curve_from_tws()` | No (stdout only) |
| **Daemon** | `tws_yield_curve_daemon` | `tws_yield_curve::fetch_yield_curve_from_tws()` | Yes, key `yield_curve.{symbol}` |
| **Backend writer** | `backend_service` `yield_curve_writer` | `YIELD_CURVE_SOURCE_URL` (HTTP) or synthetic | Yes, same key |

## Findings that affect KV

1. **Delayed ticks and RequestParameters** — The `tws_yield_curve` crate now handles delayed tick types (66, 67, 80, 81, 82) and uses `RequestParameters.min_tick` when TWS sends tick 81 as RequestParameters. So any consumer that calls `fetch_yield_curve_from_tws()` (CLI, daemon, or future backend path) gets the same fixed behavior.

2. **tws_yield_curve_daemon** — Already uses `fetch_yield_curve_from_tws()`. No code change needed; run it with TWS up and same env as CLI (`YIELD_CURVE_USE_CLOSING=1`, `TWS_CLIENT_ID`, etc.) so it gets quotes and writes real TWS data to KV.

3. **Backend yield_curve_writer** — Does **not** call TWS. It only:
   - Fetches from `YIELD_CURVE_SOURCE_URL` if set (expects JSON array of curve points), or
   - Uses `synthetic_opportunities()` (fake curve).
   So with the current backend_service, KV is only filled from TWS if the **tws_yield_curve_daemon** is running, or if you point `YIELD_CURVE_SOURCE_URL` at something that serves TWS-derived JSON.

## What to fix for KV writer (optional)

If you want **backend_service** itself to write TWS yield curve to KV without running the separate daemon:

- **Option A (recommended):** Keep using **tws_yield_curve_daemon** for TWS → KV. Ensure it is started when TWS is available and env (e.g. `YIELD_CURVE_USE_CLOSING=1`) is set. No backend_service code change.

- **Option B:** Add a TWS source to **yield_curve_writer** in `backend_service`:
  - e.g. `YIELD_CURVE_SOURCE=tws` (or detect when no URL and TWS desired).
  - In `write_cycle()`, when source is TWS, call `tws_yield_curve::fetch_yield_curve_from_tws(symbol).await` for each symbol instead of `fetch_curve_from_url` / synthetic.
  - Same env vars as CLI/daemon (TWS_HOST, TWS_PORT, TWS_CLIENT_ID, YIELD_CURVE_USE_CLOSING, etc.).
  - Consider: backend may not have TWS connectivity; daemon is often run on a host that does. So Option A is simpler operationally.

## References

- TWS → Rust mapping and tick handling: `docs/platform/TWS_API_LOG_TO_YIELD_CURVE_MAP.md`
- Yield curve writer: `agents/backend/services/backend_service/src/yield_curve_writer.rs`
- TWS daemon: `agents/backend/services/tws_yield_curve_daemon/src/main.rs`
- Box spread / TWS: `docs/platform/BOX_SPREAD_YIELD_CURVE_TWS.md`
