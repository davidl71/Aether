# IB Box Spread Web Interface

React + Vite web app that mirrors the terminal UI: live status banner, multi-tab dashboard (symbols, current/historic positions, orders, alerts), box-spread scenario explorer, and quick-action controls.

## Architecture

- **Data flow**: `useSnapshot` polls `/data/snapshot.json` (or `VITE_API_URL`) via `SnapshotClient`, matching the TUI `Snapshot` schema. `useBoxSpreadData` hydrates scenario tables from `/data/box_spread_sample.json`. Both hooks bubble errors into the UI.
- **State layout**: `App` maintains active tab + detail modal state. Keyboard shortcuts (`B`, `Shift+S`) trigger the same mock combo actions as the TUI.
- **UI composition**: Header status banner, scenario summary/table, pill-based tabs, reusable data grids with sparkline overlays, timeline + alert feeds, and modal detail sheets.
- **Styling**: Dark trading desk palette via `src/styles/app.css`, responsive down to tablet width. SVG-based sparklines emulate TUI candle strips.
- **Testing**: Vitest + Testing Library cover the dashboard render path with mocked snapshot/scenario responses.
- **Tooling watch**: stay on stable Vite `7.x` for now. Vite `8.0.0-beta.0` is worth tracking for Rolldown-based builds, but it is not the default upgrade target until the stable `8.x` line lands and plugin compatibility is rechecked.
### Generating Icons

Static web assets may still use app icons. Generate them from a source image:

```bash
# Generate icons from a 512x512 PNG source image
./scripts/generate-icons.sh path/to/your-icon.png

# Or create a placeholder icon automatically
./scripts/generate-icons.sh
```

The script requires ImageMagick:
- macOS: `brew install imagemagick`
- Linux: `apt-get install imagemagick` or `yum install ImageMagick`

Icons will be generated in `public/icons/` with all required sizes (72x72 through 512x512).

## Real data in the web app

By default the web app uses static JSON under `public/data/` (e.g. `snapshot.json`, `box_spread_sample.json`). To use **live data**:

### Connect web app to IB Gateway (live snapshot)

The web app does **not** talk to the gateway directly. It talks to the project’s **IB service** (Python), which uses the Client Portal API. Do all three:

1. **IB Gateway** – running and logged in (e.g. `./ib-gateway/run-gateway.sh`, then open https://localhost:5001).
2. **IB service** – run from repo root: `./web/scripts/run-ib-service.sh` (serves `http://127.0.0.1:8002/api/snapshot`).
3. **Web env** – in `web/.env` set `VITE_API_URL=http://127.0.0.1:8002/api/snapshot`, then **restart** the dev server (`npm run dev` in `web/`) so Vite picks up the change.

If the web app still shows static/zeros, check: gateway logged in, IB service running on 8002, `.env` has `VITE_API_URL`, and the dev server was restarted after editing `.env`.

**"Fetch error" or "Cannot connect to localhost:8002" (or 127.0.0.1:8002)?** The app is trying to reach the IB REST service on port 8002 and the connection failed (service not running or not reachable). Fix:

1. **Start the IB service** from repo root: `./scripts/service.sh start ib` or `./web/scripts/run-ib-service.sh`.
2. **Confirm it's up**: `curl -s http://127.0.0.1:8002/api/health` — you should get JSON with `ib_connected` (may be `false` until the gateway is logged in). The service serves the health endpoint **immediately** on startup and connects to the gateway in the background, so the process is marked up before the gateway is ready.
3. If using the TUI with REST provider, use the same URL: `./scripts/run_python_tui.sh rest http://127.0.0.1:8002/api/v1/snapshot`.

**Gateway on 5001 but “not used”?** The gateway is only used by the **IB service** (port 8002). Nothing else talks to it. So:

1. **IB service must be running** – e.g. `./scripts/service.sh start ib` or it’s started by `./scripts/start_all_services.sh`.
2. **Check that the service sees the gateway** – `curl -s http://127.0.0.1:8002/api/health`. If `ib_connected` is `false`, the service is getting 401 from the gateway. Browser login does **not** give the API its own session; the service may need to complete re-authentication (it calls `/iserver/reauthenticate`). Restart the IB service **after** you’re logged in at https://localhost:5000 and watch `logs/ib-service.log` for “Re-authentication” or errors. If the gateway shows a re-auth or approval prompt, complete it so the service gets a session.
3. **Then** use the web app or TUI with `VITE_API_URL=http://127.0.0.1:8002/api/snapshot` or `./scripts/run_python_tui.sh rest http://127.0.0.1:8002/api/snapshot`.

### 1. Snapshot / market data

Point the app at a backend that serves the snapshot API:

- **Copy the example env file** (if you don’t have one):
  ```bash
  cp web/env.example web/.env
  ```
- **Preferred default**: use the shared Rust/nginx origin:
  - `VITE_API_URL=http://127.0.0.1:8080/api/v1/snapshot`
- **Direct service wiring** is still supported for dev/debug:
  - IB: `VITE_API_URL=http://127.0.0.1:8002/api/snapshot`
  - Alpaca: `VITE_API_URL=http://127.0.0.1:8000/api/snapshot`
  - Tastytrade: `VITE_API_URL=http://127.0.0.1:8005/api/snapshot`
- **Start that backend** (e.g. `./scripts/start_all_services.sh` or the individual run-*-service.sh scripts).
- **Restart the dev server** so Vite picks up the new env: `npm run dev` (or restart the web service).

If `VITE_API_URL` is unset, the app uses `/data/snapshot.json` (static/offline).

### 2. Box spread scenarios

Box spread data comes from the **Rust backend** at `http://localhost:8080/api/v1/scenarios`. If that fails, the app falls back to `public/data/box_spread_sample.json`.

- **Rust backend**: start the agents backend so port 8080 serves `/api/v1/scenarios` (and optionally snapshot).
- **Port override**: set `VITE_RUST_BACKEND_REST_PORT` in `web/.env` if your Rust API runs on a different port.

### 3. Other services (accounts, bank, charts)

Charts, cash flow, and opportunity simulation use the Rust backend URL. Account/bank panels can still use `VITE_*_PORT` overrides for local development, but the long-term default is path-based shared-origin routing rather than browser-to-service port coupling.

### Quick checklist for real data

| What you want      | Preferred `web/.env` setting               | Service to run                    |
|--------------------|--------------------------------------------|-----------------------------------|
| Live snapshot      | `VITE_API_URL=http://127.0.0.1:8080/api/v1/snapshot` | Rust backend / shared origin |
| Box spread grid    | `VITE_RUST_BACKEND_REST_PORT=8080` (default) | Rust backend (agents)             |
| Charts / cash flow | Same Rust backend port                     | Rust backend                      |
| Direct backend debugging | `VITE_IB_PORT`, `VITE_ALPACA_PORT`, etc. | Corresponding backend services |

## Extending Toward Production

1. Swap `/data/snapshot.json` for a REST/WS gateway (see `SnapshotClient`), emitting the same JSON produced for the TUI.
2. Replace the combo action placeholders (`handleBuyCombo`, `handleSellCombo`) with REST/Socket calls into the order manager.
3. Expand alert/order feeds with infinite scroll or server-sent events.
4. Integrate charting (candles, heatmaps) once QuestDB / ORATS payloads are exposed.
5. Add push notifications for alerts and order updates if the web client grows a browser-notification surface.

## Scripts

### Quick Start

**Launch all web services (recommended):**
```bash
./web/scripts/launch-all-pwa-services.sh
```

This script will:
- Launch web service (Vite dev server) on port 5173
- Launch IB backend service on port 8002
- Launch optional Alpaca backend service on port 8000 when enabled
- Use tmux for session management (if available)
- Fall back to background processes if tmux is not installed

**Script commands:**
```bash
./web/scripts/launch-all-pwa-services.sh start    # Start all services (default)
./web/scripts/launch-all-pwa-services.sh stop      # Stop all services
./web/scripts/launch-all-pwa-services.sh restart # Restart all services
./web/scripts/launch-all-pwa-services.sh status   # Check service status
./web/scripts/launch-all-pwa-services.sh attach   # Attach to tmux session
```

**Services launched:**
- Web service (Vite) - Port 5173
- IB Gateway - Port 5001 (requires Java)
- Alpaca service - Port 8000 (optional; disabled by default in example config)
- IB service - Port 8002
- Discount Bank service - Port 8003
- Risk-Free Rate service - Port 8004 (new)

**Individual service scripts:**

**Start the web service:**
```bash
./web/scripts/run-web-service.sh
```

This script will:
- Check for Node.js/npm
- Install dependencies if needed
- Auto-detect and connect to Alpaca service if running
- Configure `VITE_API_URL` automatically
- Start the Vite dev server

**Start the Alpaca backend service:**
```bash
./web/scripts/run-alpaca-service.sh
```

**Start the IB backend service:**
```bash
./web/scripts/run-ib-service.sh
```

### Manual Commands

- `npm run dev` – start the Vite dev server.
- `npm run build` – produce the optimized bundle in `dist/`.
- `npm run preview` – preview the build locally.
- `npm run lint` – ESLint flat config (React, TypeScript).
- `npm run test` – Vitest run mode with jsdom.

Bootstrap with:

```bash
bash agents/web/scripts/setup.sh   # npm install
./scripts/generate-icons.sh         # Generate app icons
bash agents/web/scripts/run-tests.sh
npm run dev
```

Set `VITE_API_URL=http://127.0.0.1:8000/api/snapshot` (or your backend URL) to hit the live backend service (Alpaca or IB). The default static JSON under `public/data/` keeps the SPA functional even without a live backend.

**See [ALPACA_INTEGRATION.md](./ALPACA_INTEGRATION.md) for Alpaca setup instructions.**
**See [IB_INTEGRATION.md](./IB_INTEGRATION.md) for Interactive Brokers setup instructions.**

## Feature Parity with TUI

This web app is one of the two active frontends for the platform, alongside the Python/Textual TUI. See [Feature Tracking](../docs/FEATURE_TRACKING.md) for the current capability matrix between the web app and the active terminal client.
