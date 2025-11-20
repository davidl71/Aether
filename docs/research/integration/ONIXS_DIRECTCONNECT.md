# OnixS directConnect - Ultra Low Latency Direct Market Access SDKs

**Date**: 2025-01-27
**Source**: <https://www.onixs.biz/directconnect.html>
**Provider**: OnixS - Financial technology solutions

---

## Overview

OnixS directConnect provides ultra-low latency Direct Market Access (DMA) SDKs for a wide range of exchanges and liquidity pools. The platform offers multi-platform implementations of Market Data Handlers, Order Routing, and post-trade DropCopy/Trade Capture solutions, designed for high-performance algorithmic trading.

---

## Key Features

### 1. Ultra-Low Latency DMA Solutions

**Performance Focus**:
- Designed for lowest latency, highest performance APIs
- Supports both FIX-based and native binary protocols
- Optimized for automated trading strategies
- Calibrated and pre-certified for exchange connectivity

**Protocol Support**:
- **FIX-Based**: Implemented using OnixS ultra-low latency FIX Engine
- **Native Binary**: Alternative APIs for low latency, high throughput
- **Venue-Specific**: Custom implementations for each exchange's protocol

### 2. Multi-Platform Support

**Platforms**:
- C++ FIX Engine
- .NET Core/.NET FIX Engine
- .NET Framework / C# FIX Engine
- Java FIX Engine
- Cross-platform compatibility

### 3. Comprehensive Exchange Coverage

**Supported Venues** (relevant to box spread trading):

#### CBOE (Chicago Board Options Exchange)

**Cboe CFE (Cboe Futures Exchange)**:
- **CFE Binary Order Entry (BOE)**: Binary protocol for order entry
- **CFE FIX Order Entry & FIX Drop**: FIX-based order routing
- **CFE Multicast PITCH**: Market data feed handler

**Cboe Europe**:
- Cboe EU BIDS
- Cboe EU BXE, CXE, TRF and SI

**Relevance**: Direct access to CBOE for SPX/SPXW options trading

#### CME Group

**CME Globex**:
- CME iLink 3 Binary Order Entry
- CME MDP Conflated TCP feed Handler SDK
- CME MDP Conflated UDP feed Handler SDK
- CME MDP Premium Market Data SDK
- CME SBE Streamlined Handler SDK
- CME STP API
- CME Audit Trail Generator
- CME Drop Copy Handler

**CME BrokerTec**: Fixed income trading
**CME EBS Markets**: FX trading

**Relevance**: Interest rate futures hedging (SOFR, Eurodollar)

#### Other Relevant Venues

- **ICE (Intercontinental Exchange)**: ICE Binary Order Server, ICE FIX Order Server, ICE iMpact Multicast Price Feed
- **Nasdaq**: ITCH Handlers, Nasdaq Fixed Income (NFI)
- **Deutsche Börse**: Eurex T7® Enhanced Trading (ETI), Xetra T7®
- **London Stock Exchange**: LSE FIX Drop Copy, LSE GTP
- **Euronext**: Optiq Market Data Gateway (MDG)

### 4. Service Level Guarantee

**Maintenance Commitment**:
- SDKs kept up-to-date with venue API updates
- Service level guarantee for protocol conformance
- Calibrated and tested for each venue
- Support for venue-specific protocol changes

**Benefits**:
- Focus on trading logic, not connectivity maintenance
- Automatic updates for venue API changes
- Pre-certified connectivity
- Reduced integration complexity

### 5. Development Tools

**Included Features**:
- **Source Code Reference**: Fast-start sample implementations
- **Market Data Logging**: Log and replay for backtesting strategies
- **Complete Data Model**: Full access to venue's data model
- **Source Code Escrow**: Available for enterprise customers
- **Free 30-Day Evaluation**: Test before purchasing

---

## CBOE Integration for Box Spread Trading

### CBOE CFE SDKs

**CFE Binary Order Entry (BOE)**:
- **Protocol**: Binary (ultra-low latency)
- **Purpose**: Order entry for CBOE futures and options
- **Performance**: Optimized for high-frequency trading
- **Use Case**: Execute SPX/SPXW box spread orders directly on CBOE

