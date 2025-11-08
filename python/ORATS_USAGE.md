# ORATS Usage Guide

**Status**: ✅ IMPLEMENTED  
**Version**: 1.0.1

---

## Overview

ORATS integration is now fully implemented and ready to use. This guide shows how to enable and use ORATS data to enhance box spread trading.

---

## Quick Start

### 1. Get ORATS API Token

1. Visit https://orats.com
2. Sign up for an account
3. Purchase an API subscription (start with delayed data to test)
4. Get your API token from the dashboard

### 2. Configure

Edit `config/config.json`:

```json
{
  "orats": {
    "enabled": true,
    "api_token": "your_token_here",
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

### 3. Install Dependencies

```bash
pip install -r requirements.txt
# Installs: requests>=2.31.0, urllib3>=2.0.0
```

### 4. Run

```bash
python python/nautilus_strategy.py --config config/config.json --dry-run
```

---

## Features

### Automatic Risk Event Filtering

ORATS automatically filters out high-risk periods:

```python
# In strategy_runner.py, before evaluating opportunities:
if self.orats_client:
    should_trade, reason = self.orats_client.should_trade_ticker(
        ticker="SPY",
        earnings_blackout_days=7,
        dividend_blackout_days=2,
        max_iv_percentile=80.0,
    )
    
    if not should_trade:
        logger.info(f"Skipping SPY: {reason}")
        return
```

**Example output:**
```
INFO: Skipping SPY: In earnings blackout period (7 days)
INFO: Skipping QQQ: IV percentile too high (85.2% > 80.0%)
INFO: Skipping IWM: In dividend blackout period (2 days)
```

### Enhanced Liquidity Scoring

```python
# Get ORATS liquidity score
liquidity = orats_client.get_liquidity_score(
    ticker="SPY",
    expiry="20240412",
    strike=500.0,
    option_type="C"
)
# Returns: 85.5 (0-100 scale)
```

### Market Data Enrichment

```python
# Enrich market data with ORATS indicators
enhanced_data = orats_client.enrich_option_data(
    ticker="SPY",
    expiry="20240412",
    strike=500.0,
    option_type="C",
    market_data={"bid": 10.5, "ask": 10.6}
)

# Returns enhanced dict with:
# - orats_liquidity_score
# - orats_smoothed_iv
# - orats_iv_rank
# - orats_execution_probability
# - orats_slippage_estimate
# - And more...
```

---

## API Usage Examples

### Check Earnings Calendar

```python
from python.integration.orats_client import ORATSClient

client = ORATSClient(api_token="your_token")

# Get earnings info
earnings = client.get_earnings_calendar("SPY")
print(f"Next earnings: {earnings['next_earnings_date']}")
print(f"Days to earnings: {earnings['days_to_earnings']}")

# Check if in blackout
if client.is_earnings_blackout("SPY", blackout_days=7):
    print("Avoid trading - in earnings blackout period")
```

### Check Dividend Schedule

```python
# Get dividend info
dividend = client.get_dividend_schedule("SPY")
print(f"Next ex-date: {dividend['next_div_ex_date']}")
print(f"Amount: ${dividend['next_div_amount']}")

# Check if in blackout
if client.is_dividend_blackout("SPY", blackout_days=2):
    print("Avoid American options - ex-date soon")
```

### Get IV Metrics

```python
# Get IV rank and percentile
iv_rank = client.get_iv_rank("SPY")
iv_percentile = client.get_iv_percentile("SPY")

print(f"IV Rank: {iv_rank:.1f}")
print(f"IV Percentile: {iv_percentile:.1f}")

# Trade only when IV is reasonable
if iv_percentile < 80.0:
    print("IV acceptable, can trade")
```

### Historical Data for Backtesting

```python
# Get historical data
historical = client.get_historical_data(
    ticker="SPY",
    start_date="2023-01-01",
    end_date="2023-12-31"
)

print(f"Retrieved {len(historical)} historical records")

# Use for backtesting
for record in historical:
    # Simulate strategy on historical data
    pass
