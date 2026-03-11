# Interactive Brokers Integration for PWA

This guide shows how to connect the PWA to Interactive Brokers (IB) for live market data and account information.

## Prerequisites

1. **IB Account**: Active Interactive Brokers account
2. **IB Client Portal Gateway**: Download and install the gateway
   - **Direct Download**: [clientportal.gw.zip](https://download2.interactivebrokers.com/portal/clientportal.gw.zip)
   - **Download Page**: [Interactive Brokers Gateway](https://www.interactivebrokers.com/en/index.php?f=16457)
3. **Python Dependencies**: The IB service requires `uvicorn`, `fastapi`, and `requests`
4. **Market Data Subscriptions**: May be required for real-time quotes (check your IB account subscriptions)

## Quick Start

### 1. Start IB Client Portal Gateway

#### Option A: Automated Installation (Recommended)

```bash
./web/scripts/install-ib-gateway.sh
```

This script will:
- Download the gateway from IB
- Extract it to `ib-gateway/` directory
- Create a convenience run script
- Verify the installation

Then start the gateway:
```bash
./ib-gateway/run-gateway.sh
```

#### Option B: Manual Installation

1. **Download**:
   - **Direct Download**: [clientportal.gw.zip](https://download2.interactivebrokers.com/portal/clientportal.gw.zip)
   - **Alternative**: [IB Gateway Download Page](https://www.interactivebrokers.com/en/index.php?f=16457)
2. **Extract**: Unzip the downloaded file
3. **Run**: Start the gateway application (usually `bin/run.sh` from the Gateway package)
4. **Login**: Browser will open automatically for authentication
5. **Verify**: Gateway should be running on `https://localhost:5001` by default

**📚 Official Documentation:**

- **Setup Guide**: [IBKR Campus - Launching and Authenticating the Gateway](https://www.interactivebrokers.com/campus/trading-lessons/launching-and-authenticating-the-gateway/)
- **API Documentation**: [Client Portal Web API](https://interactivebrokers.github.io/cpwebapi/)
- **IB API Overview**: [Interactive Brokers API](https://www.interactivebrokers.com/en/trading/ib-api.php)
- **API Resources**: [IB API Resources](https://www.interactivebrokers.com/en/index.php?f=46915)

### 2. Start the IB Service

```bash
./web/scripts/run-ib-service.sh
```

The service will:
- Check for Python dependencies and install if needed
- Optionally verify IB Client Portal Gateway is running
- Start the FastAPI service on `http://127.0.0.1:8000`
- Provide endpoints compatible with the PWA

### 3. Configure the PWA

Create a `.env` file in the `web/` directory (or set environment variable):

```bash
cd web
echo "VITE_API_URL=http://127.0.0.1:8000/api/snapshot" > .env
```

### 4. Start the PWA

```bash
cd web
npm run dev
```

The PWA will now connect to the IB service instead of static JSON files.

## Features

- **Live Market Data**: Real-time quotes from IB Client Portal API
- **Account Information**: Account summary, positions, and buying power
- **Multiple Accounts**: Support for multiple IB accounts
- **PWA Compatible**: Works with service worker caching and offline support
- **Multi-Symbol Support**: Configure multiple symbols via `SYMBOLS` environment variable

## Data Source Indicator

When connected to IB, the header will show:
- **Account**: Your IB account ID (e.g., "DU123456")
- **Data Source Badge**: "Data: IBKR"

## Environment Variables

### IB Service

```bash
# Required: None (uses local IB Client Portal Gateway)

# Optional:
SYMBOLS=SPY,QQQ,IWM  # Comma-separated symbols (default: SPY,QQQ)
IB_PORTAL_URL=https://localhost:5001/v1/portal  # IB Client Portal URL (default)
SNAPSHOT_FILE_PATH=/path/to/snapshot.json  # Optional file output for TUI
SNAPSHOT_CACHE_SECONDS=3  # Seconds to cache snapshot response (0=disable). Use 5 to reduce Gateway load further.
REAUTH_SLEEP_SECONDS=0.5  # Seconds to sleep after Gateway reauth (portal client; default 0.5, clamp 0.1–2.0).
```

### PWA

```bash
VITE_API_URL=http://127.0.0.1:8000/api/snapshot  # IB service endpoint
```

## API Endpoints

### Health Check

```bash
curl http://127.0.0.1:8000/api/health
```

Response:
```json
{
  "status": "ok",
  "ts": "2025-01-27T12:00:00+00:00",
  "ib_connected": true,
  "accounts": ["DU123456"]
}
```

### Snapshot

```bash
curl http://127.0.0.1:8000/api/snapshot
```

Response matches the PWA's `SnapshotPayload` type with:
- Real-time quotes (bid/ask/last) from IB
- Account metrics (net liquidation, buying power, etc.)
- Positions from IB account
- Compatible with existing PWA components

### List Accounts

```bash
curl http://127.0.0.1:8000/api/accounts
```

### Set Active Account

```bash
curl -X POST http://127.0.0.1:8000/api/account \
  -H "Content-Type: application/json" \
  -d '{"account_id": "DU123456"}'
```

## Switching Between Data Sources

### Use IB (Live Data)

```bash
export VITE_API_URL=http://127.0.0.1:8000/api/snapshot
npm run dev
```

### Use Alpaca (Live Data)

This path is retired. See
[ALPACA_TASTYTRADE_RUNTIME_RETIREMENT.md](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/archive/ALPACA_TASTYTRADE_RUNTIME_RETIREMENT.md).

### Use Static JSON (Offline/Development)

```bash
unset VITE_API_URL
# or remove from .env file
npm run dev
```

## Troubleshooting

### Service Won't Start

- **Python not found**: Install Python 3 from [python.org](https://www.python.org/)
- **Dependencies missing**: Script will auto-install `uvicorn`, `fastapi`, `requests`
- **Port 8000 in use**: Stop other service or use different port

### IB Gateway Not Detected

- **Gateway not running**: Start IB Client Portal Gateway application
- **Wrong port**: Default is `https://localhost:5001`, check gateway settings
- **SSL errors**: Gateway uses self-signed certificates (normal, ignore SSL warnings)

### PWA Not Connecting

- **Service not running**: Check `http://127.0.0.1:8000/api/health`
- **Wrong URL**: Verify `VITE_API_URL` is set correctly
- **CORS errors**: Service includes CORS middleware (should work)
- **Browser console**: Check for network errors

### No Market Data

- **Market data subscriptions**: Verify your IB account has market data subscriptions
- **Market hours**: Some data may not be available outside trading hours
- **Symbol format**: Use standard symbols (e.g., "SPY", not "SPY.US")
- **Gateway connection**: Ensure gateway session is active (check browser)

### Authentication Issues

- **Gateway session expired**: Re-authenticate via gateway browser interface
- **Account access**: Verify account ID is correct and accessible
- **Gateway logs**: Check gateway logs for authentication errors

## Architecture

The IB integration follows the same pattern as the Alpaca integration:

1. **IB Client Portal Gateway**: Runs locally, handles authentication and IB server communication
2. **IB Service** (`python/integration/ib_service.py`): FastAPI service that:
   - Connects to IB Client Portal Gateway
   - Fetches market data, account info, and positions
   - Exposes `/api/snapshot` endpoint matching API contract
3. **PWA**: Connects to IB service via `VITE_API_URL`

## Differences from Alpaca

- **Authentication**: Browser-based (via IB Client Portal Gateway), not API keys
- **Gateway Required**: Must run IB Client Portal Gateway separately
- **No Paper Trading**: IB Client Portal always uses live trading accounts
- **Market Data**: May require market data subscriptions
- **Session Management**: Gateway handles authentication, service maintains session

## Security Best Practices

- **Local Gateway**: IB Client Portal Gateway runs locally (not exposed to internet)
- **HTTPS**: Gateway uses HTTPS with self-signed certificates (normal for localhost)
- **No Credentials in Code**: Authentication handled by gateway, not stored in service
- **Port Binding**: Service binds to `127.0.0.1` only (not exposed to network)

## Production Deployment

For production:
1. Use reverse proxy (nginx/traefik) for HTTPS termination
2. Implement authentication for service endpoints
3. Configure rate limiting
4. Use environment-specific configuration
5. Enable audit logging
6. Monitor gateway connection health

## Next Steps

- Add options chain data via IB API
- Implement order placement (requires TWS API or additional IB endpoints)
- Add WebSocket support for real-time updates
- Integrate with box spread calculation engine
- Add historical data support
