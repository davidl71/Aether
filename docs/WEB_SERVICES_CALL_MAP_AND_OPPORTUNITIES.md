# Web services: who calls whom, and refactor/async/shared-data opportunities

## 1. Do backends call each other?

**No.** The Python backend services (IB, Alpaca, TradeStation, Tastytrade, Discount Bank, Risk-free rate) do **not** make HTTP calls to one another. Each talks only to its own external API (IB Gateway, Alpaca API, etc.). The only “cross-service” usage is clients (TUI and PWA) calling multiple backends.

---

## 2. Who calls which services?

### TUI (Python)

| Caller | Target | Purpose |
|--------|--------|---------|
| RestProvider / config | One snapshot endpoint (IB 8002, Alpaca 8000, etc.) | Poll snapshot (configurable single backend). |
| `app.py` | `http://localhost:8003/api/bank-accounts` | Fetch bank accounts (Discount Bank). |
| `box_spread_loader.py` | `{rest_endpoint_base}/scenarios` | Load box spread scenarios (same host as snapshot, different path). |
| BackendHealthAggregator | `http://127.0.0.1:{port}/api/health` per backend | Health checks for status line. |

So the TUI calls **one snapshot backend** + **Discount Bank** + **scenarios on snapshot host** + **health on several ports**. No backend-to-backend.

### PWA (React)

| Caller | Target | Purpose |
|--------|--------|---------|
| Snapshot client | `VITE_API_URL` (one snapshot origin) | Snapshot + health. |
| `AccountSelector` | Alpaca 8000, TradeStation 8001, Discount Bank 8003 | Accounts from each (currently **sequential**). |
| `useBankAccounts` | Discount Bank 8003 | Bank accounts. |
| `calculations.ts` | `VITE_API_URL` (default 9000) | Rust-owned frontend read models for cash flow timeline and opportunity simulation. |
| `useBackendServices` | All service ports (health) | Health checks (**parallel** via `Promise.allSettled`). |
| `useTastytrade` | Tastytrade 8005 | Health + snapshot. |
| ServiceConfigModal | `localhost:${port}/api/health` | Per-service health. |

So the PWA is the main **multi-service client**. It does not call one backend from another; it’s the browser calling several backends.

**Port note:** the active web path no longer uses a separate Python calculations service. The remaining Python rate/benchmark service stays on port 8004 when needed.

---

## 3. Refactor / async / shared-data opportunities

### 3.1 Parallelize account loading in PWA (done)

**Where:** `web/src/components/AccountSelector.tsx` – `loadAccounts()`.

**Before:** Fetches IB/Alpaca, then TradeStation, then Discount Bank **sequentially** (three round-trips in series).

**After:** Fetch all three in **parallel** with `Promise.allSettled` so the dropdown fills faster and one slow service doesn’t block the others.

**Change:** Replace the three sequential `try { await fetch(...) }` blocks with a single batch of `Promise.allSettled([ fetch(alpaca), fetch(tradestation), fetch(discountBank) ])` and merge results.

---

### 3.2 Single origin / BFF (backend-for-frontend)

**Idea:** Have the PWA call **one** origin (e.g. the nginx proxy at 8080, or a dedicated BFF). That single backend then calls IB, Alpaca, Discount Bank, calculations API, etc. **server-side** (async, in parallel), and returns one aggregated response.

**Benefits:** Fewer browser round-trips, one CORS/TLS story, optional auth in one place.

**Trade-off:** You add or extend one service that knows about all others; more coupling.

**Where:** New BFF route (e.g. `/api/aggregated/accounts`) or extend an existing service; frontend uses a single base URL.

---

### 3.3 Shared data source (NATS / event bus)

**Idea:** Backends **publish** snapshots or events (e.g. `snapshot.ib`, `snapshot.alpaca`, `accounts.discount_bank`) to NATS. TUI and PWA **subscribe** instead of polling REST. One consumer could aggregate and serve “latest from all” over one WebSocket or SSE.

**Benefits:** Less polling, lower latency for updates, one place to add caching or replay.

**Existing:** `docs/NATS_SETUP.md` and `docs/NATS_TOPICS_REGISTRY.md`; NATS is already in the stack.

**Where:** Each backend adds a “publish snapshot/accounts on change” step; TUI/PWA use a NATS-backed provider or a small aggregator service that subscribes and exposes REST/WS.

---

### 3.4 TUI: async bank-accounts fetch

**Where:** `python/tui/app.py` – `_fetch_bank_accounts()` uses `requests.get("http://localhost:8003/api/bank-accounts")` on a timer.

**Idea:** Run the request in a thread (e.g. `asyncio.to_thread`) or use an async HTTP client so the TUI event loop isn’t blocked. Optional: cache result for a few seconds to avoid hammering Discount Bank.

---

### 3.5 Retired calculations-service split

**Resolved:** cash flow and opportunity simulation moved to the Rust frontend API, so the old Python calculations/analytics service split is no longer part of the active web architecture.

---

### 3.6 Web: single config for all service URLs

**Where:** `web/src/config/ports.ts`, `getServiceUrl()`, and various `VITE_*` env vars.

**Idea:** When using the nginx proxy (single origin), the PWA could use **path-based** URLs only (e.g. `/api/ib/...`, `/api/alpaca/...`, `/api/discount_bank/...`) and one `VITE_API_BASE=http://localhost:8080`. Then `getServiceUrl(service)` becomes `base + /api/{service}/` and all traffic goes through the proxy. No need to configure each port separately in the frontend.

---

## 4. Summary

| Topic | Finding |
|-------|--------|
| Backend → backend | None; only clients call services. |
| Main multi-service client | PWA (AccountSelector, useBankAccounts, calculations, health, snapshot). |
| Quick win | **AccountSelector:** load accounts from Alpaca, TradeStation, Discount Bank in **parallel** (implemented). |
| Larger wins | BFF/aggregator (single origin), NATS as shared data source, TUI async/cache for bank accounts. |
| Config | Keep frontend read models on `VITE_API_URL`; optional single base URL when behind proxy. |
