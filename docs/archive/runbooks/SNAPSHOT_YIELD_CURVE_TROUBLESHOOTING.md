# Snapshot and yield curve troubleshooting

When `snapshot-write` or `yield-curve` return empty or "no responders", use this runbook.

---

## Quick test

From repo root or `agents/backend`:

```bash
cd agents/backend
cargo run -p cli -- snapshot-write
cargo run -p cli -- yield-curve --symbol SPX
```

Or use the script:

```bash
./scripts/check_snapshot_yield_curve.sh
```

---

## 1. Snapshot-write: "no responders"

**Symptom:** `Error: publish_now: ... no responders: no responders`

**Cause:** No process is subscribed to `api.snapshot.publish_now`. The backend_service must be running and must have been built with the handler (added in the same change that added the CLI command).

**Fix:**

1. **Rebuild and restart backend**
   ```bash
   cd agents/backend
   cargo build -p backend_service
   # Restart backend_service (e.g. stop the running one and start again)
   ```
2. Ensure **NATS** is up on `NATS_URL` (default `nats://localhost:4222`).
3. Start **backend_service** after NATS so it can connect and register the `api.snapshot.publish_now` handler.

**Verify:** After restart, run `cargo run -p cli -- snapshot-write` again. You should see e.g. `Snapshot written at <time> to snapshot.ib`.

---

## 2. Yield-curve: "No yield curve points"

**Symptom:** `No yield curve points for SPX (backend may have no data in KV yield_curve.SPX)`

**Cause:** The backend’s `api.finance_rates.build_curve` handler fills the curve from NATS KV key `yield_curve.{symbol}` (bucket `LIVE_STATE`). If that key is missing or empty, the curve is empty.

**Requirements:**

- **NATS with JetStream** – KV bucket `LIVE_STATE` must exist. Start NATS with JetStream (e.g. `nats-server -js` or config with `jetstream { store_dir: "data/jetstream" }`). See `config/nats-server.conf`.
- **Backend yield curve writer** – Backend spawns a task that writes `yield_curve.{symbol}` for each `market_data.symbols` entry. If `market_data.symbols` is empty, the writer is not spawned.
- **Backend config** – In `agents/backend/config/default.toml` (or your config), ensure `[market_data]` has `symbols = ["SPX"]` (or the symbol you request).

**Fix:**

1. **Enable JetStream** – Use a NATS config that enables JetStream (e.g. project `config/nats-server.conf`). Restart NATS if you changed config.
2. **Ensure symbols in config** – In backend config, set e.g. `symbols = ["SPX"]` under `[market_data]`.
3. **Restart backend** – So it spawns the yield curve writer and connects to NATS. Wait a few seconds for the writer to run (it writes once on spawn, then on an interval).
4. **Optional:** Set `YIELD_CURVE_SOURCE_URL` to a URL that returns a JSON array of curve points (see `docs/platform/BOXTRADES_REFERENCE.md`). Otherwise the writer uses synthetic points.

**Verify:** `cargo run -p cli -- yield-curve --symbol SPX` should show a table of points. In the backend log you should see e.g. `yield curve writer started` and `yield curve writer: wrote` for key `yield_curve.SPX`.

**Refresh KV (e.g. to get latest synthetic strikes):** With backend and NATS running, run `cargo run -p cli -- yield-curve --symbol SPX --refresh`. The `--refresh` flag triggers the backend to run one yield-curve write cycle (subject `api.yield_curve.refresh`) before fetching, so the table includes `strike_low`/`strike_high` (e.g. 5995/6000) when using synthetic data.

**Bypass KV for testing (algorithm verification):** To force the backend to use the same in-process synthetic curve as the writer (no NATS KV or writer required), set `YIELD_CURVE_BYPASS_KV=1` when starting the backend, then run the CLI as usual. The curve is built from `yield_curve_writer::synthetic_opportunities(symbol)` and **Source** shows `synthetic`. Use this to verify strikes, APR, and Conv % without depending on KV or the writer task.

```bash
# Terminal 1: start backend with bypass
cd agents/backend
YIELD_CURVE_BYPASS_KV=1 cargo run -p backend_service --bin backend_service

# Terminal 2: fetch curve (no --refresh needed; bypass ignores KV)
cargo run -p cli -- yield-curve --symbol SPX
```

