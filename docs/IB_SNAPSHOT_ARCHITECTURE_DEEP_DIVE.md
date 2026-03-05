# IB Snapshot Pulling – Deep Architecture Dive

**Purpose:** Explain why IB snapshot is slow and where every millisecond goes.  
**Audience:** Engineers optimizing latency or debugging “snapshot pulling is way too slow.”

---

## 1. High-Level Data Flow

```
TUI/PWA (poll every 500ms–1s)
    │
    ▼ GET /api/v1/snapshot
┌─────────────────────────────────────────────────────────────────┐
│  IB Service (FastAPI, python/integration/ib_service.py)          │
│  - In-memory cache: SNAPSHOT_CACHE_SECONDS (default 3s)          │
│  - On cache miss → build_snapshot_payload()                       │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼ build_snapshot_payload()
    │
    ├─ 1. client.get_accounts()  ← SEQUENTIAL, before parallel block
    │       → GET https://localhost:5001/v1/portal/iserver/accounts
    │       → On 401: ensure_session() (see §3)
    │
    └─ 2. ThreadPoolExecutor(max_workers=3)  ← PARALLEL
            ├─ fetch_market_data()   → get_snapshots_batch(symbols)
            ├─ fetch_summary()       → get_account_summary(account_id)
            └─ fetch_positions()     → get_portfolio_positions(account_id)
    │
    ▼ After parallel join
    │
    ├─ 3. Post-process: symbol_snapshots, metrics, positions_data
    └─ 4. Optional: _build_cash_flow_timeline(positions_data)  ← CPU, can be heavy
```

**Critical point:** The “IB snapshot” path uses the **IB Client Portal Web API** (REST over HTTPS to the Gateway on port 5001), **not** the native TWS socket API. Every piece of data is one or more HTTP round-trips to the Gateway.

---

## 2. Request Inventory (Cold Path, No Cache)

Assume: 2 symbols (e.g. SPY, QQQ), single account, first request after startup (no session, no conid cache).

| Step | Component | Action | Round-trips / work |
|------|-----------|--------|--------------------|
| 1 | `build_snapshot_payload` | `client.get_accounts()` | 1× GET `/iserver/accounts` (or 2× + reauth if 401) |
| 2a | `fetch_market_data` | `get_snapshots_batch(symbols)` | |
| 2a1 | | `get_conid(sym)` per symbol | 2× POST `/iserver/secdef/search` (parallel within batch) |
| 2a2 | | `get_market_data_snapshots_batch(conids)` | 1× GET `/iserver/marketdata/snapshot?conids=...&fields=...` |
| 2a3 | | (each path calls `ensure_session()`) | 1× GET `/iserver/accounts` (or full reauth) |
| 2b | `fetch_summary` | `get_account_summary(account_id)` | `_choose_account` → get_accounts (cached?), then 1× GET `/iserver/account/{id}/summary` |
| 2b2 | | `ensure_session()` | 1× GET `/iserver/accounts` |
| 2c | `fetch_positions` | `get_portfolio_positions(account_id)` | `_choose_account` → get_accounts (cached?), then 1× GET `/iserver/account/{id}/positions` |
| 2c2 | | `ensure_session()` | 1× GET `/iserver/accounts` |
| 3 | Main thread | Cash flow timeline (if positions) | CPU only |

**Lower bound (best case, session valid, accounts cached):**

- 1× accounts (or 0 if from cache)
- 2× secdef/search (conids, cold)
- 1× marketdata/snapshot
- 1× account summary
- 1× positions  

→ **5–6 HTTP requests** even when nothing is wrong. With Gateway latency ~50–200 ms per request, that’s **~250 ms–1.2 s** before any Python post-processing.

**Worst case (session invalid, reauth):**

- Each of the three parallel branches can call `ensure_session()`.
- `ensure_session()`: GET accounts (401) → GET auth/status → POST reauthenticate → **sleep(0.5)** → GET accounts again.  
→ Up to **4 requests + 500 ms** per branch, so **~1.5 s+** just for session handling, then the data requests on top.

---

## 3. Why `ensure_session()` Hurts

- **Where:** `ibkr_portal_client.py` – `ensure_session()` is called from:
  - `get_account_summary()`
  - `get_portfolio_positions()`
  - `get_market_data_snapshots_batch()` (and hence inside `get_snapshots_batch()` via `search_contracts` → `get_conid` and the snapshot call)
- **What it does:**
  1. GET `/iserver/accounts`. If 200, return.
  2. If 401: GET `/iserver/auth/status`, POST `/iserver/reauthenticate`, **`time.sleep(0.5)`**, GET `/iserver/accounts` again.
