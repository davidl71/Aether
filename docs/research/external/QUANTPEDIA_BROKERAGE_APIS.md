# QuantPedia Brokerage APIs - Comprehensive List for Quantitative Traders

**Date**: 2025-01-27
**Source**: <https://quantpedia.com/links-tools/?category=brokerage-api>
**Provider**: QuantPedia - Encyclopedia of Quantitative Trading Strategies

---

## Overview

QuantPedia maintains a comprehensive list of brokerage APIs and tools for quantitative traders. This document summarizes the brokerage APIs listed, focusing on those relevant to box spread trading and options strategies.

---

## Brokerage APIs List

### 1. Alpaca Markets

**Status**: ✅ Already Documented

**Key Features**:

- API-first commission-free stock broker
- Free paper trading and real-time market data
- Strong backtesting integrations (QuantRocket, Blueshift, Backtrader)
- TradingView integration
- **Options Trading**: Multi-leg options support (commission-free via API)
- **Elite Smart Router**: DMA Gateway, VWAP/TWAP orders (user has account)

**Pricing**:

- Stock & ETFs: Free
- Account fee: No
- Margin rate: 3.75%
- Leverage: Up to 4X intraday & 2X overnight

**Relevance**: ✅ High - Commission-free options, modern REST API, Elite features available

**Documentation**: See `docs/API_DOCUMENTATION_INDEX.md` for comprehensive Alpaca documentation

---

### 2. Interactive Brokers (IBKR)

**Status**: ✅ Already Documented (Primary Broker)

**Key Features**:

- One of the largest online brokerage firms
- TWS API supports multiple asset classes
- Coverage of 135 markets, 33 countries, 23 currencies
- **Options Trading**: Comprehensive options support (SPX/SPXW)
- Backtesting and paper trading available

**Pricing**:

- Stock & ETFs: $1.0 for US stocks
- Account fee: No
- Inactivity fee: Yes
- Margin rate: Between 1.55% and 0.85%
- Market data: $200 - $2,000/month depending on package

**Relevance**: ✅ High - Currently integrated, comprehensive options support

**Documentation**: See `docs/API_DOCUMENTATION_INDEX.md` for TWS API documentation

---

### 3. TD Ameritrade (Now Charles Schwab)

**Status**: ⚠️ Limited Relevance

**Key Features**:

- Large online brokerage (acquired by Charles Schwab)
- API built for service integration
- US-focused, Nasdaq Level 1/2, Real-time OPRA/AMEX/NYSE quotes
- Limited backtesting integration
- No paper trading

**Pricing**:

- Stock & ETFs: Free US stocks
- Account fee: No
- Market data: Free for non-professional, $24-$110/month for professional

**Relevance**: ⚠️ Moderate - US-focused, options support, but limited API capabilities

**Note**: Acquired by Charles Schwab - verify current API status

---

### 4. Drive Wealth

**Status**: ❌ Not Relevant

**Key Features**:

- API-driven brokerage for enterprise clients
- Cloud-based brokerage infrastructure
- Fractional investing of US stocks
- Closed API (enterprise use only)

**Relevance**: ❌ Low - Enterprise-only, no options trading, not suitable for individual traders

---

### 5. Xignite - Brokerage API

**Status**: ⚠️ Market Data Only

**Key Features**:

- Cloud-native market data solutions
- Forex API (170+ currencies, 29,000+ currency pairs)
- ETFs, mutual funds, hedge fund data APIs
- Market data provider, not trading broker

**Relevance**: ⚠️ Moderate - Market data only, no trading capabilities

**Use Case**: Market data for analysis, not execution

---

### 6. IG UK

**Status**: ❌ Not Relevant

**Key Features**:

- Spread betting and CFD provider
- Access to 17,000+ global markets
- UK-focused (FCA regulated)
- Demo account available

**Relevance**: ❌ Low - Spread betting/CFD, not options trading, UK-focused

---

### 7. Lightspeed Trader API

**Status**: ✅ Potentially Relevant

**Key Features**:

- **C++ API**: Libraries for C++ programmers (matches project language!)
- **DLL Integration**: Create dynamic link libraries (DLLs)
- **List Order Entry**: Basket trading functionality
- **Order Management**: Order entry, routes, position management
- **Market Data**: Real-time Level II quotes, book data, trades
- **Risk Management**: Pre-trade risk checks, real-time risk management
- **Performance**: Up to 1,500 orders per second per ID
- **Low Latency**: Minimized latency, co-location options available
- **No Additional Fees**: No additional market data subscription fees for API use

**Pricing**:

- Stock & ETFs: Yes
- Minimum monthly commission: $25 for accounts under $15,000
- Margin rate: $0.01/share or $1.00/contract with $50 per position minimum

**Relevance**: ✅ High - C++ API matches project technology stack, high performance, options support

**Use Case**: Alternative C++ API for US equity and options trading

