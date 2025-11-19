# LEAN Broker Adapters Documentation

**Date**: 2025-11-18
**Status**: Research Complete
**Purpose**: Document LEAN broker adapter capabilities, configuration, and integration for IBKR and Alpaca

---

## Overview

This document provides comprehensive information about LEAN's broker adapters for Interactive Brokers (IBKR) and Alpaca, including configuration, capabilities, limitations, and integration requirements.

---

## LEAN Brokerage Architecture

### Brokerage Interface

All LEAN brokerages implement the `IBrokerage` interface with the following key methods:

- `Connect()`: Establish connection to broker
- `Disconnect()`: Close connection
- `PlaceOrder()`: Submit order
- `UpdateOrder()`: Modify existing order
- `CancelOrder()`: Cancel order
- `GetOpenOrders()`: Retrieve pending orders
- `GetAccountHoldings()`: Get current positions

### Brokerage Components

LEAN brokerages consist of 9 key components:

1. **IBrokerageFactory**: Creates brokerage instances
2. **IBrokerage**: Core brokerage application logic
3. **ISymbolMapper**: Translates brokerage tickers to LEAN format
4. **IBrokerageModel**: Describes order support and transaction models
5. **IDataQueueHandler**: Manages streaming market data
6. **IHistoryProvider**: Provides historical data access
7. **IDataDownloader**: Downloads data in LEAN format
8. **IFeeModel**: Defines transaction costs
9. **ISecurityTransactionModel**: Brokerage-specific transaction models

---

## Interactive Brokers (IBKR) Adapter

### Overview

LEAN's Interactive Brokers adapter uses the **TWS API** (same as our current implementation) to connect to TWS or IB Gateway.

