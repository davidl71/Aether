# TUI Usage Guide

Complete guide to using the IBKR Box Spread Terminal User Interface.

## Getting Started

### First Launch

1. **Start the TUI**:
   ```bash
   ib-box-spread-tui
   ```

2. **Check System Status**: Look at the header for status indicators:
   - `TWS: OK` - TWS connection active
   - `ORATS: Enabled` - Market data provider ready
   - `Portal: OK` - IBKR Portal API connected
   - `QuestDB: OK` - Historical data available

3. **Verify Mode**: Check for `DRY-RUN` or `LIVE` indicator
   - Press `D` to toggle between modes
   - Always start in DRY-RUN for safety

### Understanding the Layout

```
┌──────────────────────────────────────────────────────────────┐
│ IB BOX SPREAD TERMINAL                Time: 10:24:31        │ ← Header
│ Mode: DRY-RUN │ Strategy: RUNNING │ Account: DU123456        │
│ TWS: OK │ ORATS: Enabled │ Portal: OK │ QuestDB: OK           │
├──────── Tabs ────────────────────────────────────────────────┤ ← Tab Bar
│ Dashboard │ Current Positions │ Historic Positions │ Orders │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  (Main Content Area)                                        │
│                                                              │
├───────────────────────┬──────────────────────────────────────┤
│ Active Positions      │ Recent Orders                        │ ← Panels
└───────────────────────┴──────────────────────────────────────┘
│ Alerts                                                    ↑ │ ← Footer
└──────────────────────────────────────────────────────────────┘
```

## Using Each Tab

### Dashboard Tab

**Purpose**: Main overview of system status

**What to Look For**:
- **Account Metrics**: Net liquidation, buying power, margin requirements
- **Symbol Performance**: ROI percentages, price movements
- **Active Positions**: Number and total value
- **System Health**: All status indicators should be green/OK

**Actions**:
- Press `Enter` on a symbol to see details
- Press `Enter` on a position to see full position info
- Monitor alerts at the bottom

### Current Positions Tab

**Purpose**: Detailed view of open box spread positions

**Key Metrics**:
- **Strike Width**: Difference between strikes (determines profit)
- **Net Debit**: Total cost to enter position
- **Theoretical Value**: Should equal strike width
- **Arbitrage Profit**: Theoretical value - net debit
- **ROI %**: Profit as percentage of investment
- **APR %**: Annualized return percentage
- **Days to Expiry**: Time remaining

**Actions**:
- Press `B` on a position to buy additional combo (lower cost basis)
- Press `Shift+S` on a position to sell combo (realize gains)
- Press `Enter` for full position details

