# LiveVol Quoted Spreads Guide

**Date**: 2025-01-27
**Purpose**: Explore LiveVol's capabilities for accessing quoted box spreads and CBOE QSB data

---

## Summary

**Can LiveVol Get Quoted Spreads?**: ⚠️ **POSSIBLY** - Needs verification during trial

### Key Findings

1. **LiveVol API Exists**: `https://api.livevol.com/v1`
2. **CBOE Subsidiary**: LiveVol is owned by CBOE Global Markets
3. **Options Data**: LiveVol provides comprehensive options data
4. **Strategy Quotes**: API contract mentions "Cboe strategy quotes"
5. **QSB Access**: Unknown - needs exploration during trial

---

## LiveVol Overview

### What is LiveVol?

- **Company**: LiveVol (subsidiary of CBOE Global Markets)
- **Product**: Options market data and analytics platform
- **API**: <https://api.livevol.com/v1>
- **Trial**: 15-day free trial available
- **Subscription**: $380/month after trial

### LiveVol Capabilities

**Historical Data**:
- Options, equities, indexes, ETFs
- Data back to 2004
- Implied volatilities and Greeks
- Time and sales data

**Real-time Data**:
- Current market prices
- Options quotes
- Earnings, dividends
- Strategy scans

**Analytics**:
- Option strategy scanning
- Backtesting capabilities
- Volatility analysis
- Greeks calculations

---

## Quoted Spreads Access

### Current Status: ⚠️ UNKNOWN

**What We Know**:
- LiveVol is a CBOE subsidiary
- LiveVol provides options data
- API contract mentions "Cboe strategy quotes"
- QSB is a CBOE service

**What We Need to Verify**:
- ✅ Does LiveVol API expose QSB instruments?
- ✅ Can we get quoted box spread prices?
- ✅ What endpoints provide strategy quotes?
- ✅ Is real-time QSB data available?
- ✅ What authentication/subscription is required?

---

## Existing Codebase Integration

### Current Implementation

**Location**: `native/src/tui_provider.cpp` (lines 553-677)

**Status**: ⚠️ **Stub Implementation** - Not fully implemented

**Key Code**:
```cpp
class LiveVolProvider : public Provider {
  // Base URL: https://api.livevol.com/v1
  // OAuth 2.0 authentication
  // Real-time or delayed data
};
```

**Current Capabilities**:
- ✅ OAuth 2.0 authentication structure
- ✅ Base URL configuration
- ⚠️ HTTP client not fully implemented
- ❌ No quoted spreads integration yet

**API Contract Reference**:
```jsonc
// agents/shared/API_CONTRACT.md
"Livevol Integration Note"
- When Livevol credentials are present, backend should enrich
  symbols[].candle and positions[] data with Cboe strategy quotes.
```

---

## LiveVol API Exploration Plan

### Day 1: API Discovery

**Tasks**:
1. ✅ Access LiveVol API Documentation
   - URL: <https://api.livevol.com/v1/docs/>
   - Document all available endpoints
   - Identify strategy/spread-related endpoints

2. ✅ Test Authentication
   - OAuth 2.0 flow
   - Get access token
   - Test API access

3. ✅ Search for Spread/Strategy Endpoints
   - Look for "spread", "box", "complex", "qsb", "strategy"
   - Document endpoint names and parameters
   - **Output**: `docs/livevol_api_endpoints.md`

### Day 2: Quoted Spreads Testing

**Tasks**:
1. ✅ Test Strategy Quote Endpoints
   - Try strategy quote endpoints
   - Test with SPX box spreads
   - Document response format

2. ✅ Test QSB Instrument Access
   - Search for QSB-specific endpoints
   - Test QSB instrument queries
   - Document QSB data availability

3. ✅ Test Real-time vs. Delayed
   - Compare real-time and delayed data
   - Document latency
   - Note subscription requirements

**Output**: `docs/livevol_quoted_spreads_testing.md`

### Day 3: Data Export & Integration

**Tasks**:
1. ✅ Test Data Export
   - Can we export quoted spreads?
   - What formats are available?
   - Document export capabilities

2. ✅ Integration Assessment
   - How difficult is integration?
   - What's the API rate limit?
   - Document integration requirements

**Output**: `docs/livevol_integration_assessment.md`

---

## Expected API Endpoints

### Potential Endpoints (To Verify)

Based on typical options data APIs, LiveVol might have:

1. **Strategy Quotes**
   - `GET /v1/strategy/quotes`
   - `GET /v1/strategy/box-spread`
   - `GET /v1/complex/quotes`

2. **QSB Instruments**
   - `GET /v1/qsb/instruments`
   - `GET /v1/qsb/quotes`
   - `GET /v1/complex/qsb`

3. **Options Data**
   - `GET /v1/options/quotes`
   - `GET /v1/options/chains`
   - `GET /v1/options/time-and-sales`

4. **Strategy Scans**
   - `GET /v1/strategy/scan`
   - `POST /v1/strategy/scan`

---

## Integration Strategy

### Option 1: Direct API Integration (Preferred)

**If LiveVol API supports quoted spreads**:

