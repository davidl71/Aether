# Background-task lifecycle: current state and suggested improvements

This document describes how background work (threads, asyncio tasks, timers) is started and stopped across the Python TUI, FastAPI backend services, and standalone scripts. It then suggests a clearer lifecycle so shutdown is predictable and testable.

---

## 1. Current state

### 1.1 TUI (Textual app)

**Location:** `python/tui/app.py`, `python/tui/providers.py`

| Component | How it runs | Start | Stop | Notes |
|-----------|-------------|--------|------|--------|
| **Provider** (Mock, Rest, File) | `threading.Thread(target=_poll_loop or _generate_loop, daemon=True)` | `on_mount`: `provider.start()` | `on_unmount`: `provider.stop()` | Each provider has one worker thread; `stop()` sets `_running=False` and `join(timeout=2.0)`. |
| **BackendHealthAggregator** | Same: `Thread(target=_poll_loop, daemon=True)` | `on_mount`: `aggregator.start()` (if ports configured) | `on_unmount`: `aggregator.stop()`; also stopped/restarted in `_apply_config_reload` | Polls dashboard or per-backend `/api/health` every 2.5s. |
| **Timers** | Textual `set_interval(callback)` | `on_mount`: 5 intervals (0.5s snapshot, 0.25s logs, 2s box spread, 30s bank accounts, 3s config reload) | Implicit when app exits | No explicit cancel; Textual cleans up with the app. |

**Lifecycle:** Start order in `on_mount`: log handler → provider.start() → backend health aggregator.start() → set_interval(×5). Stop order in `on_unmount`: backend health aggregator.stop() → provider.stop(). No single “task registry”; each component manages its own thread.

**Gaps:**
- Intervals are not named or tracked; can’t cancel one without touching the app.
- If `on_unmount` is skipped (e.g. hard kill), daemon threads exit with the process but in-flight work may not finish cleanly.
- Config reload creates a new aggregator and starts it without a formal “replace previous” protocol beyond stop-then-start.

---

### 1.2 FastAPI backend services

**Locations:** `python/integration/ib_service.py`, `alpaca_service.py`, `tastytrade_service.py`, `tradestation_service.py`, `discount_bank_service.py`, `python/services/health_dashboard.py`.

| Service | Background work | Start | Stop | Notes |
|---------|------------------|--------|------|--------|
| **Health dashboard** | Long-lived asyncio task: `_nats_subscriber()` (connect, subscribe to `system.health`, `while True: sleep(60)`) | **Lifespan** `asynccontextmanager`: `task = asyncio.create_task(_nats_subscriber()); yield` | On yield exit: `task.cancel(); await task` | Correct pattern: one task, cancelled on shutdown. |
| **Tastytrade** | DXLink WebSocket: `_receive_task = asyncio.create_task(_receive_loop())`; reconnects via `asyncio.create_task(_reconnect())` | `@app.on_event("startup")`: `await _init_dxlink(...)` | `@app.on_event("shutdown")`: `await dxlink_client.disconnect()` | Startup/shutdown are async; DXLink owns its receive task. |
| **IB, Alpaca, TradeStation, Discount Bank, Analytics** | No long-lived background loops. | — | — | Health/snapshot publish: `asyncio.create_task(nats_client.publish_health(...))` (fire-and-forget). |

**Lifecycle:** Health dashboard uses **lifespan** for one background task and cancels it on exit. Tastytrade uses **on_event("startup")** and **on_event("shutdown")** for DXLink init/cleanup. Other services do not start background tasks; they only fire one-off tasks for NATS publish.

**Gaps:**
- Fire-and-forget `asyncio.create_task(publish_health(...))`: no await, no cancellation if the app is shutting down. Usually fine for a single publish, but if the task outlives the process it can log or fail after the app is gone.
- Mixed patterns: lifespan (health_dashboard) vs on_event (tastytrade). FastAPI recommends lifespan for startup/shutdown; on_event is deprecated in favor of lifespan.
- No shared “task registry”: each service that starts a task is responsible for cancelling it; easy to add a new task and forget to cancel.

---

### 1.3 Standalone / scripts

| Process | Background work | Start | Stop | Notes |
|---------|------------------|--------|------|--------|
| **collection-daemon QuestDB sink (Go)** | Single process: `go run ./cmd/collection-daemon` with `QUESTDB_ILP_ADDR` set (NATS subscribe + QuestDB ILP write) | Started by `./scripts/run_questdb_nats_writer.sh` | Process exit or SIGINT | Go-only; QuestDB fanout now lives in the collector sink pipeline. |
| **Strategy runner** (e.g. nautilus_strategy.py) | Sync `on_start()` / `on_stop()`; NATS connect via `asyncio.create_task(_connect_nats())` or `asyncio.run(_connect_nats())` depending on whether a loop is running | `runner.start()` → `on_start()` | `runner.stop()` → `on_stop()` | If run inside an event loop, create_task is used; otherwise asyncio.run. Multiple create_task calls for connect/reconnect are not tracked as a set. |
| **LEAN event_bridge** | Dedicated thread running a new asyncio loop: `Thread(target=_run_event_loop, daemon=True)`; loop runs forever | `event_bridge.start()` | `event_bridge.stop()`: set flag, `loop.call_soon_threadsafe(loop.stop)`, `thread.join(5.0)` | Clean shutdown with timeout; daemon=True so process exit kills thread if join times out. |

**Gaps:**
- collection-daemon QuestDB mode: run via script; no dedicated process-manager guidance yet beyond normal signal handling.
- Strategy runner: background asyncio tasks are not registered; on_stop() does not cancel them explicitly.

---

### 1.4 Summary table (current)

