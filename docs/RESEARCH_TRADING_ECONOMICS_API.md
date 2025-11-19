# Trading Economics API Research

**Date:** 2025-11-18
**Status:** Research Complete
**Related Tasks:** T-73

## Executive Summary

This document summarizes research on the Trading Economics API for accessing Israeli Consumer Price Index (CPI) data. The goal is to integrate inflation data into the investment strategy framework for CPI-linked loan adjustments and inflation-adjusted return calculations.

## Use Case

### Requirements from Investment Strategy Framework

The investment strategy framework requires Israeli CPI data for:

1. **CPI-Linked Loan Adjustments:**
   - Fixed-rate CPI-linked loans in Israeli Shekel (ILS)
   - Principal value adjusts with CPI
   - Real value calculation requires current CPI data

2. **Inflation-Adjusted Returns:**
   - Calculate real returns adjusted for Israeli inflation
   - Compare investment performance against inflation

3. **Strategy Rebalancing:**
   - Trigger rebalancing when CPI changes exceed thresholds
   - Adjust allocation based on inflation environment

## Trading Economics API Overview

### Data Coverage

**Source:** [Trading Economics API Documentation](https://api.tradingeconomics.com/)

- **Over 300,000 economic indicators** available
- **Country Coverage:** 196 countries including Israel
- **Data Types:**
  - Economic indicators (CPI, GDP, unemployment, etc.)
  - Exchange rates
  - Stock market indexes
  - Government bond yields
  - Commodity prices

### Data Formats

- **Export Formats:** XML, CSV, or JSON
- **Real-time Data:** Available for subscribed indicators
- **Historical Data:** Full historical access
- **Update Frequency:** Varies by indicator (CPI typically monthly)

## API Access

### Python Package

**Installation:**
```bash
pip install tradingeconomics
```

**Authentication:**
```python
import tradingeconomics as te
te.login('api_key:api_secret')
```

**Usage Example:**
```python
import tradingeconomics as te

# Authenticate
te.login('your_api_key:your_api_secret')

# Get Israeli CPI data
cpi_data = te.getIndicatorData(
    country=['israel'],
    indicator=['consumer price index cpi'],
    output_type='df'  # Returns pandas DataFrame
)

print(cpi_data)
```

### API Endpoints

**Key Endpoints for Israeli CPI:**
- `getIndicatorData()` - Get specific indicator data
- `getHistoricalData()` - Historical time series
- `getCalendar()` - Economic calendar with CPI release dates
- `getMarkets()` - Market data and indicators

### Data Structure

**Israeli CPI Data Format:**
- **Country:** Israel
- **Indicator:** Consumer Price Index (CPI)
- **Frequency:** Monthly
- **Base Period:** October 1951 (index points)
- **Update Schedule:** Typically released mid-month for previous month

## Integration Strategy

### 1. API Setup

**Requirements:**
- Trading Economics API subscription (free tier may have limitations)
- API key and secret for authentication
- Python package installation

**Configuration:**
```python
# config/api_config.py
TRADING_ECONOMICS_API_KEY = os.getenv('TRADING_ECONOMICS_API_KEY')
TRADING_ECONOMICS_API_SECRET = os.getenv('TRADING_ECONOMICS_API_SECRET')
```

### 2. Data Access Pattern

**Monthly Update Schedule:**
```python
import tradingeconomics as te
from datetime import datetime, timedelta

def get_israeli_cpi():
    """Fetch latest Israeli CPI data"""
    te.login(f'{API_KEY}:{API_SECRET}')

    # Get latest CPI value
    cpi_data = te.getIndicatorData(
        country=['israel'],
        indicator=['consumer price index cpi'],
        output_type='df'
    )

    return cpi_data
```

**Historical Data Access:**
```python
def get_israeli_cpi_history(start_date, end_date):
    """Fetch historical Israeli CPI data"""
    te.login(f'{API_KEY}:{API_SECRET}')

    historical_data = te.getHistoricalData(
        country='israel',
        indicator='consumer price index cpi',
        initDate=start_date,
        endDate=end_date
    )

    return historical_data
```

### 3. Caching Strategy

**Rationale:**
- CPI data updates monthly (not real-time)
- Reduces API calls and costs
- Improves performance

**Implementation:**
```python
from functools import lru_cache
from datetime import datetime

@lru_cache(maxsize=1)
def get_cached_cpi(cache_date):
    """Cached CPI data (refresh monthly)"""
    return get_israeli_cpi()

def get_current_cpi():
    """Get current CPI with monthly caching"""
    # Use first day of month as cache key
    cache_key = datetime.now().replace(day=1).date()
    return get_cached_cpi(cache_key)
```

### 4. Error Handling

**Fallback Strategy:**
```python
def get_cpi_with_fallback():
    """Get CPI data with fallback to last known value"""
    try:
        return get_israeli_cpi()
    except APIError as e:
        logger.warning(f"API error: {e}, using cached value")
        return get_last_known_cpi()  # From database/cache
    except Exception as e:
        logger.error(f"Unexpected error: {e}")
        raise
```

## Integration with Investment Strategy Framework

### 1. CPI-Linked Loan Adjustments

**Calculation:**
```python
def adjust_cpi_linked_loan(principal, base_cpi, current_cpi):
    """Adjust CPI-linked loan principal for inflation"""
    adjustment_factor = current_cpi / base_cpi
    adjusted_principal = principal * adjustment_factor
    return adjusted_principal
```

### 2. Inflation-Adjusted Returns

**Real Return Calculation:**
```python
def calculate_real_return(nominal_return, inflation_rate):
    """Calculate real return adjusted for inflation"""
    real_return = ((1 + nominal_return) / (1 + inflation_rate)) - 1
    return real_return
```

### 3. Strategy Rebalancing Triggers

**CPI Change Threshold:**
```python
def should_rebalance(current_cpi, last_cpi, threshold=0.02):
    """Check if CPI change exceeds rebalancing threshold"""
    cpi_change = abs((current_cpi - last_cpi) / last_cpi)
    return cpi_change >= threshold  # 2% threshold
```

## Pricing and Limitations

### Subscription Tiers

**Free Tier:**
- Limited API calls per month
- Delayed data (may not be real-time)
- Basic indicators only

**Paid Tiers:**
- Higher API call limits
- Real-time data access
- Full indicator coverage
- Priority support

### Rate Limits

- Varies by subscription tier
- Typical free tier: 100-500 calls/month
- Paid tiers: 1,000-10,000+ calls/month

### Recommendations

- **Development/Testing:** Free tier sufficient
- **Production:** Evaluate paid tier based on:
  - Update frequency requirements
  - Number of indicators needed
  - Real-time vs. delayed data needs

## Alternative Data Sources

### 1. Israeli Central Bank (Bank of Israel)
- **Pros:** Official source, free
- **Cons:** May require web scraping, manual updates

### 2. IMF/World Bank APIs
- **Pros:** Free, official data
- **Cons:** Delayed updates, less frequent

### 3. Bloomberg/Reuters
- **Pros:** Professional-grade, comprehensive
- **Cons:** Expensive, overkill for this use case

## Implementation Plan

### Phase 1: Basic Integration
1. Set up Trading Economics API credentials
2. Install Python package
3. Implement basic CPI data fetching
4. Add to investment strategy framework

### Phase 2: Caching and Optimization
1. Implement monthly caching strategy
2. Add error handling and fallbacks
3. Optimize API call frequency

### Phase 3: Advanced Features
1. Historical data analysis
2. Inflation trend analysis
3. Automated rebalancing triggers

## References

- [Trading Economics API Documentation](https://api.tradingeconomics.com/)
- [Trading Economics Python Package (PyPI)](https://pypi.org/project/tradingeconomics/)
- [Israeli CPI Data (Trading Economics)](https://tradingeconomics.com/israel/consumer-price-index-cpi)
- [Investment Strategy Framework](../docs/INVESTMENT_STRATEGY_FRAMEWORK.md)

## Next Steps

1. **API Subscription:** Obtain Trading Economics API key
2. **Integration:** Implement basic CPI data fetching
3. **Testing:** Validate data accuracy and update frequency
4. **Caching:** Implement monthly caching strategy
5. **Framework Integration:** Connect to investment strategy framework
6. **Documentation:** Update framework documentation with CPI integration
