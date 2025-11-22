# LEAN Alpaca Integration Setup Guide

**Date**: 2025-11-18
**Status**: Setup Guide
**Purpose**: Guide for configuring Alpaca integration with LEAN

---

## Overview

This guide covers the configuration and testing of Alpaca integration with LEAN for box spread trading. LEAN uses Alpaca's REST API to connect to Alpaca's trading platform.

---

## Prerequisites

1. **Alpaca Account** created at [alpaca.markets](https://alpaca.markets)
2. **API Keys** generated (paper and/or live)
3. **Account Permissions** verified (options trading if available)
4. **LEAN CLI** installed (see `docs/LEAN_SETUP.md`)

---

## Alpaca Account Setup

### 1. Create Account

1. Go to [alpaca.markets](https://alpaca.markets)
2. Sign up for an account
3. Complete account verification

### 2. Generate API Keys

1. Log in to Alpaca dashboard
2. Go to **"Your API Keys"** section
3. Generate **Paper Trading** keys (for testing)
4. Generate **Live Trading** keys (for production)
5. **Save keys securely** (they won't be shown again)

**Key Format:**

- Paper trading keys start with `PK...`
- Live trading keys start with `AK...`

### 3. Verify Options Trading

1. Check account permissions for options trading
2. Verify options data availability
3. Note: Options trading may have regional limitations

---

## LEAN Configuration

### Configuration File

Create or update `config/lean_alpaca_config.json`:

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
    "brokerage-type": "Alpaca",
    "alpaca": {
      "key-id": "PK...",
      "secret-key": "YOUR_SECRET_KEY",
      "base-url": "https://paper-api.alpaca.markets",
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
| `key-id` | Alpaca API Key | `PK...` (paper), `AK...` (live) | Yes |
| `secret-key` | Alpaca Secret Key | `...` | Yes |
| `base-url` | API base URL | `https://paper-api.alpaca.markets` (paper) or `https://api.alpaca.markets` (live) | Yes |
| `trading-mode` | `paper` or `live` | `paper` | Yes |

### API Endpoints

| Mode | Base URL |
|------|----------|
| Paper Trading | `https://paper-api.alpaca.markets` |
| Live Trading | `https://api.alpaca.markets` |

---

## Testing Alpaca Integration

### 1. Test Connection

```bash
# Activate LEAN environment
source python/venv312/bin/activate

# Test connection (will use config.json)
lean live --brokerage Alpaca
```

### 2. Test Market Data

Create a simple test algorithm:

```python
# Main/test_alpaca_connection.py
from AlgorithmImports import *

class TestAlpacaConnection(QCAlgorithm):
    def Initialize(self):
        self.SetStartDate(2025, 1, 1)
        self.SetCash(100000)

        # Subscribe to SPY
        self.AddEquity("SPY")

    def OnData(self, slice):
        if slice.ContainsKey("SPY"):
            spy = slice["SPY"]
            self.Log(f"SPY Price: {spy.Price}, Bid: {spy.BidPrice}, Ask: {spy.AskPrice}")
```

Run test:

```bash
lean backtest Main/test_alpaca_connection.py
```

### 3. Test Order Placement (Paper Trading)

```python
# Main/test_alpaca_orders.py
from AlgorithmImports import *

class TestAlpacaOrders(QCAlgorithm):
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

## Options Trading Considerations

### Availability

⚠️ **Important**: Alpaca options trading availability may vary by region and account type. Verify:

1. **Account Type**: Some account types may not support options
2. **Regional Availability**: Options may not be available in all regions
3. **Data Access**: Options data may require additional subscriptions

### Verification

Check Alpaca documentation and account dashboard for:

- Options trading permissions
- Available option symbols
- Data feed requirements

---

## Troubleshooting

### Authentication Errors

**Error: "Invalid API key"**

- Verify API key is correct
- Check key format (PK... for paper, AK... for live)
- Ensure secret key matches API key
- Regenerate keys if needed

**Error: "Unauthorized"**

- Check API keys are active
- Verify account status
- Check rate limits

### Connection Errors

**Error: "Connection timeout"**

- Check internet connection
- Verify API endpoint URL is correct
- Check firewall settings

**Error: "Rate limit exceeded"**

- Alpaca has rate limits on API calls
- LEAN adapter handles rate limiting automatically
- Reduce request frequency if needed

### Market Data Errors

**Error: "Symbol not found"**

- Verify symbol is correct
- Check symbol is available on Alpaca
- Some symbols may not be available for options

**Error: "Options data not available"**

- Check account has options trading enabled
- Verify options data subscriptions
- Some options may require additional permissions

### Order Errors

**Error: "Insufficient buying power"**

- Check account cash balance
- Verify buying power
- Reduce order size

**Error: "Order rejected"**

- Check order parameters
- Verify market is open
- Check account permissions

---

## Integration with Box Spread Strategy

The box spread strategy uses LEAN's Alpaca adapter automatically when configured. The strategy will:

1. Connect to Alpaca API via LEAN
2. Receive market data for options (if available)
3. Execute combo orders (box spreads) via LEAN
4. Track positions and P&L

**Note**: Options trading support on Alpaca may be limited. Verify availability before relying on Alpaca for box spread trading.

---

## Next Steps

1. ✅ **Alpaca Configuration Complete** (this guide)
2. ⏳ **Test with Box Spread Strategy** (T-47)
3. ⏳ **Verify Options Availability** (if using Alpaca for options)
4. ⏳ **End-to-End Testing** (T-47)

---

## References

- [LEAN Alpaca Integration Guide](https://www.quantconnect.com/docs/v2/lean-cli/live-trading/brokerages/alpaca)
- [LEAN Broker Adapters Documentation](docs/LEAN_BROKER_ADAPTERS.md)
- [Alpaca API Documentation](https://alpaca.markets/docs/)

---

## Status

- ✅ Configuration guide created
- ✅ Test procedures documented
- ✅ Troubleshooting guide included
- ⚠️ Options availability verification needed
- ⏳ Ready for testing
