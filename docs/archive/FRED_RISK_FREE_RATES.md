# FRED Risk-Free Rate Data Sources

This document catalogs all risk-free rate and benchmark interest rate data available from FRED (Federal Reserve Economic Data, St. Louis Fed).

## Quick Reference

| Country | Rate | FRED Series | Frequency | Source |
|---------|------|-------------|-----------|--------|
| **US** | SOFR | `SOFR` | Daily | Federal Reserve |
| **US** | Fed Funds Rate | `FEDFUNDS` | Monthly | Federal Reserve |
| **UK** | SONIA | `IUDSOIA` | Daily | Bank of England |
| **Euro** | €STR | `ECBESTRVOLWGTTRMDMNRT` | Daily | ECB |
| **Switzerland** | N/A | See alternatives | - | - |
| **Israel** | T-bill (3M) | `IR3TIB01ILQ156N` | Quarterly | OECD |
| **Australia** | AONIA | RBA rates | Daily | RBA |

---

## United States

### SOFR (Secured Overnight Financing Rate)

- **Series**: `SOFR`
- **URL**: https://fred.stlouisfed.org/series/SOFR
- **Frequency**: Daily
- **Source**: Federal Reserve Bank of New York
- **Description**: Primary risk-free rate for USD, based on Treasury repurchase agreement transactions

### Fed Funds Rate

- **Series**: `FEDFUNDS`
- **URL**: https://fred.stlouisfed.org/series/FEDFUNDS
- **Frequency**: Monthly
- **Source**: Federal Reserve
- **Description**: Effective Federal Funds Rate

### Treasury Rates

- **1-Month**: `DTB3` (discontinued, use SOFR)
- **3-Month**: `DTB3`
- **6-Month**: `DTB6`
- **1-Year**: `DTB1YR`
- **10-Year**: `DTB10`
- **URL**: https://fred.stlouisfed.org/tags/series?t=treasury

---

## United Kingdom

### SONIA (Sterling Overnight Index Average)

- **Series**: `IUDSOIA`
- **URL**: https://fred.stlouisfed.org/series/IUDSOIA
- **Frequency**: Daily
- **Source**: Bank of England
- **Description**: Primary risk-free rate for GBP

### SONIA Variants

| Series | Description |
|--------|-------------|
| `IUDZLS8` | SONIA 75th percentile |
| `IUDZOS2` | SONIA Compounded Index |
| `IUDSONIOBS` | SONIA observation rate |

---

## Euro Area

### €STR (Euro Short-Term Rate)

- **Primary Series**: `ECBESTRVOLWGTTRMDMNRT`
- **URL**: https://fred.stlouisfed.org/series/ECBESTRVOLWGTTRMDMNRT
- **Frequency**: Daily
- **Source**: European Central Bank (ECB)
- **Description**: Primary risk-free rate for EUR

### €STR Variants

| Series | Description |
|--------|-------------|
| `ECBESTRTOTVOL` | €STR Total Volume |
| `ECBESTRRT75THPCTVOL` | €STR 75th percentile |
| `ECBESTRRT25THPCTVOL` | €STR 25th percentile |
| `IRSTCI01EZA156N` | EURIBOR (Annual) |

---

## Switzerland

FRED does not have direct SARON data. Options:

### Alternative Sources

1. **SIX Group** - Direct SARON data: https://www.six-group.com/en/products-services/financial-information/rates/saron.html
2. **Swiss Government Bonds**:
   - **Series**: `CHLTLT01CHM156N`
   - **URL**: https://fred.stlouisfed.org/series/CHLTLT01CHM156N
   - **Frequency**: Monthly
   - **Description**: 10-Year Government Bond Yield

### Available in FRED

| Series | Description | Frequency |
|--------|-------------|-----------|
| `CHLTLT01CHM156N` | 10-Year Bond Yield | Monthly |
| `CHIRSTCI01GPM156N` | Interbank Rate | Monthly |

---

## Israel

### Short-Term Rates (via OECD)

| Series | Description | Frequency |
|--------|-------------|-----------|
| `IRSTCI01ILM156N` | Call Money/Interbank Rate | Monthly |
| `IR3TIB01ILQ156N` | 3-Month T-bill Rate | Quarterly |

### Long-Term Rates

| Series | Description | Frequency |
|--------|-------------|-----------|
| `IRLTLT01ILM156N` | 10-Year Government Bond | Monthly |
| `IRLTLT01ILQ156N` | 10-Year Government Bond | Quarterly |

