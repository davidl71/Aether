# Brokeree Solutions - Turnkey Solutions for Brokers

**Date**: 2025-01-27
**Source**: <https://brokeree.com/solutions/>
**Provider**: Brokeree Solutions - Technology provider for retail brokers

---

## Overview

Brokeree Solutions provides turnkey technology solutions for retail forex and CFD brokers, primarily focused on MetaTrader 4/5 (MT4/MT5) and cTrader platforms. The company offers liquidity bridges, PAMM systems, social trading, prop trading solutions, and various plugins for broker infrastructure management.

**Note**: This platform is primarily designed for retail forex/CFD brokers using MetaTrader platforms, which is different from the institutional options trading focus of this project. However, some components may have indirect relevance.

---

## Key Solutions

### 1. Liquidity Bridge

**Description**: Multi-server liquidity bridge combining smart liquidity aggregation, flexible order execution, and risk management.

**Key Features**:
- **Multi-Server Support**: Connect multiple MetaTrader servers
- **Smart Quotes Aggregation**: Aggregate quotes from multiple liquidity providers
- **Extensive LP Integrations**: Connect to many liquidity providers
- **Web-Based Control Panel**: Remote configuration and monitoring
- **Market Depth**: Assess quality of incoming market data
- **Configurable Routing**: Hedge client trades or process in-house

**Platforms**: MetaTrader 4, MetaTrader 5, DXtrade, cTrader

**Relevance**: Limited - primarily for retail forex/CFD brokers, not directly applicable to options trading on CBOE.

---

### 2. TradingView API

**Description**: REST API application that connects TradingView API to brokerage trading platforms and executes requests from end-users.

**Key Features**:
- **TradingView Integration**: Connect TradingView to trading platforms
- **Order Execution**: Open orders directly from TradingView charts
- **Price Streaming**: Real-time price data streaming
- **History**: Historical data access
- **Connected Endpoints**: Trading, price streaming, history

**Platforms**: DXtrade, cTrader

**Relevance**: Moderate - TradingView is a popular charting platform that could be used for options analysis, though this specific integration is for retail brokers.

---

### 3. MT4/MT5 FIX API

**Description**: FIX API integration for MetaTrader platforms.

**Key Features**:
- **FIX Protocol**: Industry-standard FIX protocol support
- **MetaTrader Integration**: Connect MT4/MT5 to FIX-based systems
- **Order Routing**: Route orders via FIX protocol
- **Market Data**: FIX-based market data feeds

**Platforms**: MetaTrader 4, MetaTrader 5

**Relevance**: Moderate - FIX protocol is relevant for direct exchange access, though this is specifically for MetaTrader integration (not directly applicable to C++ project).

---

### 4. MT4/MT5 REST API

**Description**: REST API for MetaTrader platforms.

**Key Features**:
- **REST API**: HTTP-based API for MetaTrader
- **Platform Integration**: Integrate external systems with MT4/MT5
- **Order Management**: Manage orders via REST API
- **Account Management**: Access account information

**Platforms**: MetaTrader 4, MetaTrader 5

**Relevance**: Limited - REST API for MetaTrader, not directly applicable to options trading on CBOE.

---

### 5. MT5 Gateways

**Description**: Gateways connecting MetaTrader 5 to various liquidity providers and brokers.

**Supported Gateways**:
- **MT5 Gateway to AC Markets (Europe)**: Stream quotes and execute orders
- **MT5 Gateway to DASTrader**: Direct Access Software for US exchanges (CBOE/BATS/EDGE, CBSX, Nasdaq, AMEX/NYSE/ARCA, OTC)
- **MT5 Gateway to LMAX**: LMAX Exchange connectivity
- **MT5 Gateway to Exante**: EXANTE multi-asset services (10,000+ instruments including options)
- **MT5 Gateway to SAXO Bank**: Saxo Bank connectivity (40,000+ instruments)

**Relevance**:
- **DASTrader Gateway**: Potentially relevant - provides direct market access to US exchanges including CBOE
- **Exante Gateway**: Potentially relevant - offers options trading
- **Others**: Limited relevance for options trading

---

### 6. MT5 Gateway to DASTrader (Potentially Relevant)

**Description**: Gateway to Direct Access Software (DAS) for low-latency order validation to US exchanges.

