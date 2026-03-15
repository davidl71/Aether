# How to Remove REST

**Status:** Option B (full removal) was implemented. Backend is NATS-only; no HTTP server.

**Follow-up (NATS-only):** Design first, then per-area implementation or removal. All six area tasks depend on the design task.

**Design task (T-1773515543967282000) — subtasks:**
- T-1773515638505099000: Document existing NATS subjects and usage (factual inventory). — **Done (Review)**; content in `docs/platform/NATS_API.md` §1.
- T-1773515639823169000: Propose request/reply naming and payload pattern (draft for NATS_API.md). — **Done (Review)**; content in `docs/platform/NATS_API.md` §2.
- T-1773515642013883000: **Human review:** Approve scope per area (expose vs remove) and finalize NATS_API.md. All area subtasks depend on this.

**Area tasks — each has two concrete subtasks (do one per area after design approves scope):**
- **Discount Bank** T-1773515415166769000: Implement NATS handlers (T-1773515650236363000) **or** remove dead code + doc (T-1773515651227121000).
- **Finance rates** T-1773515417559897000: Implement NATS (T-1773515652084062000) **or** remove + doc (T-1773515652629516000).
- **Calculate** T-1773515420144376000: Implement NATS (T-1773515653481583000) **or** library-only + remove API (T-1773515654155373000).
- **Loans + Config** T-1773515423634118000: Implement NATS (T-1773515656182114000) **or** remove + doc (T-1773515656690718000).
- **Strategy + orders/positions** T-1773515427981197000: Wire strategy control via NATS (T-1773515657237625000); optional orders/positions read (T-1773515658093601000) if needed.
- **FMP/chart/Swiftness/frontend** T-1773515431075544000: Implement NATS (T-1773515659705668000) **or** remove + doc (T-1773515660499693000).

Snapshot (positions, orders) is already NATS-only for TUI.

Options for dropping the HTTP REST (and WebSocket) server so the backend is NATS-only. TUI already uses NATS only; no in-repo client depends on REST today.

---

## Current wiring

- **backend_service** binds `rest_addr` (default 9090), builds `RestState`, and runs `RestServer::serve(rest_addr, rest_state)`. Same process also runs NATS snapshot publisher (uses `SharedSnapshot` only, not `RestState`).
- **api**: `rest.rs` has all routes and `RestServer`/`RestState`; `websocket.rs` is merged into the same Axum app. `StrategyController` lives in `rest.rs` and is used by backend_service for strategy start/stop.
- **LIVE_STATE** KV: Created in backend_service and passed into `RestState` for REST handlers `live_state` / `live_state_watch`. `collection_aggregation` still *writes* to LIVE_STATE; only REST *reads* it. If REST goes away, we simply stop passing the store into api.

---

## Option A: Minimal – don’t start the server

**Goal:** No HTTP listener; keep all REST/WS code for later or for other entry points.

**Changes:**

1. **backend_service/src/main.rs**
   - Stop calling `RestServer::serve` and remove the handle (e.g. comment out or gate with `if false` or a config flag).
   - Optionally stop building `rest_state` and `live_state_kv` if you want to avoid unused work; otherwise leave as-is.

**Pros:** One place to change; easy to revert.  
**Cons:** Large amount of dead code (all of `rest.rs`, `websocket.rs`, `handlers/`) and axum/tower deps remain.

---

## Option B: Full removal – delete REST and WebSocket

**Goal:** No REST/WS code or HTTP deps in the backend; NATS-only.

**Steps:**

1. **Extract `StrategyController` from api**
   - Add `api/src/strategy_controller.rs` with `StrategyController` (and its `watch::Sender<bool>`) moved from `rest.rs`.
   - In `api/src/lib.rs`: `pub use strategy_controller::StrategyController`, remove `RestServer`/`RestState` from exports.

2. **Remove REST/WS and their state**
   - Delete or gut `api/src/rest.rs` (no routes, no `RestServer`, no `RestState`). If you keep the file, it would only re-export or leave placeholders; cleaner to delete.
   - Delete `api/src/websocket.rs`.
   - Delete `api/src/handlers/` (chart, config, health, models, orders, strategy) or at least all code that only exists to serve REST/WS.

3. **api/src/lib.rs**
   - Remove `mod rest;` and `mod websocket;`, remove `pub use rest::{...};` and `pub use websocket::WebSocketServer`.
   - Add `mod strategy_controller;` and `pub use strategy_controller::StrategyController`.
   - Remove any `mod handlers;` (or the individual handler modules) if they were only used by the router.

4. **api/Cargo.toml**
   - Drop axum, tower, and any deps that were only used by rest/websocket (e.g. axum’s ws feature, tower). Keep deps still needed for runtime_state, health, loans, discount_bank, etc.

5. **backend_service/src/main.rs**
   - Remove `RestServer`, `RestState` from imports.
   - Remove `rest_addr` from config (and from `BackendConfig` / defaults) and any `rest_handle`.
   - Stop building `rest_state` and stop calling `RestServer::serve`. Optionally stop creating `live_state_kv` if nothing else needs it; `collection_aggregation` can keep publishing to LIVE_STATE without passing the store into api.

6. **TUI / config**
   - TUI is NATS-only; remove or ignore any `rest_url` / REST fallback config that was only for the old REST snapshot endpoint. Update defaults or docs so it’s clear the backend has no HTTP API.

**Pros:** No dead REST/WS code; smaller api crate and fewer deps.  
**Cons:** Any future HTTP (e.g. health, debug, or a new UI) would need a new small server or a separate binary.

---

## Recommendation

- Use **Option A** if you might re-enable REST soon or want a single config toggle to bring it back.
- Use **Option B** if you’re committed to NATS-only and want a cleaner codebase; add a new minimal HTTP (or WS) server later only if needed.

---

## Files to touch (Option B)

| Action | Path |
|--------|------|
| Create | `api/src/strategy_controller.rs` |
| Delete or gut | `api/src/rest.rs` |
| Delete | `api/src/websocket.rs` |
| Delete | `api/src/handlers/*.rs` (and `handlers/mod.rs`) |
| Edit | `api/src/lib.rs` |
| Edit | `api/Cargo.toml` |
| Edit | `backend_service/src/main.rs` |
| Optional | TUI/config: rest_url, rest_fallback, snapshot REST fallback docs |
