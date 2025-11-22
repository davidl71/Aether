# Risk-Free Rate Calculation Methodology

**Date**: 2025-01-27
**References**:

- [New York Fed: Options for Calculating Risk-Free Rates](https://libertystreeteconomics.newyorkfed.org/2023/10/options-for-calculating-risk-free-rates/)
- [CME Group: Pricing and Hedging USD SOFR Interest Swaps with SOFR Futures](https://www.cmegroup.com/articles/2025/price-and-hedging-usd-sofr-interest-swaps-with-sofr-futures.html)

---

## Overview

This document describes how box spreads are used to calculate risk-free rates, building yield curves, and comparing with traditional benchmarks (SOFR, Treasury rates).

## Theory: Box Spreads as Risk-Free Instruments

### What is a Box Spread?

A box spread is a four-legged options strategy that creates a synthetic forward contract:

- **Long Call** at lower strike (K1)
- **Short Call** at higher strike (K2)
- **Long Put** at higher strike (K2)
- **Short Put** at lower strike (K1)

The payoff at expiration is **always** equal to the strike width (K2 - K1), regardless of the underlying price. This makes box spreads theoretically **risk-free**.

### Why Box Spreads Represent Risk-Free Rates

1. **Locked Payoff**: The payoff is fixed at strike width, independent of underlying price
2. **No Market Risk**: Delta, gamma, theta, and vega are all approximately zero
3. **Arbitrage-Free**: In efficient markets, the cost of a box spread should equal the present value of the strike width
4. **Implied Rate**: The difference between the strike width and the box spread cost implies an interest rate

### Implied Rate Calculation

For a box spread with:

- Strike width: `W = K2 - K1`
- Net debit (cost to buy): `D`
- Days to expiry: `T`

**Buying (Borrowing Scenario)**:

```
Implied Rate = ((D - W) / W) × (365 / T) × 100%
```

**Selling (Lending Scenario)**:

```
Implied Rate = ((W - C) / C) × (365 / T) × 100%
```

where `C` is the net credit received.

**Mid Rate**:

```
Mid Rate = (Buy Rate + Sell Rate) / 2
```

## Implementation

### 1. Rate Extraction (`risk_free_rate_extractor.py`)

The `RiskFreeRateExtractor` class extracts risk-free rates from box spread opportunities:

```python
from integration.risk_free_rate_extractor import RiskFreeRateExtractor

extractor = RiskFreeRateExtractor(min_liquidity_score=50.0)

# Extract rate from a single box spread
point = extractor.extract_from_box_spread(
    symbol="SPX",
    expiry="20250131",
    days_to_expiry=30,
    strike_width=50.0,
    buy_implied_rate=5.25,
    sell_implied_rate=5.15,
    net_debit=49.50,
    net_credit=49.60,
    liquidity_score=75.0
)
```

### 2. Yield Curve Construction

Aggregate rates across multiple expirations to build a term structure:

```python
# Build curve from multiple opportunities
curve = extractor.build_curve_from_opportunities(opportunities, symbol="SPX")

# Get rate at specific days to expiry
rate_30d = curve.get_rate_at_dte(30, tolerance=5)

# Filter by liquidity
liquid_curve = curve.filter_by_liquidity(min_liquidity=70.0)
```

### 3. Benchmark Comparison (`sofr_treasury_client.py`)

Compare box spread rates with traditional benchmarks:

```python
from integration.sofr_treasury_client import SOFRTreasuryClient, RateComparison

client = SOFRTreasuryClient()

# Get SOFR overnight rate
sofr = client.get_sofr_overnight()

# Compare curves
comparison = RateComparison.compare_curves(box_spread_curve, benchmark_rates)

# Calculate spread in basis points
spread_bps = RateComparison.calculate_spread(box_rate=5.20, benchmark_rate=5.00)
# Result: 20 bps
```

## API Endpoints

### Extract Rate from Box Spread

```bash
POST /api/extract-rate
Content-Type: application/json

{
  "symbol": "SPX",
  "expiry": "20250131",
  "days_to_expiry": 30,
  "strike_width": 50.0,
  "buy_implied_rate": 5.25,
  "sell_implied_rate": 5.15,
  "net_debit": 49.50,
  "net_credit": 49.60,
  "liquidity_score": 75.0
}
```

### Build Yield Curve

```bash
POST /api/build-curve
Content-Type: application/json

{
  "opportunities": [...],  # List of box spread opportunities
  "symbol": "SPX"
}
```

### Compare with Benchmarks

```bash
POST /api/compare
Content-Type: application/json

{
  "opportunities": [...],
  "symbol": "SPX"
}
```

## Comparison with SOFR and Treasury Rates

### SOFR (Secured Overnight Financing Rate)

- **Source**: Federal Reserve Bank of New York
- **Type**: Overnight secured lending rate
- **Use**: Primary risk-free rate benchmark (replaced LIBOR)
- **Term Structure**: Can be derived from SOFR futures (CME Group)

### Treasury Rates

- **Source**: U.S. Department of the Treasury
- **Type**: Government borrowing costs
- **Maturities**: 1M, 3M, 6M, 1Y, 2Y, 5Y, 10Y, 30Y
- **Use**: Traditional risk-free rate benchmark

### Box Spread Rates vs. Benchmarks

**Advantages of Box Spread Rates**:

- Market-based (reflects actual trading)
- Real-time (updates with market data)
- No counterparty risk (exchange-cleared)
- Can be constructed for any expiration

**Considerations**:

- Requires liquid options market
- Bid-ask spreads affect accuracy
- May include some liquidity premium
- Execution costs reduce effective rate

## Yield Curve Construction

### Aggregation Methods

When multiple box spreads exist at the same expiration:

1. **Weighted Average** (default): Weight by liquidity score

   ```
   Rate = Σ(Rate_i × Liquidity_i) / Σ(Liquidity_i)
   ```

2. **Best Liquidity**: Use the spread with highest liquidity

   ```
   Rate = Rate_max(liquidity)
   ```

3. **Simple Average**: Average all rates

   ```
   Rate = Σ(Rate_i) / N
   ```

### Term Structure Points

A complete yield curve includes:

- **Overnight**: 1 day
- **Short-term**: 7, 14, 30 days
- **Medium-term**: 60, 90, 180 days
- **Long-term**: 1 year, 2 years (if available)

## Use Cases

### 1. Arbitrage Detection

Compare box spread rates with SOFR/Treasury to identify:

- **Positive Spread**: Box spread rate > benchmark → potential arbitrage opportunity
- **Negative Spread**: Box spread rate < benchmark → market inefficiency or execution costs
- **Spread Analysis**: Track spreads over time to identify patterns

**Example:**

```python
comparison = RateComparison.compare_curves(box_curve, benchmarks)
for dte, comp in comparison.items():
    if comp["spread_bps"] > 20:  # 20 bps threshold
        print(f"Arbitrage opportunity at {dte} days: {comp['spread_bps']:.1f} bps")
```

### 2. Funding Cost Analysis

Use box spread rates to:

- **Estimate funding costs** for different maturities
- **Compare borrowing vs. lending rates** (buy vs. sell implied rates)
- **Analyze term structure** of funding costs
- **Optimize capital allocation** across maturities

**Example:**

```python
# Get funding cost for 30-day period
rate_30d = curve.get_rate_at_dte(30)
if rate_30d:
    funding_cost = principal * (rate_30d / 100) * (30 / 365)
    print(f"30-day funding cost: ${funding_cost:.2f}")
```

### 3. Risk-Free Rate Proxy

When SOFR/Treasury data is unavailable or delayed:

- **Real-time proxy**: Use box spread rates as immediate risk-free rate
- **Custom term structure**: Build curves for specific expirations
- **Option pricing**: Calculate discount factors for Black-Scholes
- **Yield curve modeling**: Input for term structure models

**Example:**

```python
# Use box spread rate for option pricing
risk_free_rate = curve.get_rate_at_dte(option_dte) / 100.0
discount_factor = math.exp(-risk_free_rate * (option_dte / 365.0))
```

### 4. Market Efficiency Analysis

Compare box spread rates across:

- **Different underlyings**: SPX vs. XSP vs. ES
- **Different strike widths**: 25 vs. 50 vs. 100
- **Different expirations**: Short-term vs. long-term
- **Time periods**: Intraday vs. daily vs. weekly

This helps identify:

- Market inefficiencies
- Liquidity differences
- Execution cost variations

## Data Quality Considerations

### Liquidity Filters

- **Minimum Liquidity Score**: Filter out illiquid spreads (default: 50.0)
- **Volume/Open Interest**: Ensure sufficient market depth
- **Bid-Ask Spreads**: Narrow spreads indicate better pricing

### Validation

- **Strike Width Consistency**: All points should use same strike width (or normalize)
- **Rate Reasonableness**: Rates should be within expected range (0-20% typically)
- **Expiration Alignment**: Ensure expirations match option contract dates

## References

1. **New York Fed Article** (October 2023):
   - Discusses using box spreads to calculate risk-free rates
   - Compares with SOFR and Treasury rates
   - Methodology for building term structures

2. **CME Group Article** (2025):
   - SOFR futures pricing and hedging
   - Term structure construction from futures
   - Relationship between SOFR swaps and futures

3. **Academic Literature**:
   - Put-Call Parity and Box Spread Arbitrage
   - Risk-Free Rate Extraction from Options Markets
   - Term Structure Modeling

## API Integration

### FRED API (Federal Reserve Economic Data)

The SOFR/Treasury client supports FRED API for real-time benchmark rates:

**Setup:**

1. Get free API key: <https://fred.stlouisfed.org/docs/api/api_key.html>
2. Set environment variable: `export FRED_API_KEY=your_key_here`
3. Or pass to client: `SOFRTreasuryClient(fred_api_key="your_key")`

**Available Series:**

- **SOFR**: `SOFR` - Secured Overnight Financing Rate
- **SOFR Term**: `SOFR30DAYAVG`, `SOFR90DAYAVG`, `SOFR180DAYAVG`
- **Treasury**: `DGS1MO`, `DGS3MO`, `DGS6MO`, `DGS1`, `DGS2`, `DGS5`, `DGS10`, `DGS30`

**Example:**

```python
from integration.sofr_treasury_client import SOFRTreasuryClient

client = SOFRTreasuryClient(fred_api_key="your_key")
sofr = client.get_sofr_overnight()
treasury_rates = client.get_treasury_rates()
```

### New York Fed API

Alternative source for SOFR data (structure may vary):

- Base URL: `https://markets.newyorkfed.org/api`
- Endpoint: `/rates/all` or `/rates/sofr`

## Integration with C++ Code

The Python module integrates with existing C++ yield curve building:

```python
from integration.risk_free_rate_extractor import RiskFreeRateExtractor

extractor = RiskFreeRateExtractor()

# Build from C++ YieldCurve data (via bindings)
cpp_curve = native_strategy.build_yield_curve("SPX", 50.0)
python_curve = extractor.build_curve_from_cpp_yield_curve(cpp_curve, "SPX")
```

## Future Enhancements

1. **CME SOFR Futures Integration**: Fetch SOFR futures prices to derive term rates
2. **Historical Analysis**: Compare box spread rates over time
3. **Cross-Asset Comparison**: Compare rates across SPX, XSP, ES, etc.
4. **Real-Time Updates**: WebSocket feeds for live rate updates
5. **Visualization**: Yield curve plotting and comparison charts