**Understanding the Metrics**:
- **Green ROI**: Profitable position
- **Red ROI**: Losing position (shouldn't happen with box spreads)
- **APR vs Benchmark**: Compare to Treasury Bill rates
- **Maker/Taker Counts**: Track order routing

### Historic Positions Tab

**Purpose**: Review closed positions and performance

**Information Shown**:
- Closed position details
- Entry and exit prices
- Realized profit/loss
- Execution timestamps
- Performance metrics

**Use Cases**:
- Review trading history
- Analyze strategy performance
- Identify patterns
- Track realized gains

### Orders Tab

**Purpose**: Monitor order status and execution history

**Order States**:
- **Pending**: Order submitted, awaiting confirmation
- **Submitted**: Order confirmed by broker
- **Filled**: Order executed
- **Partially Filled**: Partial execution
- **Cancelled**: Order cancelled
- **Rejected**: Order rejected by broker

**Actions**:
- Press `Enter` on order for execution details
- Press `K` to cancel pending orders
- Monitor fill prices and execution quality

### Alerts Tab

**Purpose**: System notifications and warnings

**Alert Types**:
- **Info** (Blue): Informational messages
- **Success** (Green): Successful operations
- **Warning** (Yellow): Warnings requiring attention
- **Error** (Red): Errors requiring immediate action

**Common Alerts**:
- Connection status changes
- Strategy start/stop notifications
- Risk limit warnings
- Order execution confirmations
- System errors

## Common Workflows

### Starting a Trading Session

1. **Launch TUI**: `ib-box-spread-tui`
2. **Verify Connections**: Check all status indicators are OK
3. **Enable Dry-Run**: Press `D` (should show DRY-RUN mode)
4. **Start Strategy**: Press `S` to start strategy
5. **Monitor Dashboard**: Watch for opportunities and positions
6. **Review Alerts**: Check for any warnings or errors

### Monitoring Active Positions

1. **Switch to Positions Tab**: Press `Tab` until "Current Positions" is selected
2. **Review Metrics**: Check ROI, APR, days to expiry
3. **Check Profitability**: Ensure all positions show positive ROI
4. **Monitor Expiration**: Watch days to expiry countdown
5. **Consider Actions**: Use `B` or `Shift+S` to manage positions

### Managing Orders

1. **Switch to Orders Tab**: Press `Tab` until "Orders" is selected
2. **Check Status**: Review pending and filled orders
3. **Monitor Fills**: Verify execution prices match expectations
4. **Cancel if Needed**: Press `K` to cancel all pending orders
5. **Review History**: Scroll through past orders

### Responding to Alerts

1. **Check Alert Tab**: Press `Tab` until "Alerts" is selected
2. **Review Severity**: Color indicates severity (yellow=warning, red=error)
3. **Read Details**: Press `Enter` on alert for full information
4. **Take Action**: Follow alert instructions
5. **Monitor Resolution**: Watch for follow-up alerts

## Best Practices

### Safety

1. **Always Start in Dry-Run**: Press `D` to enable dry-run mode
2. **Verify Connections**: Ensure all systems show OK before trading
3. **Monitor Alerts**: Check alerts tab regularly
4. **Review Positions**: Regularly review open positions
5. **Check Orders**: Verify order execution quality

### Performance

1. **Use Appropriate Polling**: Adjust `--interval` if needed
2. **Monitor Network**: Ensure stable connection to backend
3. **Check Resources**: Monitor system CPU/memory usage
4. **Review Logs**: Check logs if issues occur

### Trading

1. **Understand Metrics**: Know what ROI, APR, and other metrics mean
2. **Monitor Expiration**: Track days to expiry for positions
3. **Review Benchmarks**: Compare APR to Treasury Bill rates
4. **Track Performance**: Review historic positions regularly
5. **Manage Risk**: Monitor margin requirements and exposure

## Troubleshooting

### No Data Displayed

- **Check Backend**: Verify REST endpoint is running
- **Check Connection**: Use `--mock` to test with mock data
- **Check Endpoint**: Verify `--endpoint` URL is correct
- **Check Logs**: Review application logs for errors

### Colors Not Working

- **Check Terminal**: Ensure terminal supports colors
- **Set TERM**: `export TERM=xterm-256color`
- **Check Config**: Verify color support in terminal settings

### Slow Updates

- **Increase Interval**: Use `--interval 5` for slower updates
- **Check Network**: Verify network latency to endpoint
- **Check Backend**: Ensure backend is responding quickly

### Connection Issues

- **Verify Endpoint**: Check REST API is accessible
- **Check Firewall**: Ensure firewall allows connections
- **Use Mock**: Test with `--mock` to isolate issues
- **Review Logs**: Check for connection error messages

## Advanced Usage

### Custom Configuration

Create a configuration file (if supported):
```json
{
  "endpoint": "http://localhost:8080/api/snapshot",
  "interval": 1,
  "provider": "rest"
}
```

### Integration with Other Tools

- **Logs**: Redirect output to log file
- **Monitoring**: Use with system monitoring tools
- **Automation**: Integrate with trading scripts

## See Also

- **Keyboard Shortcuts**: `docs/KEYBOARD_SHORTCUTS.md`
- **Man Page**: `man ib-box-spread-tui`
- **Design**: `docs/TUI_DESIGN.md`
- **Testing**: `docs/TUI_TESTING.md`