- **Problem:** No shared “session valid” flag. So **every** data path does its own validation. With 3 parallel workers we can get **3×** the same check (or 3× reauth flows). The **0.5 s sleep** is in the critical path on reauth.

So: **redundant session checks** and **fixed 500 ms delay** on reauth both add directly to snapshot latency.

---

## 4. Conid Resolution (Market Data)

- **Flow:** `get_snapshots_batch(symbols)` → for each symbol `get_conid(symbol)` (in parallel) → then one `get_market_data_snapshots_batch(conids)`.
- **`get_conid()`:** Uses `_conid_cache`. On cache miss it calls `search_contracts(symbol)` = **POST `/iserver/secdef/search`**.
- **Cost:** One POST per symbol per **process lifetime** until cache is warm. For SPY, QQQ that’s 2 POSTs (in parallel). For 10 symbols, 10 POSTs. Each is a Gateway round-trip (same 50–200 ms order).
- **Persistence:** Cache is in-memory only. Restart the IB service and all conids are cold again.

So: **cold conid cache** multiplies round-trips by (1 + N symbols) for market data, and **restarts keep forcing cold cache**.

---

## 5. Accounts and `_choose_account`

- **`get_accounts()`:** Uses a short TTL cache (`ACCOUNTS_CACHE_TTL_SECONDS = 2.0`). Good.
- **`_choose_account(account_id)`:** Used by `get_account_summary` and `get_portfolio_positions`. It calls **`get_accounts()`** again to resolve “which account to use.”
- So in one snapshot we can call `get_accounts()`:
  - Once in `build_snapshot_payload` (to get `effective_account_id`).
  - Again in `fetch_summary` and again in `fetch_positions` via `_choose_account`.  
  With the 2 s TTL we often hit cache, but we still do **redundant lookups** and dependency on account list for every summary/positions call.

---

## 6. Cache Layers and Polling

- **Snapshot response cache:** Key `(symbols_tuple, account_id)`, TTL = `SNAPSHOT_CACHE_SECONDS` (default **3**). So when the TUI polls every **500 ms**, we get at most 1 fresh build per 3 s and more cache hits. Good for load, but when we do build, we pay full cost.
- **Accounts cache:** 2 s TTL in the portal client. Reduces repeated GET `/iserver/accounts` within the same snapshot and across quick successive snapshots.
- **Conid cache:** No TTL, process-scoped. Helps only after the first resolution per symbol; restarts clear it.

So: **First request after startup or after cache expiry** always pays the full cold path. Polling interval (500 ms–1 s) doesn’t reduce that; it only determines how often we *attempt* a build vs hit cache.

---

## 7. Post-Processing

- **Cash flow timeline:** `_build_cash_flow_timeline(positions_data)` runs **after** the parallel fetch. It’s pure Python (and can pull in `cash_flow_timeline` logic). For many positions or complex logic this can add tens of milliseconds. Not the main cost, but non-trivial on a “way too slow” path.

---

## 8. Gateway and Network

- **Each request:** TLS handshake (if not kept alive), Gateway processing, and Gateway ↔ IB backend. Documented and observed latencies are often in the **50–200 ms per request** range for localhost. So 5–6 requests → **250 ms–1.2 s** even without reauth or cold conids.
- **Connection reuse:** `requests.Session()` is used, so TCP/TLS reuse helps within the process. Still one round-trip per logical request.

---

## 9. Summary: Why It Feels “Way Too Slow”

| Cause | Impact |
|-------|--------|
| **Many round-trips** | 5–6+ HTTP calls per snapshot (accounts, conids, snapshot, summary, positions). 50–200 ms each → 250 ms–1.2 s. |
| **`ensure_session()` in every path** | 3× session checks (or reauths) in parallel; reauth includes **500 ms sleep**. |
| **Cold conid cache** | N extra POSTs for N symbols; every service restart resets cache. |
| **Serial get_accounts() then parallel** | One round-trip before the parallel block; plus `_choose_account` calls get_accounts again in 2 branches. |
| **Cache TTL vs poll interval** | When cache misses (e.g. first request, or after 3 s), full cost is paid; 500 ms polling doesn’t reduce that. |
| **Post-processing** | Cash flow timeline adds some CPU after data is in. |

---

## 10. Recommendations (Short List)

1. **Ensure session once per snapshot**  
   Call `ensure_session()` (or a single “session valid” check) **before** submitting the 3 parallel tasks. Remove or gate `ensure_session()` inside `get_account_summary`, `get_portfolio_positions`, and `get_snapshots_batch` when building a snapshot so they don’t each re-validate.

