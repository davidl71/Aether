# eToro - Social Trading Platform

**Date**: 2025-01-27
**Source**: <https://www.etoro.com/discover>
**Provider**: eToro - Social trading and investment platform

---

## Overview

eToro is a social trading platform that enables users to invest in various assets and copy trades from experienced investors using CopyTrader™ technology. The platform focuses on retail investors and social trading features rather than algorithmic trading or options strategies.

---

## Key Features

### 1. Social Trading

**CopyTrader™ Technology**:

- Replicate trades of experienced investors in real-time
- Follow and copy successful traders
- Social trading community
- Performance statistics and rankings

**Use Case**: Retail investors copying other traders' strategies

### 2. Asset Classes

**Available Assets**:

- **Stocks**: Global stock markets
- **Cryptocurrencies**: Limited for U.S. customers (see restrictions below)
- **ETFs**: Exchange-traded funds
- **CFDs**: Contracts for Difference (not available in all jurisdictions)

**Not Available**:

- **Options Trading**: ❌ No options trading support
- **Futures**: ❌ No futures trading
- **Complex Strategies**: ❌ No multi-leg options strategies

### 3. U.S. Market Restrictions

**Cryptocurrency Limitations** (September 2024 SEC Settlement):

- **Limited Crypto**: U.S. customers can only trade Bitcoin, Bitcoin Cash, and Ethereum
- **Most Crypto Removed**: eToro ceased offering most cryptocurrency trading to U.S. customers
- **Settlement**: Result of SEC settlement in September 2024

**Impact**: Significantly reduced cryptocurrency offerings for U.S. users

### 4. Platform Type

**Retail-Focused**:

- Designed for individual retail investors
- Social trading and copy trading emphasis
- User-friendly interface
- Community features

**Not Designed For**:

- Algorithmic trading
- Institutional trading
- API-driven trading systems
- Complex options strategies

---

## API and Integration

### API Availability

**Status**: ⚠️ Limited or No Public API

**Note**: eToro is primarily a retail social trading platform. Unlike brokers like IBKR or Alpaca, eToro does not appear to offer a comprehensive public API for algorithmic trading.

**Integration Options**:

- Web-based platform
- Mobile applications
- CopyTrader™ functionality
- Social trading features

**Relevance**: ❌ Not suitable for algorithmic box spread trading

---

## Relevance to Box Spread Trading

### Direct Relevance: ❌ None

**Reasons**:

1. **No Options Trading**: eToro does not offer options trading
2. **No API**: No comprehensive API for algorithmic trading
3. **Retail Focus**: Designed for social/copy trading, not algorithmic strategies
4. **Asset Limitations**: Stocks, limited crypto, ETFs only - no options

### Indirect Relevance: ⚠️ Very Limited

**Potential Use Cases** (if any):

- **Market Research**: View stock market trends (not options)
- **Social Signals**: Monitor what other traders are doing (not applicable to box spreads)
- **Educational**: Learn about trading (but not options-specific)

**Conclusion**: eToro is **not relevant** for box spread trading due to lack of options trading and API capabilities.

---

## Comparison with Project Requirements

### Current Project Needs

**Required Features**:

- ✅ Options trading (SPX/SPXW)
- ✅ Multi-leg order support
- ✅ API for algorithmic trading
- ✅ Real-time market data
- ✅ Direct exchange access

### eToro Capabilities

**Available Features**:

- ❌ No options trading
- ❌ No multi-leg strategies
- ❌ No algorithmic trading API
- ⚠️ Limited market data (retail-focused)
- ❌ No direct exchange access

**Conclusion**: eToro does not meet any of the project's requirements for box spread trading.

---

## Alternative Platforms

### For Box Spread Trading

**Recommended Brokers** (already documented):

1. **Interactive Brokers (IBKR)**: ✅ Currently integrated - Comprehensive options support, TWS API
2. **Alpaca Markets**: ✅ Documented - Commission-free options, REST API, Elite features
3. **Lightspeed Trader API**: ⚠️ Evaluate - C++ native API, high performance, options support
4. **Lime Brokerage**: ⚠️ Evaluate - FIX/API access to US Options markets

### For Social Trading (Not Box Spreads)

**If Interested in Social Trading**:

- eToro: Social trading platform
- ZuluTrade: Copy trading platform
- Other social trading platforms

**Note**: Social trading is not relevant to algorithmic box spread trading.

---

## Resources

### Official Resources

- **eToro Discover**: <https://www.etoro.com/discover>
- **eToro Website**: <https://www.etoro.com/>
- **SEC Settlement News**: <https://www.reuters.com/technology/etoro-shut-down-nearly-all-crypto-trading-settlement-with-us-sec-2024-09-12/>

### Related Documentation

- **IBKR TWS API**: `docs/API_DOCUMENTATION_INDEX.md` - Current primary broker
- **Alpaca API**: `docs/API_DOCUMENTATION_INDEX.md` - Alternative broker option
- **Broker Selection**: `docs/BROKER_SELECTION_ISRAEL.md` - Broker comparison guide

---

## Key Takeaways

1. **Social Trading Platform**: Focus on retail investors and copy trading
2. **No Options Trading**: Does not support options or complex strategies
3. **No API**: No comprehensive API for algorithmic trading
4. **Retail-Focused**: Designed for social trading, not institutional/algorithmic trading
5. **Crypto Restrictions**: Limited cryptocurrency trading for U.S. customers (SEC settlement)
6. **Not Relevant**: eToro is not suitable for box spread trading

---

## Recommendation

**For Box Spread Trading**: ❌ **Not Recommended**

**Reasons**:

- No options trading capabilities
- No algorithmic trading API
- Retail social trading focus
- Does not meet project requirements

**Use Instead**:

- **IBKR (TWS API)**: Currently integrated, comprehensive options support
- **Alpaca Markets**: Documented alternative, commission-free options
- **Lightspeed/Lime**: Evaluate for high-performance alternatives

**For Social Trading** (if interested separately):

- eToro may be useful for social/copy trading of stocks
- Not relevant to box spread algorithmic trading

---

## Related Documentation

- **IBKR TWS API**: `docs/API_DOCUMENTATION_INDEX.md` - Current primary broker
- **Alpaca API**: `docs/API_DOCUMENTATION_INDEX.md` - Alternative broker option
- **Broker Selection**: `docs/BROKER_SELECTION_ISRAEL.md` - Comprehensive broker comparison

---

**Note**: eToro is a social trading platform designed for retail investors to copy trades from other traders. It does not offer options trading, algorithmic trading APIs, or the capabilities required for box spread trading. For box spread strategies, use IBKR, Alpaca, or other options-capable brokers with API access.