```python
# python/integration/livevol_client.py
class LiveVolClient:
    def get_quoted_box_spread(
        self,
        symbol: str,
        strike_low: float,
        strike_high: float,
        expiry: str
    ) -> Optional[Dict]:
        """Get quoted box spread from LiveVol API."""
        # Test endpoint during trial
        pass
```

**Advantages**:
- Direct access to quoted spreads
- Real-time data (if subscribed)
- Programmatic access

**Requirements**:
- LiveVol API subscription
- OAuth 2.0 authentication
- API endpoint availability

### Option 2: Build from Individual Legs (Fallback)

**If LiveVol doesn't support quoted spreads directly**:

```python
# Build box spread from individual option quotes
def build_box_spread_from_livevol(
    livevol_client: LiveVolClient,
    symbol: str,
    strike_low: float,
    strike_high: float,
    expiry: str
) -> Optional[BoxSpread]:
    """Build box spread from LiveVol individual option quotes."""
    # Get 4 individual option quotes
    long_call = livevol_client.get_option_quote(symbol, expiry, strike_low, "C")
    short_call = livevol_client.get_option_quote(symbol, expiry, strike_high, "C")
    long_put = livevol_client.get_option_quote(symbol, expiry, strike_high, "P")
    short_put = livevol_client.get_option_quote(symbol, expiry, strike_low, "P")

    # Build box spread
    return BoxSpread.from_legs(long_call, short_call, long_put, short_put)
```

**Advantages**:
- Works with standard options data
- No special subscription needed
- Flexible (any strike combination)

**Disadvantages**:
- Not direct quoted spreads
- May miss QSB opportunities
- Execution risk (partial fills)

---

## Trial Exploration Script

### Script: `scripts/livevol_api_explorer.py`

**Purpose**: Explore LiveVol API during trial to find quoted spread endpoints

**Features**:
- OAuth 2.0 authentication
- Endpoint discovery
- Strategy quote testing
- QSB instrument search
- Data export testing

**Usage**:
```bash
python scripts/livevol_api_explorer.py \
  --client-id YOUR_CLIENT_ID \
  --client-secret YOUR_CLIENT_SECRET \
  --output-dir docs/livevol_exploration
```

---

## Key Questions to Answer

### API Access
- [ ] What endpoints are available?
- [ ] Is there a strategy/spread endpoint?
- [ ] Can we query QSB instruments?
- [ ] What authentication is required?

### Quoted Spreads
- [ ] Can we get quoted box spread prices?
- [ ] What data fields are available?
- [ ] Is real-time data available?
- [ ] What's the data latency?

### QSB Access
- [ ] Does LiveVol expose QSB instruments?
- [ ] Can we get QSB quotes?
- [ ] What QSB instruments are available?
- [ ] Is QSB data real-time or delayed?

### Integration
- [ ] What's the API rate limit?
- [ ] Can we export data programmatically?
- [ ] What's the integration complexity?
- [ ] Are there SDKs/libraries available?

### Costs
- [ ] What's included in the trial?
- [ ] What requires a subscription?
- [ ] What are the subscription costs?
- [ ] Are there usage-based fees?

---

## Recommendations

### During Trial (This Week)

1. **🔴 HIGH PRIORITY**: Test API for quoted spread endpoints
   - Focus on strategy/complex/QSB endpoints
   - Document any quoted spread capabilities
   - Test with SPX box spreads

2. **🔴 HIGH PRIORITY**: Verify QSB Access
   - Search for QSB-specific endpoints
   - Test QSB instrument queries
   - Document QSB data availability

3. **🟡 MEDIUM PRIORITY**: Test Data Export
   - Can we export quoted spreads?
   - What formats are available?
   - Document export capabilities

### After Trial

**If LiveVol supports quoted spreads**:
- ✅ Integrate LiveVol API for quoted spreads
- ✅ Use as primary source for QSB quotes
- ✅ Consider subscription if valuable

**If LiveVol doesn't support quoted spreads**:
- ⚠️ Use LiveVol for individual options data
- ⚠️ Build box spreads from individual legs
- ⚠️ Continue searching for QSB quote sources

---

## Next Steps

1. **Sign up for LiveVol Pro trial** (if not already done)
   - URL: <https://datashop.cboe.com/livevol-pro>
   - Get API credentials (client ID, client secret)

2. **Run API exploration script**
   ```bash
   python scripts/livevol_api_explorer.py \
     --client-id YOUR_CLIENT_ID \
     --client-secret YOUR_CLIENT_SECRET
   ```

3. **Test quoted spread endpoints**
   - Search API docs for strategy/spread endpoints
   - Test with SPX box spreads
   - Document findings

4. **Document results**
   - Create `docs/livevol_quoted_spreads_findings.md`
   - Update integration plan based on findings

---

## References

- **LiveVol API Docs**: <https://api.livevol.com/v1/docs/>
- **LiveVol Data Shop**: <https://datashop.cboe.com/>
- **CBOE QSB FAQ**: <https://cdn.cboe.com/resources/membership/Quoted_Spread_Book_FAQ.pdf>
- **Existing Code**: `native/src/tui_provider.cpp` (LiveVolProvider)

---

**Last Updated**: 2025-01-27
