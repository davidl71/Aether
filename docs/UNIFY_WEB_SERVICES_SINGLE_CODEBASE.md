# Unifying web services into a single codebase (call functions directly)

## Question

Where does it make sense to unify web services into a **single codebase** so that one service can **call another’s logic directly** (in-process) instead of over HTTP?

---

## Current layout

| Service | Port | Role | Depends on |
|--------|------|------|------------|
| ib_service | 8002 | Snapshot (IB Gateway) | ibkr_portal_client |
| alpaca_service | 8000 | Snapshot + accounts | alpaca_client |
| tradestation_service | 8001 | Snapshot + accounts | tradestation client |
| tastytrade_service | 8005 | Snapshot + accounts | tastytrade client |
| discount_bank_service | 8003 | Bank accounts (file/DB) | file + ledger DB |
| risk_free_rate_service | 8004 | Yield curve, SOFR/Treasury | risk_free_rate_extractor, sofr_treasury_client |
| calculations_api | 8004* | Cash flow, opportunity simulation | cash_flow_timeline, opportunity_simulation_calculator |

\* calculations_api and risk_free_rate_service both default to 8004; run one on another port.

**Shared today:** All use `security_integration_helper`; calculations and risk-free rate are thin REST wrappers around `python/integration/` modules. **No backend calls another over HTTP.**

---

## Where unification makes sense

### 1. **Calculations + Risk-free rate → one “analytics” app** (high value, low risk) ✅ Implemented

**Idea:** Merge into a single FastAPI app (e.g. keep `calculations_api.py` and mount risk-free rate routes, or create `python/services/analytics_api.py`).

**Implementation:** `python/services/analytics_api.py` includes both `router_calculations` (from `calculations_api`) and `router_risk_free_rate` (from `risk_free_rate_service`). Single port **8007** (env `ANALYTICS_API_PORT` or `CALCULATIONS_API_PORT`). Run with `./web/scripts/run-analytics-api.sh` or `python -m uvicorn services.analytics_api:app --host 0.0.0.0 --port 8007` from the `python` directory. Supervisord program `analytics` (autostart); systemd user unit `ib-box-spread-analytics.service`. Nginx location `/api/analytics/` proxies to 8007. PWA: set `VITE_CALCULATIONS_API_URL=http://localhost:8007` to use the unified API for cash-flow and opportunity simulation; risk-free rate endpoints are on the same origin.

**Why it fits:**

- Both are **stateless** and only wrap integration modules.
- No long-lived broker connections; same process can safely host both.
- **Direct calls:** A future endpoint (e.g. “yield curve vs opportunity scenarios”) can call `calculate_cash_flow_timeline`, `find_available_scenarios`, and `RiskFreeRateExtractor` / `SOFRTreasuryClient` in-process without HTTP.
- One port, one process, simpler deployment and no 8004 collision.

**Concrete steps:** Done. Routers are exposed from `calculations_api` and `risk_free_rate_service`; `analytics_api.py` includes both and serves on 8007.

**Result:** Single codebase for “analytics”; all calls between calculations and risk-free rate are in-process function calls. **Done:** See `python/services/analytics_api.py`, port 8007, and config (supervisord, nginx, systemd, scripts/service.sh).

---

### 2. **BFF (Backend-for-Frontend) that imports broker + analytics logic** (high value, higher effort)

**Idea:** One FastAPI app that the PWA (and optionally TUI) calls exclusively. It **imports** the same Python modules the current services use and exposes routes that call functions directly instead of issuing HTTP to other services.

**Why it fits:**

- PWA today calls many origins (IB, Alpaca, TradeStation, Discount Bank, calculations). A BFF can expose e.g. `/api/aggregated/accounts` and call `alpaca_client.get_accounts()`, tradestation client, and discount_bank ledger logic **in-process**, in parallel (e.g. `asyncio.gather` or `ThreadPoolExecutor`).
- Aggregated snapshot, cash-flow, opportunity simulation, risk-free rate can all be in-process calls.
- Single origin for the frontend (CORS, auth, deployment simplicity).

**Design options:**

- **Option A – BFF only aggregates; brokers still run separately:**  
  BFF calls broker services over HTTP (as today) but aggregates in one place. No “direct function calls” for broker logic; only aggregation and possibly analytics in-process.

- **Option B – BFF hosts broker logic in-process:**  
  BFF imports `ib_service.build_snapshot_payload`, AlpacaClient, etc., and runs them inside the same process (e.g. in threads or async). Then **all** cross-“service” calls are direct. Trade-off: one process holds all broker credentials and connections; a bug or heavy load in one broker can affect others. Best if you want one deployment unit and can tolerate that.

