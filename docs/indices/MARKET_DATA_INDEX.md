# Market Data Providers Index

<!--
@index: market-data
@category: market-data
@tags: market-data, options, analytics, c++, fix-api, rest-api
@last-updated: 2025-01-27
-->

**Purpose**: Focused index of all market data providers for quick reference.

**Full Documentation**: See `API_DOCUMENTATION_INDEX.md` for complete details.

---

## Quick Comparison

| Provider | Focus | API Types | Options Analytics | C++ Support | Best For |
|----------|-------|-----------|-------------------|-------------|----------|
| **dxFeed** | Multi-asset | FIX, C++, Java, Python, REST | ✅ Greeks, IV | ✅ Native C++ | C++ integration, FIX protocol |
| **ORATS** | Options | REST API | ✅ Extensive | ❌ | Options-specific analytics |
| **Massive.com** | Historical | REST, WebSocket | ⚠️ Limited | ❌ | Historical data, backtesting |
| **Alpha Vantage** | Multi-asset | REST, MCP | ⚠️ Basic | ❌ | Free tier, technical indicators |
| **Finnhub** | Multi-asset | REST, WebSocket | ⚠️ Basic | ❌ | Generous free tier, fundamentals |
| **OpenBB** | Financial Data | API | ⚠️ Unknown | ❌ | Financial analytics platform |

---

## Decision Tree

### Which Market Data Provider?

```
Need options analytics?
  → Yes → ORATS (REST) or dxFeed (FIX/C++)
  → No → Continue...

Need C++ native integration?
  → Yes → dxFeed (C++ APIs)
  → No → Continue...

Need FIX protocol?
  → Yes → dxFeed (FIX API)
  → No → REST APIs (ORATS, Alpha Vantage, Finnhub)

Need free tier?
  → Yes → Alpha Vantage (5 calls/min) or Finnhub (60 calls/min)
  → No → dxFeed or ORATS (paid)

Need historical data?
  → Yes → Massive.com or dxFeed
  → No → Real-time providers
```

---

## Provider Details

### dxFeed
- **Best For**: C++ integration, FIX protocol, options analytics
- **Key Features**: Native C++ APIs, FIX API, Greeks, IV, multi-asset
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#dxfeed`

### ORATS
- **Best For**: Options-specific analytics, liquidity scores
- **Key Features**: Extensive options analytics, IV rank, earnings calendar
- **Documentation**: `../API_DOCUMENTATION_INDEX.md` (search for ORATS)

### Massive.com
- **Best For**: Historical data, backtesting
- **Key Features**: Historical and real-time data, S3-compatible interface
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#massive`

### Alpha Vantage
- **Best For**: Free tier, technical indicators
- **Key Features**: 60+ technical indicators, MCP server support
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#alpha-vantage`

### Finnhub
- **Best For**: Generous free tier, fundamentals
- **Key Features**: 60 calls/min free tier, AI-powered sentiment analysis
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#finnhub`

### OpenBB
- **Best For**: Financial analytics platform
- **Key Features**: Financial data and analytics
- **Documentation**: `../API_DOCUMENTATION_INDEX.md#openbb`

---

## Integration Considerations

### C++ Integration
- **dxFeed**: Native C++ APIs available
- **Others**: REST API only (use HTTP client libraries)

### FIX Protocol
- **dxFeed**: FIX 4.4 protocol support
- **Others**: REST API only

### Options Analytics
- **dxFeed**: Pre-calculated Greeks and IV
- **ORATS**: Extensive options analytics including liquidity scores
- **Others**: Basic options data only

### Free Tier
- **Alpha Vantage**: 5 calls/min, 500 calls/day
- **Finnhub**: 60 calls/min (more generous)
- **Others**: Paid subscriptions required

---

## Use Cases

### Box Spread Trading
- **Primary**: dxFeed (C++ APIs, options analytics)
- **Secondary**: ORATS (options-specific analytics)
- **Validation**: Alpha Vantage or Finnhub (free tier for cross-validation)

### Backtesting
- **Historical Data**: Massive.com or dxFeed
- **Options Data**: ORATS or dxFeed

### Real-Time Trading
- **C++ Integration**: dxFeed
- **REST API**: ORATS, Alpha Vantage, or Finnhub

---

## See Also

- **Full Documentation**: `../API_DOCUMENTATION_INDEX.md#market-data-providers`
- **Summary**: `../API_DOCUMENTATION_SUMMARY.md`
- **NotebookLM Suggestions**: `../NOTEBOOKLM_API_DOCUMENTATION_SUGGESTIONS.md`