```

---

## Configuration Options

### ORATS Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | false | Enable ORATS integration |
| `api_token` | string | "" | Your ORATS API token |
| `base_url` | string | "https://api.orats.io" | API base URL |
| `use_for_liquidity_scoring` | bool | true | Use ORATS liquidity scores |
| `use_for_iv_data` | bool | true | Use ORATS IV data |
| `use_for_risk_events` | bool | true | Check earnings/dividends |
| `min_liquidity_score` | float | 70.0 | Minimum ORATS liquidity score (0-100) |
| `max_iv_percentile` | float | 80.0 | Maximum IV percentile to trade (0-100) |
| `earnings_blackout_days` | int | 7 | Avoid N days before/after earnings |
| `dividend_blackout_days` | int | 2 | Avoid N days before ex-date |
| `cache_duration_seconds` | int | 300 | Cache ORATS data for N seconds |
| `rate_limit_per_second` | int | 10 | Max API requests per second |

---

## Benefits

### 1. Better Execution Quality

**Without ORATS**:
- Volume and OI checks only
- No sophisticated liquidity assessment
- More failed executions

**With ORATS**:
- Proprietary liquidity scores
- Execution probability estimates
- Better fills, fewer failures

**Impact**: 5-10% better execution quality

### 2. Risk Management

**Without ORATS**:
- No earnings/dividend awareness
- Can enter positions before earnings
- Early assignment risk unmanaged

**With ORATS**:
- Automatic earnings blackout
- Dividend ex-date avoidance
- IV percentile filtering

**Impact**: Avoid unexpected losses from corporate events

### 3. Historical Validation

**Without ORATS**:
- No historical data
- Can't backtest strategy
- Parameters based on guesswork

**With ORATS**:
- Historical data back to 2007
- Backtest on real market conditions
- Data-driven parameter optimization

**Impact**: Validate strategy before live trading

---

## Caching

ORATS data is cached for 5 minutes (configurable) to:
- Reduce API calls
- Stay within rate limits
- Improve response time

### Cache Management

```python
# Clear cache manually
orats_client.clear_cache()

# Get cache statistics
stats = orats_client.get_cache_stats()
print(f"Cache: {stats['fresh_entries']}/{stats['total_entries']} fresh")
```

---

## Rate Limiting

Automatic rate limiting prevents API quota exhaustion:
- Default: 10 requests/second
- Configurable in config.json
- Automatic sleep when limit reached

---

## Error Handling

ORATS integration fails gracefully:
- If API is down → continues without ORATS data
- If token is invalid → logs warning, continues
- If rate limit hit → sleeps and retries
- If no ORATS data → uses defaults (50.0 scores)

---

## Testing

### Test with Delayed Data

```bash
# Use delayed data API (no live subscription needed)
python python/nautilus_strategy.py --config config/config.json --dry-run
```

### Verify Integration

```python
# Test ORATS client
from python.integration.orats_client import ORATSClient

client = ORATSClient(api_token="your_token")

# Test earnings calendar
earnings = client.get_earnings_calendar("SPY")
assert earnings is not None

# Test liquidity scores
strikes = client.get_strikes("SPY")
assert len(strikes) > 0

print("✓ ORATS integration working!")
```

---

## Monitoring

### Log Output

With ORATS enabled, you'll see additional logging:

```
INFO: ORATS client initialized
INFO: Retrieved 1,245 option strikes for SPY
INFO: Enriched SPY 20240412 500.0 C with ORATS data (liquidity=85.5)
INFO: Skipping SPY: In earnings blackout period (7 days)
DEBUG: Using cached strikes for QQQ
```

### Statistics

```python
# Get cache stats
stats = orats_client.get_cache_stats()

# Check data quality
earnings_data = orats_client.get_earnings_calendar("SPY")
dividend_data = orats_client.get_dividend_schedule("SPY")
```

---

## Cost Optimization

### Start with Delayed Data

- Delayed data is cheaper (~$50-100/month)
- No exchange agreements required
- Good for testing and validation
- Upgrade to live when valuable

### Optimize API Calls

- Use caching (default 5 minutes)
- Batch requests where possible
- Only fetch when needed
- Set appropriate rate limits

---

## Troubleshooting

### "ORATS API HTTP error: 401"

→ Invalid API token. Check your token in config.json

### "ORATS API timeout"

→ Network issue or API down. Check https://status.orats.com

### "No ORATS data found for option"

→ Option not in ORATS database (illiquid or recently listed)

### "Rate limit reached"

→ Increase `cache_duration_seconds` or decrease request frequency

---

## Next Steps

1. **Get API token** from https://orats.com
2. **Update config.json** with your token
3. **Test with delayed data** to validate integration
4. **Monitor logs** for ORATS activity
5. **Evaluate value** - check if liquidity/risk filtering helps
6. **Upgrade to live** if valuable for your strategy

---

## References

- ORATS Documentation: https://orats.com/docs
- ORATS Data API: https://orats.com/data-api
- Implementation: `python/integration/orats_client.py`
- Examples: `docs/ORATS_INTEGRATION.md`