**Get data from TWS (direct, no backend/NATS):** Use the CLI with `--source tws`. The CLI uses the `tws_yield_curve` crate directly (TWS option chain + quotes → build_curve). No NATS or backend required. Optional: `--publish-to-nats` publishes the result to subject `yield_curve.direct.{symbol}`.

```bash
cd agents/backend
# TWS/IB Gateway running (e.g. port 7496). Optional: TWS_PORT=7496
cargo run -p cli -- yield-curve --symbol SPX --source tws
```

**TWS as a separate daemon (recommended; no backend rebuild):** Run the standalone `tws_yield_curve_daemon`. It fetches from TWS and writes to NATS KV `yield_curve.{symbol}` on an interval. Backend and CLI (default `--source nats`) read from KV; you only rebuild/restart the daemon when changing TWS logic.

```bash
# NATS and TWS/IB Gateway running. Then:
just run-tws-yield-daemon
# Or: cd agents/backend && cargo run -p tws_yield_curve_daemon
# Env: TWS_PORT=7496, SYMBOLS=SPX, INTERVAL_SECS=60 (optional)
```

**Get data from TWS via backend (NATS):** Start backend with `YIELD_CURVE_USE_TWS=1`; it uses the same `tws_yield_curve` crate when handling build_curve requests. Then run `yield-curve --symbol SPX` (default `--source nats`). Optional: `YIELD_CURVE_REFERENCE_SPOT_SPX`, `TWS_PORT`. See `docs/platform/BOX_SPREAD_YIELD_CURVE_TWS.md` for architecture.

**Example table from CLI (successful run):**

```bash
cargo run -p cli -- yield-curve --symbol SPX --refresh
```

```
Box spread yield curve: SPX (strike width 4 pts)
Underlying price at report time: 6000.00
Symbol   Expiry         DTE Bucket         Width     Strikes  Long % Short %   Spread   APR %  Conv %
---------------------------------------------------------------------------------------------------
SPX      2026-04-17      30 2 months           4   5998/6002    4.40    5.20      +80    4.80    0.50
SPX      2026-05-17      60 3 months           4   5998/6002    4.40    5.20      +80    4.80    0.50
SPX      2026-06-16      90 4 months           4   5998/6002    4.40    5.20      +80    4.80    0.50
...
```

- *Width*: put/call spread (strike width in points). Default synthetic uses width 4 (±2 around the money).
- *Strikes*: `K_low/K_high` from curve data or filled from reference spot when missing (env `YIELD_CURVE_REFERENCE_SPOT_{SYMBOL}`, default 6000).
- *Conv %*: convenience yield when present (synthetic writer emits 0.5%).
- Restart backend after code changes so `fill_missing_strikes` and latest writer (strikes, convenience yield) are active.

**Why Strikes show "—":** The backend’s `build_curve` handler fills missing `strike_low`/`strike_high` from the reference spot and `strike_width` (`fill_missing_strikes`). If you still see "—": (1) **Restart backend_service** so it runs the binary that includes this logic. (2) Ensure NATS and the yield-curve writer have run at least once (so KV has data). The handler now always runs `fill_missing_strikes` (using `YIELD_CURVE_REFERENCE_SPOT_{SYMBOL}` or default 6000), so after restart the table should show e.g. `5998/6002`.

Rates are in decimal internally (e.g. 0.05 = 5%); Long % / Short % / APR % show ×100. Spread is bid-ask width in basis points (positive). Synthetic writer uses a modest term premium (~1.2% per year) so 1Y is ~6%.

---

## 3. Both depend on NATS and backend

| Check              | Command / action |
|--------------------|------------------|
| NATS listening     | `lsof -i :4222` or `nats-server -c config/nats-server.conf` |
| Backend running    | `pgrep -fl backend_service` |
| Backend built recently | `cargo build -p backend_service` then restart |
| JetStream enabled  | NATS config has `jetstream { store_dir: "..." }` |
| Symbols configured | Backend config `[market_data] symbols = ["SPX", ...]` |

---

## 4. Script: `scripts/check_snapshot_yield_curve.sh`

Runs `snapshot-write` and `yield-curve --symbol SPX`, then prints concise next steps if either fails. Use from repo root:

```bash
./scripts/check_snapshot_yield_curve.sh
```