**CFE FIX Order Entry & FIX Drop**:
- **Protocol**: FIX-based
- **Purpose**: Order routing and trade capture
- **Compatibility**: Standard FIX protocol
- **Use Case**: Alternative to BOE for FIX-based systems

**CFE Multicast PITCH**:
- **Protocol**: Multicast market data
- **Purpose**: Real-time market data feed
- **Performance**: High-throughput market data
- **Use Case**: Real-time SPX/SPXW options quotes for box spread scanning

### Integration Benefits

**Direct CBOE Access**:
- Bypass broker middleware
- Lower latency execution
- Better fill rates
- Direct exchange connectivity

**Market Data**:
- Real-time options chain data
- Multicast PITCH feed for low latency
- Complete market depth
- Trade and quote data

**Order Execution**:
- Binary Order Entry for fastest execution
- FIX Order Entry for standard protocol
- Multi-leg order support
- Order status tracking

---

## Technical Architecture

### SDK Components

**1. Market Data Handler**:
- Real-time market data processing
- Multicast feed handling
- Order book reconstruction
- Quote and trade processing

**2. Order Routing**:
- Order entry and management
- Order status tracking
- Fill reporting
- Risk management integration

**3. DropCopy/Trade Capture**:
- Post-trade processing
- Trade reconciliation
- Audit trail generation
- Compliance reporting

### Protocol Implementation

**FIX-Based Venues**:
- OnixS ultra-low latency FIX Engine
- Optimized FIX message processing
- Session management
- Heartbeat and recovery

**Binary Protocol Venues**:
- Native binary protocol implementations
- Custom message encoding/decoding
- High-performance data structures
- Minimal latency overhead

---

## Relevance to Box Spread Trading

### 1. Direct CBOE Access

**Benefits**:
- Execute SPX/SPXW box spreads directly on CBOE
- Lower latency than broker APIs
- Better fill rates for arbitrage opportunities
- Direct control over execution

**Implementation**:
- Use CFE Binary Order Entry (BOE) for fastest execution
- Integrate CFE Multicast PITCH for real-time market data
- Route multi-leg box spread orders atomically

### 2. Ultra-Low Latency Execution

**Critical for Arbitrage**:
- Box spread opportunities are time-sensitive
- Fast execution before opportunity disappears
- Minimize slippage on multi-leg orders
- Real-time opportunity detection and execution

**Performance**:
- Binary protocols for minimal latency
- Optimized for high-frequency trading
- Calibrated and pre-certified for CBOE

### 3. Market Data Integration

**Real-Time Options Chain**:
- Multicast PITCH feed for SPX/SPXW options
- Real-time quotes for all strikes
- Market depth for liquidity assessment
- Trade data for execution quality analysis

**Use Case**: Scan for box spread opportunities in real-time using CBOE market data

### 4. Multi-Leg Order Support

**Box Spread Execution**:
- Execute 4-leg box spread orders atomically
- Order routing for complex strategies
- Fill reporting for each leg
- Position tracking

### 5. Interest Rate Futures Hedging

**CME Integration**:
- CME iLink 3 for SOFR/Eurodollar futures
- Hedge box spread interest rate risk
- Real-time futures market data
- Cross-venue arbitrage

---

## Comparison with Current Solutions

### OnixS directConnect vs. TWS API (IBKR)

| Feature | OnixS directConnect | TWS API (IBKR) |
|---------|---------------------|----------------|
| **Protocol** | Binary/FIX (venue-specific) | Socket-based (TWS) |
| **Latency** | Ultra-low | Low-Medium |
| **Direct Exchange** | ✅ Yes (CBOE) | ❌ Via broker |
| **Market Data** | Multicast PITCH | TWS market data |
| **Order Entry** | Binary Order Entry | TWS order API |
| **Multi-Venue** | ✅ Multiple venues | ❌ IBKR only |
| **Options Support** | ✅ CBOE native | ✅ Via broker |
| **Cost** | SDK license | Broker commissions |
| **Maintenance** | OnixS maintains | IBKR maintains |

