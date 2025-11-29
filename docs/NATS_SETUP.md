# NATS Server Setup Guide

This guide covers setting up and running the NATS message queue server for local development.

## Quick Start

```bash
# Install NATS server
./scripts/install_nats.sh

# Start NATS server
./scripts/start_nats.sh

# Verify it's running
curl http://localhost:8222/healthz

# Stop NATS server
./scripts/stop_nats.sh
```

## Installation

### Method 1: Homebrew (Recommended)

```bash
brew tap nats-io/nats-tools
brew install nats-server
brew install nats-io/nats-tools/nats  # Optional: CLI tools
```

### Method 2: Direct Binary

The `install_nats.sh` script will automatically download and install the latest NATS server binary if Homebrew is not available.

### Method 3: Manual Installation

1. Download from [NATS releases](https://github.com/nats-io/nats-server/releases)
2. Extract and place `nats-server` in your PATH
3. Verify: `nats-server -v`

## Configuration

The NATS server configuration is located at `config/nats-server.conf`.

### Default Settings (Development)

- **Port**: 4222 (client connections)
- **HTTP Port**: 8222 (monitoring)
- **Host**: 0.0.0.0 (all interfaces)
- **Logging**: Enabled, logs to `logs/nats-server.log`
- **Authentication**: Disabled (development only)
- **TLS**: Disabled (development only)
- **JetStream**: Disabled (Phase 1)

### Production Settings

For production, you should:
1. Enable authentication (see `config/nats-server.conf` comments)
2. Enable TLS encryption
3. Configure clustering (if needed)
4. Enable JetStream for persistence (if needed)

## Running the Server

### Start Server

```bash
./scripts/start_nats.sh
```

Or manually:
```bash
nats-server -c config/nats-server.conf
```

### Stop Server

```bash
./scripts/stop_nats.sh
```

Or manually:
```bash
pkill nats-server
```

### Check Status

```bash
# Health check
curl http://localhost:8222/healthz

# Server info
curl http://localhost:8222/varz

# Connections
curl http://localhost:8222/connz

# Subscriptions
curl http://localhost:8222/subsz
```

## Monitoring

### HTTP Endpoints

- **Health**: `http://localhost:8222/healthz`
- **Server Info**: `http://localhost:8222/varz`
- **Connections**: `http://localhost:8222/connz`
- **Subscriptions**: `http://localhost:8222/subsz`
- **Routes**: `http://localhost:8222/routez` (clustering)

### NATS CLI Tool

If you installed the NATS CLI tools:

```bash
# Subscribe to a topic
nats sub "market-data.tick.>"

# Publish a message
nats pub "market-data.tick.SPY" "Hello World"

# Request/Reply
nats req "rpc.snapshot" ""

# Monitor server
nats server check
```

## Integration with Launch Scripts

The NATS server can be started automatically with other agents:

```bash
# Start all agents (including NATS)
./agents/launch_all_agents.sh
```

## Troubleshooting

### Port Already in Use

If port 4222 is already in use:

```bash
# Find process using port
lsof -i :4222

# Kill process or change port in config/nats-server.conf
```

### Permission Issues

If you get permission errors:

```bash
# Make scripts executable
chmod +x scripts/*.sh

# Check NATS server permissions
ls -l $(which nats-server)
```

### Connection Refused

If clients can't connect:

1. Check if server is running: `pgrep -f nats-server`
2. Check server logs: `tail -f logs/nats-server.log`
3. Verify port: `lsof -i :4222`
4. Check firewall settings

### High Memory Usage

For development, NATS uses minimal memory. If you see high usage:
- Check for message backlog (unconsumed messages)
- Reduce max connections in config
- Enable JetStream with storage limits

## Next Steps

After NATS server is running:

1. **Test Connection**: Use NATS CLI or test script
2. **Integrate Backend**: See Phase 1 implementation tasks
3. **Monitor**: Set up monitoring dashboards
4. **Scale**: Configure clustering for production

## References

- [NATS Documentation](https://docs.nats.io/)
- [NATS Server Configuration](https://docs.nats.io/running-a-nats-service/configuration)
- [NATS Monitoring](https://docs.nats.io/running-a-nats-service/nats_server/monitoring)
- [Message Queue Architecture](research/architecture/MESSAGE_QUEUE_ARCHITECTURE.md)
