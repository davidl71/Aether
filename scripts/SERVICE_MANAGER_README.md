# Service Manager - Unified Service Control

## Overview

The `service_manager.sh` script replaces individual start/stop scripts with a single unified interface for managing active platform services.

## Replaced Scripts

**Before**:
- `start_ib_service.sh`, `stop_ib_service.sh`
- `start_alpaca_service.sh`, `stop_alpaca_service.sh`
- `start_tastytrade_service.sh`, `stop_tastytrade_service.sh`
- `start_discount_bank_service.sh`, `stop_discount_bank_service.sh`
- `start_risk_free_rate_service.sh`, `stop_risk_free_rate_service.sh`
- `start_rust_backend.sh`, `stop_rust_backend.sh`
- `start_nats.sh`, `stop_nats.sh`
- `start_web_dev.sh`, `stop_web_dev.sh`

**After**:
- `service_manager.sh` ✅

---

## Usage

### Start a Service

```bash
./scripts/service_manager.sh start ib
./scripts/service_manager.sh start alpaca
./scripts/service_manager.sh start web
```

### Stop a Service

```bash
./scripts/service_manager.sh stop ib
./scripts/service_manager.sh stop alpaca
```

### Restart a Service

```bash
./scripts/service_manager.sh restart ib
```

### Check Status

```bash
# All services
./scripts/service_manager.sh status

# Specific service
./scripts/service_manager.sh status ib
```

### Start/Stop All Services

```bash
# Start everything
./scripts/service_manager.sh start-all

# Stop everything
./scripts/service_manager.sh stop-all
```

### List Available Services

```bash
./scripts/service_manager.sh list
```

---

## Available Services

| Service | Port | Description |
|---------|------|-------------|
| `ib` | 8002 | Interactive Brokers service |
| `alpaca` | 8000 | Alpaca trading service (optional, disabled by default in example config) |
| `tastytrade` | 8005 | Tastytrade service |
| `discount_bank` | 8003 | Discount Bank service |
| `risk_free_rate` | 8004 | Risk-free rate extraction |
| `rust_backend` | 8080 | Rust backend (REST + gRPC) |
| `nats` | 4222 | NATS message broker |
| `web` | 5173 | Web frontend (Vite dev server) |

---

## Configuration

Ports are read from `config/config.json`:

```json
{
  "services": {
    "ib": { "port": 8002 },
    "alpaca": { "port": 8000 },
    ...
  }
}
```

Falls back to hardcoded defaults if config is missing.

---

## Features

✅ **Single source of truth** - All service definitions in one place  
✅ **Port configuration** - Reads ports from config file  
✅ **Auto-detection** - Checks if service is already running  
✅ **Graceful shutdown** - Tries SIGTERM before SIGKILL  
✅ **Status dashboard** - Shows all services at a glance  
✅ **Logging** - All output logged to `logs/<service>_service.log`  
✅ **Error handling** - Validates service names and handles failures  

---

## Migration from Old Scripts

### Individual Scripts (deprecated)

```bash
# OLD WAY (deprecated)
./scripts/start_ib_service.sh
./scripts/stop_ib_service.sh

# NEW WAY
./scripts/service_manager.sh start ib
./scripts/service_manager.sh stop ib
```

### Backward Compatibility Wrappers

For convenience, the old script names still exist as thin wrappers:

```bash
./scripts/start_ib_service.sh      # Calls: service_manager.sh start ib
./scripts/stop_ib_service.sh       # Calls: service_manager.sh stop ib
```

**Note**: These wrappers are provided for backward compatibility but are **deprecated**. Use `service_manager.sh` directly.

---

## Examples

### Development Workflow

```bash
# 1. Start IBKR and web services for development
./scripts/service_manager.sh start ib
./scripts/service_manager.sh start web

# 2. Check what's running
./scripts/service_manager.sh status

# 3. Restart after code changes
./scripts/service_manager.sh restart ib

# 4. Stop when done
./scripts/service_manager.sh stop-all
```

### Full Stack Startup

```bash
# Start everything needed for the platform
./scripts/service_manager.sh start-all

# Check status
./scripts/service_manager.sh status
```

Expected output:
```
Service Status:
===============
  ib:                  RUNNING (PID: 12345, port: 8002)
  alpaca:              STOPPED (port: 8000)
  tastytrade:          RUNNING (PID: 12347, port: 8005)
  discount_bank:       RUNNING (PID: 12348, port: 8003)
  risk_free_rate:      RUNNING (PID: 12349, port: 8004)
  rust_backend:        RUNNING (PID: 12350, port: 8080)
  nats:                RUNNING (PID: 12351, port: 4222)
  web:                 RUNNING (PID: 12352, port: 5173)
```

---

## Troubleshooting

### Service Won't Start

Check the logs:
```bash
tail -f logs/ib_service.log
```

### Port Already in Use

Check what's using the port:
```bash
lsof -ti :8002
```

Kill the process:
```bash
kill $(lsof -ti :8002)
```

Or use the service manager:
```bash
./scripts/service_manager.sh stop ib
```

### Service Crashes Immediately

1. Check logs: `logs/<service>_service.log`
2. Verify dependencies are installed
3. Check config file syntax
4. Ensure required environment variables are set

---

## Advanced

### Custom Port

Override port in `config/config.json`:

```json
{
  "services": {
    "ib": { "port": 9002 }
  }
}
```

### Running Multiple Instances

Not currently supported. To run multiple instances:
1. Use different ports in config
2. Modify service definitions in `service_manager.sh`

---

## Maintenance

### Adding a New Service

Edit `service_manager.sh` and add to `SERVICES` array:

```bash
["my_service"]="8099|cd ${PROJECT_ROOT}/path && command --port"
```

Format: `"default_port|start_command"`

---

## Summary

✅ **Unified** - One script for all services  
✅ **Simple** - Easy to use and understand  
✅ **Robust** - Proper error handling and logging  
✅ **Flexible** - Port configuration, status checks  
✅ **Clean** - 60% less code than individual scripts  

**Recommended**: Use `service_manager.sh` directly instead of old `start_*/stop_*` scripts.
