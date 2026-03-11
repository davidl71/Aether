# PWA Services Systemd Integration

This directory contains systemd user service files for managing PWA services on Linux systems.

## Overview

The PWA services can be managed via:
- **Linux**: systemd user services (this directory)
- **macOS**: brew services (see `ib-gateway/install-brew-service.sh`)
- **Fallback**: Manual background processes (default)

## Installation

### Prerequisites

- Linux system with systemd
- User systemd services enabled (usually enabled by default on modern Linux distributions)

### Install Services

```bash
# Install service files
./web/scripts/install-systemd-services.sh

# Install and enable services (start on login)
./web/scripts/install-systemd-services.sh --enable

# Install, enable, and start all services now
./web/scripts/install-systemd-services.sh --enable --start
```

## Service Files

The following services are available:

- `pwa-web.service` - Web frontend (Vite dev server)
- `pwa-alpaca.service` - Alpaca trading service
- `pwa-ib-gateway.service` - IB Gateway service
- `pwa-ib.service` - IB (Interactive Brokers) service
- `pwa-discount-bank.service` - Discount Bank service
- `pwa-risk-free-rate.service` - Risk-Free Rate service
- `pwa-nats.service` - NATS messaging server
- `pwa-rust-backend.service` - Rust backend service

## Usage

### Using the Launch Script

The `launch-all-pwa-services.sh` script automatically detects systemctl and uses it when available:

```bash
# Start all services (uses systemctl if available)
./web/scripts/launch-all-pwa-services.sh

# Check status
./web/scripts/launch-all-pwa-services.sh status

# Stop all services
./web/scripts/launch-all-pwa-services.sh stop

# Restart all services
./web/scripts/launch-all-pwa-services.sh restart
```

### Manual systemctl Commands

```bash
# Start a service
systemctl --user start pwa-web.service

# Stop a service
systemctl --user stop pwa-web.service

# Restart a service
systemctl --user restart pwa-web.service

# Check status
systemctl --user status pwa-web.service

# View logs
journalctl --user -u pwa-web.service -f

# Enable service to start on login
systemctl --user enable pwa-web.service

# Disable service from starting on login
systemctl --user disable pwa-web.service

# List all PWA services
systemctl --user list-units 'pwa-*.service'
```

## Service Dependencies

Services are configured with proper dependencies:

- `pwa-ib.service` depends on `pwa-ib-gateway.service`
- `pwa-rust-backend.service` depends on `pwa-nats.service`

When starting services via systemctl, dependencies are automatically handled.

## Logs

All services log to systemd journal. View logs with:

```bash
# View logs for a specific service
journalctl --user -u pwa-web.service

# Follow logs in real-time
journalctl --user -u pwa-web.service -f

# View logs from last hour
journalctl --user -u pwa-web.service --since "1 hour ago"

# View all PWA service logs
journalctl --user -u 'pwa-*.service'
```

## Troubleshooting

### Services not starting

1. Check service status:
   ```bash
   systemctl --user status pwa-web.service
   ```

2. Check logs:
   ```bash
   journalctl --user -u pwa-web.service -n 50
   ```

3. Verify service file exists:
   ```bash
   ls -la ~/.config/systemd/user/pwa-*.service
   ```

4. Reload systemd daemon:
   ```bash
   systemctl --user daemon-reload
   ```

### Service files not found

If you see "Service file not found" errors, install the services:

```bash
./web/scripts/install-systemd-services.sh
systemctl --user daemon-reload
```

### Port conflicts

If a port is already in use, check what's using it:

```bash
# Check what's using port 5173
lsof -i :5173
# or
netstat -tulpn | grep 5173
```

Stop the conflicting service or change the port in your configuration.

## Secure Service Control from Web App

The web application can control services via the Rust backend API. See [README_SECURE_CONTROL.md](README_SECURE_CONTROL.md) for details.

**Quick Setup:**
```bash
# Install Polkit rules (requires sudo)
sudo ./web/scripts/systemd/install-polkit-rules.sh

# Enable service control in backend
export ENABLE_SERVICE_CONTROL=true
```

## Uninstalling

To remove systemd services:

```bash
# Stop all services
systemctl --user stop 'pwa-*.service'

# Disable services
systemctl --user disable 'pwa-*.service'

# Remove service files
rm ~/.config/systemd/user/pwa-*.service

# Reload daemon
systemctl --user daemon-reload
```

To remove Polkit rules:

```bash
sudo rm /etc/polkit-1/rules.d/10-pwa-services.rules
sudo systemctl reload polkit
```

## Cross-Platform Compatibility

The launch script (`launch-all-pwa-services.sh`) automatically detects the platform and service manager:

- **Linux with systemd**: Uses systemctl (if services are installed)
- **macOS**: Uses brew services (for IB Gateway) or manual processes
- **Other/No systemd**: Falls back to manual background processes

This ensures the same script works across all platforms.