### OnixS directConnect vs. TFB FIX API

| Feature | OnixS directConnect | TFB FIX API |
|---------|---------------------|-------------|
| **Protocol** | Binary/FIX (venue-specific) | FIX Protocol |
| **Latency** | Ultra-low | Ultra-low |
| **Direct Exchange** | ✅ Yes | ✅ Yes |
| **Venue Coverage** | 20+ exchanges | 100+ liquidity providers |
| **CBOE Support** | ✅ Native (CFE BOE) | ⚠️ Via FIX |
| **Market Data** | ✅ Multicast PITCH | ✅ Yes |
| **Options** | ✅ CBOE native | ✅ Yes |
| **SDK vs. Platform** | SDK (integrate into app) | Platform (use their system) |

---

## Integration Considerations

### Current Project Architecture

**Primary Broker**: Interactive Brokers (TWS API)
- Socket-based protocol
- Comprehensive options support
- Already integrated

**Potential Addition**: OnixS directConnect
- Direct CBOE access via CFE BOE
- Ultra-low latency execution
- Real-time market data via Multicast PITCH

### Integration Pattern

**Option 1: Complementary Market Data**
- Use OnixS for CBOE market data (Multicast PITCH)
- Use TWS API for order execution
- Best of both: fast data + broker execution

**Option 2: Direct CBOE Execution**
- Use OnixS for both market data and execution
- Bypass broker for CBOE orders
- Lower latency, better control

**Option 3: Multi-Venue Strategy**
- Use OnixS for CBOE (options)
- Use OnixS for CME (futures hedging)
- Use TWS API for other venues
- Optimal routing across venues

### Technical Requirements

**SDK Integration**:
- C++ SDK available (matches project language)
- Source code reference implementations
- Market data logging for backtesting
- Complete data model access

**Development**:
- Free 30-day evaluation
- Source code samples included
- Technical support available
- Documentation and examples

### Cost Considerations

**Licensing Model**:
- SDK license (contact OnixS for pricing)
- Service level support
- Maintenance and updates included
- Source code escrow available

**Cost-Benefit Analysis**:
- Lower latency → better fills → higher profits
- Direct exchange access → potentially lower fees
- Real-time market data → better opportunity detection
- SDK cost vs. execution improvements

---

## Use Cases for Box Spread Trading

### 1. Direct CBOE Box Spread Execution

**Scenario**: Execute SPX/SPXW box spreads directly on CBOE

**Implementation**:
- Use CFE Binary Order Entry (BOE) for order execution
- Use CFE Multicast PITCH for real-time market data
- Execute 4-leg box spread orders atomically
- Monitor execution quality in real-time

**Benefits**:
- Lower latency than broker APIs
- Better fill rates
- Direct control over execution
- Real-time market data

### 2. High-Frequency Box Spread Scanning

**Scenario**: Rapidly scan and execute box spread opportunities

**Implementation**:
- Integrate CFE Multicast PITCH for real-time quotes
- Scan for opportunities using OnixS market data
- Execute immediately via CFE BOE when opportunity found
- Log market data for backtesting

**Benefits**:
- Ultra-low latency market data
- Fast opportunity detection
- Quick execution before opportunity disappears
- Market data logging for strategy development

### 3. Multi-Venue Arbitrage

**Scenario**: Execute box spread legs across different venues

**Implementation**:
- Use OnixS for CBOE (options)
- Use OnixS for CME (futures hedging)
- Compare prices across venues
- Route to best venue for each leg

**Benefits**:
- Access to multiple venues
- Best execution prices
- Cross-venue arbitrage opportunities
- Optimal routing

### 4. Interest Rate Futures Hedging

**Scenario**: Hedge box spread interest rate risk with CME futures

**Implementation**:
- Use CME iLink 3 for SOFR/Eurodollar futures
- Real-time futures market data
- Execute hedge orders via CME
- Monitor basis risk

**Benefits**:
- Direct CME access
- Low-latency futures execution
- Real-time hedge monitoring
- Cross-venue integration

