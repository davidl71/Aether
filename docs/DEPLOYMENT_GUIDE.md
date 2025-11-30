# Production Deployment Guide

**Date**: 2025-11-17
**Status**: Active Deployment Procedures
**Purpose**: Comprehensive guide for deploying the IBKR Box Spread Generator to production environments

---

## Overview

This guide provides step-by-step procedures for deploying the box spread trading system to production. It covers environment setup, configuration, security, monitoring, and operational procedures.

**⚠️ CRITICAL**: Always test thoroughly in paper trading before deploying to production with real money.

---

## Pre-Deployment Checklist

### 1. Code Readiness

- [ ] All integration tests pass (`ctest --test-dir build --output-on-failure`)
- [ ] Paper trading validation complete (5-day plan executed)
- [ ] Code review completed
- [ ] Security audit passed
- [ ] Performance benchmarks met
- [ ] Documentation updated

### 2. Environment Readiness

- [ ] Production server provisioned
- [ ] TWS/Gateway installed and configured
- [ ] Network connectivity verified
- [ ] Firewall rules configured
- [ ] SSL certificates installed (if using HTTPS)
- [ ] Backup systems configured

### 3. Configuration Readiness

- [ ] Production configuration file prepared
- [ ] API keys and credentials secured
- [ ] Trading permissions verified
- [ ] Risk limits configured
- [ ] Monitoring systems configured

---

## Deployment Architecture

### Recommended Production Setup

```
┌─────────────────────────────────────────────────────────────┐
│                    Production Server                        │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  C++ Trading Binary (ib_box_spread)                   │ │
│  │  - Runs as systemd service                             │ │
│  │  - Auto-restart on failure                              │ │
│  │  - Log rotation configured                              │ │
│  └──────────────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  Python Services (Optional)                          │ │
│  │  - FastAPI services for web/TUI                       │ │
│  │  - Systemd service management                         │ │
│  └──────────────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  Rust Backend Services (Optional)                    │ │
│  │  - REST API service                                   │ │
│  │  - gRPC services                                      │ │
│  └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
          │
┌─────────┼─────────────────────────────────────────────────┐
│         │  TWS/Gateway (Local or Remote)                    │
│  ┌──────▼──────────────────────────────────────────────┐   │
│  │  Interactive Brokers TWS/Gateway                    │   │
│  │  - API enabled                                       │   │
│  │  - Port 7496 (live) or 7497 (paper)                  │   │
│  │  - IP whitelist configured                           │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Step 1: Server Setup

### 1.1 System Requirements

**Minimum Requirements**:

- **OS**: Linux (Ubuntu 22.04+ recommended) or macOS 11.0+
- **CPU**: 4+ cores
- **RAM**: 8GB+ (16GB recommended)
- **Storage**: 50GB+ SSD
- **Network**: Stable internet connection, low latency to IB servers

**Recommended**:

- Dedicated server (not shared)
- Redundant network connections
- UPS backup power
- Monitoring and alerting infrastructure

### 1.2 Operating System Setup

**Linux (Ubuntu/Debian)**:

```bash

# Update system

sudo apt update && sudo apt upgrade -y

# Install build dependencies

sudo apt install -y \
    build-essential \
    cmake \
    ninja-build \
    git \
    curl \
    wget \
    python3 \
    python3-pip \
    rust \
    golang

# Install runtime dependencies

sudo apt install -y \
    libprotobuf-dev \
    libabsl-dev \
    pkg-config
```

**macOS**:

```bash

# Install Homebrew (if not installed)

/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies

brew install cmake ninja git python@3.11 rust go
```

### 1.3 User and Permissions

```bash

# Create dedicated user for trading system

sudo useradd -m -s /bin/bash ib_trading
sudo mkdir -p /opt/ib_box_spread
sudo chown ib_trading:ib_trading /opt/ib_box_spread

# Set up SSH access (if remote)
# Configure SSH keys for ib_trading user
```

---

## Step 2: Build and Install

### 2.1 Clone Repository

```bash
cd /opt/ib_box_spread
sudo -u ib_trading git clone <repository-url> .
sudo -u ib_trading git checkout <production-branch>
```

### 2.2 Build C++ Binary

```bash
cd /opt/ib_box_spread

# Configure build

cmake --preset macos-universal-release  # or linux-release

# Build

cmake --build --preset macos-universal-release

# Verify build

./build/ib_box_spread --version
```

### 2.3 Install Python Dependencies (if using Python services)

```bash
cd /opt/ib_box_spread/python
sudo -u ib_trading pip3 install --user -r requirements.txt

