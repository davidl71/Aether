# Getting real IB positions (Client Portal)

Use the **IB Client Portal API** so the backend and TUI show your live Interactive Brokers positions.

**Important:** Client Portal and TWS socket API are **exclusive** — only one can be logged in at a time. For real positions via this runbook, use Client Portal (not TWS/Gateway socket on 7497).

---

## 0. Install the IB Client Portal Gateway

### Requirements

- **Java:** JRE 8 update 192 or later. Check with `java -version`. If missing, install from [java.com](https://www.java.com/en/download/).
- **Account:** Active, funded IBKR **Pro** account (Client Portal API does not support IBKR Lite).

### Download

- **Standard:** [clientportal.gw.zip](https://download2.interactivebrokers.com/portal/clientportal.gw.zip)
- **Beta** (if standard has issues): [clientportal.beta.gw.zip](https://download2.interactivebrokers.com/portal/clientportal.beta.gw.zip)

### Install and run

1. **Unzip** the archive (e.g. to `~/Downloads/clientportal.gw` or `C:\Users\You\Downloads\clientportal.gw`).

2. **Optional – use port 5001** (this project’s default; macOS often has something on 5000):
   - Open `root/conf.yaml` inside the unzipped folder.
   - Set `listenPort: 5001` (line 4; default is 5000).

3. **Start the gateway** from a terminal (must be run from the unzipped directory):

   **macOS / Linux:**
   ```bash
   cd /path/to/clientportal.gw
   bin/run.sh root/conf.yaml
   ```

   **Windows (Command Prompt):**
   ```cmd
   cd C:\path\to\clientportal.gw
   bin\run.bat root\conf.yaml
   ```

4. **Browser:** Open **https://localhost:5001** (or 5000 if you didn’t change the port). Accept the browser’s “insecure connection” warning — the gateway uses a self-signed cert; traffic from your machine to IBKR is still encrypted.

5. **Log in** with your IBKR username and password and complete 2FA. You should see “Client login succeeds.” For **paper trading**, use your **Paper Trading username** (Account Settings → Paper Trading in the IBKR portal).

**References:** [IBKR Campus – Client Portal API](https://ibkrcampus.com/ibkr-api-page/cpapi-v1/), [WebAPI tutorial](https://www.interactivebrokers.com/campus/trading-course/ibkrs-client-portal-api/).

---

## 1. Start the gateway and log in

If you just installed it (section 0), run `bin/run.sh root/conf.yaml` (or `bin\run.bat root\conf.yaml` on Windows), then open **https://localhost:5001** in a browser and log in. Session cookies are used for API calls; re-authenticate after midnight or if the session times out.

---

## 2. Configure the backend

Set the portal base URL (no trailing slash). The backend uses `/iserver/accounts` and `/iserver/account/{account}/positions` under this base.

```bash
export IB_PORTAL_URL=https://localhost:5001/v1/portal
```

If your Gateway uses a different path (e.g. `/v1/api`), set `IB_PORTAL_URL` to that base. The code tries both `iserver` and `portfolio` endpoints.

---

## 3. Start the backend and TUI

```bash
# From repo root
cd agents/backend

# Optional: NATS for TUI snapshot stream
export NATS_URL=nats://localhost:4222

# Enable IB positions (fetcher runs when IB_PORTAL_URL is set)
export IB_PORTAL_URL=https://localhost:5001/v1/portal

# Start backend (and REST snapshot if you use the TUI)
export REST_SNAPSHOT_PORT=9090
cargo run -p backend_service
```

When `IB_PORTAL_URL` is set, the backend:

- Spawns a **background task** that fetches positions from the Client Portal every **60 seconds** and merges them into the shared `SystemSnapshot`. Positions are tagged with id prefix `ib-` (e.g. `ib-756733`).
- Subscribes to **NATS** subject `api.ib.positions` for on-demand requests. Request body optional: `{}` or `{ "account_id": "DU123456" }`. Reply: JSON array of positions or `{ "error": "..." }`.

Start the TUI in another terminal (it subscribes to `snapshot.{backend_id}` and shows positions from the snapshot):

```bash
cd agents/backend
NATS_URL=nats://localhost:4222 cargo run -p tui_service
```

---

## 4. Verify

- **Backend log:** You should see `IB Client Portal position fetcher enabled (IB_PORTAL_URL set)` and, after the first fetch, `merged N IB positions into SystemSnapshot`.
- **TUI:** Positions panel should list your IB positions (after the first 60s cycle or when you trigger a snapshot).
- **On-demand (NATS):** Use a NATS client to send a request to `api.ib.positions` with an optional reply subject; you get a JSON array of `IbPositionDto` (or an error object).

---

## 5. Troubleshooting

### No positions in TUI

If the Positions tab shows **"No data"** or an empty table:

1. **TUI not receiving snapshot (shows "No data")**
   - **NATS:** Backend and TUI must use the same `NATS_URL`. TUI must connect to NATS; check status in the TUI (e.g. "NATS: OK").
   - **Subject:** Backend publishes to `snapshot.{BACKEND_ID}`. TUI subscribes using its `BACKEND_ID`. Default is `ib` for both; if you override one, override both the same way.
   - **Backend:** Backend must have NATS connected to publish snapshots. Start backend with `NATS_URL` set.

2. **Snapshot received but 0 positions**
   - **IB_PORTAL_URL:** Backend only fetches IB positions when `IB_PORTAL_URL` is set. You should see at startup: `IB Client Portal position fetcher enabled (IB_PORTAL_URL set)`. If you see `IB positions disabled`, set `IB_PORTAL_URL` (e.g. `https://localhost:5001/v1/portal`) and restart.
   - **Client Portal running and logged in:** Gateway must be running and you must have logged in at https://localhost:5001 (or your port). If not logged in, `/iserver/accounts` returns empty and you get 0 positions. Backend logs: `IB Client Portal returned 0 accounts` or `IB positions: 0 positions (check Client Portal...)`.
   - **First fetch / interval:** Positions are fetched on startup and then every **60 seconds**. Wait at least one cycle after starting the backend. If the first fetch fails (e.g. gateway not ready), wait for the next one or restart once the gateway is up and logged in.
   - **REST snapshot only:** If you use REST snapshot instead of NATS, ensure the backend is publishing the snapshot on the port the TUI uses; positions come from the same shared snapshot.

### Other issues

| Issue | Check |
|-------|--------|
| Connection refused | Gateway not running or wrong port; confirm URL in browser. |
| 401 Unauthorized | Log in again at the Gateway URL; session may have expired. |
| Empty positions | Account may have no positions; or wrong account — try `account_id` in NATS request. |
| TWS expected instead | For **socket** API (7497/7496) use the **ib_adapter** (TWS) path; see `docs/platform/IB_ADAPTER_REVIEW.md`. Client Portal and TWS cannot be active at the same time. |

---

## 6. Optional: test script

A small Python script calls the same Client Portal endpoints as the Rust backend:

```bash
IB_PORTAL_URL=https://localhost:5001/v1/portal uv run python scripts/test_ib_positions.py
```

Ensure the Gateway is running and you are logged in before running the script or the backend.