---

## Australia

### AONIA (Australian Interbank Overnight Cash Rate)

- **Series**: `RBAIORB`
- **URL**: https://fred.stlouisfed.org/series/RBAIORB
- **Frequency**: Daily
- **Source**: Reserve Bank of Australia
- **Description**: Primary risk-free rate for AUD

### Australian Rates in FRED

| Series | Description | Frequency |
|--------|-------------|-----------|
| `RBAIORB` | Overnight Cash Rate | Daily |
| `AUS10YRSEC` | 10-Year Government Bond | Monthly |

---

## Canada

### Canadian Overnight Rate

- **Series**: `CORRATENTD`
- **URL**: https://fred.stlouisfed.org/series/CORRATENTD
- **Frequency**: Daily
- **Source**: Bank of Canada
- **Description**: Target Overnight Rate

### Canadian Rates in FRED

| Series | Description | Frequency |
|--------|-------------|-----------|
| `CORRATENTD` | Overnight Rate Target | Daily |
| `CANSLT10YRSEC` | 10-Year Bond Yield | Monthly |

---

## Japan

### Yen Overnight Rate (TONAR)

- **Series**: `JPONATEL` (experimental)
- **URL**: https://fred.stlouisfed.org/series/JPONATEL
- **Frequency**: Daily
- **Source**: Bank of Japan
- **Description**: Tokyo Overnight Average Rate

### Japanese Rates in FRED

| Series | Description | Frequency |
|--------|-------------|-----------|
| `JPONATEL` | Overnight Average | Daily |
| `JPYLTTOT` | 10-Year Government Bond | Monthly |

---

## API Usage

### FRED API Endpoint

```bash
# Get latest observation
curl "https://api.stlouisfed.org/fred/series/observations?series_id=SOFR&api_key=YOUR_KEY&file_type=json&limit=1&sort_order=desc"
```

### Rate Limiting

- **Free tier**: 120 requests/minute
- **Commercial**: Higher limits available

### Authentication

Get API key at: https://fred.stlouisfed.org/docs/api/api_key.html

---

## Implementation Notes

1. **Use SOFR for USD box spreads** - Most accurate for US financing rates
2. **SONIA/€STR require API key** - FRED requires authentication
3. **Israel rates are quarterly** - Not suitable for short-term DTE calculations
4. **Swiss SARON unavailable** - Consider using CHF Government Bond yield as proxy
5. **Cache aggressively** - Rates change daily, cache for 1-24 hours
6. **yfinance fallback** - When FRED is unavailable, Treasury yields can be fetched via yfinance (^IRX, ^FVX, ^TNX, ^TYX)

---

## yfinance Fallback

When FRED API is unavailable (no API key, rate limits, network issues), the system falls back to Yahoo Finance Treasury yields.

### Installation

```bash
pip install yfinance
```

### Yahoo Finance Ticker Symbols

| Symbol | Description | Approximate DTE |
|--------|-------------|-----------------|
| `^IRX` | 13-Week Treasury Bill | 91 days |
| `^FVX` | 5-Year Treasury Note | 1,825 days |
| `^TNX` | 10-Year Treasury Note | 3,650 days |
| `^TYX` | 30-Year Treasury Bond | 10,950 days |

### Limitations

- **No true risk-free rates**: yfinance only provides Treasury yields, not SOFR/SONIA/€STR
- **Treasury ≠ Risk-Free**: Treasury yields include credit risk premium
- **International rates**: Limited data for non-USD currencies - falls back to US Treasury with warning
- **Delayed data**: May have slight delays vs real-time FRED data

### Usage in Code

```python
from python.integration.sofr_treasury_client import SOFRTreasuryClient, YFinanceRateClient

# FRED-based (primary)
client = SOFRTreasuryClient(fred_api_key="your-key")
rates = client.get_treasury_rates()

# yfinance fallback (automatic when FRED unavailable)
# Set YFINANCE_AVAILABLE=1 to enable explicit fallback
```

---

## References

- FRED API: https://fred.stlouisfed.org/docs/api/fred/
- FRED Tags: https://fred.stlouisfed.org/tags/
- Bank of England: https://www.bankofengland.co.uk/markets/sonia
- ECB €STR: https://www.ecb.europa.eu/stats/payment_and_money_money_market_structures/html/index.en.html
- yfinance GitHub: https://github.com/ranaroussi/yfinance
