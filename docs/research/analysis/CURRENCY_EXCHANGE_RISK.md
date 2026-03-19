# Currency Exchange Risk in Box Spread Trading

**Date**: 2025-01-27
**Focus**: Managing currency exchange risk when trading USD-denominated box spreads with non-USD account currencies

---

## Overview

When trading box spreads on US exchanges (CBOE for SPX/SPXW options), positions are denominated in USD, but your account may be denominated in a different currency (e.g., ILS, EUR, GBP). Currency fluctuations between trade entry and exit can significantly impact realized returns, even if the box spread itself is profitable in USD terms.

---

## The Currency Risk Problem

### Example Scenario: USD/ILS

**Initial Setup**:

- Account currency: ILS (Israeli Shekel)
- Box spread: SPX options (USD-denominated)
- Entry date: January 1, 2025
- USD/ILS exchange rate: 3.65
- Box spread notional: $100,000 USD
- Box spread yield: 5.0% APR (USD)

**Currency Risk**:

- If USD/ILS appreciates (USD strengthens): ILS returns increase
- If USD/ILS depreciates (USD weakens): ILS returns decrease

**Example Outcomes**:

| Scenario | USD/ILS Rate Change | USD Return | ILS Return | Impact |
|----------|-------------------|------------|------------|--------|
| **Entry** | 3.65 | - | - | - |
| **USD Strengthens** | 3.80 (+4.1%) | 5.0% | 9.4% | +4.4% |
| **No Change** | 3.65 (0%) | 5.0% | 5.0% | 0% |
| **USD Weakens** | 3.50 (-4.1%) | 5.0% | 0.6% | -4.4% |

**Key Insight**: A 4.1% currency move can turn a 5.0% USD return into either 9.4% or 0.6% in ILS terms.

---

## Currency Risk Components

### 1. Trade Entry Risk

**When**: Converting account currency to USD for trade entry

**Risk**: Exchange rate at entry may be unfavorable

**Mitigation**:

- Use limit orders for currency conversion
- Monitor exchange rates before large trades
- Consider currency futures/forwards for hedging

### 2. Position Holding Risk

**When**: During the life of the box spread position

**Risk**: Currency fluctuations affect unrealized P&L

**Impact**:

- Daily mark-to-market in account currency
- Margin requirements may fluctuate
- Unrealized gains/losses in account currency

**Mitigation**:

- Currency hedging (futures, forwards, options)
- Monitor currency exposure continuously
- Set currency risk limits

### 3. Trade Exit Risk

**When**: Converting USD proceeds back to account currency

**Risk**: Exchange rate at exit may be unfavorable

**Mitigation**:

- Plan exit timing around currency considerations
- Use currency hedging to lock in rates
- Consider rolling positions if currency moves are unfavorable

---

## Currency Risk Calculation

### Exposure Calculation

```
Currency Exposure = Position Notional × Exchange Rate
```

**Example**:

- Box spread notional: $100,000 USD
- USD/ILS rate: 3.65
- ILS exposure: 365,000 ILS

### Risk Metrics

#### 1. Currency Delta

**Definition**: Sensitivity of position value to exchange rate changes

```
Currency Delta = Position Notional (USD)
```

**Example**:

- $100,000 USD position
- Currency delta: $100,000
- 1% USD/ILS move = $1,000 USD impact

#### 2. Currency VaR (Value at Risk)

**Definition**: Potential loss from currency moves at a given confidence level

```
Currency VaR = Position Notional × Exchange Rate Volatility × Z-Score
```

**Example** (95% confidence, 10% annual volatility):

- Position: $100,000 USD
- Rate: 3.65
- Daily volatility: 10% / √252 ≈ 0.63%
- 95% Z-score: 1.96
- Daily VaR: $100,000 × 0.63% × 1.96 ≈ $1,235 USD

#### 3. Currency Impact on Returns

```
ILS Return = USD Return + Currency Return + (USD Return × Currency Return)
```

**Example**:

- USD return: 5.0%
- Currency return: +4.1% (USD strengthens)
- ILS return: 5.0% + 4.1% + (5.0% × 4.1%) ≈ 9.4%

