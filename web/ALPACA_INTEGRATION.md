# Alpaca Integration for PWA

This guide shows how to connect the PWA to Alpaca Markets for live market data.

## Prerequisites

1. **Alpaca Account**: Sign up at [alpaca.markets](https://alpaca.markets) (free paper trading available)
2. **API Credentials**: Get your API key ID and secret key from the Alpaca dashboard, **or** use OAuth (client_id + client_secret from an OAuth app) — see [docs/ALPACA_OAUTH.md](../docs/ALPACA_OAUTH.md)
3. **Python Dependencies**: The Alpaca service requires `uvicorn` and `fastapi`
4. **Optional - 1Password CLI**: For secure credential management (recommended)

## Quick Start

### Option 1: Using 1Password (Recommended)

Store your Alpaca credentials in 1Password and reference them. Three methods are supported:

#### Method A: Using Item UUID (Simplest)

If you have the 1Password item UUID (e.g., `ldfc5jfigtmjvlg6ls4tgpgsuu`):

```bash
op signin
export OP_ALPACA_ITEM_UUID='ldfc5jfigtmjvlg6ls4tgpgsuu'
# Script will auto-detect vault and field names
./web/scripts/run-alpaca-service.sh
```

**To inspect your item and see available fields:**
```bash
./web/scripts/check-alpaca-1password-item.sh ldfc5jfigtmjvlg6ls4tgpgsuu
```

#### Method B: Using Full Paths

```bash
# Set 1Password secret references
export OP_ALPACA_API_KEY_ID_SECRET='op://Vault/Item Name/API Key ID'
export OP_ALPACA_API_SECRET_KEY_SECRET='op://Vault/Item Name/API Secret Key'

# Optional configuration
export ALPACA_PAPER=1  # Use paper trading (default)
export SYMBOLS=SPY,QQQ,IWM  # Optional: comma-separated symbols (default: SPY,QQQ)
```

#### Method C: Using UUID with Explicit Paths

```bash
export OP_ALPACA_API_KEY_ID_SECRET='op://Vault/<uuid>/API Key ID'
export OP_ALPACA_API_SECRET_KEY_SECRET='op://Vault/<uuid>/API Secret Key'
```

**1Password Setup:**
1. Create a new item in 1Password (e.g., "Alpaca API Credentials")
2. Add two fields (field names can be customized):
   - `API Key ID` - your Alpaca API key ID
   - `API Secret Key` - your Alpaca secret key
3. Get the item UUID from 1Password (right-click item → Copy UUID)

**Note**: Make sure you're signed into 1Password CLI:
```bash
op signin
```

### Option 2: Using Environment Variables

```bash
export ALPACA_API_KEY_ID=your_key_id_here
export ALPACA_API_SECRET_KEY=your_secret_key_here
export ALPACA_PAPER=1  # Use paper trading (default)
export SYMBOLS=SPY,QQQ,IWM  # Optional: comma-separated symbols (default: SPY,QQQ)
```

### 2. Start the Alpaca Service

```bash
./web/scripts/run-alpaca-service.sh
```

The service will start on `http://127.0.0.1:8000` and provide:
- `GET /api/health` - Health check endpoint
- `GET /api/snapshot` - Market data snapshot (compatible with PWA)

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

The PWA will now connect to the Alpaca service instead of the static JSON files.

## Features

- **Live Market Data**: Real-time quotes from Alpaca's market data API
- **Paper Trading**: Safe testing environment (default)
- **PWA Compatible**: Works with service worker caching and offline support
- **Multi-Symbol Support**: Configure multiple symbols via `SYMBOLS` environment variable

## Data Source Indicator

When connected to Alpaca, the header will show:
- **Account**: `ALPACA`
- **Data Source Badge**: "Data: Alpaca"

## Switching Between Data Sources

### Use Alpaca (Live Data)
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

## Troubleshooting

### Service Won't Start
- **1Password users**: Check that `op` CLI is installed and you're signed in: `op whoami`
- **1Password users**: Verify secret paths are correct: `op read 'op://Vault/Item/field'`
- **Environment variable users**: Check that credentials are set: `echo $ALPACA_API_KEY_ID`
- Install dependencies: `pip install uvicorn fastapi requests`
- Verify Python path: `cd python && python -c "from integration.alpaca_service import app"`

### PWA Not Connecting
- Check service is running: `curl http://127.0.0.1:8000/api/health`
- Verify `VITE_API_URL` is set correctly
- Check browser console for CORS errors (service includes CORS middleware)

### No Data Showing
- Verify symbols are valid: `SYMBOLS=SPY,QQQ`
- Check Alpaca API status: [status.alpaca.markets](https://status.alpaca.markets)
- Review service logs for API errors

## Security Best Practices

### 1Password Integration

Using 1Password is the recommended approach for credential management. The project supports two 1Password authentication methods:

#### Personal Account (Local Development)
- ✅ Credentials never stored in environment variables
- ✅ No credentials in shell history
- ✅ Centralized secret management
- ✅ Easy rotation and audit trails
- ✅ Works with Cursor 1Password extension
- **Setup**: `op signin` (one-time authentication)

#### Service Accounts (CI/CD & Automation)
For automated environments, CI/CD pipelines, or shared infrastructure, use [1Password Service Accounts](https://developer.1password.com/docs/service-accounts):

- ✅ Not associated with individual users
- ✅ Better for shared environments
- ✅ Works with 1Password CLI
- ✅ Included with 1Password subscription
- ⚠️ Has rate limits (see [comparison](https://developer.1password.com/docs/secrets-automation#comparison))

**Setup for Service Accounts:**
```bash
# Authenticate with service account token
export OP_SERVICE_ACCOUNT_TOKEN="your_service_account_token"
export OP_ALPACA_API_KEY_ID_SECRET='op://Vault/Item/API Key ID'
export OP_ALPACA_API_SECRET_KEY_SECRET='op://Vault/Item/API Secret Key'

./web/scripts/run-alpaca-service.sh
```

The script automatically detects and uses service account authentication when `OP_SERVICE_ACCOUNT_TOKEN` is set.

**For high-volume or self-hosted scenarios**, consider [1Password Connect Servers](https://developer.1password.com/docs/connect) which provide unlimited requests after initial caching.

### Environment Variables

If not using 1Password:
- ⚠️ Never commit credentials to version control
- ⚠️ Use `.env` files with `.gitignore` protection
- ⚠️ Rotate credentials regularly
- ⚠️ Use different credentials for dev/staging/prod

## Production Deployment

For production:
1. Set `ALPACA_PAPER=0` for live trading (use with caution!)
2. Use 1Password Service Accounts for all credential management (not personal accounts)
3. Use environment-specific configuration
4. Set up proper authentication/authorization
5. Configure rate limiting
6. Use HTTPS for all connections
7. Enable audit logging for credential access
8. Consider 1Password Connect Servers for high-volume scenarios

### CI/CD Integration

For GitHub Actions or other CI/CD pipelines, use 1Password Service Accounts:

```yaml
# .github/workflows/example.yml
env:
  OP_SERVICE_ACCOUNT_TOKEN: ${{ secrets.OP_SERVICE_ACCOUNT_TOKEN }}
  OP_ALPACA_API_KEY_ID_SECRET: 'op://Vault/Item/API Key ID'
  OP_ALPACA_API_SECRET_KEY_SECRET: 'op://Vault/Item/API Secret Key'
```

See [1Password CI/CD integrations](https://developer.1password.com/docs/ci-cd) for platform-specific setup.

## API Endpoints

### Health Check
```bash
curl http://127.0.0.1:8000/api/health
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
curl http://127.0.0.1:8000/api/snapshot
```

Response matches the PWA's `SnapshotPayload` type with:
- Real-time quotes (bid/ask/last)
- Symbol snapshots
- Account metrics
- Compatible with existing PWA components

## Next Steps

- Add options chain data via Alpaca's options API
- Implement order placement (requires Alpaca trading API)
- Add WebSocket support for real-time updates
- Integrate with box spread calculation engine