# Build Cython bindings

cd bindings
sudo -u ib_trading pip3 install --user -e .
```

### 2.4 Build Rust Services (if using)

```bash
cd /opt/ib_box_spread/agents/backend
sudo -u ib_trading cargo build --release
```

---

## Step 3: Configuration

### 3.1 Create Production Configuration

```bash

# Create config directory

sudo -u ib_trading mkdir -p ~/.config/ib_box_spread

# Generate sample config

sudo -u ib_trading /opt/ib_box_spread/build/ib_box_spread --init-config

# Edit configuration

sudo -u ib_trading nano ~/.config/ib_box_spread/config.json
```

### 3.2 Production Configuration Template

```json
{
  "tws": {
    "host": "127.0.0.1",
    "port": 7496,
    "client_id": 1,
    "use_mock": false,
    "auto_reconnect": true,
    "max_reconnect_attempts": 10,
    "connection_timeout_ms": 10000
  },
  "strategy": {
    "symbols": ["SPX", "XSP"],
    "min_arbitrage_profit": 0.25,
    "min_roi_percent": 1.0,
    "max_position_size": 5,
    "min_days_to_expiry": 7,
    "max_days_to_expiry": 60,
    "max_bid_ask_spread": 0.50,
    "min_volume": 50,
    "min_open_interest": 500
  },
  "risk": {
    "max_total_exposure": 100000.0,
    "max_positions": 10,
    "max_daily_loss": 5000.0,
    "max_position_size": 5
  },
  "dry_run": false,
  "continue_on_error": false,
  "logging": {
    "log_file": "/var/log/ib_box_spread/ib_box_spread.log",
    "log_level": "info",
    "log_to_console": false,
    "max_file_size_mb": 100,
    "max_files": 30
  },
  "loop_delay_ms": 1000
}
```

### 3.3 Security Configuration

**File Permissions**:

```bash

# Secure configuration file

chmod 600 ~/.config/ib_box_spread/config.json
chown ib_trading:ib_trading ~/.config/ib_box_spread/config.json

# Secure log directory

sudo mkdir -p /var/log/ib_box_spread
sudo chown ib_trading:ib_trading /var/log/ib_box_spread
chmod 750 /var/log/ib_box_spread
```

**Environment Variables** (if needed):

```bash

# Add to ~/.bashrc or systemd service file

export IB_BOX_SPREAD_CONFIG=/home/ib_trading/.config/ib_box_spread/config.json
export TWS_MOCK=false
```

---

## Step 4: TWS/Gateway Configuration

### 4.1 TWS API Settings

1. **Enable API**:
   - TWS → Configure → API → Settings
   - Check "Enable ActiveX and Socket Clients"
   - Set port: 7496 (live) or 7497 (paper)
   - Add trusted IP: 127.0.0.1 (or server IP)

2. **API Permissions**:
   - Enable "Read-Only API" (if testing)
   - Enable "Modify Orders" (for trading)
   - Enable "Download Account Data"

3. **Connection Settings**:
   - Master API client ID: 0
   - Allow connections from localhost only (recommended)

### 4.2 Gateway Configuration (Headless)

**IB Gateway Setup**:

- Download IB Gateway from IB website
- Install on server
- Configure API settings (same as TWS)
- Set to auto-start on boot

**Systemd Service** (if needed):

```ini
[Unit]
Description=Interactive Brokers Gateway
After=network.target

[Service]
Type=simple
User=ib_trading
ExecStart=/path/to/ibgateway
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

---

## Step 5: Service Management

### 5.1 Systemd Service (Linux)

**Create Service File**: `/etc/systemd/system/ib-box-spread.service`

```ini
[Unit]
Description=IBKR Box Spread Generator
After=network.target

[Service]
Type=simple
User=ib_trading
Group=ib_trading
WorkingDirectory=/opt/ib_box_spread
ExecStart=/opt/ib_box_spread/build/ib_box_spread \
    --config /home/ib_trading/.config/ib_box_spread/config.json
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security settings

NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/log/ib_box_spread

# Resource limits

LimitNOFILE=65536
MemoryMax=4G

[Install]
WantedBy=multi-user.target
```

**Enable and Start**:

```bash
sudo systemctl daemon-reload
sudo systemctl enable ib-box-spread
sudo systemctl start ib-box-spread
sudo systemctl status ib-box-spread
```

### 5.2 Launchd Service (macOS)

