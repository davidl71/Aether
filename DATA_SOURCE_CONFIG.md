# Data Source Configuration Guide

## Overview

All components (CLI, TUI, PWA) now default to **real IBKR paper trading data** instead of mock/simulated data.

## Default Configuration (After Patch)

| Component | Data Source | Port | Mode |
|-----------|-------------|------|------|
| **CLI** | IBKR TWS | 7497 | Paper Trading (real data) |
| **TUI** | REST API → IB Service | 8002 | Real data via HTTP |
| **PWA** | IB Service | 8002 | Real data via HTTP |

## Prerequisites

To use real IBKR data, you must have **TWS (Trader Workstation)** or **IB Gateway** running:

1. **Download**: Get TWS from [Interactive Brokers](https://www.interactivebrokers.com/en/trading/tws.php)
2. **Install**: Follow the installation wizard
3. **Configure**:
   - Enable API connections: `Global Configuration → API → Settings`
   - Enable "Enable ActiveX and Socket Clients"
   - Set "Socket port" to `7497` (paper trading)
   - Disable "Read-Only API" if you want order placement (keep enabled for safety)
4. **Start**: Launch TWS and login with paper trading account

## Configuration Details

### CLI (Native Binary)

**File**: `config/config.example.json` → copy to `config/config.json`

```json
{
  "tws": {
    "host": "127.0.0.1",
    "port": 7497,  // Paper trading (real data, simulated execution)
    "use_mock": false
  },
  "dry_run": true  // Simulate orders, don't execute
}
```

**To switch to mock data** (no TWS needed):

```bash
./build/bin/ib_box_spread --mock-tws
# or
export TWS_MOCK=1
```

---

### TUI (Rust Ratatui)

**Entry point**: `./scripts/run_rust_tui.sh`

**Default**: reads shared config and connects through the Rust backend/NATS path

**To switch data sources**:

```bash
# Override REST fallback endpoint
export REST_URL=http://127.0.0.1:8002/api/snapshot

# Override NATS server
export NATS_URL=nats://127.0.0.1:4222
```

---

### PWA (Web App)

**File**: `web/src/config/sharedConfig.ts`

**Default**: `primary: 'ib'` with fallback to `'alpaca'`

**To switch data sources**:

```bash
# Use Alpaca instead
export VITE_DATA_SOURCE_PRIMARY=alpaca

# Use mock data
export VITE_DATA_SOURCE_PRIMARY=mock
```

---

## Port Reference

### TWS/IB Gateway Ports

- **7497**: TWS Paper Trading ✅ **DEFAULT** (real market data, simulated execution)
- **7496**: TWS Live Trading ⚠️ **REAL MONEY**
- **4002**: IB Gateway Paper Trading
- **4001**: IB Gateway Live Trading ⚠️ **REAL MONEY**

### Service Ports

- **5173**: Web frontend (Vite dev server)
- **8000**: Alpaca service
- **8002**: IB service ✅ **DEFAULT** (for TUI/PWA to connect)
- **8080**: Rust backend REST API
- **4222**: NATS messaging

---

## Safety Features

### Built-in Safety

1. ✅ **Paper Trading by Default**: Port 7497 uses paper account (no real money)
2. ✅ **Dry-Run Mode**: CLI defaults to `dry_run: true` (simulates orders)
3. ✅ **Fallback Sources**: PWA falls back to Alpaca if IB unavailable

### How to Revert to Mock Data

If you want to run without TWS/IB Gateway:

**CLI**:

```bash
export TWS_MOCK=1
./build/bin/ib_box_spread
```

**TUI**:

```bash
REST_FALLBACK=1 ./scripts/run_rust_tui.sh
```

**PWA**:
Edit `web/src/config/sharedConfig.ts`:

```typescript
primary: 'mock',  // Change from 'ib' to 'mock'
```

---

## Troubleshooting

### "Connection refused" errors

**Cause**: TWS/IB Gateway not running or API not enabled

**Solution**:

1. Start TWS/IB Gateway
2. Go to `Global Configuration → API → Settings`
3. Enable "Enable ActiveX and Socket Clients"
4. Check port is `7497` for paper trading
5. Restart TWS

### "No data returned" errors

**Cause**: Rust backend snapshot service (port 8002/8080 depending on setup) not running

**Solution**:

```bash
# Start backend services
./scripts/start_all_services.sh
```

### "Permission denied" errors

**Cause**: TWS API is in Read-Only mode or client ID conflict

**Solution**:

1. TWS: `Global Configuration → API → Settings → Disable "Read-Only API"`
2. Change `client_id` in config if multiple connections

---

## Advanced: Using Multiple Data Sources

You can configure fallback sources in `config.json`:

```json
{
  "broker": {
    "primary": "IB",
    "priorities": ["ib", "alpaca", "mock"]
  }
}
```

The system will try IB first, then Alpaca if IB fails, then mock as last resort.

---

## Migration from Mock Data

**Old Behavior** (before patch):

- TUI: Mock data (no real connections)
- PWA: Alpaca paper trading
- CLI: Real IBKR data (port 7497)

**New Behavior** (after patch):

- TUI: Real IBKR data via IB service (port 8002)
- PWA: Real IBKR data via IB service (port 8002)
- CLI: Real IBKR data (port 7497) - unchanged

**To keep old behavior**: See "How to Revert to Mock Data" above.

---

## Summary

✅ **Default now**: All components use real IBKR paper trading data
✅ **Safe**: Uses paper trading account (port 7497), no real money
✅ **Flexible**: Easy to switch back to mock data via environment variables
✅ **Fallback**: PWA falls back to Alpaca if IB unavailable

For more details, see:

- `config/config.example.json` - Full configuration reference
- `./scripts/run_rust_tui.sh` - TUI runner
- `web/src/config/sharedConfig.ts` - PWA configuration