**Result:** Single codebase (the BFF) that can call snapshot builders, account fetchers, calculations, and risk-free rate **directly**; frontend talks to one origin.

---

### 3. **Broker snapshot services (IB, Alpaca, TS, Tastytrade) in one process** (possible but usually not worth it)

**Idea:** One FastAPI app with routers: `/api/ib/`, `/api/alpaca/`, `/api/tradestation/`, `/api/tastytrade/`. Each router uses the same modules as today; “aggregated snapshot” could call `build_snapshot_payload` for each broker in parallel **in-process**.

**Downsides:**

- One process holds **all** broker connections and credentials.
- Different env vars and lifecycles per broker; more complex startup and config.
- A crash or memory spike in one adapter affects all.
- You already get “single origin” for the browser via nginx path-based routing; unifying processes doesn’t improve that.

**When it might be worth it:** You explicitly want a single process that can combine multiple broker snapshots in one response with minimal latency and no HTTP between “services.” Even then, a BFF (option 2) that imports these builders is usually clearer than merging the four services into one.

**Result:** Direct function calls between broker snapshot builders in one process; not recommended unless you have a strong reason to colocate them.

---

### 4. **Discount Bank as a router inside another app** (optional)

**Idea:** Discount Bank only reads files and SQLite; no long-lived connection. Its routes could be mounted into the “analytics” app or the BFF so `/api/bank-accounts` is served in-process.

**Benefit:** One less process; BFF or analytics can call the same bank-account logic directly when building aggregated views.

**Result:** Direct calls to bank-account logic from the same app that serves aggregated accounts or cash-flow.

---

## Summary: where to unify for direct calls

| Unification | Call functions directly? | Suggested action |
|-------------|--------------------------|-------------------|
| **Calculations + Risk-free rate** | Yes; shared analytics app | Merge into one FastAPI app (e.g. extend calculations_api with risk-free rate routes). |
| **BFF** | Yes; BFF imports all builders/calculators | Add a BFF that exposes aggregated and per-service routes and calls integration modules in-process. |
| **All broker services in one process** | Yes, but mixed credentials/lifecycle | Prefer BFF that imports broker logic; only merge processes if you need a single aggregation process. |
| **Discount Bank into BFF or analytics** | Yes | Optional: mount discount_bank routes (or reuse its logic) in BFF or analytics app. |

**Recommended order:** (1) Merge calculations + risk-free rate into one app. (2) If you want a single frontend origin and fewer round-trips, add a BFF that calls those plus (optionally) broker and discount-bank logic in-process.

---

## Existing in-process usage

- **TUI** already calls `calculate_cash_flow_timeline` and `find_available_scenarios` **directly** from `python/integration/` (same process as the TUI); it does not use the calculations_api HTTP for those.
- **IB service** optionally imports `cash_flow_timeline` and calls `calculate_cash_flow_timeline` in-process when building the snapshot (see `_build_cash_flow_timeline` in `ib_service.py`).
- **calculations_api** is a thin HTTP wrapper around the same integration modules; merging it with risk_free_rate_service keeps that “direct call” model and removes one service and the 8004 conflict.

---

## Further optimizations (.pyc, Cython, C++ dedup)

### .pyc (bytecode)

- Python compiles to `.pyc` and caches in `__pycache__`; no extra step for “dedup.”
- Slightly faster import after first run. Shipping precompiled bytecode (`python -m compileall`) or keeping `.pyc` in deployment does **not** remove duplication between Python and C++.

### Cython to deduplicate C++ code

- **Cython is already used:** `python/bindings/box_spread_bindings.pyx` (and `.pxd`) expose C++ box spread / option / risk types to Python. Build: `cmake --build build --target python_bindings` or `cd python/bindings && pip install -e .`.
- **Dedup strategy (see CROSS_LANGUAGE_DEDUP_PLAN):** C++ is the single source for box spread math and risk; Python should call C++ via these bindings. Today `python/integration/box_spread_strategy.py` and risk code still have a full Python port; the plan is to **thin them to wrappers** that call the Cython bindings.
- **Next steps:** (1) Expose all 14 calculator/validator methods in the bindings and use them from `box_spread_strategy.py`. (2) Add C++ risk bindings or document Python risk as stub (Phase 5). (3) No new Cython layer needed—use existing bindings to remove duplicated logic.
- **Nuitka:** Optional whole-program compile of Python services to native; consider only if HTTP service startup/throughput becomes a bottleneck.

