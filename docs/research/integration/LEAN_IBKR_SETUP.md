# LEAN IBKR Integration Setup Guide

**Date**: 2025-11-18
**Status**: Setup Guide
**Purpose**: Guide for configuring Interactive Brokers (IBKR) integration with LEAN

---

## Overview

This guide covers the configuration and testing of Interactive Brokers (IBKR) integration with LEAN for box spread trading. LEAN uses the TWS API (same as our current implementation) to connect to TWS or IB Gateway.

---

## Prerequisites

1. **TWS or IB Gateway** installed and running
2. **IB Account** (paper trading account recommended for testing)
3. **API Enabled** in TWS/Gateway settings
4. **LEAN CLI** installed (see `docs/LEAN_SETUP.md`)

---

## TWS/IB Gateway Configuration

### 1. Enable API Access

1. Open TWS or IB Gateway
2. Go to **Configure** → **API** → **Settings**
3. Check **"Enable ActiveX and Socket Clients"**
4. Set **"Socket port"** to:
   - **7497** for Paper Trading (TWS)
   - **7496** for Live Trading (TWS)
   - **4002** for Paper Trading (IB Gateway)
   - **4001** for Live Trading (IB Gateway)
5. Add **127.0.0.1** to **"Trusted IPs"** (if required)
6. Click **"OK"** and restart TWS/Gateway

### 2. Verify Connection

Test connection using LEAN CLI:

```bash
source python/venv312/bin/activate
lean live --brokerage InteractiveBrokers --data-provider InteractiveBrokers
```

---

## LEAN Configuration

### Configuration File

Create or update `config/lean_ibkr_config.json`:

```json
{
  "algorithm-type-name": "BoxSpreadAlgorithm",
  "algorithm-language": "Python",
  "algorithm-location": "Main/box_spread_algorithm.py",
  "job-queue-handler": "QuantConnect.Queues.JobQueue",
  "api-handler": "QuantConnect.Api.Api",
  "data-folder": "data/",
  "results-folder": "results/",
  "brokerage": {
    "brokerage-type": "InteractiveBrokers",
    "interactive-brokers": {
      "host": "127.0.0.1",
      "port": 7497,
      "account": "DU123456",
      "username": "",
      "password": "",
      "trading-mode": "paper"
    }
  },
  "data-queue-handler": "QuantConnect.Lean.Engine.DataFeeds.BrokerageDataQueueHandler",
  "data-aggregator": "QuantConnect.Lean.Engine.DataFeeds.AggregationManager"
}
```

### Configuration Parameters

| Parameter | Description | Example | Required |
|-----------|-------------|---------|----------|
| `host` | TWS/IB Gateway host | `127.0.0.1` | Yes |
| `port` | TWS/IB Gateway port | `7497` (paper), `7496` (live) | Yes |
| `account` | IB account ID | `DU123456` (paper), `U123456` (live) | Yes |
| `username` | IB username (optional) | `your_username` | No |
| `password` | IB password (optional) | `your_password` | No |
| `trading-mode` | `paper` or `live` | `paper` | Yes |

### Standard Ports

| Broker | Mode | Port |
|--------|------|------|
| TWS | Paper Trading | 7497 |
| TWS | Live Trading | 7496 |
| IB Gateway | Paper Trading | 4002 |
| IB Gateway | Live Trading | 4001 |

---

## Testing IBKR Integration

### 1. Test Connection

```bash
# Activate LEAN environment
source python/venv312/bin/activate

# Test connection (will use config.json)
lean live --brokerage InteractiveBrokers
```

### 2. Test Market Data

Create a simple test algorithm to verify market data:

```python
# Main/test_ibkr_connection.py
from AlgorithmImports import *

class TestIBKRConnection(QCAlgorithm):
    def Initialize(self):
        self.SetStartDate(2025, 1, 1)
        self.SetCash(100000)

        # Subscribe to SPY options
        option = self.AddOption("SPY")
        option.SetFilter(lambda u: u.Strikes(-5, +5).Expiration(0, 30))

    def OnData(self, slice):
        option_chain = slice.OptionChains.get("SPY", None)
        if option_chain:
            self.Log(f"Received {len(option_chain)} option contracts")
            if len(option_chain) > 0:
                contract = option_chain[0]
                self.Log(f"Sample contract: {contract.Symbol} - Bid: {contract.BidPrice}, Ask: {contract.AskPrice}")
```

Run test:

```bash
lean backtest Main/test_ibkr_connection.py
```

### 3. Test Order Placement (Paper Trading)

```python
# Main/test_ibkr_orders.py
from AlgorithmImports import *

class TestIBKROrders(QCAlgorithm):
    def Initialize(self):
        self.SetStartDate(2025, 1, 1)
        self.SetCash(100000)

        # Subscribe to SPY
        self.AddEquity("SPY")
        self.order_placed = False

    def OnData(self, slice):
        if not self.order_placed and slice.ContainsKey("SPY"):
            # Place a simple market order
            self.MarketOrder("SPY", 10)
            self.order_placed = True
            self.Log("Market order placed for SPY")

    def OnOrderEvent(self, orderEvent):
        self.Log(f"Order event: {orderEvent.Status} - {orderEvent.Message}")
```

---

## Troubleshooting

### Connection Errors

**Error: "Connection rejected" (502)**
- Ensure TWS/Gateway is running
- Check API is enabled in TWS Settings
- Verify port number matches configuration
- Check IP address is trusted (127.0.0.1)

**Error: "Not connected" (504)**
- Verify TWS/Gateway is running
- Check API port is correct
- Restart TWS/Gateway

**Error: "Connection lost" (1100)**
- Check internet connection
- Verify TWS/Gateway is still running
- Check IB network status

### Market Data Errors

**Error: "No security definition found" (200)**
- Verify symbol is correct
- Check market data subscriptions
- Ensure market is open

**Error: "Requested market data is not subscribed" (10167)**
- Check IB account has required data subscriptions
- Verify market data permissions
- Some options may require additional subscriptions

### Order Errors

**Error: "Order rejected - invalid contract" (201)**
- Verify contract details (symbol, expiry, strike, right)
- Check contract is valid for trading
- Ensure market is open

**Error: "Order size exceeds account limits" (10148)**
- Check account buying power
- Verify position limits
- Reduce order size

---

## Integration with Box Spread Strategy

The box spread strategy uses LEAN's IBKR adapter automatically when configured. No additional code changes are needed - the strategy will:

1. Connect to TWS/IB Gateway via LEAN
2. Receive market data for options
3. Execute combo orders (box spreads) via LEAN
4. Track positions and P&L

---

## Next Steps

1. ✅ **IBKR Configuration Complete** (this guide)
2. ⏳ **Test with Box Spread Strategy** (T-47)
3. ⏳ **Configure Alpaca Integration** (T-45)
4. ⏳ **End-to-End Testing** (T-47)

---

## References

- [LEAN IBKR Integration Guide](https://www.quantconnect.com/docs/v2/lean-cli/live-trading/brokerages/interactive-brokers)
- [LEAN Broker Adapters Documentation](docs/LEAN_BROKER_ADAPTERS.md)
- [TWS API Documentation](https://interactivebrokers.github.io/tws-api/)

---

## Status

- ✅ Configuration guide created
- ✅ Test procedures documented
- ✅ Troubleshooting guide included
- ⏳ Ready for testing