**Create Plist**: `~/Library/LaunchAgents/com.ib_box_spread.plist`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.ib_box_spread</string>
    <key>ProgramArguments</key>
    <array>
        <string>/opt/ib_box_spread/build/ib_box_spread</string>
        <string>--config</string>
        <string>/Users/ib_trading/.config/ib_box_spread/config.json</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/var/log/ib_box_spread/ib_box_spread.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/ib_box_spread/ib_box_spread.error.log</string>
</dict>
</plist>
```

**Load Service**:

```bash
launchctl load ~/Library/LaunchAgents/com.ib_box_spread.plist
launchctl start com.ib_box_spread
```

---

## Step 6: Logging and Monitoring

### 6.1 Log Configuration

**Log Rotation** (logrotate):

```bash

# Create logrotate config: /etc/logrotate.d/ib-box-spread

/var/log/ib_box_spread/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0640 ib_trading ib_trading
    sharedscripts
    postrotate
        systemctl reload ib-box-spread > /dev/null 2>&1 || true
    endscript
}
```

### 6.2 Monitoring Setup

**Health Check Script**:

```bash

#!/bin/bash
# /opt/ib_box_spread/scripts/health_check.sh

# Check if service is running

if ! systemctl is-active --quiet ib-box-spread; then
    echo "ERROR: Service is not running"
    exit 1
fi

# Check TWS connection (if health endpoint exists)
# curl -f http://localhost:8080/health || exit 1

# Check log for recent errors

if tail -n 100 /var/log/ib_box_spread/ib_box_spread.log | grep -q "ERROR\|CRITICAL"; then
    echo "WARNING: Recent errors in log"
fi

echo "OK: Service is healthy"
exit 0
```

**Cron Job** (every 5 minutes):

```bash

# Add to crontab: crontab -e
*/5 * * * * /opt/ib_box_spread/scripts/health_check.sh
```

### 6.3 Alerting

**Email Alerts** (configure in config.json):

```json
{
  "notifications": {
    "enabled": true,
    "channels": [
      {
        "type": "email",
        "smtp_host": "smtp.gmail.com",
        "smtp_port": 587,
        "from": "alerts@yourdomain.com",
        "to": ["ops@yourdomain.com"],
        "events": [
          "connection_lost",
          "order_rejected",
          "efficiency_ratio_low",
          "daily_loss_limit"
        ]
      }
    ]
  }
}
```

---

## Step 7: Security Hardening

### 7.1 Network Security

**Firewall Rules**:

```bash

# Allow only localhost connections to TWS

sudo ufw allow from 127.0.0.1 to any port 7496
sudo ufw deny 7496

# Allow SSH (if remote)

sudo ufw allow 22/tcp

# Enable firewall

sudo ufw enable
```

### 7.2 File System Security

```bash

# Restrict access to trading directory

sudo chmod 750 /opt/ib_box_spread
sudo chown -R ib_trading:ib_trading /opt/ib_box_spread

# Secure configuration

chmod 600 ~/.config/ib_box_spread/config.json

# Secure logs

