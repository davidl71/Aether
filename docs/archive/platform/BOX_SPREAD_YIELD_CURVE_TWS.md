# Box Spread Yield Curve from TWS Data

**Purpose:** Document how the box-spread yield curve is built or sourced for scenarios, pricing, and display; the TWS (IBKR) integration point; and fallback when TWS data is missing.

**See also:** `NATS_KV_USAGE_AND_RECOMMENDATIONS.md` (KV key format), `NATS_API.md` (§4 finance_rates), `IB_ADAPTER_REVIEW.md` (paper/live, ports).

---

## 1. Architecture: TWS Separate from NATS

TWS API interaction is in a **standalone crate** with no NATS dependency. CLI and tests can use it directly; NATS is an optional output path.

| Layer | Role | Location |
|-------|------|----------|
| **tws_yield_curve** | TWS-only: connect, option chain, option quotes → `Vec<Value>` (`{ "spread": BoxSpreadInput }`). No NATS. | `crates/tws_yield_curve` |
| **api (finance_rates)** | Pure calculation: `build_curve(CurveRequest, CurveQuery)` → `CurveResponse`. No TWS, no NATS. | `crates/api/src/finance_rates.rs` |
| **CLI** | Can use **direct TWS** (`--source tws`) or **NATS** (`--source nats`, default). Optional `--publish-to-nats` to publish the result to `yield_curve.direct.{symbol}`. | `bin/cli` |
| **Backend** | NATS handler for `api.finance_rates.build_curve`: when `YIELD_CURVE_USE_TWS=1` and request empty, calls `tws_yield_curve::fetch_yield_curve_from_tws`, then `build_curve`; responds via NATS. Can also load from KV or use synthetic. | `services/backend_service` |
| **NATS KV** | Optional sink: key `yield_curve.{symbol}`. Filled by yield curve writer (synthetic) or by backend when using KV path. | `api_handlers.rs`, `yield_curve_writer.rs` |
| **TUI Yield tab** | Requests `api.finance_rates.build_curve` via NATS; backend fills from TWS, KV, or synthetic and returns `CurveResponse`. | `tui_service` |

**Direct use (no backend):** `cargo run -p cli -- yield-curve --symbol SPX --source tws` uses the TWS crate and `api::finance_rates::build_curve` only; no NATS or backend required. Add `--publish-to-nats` to also publish the curve to NATS.

**TWS client ID allocation (paper/live port 7497/7496):** TWS allows up to 32 clients per session. To avoid "client id is already in use":

| Client ID | Component | Env / config |
|-----------|-----------|--------------|
| 0 | backend_service TWS market data | `TWS_CLIENT_ID` (default 0) |
| 1 | backend_service TWS positions | `TWS_CLIENT_ID` + 1 |
| **12** | **tws_yield_curve** (CLI and backend build_curve when `YIELD_CURVE_USE_TWS=1`) | `TWS_CLIENT_ID` (default **10**) + 2 |

If you see "client id 2" in TWS, it was from an older run of the backend or CLI (yield-curve path) before the default was moved to 12. Restart the backend so it uses 12; then 2 is free for other tools or leave unused.

**TWS_CLIENT_ID must be a valid integer.** If the variable is set but not parseable as `i32` (e.g. empty, non-numeric, or leading/trailing spaces), it is **ignored** and the component default is used (0 for market data/positions, 10 for yield curve). A warning is logged so you can fix the value. Use e.g. `TWS_CLIENT_ID=0` or `TWS_CLIENT_ID=10`, not `TWS_CLIENT_ID= auto` or unquoted spaces.

**SPX, NDX, XSP are index underlyings.** The yield-curve code requests the option chain with `SecurityType::Index` and exchange `CBOE` for these symbols. Using `STK` (stock) for SPX causes TWS to return "Invalid contract id" (error 321). For **market data** on index options, the code resolves each option via `reqContractDetails` and uses the returned contract (with conid) for `reqMktData`, which avoids error 200 ("Invalid contract id" on the market data request). Other symbols (e.g. SPY, QQQ) use `SecurityType::Stock` and `SMART` and do not need contract_details resolution.

