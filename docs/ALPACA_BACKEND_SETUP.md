## Alpaca Backend Setup (Broker-Agnostic Feed for TUI and PWA)

This backend serves a unified `SnapshotPayload` for both the TUI and the PWA, powered by Alpaca Market Data v2 and Trading API.

### Prerequisites
- Python 3.10+
- Packages: `fastapi`, `uvicorn`, `requests`
  - Install: `pip install fastapi uvicorn requests`
- Alpaca API keys (paper recommended for testing)

### Quick Start

**Using the startup script (recommended):**
```bash
export ALPACA_API_KEY_ID=your_key_id
export ALPACA_API_SECRET_KEY=your_secret_key
export ALPACA_PAPER=1
export SYMBOLS=SPY,QQQ
# Optional: enable file polling for TUI
export SNAPSHOT_FILE_PATH=$(pwd)/web/public/data/snapshot.json

./scripts/start_alpaca_service.sh
```

**Manual start:**
```bash
export ALPACA_API_KEY_ID=xxx
export ALPACA_API_SECRET_KEY=yyy
export ALPACA_PAPER=1
export SYMBOLS=SPY,QQQ
export SNAPSHOT_FILE_PATH=$(pwd)/web/public/data/snapshot.json

uvicorn python.integration.alpaca_service:app --host 127.0.0.1 --port 8000
```

### Environment Variables
- `ALPACA_API_KEY_ID`: Your Alpaca API key (required)
- `ALPACA_API_SECRET_KEY`: Your Alpaca API secret (required)
- `ALPACA_PAPER`: Set to `1` (default) for paper trading endpoints
- `ALPACA_BASE_URL` (optional): Override trading base URL
- `ALPACA_DATA_BASE_URL` (optional): Override data base URL
- `SYMBOLS`: Comma-separated list of symbols (default: `SPY,QQQ`)
- `SNAPSHOT_FILE_PATH` (optional): If set, the service writes the latest snapshot JSON to this path, enabling the TUI to poll from disk
- `PORT`: Server port (default: `8000`)
- `HOST`: Server host (default: `127.0.0.1`)

### API Endpoints

#### Health Check
```bash
curl -s http://127.0.0.1:8000/api/health | jq
```
Returns service status and Alpaca connection status.

#### Snapshot (Complete Data)
```bash
curl -s http://127.0.0.1:8000/api/snapshot | jq
```
Returns complete snapshot with:
- Market data for configured symbols
- Account metrics (equity, buying power, etc.)
- Open positions
- Open orders
- Account ID and trading mode

#### Account Information
```bash
curl -s http://127.0.0.1:8000/api/account | jq
```
Returns full account details from Alpaca.

#### Positions
```bash
curl -s http://127.0.0.1:8000/api/positions | jq
```
Returns all open positions.

#### Orders
```bash
# All orders
curl -s http://127.0.0.1:8000/api/orders | jq

# Open orders only
curl -s "http://127.0.0.1:8000/api/orders?status=open" | jq

# Closed orders
curl -s "http://127.0.0.1:8000/api/orders?status=closed&limit=100" | jq
```

### TUI Integration

#### Option 1: File Polling (Recommended)
The TUI can use the `FileProvider` to read snapshot JSON files:

```bash
# Start service with file output
export SNAPSHOT_FILE_PATH=$(pwd)/web/public/data/snapshot.json
./scripts/start_alpaca_service.sh

# In another terminal, run TUI with file provider
export TUI_SNAPSHOT_FILE=$(pwd)/web/public/data/snapshot.json
./build/ib_box_spread --tui
```

The `FileProvider` automatically polls the file and updates when it changes.

#### Option 2: HTTP Polling (Implemented)
The TUI `RestProvider` now supports HTTP polling using libcurl:

```bash
# Start service
./scripts/start_alpaca_service.sh

# In another terminal, run TUI with REST endpoint
export TUI_API_URL=http://127.0.0.1:8000/api/snapshot
./build/ib_box_spread --tui --backend rest
```

Or configure it in the TUI config file. The `RestProvider` will automatically poll the endpoint at the configured interval (default: 1 second).

### PWA Integration

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

### Features

- **Real Account Data**: Fetches actual account equity, buying power, positions, and orders
- **Market Data**: Real-time quotes and trades via Alpaca Market Data v2
- **Error Handling**: Graceful error handling with informative error messages
- **File Polling**: Optional file-based snapshot for TUI integration
- **CORS Enabled**: Ready for web/PWA integration
- **Health Monitoring**: Health endpoint verifies Alpaca connectivity

### Troubleshooting

**Service won't start:**
- Verify `ALPACA_API_KEY_ID` and `ALPACA_API_SECRET_KEY` are set
- Check Python dependencies: `pip install fastapi uvicorn requests`

**No data in snapshot:**
- Verify Alpaca credentials are correct
- Check account has market data permissions
- Ensure symbols are valid (e.g., `SPY`, `QQQ`, not `SPX` which requires options data subscription)

**TUI not updating:**
- Verify `SNAPSHOT_FILE_PATH` is set and writable
- Check file permissions
- Ensure TUI is configured to use `FileProvider` with the correct path