2. **Pass account_id through; avoid redundant get_accounts**  
   `build_snapshot_payload` already has `effective_account_id`. Pass it into the portal client for summary and positions so they don’t need to call `_choose_account` → `get_accounts()` again (or make `_choose_account` use a pre-resolved list when account is already known).

3. **Persist or prewarm conids**  
   Either persist conid cache (e.g. file or Redis with TTL) or, on startup, trigger one batch resolution for default symbols so the first snapshot doesn’t pay N extra POSTs.

4. **Increase snapshot cache TTL**  
   e.g. `SNAPSHOT_CACHE_SECONDS=5` (or configurable) so more polls are served from cache and we do fewer full builds. Trade-off: data freshness.

5. **Optional: single “dashboard” endpoint on Gateway**  
   If the Client Portal API ever offers one call that returns accounts + summary + positions + market snapshots, using it would cut round-trips to one. Today we’re constrained by the existing REST surface.

6. **Reduce reauth sleep**  
   Replace or reduce the fixed `time.sleep(0.5)` in `ensure_session()` (e.g. shorter backoff or make it configurable) so reauth doesn’t add a hard 500 ms every time.

7. **Profile and optionally defer cash flow timeline**  
   If `_build_cash_flow_timeline` is expensive, consider computing it asynchronously or on a longer interval and attaching it to a later snapshot or a separate endpoint.

---

## 11. Related Files

| File | Role |
|------|------|
| `python/integration/ib_service.py` | Snapshot endpoint, `build_snapshot_payload`, cache, parallel fetch |
| `python/integration/ibkr_portal_client.py` | All Gateway REST calls, `ensure_session`, `get_accounts`, conid cache, `get_snapshots_batch` |
| `python/integration/cash_flow_timeline.py` | Post-processing after positions are fetched |
| `python/tui/providers.py` | REST provider poll loop (interval) |
| `python/tui/app.py` | `set_interval(0.5, _update_snapshot)` |
| `docs/research/learnings/ECLIENT_EWRAPPER_ARCHITECTURE.md` | TWS socket API (different from Client Portal used here) |
| `docs/IBKR_POSITION_RETRIEVAL.md` | Native TWS position retrieval (C++, not used by Python snapshot) |

---

## 12. Diagram: Cold Snapshot Request (Simplified)

```
Time →
[Main thread]
  get_accounts() ──────────────────────────────────────────► 1× GET /iserver/accounts (or reauth)
  effective_account_id = accounts[0]

  ThreadPoolExecutor(3)
  ├─ Worker 1 (market data):
  │    ensure_session() ──► GET /iserver/accounts
  │    get_conid(SPY) ────► POST /iserver/secdef/search
  │    get_conid(QQQ) ───► POST /iserver/secdef/search
  │    get_market_data_snapshots_batch([conid1, conid2]) ► GET /iserver/marketdata/snapshot?conids=...
  │
  ├─ Worker 2 (summary):
  │    ensure_session() ──► GET /iserver/accounts
  │    _choose_account() ─► get_accounts() (cached)
  │    GET /iserver/account/{id}/summary
  │
  └─ Worker 3 (positions):
       ensure_session() ──► GET /iserver/accounts
       _choose_account() ─► get_accounts() (cached)
       GET /iserver/account/{id}/positions

  ◄──────────────────────────────────────────────────────── join
  Build payload, cash_flow_timeline(positions), return JSON
```

So: **IB snapshot slowness** is dominated by **many Client Portal REST round-trips**, **repeated session checks**, **cold conid resolution**, and **fixed reauth sleep**. The architecture is pull-based REST with process-local caches; the main levers are reducing round-trips and ensuring session and account resolution once per snapshot.

---

## 13. Implementation Status

The following improvements from §10 have been implemented:

| Improvement | Status | Notes |
|-------------|--------|--------|
| Ensure session once per snapshot | Done | `build_snapshot_payload()` calls `ensure_session()` once before the parallel block and sets a request-scoped flag so workers skip `ensure_session()`. |
| Pass account_id through; avoid redundant get_accounts | Done | `_choose_account(explicit)` returns `[explicit]` when non-empty without calling `get_accounts()`. |
| Prewarm conids | Done | `prewarm_conids(symbols)` on the portal client; triggered lazily on first snapshot build per symbol set. |
| Configurable reauth sleep | Done | `REAUTH_SLEEP_SECONDS` env (default 0.5, clamp 0.1–2.0) in `ibkr_portal_client.py`. |
| Snapshot cache TTL | Documented | `SNAPSHOT_CACHE_SECONDS` (default 3) documented in `ib_service.py` and `web/IB_INTEGRATION.md`; increase to 5 to reduce load further. |
| Defer cash flow timeline | Out of scope | Left for a follow-up if profiling justifies it. |
