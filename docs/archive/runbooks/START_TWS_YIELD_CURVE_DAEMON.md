# Runbook: Start TWS yield curve daemon when TWS is available

When TWS (or IB Gateway) is running and you want **yield curve data from TWS** to be written to NATS KV (so the backend and TUI can use it for `api.finance_rates.build_curve` and yield-curve views), run the **tws_yield_curve_daemon**.

## Prerequisites

- **NATS** with JetStream (KV bucket `LIVE_STATE` exists or will be created).
- **TWS or IB Gateway** running and accepting connections (e.g. port 7497 paper, 7496 live; or 4002/4001 for Gateway).
- Env (optional but recommended): `YIELD_CURVE_USE_CLOSING=1`, `TWS_CLIENT_ID`, and if needed `TWS_HOST` / `TWS_PORT`.

## Command

From repo root:

```bash
just run-tws-yield-daemon
```

Or from `agents/backend`:

```bash
cargo run -p tws_yield_curve_daemon
```

## Env (optional)

| Env | Default | Description |
|-----|---------|--------------|
| `NATS_URL` | `nats://localhost:4222` | NATS server URL |
| `NATS_KV_BUCKET` | `LIVE_STATE` | KV bucket name |
| `SYMBOLS` | `SPX` | Comma-separated symbols (e.g. `SPX,SPXW`) |
| `INTERVAL_SECS` | `60` | Seconds between TWS fetch and KV write |
| `TWS_HOST` | `127.0.0.1` | TWS/IB Gateway host |
| `TWS_PORT` | (auto-try 7497, 4002, 7496, 4001) | Set to e.g. `7496` to pin port |
| `TWS_CLIENT_ID` | `0` | Base client ID (daemon uses +2) |
| `YIELD_CURVE_USE_CLOSING` | — | Set to `1` to use closing prices for curve (recommended) |
| `YIELD_CURVE_REFERENCE_SPOT_SPX` | `6000` | Reference spot for strike selection |

## Verify

- **NATS KV:** After one interval, check key `yield_curve.{symbol}` (e.g. `yield_curve.SPX`) in bucket `LIVE_STATE`. See [LIVE_STATE_KV_VERIFICATION.md](../platform/LIVE_STATE_KV_VERIFICATION.md).
- **Backend/TUI:** Call `api.finance_rates.build_curve` with the same symbol; backend reads from KV. If the daemon is not running, build_curve may use synthetic or URL source instead.

## References

- Daemon README: `agents/backend/services/tws_yield_curve_daemon/README.md`
- TWS → KV data path: `docs/platform/TWS_YIELD_CURVE_KV_WRITER.md`
- LIVE_STATE KV key patterns: `docs/platform/NATS_API.md` § LIVE_STATE KV