**Key Features**:
- **US Exchange Access**: CBOE/BATS/EDGE, CBSX, Nasdaq, AMEX/NYSE/ARCA, OTC
- **Direct Market Access**: Direct market access to major exchanges
- **Service Bureau FIX**: FIX connectivity to exchanges
- **Low Latency**: Optimized for low-latency order validation
- **Data Centers**: Direct pipes to NY Equinox data centers

**DAS (Direct Access Software)**:
- Connectivity provider for US exchanges
- Direct market access and Service Bureau FIX connectivity
- Maintains direct pipes to Nasdaq and NY Equinox data centers

**Relevance**: **Moderate** - DASTrader provides direct access to CBOE, which is relevant for box spread trading. However, this gateway is specifically for MetaTrader 5 integration, not direct C++ integration.

**Use Case**: If using MetaTrader 5 as a trading platform, this gateway could provide CBOE access. However, the project uses C++ with TWS API, so direct integration would be more appropriate.

---

### 7. MT5 Gateway to Exante (Potentially Relevant)

**Description**: Gateway to EXANTE for multi-asset trading including options.

**Key Features**:
- **Options Trading**: EXANTE offers extensive options portfolio
- **Multi-Asset**: 10,000+ stocks, ETFs, currencies, metals, futures, options
- **Global Markets**: US, EU, Asia-Pacific access
- **Automated Symbol Sync**: Option symbols settings synchronization
- **Quote Streaming**: Stream quotes from EXANTE
- **Order Execution**: Execute orders directly at EXANTE

**EXANTE**:
- International investment services company
- Global multi-asset financial services
- Direct access to US, EU, Asia-Pacific markets
- Extensive options portfolio

**Relevance**: **Moderate** - EXANTE offers options trading, which could be relevant. However, this is a MetaTrader 5 gateway, not direct API access.

---

### 8. Other Solutions

**PAMM (Percent Allocation Management Module)**:
- Investment system for money managers and investors
- Not directly relevant to box spread trading

**Social Trading**:
- Copy trading solution
- Not directly relevant to box spread trading

**Prop Pulse**:
- Account management for prop trading firms
- Not directly relevant to box spread trading

**Various Plugins**:
- Dynamic Margin & Leverage
- Swap Manager
- Margin-Credit Tracker
- Trade Copier
- Exposure Manager
- And many more...

**Relevance**: Limited - primarily for retail forex/CFD broker infrastructure management.

---

## Relevance to Box Spread Trading

### Direct Relevance: Limited

**Primary Focus**: Brokeree Solutions is designed for retail forex/CFD brokers using MetaTrader platforms, which is quite different from institutional options trading on CBOE.

**Key Differences**:
- **Platform**: MetaTrader 4/5 vs. C++ trading system
- **Asset Class**: Forex/CFD vs. Options
- **Market**: Retail broker infrastructure vs. Direct exchange access
- **Target Audience**: Retail brokers vs. Institutional/algorithmic traders

### Indirect Relevance: Moderate

**Potentially Useful Components**:

1. **MT5 Gateway to DASTrader**:
   - Provides access to CBOE via Direct Access Software
   - However, requires MetaTrader 5 platform
   - Direct DAS integration would be more appropriate for C++ project

2. **MT5 Gateway to Exante**:
   - EXANTE offers options trading
   - However, requires MetaTrader 5 platform
   - Direct EXANTE API would be more appropriate

3. **TradingView API**:
   - TradingView is useful for options analysis
   - However, this integration is for retail brokers
   - Direct TradingView API access would be more appropriate

4. **FIX API**:
   - FIX protocol is relevant for direct exchange access
   - However, this is MetaTrader-specific
   - Direct FIX implementation (OnixS, TFB) would be more appropriate

---

## Comparison with Project Requirements

### Current Project Architecture

**Technology Stack**:
- **Language**: C++ (native)
- **Broker API**: TWS API (Interactive Brokers)
- **Platform**: Custom trading system
- **Asset Class**: Options (SPX/SPXW box spreads)
- **Market**: Direct CBOE access (via broker or direct)

### Brokeree Solutions Architecture

**Technology Stack**:
- **Language**: MetaTrader MQL4/MQL5, plugins
- **Broker API**: MetaTrader platform APIs
- **Platform**: MetaTrader 4/5, cTrader
- **Asset Class**: Forex, CFDs, some options
- **Market**: Retail broker infrastructure

