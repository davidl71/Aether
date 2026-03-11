# Dynamic Reload for TUI and PWA

Research and options for hot reload / live refresh during development so code or config changes apply without a full manual restart.

---

## Current State

| Surface | What reloads today | Gap |
|--------|---------------------|-----|
| **PWA (Vite)** | React components, CSS, TS: **HMR** (Vite default). Env vars (`VITE_*`) require restart or full refresh. | Env/config change = manual refresh or restart dev server. |
| **TUI (Textual)** | Config and code: **read on startup only**. No file watcher. | T-114 decision: "Watch for changes (better UX, allows hot-reload)" — not implemented. |
| **Backend Python** | uvicorn **`--reload`** in run scripts (IB, Alpaca, health dashboard, etc.). | Already good for Python backend iteration. |
| **C++ binary** | None. | `scripts/dev_watch_binary.sh` runs a command when build output changes (e.g. restart a service). |

---

## TUI (Textual)

### 1. Textual dev mode (`textual run --dev`)

- **What it does:** Live editing of **CSS** only. Saving a `.tss` or CSS file updates the running app without restart.
- **What it does not do:** No Python or config file reload; no full app restart.
- **How to use:** Run the app via Textual’s runner instead of `python -m python.tui`:

  ```bash
  textual run python/tui/app.py --dev
  ```

  Or add a script/Just recipe that runs this from project root so `python/tui/app.py` resolves.

- **Recommendation:** Use `textual run ... --dev` in a dev run script so CSS iteration is fast; keep `python -m python.tui` for production/CI.

### 2. Config file watch (T-114)

- **Goal:** When `tui_config.json` or shared config (`config/config.json` / `IB_BOX_SPREAD_CONFIG`) changes, apply without restart.
- **Options:**
  - **A. In-process watcher:** Use `watchfiles` or `watchdog` in the TUI process; on config file change, call `load_config()` and apply (e.g. update `rest_endpoint`, restart `BackendHealthAggregator` with new `health_dashboard_url`, or reconnect provider). Preserves single process; requires careful handling of provider/aggregator lifecycle.
  - **B. External watcher:** A small script (e.g. `scripts/dev_watch_tui.sh`) that watches `~/.config/ib_box_spread/tui_config.json` and `config/config.json`; on change runs `pkill -f "python.*python.tui"` and re-executes `python -m python.tui`. Simpler but restarts the whole app (state lost).
- **Recommendation:** Implement A for config-only reload (rest_endpoint, health_dashboard_url, backend_ports); document B as a simple “full restart on any change” option.

### 3. Python source reload (full TUI restart)

- **Goal:** When `.py` files under `python/tui/` change, restart the TUI.
- **Options:**
  - **External watcher:** e.g. `fswatch python/tui -- ./scripts/run_python_tui.sh` or a dedicated `scripts/dev_watch_tui.sh` that restarts `python -m python.tui` on `.py` change. Same as B above but watching source.
  - **reload/watch in run script:** `run_python_tui.sh` could accept a `--watch` flag that runs the TUI in a loop and restarts it when `watchfiles` or `fswatch` detects changes (optional dependency).
- **Recommendation:** Reuse the same dev watcher script for both config and Python; one process that restarts the TUI on any watched file change.

---

## PWA (Vite + React)

### 1. HMR (already on)

- Vite’s HMR updates React components, CSS, and TS without full page reload. No change needed.

### 2. Env / config reload

- **Issue:** `import.meta.env.VITE_*` is inlined at build time. Changing `.env` or env vars does not update the running app until the dev server is restarted or the page is hard-refreshed.
- **Options:**
  - **A. Restart dev server on env change:** A watcher (e.g. `fswatch web/.env`) runs `npm run dev` in a loop (kill + start). Simple but full restart.
  - **B. Runtime config endpoint:** PWA fetches config from a small endpoint (e.g. `/api/config` or `/config.json`) and uses that for API URLs, feature flags, etc. Changing server-side config and refreshing the endpoint (or polling) updates the app without restarting Vite. Requires backend or static JSON and PWA code to use it.
  - **C. Document current behavior:** Env changes require “restart dev server or hard refresh” and keep iteration on code/CSS via HMR.
- **Recommendation:** Short term: C. If env-driven config becomes frequent, add B (runtime config) and optionally A for `.env`-driven restarts.

### 3. Service worker in dev

- `vite.config.ts` has `devOptions.enabled: false` for the PWA plugin so the service worker doesn’t cache in dev. Good for iteration; no change needed.

---

## Implemented / Suggested Scripts

| Script | Purpose |
|--------|--------|
| `scripts/dev_watch_binary.sh` | Watch `build/` (or binary path); on change run a command (e.g. restart IB service). For C++ binary iteration. |
| **Suggested:** `scripts/dev_watch_tui.sh` | Watch `python/tui/**/*.py` and `~/.config/ib_box_spread/tui_config.json` (and optional `config/config.json`); on change restart `python -m python.tui`. Optional: `--dev` to run `textual run python/tui/app.py --dev` for CSS live edit. |
| **Optional:** `scripts/dev_watch_pwa.sh` | Watch `web/.env`; on change kill and restart `npm run dev`. Low priority if env changes are rare. |

---

## Summary

- **TUI:** Use `textual run python/tui/app.py --dev` for CSS hot reload; add a dev watcher (config + optional Python) to restart the TUI on file change; later, add in-process config reload (T-114) to avoid restart for config-only edits.
- **PWA:** Rely on Vite HMR for code/CSS; treat env as “restart or hard refresh”; consider a runtime config endpoint if env-driven config grows.
- **Backends:** Already covered by uvicorn `--reload` and `dev_watch_binary.sh` for the C++ binary.

---

## References

- Textual dev mode: [Textual devtools – Live editing](https://github.com/textualize/textual/blob/main/docs/guide/devtools.md) (`textual run --dev`).
- T-114: `docs/TASK_CLARIFICATION_RESOLUTION.md` — “Watch for changes (better UX, allows hot-reload)”.
- Binary watch: `scripts/dev_watch_binary.sh`, `docs/HEALTH_DASHBOARD.md` (§ Dev iteration).