chmod 640 /var/log/ib_box_spread/*.log
```

### 7.3 Credential Management

**Never commit credentials**:

- Use environment variables for sensitive data
- Store API keys in secure vault (HashiCorp Vault, AWS Secrets Manager)
- Rotate credentials regularly

**Example**:

```bash

# Use environment variables

export IB_API_KEY=$(vault kv get -field=api_key secret/ib_box_spread)
export IB_API_SECRET=$(vault kv get -field=api_secret secret/ib_box_spread)
```

---

## Step 8: Backup and Recovery

### 8.1 Backup Strategy

**Configuration Backup**:

```bash

# Daily backup script
#!/bin/bash

BACKUP_DIR="/backup/ib_box_spread"
DATE=$(date +%Y%m%d)

mkdir -p "$BACKUP_DIR/$DATE"
cp ~/.config/ib_box_spread/config.json "$BACKUP_DIR/$DATE/"
cp -r /var/log/ib_box_spread "$BACKUP_DIR/$DATE/logs/"

# Keep last 30 days

find "$BACKUP_DIR" -type d -mtime +30 -exec rm -rf {} +
```

**Database Backup** (if using QuestDB):

```bash

# Backup QuestDB data

questdb backup /backup/ib_box_spread/questdb_$(date +%Y%m%d).tar.gz
```

### 8.2 Recovery Procedures

**Service Recovery**:

```bash

# Restart service

sudo systemctl restart ib-box-spread

# Check status

sudo systemctl status ib-box-spread

# View logs

sudo journalctl -u ib-box-spread -f
```

**Configuration Recovery**:

```bash

# Restore from backup

cp /backup/ib_box_spread/YYYYMMDD/config.json ~/.config/ib_box_spread/
sudo systemctl restart ib-box-spread
```

---

## Step 9: Performance Tuning

### 9.1 System Tuning

**Linux**:

```bash

# Increase file descriptor limits

echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Network tuning

sudo sysctl -w net.core.somaxconn=4096
sudo sysctl -w net.ipv4.tcp_max_syn_backlog=4096
```

**macOS**:

```bash

# Increase file descriptor limits

sudo launchctl limit maxfiles 65536 200000
```

### 9.2 Application Tuning

**Configuration Optimizations**:

```json
{
  "loop_delay_ms": 500,  // Reduce for faster scanning
  "tws": {
    "connection_timeout_ms": 5000,  // Optimize timeout
    "auto_reconnect": true,
    "max_reconnect_attempts": 10
  },
  "logging": {
    "log_level": "info"  // Reduce verbosity in production
  }
}
```

---

## Step 10: Verification and Testing

### 10.1 Pre-Production Verification

```bash

# 1. Verify service is running

sudo systemctl status ib-box-spread

# 2. Check TWS connection
# (Service should log "Connected to TWS")

# 3. Verify configuration

/opt/ib_box_spread/build/ib_box_spread --validate --config ~/.config/ib_box_spread/config.json

# 4. Test in dry-run mode first
# Edit config: "dry_run": true
# Run for 1 hour, verify no real orders placed

# 5. Check logs for errors

tail -f /var/log/ib_box_spread/ib_box_spread.log
```

### 10.2 Production Readiness Checklist

- [ ] Service starts automatically on boot
- [ ] TWS connection established
- [ ] Configuration validated
- [ ] Logging working correctly
- [ ] Monitoring and alerts configured
- [ ] Backup system tested
- [ ] Security hardening applied
- [ ] Performance acceptable
- [ ] Paper trading validation complete (5 days)
- [ ] Team trained on operations

---

## Operational Procedures

### Daily Operations

**Morning Checklist**:

1. Check service status: `sudo systemctl status ib-box-spread`
2. Review overnight logs: `tail -n 100 /var/log/ib_box_spread/ib_box_spread.log`
3. Verify TWS connection active
4. Check for alerts/notifications
5. Review positions and orders

**Monitoring**:

- Service uptime
- TWS connection status
- Order execution rate
- Efficiency ratio
- Error rates
- System resource usage

### Weekly Operations

1. Review performance metrics
2. Check log rotation
3. Verify backup integrity
4. Review security logs
5. Update documentation if needed

### Monthly Operations

1. Review trading performance
2. Analyze error patterns
3. Update dependencies (if needed)
4. Security audit
5. Performance optimization review

---

## Troubleshooting

### Common Issues

**Service Won't Start**:

```bash

# Check logs

sudo journalctl -u ib-box-spread -n 50

# Verify configuration

/opt/ib_box_spread/build/ib_box_spread --validate

# Check permissions

ls -la /opt/ib_box_spread/build/ib_box_spread
ls -la ~/.config/ib_box_spread/config.json
```

**TWS Connection Failures**:

```bash

# Verify TWS is running

netstat -an | grep 7496

# Check TWS API settings
# TWS → Configure → API → Settings

# Test connection manually

telnet 127.0.0.1 7496
```

**High Error Rates**:

```bash

# Check error logs

grep ERROR /var/log/ib_box_spread/ib_box_spread.log | tail -20

# Review system resources

top
df -h
free -h
```

---

## Rollback Procedures

### Quick Rollback

```bash

# Stop service

sudo systemctl stop ib-box-spread

# Restore previous version

cd /opt/ib_box_spread
git checkout <previous-version>
cmake --build build

# Restore previous configuration

cp /backup/ib_box_spread/YYYYMMDD/config.json ~/.config/ib_box_spread/

# Restart service

sudo systemctl start ib-box-spread
```

### Full Rollback

1. Stop all services
2. Restore from backup
3. Rebuild if needed
4. Restore configuration
5. Verify and restart

---

## References

- `docs/PAPER_TRADING_VALIDATION_PLAN.md` - Pre-production validation
- `docs/MULTI_LANGUAGE_ARCHITECTURE.md` - Architecture overview
- `README.md` - General project documentation
- `docs/MERGED_ACTION_PLAN.md` - Production readiness checklist

---

**Document Status**: ✅ Complete - Comprehensive production deployment guide
