# TWS/IB connection retry and backoff

TWS market data and position fetchers use **exponential backoff with cap** on connection failure instead of a fixed sleep. Same schedule is used across backend_service and tws_yield_curve_daemon.

## Schedule

- **Initial delay:** 2s  
- **Multiplier:** 2x  
- **Cap:** 60s  
- **Logging:** One log line per attempt (e.g. "reconnecting…" with `delay_secs` and optional `attempt`).

Example sequence: 2s → 4s → 8s → 16s → 32s → 60s → 60s …

## Where it is implemented

| Location | Behavior |
|----------|----------|
| `backend_service::tws_market_data` | Connect loop: try all ports; on failure log "TWS connection failed (tried all ports), reconnecting…", apply backoff, sleep, retry. On success reset backoff. |
| `backend_service::tws_positions` | Interval loop (60s): on `fetch_and_merge_positions` error log "TWS positions: reconnecting…", apply backoff, sleep, then next tick. On success reset backoff. |
| `tws_yield_curve_daemon` | After each write_cycle: if any symbol had a TWS fetch error, log "TWS yield curve: reconnecting…", apply backoff, sleep, then next cycle. On success reset backoff. |
| `ib_adapter` | No retry loop; callers are expected to implement retry with exponential backoff when calling `connect()`. |

## References

- Backoff crate: `backoff::ExponentialBackoffBuilder` (initial 2s, max 60s, multiplier 2.0).
- NATS reconnects in TUI use a similar pattern with circuit breaker: `tui_service::circuit_breaker` (2s → 60s, plus circuit open 30s pause).