**Conclusion**: Significant architectural mismatch. Brokeree Solutions is not directly applicable to the project's C++ options trading system.

---

## Alternative Approaches

### For Direct CBOE Access

**Better Options**:
1. **OnixS directConnect**: C++ SDKs for direct CBOE access (CFE BOE, Multicast PITCH)
2. **TFB FIX API**: FIX platform for direct exchange access
3. **Direct FIX Implementation**: Implement FIX protocol directly (OnixS FIX Engine)
4. **TWS API**: Current solution - broker API with CBOE access

### For Options Trading

**Better Options**:
1. **Interactive Brokers (TWS API)**: Current solution - comprehensive options support
2. **Alpaca Markets**: REST API with options support
3. **Direct CBOE Access**: OnixS directConnect for native CBOE connectivity

### For Market Data

**Better Options**:
1. **TWS API Market Data**: Current solution
2. **CBOE Market Data**: Direct CBOE feeds (OPRA, Multicast PITCH)
3. **ORATS API**: Options-specific market data
4. **LiveVol API**: CBOE options data

---

## Resources

### Official Resources

- **Brokeree Solutions**: <https://brokeree.com/solutions/>
- **Liquidity Bridge**: <https://brokeree.com/solutions/liquidity-bridge/>
- **TradingView API**: <https://brokeree.com/solutions/tradingview-api/>
- **Contact**: <[email protected]>
- **Phone**: +372 602 71 22 (Estonia), +357 25 011886 (Cyprus)

### Related Documentation

- **OnixS directConnect**: `docs/ONIXS_DIRECTCONNECT.md` - Direct CBOE access SDKs
- **TFB FIX API**: `docs/TOOLS_FOR_BROKERS_FIX_API.md` - FIX platform for direct access
- **TWS API**: `docs/API_DOCUMENTATION_INDEX.md` - Interactive Brokers API
- **Alpaca API**: `docs/API_DOCUMENTATION_INDEX.md` - Alpaca Markets API

---

## Key Takeaways

1. **Primary Focus**: Retail forex/CFD brokers using MetaTrader platforms
2. **Limited Direct Relevance**: Not designed for institutional options trading
3. **Architectural Mismatch**: MetaTrader-based vs. C++ native system
4. **Potentially Useful**: DASTrader gateway provides CBOE access, but requires MT5
5. **Better Alternatives**: OnixS directConnect, TFB FIX API, or direct FIX implementation
6. **Recommendation**: Not recommended for this project - use alternatives better suited for C++ options trading

---

## Recommendation

**For Box Spread Trading**: Brokeree Solutions is **not recommended** for this project because:

1. **Platform Mismatch**: Designed for MetaTrader, not C++ native systems
2. **Asset Class Focus**: Primarily forex/CFD, not options-focused
3. **Retail Broker Focus**: Infrastructure for retail brokers, not institutional trading
4. **Better Alternatives Available**: OnixS directConnect, TFB FIX API, or direct FIX implementation

**If Considering MetaTrader Integration** (not recommended for this project):
- MT5 Gateway to DASTrader could provide CBOE access
- Would require significant architecture changes
- Less efficient than direct C++ integration

**Recommended Alternatives**:
- **OnixS directConnect**: C++ SDKs for direct CBOE access
- **TFB FIX API**: FIX platform for direct exchange access
- **Current TWS API**: Continue using IBKR for options trading
- **Alpaca API**: Alternative broker API with options support

---

## Related Documentation

- **OnixS directConnect**: `docs/ONIXS_DIRECTCONNECT.md` - Direct CBOE access (recommended)
- **TFB FIX API**: `docs/TOOLS_FOR_BROKERS_FIX_API.md` - FIX platform (recommended)
- **TWS API**: `docs/API_DOCUMENTATION_INDEX.md` - Current broker API
- **Alpaca API**: `docs/API_DOCUMENTATION_INDEX.md` - Alternative broker API

---

**Note**: Brokeree Solutions is primarily designed for retail forex/CFD brokers using MetaTrader platforms. While some components (DASTrader gateway, Exante gateway) provide access to options markets, the platform is not well-suited for institutional C++ options trading systems. For box spread trading, OnixS directConnect, TFB FIX API, or direct FIX implementation would be more appropriate alternatives.
