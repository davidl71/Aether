# Massive.com Integration Opportunities

**Date**: 2025-01-27
**Source**: <https://massive.com/docs/rest/quickstart>
**Purpose**: Analyze how Massive.com can enhance this box spread arbitrage project

---

## Overview

[Massive.com](https://massive.com/docs/rest/quickstart) provides comprehensive access to historical and real-time market data from major U.S. exchanges. The REST API offers dividends, trades, quotes, fundamental data, and more. This document outlines integration opportunities for improving box spread detection, execution, and analysis.

---

## What Massive.com Provides

### 1. REST API Endpoints

**Core Data Types**:

- **Dividends**: Dividend records and schedules
- **Trades**: Historical and real-time trade data
- **Quotes**: Bid/ask quotes with sizes
- **Fundamental Data**: Company fundamentals and financials
- **Indices**: Index data and components
- **Forex**: Foreign exchange rates
- **Crypto**: Cryptocurrency market data
- **Economy**: Economic indicators

**API Features**:

- Structured JSON responses
- Consistent response format (`status`, `count`, `results`, `request_id`)
- Query string or header-based authentication
- Comprehensive filtering and pagination

### 2. Flat Files (CSV Format)

**S3-Compatible Interface**:

- Downloadable historical data in CSV format
- Web-based file browser
- Bulk data downloads
- Suitable for backtesting and analysis

### 3. WebSocket API

**Real-Time Streaming**:

- Continuous market updates
- All major asset classes
- Low-latency data delivery
- Suitable for live trading applications

### 4. Client Libraries

**Official Libraries**:

- **Python**: GitHub - Python Client
- **Go**: GitHub - Go Client
- **Kotlin**: GitHub - JVM Client
- **JavaScript**: GitHub - JavaScript Client

**Benefits**:

- Authentication and credential management
- Request formatting and error handling
- Data parsing and integration

---

## Integration Opportunities

### High Priority Integrations

#### 1. Historical Trade Data for Backtesting

**Current State**:

- No backtesting capability
- No historical data integration
- Can only paper trade or live trade

**Massive.com Enhancement**:

- Access to historical trade data
- Backtest strategies on past market conditions
- Validate profitability thresholds
- Optimize parameters (min_profit, min_roi, etc.)

**Implementation**:

```python
# python/integration/massive_client.py
class MassiveClient:
    def get_historical_trades(
        self,
        symbol: str,
        start_date: str,
        end_date: str
    ) -> List[Dict]:
        """Fetch historical trade data from Massive.com."""
        endpoint = f"{self.base_url}/trades"
        params = {
            "symbol": symbol,
            "start_date": start_date,
            "end_date": end_date,
            "apiKey": self.api_key
        }
        response = self._session.get(endpoint, params=params)
        response.raise_for_status()
        return response.json().get("results", [])
```

**Use Cases**:

- Backtest box spread strategy on historical data
- Identify which strike widths are most profitable
- Determine optimal DTE range
- Test strategy during different market conditions

**Benefits**:

- Validate strategy before live trading
- Optimize parameters with data-driven approach
- Estimate expected returns

---

#### 2. Real-Time Quotes via WebSocket

**Current State**:

- Gets quotes from TWS API callbacks
- Single source of truth (TWS only)
- No cross-validation

**Massive.com Enhancement**:

- Real-time quotes via WebSocket API
- Cross-validate with TWS quotes
- Detect data discrepancies
- Redundancy for critical data

**Implementation**:

```python
# python/integration/massive_websocket.py
class MassiveWebSocketClient:
    def __init__(self, api_key: str):
        self.api_key = api_key
        self.ws = None

    def connect(self):
        """Connect to Massive.com WebSocket API."""
        # Implementation
        pass

    def subscribe_quotes(self, symbols: List[str]):
        """Subscribe to real-time quotes."""
        # Implementation
        pass

    def on_quote(self, callback):
        """Register callback for quote updates."""
        # Implementation
        pass
```

**Use Cases**:

- Cross-validate TWS quotes with Massive.com quotes
- Detect pricing anomalies
- Redundancy for critical trading decisions
- Compare bid/ask spreads across sources

**Benefits**:

- Data quality validation
- Reduced risk from bad data
- Better execution decisions

---

#### 3. Dividend Data Integration

**Current State**:

- No dividend tracking
- No dividend ex-date awareness
- Early assignment risk not fully managed

**Massive.com Enhancement**:

- Dividend records and schedules
- Ex-dates and amounts
- Historical dividend data
- Upcoming dividend calendar

**Implementation**:

```cpp
// include/types.h - enhance MarketData
struct MarketData {
    // Existing fields...

    // Add Massive.com dividend data
    std::optional<double> next_dividend_amount;
    std::optional<std::string> next_dividend_exdate;
    std::optional<int> days_to_exdate;
};
```

**Use Cases**:

- Avoid box spreads near dividend ex-dates (early assignment risk)
- Filter opportunities during high-risk periods
- Better timing of entries/exits
- Calculate dividend-adjusted returns

**Benefits**:

- Reduced risk from early assignment
- Better timing of entries/exits
- Avoid unexpected dividend-related losses

---

#### 4. Fundamental Data for Risk Assessment

**Current State**:

- No fundamental data integration
- No company health assessment
- Strategy doesn't consider company fundamentals

**Massive.com Enhancement**:

- Company fundamentals and financials
- Market cap, P/E ratio, etc.
- Financial health indicators
- Sector and industry data

**Implementation**:

```cpp
// include/config_manager.h
struct StrategyParams {
    // Add fundamental filters
    bool filter_by_market_cap = true;
    double min_market_cap = 1e9;  // $1B minimum
    bool filter_by_pe_ratio = true;
    double max_pe_ratio = 50.0;
    bool avoid_penny_stocks = true;
};
```

**Use Cases**:

- Filter out low-quality companies
- Avoid penny stocks
- Focus on liquid, established companies
- Better risk-adjusted returns

**Benefits**:

- Reduced risk from low-quality companies
- Better opportunity filtering
- Improved risk-adjusted returns

---

### Medium Priority Integrations

#### 5. Historical Quote Data

**Massive.com Data**:

- Historical bid/ask quotes
- Spread analysis over time
- Liquidity patterns
- Market microstructure data

**Use Cases**:

- Analyze historical spreads
- Identify optimal entry/exit times
- Understand liquidity patterns
- Backtest execution strategies

---

#### 6. Trade Data Analysis

**Massive.com Data**:

- Historical trade data
- Volume patterns
- Price action analysis
- Market impact studies

**Use Cases**:

- Analyze historical execution quality
- Understand market impact
- Optimize order sizing
- Improve execution timing

---

#### 7. Index Data Integration

**Massive.com Data**:

- Index components and weights
- Index performance data
- Sector and industry data

**Use Cases**:

- Monitor index-level opportunities
- Sector rotation strategies
- Diversification analysis
- Market correlation studies

---

### Low Priority Integrations

#### 8. Forex and Crypto Data

**Massive.com Data**:

- Foreign exchange rates
- Cryptocurrency market data

**Use Cases**:

- Multi-asset strategies
- Currency hedging
- Crypto options (if available)

---

## Implementation Plan

### Phase 1: Foundation (Week 1)

1. **Add Massive.com Configuration**
   - Add Massive.com section to config.json
   - API key management
   - Enable/disable flags

2. **Create Massive.com Client**
   - Python client for REST API
   - Authentication handling
   - Rate limiting
   - Error handling

3. **Basic Data Integration**
   - Fetch dividend data
   - Fetch fundamental data
   - Compare with TWS data
   - Log discrepancies

### Phase 2: Historical Data (Week 2)

1. **Historical Trade Data**
   - Fetch historical trades
   - Build backtesting framework
   - Run strategy on historical data

2. **Historical Quote Data**
   - Fetch historical quotes
   - Analyze spreads over time
   - Optimize execution timing

### Phase 3: Real-Time Integration (Week 3)

1. **WebSocket Integration**
   - Connect to WebSocket API
   - Subscribe to real-time quotes
   - Cross-validate with TWS

2. **Quote Comparison**
   - Compare TWS vs Massive.com quotes
   - Detect discrepancies
   - Alert on data quality issues

### Phase 4: Risk Management (Week 4)

1. **Dividend Integration**
   - Fetch dividend schedules
   - Add blackout period filtering
   - Avoid opportunities near ex-dates

2. **Fundamental Filtering**
   - Fetch fundamental data
   - Add quality filters
   - Filter out low-quality companies

---

## Code Implementation

### Configuration

```json
{
  "massive": {
    "enabled": true,
    "api_key": "${MASSIVE_API_KEY}",
    "base_url": "https://api.massive.com",

    "use_for_historical_data": true,
    "use_for_realtime_quotes": true,
    "use_for_dividend_data": true,
    "use_for_fundamental_data": true,

    "websocket_enabled": true,
    "websocket_url": "wss://api.massive.com/ws",

    "min_market_cap": 1000000000,
    "max_pe_ratio": 50.0,
    "avoid_penny_stocks": true,

    "dividend_blackout_days": 2,

    "cache_duration_seconds": 300,
    "rate_limit_per_second": 10
  }
}
```

### Python Massive.com Client

```python
# python/integration/massive_client.py
"""
massive_client.py - Client for Massive.com API integration
"""
import logging
import requests
from typing import Dict, List, Optional
from datetime import datetime, timedelta
from functools import lru_cache

logger = logging.getLogger(__name__)


class MassiveClient:
    """
    Client for Massive.com REST API.
    Provides market data, dividends, fundamentals, and historical data.
    """

    def __init__(self, api_key: str, base_url: str = "https://api.massive.com"):
        self.api_key = api_key
        self.base_url = base_url
        self._session = requests.Session()
        self._session.headers.update({
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json"
        })
        self._cache = {}
        self._cache_duration = timedelta(minutes=5)

    def get_dividends(
        self,
        symbol: Optional[str] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None
    ) -> List[Dict]:
        """
        Get dividend records from Massive.com.

        Args:
            symbol: Stock ticker (optional, for filtering)
            start_date: Start date in YYYY-MM-DD format
            end_date: End date in YYYY-MM-DD format

        Returns:
            List of dividend records
        """
        cache_key = f"dividends_{symbol}_{start_date}_{end_date}"
        if self._is_cached(cache_key):
            return self._cache[cache_key]["data"]

        endpoint = f"{self.base_url}/dividends"
        params = {
            "apiKey": self.api_key
        }

        if symbol:
            params["symbol"] = symbol
        if start_date:
            params["start_date"] = start_date
        if end_date:
            params["end_date"] = end_date

        try:
            response = self._session.get(endpoint, params=params, timeout=10)
            response.raise_for_status()
            data = response.json().get("results", [])

            self._cache[cache_key] = {
                "data": data,
                "timestamp": datetime.now()
            }

            return data

        except requests.exceptions.RequestException as e:
            logger.error(f"Massive.com API error: {e}")
            return []

    def get_trades(
        self,
        symbol: str,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None
    ) -> List[Dict]:
        """
        Get historical trade data from Massive.com.

        Args:
            symbol: Stock ticker
            start_date: Start date in YYYY-MM-DD format
            end_date: End date in YYYY-MM-DD format

        Returns:
            List of trade records
        """
        endpoint = f"{self.base_url}/trades"
        params = {
            "symbol": symbol,
            "apiKey": self.api_key
        }

        if start_date:
            params["start_date"] = start_date
        if end_date:
            params["end_date"] = end_date

        try:
            response = self._session.get(endpoint, params=params, timeout=10)
            response.raise_for_status()
            return response.json().get("results", [])

        except requests.exceptions.RequestException as e:
            logger.error(f"Massive.com API error: {e}")
            return []

    def get_quotes(
        self,
        symbol: str,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None
    ) -> List[Dict]:
        """
        Get historical quote data from Massive.com.

        Args:
            symbol: Stock ticker
            start_date: Start date in YYYY-MM-DD format
            end_date: End date in YYYY-MM-DD format

        Returns:
            List of quote records
        """
        endpoint = f"{self.base_url}/quotes"
        params = {
            "symbol": symbol,
            "apiKey": self.api_key
        }

        if start_date:
            params["start_date"] = start_date
        if end_date:
            params["end_date"] = end_date

        try:
            response = self._session.get(endpoint, params=params, timeout=10)
            response.raise_for_status()
            return response.json().get("results", [])

        except requests.exceptions.RequestException as e:
            logger.error(f"Massive.com API error: {e}")
            return []

    def get_fundamentals(self, symbol: str) -> Optional[Dict]:
        """
        Get fundamental data for a symbol.

        Args:
            symbol: Stock ticker

        Returns:
            Fundamental data dictionary
        """
        endpoint = f"{self.base_url}/fundamentals"
        params = {
            "symbol": symbol,
            "apiKey": self.api_key
        }

        try:
            response = self._session.get(endpoint, params=params, timeout=10)
            response.raise_for_status()
            results = response.json().get("results", [])
            return results[0] if results else None

        except requests.exceptions.RequestException as e:
            logger.error(f"Massive.com API error: {e}")
            return None

    def _is_cached(self, key: str) -> bool:
        """Check if data is cached and not stale."""
        if key not in self._cache:
            return False

        age = datetime.now() - self._cache[key]["timestamp"]
        return age < self._cache_duration
```

### Configuration Integration

```cpp
// include/config_manager.h
struct MassiveConfig {
    bool enabled = false;
    std::string api_key;
    std::string base_url = "https://api.massive.com";

    bool use_for_historical_data = true;
    bool use_for_realtime_quotes = true;
    bool use_for_dividend_data = true;
    bool use_for_fundamental_data = true;

    bool websocket_enabled = false;
    std::string websocket_url = "wss://api.massive.com/ws";

    double min_market_cap = 1e9;  // $1B
    double max_pe_ratio = 50.0;
    bool avoid_penny_stocks = true;

    int dividend_blackout_days = 2;

    int cache_duration_seconds = 300;
    int rate_limit_per_second = 10;
};

struct Config {
    TWSConfig tws;
    StrategyParams strategy;
    RiskConfig risk;
    LogConfig logging;
    ORATSConfig orats;
    MassiveConfig massive;  // Add Massive.com config

    // Existing fields...
};
```

---

## Benefits of Massive.com Integration

### 1. Historical Data for Backtesting

- Access to historical trades and quotes
- Backtest strategies on past market conditions
- Validate profitability thresholds
- Optimize parameters empirically

### 2. Data Quality Validation

- Cross-validate TWS quotes with Massive.com quotes
- Detect pricing anomalies
- Redundancy for critical trading decisions
- Better execution decisions

### 3. Risk Management

- Dividend tracking for early assignment risk
- Fundamental data for quality filtering
- Better timing of entries/exits
- Avoid low-quality companies

### 4. Real-Time Data Redundancy

- WebSocket API for real-time quotes
- Cross-validation with TWS
- Reduced risk from bad data
- Better execution quality

### 5. Comprehensive Market Data

- Dividends, trades, quotes, fundamentals
- Multiple asset classes
- Historical and real-time data
- S3-compatible flat files for bulk downloads

---

## Cost Considerations

**Massive.com Pricing** (verify on website):

- Pricing structure not specified in quickstart docs
- Likely tiered based on data access level
- Historical data may have different pricing than real-time
- WebSocket API may require separate subscription

**ROI Analysis**:

- Historical backtesting → parameter optimization → better long-term results
- Data quality validation → reduced risk from bad data
- Dividend tracking → avoid early assignment → reduced losses
- Fundamental filtering → better opportunity quality → improved returns

**Recommendation**: Start with REST API for historical data and dividends, upgrade to WebSocket if real-time redundancy is valuable.

---

## Integration Architecture

### Data Flow

```
Historical Backtesting:
1. Fetch historical trades/quotes from Massive.com
2. Run box spread strategy on historical data
3. Analyze performance and optimize parameters
4. Validate strategy before live trading

Real-Time Quotes:
1. Subscribe to TWS quotes (primary)
2. Subscribe to Massive.com WebSocket (secondary)
3. Cross-validate quotes
4. Alert on discrepancies
5. Use validated data for trading decisions

Dividend Management:
1. Fetch dividend schedules from Massive.com
2. Check ex-dates for monitored symbols
3. Add blackout period filtering
4. Avoid opportunities near ex-dates

Fundamental Filtering:
1. Fetch fundamental data from Massive.com
2. Check market cap, P/E ratio, etc.
3. Filter out low-quality companies
4. Focus on liquid, established companies
```

### Caching Strategy

- Cache Massive.com data for 5 minutes (configurable)
- Refresh on stale data
- Store in appropriate managers
- Reduce API calls, stay within rate limits

---

## Specific Use Cases

### Use Case 1: Historical Backtesting

**Before (No backtesting)**:

- Can only paper trade or live trade
- No historical validation
- Parameter selection is guesswork

**After (With Massive.com)**:

```python
# python/backtesting/backtest_runner.py
class BoxSpreadBacktester:
    def run_backtest(
        self,
        symbol: str,
        start_date: str,
        end_date: str,
        strategy_params: Dict
    ) -> Dict:
        """Run historical backtest using Massive.com data."""

        # Fetch historical trades from Massive.com
        historical_trades = massive_client.get_trades(
            symbol, start_date, end_date
        )

        # Fetch historical quotes
        historical_quotes = massive_client.get_quotes(
            symbol, start_date, end_date
        )

        # Simulate strategy on historical data
        results = self._simulate_strategy(
            historical_trades,
            historical_quotes,
            strategy_params
        )

        return results  # P&L, win rate, Sharpe ratio, etc.
```

**Benefits**:

- Validate strategy on years of data
- Optimize parameters empirically
- Estimate expected returns

---

### Use Case 2: Dividend Blackout Filtering

**Before (No filtering)**:

```cpp
// Evaluate all opportunities
if (is_profitable(spread)) {
    execute_box_spread(spread);
}
```

**After (With Massive.com)**:

```cpp
// Check dividend calendar
if (is_profitable(spread) && !is_dividend_blackout(symbol)) {
    execute_box_spread(spread);
}

bool is_dividend_blackout(const std::string& symbol) {
    auto dividends = massive_client.get_dividends(symbol);
    if (dividends.empty()) return false;

    auto next_dividend = dividends[0];  // Assuming sorted by date
    auto exdate = parse_date(next_dividend["exdate"]);
    int days_to_exdate = calculate_days_to(exdate);

    return days_to_exdate <= dividend_blackout_days;
}
```

**Impact**: Avoid early assignment risk, reduce unexpected losses

---

### Use Case 3: Quote Cross-Validation

**Before (TWS only)**:

```cpp
// Trust TWS quotes
double bid = tws_quote.bid;
double ask = tws_quote.ask;
```

**After (TWS + Massive.com)**:

```cpp
// Cross-validate quotes
double bid = tws_quote.bid;
double ask = tws_quote.ask;

// Get Massive.com quote
auto massive_quote = massive_websocket.get_quote(symbol);
if (massive_quote.has_value()) {
    double massive_bid = massive_quote->bid;
    double massive_ask = massive_quote->ask;

    // Check for large discrepancies
    double bid_diff = std::abs(bid - massive_bid) / bid;
    double ask_diff = std::abs(ask - massive_ask) / ask;

    if (bid_diff > 0.01 || ask_diff > 0.01) {  // 1% threshold
        spdlog::warn("Quote discrepancy detected: TWS bid={}, Massive bid={}",
                     bid, massive_bid);
        // Alert or skip trade
    }
}
```

**Benefits**:

- Detect data quality issues
- Reduce risk from bad data
- Better execution decisions

---

### Use Case 4: Fundamental Quality Filtering

**Before (No filtering)**:

```cpp
// Evaluate all opportunities
if (is_profitable(spread)) {
    execute_box_spread(spread);
}
```

**After (With Massive.com)**:

```cpp
// Check fundamental quality
if (is_profitable(spread) && meets_quality_criteria(symbol)) {
    execute_box_spread(spread);
}

bool meets_quality_criteria(const std::string& symbol) {
    auto fundamentals = massive_client.get_fundamentals(symbol);
    if (!fundamentals.has_value()) return false;

    double market_cap = fundamentals->market_cap;
    double pe_ratio = fundamentals->pe_ratio;
    double price = fundamentals->price;

    return market_cap >= min_market_cap &&
           pe_ratio <= max_pe_ratio &&
           price >= 5.0;  // Avoid penny stocks
}
```

**Benefits**:

- Focus on quality companies
- Reduced risk from low-quality companies
- Better risk-adjusted returns

---

## Recommended Integration Priority

### Immediate (Week 1-2)

1. **Dividend Data**: Simple risk management win
2. **Historical Trade Data**: Foundation for backtesting

### Short-term (Week 3-4)

1. **Historical Quote Data**: Complete backtesting capability
2. **Fundamental Filtering**: Quality improvement

### Medium-term (Month 2-3)

1. **WebSocket Real-Time Quotes**: Data quality validation
2. **Quote Cross-Validation**: Redundancy and quality

### Long-term (Future)

1. **Flat Files Integration**: Bulk historical analysis
2. **Multi-Asset Support**: Forex, crypto, etc.

---

## Cost-Benefit Analysis

### Costs

- Massive.com API subscription: Verify pricing on website
- Development time: ~2-4 weeks
- Maintenance: Minimal (API is stable)

### Benefits

- **Historical Backtesting**: Validate strategy before live trading
- **Data Quality**: Cross-validation reduces risk
- **Risk Management**: Dividend and fundamental filtering
- **Redundancy**: WebSocket quotes as backup to TWS

### ROI Estimate

- Historical backtesting → better parameters → improved returns
- Dividend filtering → avoid early assignment → reduced losses
- Fundamental filtering → better opportunities → improved returns
- Data quality validation → reduced bad trades → improved returns

---

## Next Steps

1. **Obtain Massive.com API Key**: Sign up at <https://massive.com>
2. **Test REST API**: Start with dividends and historical data
3. **Implement MassiveClient**: Create Python client
4. **Add Dividend Filtering**: Implement blackout periods
5. **Build Backtesting Framework**: Use historical data
6. **Add WebSocket Integration**: Real-time quote cross-validation
7. **Add Fundamental Filtering**: Quality-based opportunity filtering

---

## References

- Massive.com Quickstart: <https://massive.com/docs/rest/quickstart>
- Massive.com Documentation: <https://massive.com/docs> (verify full documentation URL)
- Client Libraries: Check GitHub for Python, Go, Kotlin, JavaScript clients

---

## Notes

- Massive.com complements TWS API (doesn't replace it)
- TWS provides execution, Massive.com provides data and analytics
- Historical data doesn't require exchange agreements
- Real-time data may require separate subscriptions
- Consider starting with historical data to test value before real-time subscription
- Compare with ORATS for overlapping use cases (historical data, dividends)
