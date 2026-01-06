# Secure Service Control from Web App - Complete ✅

**Date:** 2025-01-27
**Status:** Complete
**Platform:** Ubuntu/Linux with systemd

---

## Summary

Successfully implemented secure service control from the web application on Ubuntu, allowing the web app to start/stop/restart/enable/disable PWA services via systemctl without requiring sudo.

---

## Implementation

### 1. ✅ Polkit Rules

**File:** `web/scripts/systemd/polkit-rules/10-pwa-services.rules`

- Authorization rules for systemctl commands
- Allows user-level service control for PWA services
- Installed to `/etc/polkit-1/rules.d/`

**Installation:**

```bash
sudo ./web/scripts/systemd/install-polkit-rules.sh
```

### 2. ✅ Secure Helper Script

**File:** `web/scripts/systemd/systemctl-helper.sh`

**Security Features:**

- Service name whitelist (only allows predefined PWA services)
- Action validation (only safe systemctl actions)
- Service name mapping (API names → systemd names)
- Prevents command injection

**Allowed Services:**

- pwa-web, pwa-alpaca, pwa-tradestation, pwa-ib-gateway, pwa-ib
- pwa-discount-bank, pwa-risk-free-rate, pwa-jupyterlab
- pwa-nats, pwa-rust-backend

**Allowed Actions:**

- start, stop, restart, status, enable, disable, is-active, is-enabled

### 3. ✅ Rust Backend Integration

**File:** `agents/backend/crates/api/src/rest.rs`

**Updates:**

- Added `is_systemctl_available()` - Detects if systemctl is available
- Added `execute_systemctl_command()` - Executes systemctl via helper script
- Updated `get_service_status_internal()` - Uses systemctl for status when available
- Updated `service_start()` - Uses systemctl when available
- Updated `service_stop()` - Uses systemctl when available
- Updated `service_restart()` - Uses systemctl when available
- Updated `service_enable()` - Uses systemctl when available
- Updated `service_disable()` - Uses systemctl when available

**Fallback Behavior:**

- If systemctl not available, falls back to shell scripts
- Maintains backward compatibility

### 4. ✅ Documentation

**Files:**

- `web/scripts/systemd/README_SECURE_CONTROL.md` - Complete guide
- Updated `web/scripts/systemd/README.md` - Added secure control section

---

## API Endpoints

All endpoints require `ENABLE_SERVICE_CONTROL=true` environment variable.

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

---

## Service Name Mapping

| API Name | Systemd Service |
|----------|----------------|
| web | pwa-web |
| alpaca | pwa-alpaca |
| tradestation | pwa-tradestation |
| ib, ib-gateway, gateway | pwa-ib-gateway |
| ib | pwa-ib |
| discount-bank, discount_bank | pwa-discount-bank |
| risk-free-rate, risk_free_rate | pwa-risk-free-rate |
| jupyterlab | pwa-jupyterlab |
| nats | pwa-nats |
| rust-backend, rust_backend | pwa-rust-backend |

---

## Security Features

1. **Service Control Disabled by Default**
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
   - Can be customized for security requirements

---

## Setup Instructions

### 1. Install Systemd Services

```bash
./web/scripts/install-systemd-services.sh --enable
```

### 2. Install Polkit Rules

```bash
sudo ./web/scripts/systemd/install-polkit-rules.sh
```

### 3. Enable Service Control in Backend

```bash

# Set environment variable

export ENABLE_SERVICE_CONTROL=true

# Or add to .env file

echo "ENABLE_SERVICE_CONTROL=true" >> .env
```

### 4. Restart Rust Backend

The backend will now use systemctl for service control when available.

---

## Testing

### Test Helper Script

```bash
./web/scripts/systemd/systemctl-helper.sh status web
./web/scripts/systemd/systemctl-helper.sh start web
```

### Test API Endpoints

```bash

# Start service

curl -X POST http://localhost:8080/api/v1/services/web/start \
  -H "Content-Type: application/json" \
  -d '{}'

# Check status

curl http://localhost:8080/api/v1/services/web/status

# Stop service

curl -X POST http://localhost:8080/api/v1/services/web/stop \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

## Files Created/Modified

### Created

- `web/scripts/systemd/polkit-rules/10-pwa-services.rules`
- `web/scripts/systemd/install-polkit-rules.sh`
- `web/scripts/systemd/systemctl-helper.sh`
- `web/scripts/systemd/README_SECURE_CONTROL.md`
- `docs/SECURE_SERVICE_CONTROL_COMPLETE.md` (this file)

### Modified

- `agents/backend/crates/api/src/rest.rs` (added systemctl support)
- `web/scripts/systemd/README.md` (added secure control section)

---

## How It Works

1. **Web App** → Makes API request to Rust backend
2. **Rust Backend** → Checks `ENABLE_SERVICE_CONTROL` flag
3. **Backend** → Detects if systemctl is available
4. **Backend** → Calls `systemctl-helper.sh` with action and service name
5. **Helper Script** → Validates service name and action
6. **Helper Script** → Executes `systemctl --user` command
7. **Systemd** → Performs the operation

---

## Troubleshooting

### Service Control Returns 403 Forbidden

- Check that `ENABLE_SERVICE_CONTROL=true` is set
- Verify backend is running with the environment variable

### systemctl Commands Fail

- Verify systemd services are installed: `ls ~/.config/systemd/user/pwa-*.service`
- Check Polkit rules: `ls /etc/polkit-1/rules.d/10-pwa-services.rules`
- Test systemctl directly: `systemctl --user status pwa-web.service`

### Helper Script Not Found

- Ensure script exists: `ls -la web/scripts/systemd/systemctl-helper.sh`
- Make executable: `chmod +x web/scripts/systemd/systemctl-helper.sh`

---

## Next Steps

1. Test on Ubuntu system
2. Verify web app can control services via UI
3. Consider adding audit logging for service control actions
4. Add rate limiting to prevent abuse

---

## Notes

- User-level systemd services don't strictly require Polkit, but having rules provides an extra security layer
- The helper script provides defense-in-depth with service name validation
- All service control is logged by systemd journal
- The implementation maintains backward compatibility with script-based control
