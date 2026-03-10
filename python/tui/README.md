# Python TUI for IB Box Spread Trading

This is the active terminal client for the platform. It is the canonical TUI alongside the React web app and shares core data contracts with the web frontend.

## Features

- **Shared Data Models**: Uses the same data structures as the PWA (`web/src/types/snapshot.ts`)
- **Multiple Providers**: Supports mock, REST API, file, and NATS data sources
- **Mock as fallback**: When your preferred provider is REST/file/NATS, the TUI starts with **mock data** for a quick first paint, then switches to **real data** as soon as the backend responds. If the backend is unavailable, you keep seeing mock until it comes up.
- **Snapshot cache**: When using a REST provider, the latest snapshot is stored in a SQLite database (`~/.config/ib_box_spread/snapshot_cache.db` by default, or `SNAPSHOT_CACHE_DB`). On startup the TUI loads the last snapshot so there is something to display immediately; when the backend is down, the last cached snapshot is shown. Set `snapshot_cache_path` to `""` in TUI config or `SNAPSHOT_CACHE_DB=disable` to turn off.
- **Settings persistence**: Provider choice (mock/rest/file/nats) and endpoint settings are saved to `~/.config/ib_box_spread/tui_config.json` when you switch in Setup (F2).
- **Watchlist sync**: Symbol watchlist is shared between the dashboard and the mock provider (via `config.watchlist`), so mock data is generated for the same symbols shown in the dashboard and the "missing symbols" warning goes away when using mock.
- **Modern UI**: Built with [Textual](https://textual.textualize.io/) for a responsive terminal interface
- **Migration Ready**: Well-documented for future C++ migration via pybind11

## Screenshot

A sample screenshot of the TUI is in the repo: [screenshots/tui-screenshot.svg](screenshots/tui-screenshot.svg).

For QA/sanity, the TUI can capture a screenshot automatically (e.g. in CI or before releases):

- **Script:** `./scripts/tui_screenshot_qa.sh` or `just qa-tui-screenshot`
- **Output:** `build/qa/tui/tui-screenshot-<timestamp>.svg` (gitignored). Override with `TUI_QA_SCREENSHOT_DIR`.
- **Sanity check:** `just sanity` runs Python tests then captures a TUI screenshot.

**Display each screen without simulating interaction:**

- **Codified tab IDs:** `python.tui.app.TUI_TAB_IDS` lists all tab ids (e.g. `dashboard-tab`, `scenarios-tab`, `logs-tab`).
- **Switch tab from code:** `app.switch_to_tab(tab_id)` — no user interaction required.
- **QA screenshot by tab:** `python -m python.tui.qa_screenshot --tab scenarios-tab` captures that tab only.
- **Capture all tabs:** `python -m python.tui.qa_screenshot --all-tabs` writes `tui-<tab_id>.svg` for each tab.
- **List tab ids:** `python -m python.tui.qa_screenshot --list-tabs`

**Hidden debug metadata (SVG):**  
Every QA screenshot SVG includes a hidden comment right after the root `<svg>` tag that tools and AI can parse without changing the visible image:

- **Format:** `<!-- tui-qa: {"tab_id":"dashboard-tab","timestamp":"...","provider":"mock","textual_version":"...","python_version":"...","git_rev":"..."} -->`
- **Extract:** e.g. `grep -o 'tui-qa: {.*}' file.svg` then parse the JSON.
- **Fields:** `tab_id`, `timestamp` (UTC ISO), `provider`, `textual_version`, `python_version`, `git_rev` (if in a git repo).

**Provider health bar debugging:**  
The same metadata includes the **status bar state** at capture time so you can debug what the provider health dashbar showed:

- **`provider_label`** – e.g. `"mock"` or `"rest (127.0.0.1:8002)"`
- **`backend_health`** – full backend health dict (e.g. `{"ib": {"status": "ok", "ib_connected": true}, "alpaca": {"status": "disabled", "error": "..."}}`)
- **`status_line`** – the exact formatted string shown in the status bar (Backend: … | Provider: … | Time: … | …)

To inspect: `grep -o 'tui-qa: {.*}' file.svg | sed 's/ -->$//; s/^tui-qa: //' | python -m json.tool`

**Provider status and mock/paper/live in the TUI:**  
The status bar shows an **environment badge** and **colour strip** so you can see provider status and mode at a glance:
- **[MOCK]** (cyan bar/tint) – synthetic data from the mock provider
- **[PAPER]** (amber bar/tint) – real backend with `mode: DRY-RUN`
- **[LIVE]** (red bar/tint) – real backend with `mode: LIVE`
After the badge, the bar shows backend health (e.g. Alpaca: disabled, TWS/IBKR: ok), then Provider, Time, Mode, Strategy, Account.

**Switch mode with function keys:** **F7** = MOCK, **F8** = PAPER (TWS 7497 / Alpaca paper), **F9** = LIVE (TWS 7496 / Alpaca live). PAPER/LIVE update the shared config (e.g. `~/.config/ib_box_spread/config.json`) and switch the data source to REST; restart the IB or Alpaca service for the new mode to take effect on the backend.

**Why does the badge show [PAPER]?** The **[PAPER]** / **[LIVE]** badge is **not** a TUI setting you switch – it reflects the **mode the backend reports** in the snapshot. For IB, the Python IB service infers it from your account ID: accounts starting with **DU** are treated as paper, others as live. So if you are logged into a paper TWS/Gateway session (DU… account), the badge correctly shows PAPER. F7/F8/F9 change which backend/port the TUI uses and the shared config; the badge still comes from the snapshot. To see LIVE, use a live Gateway session (port 7496) and an account ID that does not start with DU.

**TWS data not loading?** Check: (1) **IB service** is running (e.g. `./scripts/service.sh start ib` or process on port 8002). (2) **TWS or IB Gateway** is running and logged in (port 7497 for paper or 7496 for live). (3) **Preferred provider** is rest_ib or rest_tws_gateway (Setup → Set preferred → IB/TWS). (4) If the TUI stays on [MOCK], the REST provider never got a successful snapshot from 8002 – check status bar pills for "ib" / "tws" and any error hint. (5) Snapshot cache: with cache enabled, the TUI may show the last cached snapshot when the backend is down; disable cache to avoid stale data.

**SVG vs PNG for AI/tooling:**  
Textual exports SVG only. SVG is text-based, so any AI or script can read and parse it (including the debug comment). For consumers that need a raster image (e.g. a vision model that only accepts PNG/JPEG), you can convert the SVG after capture (e.g. `cairosvg` or a headless browser). The project does not currently add a PNG export step; add one if your pipeline requires it.

## Installation

Install dependencies:

```bash
pip install textual requests
```

Or add to `requirements.txt`:

```
textual>=8.1.1
requests>=2.31.0
```

## Usage

### Basic Usage

Run the TUI with default (mock) provider:

```bash
python -m python.tui
```

### With REST API Provider

```bash
export TUI_BACKEND=rest
export TUI_API_URL=http://localhost:8080/api/snapshot
python -m python.tui
```

### Using an API router (single base URL)

When you run a reverse proxy or gateway that exposes snapshot, scenarios, bank-accounts, and health under one base URL, set `api_base_url` (or `TUI_API_BASE_URL`) so the TUI uses it for all HTTP calls:

- **Snapshot:** `{api_base_url}/api/v1/snapshot`
- **Scenarios:** `{api_base_url}/scenarios`
- **Bank accounts:** `{api_base_url}/api/bank-accounts`
- **Health:** `{api_base_url}/api/health`

Example:

```bash
export TUI_API_BASE_URL=http://localhost:9999
export TUI_BACKEND=rest
python -m python.tui
```

Or in shared config `config.json` under `tui`: `"apiBaseUrl": "http://localhost:9999"`. When `api_base_url` is set, `rest_endpoint` is ignored for the snapshot URL (the router base is used).

### With File Provider

```bash
export TUI_BACKEND=file
export TUI_SNAPSHOT_FILE=web/public/data/snapshot.json
python -m python.tui
```

### Export to Google Sheets / Excel

Press **F6** in the TUI to export the current snapshot (positions, future events) and box spread scenarios to **CSV** and **Excel** (.xlsx) files. Files are written with timestamped names to `build/export/` (or `TUI_EXPORT_DIR` if set).

- **CSV** opens in Excel and Google Sheets (File → Import, or drag-and-drop). Use for one-off analysis or importing into a Sheet.
- **Excel** (.xlsx) uses openpyxl; multiple sheets (e.g. Positions, Future events) in one file.

**Live / recurring:** To keep a Google Sheet in sync, you can (1) re-export on demand with F6 and re-import the CSV in Sheets, or (2) point Sheets at a file path if you use Google Drive desktop sync and export to a synced folder, or (3) use the Google Sheets API to push data (not included; implement via `gspread` or Sheets API and call `export_all()` on a timer).

### Configuration File

Configuration is stored in `~/.config/ib_box_spread/tui_config.json`:

- **Provider (Mock/REST/File/NATS)** and endpoint settings are **persisted** when you change them in Setup (F2). Next launch uses the saved provider.
- When a shared config is used, provider and endpoint from `tui_config.json` overlay the shared config so your last TUI choice is restored.

```json
{
  "provider_type": "rest",
  "rest_endpoint": "http://localhost:8080/api/snapshot",
  "update_interval_ms": 1000,
  "refresh_rate_ms": 500,
  "rest_timeout_ms": 5000,
  "rest_verify_ssl": false,
  "show_colors": true,
  "show_footer": true,
  "watchlist": ["SPX", "XSP", "NANOS", "TLT", "DSP"]
}
```

## Keyboard Shortcuts

- `Q` or `F10`: Quit
- `F1`: Help
- `F2`: Setup
- `F5`: Refresh
- `F6`: Export snapshot and box spread to CSV/Excel (Google Sheets / Excel)

Use Tab / Shift+Tab in the terminal to switch focus; tab content is switched via the tab bar.

## Architecture

### Async, non-blocking data flow

The TUI is designed so the **UI thread never blocks on I/O**. All data fetching and message handling run in background threads or async loops; the UI only reads from in-memory state (and optional SQLite cache on startup).

- **Display source**: The screen always shows the **latest in-memory snapshot** (and optional **snapshot cache** SQLite DB for instant first paint or when the backend is down). No network or disk I/O on the UI thread.
- **REST provider**: A **background thread** runs the poll loop; it fetches from the REST API, writes into in-memory snapshot, and optionally persists to the snapshot cache. `get_snapshot()` only copies from memory under a lock.
- **NATS provider**: An **asyncio loop in a background thread** subscribes to snapshot and health subjects; each message updates the in-memory snapshot and health state. The UI is driven by events (no polling of the network from the main thread).
- **File provider**: Same pattern: a background thread polls the file and updates the in-memory snapshot.
- **Heavy work** (e.g. bank-accounts fetch, box-spread load) runs via Textual **workers** with `thread=True`; results are applied in `on_worker_state_changed` so the UI stays responsive.

**Message-queue–driven updates**: With **NATS** enabled, backends (e.g. IB service) publish snapshots and health to NATS; the TUI **NatsProvider** subscribes and updates the displayed data on each message. For a fully event-driven setup, run the IB (or other) service with NATS publishing and point the TUI at NATS; the GUI then reflects backend state updated by the message queue without blocking.

### Data Flow

```
Provider (Mock / REST poll thread / NATS subscriber thread / File poll thread)
    -> writes to in-memory snapshot; optional SQLite cache
SnapshotPayload (shared model)
    -> get_snapshot() reads from memory only
TUI App (Textual, UI thread)
    ->
Terminal Display
```

### File layout

- **`app.py`** – Main `TUIApp`, composition, refresh intervals, provider factory. No inline tab classes.
- **`box_spread_loader.py`** – Loads box spread scenarios from REST or file (`get_box_spread_payload()`).
- **`components/`** – Tab and widget components:
  - **`base.py`** – `SnapshotTabBase` for tabs that display `SnapshotPayload`.
  - **`snapshot_display.py`** – Header line (time, mode, strategy, account).
  - **`dashboard.py`**, **`positions.py`**, **`orders.py`**, **`alerts.py`** – Snapshot-based tabs.
  - **`scenarios.py`** – Box spread scenarios (uses `update_data(box_spread_data)`).
  - **`historic.py`** – Historic positions (stub; “No historic data” until implemented).
  - **`unified_positions.py`**, **`cash_flow.py`**, **`opportunity_simulation.py`**, **`relationship_visualization.py`**, **`loan_entry.py`** – Extended tabs.

See `docs/TUI_REFACTORING_PLAN.md` for the refactoring plan.

### Shared Models

The `tui/models.py` module defines data structures that match:
- TypeScript types in `web/src/types/snapshot.ts`
- C++ types in `native/include/tui_data.h`

This ensures consistency across all frontends.

### Providers

Providers fetch data from various sources (all **non-blocking**: work runs in background threads or async loops; the UI only reads from in-memory state):

- **MockProvider**: Generates synthetic data (sync on access; fast).
- **RestProvider**: Polls REST API in a background thread; optional SQLite snapshot cache for first paint and when backend is down.
- **NatsProvider**: Subscribes to NATS `snapshot.{backend}` and `system.health` in an asyncio loop (event-driven, no polling from UI).
- **FileProvider**: Polls a JSON file in a background thread.

## Migration to C++ (Future)

This Python TUI is designed to be migrated back to C++ using pybind11. Key migration points:

### Data Models (`tui/models.py`)

- Expose dataclasses via `pybind11::class_` with properties
- Use `nlohmann/json` for JSON serialization on C++ side
- Keep Python models as source of truth for API contracts

### Providers (`tui/providers.py`)

- Provider interface can be abstract C++ class
- Python providers can call C++ implementations via pybind11
- Consider keeping Python as thin wrappers around C++ core

### UI (`tui/app.py`)

- UI rendering can stay in Python (Textual is Python-only)
- Data processing can move to C++ and expose via pybind11
- Keep Python TUI as reference implementation

### Example pybind11 Binding

```cpp
// C++ side
#include <pybind11/pybind11.h>
#include <pybind11/stl.h>

class Provider {
public:
    virtual ~Provider() = default;
    virtual void Start() = 0;
    virtual void Stop() = 0;
    virtual SnapshotPayload GetSnapshot() = 0;
};

// Python binding
PYBIND11_MODULE(tui_cpp, m) {
    py::class_<Provider>(m, "Provider")
        .def("start", &Provider::Start)
        .def("stop", &Provider::Stop)
        .def("get_snapshot", &Provider::GetSnapshot);
}
```

## Development

### Running Tests

```bash
pytest python/tui/tests/
```

### Code Style

Follow PEP 8 and use type hints. Run linters:

```bash
ruff check python/tui/
mypy python/tui/
```

## Comparison with C++ TUI

| Feature | C++ TUI | Python TUI |
|---------|---------|------------|
| Library | FTXUI | Textual |
| Performance | Faster | Fast enough |
| Maintenance | Complex | Easier |
| Shared Code | No | Yes (with PWA) |
| Migration | N/A | Ready for pybind11 |

## Notes

- The Python TUI is faster to develop and maintain
- Shared models ensure consistency with PWA
- Well-documented for future C++ migration
- Can use official Python APIs (IBKR, etc.) directly
