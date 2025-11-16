## Alpaca Backend Setup (Broker-Agnostic Feed for TUI and PWA)

This backend serves a unified `SnapshotPayload` for both the TUI and the PWA, powered by Alpaca Market Data v2.

### Prerequisites
- Python 3.10+
- Packages: `fastapi`, `uvicorn`, `requests`
  - Install: `pip install fastapi uvicorn requests`
- Alpaca API keys (paper recommended)

### Environment Variables
- `ALPACA_API_KEY_ID`: Your Alpaca API key
- `ALPACA_API_SECRET_KEY`: Your Alpaca API secret
- `ALPACA_PAPER`: Set to `1` (default) for paper trading endpoints
- `ALPACA_BASE_URL` (optional): Override trading base URL
- `ALPACA_DATA_BASE_URL` (optional): Override data base URL
- `SYMBOLS`: Comma-separated list of symbols (default: `SPY,QQQ`)
- `SNAPSHOT_FILE_PATH` (optional): If set, the service writes the latest snapshot JSON to this path, enabling the TUI to poll from disk

### Run the Backend
```bash
export ALPACA_API_KEY_ID=xxx
export ALPACA_API_SECRET_KEY=yyy
export ALPACA_PAPER=1
export SYMBOLS=SPY,QQQ
# Optional: write a snapshot file so the TUI can poll from disk
export SNAPSHOT_FILE_PATH=$(pwd)/web/public/data/snapshot.json

uvicorn python.integration.alpaca_service:app --host 127.0.0.1 --port 8000
```

Health check:
```bash
curl -s http://127.0.0.1:8000/api/health | jq
```

Snapshot:
```bash
curl -s http://127.0.0.1:8000/api/snapshot | jq
```

### Wire the PWA
Set an environment variable for the web app:
```bash
# In a shell where you run `npm run dev` or build:
export VITE_API_URL=http://127.0.0.1:8000/api/snapshot
```
Then:
```bash
cd web
npm run dev
```
The PWA will poll the backend for live data.

### TUI Integration (File-Polling Mode)
If `SNAPSHOT_FILE_PATH` is configured, the backend writes `SnapshotPayload` JSON on each request. Configure the TUI to read this file as a fallback source (HTTP-free). This avoids introducing new C++ HTTP dependencies and provides a broker-agnostic data path.

Recommended next step: add a small file-polling reader in the TUI loop to parse the JSON (using existing `nlohmann::json`) and hydrate the same structures used by the strategy when a broker adapter isn’t active.
