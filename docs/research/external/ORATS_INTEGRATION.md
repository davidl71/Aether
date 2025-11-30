# ORATS Integration Opportunities

**Date**: 2025-01-27
**Source**: <https://orats.com/docs>
**Purpose**: Analyze how ORATS can enhance this box spread arbitrage project

---

## Overview

[ORATS (Option Research & Technology Services)](https://orats.com/docs) provides institutional-quality options data, including live/delayed market data, historical data back to 2007, and hundreds of proprietary indicators. This document outlines integration opportunities for improving box spread detection and execution.

---

## What ORATS Provides

### 1. Options Data APIs

**Live Data API**:

- Real-time options data with <10 seconds delay
- Bid/ask quotes with sizes
- Implied volatility calculations
- Greeks (delta, gamma, theta, vega)
- Volume and open interest
- [Live Data API Documentation](https://orats.com/data-api)

**Delayed Data API**:

- Historical end-of-day data back to 2007
- Same metrics as live data
- No market data subscription fees required

**Live Intraday API**:

- 1-minute options data
- Real-time with <10 seconds delay
- High granularity for backtesting

**Historical Intraday API**:

- 1-minute historical data back to August 2020
- 5,000+ symbols
- Comprehensive historical analysis

### 2. Proprietary Indicators

ORATS provides hundreds of proprietary indicators including:

- **Smoothed IV Curves**: Better than raw IV for strategy decisions
- **IV Rank/Percentile**: Identify high/low volatility periods
- **Earnings Calendar**: Avoid/target earnings events
- **Dividend Data**: Ex-dates up to 2.8 years in advance
- **Liquidity Scores**: Proprietary liquidity metrics
- **Price Targets**: Consensus price targets
- **Volatility Surface**: Complete volatility surface data

---

## Integration Opportunities

### High Priority Integrations

#### 1. Enhanced Liquidity Scoring

**Current State**:

- This project uses basic volume and open interest checks
- Liquidity score is stubbed (returns 50.0)
- No sophisticated liquidity assessment

**ORATS Enhancement**:

- Use ORATS proprietary liquidity scores
- Better execution probability estimates
- Filter out illiquid opportunities early

**Implementation**:

```cpp
// In option_chain.h
struct OptionChainEntry {
    // Add ORATS fields
    double orats_liquidity_score = 0.0;
    double orats_execution_probability = 0.0;
    double orats_slippage_estimate = 0.0;
};
```

**Configuration**:

```json
{
  "orats": {
    "enabled": true,
    "api_token": "your_token_here",
    "base_url": "https://api.orats.io",
    "use_for_liquidity_scoring": true,
    "min_orats_liquidity_score": 70.0
  }
}
```

**Benefits**:

- Better execution probability estimates
- Reduced failed executions
- Better risk-adjusted returns

---

#### 2. Historical Data for Backtesting

**Current State**:

- No backtesting capability
- No historical data integration
- Can only paper trade or live trade

**ORATS Enhancement**:

- Access to historical data back to 2007
- Backtest strategies on past market conditions
- Validate profitability thresholds
- Optimize parameters (min_profit, min_roi, etc.)

**Implementation**:

```python

# python/integration/orats_client.py

class ORATSClient:
    def get_historical_options(
        self,
        symbol: str,
        start_date: str,
        end_date: str
    ) -> List[Dict]:
        """Fetch historical options data from ORATS."""
        # Implementation
```

**Use Cases**:

- Backtest box spread strategy on 2020-2024 data
- Identify which strike widths are most profitable
- Determine optimal DTE range
- Test strategy during different market conditions

**Benefits**:

- Validate strategy before live trading
- Optimize parameters with data-driven approach
- Estimate expected returns

---

#### 3. Improved Implied Volatility Data

**Current State**:

- Gets IV from TWS API callbacks
- No volatility surface analysis
- No IV skew/term structure analysis

**ORATS Enhancement**:

- Smoothed IV curves (more reliable than raw IV)
- Complete volatility surface
- IV rank and percentile
- Term structure analysis

**Implementation**:

```cpp
// In types.h - enhance MarketData
struct MarketData {
    // Existing fields...

    // Add ORATS IV data
    std::optional<double> orats_smoothed_iv;
    std::optional<double> iv_rank;        // 0-100
    std::optional<double> iv_percentile;  // 0-100
    std::optional<double> atm_iv;         // ATM IV for reference
};
```

**Use Cases**:

- Better pricing validation (compare ORATS IV with TWS IV)
- Detect mispriced options (large IV discrepancies)
- Avoid trades during IV spikes (earnings, news)

**Benefits**:

- More accurate theoretical value calculations
- Better arbitrage detection
- Risk management (avoid high volatility periods)

---

#### 4. Earnings and Corporate Events

**Current State**:

- No earnings calendar integration
- No dividend tracking
- No corporate event awareness

**ORATS Enhancement**:

- Earnings calendar with dates and times
- Dividend ex-dates and amounts
- Corporate actions (splits, mergers)

**Implementation**:

```cpp
// In config_manager.h
struct StrategyParams {
    // Add ORATS event filters
    bool avoid_earnings_period = true;
    int earnings_blackout_days = 7;  // Days before/after earnings
    bool avoid_dividend_exdate = true;
    int dividend_blackout_days = 2;
};
```

**Use Cases**:

- Avoid box spreads near earnings (volatility risk)
- Avoid positions near dividend ex-dates (early assignment risk)
- Filter opportunities during high-risk periods

**Benefits**:

- Reduced risk from unexpected events
- Better timing of entries/exits
- Avoid early assignment scenarios

---

### Medium Priority Integrations

#### 5. Advanced Greeks and Risk Metrics

**ORATS Data**:

- More accurate Greeks than standard calculations
- Second-order Greeks (vanna, charm, etc.)
- Risk metrics (probability of profit, expected value)

**Use Cases**:

- Portfolio-level risk assessment
- Better confidence scoring
- Stress testing positions

---

#### 6. Volatility Smile/Skew Analysis

**ORATS Data**:

- Volatility surface data
- IV skew metrics
- Put/call IV differential

**Use Cases**:

- Detect unusual IV patterns
- Identify mispriced options
- Better spread selection

---

#### 7. Options Flow and Unusual Activity

**ORATS Data**:

- Unusual options activity
- Large volume/OI changes
- Institutional flow

**Use Cases**:

- Avoid crowded trades
- Detect potential liquidity issues
- Identify high-conviction opportunities

---

### Low Priority Integrations

#### 8. Strategy Backtesting Engine

**ORATS Data**:

- Historical intraday data
- Complete option chains historically

**Benefits**:

- Full strategy validation
- Parameter optimization
- Performance analysis

---

## Implementation Plan

### Phase 1: Foundation (Week 1)

1. **Add ORATS Configuration**
   - Add ORATS section to config.json
   - API token management
   - Enable/disable flags

2. **Create ORATS Client**
   - Python client for ORATS API
   - Authentication handling
   - Rate limiting
   - Error handling

3. **Basic Data Integration**
   - Fetch option chain data from ORATS
   - Compare with TWS data
   - Log discrepancies

### Phase 2: Enhanced Liquidity (Week 2)

1. **Integrate Liquidity Scores**
   - Add ORATS liquidity fields to OptionChainEntry
   - Update confidence score calculations
   - Filter by ORATS liquidity thresholds

2. **Execution Probability**
   - Use ORATS slippage estimates
   - Improve `execution_probability` calculation
   - Better opportunity ranking

### Phase 3: Risk Management (Week 3)

1. **Earnings Calendar**
   - Fetch earnings dates for monitored symbols
   - Add blackout period filtering
   - Avoid opportunities near earnings

2. **Dividend Tracking**
   - Fetch dividend ex-dates
   - Avoid American options near ex-dates
   - Early assignment risk management

### Phase 4: Backtesting (Week 4+)

1. **Historical Data Integration**
   - Fetch historical options data
   - Build backtesting framework
   - Run strategy on historical data
   - Parameter optimization

---

## Code Implementation

### Configuration

```json
{
  "orats": {
    "enabled": true,
    "api_token": "${ORATS_API_TOKEN}",
    "base_url": "https://api.orats.io",

    "use_for_liquidity_scoring": true,
    "use_for_iv_data": true,
    "use_for_risk_events": true,

    "min_liquidity_score": 70.0,
    "max_iv_percentile": 80.0,

    "earnings_blackout_days": 7,
    "dividend_blackout_days": 2,

    "cache_duration_seconds": 300,
    "rate_limit_per_second": 10
  }
}
```

### Python ORATS Client

```python

# python/integration/orats_client.py

import requests
from typing import Dict, List, Optional
from datetime import datetime

class ORATSClient:
    """Client for ORATS API integration."""

    def __init__(self, api_token: str, base_url: str = "https://api.orats.io"):
        self.api_token = api_token
        self.base_url = base_url
        self._session = requests.Session()
        self._session.headers.update({"Authorization": f"Token {api_token}"})

    def get_option_chain(
        self,
        ticker: str,
        trade_date: Optional[str] = None
    ) -> Dict:
        """
        Get option chain data from ORATS.

        Args:
            ticker: Stock ticker (e.g., "SPY")
            trade_date: Date in YYYY-MM-DD format (default: today)

        Returns:
            Option chain data with ORATS indicators
        """
        endpoint = f"{self.base_url}/data/strikes"
        params = {
            "ticker": ticker,
            "tradeDate": trade_date or datetime.now().strftime("%Y-%m-%d"),
        }

        response = self._session.get(endpoint, params=params)
        response.raise_for_status()

        return response.json()

    def get_liquidity_scores(self, ticker: str) -> Dict:
        """Get ORATS liquidity scores for a symbol."""
        # Implementation
        pass

    def get_earnings_calendar(self, ticker: str) -> Dict:
        """Get earnings dates for a symbol."""
        # Implementation
        pass

    def get_dividend_schedule(self, ticker: str) -> Dict:
        """Get dividend ex-dates for a symbol."""
        # Implementation
        pass
```

### Enhanced OptionChainEntry

```cpp
// include/types.h
struct OptionChainEntry {
    types::OptionContract contract;
    types::MarketData market_data;

    // Existing fields
    int open_interest = 0;
    int volume = 0;

    // ORATS-enhanced fields
    double orats_liquidity_score = 0.0;        // 0-100
    double orats_execution_probability = 0.0; // 0-1.0
    double orats_slippage_estimate = 0.0;     // Expected slippage
    double orats_smoothed_iv = 0.0;           // Smoothed IV
    double orats_iv_rank = 0.0;               // 0-100
    double orats_iv_percentile = 0.0;         // 0-100

    bool has_orats_data() const {
        return orats_liquidity_score > 0;
    }
};
```

---

## Benefits of ORATS Integration

### 1. Better Opportunity Detection

- Proprietary indicators improve filtering
- More accurate liquidity assessment
- Better execution probability estimates

### 2. Risk Management

- Earnings calendar integration
- Dividend tracking for early assignment risk
- Corporate event awareness

### 3. Historical Analysis

- Backtest on years of historical data
- Optimize parameters empirically
- Validate strategy profitability

### 4. Data Quality

- Smoothed IV curves (less noise)
- Institutional-quality data
- Cross-validation with TWS data

### 5. Reduced Execution Risk

- Better liquidity scores → better fills
- Slippage estimates → better cost modeling
- Execution probability → better opportunity ranking

---

## Cost Considerations

**ORATS Pricing** (approximate, verify on website):

- Basic API: ~$50-100/month
- Live Data: ~$100-300/month (requires exchange agreements)
- Historical Data: One-time or monthly fees
- Intraday Data: Additional cost

**ROI Analysis**:

- If ORATS helps avoid 1-2 bad trades per month → pays for itself
- Better liquidity scoring → better fills → improved returns
- Historical backtesting → parameter optimization → better long-term results

**Recommendation**: Start with delayed data API to test integration, upgrade to live if valuable.

---

## Integration Architecture

### Data Flow

```
Option Chain Request:
1. Request option chain from TWS (primary)
2. Enrich with ORATS data (liquidity, IV, indicators)
3. Store in OptionChain structure
4. Use enhanced data for box spread detection

Box Spread Evaluation:
1. Find candidate spreads (existing logic)
2. Check ORATS liquidity scores
3. Check earnings/dividend calendar
4. Calculate confidence with ORATS metrics
5. Execute if all criteria met
```

### Caching Strategy

- Cache ORATS data for 5 minutes (configurable)
- Refresh on stale data
- Store in option_chain_manager
- Reduce API calls, stay within rate limits

---

## Specific Use Cases

### Use Case 1: Enhanced Liquidity Filtering

**Before (TWS only)**:

```cpp
bool meets_liquidity = volume >= min_volume && open_interest >= min_oi;
```

**After (TWS + ORATS)**:

```cpp
bool meets_liquidity =
    volume >= min_volume &&
    open_interest >= min_oi &&
    orats_liquidity_score >= min_orats_liquidity_score &&
    orats_execution_probability >= min_execution_probability;
```

**Impact**: Fewer failed executions, better fills

---

### Use Case 2: Earnings Blackout Filtering

**Before (No filtering)**:

```cpp
// Evaluate all opportunities
if (is_profitable(spread)) {
    execute_box_spread(spread);
}
```

**After (With ORATS)**:

```cpp
// Check earnings calendar
if (is_profitable(spread) && !is_earnings_blackout(symbol)) {
    execute_box_spread(spread);
}

bool is_earnings_blackout(const std::string& symbol) {
    auto earnings_date = orats_client.get_next_earnings(symbol);
    if (!earnings_date.has_value()) return false;

    int days_to_earnings = calculate_days_to(earnings_date.value());
    return days_to_earnings <= earnings_blackout_days;
}
```

**Impact**: Avoid high-risk periods, reduce unexpected losses

---

### Use Case 3: Historical Backtesting

**Before (No backtesting)**:

- Can only paper trade or live trade
- No historical validation
- Parameter selection is guesswork

**After (With ORATS)**:

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
        """Run historical backtest using ORATS data."""

        # Fetch historical option chains from ORATS
        historical_chains = orats_client.get_historical_chains(
            symbol, start_date, end_date
        )

        # Simulate strategy on historical data
        results = self._simulate_strategy(historical_chains, strategy_params)

        return results  # P&L, win rate, Sharpe ratio, etc.
```

**Benefits**:

- Validate strategy on years of data
- Optimize parameters empirically
- Estimate expected returns

---

### Use Case 4: IV-Based Opportunity Filtering

**Before (Price-based only)**:

```cpp
// Only check prices
bool is_profitable =
    arbitrage_profit >= min_arbitrage_profit &&
    roi_percent >= min_roi_percent;
```

**After (Price + IV)**:

```cpp
// Check prices AND IV conditions
bool is_profitable =
    arbitrage_profit >= min_arbitrage_profit &&
    roi_percent >= min_roi_percent &&
    iv_rank < max_iv_rank &&  // Avoid high IV periods
    iv_percentile < max_iv_percentile;
```

**Benefits**:

- Avoid trades during extreme volatility
- Better timing of entries
- Reduced risk from vol spikes

---

## Implementation Code

### Python ORATS Client

```python

# python/integration/orats_client.py

"""
orats_client.py - Client for ORATS API integration
"""
import logging
import requests
from typing import Dict, List, Optional
from datetime import datetime, timedelta
from functools import lru_cache

logger = logging.getLogger(__name__)

class ORATSClient:
    """
    Client for ORATS API.
    Provides options data, liquidity scores, and corporate events.
    """

    def __init__(self, api_token: str, base_url: str = "https://api.orats.io"):
        self.api_token = api_token
        self.base_url = base_url
        self._session = requests.Session()
        self._session.headers.update({
            "Authorization": f"Token {api_token}",
            "Content-Type": "application/json"
        })
        self._cache = {}
        self._cache_duration = timedelta(minutes=5)

    def get_strikes(self, ticker: str, trade_date: Optional[str] = None) -> List[Dict]:
        """
        Get option strikes with ORATS indicators.

        Args:
            ticker: Stock ticker
            trade_date: Date in YYYY-MM-DD format

        Returns:
            List of option data with ORATS indicators
        """
        cache_key = f"strikes_{ticker}_{trade_date}"
        if self._is_cached(cache_key):
            return self._cache[cache_key]["data"]

        endpoint = f"{self.base_url}/datav2/strikes"
        params = {
            "ticker": ticker,
            "tradeDate": trade_date or datetime.now().strftime("%Y-%m-%d"),
        }

        try:
            response = self._session.get(endpoint, params=params, timeout=10)
            response.raise_for_status()
            data = response.json().get("data", [])

            self._cache[cache_key] = {
                "data": data,
                "timestamp": datetime.now()
            }

            return data

        except requests.exceptions.RequestException as e:
            logger.error(f"ORATS API error: {e}")
            return []

    def get_earnings_calendar(self, ticker: str) -> Optional[Dict]:
        """Get next earnings date for a ticker."""
        # Implementation similar to above
        pass

    def get_dividend_schedule(self, ticker: str) -> Optional[Dict]:
        """Get next dividend ex-date for a ticker."""
        pass

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
struct ORATSConfig {
    bool enabled = false;
    std::string api_token;
    std::string base_url = "https://api.orats.io";

    bool use_for_liquidity = true;
    bool use_for_iv_data = true;
    bool use_for_risk_events = true;

    double min_liquidity_score = 70.0;
    double max_iv_percentile = 80.0;

    int earnings_blackout_days = 7;
    int dividend_blackout_days = 2;

    int cache_duration_seconds = 300;
    int rate_limit_per_second = 10;
};

struct Config {
    TWSConfig tws;
    StrategyParams strategy;
    RiskConfig risk;
    LogConfig logging;
    ORATSConfig orats;  // Add ORATS config

    // Existing fields...
};
```

---

## Recommended Integration Priority

### Immediate (Week 1-2)

1. **Liquidity Scoring**: Most impactful for execution quality
2. **Earnings Calendar**: Simple risk management win

### Short-term (Week 3-4)

1. **IV Data Enhancement**: Improves pricing accuracy
2. **Dividend Tracking**: Reduces early assignment risk

### Medium-term (Month 2-3)

1. **Historical Backtesting**: Validates strategy
2. **Parameter Optimization**: Data-driven improvements

### Long-term (Future)

1. **Advanced Greeks**: Portfolio-level risk
2. **Volatility Surface**: Advanced opportunity detection

---

## Cost-Benefit Analysis

### Costs

- ORATS API subscription: ~$100-300/month
- Development time: ~2-4 weeks
- Maintenance: Minimal (API is stable)

### Benefits

- **Improved Execution**: 5-10% better fills from liquidity scoring
- **Risk Reduction**: Avoid earnings/dividend events
- **Better Opportunities**: Filter by IV conditions
- **Validation**: Historical backtesting
- **Optimization**: Data-driven parameter tuning

### ROI Estimate

- If trading $50k position sizes
- 5% improvement in execution = $2,500/trade
- 1-2 better trades/month = $2,500-5,000/month
- Cost: ~$100-300/month
- **Net benefit: $2,200-4,700/month**

---

## Next Steps

1. **Obtain ORATS API Token**: Sign up at <https://orats.com>
2. **Test Delayed API**: Start with free/delayed data to test integration
3. **Implement ORATSClient**: Create Python client
4. **Enhance Liquidity Scoring**: Add ORATS liquidity fields
5. **Add Earnings Filtering**: Implement blackout periods
6. **Backtest Strategy**: Validate with historical data
7. **Upgrade to Live**: Once validated, upgrade to live data

---

## References

- ORATS Documentation: <https://orats.com/docs>
- ORATS Data API: <https://orats.com/data-api>
- ORATS Intraday API: <https://orats.com/intraday-data-api>
- ORATS Python SDK: <https://pypi.org/project/orats/>
- ORATS Core Research: <https://docs.orats.io/>

---

## Notes

- ORATS complements TWS API (doesn't replace it)
- TWS provides execution, ORATS provides analytics
- ORATS data requires separate exchange agreements for live data
- Historical/delayed data doesn't require agreements
- Consider starting with delayed data to test value before live subscription
