# Backend Services - Daemonized Startup Scripts

All backend services now have daemonized start/stop scripts for easy management.

## Available Services

| Service | Port | Start Script | Stop Script | Status |
|---------|------|--------------|-------------|--------|
| **Alpaca** | 8000 | `./scripts/start_alpaca_service.sh` | `./scripts/stop_alpaca_service.sh` | ❌ Not running |
| **IB (Interactive Brokers)** | 8002 | `./scripts/start_ib_service.sh` | `./scripts/stop_ib_service.sh` | ❌ Not running |
| **Discount Bank** | 8003 | `./scripts/start_discount_bank_service.sh` | `./scripts/stop_discount_bank_service.sh` | ✅ Running |
| **Risk-Free Rate** | 8004 | `./scripts/start_risk_free_rate_service.sh` | `./scripts/stop_risk_free_rate_service.sh` | ❌ Not running |
| **Tastytrade** | 8005 | `./scripts/start_tastytrade_service.sh` | `./scripts/stop_tastytrade_service.sh` | ❌ Not running |
| **Rust Backend** | 8080 | (Manual/Cargo) | (Manual) | ✅ Running |

## Quick Start

### Start a single service

```bash
./scripts/start_alpaca_service.sh
./scripts/start_ib_service.sh
./scripts/start_discount_bank_service.sh

# etc.
```

### Stop a single service

```bash
./scripts/stop_alpaca_service.sh
./scripts/stop_ib_service.sh
./scripts/stop_discount_bank_service.sh


# etc.
```

### Check service status

```bash

# Check if port is in use


lsof -ti :8000 && echo "✅ Alpaca running" || echo "❌ Alpaca not running"

# Check health endpoint

curl http://localhost:8000/api/health
```

### View service logs

```bash

tail -f logs/alpaca-service.log
tail -f logs/ib-service.log
tail -f logs/discount-bank-service.log

# etc.
```

## Features

All daemonized scripts include:

- ✅ **Port conflict detection** - Checks if service is already running
- ✅ **Background execution** - Runs in background with `disown`
- ✅ **Logging** - Outputs to `logs/{service-name}-service.log`
- ✅ **Health check** - Verifies service started successfully

- ✅ **Graceful shutdown** - Stop scripts kill processes cleanly

## Service Details

### Alpaca Service (Port 8000)

- **Endpoint**: `http://localhost:8000/api/snapshot`
- **Health**: `http://localhost:8000/api/health`
- **Requirements**: Alpaca API credentials (1Password or env vars)
- **Log**: `logs/alpaca-service.log`

### IB Service (Port 8002)

- **Endpoint**: `http://localhost:8002/api/snapshot`
- **Health**: `http://localhost:8002/api/health`
- **Requirements**: IB Gateway must be running (port 5000)
- **Log**: `logs/ib-service.log`

### Discount Bank Service (Port 8003)

- **Endpoint**: `http://localhost:8003/api/bank-accounts`
- **Health**: `http://localhost:8003/api/health`
- **Requirements**: DISCOUNT.dat file (default: `~/Downloads/DISCOUNT.dat`)
- **Log**: `logs/discount-bank-service.log`

### Risk-Free Rate Service (Port 8004)

- **Endpoint**: `http://localhost:8004/api/health`
- **Health**: `http://localhost:8004/api/health`
- **Purpose**: Extracts risk-free rates from box spreads
- **Log**: `logs/risk-free-rate-service.log`

### Tastytrade Service (Port 8005)

- **Endpoint**: `http://localhost:8005/api/health`
- **Health**: `http://localhost:8005/api/health`
- **Requirements**: Tastytrade API credentials
- **Log**: `logs/tastytade-service.log`

## Configuration

Ports can be configured in:

- `config/config.json` (if exists)

- `config/config.example.json` (defaults)
- Environment variables (override):
  - `ALPACA_PORT`
  - `IB_PORT`
  - `DISCOUNT_BANK_PORT`
  - `RISK_FREE_RATE_PORT`
  - `TASTYTRADE_PORT`

## Historical Notes

- TradeStation is no longer part of the active supported backend surface.
- The old TradeStation daemon scripts and proxy path were removed. Keep any remaining TradeStation docs as historical reference only.

## Troubleshooting

### Service won't start

1. Check if port is already in use: `lsof -ti :8000`
2. Check logs: `tail -f logs/{service}-service.log`
3. Verify credentials are set (for services requiring API keys)
4. Check dependencies (e.g., IB Gateway for IB service)

### Service started but not responding

1. Wait a few seconds for service to fully initialize
2. Check health endpoint: `curl http://localhost:{PORT}/api/health`
3. Review logs for errors: `tail -20 logs/{service}-service.log`

### Stop script doesn't work

1. Find process manually: `lsof -ti :{PORT}`
2. Kill manually: `kill $(lsof -ti :{PORT})`
3. Force kill if needed: `kill -9 $(lsof -ti :{PORT})`

## Notes

- All services run in background and continue after terminal closes
- Logs are automatically created in `logs/` directory
- Services check for existing instances before starting (prevents duplicates)
- Stop scripts gracefully terminate processes (SIGTERM, then SIGKILL if needed)
