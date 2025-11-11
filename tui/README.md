# IBKR Box Spread TUI

A terminal-based dashboard for monitoring and managing the IBKR Box Spread trading system.

## Overview

The TUI (Terminal User Interface) provides a real-time, interactive view of your trading system in a familiar `top`/`htop`-style interface. It displays account metrics, positions, orders, and strategy status with color-coded visual feedback.

## Features

- **Real-time Monitoring**: Live updates of account status, positions, and orders
- **Interactive Navigation**: Keyboard-driven interface with tab navigation
- **Visual Feedback**: Color-coded metrics and status indicators
- **Multiple Data Sources**: REST API, Mock data, or Nautilus Trader integration
- **Offline Testing**: Mock data provider for development without backend

## Installation

### From Source

```bash
cd tui
go build -o ib-box-spread-tui ./cmd/tui
sudo mv ib-box-spread-tui /usr/local/bin/
```

### Install Man Page

```bash
sudo mkdir -p /usr/local/share/man/man1
sudo cp man/ib-box-spread-tui.1 /usr/local/share/man/man1/
sudo mandb
```

## Quick Start

### Basic Usage

```bash
# Start TUI (connects to default REST endpoint)
ib-box-spread-tui

# Use mock data for offline testing
ib-box-spread-tui --mock

# Connect to custom endpoint
ib-box-spread-tui --endpoint http://localhost:3000/api/snapshot
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle through tabs |
| `↑` `↓` `←` `→` | Navigate and scroll |
| `Enter` | Open detail popup |
| `S` | Start/resume strategy |
| `T` | Stop strategy |
| `K` | Cancel all orders |
| `D` | Toggle dry-run mode |
| `B` | Buy combo (lower cost basis) |
| `Shift+S` | Sell combo (realize gains) |
| `?` | Show help |
| `Q` | Quit |

## Tabs

### Dashboard
Main overview showing:
- Account summary (net liquidation, buying power, margin)
- Symbol watchlist with prices and ROI
- Active positions summary
- Recent orders
- System status (TWS, ORATS, Portal, QuestDB)

### Current Positions
Open box spread positions with:
- Strike prices and expiration dates
- Net debit and theoretical value
- Arbitrage profit and ROI
- Days to expiration
- Maker/taker counts
- APR vs benchmark comparison

### Historic Positions
Recently closed positions:
- Closed position details
- Realized P&L
- Execution timestamps
- Performance metrics

### Orders
Order timeline:
- Submission time
- Status (pending, filled, cancelled)
- Fill details
- Order parameters

### Alerts
Notification feed:
- System alerts (info, warning, error)
- Strategy notifications
- Connection status
- Risk warnings

## Data Sources

### REST API (Default)
Polls REST endpoints for:
- Strategy status
- Positions and account data
- Orders and executions
- Alerts

**Default endpoint**: `http://localhost:8080/api/snapshot`
**Default interval**: 1 second

### Mock Provider
Generates synthetic data for offline testing:
```bash
ib-box-spread-tui --mock
```

### Nautilus Trader
Integration with Nautilus Trader:
```bash
ib-box-spread-tui --nautilus
```

## Configuration

### Environment Variables

- `TUI_ENDPOINT`: Default REST API endpoint
- `TUI_INTERVAL`: Default polling interval (seconds)
- `TERM`: Terminal type (affects color support)

### Command-Line Options

```
-h, --help              Show help message
-v, --version           Show version information
--endpoint URL          REST API endpoint (default: http://localhost:8080/api/snapshot)
--interval SECONDS      Polling interval (default: 1)
--mock                  Use mock data provider
--nautilus              Use Nautilus Trader provider
```

## Color Scheme

- **Green**: Positive values, healthy status, profits
- **Red**: Negative values, errors, losses
- **Yellow**: Warnings, margin requirements, benchmarks
- **Cyan**: Headers, maker counts, system OK
- **Magenta**: Taker counts, commissions
- **Blue**: Informational messages

## Examples

### Monitor Live Trading
```bash
# Start TUI and monitor live trading
ib-box-spread-tui

# Press 'D' to toggle dry-run mode for safety
```

### Development Testing
```bash
# Use mock data for UI development
ib-box-spread-tui --mock
```

### Custom Backend
```bash
# Connect to custom REST API
ib-box-spread-tui --endpoint http://api.example.com:3000/snapshot --interval 2
```

### View Man Page
```bash
man ib-box-spread-tui
```

## Troubleshooting

### Colors Not Displaying
- Check `TERM` environment variable
- Ensure terminal supports 256 colors
- Try: `export TERM=xterm-256color`

### Connection Issues
- Verify REST endpoint is accessible
- Check firewall settings
- Use `--mock` for offline testing

### Performance Issues
- Increase polling interval: `--interval 5`
- Check network latency to endpoint
- Monitor system resources

## Development

### Building
```bash
cd tui
go build -o ib-box-spread-tui ./cmd/tui
```

### Testing
```bash
# Run unit tests
go test -short ./...

# Run integration tests
go test -run TestTUIHelpAndQuit

# Run snapshot tests
go test -run TestTUISnapshot

# Or use test script
../scripts/test_tui.sh
```

### Contributing
See `docs/TUI_TESTING.md` for testing guidelines and `docs/TUI_DESIGN.md` for design documentation.

## Related Documentation

- **Design**: `docs/TUI_DESIGN.md` - TUI design and layout
- **Testing**: `docs/TUI_TESTING.md` - Testing guide
- **Man Page**: `man/ib-box-spread-tui.1` - Unix manual page

## License

See main project LICENSE file.

## Support

For issues and questions:
- Check documentation in `docs/`
- Review keyboard shortcuts with `?` key
- See man page: `man ib-box-spread-tui`
