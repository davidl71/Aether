# TradeStation Integration for Web

TradeStation is no longer part of the active supported runtime in this repo.

This file is kept only as a historical note so old references do not imply current support.

## Current Status

- the TradeStation proxy/service path was removed
- the web app no longer expects a TradeStation backend
- Alpaca, IBKR, Tastytrade, Discount Bank, and manual imports are the current active paths

## If Revisited Later

- treat TradeStation as a new integration effort
- re-evaluate gateway, web, and service-manager wiring from scratch
- use exarp task `T-1773177438759864000` as the backlog placeholder
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
