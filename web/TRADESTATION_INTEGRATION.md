# TradeStation Integration for PWA

This guide shows how to connect the PWA to TradeStation API for live market data.

## Prerequisites

1. **TradeStation Account**: Sign up at [tradestation.com](https://www.tradestation.com) (SIM environment available for testing)
2. **API Credentials**: Request API access by emailing ClientExperience@tradestation.com with your funded account
3. **Python Dependencies**: The TradeStation service requires `uvicorn` and `fastapi`

## Quick Start

### 1. Set Environment Variables

```bash
export TRADESTATION_CLIENT_ID=your_client_id_here
export TRADESTATION_CLIENT_SECRET=your_client_secret_here
export TRADESTATION_SIM=1  # Use SIM environment (default)
export SYMBOLS=SPY,QQQ,IWM  # Optional: comma-separated symbols (default: SPY,QQQ)
```

### 2. Start the TradeStation Service

```bash
./web/scripts/run-tradestation-service.sh
```

The service will start on `http://127.0.0.1:8001` and provide:
- `GET /api/health` - Health check endpoint
- `GET /api/snapshot` - Market data snapshot (compatible with PWA)

### 3. Configure the PWA

Create a `.env` file in the `web/` directory (or set environment variable):

```bash
cd web
echo "VITE_API_URL=http://127.0.0.1:8001/api/snapshot" > .env
```

### 4. Start the PWA

```bash
cd web
npm run dev
```

The PWA will now connect to the TradeStation service instead of the static JSON files.

## Features

- **Live Market Data**: Real-time quotes from TradeStation's market data API v3
- **SIM Environment**: Safe testing environment (default)
- **PWA Compatible**: Works with service worker caching and offline support
- **Multi-Symbol Support**: Configure multiple symbols via `SYMBOLS` environment variable
- **OAuth 2.0**: Secure authentication using client credentials flow

## Data Source Indicator

When connected to TradeStation, the header will show:
- **Account**: `TRADESTATION`
- **Mode**: `SIM` (for paper trading) or `LIVE` (for production)
- **Data Source Badge**: "Data: TradeStation"

## Switching Between Data Sources

### Use TradeStation (Live Data)
```bash
export VITE_API_URL=http://127.0.0.1:8001/api/snapshot
npm run dev
```

### Use Alpaca
```bash
export VITE_API_URL=http://127.0.0.1:8000/api/snapshot
npm run dev
```

### Use Static JSON (Offline/Development)
```bash
unset VITE_API_URL
# or remove from .env file
npm run dev
```

## TradeStation API Details

### API Versions

TradeStation supports two API versions:
- **v3**: Recommended for new developments (default)
- **v2**: Available for legacy integrations

The service uses v3 by default. To use v2, set:
```bash
export TRADESTATION_BASE_URL=https://api.tradestation.com/v2
```

### Authentication

TradeStation uses OAuth 2.0 client credentials flow:
1. Client ID and Client Secret are used to obtain an access token
2. Access tokens are automatically refreshed when expired
3. Tokens are cached to minimize authentication requests

### Environments

- **SIM**: `https://sim-api.tradestation.com/v3` (paper trading, default)
- **LIVE**: `https://api.tradestation.com/v3` (production)

Set `TRADESTATION_SIM=0` to use the live environment.

## Troubleshooting

### Service Won't Start
- Check that TradeStation credentials are set: `echo $TRADESTATION_CLIENT_ID`
- Install dependencies: `pip install uvicorn fastapi requests`
- Verify Python path: `cd python && python -c "from integration.tradestation_service import app"`
- Check OAuth token endpoint - TradeStation may use different endpoints. See `tradestation_client.py` for details.

### PWA Not Connecting
- Check service is running: `curl http://127.0.0.1:8001/api/health`
- Verify `VITE_API_URL` is set correctly
- Check browser console for CORS errors (service includes CORS middleware)

### Authentication Errors
- Verify credentials are correct
- Check that your account has API access enabled
- Ensure you're using the correct environment (SIM vs LIVE)
- Review TradeStation API documentation for OAuth endpoint changes

### No Data Showing
- Verify symbols are valid: `SYMBOLS=SPY,QQQ`
- Check TradeStation API status
- Review service logs for API errors
- Verify OAuth token is being obtained successfully

## Production Deployment

For production:
1. Set `TRADESTATION_SIM=0` for live trading (use with caution!)
2. Use environment-specific configuration
3. Set up proper authentication/authorization
4. Configure rate limiting (TradeStation has rate limits)
5. Use HTTPS for all connections
6. Store credentials securely (use 1Password or similar)

## API Endpoints

### Health Check
```bash
curl http://127.0.0.1:8001/api/health
```

Response:
```json
{
  "status": "ok",
  "ts": "2025-01-27T12:00:00+00:00"
}
```

### Snapshot
```bash
curl http://127.0.0.1:8001/api/snapshot
```

Response matches the PWA's `SnapshotPayload` type with:
- Real-time quotes (bid/ask/last)
- Symbol snapshots
- Account metrics
- Compatible with existing PWA components

## TradeStation API Resources

- **Documentation**: [https://api.tradestation.com/docs/](https://api.tradestation.com/docs/)
- **API Specifications**: [https://api.tradestation.com/docs/specifications](https://api.tradestation.com/docs/specifications)
- **Support**: ClientExperience@tradestation.com

## Next Steps

- Add options chain data via TradeStation's options API
- Implement order placement (requires TradeStation trading API)
- Add WebSocket support for real-time updates
- Integrate with box spread calculation engine
- Add account balance and position data

## Differences from Alpaca Integration

1. **Authentication**: TradeStation uses OAuth 2.0 (client credentials) vs Alpaca's API key/secret
2. **Port**: TradeStation service runs on port 8001 (Alpaca uses 8000)
3. **Environment**: TradeStation uses SIM/LIVE vs Alpaca's paper/live distinction
4. **API Structure**: TradeStation v3 may have different response formats - adjust `tradestation_client.py` as needed