**Source Code**: [LEAN IBKR Brokerage](https://github.com/QuantConnect/Lean/tree/master/Brokerages/InteractiveBrokers)

### Capabilities

✅ **Supported Asset Classes:**
- Equities
- Equity Options
- Futures
- Forex
- Bonds
- CFDs

✅ **Order Types:**
- Market Orders
- Limit Orders
- Stop Orders
- Stop-Limit Orders
- **Combo Orders** (multi-leg orders including box spreads)

✅ **Features:**
- Real-time market data
- Historical data
- Paper trading support
- Live trading support
- Position management
- Account information

### Configuration

**LEAN Configuration (`config.json`):**

```json
{
  "job-queue-handler": "QuantConnect.Queues.JobQueue",
  "algorithm-type-name": "BoxSpreadAlgorithm",
  "algorithm-language": "Python",
  "algorithm-location": "Main/box_spread_algorithm.py",
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

**Configuration Parameters:**

| Parameter | Description | Example | Required |
|-----------|-------------|---------|----------|
| `host` | TWS/IB Gateway host | `127.0.0.1` | Yes |
| `port` | TWS/IB Gateway port | `7497` (paper), `7496` (live) | Yes |
| `account` | IB account ID | `DU123456` | Yes |
| `username` | IB username (optional) | `your_username` | No |
| `password` | IB password (optional) | `your_password` | No |
| `trading-mode` | `paper` or `live` | `paper` | Yes |

**Standard Ports:**
- **TWS Paper Trading**: `7497`
- **TWS Live Trading**: `7496`
- **IB Gateway Paper Trading**: `4002`
- **IB Gateway Live Trading**: `4001`

### Setup Requirements

1. **TWS or IB Gateway Running:**
   - TWS/IB Gateway must be running before LEAN connects
   - API must be enabled in TWS Settings > API > Settings
   - "Enable ActiveX and Socket Clients" must be checked
   - IP address must be trusted (127.0.0.1 for local)

2. **Account Configuration:**
   - Paper trading account for testing
   - Live account for production
   - Account ID format: `DU123456` (paper) or `U123456` (live)

3. **Market Data Subscriptions:**
   - Required data subscriptions for options trading
   - Real-time data for live trading
   - Historical data for backtesting

### Integration with Current Codebase

**Current Implementation:**
- `native/src/tws_client.cpp`: Direct TWS API client (C++)
- Uses same TWS API as LEAN adapter
- Port configuration: `7497` (paper), `7496` (live)
- Connection management with auto-reconnect

**LEAN Integration:**
- LEAN adapter uses TWS API (C# implementation)
- Same connection parameters (host, port, account)
- Can reuse existing TWS/IB Gateway setup
- No changes needed to TWS configuration

**Migration Path:**
1. Keep existing TWS client for C++ calculations
2. Use LEAN adapter for order execution and market data
3. Bridge between C++ and LEAN via Python

### Limitations

⚠️ **IB Client Portal API:**
- LEAN does **not** natively support IB Client Portal API
- Only TWS API is supported
- For Client Portal integration, would need:
  - Custom adapter development (complex)
  - Or use TWS adapter (recommended)

⚠️ **Options Trading:**
- Requires appropriate account permissions
- Market data subscriptions required
- Some options may not be available for all symbols

---

## Alpaca Adapter

### Overview

LEAN's Alpaca adapter uses the **REST API** to connect to Alpaca's trading platform.

**Source Code**: [LEAN Alpaca Brokerage](https://github.com/QuantConnect/Lean/tree/master/Brokerages/Alpaca)

### Capabilities

✅ **Supported Asset Classes:**
- US Equities
- **Equity Options** (if available on Alpaca)
- Cryptocurrencies (via Alpaca Crypto)

✅ **Order Types:**
- Market Orders
- Limit Orders
- Stop Orders
- Stop-Limit Orders
- Bracket Orders
- OCO (One-Cancels-Other) Orders

✅ **Features:**
- Real-time market data (via Alpaca WebSocket)
- Historical data
- Paper trading support
- Live trading support
- Position management
- Account information

### Configuration

**LEAN Configuration (`config.json`):**

```json
{
  "job-queue-handler": "QuantConnect.Queues.JobQueue",
  "algorithm-type-name": "BoxSpreadAlgorithm",
  "algorithm-language": "Python",
  "algorithm-location": "Main/box_spread_algorithm.py",
  "brokerage": {
    "brokerage-type": "Alpaca",
    "alpaca": {
      "key-id": "YOUR_API_KEY",
      "secret-key": "YOUR_SECRET_KEY",
      "base-url": "https://paper-api.alpaca.markets",
      "trading-mode": "paper"
    }
  },
  "data-queue-handler": "QuantConnect.Lean.Engine.DataFeeds.BrokerageDataQueueHandler",
  "data-aggregator": "QuantConnect.Lean.Engine.DataFeeds.AggregationManager"
}
```

**Configuration Parameters:**

| Parameter | Description | Example | Required |
|-----------|-------------|---------|----------|
| `key-id` | Alpaca API Key | `PK...` (paper) or `AK...` (live) | Yes |
| `secret-key` | Alpaca Secret Key | `...` | Yes |
| `base-url` | API base URL | `https://paper-api.alpaca.markets` (paper) or `https://api.alpaca.markets` (live) | Yes |
| `trading-mode` | `paper` or `live` | `paper` | Yes |

**API Endpoints:**
- **Paper Trading**: `https://paper-api.alpaca.markets`
- **Live Trading**: `https://api.alpaca.markets`

### Setup Requirements

1. **Alpaca Account:**
   - Create account at [alpaca.markets](https://alpaca.markets)
   - Generate API keys (paper and/or live)
   - Verify account permissions

2. **API Keys:**
   - Paper trading keys start with `PK...`
   - Live trading keys start with `AK...`
   - Store securely (use environment variables or config files)

3. **Options Trading:**
   - Verify Alpaca options trading availability (may vary by region)
   - Check account permissions for options trading
   - Confirm options data availability

### Integration with Current Codebase

**Current Implementation:**
- `python/integration/alpaca_client.py`: Alpaca REST API client (Python)
- `python/integration/alpaca_service.py`: Alpaca service wrapper
- Uses same REST API as LEAN adapter
- API key authentication

**LEAN Integration:**
- LEAN adapter uses Alpaca REST API (C# implementation)
- Same authentication (API key + secret)
- Can reuse existing API keys
- No changes needed to Alpaca account

**Migration Path:**
1. Keep existing Alpaca client for reference
2. Use LEAN adapter for order execution and market data
3. Bridge between C++ and LEAN via Python

### Limitations

⚠️ **Options Trading:**
- Alpaca options trading availability may be limited
- Verify options support for your region
- Some options features may not be available

⚠️ **Rate Limiting:**
- Alpaca has rate limits on API calls
- LEAN adapter handles rate limiting automatically
- Monitor for rate limit errors

⚠️ **Market Data:**
- Real-time data requires appropriate subscriptions
- Historical data may have limitations
- Options data availability varies

---

## IB Client Portal API

### Overview

**LEAN does NOT natively support IB Client Portal API.**

### Options

**Option 1: Use TWS Adapter (Recommended)**
- Use LEAN's existing IBKR TWS adapter
- TWS API provides same functionality
- No custom development needed
- ✅ **Recommended approach**

**Option 2: Custom Adapter Development**
- Create custom LEAN brokerage adapter
- Implement all 9 brokerage components
- Significant development effort (weeks)
- ⚠️ **Complex, not recommended unless necessary**

**Option 3: Hybrid Approach**
- Use TWS adapter for LEAN
- Keep existing Client Portal client for specific features
- Bridge between systems
- ⚠️ **Adds complexity**

### Recommendation

**Use TWS Adapter** for IBKR integration:
- Same functionality as Client Portal
- Already supported by LEAN
- No custom development needed
- Better integration with LEAN ecosystem

---

## Configuration Examples

### IBKR Paper Trading

```json
{
  "brokerage": {
    "brokerage-type": "InteractiveBrokers",
    "interactive-brokers": {
      "host": "127.0.0.1",
      "port": 7497,
      "account": "DU123456",
      "trading-mode": "paper"
    }
  }
}
```

### IBKR Live Trading

```json
{
  "brokerage": {
    "brokerage-type": "InteractiveBrokers",
    "interactive-brokers": {
      "host": "127.0.0.1",
      "port": 7496,
      "account": "U123456",
      "trading-mode": "live"
    }
  }
}
```

### Alpaca Paper Trading

```json
{
  "brokerage": {
    "brokerage-type": "Alpaca",
    "alpaca": {
      "key-id": "PK...",
      "secret-key": "...",
      "base-url": "https://paper-api.alpaca.markets",
      "trading-mode": "paper"
    }
  }
}
```

### Alpaca Live Trading

```json
{
  "brokerage": {
    "brokerage-type": "Alpaca",
    "alpaca": {
      "key-id": "AK...",
      "secret-key": "...",
      "base-url": "https://api.alpaca.markets",
      "trading-mode": "live"
    }
  }
}
```

---

## Integration Requirements

### For IBKR Integration

1. ✅ TWS or IB Gateway running
2. ✅ API enabled in TWS settings
3. ✅ IP address trusted (127.0.0.1)
4. ✅ Account ID configured
5. ✅ Market data subscriptions (for options)

### For Alpaca Integration

1. ✅ Alpaca account created
2. ✅ API keys generated
3. ✅ Account permissions verified
4. ✅ Options trading enabled (if needed)
5. ✅ Market data subscriptions (if needed)

---

## Comparison: Current vs LEAN

| Feature | Current Implementation | LEAN Adapter |
|---------|----------------------|--------------|
| **IBKR TWS** | ✅ C++ direct TWS API | ✅ C# TWS API (via LEAN) |
| **IB Client Portal** | ✅ Python REST API | ❌ Not supported |
| **Alpaca** | ✅ Python REST API | ✅ C# REST API (via LEAN) |
| **Options Trading** | ✅ Supported | ✅ Supported |
| **Combo Orders** | ✅ Supported | ✅ Supported |
| **Multi-Broker** | ⚠️ Custom implementation | ✅ Built-in support |

---

## Next Steps

1. ✅ **Research Complete** (this document)
2. ⏳ **Design LEAN Strategy Architecture** (T-41)
3. ⏳ **Implement Data Conversion Layer** (T-42)
4. ⏳ **Implement LEAN Box Spread Strategy** (T-43)
5. ⏳ **Configure IBKR Integration** (T-45)
6. ⏳ **Configure Alpaca Integration** (T-46)

---

## References

- [LEAN IBKR Integration Guide](https://www.quantconnect.com/docs/v2/lean-cli/live-trading/brokerages/interactive-brokers)
- [LEAN Alpaca Integration Guide](https://www.quantconnect.com/docs/v2/lean-cli/live-trading/brokerages/alpaca)
- [LEAN IBKR Brokerage Source](https://github.com/QuantConnect/Lean/tree/master/Brokerages/InteractiveBrokers)
- [LEAN Alpaca Brokerage Source](https://github.com/QuantConnect/Lean/tree/master/Brokerages/Alpaca)
- [LEAN Brokerage Development Guide](https://gist.github.com/jaredbroad/fcccb18f7c3088e43a54365a45ae5760)
- [LEAN Brokerage Template](https://github.com/QuantConnect/Lean.Brokerages.Template)

---

## Status

- ✅ IBKR TWS adapter research complete
- ✅ Alpaca adapter research complete
- ✅ IB Client Portal status documented
- ✅ Configuration examples created
- ✅ Integration requirements identified
- ✅ Comparison with current implementation