**Contact**: <https://www.lightspeed.com/trading-api/>

---

### 8. E*TRADE

**Status**: ⚠️ Limited Relevance

**Key Features**:

- Online trading for retail investors
- Stocks, Options, Futures, ETFs, Mutual Funds, Bonds, CDs
- Multiple account types available

**Pricing**:

- Account fee: No
- Margin rate: Between 5.45% and 8.95%

**Relevance**: ⚠️ Moderate - Options support, but primarily retail-focused, limited API capabilities

---

### 9. Ally Self-Directed Trading

**Status**: ❌ Not Relevant

**Key Features**:

- Self-directed trading platform
- Commission-free ETFs
- No account minimum

**Relevance**: ❌ Low - Retail-focused, limited API capabilities, no options focus

---

### 10. Lime Brokerage

**Status**: ✅ Potentially Relevant

**Key Features**:

- **Interface Options**: APIs, FIX, or trading applications
- **Market Access**: All U.S. Equity and Options markets
- **Trading Management**: Advanced strategies, risk monitoring
- **Performance Focus**: Innovative and faster solutions for real-time performance
- **Infrastructure**: Lime Network Operations for automated trading
- **Market Data**: Comprehensive and accurate market data for executions

**Relevance**: ✅ High - Direct access to US Equity and Options markets, FIX support, performance-focused

**Use Case**: Alternative broker for US options trading with FIX/API access

**Contact**: <https://www.limebrokerage.com/contact/>

---

### 11. Infront Professional Terminal APIs

**Status**: ⚠️ Market Data Only

**Key Features**:

- **Excel Integration**: Live streaming market data into Excel
- **Desktop APIs**: R and Python APIs
- **Market Data**: Streaming data from Infront terminal
- **Portfolio Analysis**: Portfolio analyses, simulations, visualizations

**Relevance**: ⚠️ Moderate - Market data and analysis tools, not trading execution

**Use Case**: Market data analysis and portfolio management, not box spread execution

**Free Trial**: <https://www.infrontfinance.com/order/try/>

---

## Relevance Assessment for Box Spread Trading

### Highly Relevant APIs

1. **Interactive Brokers (TWS API)**: ✅ Currently integrated, comprehensive options support
2. **Alpaca Markets**: ✅ Documented, commission-free options, Elite features available
3. **Lightspeed Trader API**: ✅ C++ API matches project stack, high performance, options support
4. **Lime Brokerage**: ✅ FIX/API access to US Options markets, performance-focused

### Moderately Relevant APIs

1. **TD Ameritrade**: ⚠️ US-focused, options support, but limited API capabilities
2. **E*TRADE**: ⚠️ Options support, but retail-focused, limited API
3. **Xignite**: ⚠️ Market data only, no trading
4. **Infront**: ⚠️ Market data and analysis, no trading

### Not Relevant APIs

1. **Drive Wealth**: ❌ Enterprise-only, no options
2. **IG UK**: ❌ Spread betting/CFD, UK-focused
3. **Ally**: ❌ Retail-focused, limited API

---

## Detailed Comparison

### Options Trading Support

| Broker | Options Support | API Type | Language | Status |
|--------|----------------|----------|----------|--------|
| **IBKR** | ✅ Excellent | TWS API (Socket) | C++/Python/Java | ✅ Integrated |
| **Alpaca** | ✅ Good | REST API | Python/.NET/Go/JS | ✅ Documented |
| **Lightspeed** | ✅ Yes | C++ API (DLL) | C++ | ⚠️ Not Integrated |
| **Lime** | ✅ Yes | FIX/API | Multiple | ⚠️ Not Integrated |
| **TD Ameritrade** | ⚠️ Limited | REST API | Multiple | ⚠️ Not Integrated |
| **E*TRADE** | ⚠️ Limited | Limited | Limited | ❌ Not Relevant |

### Performance Characteristics

| Broker | Latency | Throughput | Co-Location | Notes |
|--------|---------|------------|-------------|-------|
| **IBKR** | Low-Medium | Good | Available | Via broker |
| **Alpaca** | Low | Good | Available | REST API |
| **Lightspeed** | Low | Excellent (1,500/sec) | Available | C++ API, optimized |
| **Lime** | Low | High | Available | FIX/API, performance-focused |

### Cost Comparison

| Broker | Commission | Account Fee | Market Data | Minimum |
|--------|-----------|-------------|-------------|---------|
| **IBKR** | $1.0/stock | No | $200-$2,000/mo | $0 |
| **Alpaca** | Free (API) | No | May apply | $0 (US), $30k (non-US) |
| **Lightspeed** | $0.01/share | No | Included | $25/mo min |
| **Lime** | Contact | Contact | Contact | Contact |

---

## Recommendations for Box Spread Trading

### Primary Broker: Interactive Brokers (IBKR)

**Rationale**:

- ✅ Already integrated in project
- ✅ Comprehensive options support (SPX/SPXW)
- ✅ Global market access
- ✅ TWS API well-documented