---

## Hedging Strategies

### 1. Currency Futures

**How**: Short currency futures to hedge long USD exposure

**Example (USD/ILS)**:

- Long $100,000 USD box spread
- Short ILS futures equivalent to $100,000 USD
- Lock in exchange rate for position duration

**Pros**:

- Direct hedge
- Exchange-traded (liquid, transparent)
- No counterparty risk (cleared)

**Cons**:

- Basis risk (futures vs. spot)
- Margin requirements
- Roll costs for longer positions

### 2. Currency Forwards

**How**: Enter forward contract to lock exchange rate

**Example**:

- Long $100,000 USD box spread
- Forward contract: Sell $100,000 USD, buy ILS at fixed rate
- Settlement at box spread maturity

**Pros**:

- Exact hedge (no basis risk)
- Customizable terms
- No margin (for qualified parties)

**Cons**:

- Counterparty risk
- Less liquid than futures
- May require credit lines

### 3. Currency Options

**How**: Use currency options for asymmetric hedging

**Example**:

- Long $100,000 USD box spread
- Buy USD put / ILS call to protect against USD weakness
- Retain upside if USD strengthens

**Pros**:

- Asymmetric protection
- Limited downside (premium only)
- Flexible strike selection

**Cons**:

- Premium cost
- Time decay
- More complex than futures/forwards

### 4. Natural Hedging

**How**: Offset currency exposure with other positions

**Example**:

- Long USD box spread (long USD exposure)
- Short USD-denominated asset (short USD exposure)
- Net currency exposure: zero

**Pros**:

- No explicit hedge cost
- Natural portfolio offset

**Cons**:

- Requires offsetting positions
- May not be available
- Correlation risk

---

## Implementation in Codebase

### Existing Infrastructure

The codebase already includes currency hedging support:

**Location**: `native/include/hedge_manager.h`, `native/src/hedge_manager.cpp`

**Key Structures**:

```cpp
struct CurrencyHedge {
    std::string base_currency;      // Base currency (e.g., "USD")
    std::string hedge_currency;     // Hedge currency (e.g., "ILS")
    std::string pair_symbol;        // Currency pair (e.g., "USDILS")
    double current_rate;             // Current exchange rate
    double exposure_amount;          // Amount exposed in base currency
    double hedge_amount;             // Amount to hedge in hedge currency
    double calculate_hedge_amount(double exposure_usd) const;
    double calculate_hedge_cost() const;
};
```

**Usage Example**:

```cpp
// Calculate currency hedge for box spread
auto currency_hedge = hedge_mgr.calculate_currency_hedge(
    box_spread,
    "USD",      // Base currency
    "ILS",      // Hedge currency
    box_spread_notional
);

// Calculate hedge cost
double currency_cost = hedge_mgr.calculate_currency_hedge_cost(currency_hedge);
```

### Configuration

**Strategy Configuration** (`config::StrategyParams`):

```cpp
struct StrategyParams {
    // Currency hedging
    bool hedge_currency = false;
    std::string hedge_currency_code;  // e.g., "ILS"
    double currency_hedge_ratio = 1.0;  // Full hedge (1.0) or partial (0.5)
};
```

### Exchange Rate Data

**Current Implementation**: Stub with hardcoded rates

**Location**: `native/src/hedge_manager.cpp`

```cpp
double HedgeManager::get_exchange_rate(
    const std::string& base_currency,
    const std::string& hedge_currency) const {

    // Stub implementation
    if (base_currency == "USD" && hedge_currency == "ILS") {
        return 3.65;  // Approximate USD/ILS rate
    }
    // ...
}
```

**Future Enhancement**: Integrate with TWS API for real-time rates

---

## Real-Time Exchange Rate Integration

### TWS API Currency Data

**IBKR TWS API** provides currency market data:

```cpp
// Request currency contract
Contract currency_contract;
currency_contract.symbol = "USD";
currency_contract.secType = "CASH";
currency_contract.currency = "ILS";
currency_contract.exchange = "IDEALPRO";

// Request market data
tws_client->reqMktData(
    reqId,
    currency_contract,
    "233",  // Generic tick: last price
    false,  // Snapshot
    false   // Regulatory snapshot
);
```

