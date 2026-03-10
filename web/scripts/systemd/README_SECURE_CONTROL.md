# Secure Service Control from Web App

This document describes how to enable secure service control from the web application on Ubuntu.

## Overview

The web application can control PWA services (start, stop, restart, enable, disable) via the Rust backend API. On Ubuntu with systemd, this uses:

1. **Polkit rules** - Authorization for systemctl commands
2. **systemctl-helper.sh** - Secure wrapper script with service name validation
3. **Rust backend** - API endpoints that use systemctl when available

## Security Features

### 1. Service Name Whitelist

The `systemctl-helper.sh` script only allows operations on predefined PWA services:
- pwa-web
- pwa-alpaca
- pwa-ib-gateway
- pwa-ib
- pwa-discount-bank
- pwa-risk-free-rate
- pwa-jupyterlab
- pwa-nats
- pwa-rust-backend

### 2. Action Validation

Only allowed actions are permitted:
- start
- stop
- restart
- status
- enable
- disable
- is-active
- is-enabled

### 3. Polkit Authorization

Polkit rules ensure only authorized users can control services.

## Installation

### Step 1: Install Systemd Services

```bash
./web/scripts/install-systemd-services.sh --enable
```

### Step 2: Install Polkit Rules

```bash
sudo ./web/scripts/systemd/install-polkit-rules.sh
```

This installs rules to `/etc/polkit-1/rules.d/10-pwa-services.rules`.

### Step 3: Enable Service Control in Backend

Set the environment variable when starting the Rust backend:

```bash
export ENABLE_SERVICE_CONTROL=true
# Or in .env file:
echo "ENABLE_SERVICE_CONTROL=true" >> .env
```

## API Endpoints

The Rust backend provides these endpoints (when `ENABLE_SERVICE_CONTROL=true`):

### Start Service
```
POST /api/v1/services/{service_name}/start
```

### Stop Service
```
POST /api/v1/services/{service_name}/stop
```

### Restart Service
```
POST /api/v1/services/{service_name}/restart
```

### Enable Service
```
POST /api/v1/services/{service_name}/enable
```

### Disable Service
```
POST /api/v1/services/{service_name}/disable
```

### Get Service Status
```
GET /api/v1/services/{service_name}/status
```

## Service Name Mapping

The API accepts these service names (mapped to systemd service names):

| API Name | Systemd Service |
|----------|----------------|
| web | pwa-web |
| alpaca | pwa-alpaca |
| ib, ib-gateway, gateway | pwa-ib-gateway |
| ib | pwa-ib |
| discount-bank, discount_bank | pwa-discount-bank |
| risk-free-rate, risk_free_rate | pwa-risk-free-rate |
| jupyterlab | pwa-jupyterlab |
| nats | pwa-nats |
| rust-backend, rust_backend | pwa-rust-backend |

## How It Works

1. **Web App** → Makes API request to Rust backend
2. **Rust Backend** → Checks `ENABLE_SERVICE_CONTROL` flag
3. **Backend** → Detects if systemctl is available
4. **Backend** → Calls `systemctl-helper.sh` with action and service name
5. **Helper Script** → Validates service name and action
6. **Helper Script** → Executes `systemctl --user` command
7. **Polkit** → Authorizes the action (if needed)
8. **Systemd** → Performs the operation

## Fallback Behavior

If systemctl is not available (non-Linux or systemd not installed), the backend falls back to:
- Shell script execution (start_*.sh, stop_*.sh)
- Port-based status checking
- Config file-based enable/disable

## Security Considerations

1. **Service Control is Disabled by Default**
   - Must set `ENABLE_SERVICE_CONTROL=true` to enable
   - Prevents accidental service control

2. **Service Name Whitelist**
   - Only predefined services can be controlled
   - Prevents arbitrary command execution

3. **Action Validation**
   - Only safe systemctl actions are allowed
   - Prevents injection attacks

4. **User-Level Services**
   - Uses `systemctl --user` (no sudo required)
   - Limited to user's own services

5. **Polkit Rules**
   - Additional authorization layer
   - Can be customized for your security requirements

## Troubleshooting

### Service Control Returns 403 Forbidden

Check that `ENABLE_SERVICE_CONTROL=true` is set in the backend environment.

### systemctl Commands Fail

1. Verify systemd services are installed:
   ```bash
   ls ~/.config/systemd/user/pwa-*.service
   ```

2. Check Polkit rules are installed:
   ```bash
   ls /etc/polkit-1/rules.d/10-pwa-services.rules
   ```

3. Test systemctl directly:
   ```bash
   systemctl --user status pwa-web.service
   ```

### Helper Script Not Found

Ensure the helper script exists and is executable:
```bash
ls -la web/scripts/systemd/systemctl-helper.sh
chmod +x web/scripts/systemd/systemctl-helper.sh
```

## Testing

Test service control from command line:

```bash
# Test helper script directly
./web/scripts/systemd/systemctl-helper.sh status web

# Test via API (with backend running)
curl -X POST http://localhost:8080/api/v1/services/web/start \
  -H "Content-Type: application/json" \
  -d '{}'

curl http://localhost:8080/api/v1/services/web/status
```

## Uninstalling

To remove Polkit rules:

```bash
sudo rm /etc/polkit-1/rules.d/10-pwa-services.rules
sudo systemctl reload polkit
```

To disable service control:

```bash
unset ENABLE_SERVICE_CONTROL
# Or remove from .env file
```
