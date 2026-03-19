# TWS Yield Curve Daemon

Standalone process that fetches box spread yield curve from TWS (option chain + option quotes) and writes to NATS KV key `yield_curve.{symbol}`. Backend and TUI read from KV as usual—no need to rebuild the main backend when iterating on TWS logic.

## Run

Requires **NATS** (with JetStream) and **TWS or IB Gateway** running.

```bash
# From repo root
just run-tws-yield-daemon

# Or from agents/backend
cargo run -p tws_yield_curve_daemon
```

## Env (optional)

| Env | Default | Description |
|-----|---------|-------------|
| `NATS_URL` | `nats://localhost:4222` | NATS server URL |
| `NATS_KV_BUCKET` | `LIVE_STATE` | KV bucket name |
| `SYMBOLS` | `SPX` | Comma-separated symbols (e.g. `SPX,SPXW`) |
| `INTERVAL_SECS` | `60` | Seconds between TWS fetch + KV write |
| `TWS_HOST` | `127.0.0.1` | TWS/IB Gateway host |
| `TWS_PORT` | (try 7497, 4002, 7496, 4001) | Set to e.g. `7496` to use a single port |
| `TWS_CLIENT_ID` | `0` | Base client ID (daemon uses +2) |
| `YIELD_CURVE_REFERENCE_SPOT_SPX` | `6000` | Reference spot for strike selection |

## Build only the daemon

```bash
cd agents/backend && cargo build -p tws_yield_curve_daemon
```

Faster than building the full workspace when you only change the daemon or `tws_yield_curve` crate.