---

## Resources

### Official Resources

- **OnixS directConnect**: <https://www.onixs.biz/directconnect.html>
- **CBOE CFE SDKs**: <https://www.onixs.biz/directconnect/cboe-cfe.html>
- **CME Globex SDKs**: <https://www.onixs.biz/directconnect/cme-globex.html>
- **Contact Sales**: <sales@onixs.biz>
- **Technical Support**: <support@onixs.biz>
- **Phone UK**: +44 20 7117 0111
- **Phone US**: +1 312 999 6040

### Evaluation

- **Free 30-Day Evaluation**: Available for testing
- **Request Demo**: Hands-on introduction available
- **Source Code Samples**: Included with SDK

### Related Documentation

- **FIX Protocol**: `docs/API_DOCUMENTATION_INDEX.md` - FIX Trading Community
- **TFB FIX API**: `docs/TOOLS_FOR_BROKERS_FIX_API.md` - Alternative FIX platform
- **CBOE Fees**: `docs/CBOE_FREQUENT_TRADER_PROGRAM.md` - Fee rebate program
- **CME Fees**: `docs/CME_FEE_SCHEDULE_REBATES.md` - CME fee schedules

---

## Key Takeaways

1. **Ultra-Low Latency**: Designed for highest performance, lowest latency execution
2. **Direct CBOE Access**: Native CBOE CFE SDKs (BOE, FIX, Multicast PITCH)
3. **Multi-Venue Support**: 20+ exchanges including CBOE, CME, ICE, Nasdaq
4. **Protocol Flexibility**: FIX-based and native binary protocols
5. **Service Guarantee**: SDKs maintained and updated with venue API changes
6. **Development Tools**: Source code samples, market data logging, backtesting support
7. **C++ SDK**: Available for integration with existing C++ project
8. **Free Evaluation**: 30-day trial available

---

## Potential Integration Opportunities

### For Box Spread Trading

1. **Direct CBOE Execution**: Use CFE BOE for SPX/SPXW box spread orders
2. **Real-Time Market Data**: Integrate CFE Multicast PITCH for options chain scanning
3. **Low-Latency Arbitrage**: Fast execution for time-sensitive opportunities
4. **Futures Hedging**: CME iLink 3 for interest rate futures hedging
5. **Multi-Venue Trading**: Execute across CBOE and CME for optimal execution

### Integration Strategy

**Phase 1: Evaluation**
- Request 30-day free evaluation
- Test CFE Multicast PITCH for market data
- Evaluate CFE BOE for order execution
- Compare latency with TWS API

**Phase 2: Market Data Integration**
- Integrate CFE Multicast PITCH for real-time quotes
- Use for box spread opportunity scanning
- Compare with TWS API market data quality
- Log market data for backtesting

**Phase 3: Order Execution**
- Integrate CFE BOE for direct CBOE execution
- Test multi-leg box spread orders
- Compare execution quality with TWS API
- Monitor latency improvements

**Phase 4: Production**
- Deploy as complementary execution venue
- Use for high-frequency opportunities
- Monitor execution quality
- Scale based on results

---

## Related Documentation

- **FIX Protocol**: `docs/API_DOCUMENTATION_INDEX.md` - FIX Trading Community standards
- **TFB FIX API**: `docs/TOOLS_FOR_BROKERS_FIX_API.md` - Alternative FIX platform
- **CBOE Fees**: `docs/CBOE_FREQUENT_TRADER_PROGRAM.md` - Fee rebate program
- **CME Fees**: `docs/CME_FEE_SCHEDULE_REBATES.md` - CME fee schedules
- **Broker Selection**: `docs/BROKER_SELECTION_ISRAEL.md` - Broker comparison

---

**Note**: OnixS directConnect provides ultra-low latency SDKs for direct exchange access, particularly valuable for CBOE options trading. The C++ SDK aligns with the project's technology stack. Contact OnixS for pricing, evaluation, and integration support. Consider as alternative to broker APIs for direct CBOE access and ultra-low latency execution.