### Secondary Broker: Alpaca Markets

**Rationale**:

- ✅ Already documented
- ✅ Commission-free options (API)
- ✅ Modern REST API
- ✅ Elite features available (DMA, VWAP/TWAP)

### Alternative Options (Not Yet Integrated)

**Lightspeed Trader API**:

- ✅ C++ API matches project technology stack
- ✅ High performance (1,500 orders/sec)
- ✅ Low latency with co-location options
- ✅ Options support
- ⚠️ Requires evaluation and integration

**Lime Brokerage**:

- ✅ FIX/API access to US Options markets
- ✅ Performance-focused
- ✅ Direct market access
- ⚠️ Requires evaluation and integration

---

## Integration Considerations

### Lightspeed Trader API

**Advantages**:

- **C++ Native**: Matches project technology stack perfectly
- **High Performance**: 1,500 orders/second per ID
- **Low Latency**: Optimized for performance, co-location available
- **No Additional Fees**: No extra market data fees for API use
- **DLL Integration**: Can integrate as dynamic library

**Considerations**:

- Requires Lightspeed Trader platform
- Minimum monthly commission ($25 for accounts under $15,000)
- Need to evaluate options trading capabilities
- Integration effort required

**Use Case**: Alternative C++ API for high-performance options trading

### Lime Brokerage

**Advantages**:

- **FIX/API Access**: Industry-standard protocols
- **US Options Markets**: Direct access
- **Performance-Focused**: Optimized for automated trading
- **Comprehensive Market Data**: Real-time data for executions

**Considerations**:

- Requires evaluation of pricing and capabilities
- Need to verify options trading support
- Integration effort required
- Contact required for details

**Use Case**: Alternative broker with FIX/API access for US options

---

## Resources

### QuantPedia Resources

- **Brokerage APIs List**: <https://quantpedia.com/links-tools/?category=brokerage-api>
- **QuantPedia Homepage**: <https://quantpedia.com/>
- **Other Categories**: Backtesting Software, Alternative Data, Historical Data, Education

### Broker Contacts

- **Lightspeed**: <https://www.lightspeed.com/trading-api/>
- **Lime Brokerage**: <https://www.limebrokerage.com/contact/>
- **TD Ameritrade**: Verify current status (acquired by Charles Schwab)
- **E*TRADE**: <https://us.etrade.com/home>

### Related Documentation

- **IBKR TWS API**: `docs/API_DOCUMENTATION_INDEX.md` - Interactive Brokers API
- **Alpaca API**: `docs/API_DOCUMENTATION_INDEX.md` - Alpaca Markets API
- **Broker Selection**: `docs/BROKER_SELECTION_ISRAEL.md` - Broker comparison guide

---

## Key Takeaways

1. **QuantPedia List**: Comprehensive resource for discovering brokerage APIs
2. **Already Documented**: IBKR and Alpaca are already documented in project
3. **New Opportunities**: Lightspeed (C++ API) and Lime (FIX/API) are potentially valuable alternatives
4. **Technology Match**: Lightspeed C++ API matches project technology stack
5. **Performance Options**: Both Lightspeed and Lime offer high-performance solutions
6. **Evaluation Needed**: Lightspeed and Lime require evaluation for options trading capabilities
7. **Current Strategy**: Continue with IBKR primary, Alpaca secondary, evaluate alternatives as needed

---

## Action Items

### For Immediate Consideration

1. **Evaluate Lightspeed Trader API**:
   - Review C++ API documentation
   - Verify options trading capabilities
   - Assess integration complexity
   - Compare costs with current brokers

2. **Evaluate Lime Brokerage**:
   - Contact for pricing and capabilities
   - Verify FIX/API options support
   - Assess performance characteristics
   - Compare with current solutions

### For Future Reference

1. **Monitor TD Ameritrade**: Verify API status after Charles Schwab acquisition
2. **Market Data APIs**: Consider Xignite or Infront for market data analysis
3. **QuantPedia Updates**: Monitor QuantPedia for new broker additions

---

## Related Documentation

- **IBKR TWS API**: `docs/API_DOCUMENTATION_INDEX.md` - Current primary broker
- **Alpaca API**: `docs/API_DOCUMENTATION_INDEX.md` - Secondary broker option
- **Broker Selection**: `docs/BROKER_SELECTION_ISRAEL.md` - Comprehensive broker comparison
- **FIX Protocol**: `docs/API_DOCUMENTATION_INDEX.md` - FIX protocol for direct exchange access

---

**Note**: QuantPedia provides a valuable resource for discovering brokerage APIs. The project already uses IBKR (integrated) and has Alpaca documented. Lightspeed Trader API (C++ native) and Lime Brokerage (FIX/API) are potentially valuable alternatives worth evaluating for high-performance options trading. Always verify current API status, pricing, and capabilities before integration.