**Standalone daemon (no backend rebuild):** Run `tws_yield_curve_daemon` as a separate process. It fetches from TWS and writes to NATS KV `yield_curve.{symbol}` on an interval. Backend and TUI read from KV as usual; no need to enable TWS in the backend or rebuild it. Build/run only the daemon when iterating on TWS logic.

```bash
# Terminal 1: NATS (e.g. nats-server -js). Terminal 2: TWS/IB Gateway.
# Terminal 3: daemon (optional TWS_PORT=7496, SYMBOLS=SPX, INTERVAL_SECS=60)
cd agents/backend && cargo run -p tws_yield_curve_daemon
# Or: just run-tws-yield-daemon
```

## 2. Current State (reference)

| Component | Role | Location |
|-----------|------|----------|
| **NATS KV** | Sink for yield curve per symbol. Key: `yield_curve.{symbol}`. Value: JSON array of `{ "spread": BoxSpreadInput }`. Bucket: `NATS_KV_BUCKET` (default `LIVE_STATE`). | `api_handlers.rs`, `yield_curve_writer.rs` |
| **api.finance_rates.build_curve** | NATS request/reply: accepts `CurveRequest`. If empty + symbol, backend may use TWS (when `YIELD_CURVE_USE_TWS=1`), KV, or synthetic, then builds curve. | `api_handlers.rs`, `api/finance_rates.rs` |
| **Yield curve writer** | **Synthetic** source: interval-based write to `yield_curve.{symbol}`. | `backend_service/src/yield_curve_writer.rs` |
| **TUI Yield tab** | Requests build_curve via NATS; backend responds with curve. | `tui_service` |

---

## 3. Integration Point: TWS API and Backend Adapter

### 3.1 TWS API (IBKR)

To build a **real** box-spread yield curve from TWS:

- **Option chain data** is required: for each symbol (e.g. SPX, SPXW) and expiry, need bid/ask (or last) for the four legs of a box (two calls, two puts, two strikes).
- **Relevant TWS/API calls** (conceptually; actual API depends on ibapi/EClient):
  - **reqSecDefOptParams** / option chain request: get expirations and strikes for an underlying.
  - **reqMktData** (or snapshot) for each option contract: get bid/ask/last to compute implied rates.
- **Box spread identification:** For each expiry, take two strikes (e.g. K1, K2); the box is long call K1 + long put K2 + short call K2 + short put K1. From mid prices (or bid/ask), compute net debit/credit and implied financing rate per `BoxSpreadInput` (see `api/finance_rates.rs`).

**Current backend:** `tws_market_data.rs` subscribes to **stock** contracts only (`Contract::stock(&symbol)`), not options. It does **not** request option chains or option ticks. So today there is **no TWS-sourced option data** for the yield curve.

### Box spread net debit/credit in positions/snapshot

For **existing box positions** (from TWS/Client Portal), we surface net debit/credit in the snapshot and API so the UI can show spread bid/ask:

- **Data source:** Today we use **leg_sum_mark**: `api::combo_strategy::apply_derived_strategy_types` groups positions by combo key, infers Box, and sets `combo_net_bid` / `combo_net_ask` to `sum(leg.mark * leg.quantity)` with `combo_quote_source = "leg_sum_mark"`. Future sources: **leg_sum** when per-leg bid/ask exist; **tws** when TWS provides a combo quote.
- **Exposure:** Optional fields on each position in the snapshot (proto `Position`, REST/NATS snapshot): `combo_net_bid`, `combo_net_ask`, `combo_quote_source`. See [TWS_COMPLEX_ORDER_ECOSYSTEM.md](TWS_COMPLEX_ORDER_ECOSYSTEM.md) for the source table.

### 3.2 Backend Adapter Options

| Approach | Description | Pros / cons |
|----------|-------------|-------------|
| **A. Extend tws_market_data** | Add option-chain requests and, for each symbol/expiry, subscribe to the four legs of selected box spreads; aggregate ticks into `BoxSpreadInput` and write to KV. | Reuses existing TWS connection; adds option subscription and box logic to one module. |
| **B. Dedicated TWS yield-curve job** | New task (e.g. `tws_yield_curve.rs`): connect to TWS (separate client_id to avoid conflict with `tws_market_data` and `tws_positions`), request option chain per symbol, compute box spreads, write `yield_curve.{symbol}` to KV. | Clear separation; same KV key so TUI/API unchanged. Requires option chain + box-spread logic. |
| **C. External process** | Script or service (e.g. Python/other) calls TWS API, computes curve, writes to NATS KV (same key format). | Backend stays unchanged; operational complexity of another process. |