**Available Currency Pairs**:

- USD/ILS (US Dollar / Israeli Shekel)
- USD/EUR (US Dollar / Euro)
- USD/GBP (US Dollar / British Pound)
- USD/JPY (US Dollar / Japanese Yen)
- And many more...

### Alternative Data Sources

1. **Forex APIs**:
   - OANDA API
   - Alpha Vantage (forex data)
   - Finnhub (forex data)
   - **⚠️ FXCM Restriction**: FXCM does not allow residents of Israel (not available for Israeli traders)

2. **Exchange APIs**:
   - CME Group (currency futures)
   - ICE (currency futures)

3. **Market Data Providers**:
   - Bloomberg
   - Reuters
   - Interactive Brokers (TWS API)

---

## Risk Management Framework

### 1. Currency Risk Limits

**Position Limits**:

- Maximum currency exposure per position
- Maximum total currency exposure
- Maximum currency exposure per currency pair

**Example**:

```cpp
struct CurrencyRiskLimits {
    double max_exposure_per_position = 100000.0;  // USD
    double max_total_exposure = 1000000.0;        // USD
    double max_exposure_per_pair = 500000.0;      // USD
    double max_currency_var = 5000.0;            // USD (daily)
};
```

### 2. Currency Risk Monitoring

**Real-Time Metrics**:

- Current currency exposure
- Currency delta
- Currency VaR
- Unrealized currency P&L
- Currency hedge effectiveness

**Reporting**:

- Daily currency exposure report
- Currency risk dashboard
- Alert on limit breaches

### 3. Currency Hedge Effectiveness

**Measurement**:

```
Hedge Effectiveness = 1 - (Hedged P&L / Unhedged P&L)
```

**Target**: > 80% effectiveness

**Monitoring**:

- Track hedge effectiveness over time
- Rebalance hedges if effectiveness drops
- Adjust hedge ratios based on correlation

---

## Practical Examples

### Example 1: USD/ILS Box Spread with Currency Hedge

**Setup**:

- Account currency: ILS
- Box spread: SPX $100,000 USD notional
- Entry USD/ILS: 3.65
- Box spread yield: 5.0% APR (USD)
- Position duration: 30 days

**Currency Hedge**:

- Short ILS futures equivalent to $100,000 USD
- Lock rate: 3.65
- Hedge cost: 0.1% (annualized)

**Outcomes**:

| Scenario | USD/ILS | USD Return | Currency Impact | ILS Return (Unhedged) | ILS Return (Hedged) |
|---------|---------|------------|-----------------|----------------------|---------------------|
| USD +5% | 3.83 | 5.0% | +5.0% | 10.3% | 4.9% |
| No change | 3.65 | 5.0% | 0% | 5.0% | 4.9% |
| USD -5% | 3.47 | 5.0% | -5.0% | -0.3% | 4.9% |

**Result**: Hedged return is stable at ~4.9% ILS (5.0% USD - 0.1% hedge cost), regardless of currency moves.

### Example 2: Multi-Currency Portfolio

**Setup**:

- Account currencies: ILS, EUR, GBP
- Multiple box spread positions in USD
- Total USD exposure: $500,000

**Hedging Strategy**:

- Hedge each currency separately
- USD/ILS: Short ILS futures
- USD/EUR: Short EUR futures
- USD/GBP: Short GBP futures

**Portfolio Currency Risk**:

- Aggregate exposure across all positions
- Net currency delta: $500,000 USD
- Currency VaR: $3,000 USD (daily, 95% confidence)

---

## Cost-Benefit Analysis

### Currency Hedge Costs

**Futures**:

- Margin requirement: ~2-5% of notional
- Roll cost: ~0.05-0.1% per roll
- Bid-ask spread: ~0.01-0.02%

**Forwards**:

- Spread: ~0.1-0.3% (depending on pair)
- Credit line: May be required

**Options**:

- Premium: 1-3% of notional (depending on strike/maturity)
- Time decay: Ongoing cost

### When to Hedge

**Hedge When**:

- Currency exposure > 10% of account value
- Currency volatility > 10% annualized
- Position duration > 30 days
- Currency risk exceeds risk tolerance

**Don't Hedge When**:

- Currency exposure < 5% of account value
- Position duration < 7 days
- Natural hedging available
- Hedge cost > expected currency risk

---

## Integration with Box Spread Strategy

### Enhanced Profitability Calculation

**Current** (USD only):

```
Net Profit = Box Spread Yield - Transaction Costs
```

**Enhanced** (with currency):

```
Net Profit (ILS) = (Box Spread Yield - Transaction Costs) × Exchange Rate
                 - Currency Hedge Cost
                 + Currency Return (if unhedged)
```

### Strategy Configuration

```json
{
  "strategy": {
    "hedge_currency": true,
    "hedge_currency_code": "ILS",
    "currency_hedge_ratio": 1.0,
    "currency_risk_limits": {
      "max_exposure_per_position": 100000,
      "max_total_exposure": 1000000,
      "max_currency_var": 5000
    }
  }
}
```

---

## Best Practices

### 1. Currency Risk Assessment

- **Before Trade**: Calculate currency exposure
- **During Trade**: Monitor currency moves and hedge effectiveness
- **After Trade**: Analyze currency impact on returns

### 2. Hedge Selection

- **Short-term** (< 30 days): Consider not hedging (cost may exceed benefit)
- **Medium-term** (30-90 days): Currency futures or forwards
- **Long-term** (> 90 days): Currency forwards or options

### 3. Hedge Timing

- **Entry**: Hedge at trade entry to lock rate
- **Exit**: Unwind hedge at trade exit
- **Rolling**: Roll hedges for longer positions

### 4. Monitoring

- **Daily**: Check currency exposure and hedge effectiveness
- **Weekly**: Review currency risk metrics
- **Monthly**: Analyze currency impact on returns

---

## Resources

### Exchange Rate Data

- **IBKR TWS API**: Real-time currency rates
- **OANDA API**: Historical and real-time forex data
- **Alpha Vantage**: Forex API
- **Finnhub**: Forex market data

### Currency Futures

- **CME Group**: Currency futures (USD/ILS, USD/EUR, etc.)
- **ICE**: Currency futures
- **Exchange Websites**: Contract specifications and pricing

### Documentation

- **Currency Hedging Implementation**: `docs/COMMISSIONS_AND_HEDGING_IMPLEMENTATION.md`
- **Hedge Manager Code**: `agents/backend/crates/quant/` (Rust reimplementation; C++ removed)
- **API Documentation Index**: `docs/API_DOCUMENTATION_INDEX.md`

---

## Key Takeaways

1. **Currency Risk is Real**: Currency fluctuations can significantly impact returns for non-USD accounts
2. **Hedging Available**: Multiple hedging instruments (futures, forwards, options)
3. **Cost-Benefit**: Evaluate hedge cost vs. currency risk
4. **Infrastructure Exists**: Codebase has currency hedging framework
5. **Real-Time Data**: Integrate TWS API or forex APIs for exchange rates
6. **Risk Management**: Set currency risk limits and monitor exposure
7. **Best Practice**: Hedge when exposure is significant and position duration is long

---

## Related Documentation

- **Hedging Implementation**: `docs/COMMISSIONS_AND_HEDGING_IMPLEMENTATION.md` - Currency and interest rate hedging
- **API Documentation Index**: `docs/API_DOCUMENTATION_INDEX.md` - TWS API and forex data sources
- **CME Fee Schedules**: `docs/CME_FEE_SCHEDULE_REBATES.md` - Currency futures trading costs
- **Risk Calculator**: `agents/backend/crates/risk/` - Risk management framework (Rust)

---

**Note**: Currency exchange rates are highly volatile and can move significantly during the life of a box spread position. Always assess currency risk before entering positions and consider hedging when exposure is significant. Verify current exchange rates and hedge costs before making trading decisions.