| Layer | Pattern | Cancellation / shutdown | Risk |
|-------|---------|--------------------------|------|
| TUI providers | Thread + `_running` flag + join(timeout) | on_unmount stops aggregator then provider | Good; daemon threads if join hangs |
| TUI intervals | set_interval (Textual) | App exit | Good |
| Health dashboard | lifespan + create_task + cancel | task.cancel(); await task | Good |
| Tastytrade DXLink | on_event startup/shutdown | disconnect() on shutdown | OK; prefer lifespan |
| Other backends | create_task(publish_health) fire-and-forget | None | Low (single shot) |
| collection-daemon QuestDB mode (Go) | run via script (go run) | Process exit / SIGINT | OK for script |
| LEAN event_bridge | Thread + own event loop | stop() stops loop and joins thread | Good |
| Strategy runner | create_task for NATS | on_stop does not cancel tasks | Medium |

---

## 2. Suggested improvements

### 2.1 Single coordinator per process

**Idea:** One place that knows all background work and runs shutdown in reverse order of start.

- **TUI:** Keep starting provider and aggregator in on_mount and stopping in on_unmount, but introduce a small **TaskRegistry** or **BackgroundManager** that registers (name, start_fn, stop_fn). on_mount calls start for each; on_unmount calls stop in reverse order. Intervals could be registered as (name, interval_handle) and cancelled on unmount.
- **FastAPI:** Use **lifespan** everywhere (no on_event). In lifespan, create a **list or registry of asyncio tasks**; on startup append each task; on shutdown cancel all and await with a short timeout. Any new long-lived task is created via a helper that adds it to the registry.

**Benefits:** Predictable order, no “forgot to cancel,” easier tests (inject a registry and assert all stopped).

### 2.2 Prefer asyncio for new background work (where possible)

- **Backend services:** Already async (FastAPI). New long-lived work (e.g. NATS consumer, periodic health publish) should be asyncio tasks created in lifespan and added to the task registry, not threads.
- **TUI:** Providers are sync (REST calls, file I/O) and run in threads today. Optionally move to asyncio + httpx in a worker task so the TUI has one event loop and all background work is tasks (bigger refactor). Short term: keep threads but register them in a coordinator.

### 2.3 Fire-and-forget: optional await or shield

- For `asyncio.create_task(publish_health(...))`: either await the task in the endpoint (adds latency) or keep fire-and-forget but document that shutdown does not wait for in-flight publish. If we add a “shutdown” phase in lifespan, we could await a short “drain” of pending publish tasks (e.g. with a bounded set of tasks and asyncio.gather with timeout). Lower priority.

### 2.4 Migrate Tastytrade to lifespan

- Replace `@app.on_event("startup")` and `@app.on_event("shutdown")` with a single **lifespan** context manager that calls `_init_dxlink` on enter and `dxlink_client.disconnect()` on exit. Aligns with health_dashboard and FastAPI recommendation.

### 2.5 Strategy runner: cancel NATS tasks in on_stop

- In `on_stop()`, cancel any asyncio tasks started for NATS (e.g. store `_nats_connect_task` and call `task.cancel()`; if there is an event loop, await the task with a timeout). Prevents “strategy stopped but NATS task still running.”

### 2.6 Standalone scripts: explicit SIGTERM/SIGINT handler

- For collection-daemon QuestDB mode and similar: register a signal handler that sets a “shutdown” flag and stops the main loop (or closes the NATS connection so the loop exits). Ensures clean exit when run under process managers (e.g. systemd, supervisord).

### 2.7 Document lifecycle in one place

- Add a short **“Background task lifecycle”** section to the main docs (e.g. `docs/API_DOCUMENTATION_INDEX.md` or `docs/DEVELOPER_GUIDE.md`) that points to this document and states: TUI uses on_mount/on_unmount and stops provider then aggregator; FastAPI services use lifespan and should register tasks; scripts should handle SIGTERM for clean shutdown.

---

## 3. Implementation order (suggested)

| Step | What | Effort |
|------|------|--------|
| 1 | Document current state (this file) and add a pointer from main docs | Done |
| 2 | Migrate Tastytrade from on_event to lifespan; keep DXLink init/cleanup in lifespan | Small |
| 3 | Add a simple task registry in health_dashboard lifespan (e.g. list of tasks; cancel all on exit) as reference for other services | Small |
| 4 | Strategy runner: store and cancel NATS connect task(s) in on_stop | Small |
| 5 | TUI: optional BackgroundManager that registers provider + aggregator + optional interval handles; stop in reverse order in on_unmount | Medium |
| 6 | collection-daemon QuestDB mode (and similar scripts): SIGTERM/SIGINT handler that triggers clean exit | Small |
| 7 | (Optional) Fire-and-forget: document or add a short drain in lifespan for in-flight publish tasks | Low |

---

## 4. References

- **TUI:** `python/tui/app.py` (on_mount, on_unmount, set_interval), `python/tui/providers.py` (BackendHealthAggregator, RestProvider, FileProvider, MockProvider).
- **Health dashboard:** `python/services/health_dashboard.py` (lifespan, _nats_subscriber).
- **Tastytrade:** `python/integration/tastytrade_service.py` (on_event startup/shutdown, DXLink), `python/integration/dxlink_client.py` (_receive_task, disconnect).
- **Strategy runner:** `python/integration/strategy_runner.py` (on_start, on_stop, _connect_nats).
- **QuestDB writer:** `agents/go/cmd/collection-daemon` with `QUESTDB_ILP_ADDR` set (run via `scripts/run_questdb_nats_writer.sh`).
- **LEAN event bridge:** `python/lean_integration/event_bridge.py` (start, stop, _run_event_loop).
- **FastAPI lifespan:** [FastAPI Lifespan](https://fastapi.tiangolo.com/advanced/events/) (recommended over on_event).