**Recommendation:** For a first TWS-backed curve, either **B** (dedicated task in backend_service) or **C** keeps the existing NATS/KV contract; the TUI and `api.finance_rates.build_curve` need no change. The **integration point** is: **whoever produces the curve must write a JSON array of `{ "spread": BoxSpreadInput }` to NATS KV key `yield_curve.{symbol}`**.

---

## 4. Real-Time vs Snapshot

| Mode | Description | Current / possible |
|------|-------------|--------------------|
| **Snapshot (interval)** | On a timer (e.g. every 60s), fetch option chain / box spreads and overwrite `yield_curve.{symbol}` in KV. | **Current:** synthetic writer runs on `YIELD_CURVE_WRITER_INTERVAL_SECS` (default 60). A TWS job could do the same. |
| **Real-time (stream)** | On every tick (or batched), recompute affected box spreads and update KV. | Possible with TWS option ticks; higher load and complexity; KV put on every update. |

**Practical choice:** Start with **snapshot on interval** (same as current synthetic writer). Move to real-time only if the product needs sub-minute curve updates.

---

## 4. Paper vs Live

Same convention as existing TWS components (`tws_market_data.rs`, `tws_positions.rs`, `IB_ADAPTER_REVIEW.md`):

| Environment | Typical port | Env |
|-------------|--------------|-----|
| **Paper** | 7497 (TWS), 4002 (IB Gateway) | `TWS_PORT=7497` or unset (autodetect tries paper first) |
| **Live** | 7496 (TWS), 4001 (IB Gateway) | `TWS_PORT=7496` |

`TWS_HOST` (default `127.0.0.1`) and `TWS_CLIENT_ID` apply. A TWS yield-curve task must use a **distinct client_id** (e.g. `TWS_CLIENT_ID + 2`) so it does not conflict with market-data (0) and positions (1).

---

## 5. Fallback When TWS Data Is Missing

| Situation | Behavior |
|-----------|----------|
| **No TWS connection** | Synthetic writer continues to run and overwrites `yield_curve.{symbol}` with synthetic points. TUI and `build_curve` keep working with that data. |
| **TWS-backed writer running** | If a TWS job writes the key, it overwrites KV. Disable or don’t start the synthetic writer when a live TWS source is used (e.g. config flag `yield_curve.source = "tws"` vs `"synthetic"`). |
| **KV read failure** | If `build_curve` is called with empty opportunities and symbol, but KV get fails or key is missing, `load_yield_curve_opportunities_from_kv` returns `None` and the request uses empty opportunities; `build_curve` then returns a curve with zero points. TUI shows empty or “no data” as already handled. |
| **Stale KV** | KV has no TTL in this design; last writer wins. If TWS disconnects and synthetic writer is disabled, keys stay at last TWS write until overwritten or backend restarted with synthetic enabled. |

**Recommended:** Keep the synthetic writer as default so that with no TWS, the Yield tab still shows a curve. When a TWS yield-curve source is enabled, disable the synthetic writer for those symbols (or entirely) to avoid overwriting live data.

---

## 6. Summary

- **Build or source:** Curve is **sourced** from NATS KV key `yield_curve.{symbol}`. **Built** from that array by `api.finance_rates.build_curve` (same as today). A **TWS source** would: (1) connect to TWS, (2) request option chain / box spreads, (3) produce `BoxSpreadInput` list, (4) write to KV.
- **Integration point:** TWS API (option chain + mkt data) → backend task or external process → NATS KV `yield_curve.{symbol}`. Backend adapter: either new `tws_yield_curve` task or extension of TWS market-data with option support.
- **Real-time vs snapshot:** Current design is snapshot (interval). Real-time is optional and would require TWS option ticks and more frequent KV writes.
- **Paper vs live:** Use same ports and env as `tws_market_data` / `tws_positions`; separate client_id for yield-curve task.
- **Fallback:** Synthetic writer fills KV when no TWS source is present; disable it when a live TWS yield-curve writer is used.
