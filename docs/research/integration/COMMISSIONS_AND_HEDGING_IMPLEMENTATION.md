# Commissions and Hedging Implementation Summary

**Date**: 2025-01-27
**Status**: ✅ Core Functionality Complete

---

## Overview

This document summarizes the implementation of:
1. **IBKR Pro Commission System** - Accurate commission calculation using IBKR Pro published rates
2. **Interest Rate Futures Hedging** - Hedge box spread positions using SOFR, Eurodollar, and other interest rate futures
3. **Currency Hedging** - Hedge currency exposure (e.g., USD to ILS) for box spread positions

---

## 1. IBKR Pro Commission System

### Commission Configuration

**Files Modified**:
- `native/include/config_manager.h` - Added `CommissionConfig` structure
- `native/include/box_spread_strategy.h` - Added IBKR Pro commission methods
- `native/src/box_spread_strategy.cpp` - Added IBKR Pro commission calculations

**New Structures**:
```cpp
enum class CommissionTier {
    Standard,    // 0-10,000 contracts/month: $0.65/contract
    Tier1,       // 10,001-50,000: $0.60/contract
    Tier2,       // 50,001-100,000: $0.55/contract
    Tier3        // 100,001+: $0.50/contract
};

struct CommissionConfig {
    bool use_ibkr_pro_rates = true;
    double per_contract_fee = 0.65;      // Default rate
    double minimum_order_fee = 1.00;     // Min per order (waived if monthly > $30)
    double maximum_order_fee_pct = 1.0;  // Max 1% of trade value
    int monthly_contract_volume = 0;     // Track volume for tier calculation
    CommissionTier current_tier = CommissionTier::Standard;
};
```

**Features**:
- ✅ Volume-based tiered pricing (0-10K, 10K-50K, 50K-100K, 100K+ contracts/month)
- ✅ Automatic tier calculation based on monthly volume
- ✅ Minimum order fee handling ($1.00, waived if monthly commissions > $30)
- ✅ Maximum order fee cap (1% of trade value for orders < $1/share)

**New Methods**:
```cpp
// Calculate commission using IBKR Pro rates
static double calculate_commission_ibkr_pro(
    const types::BoxSpreadLeg& spread,
    const config::CommissionConfig& commission_config
);

// Calculate total cost including IBKR Pro commission
static double calculate_total_cost_ibkr_pro(
    const types::BoxSpreadLeg& spread,
    const config::CommissionConfig& commission_config
);

// Calculate effective rate using IBKR Pro commission
static double calculate_effective_interest_rate(
    const types::BoxSpreadLeg& spread,
    const config::CommissionConfig& commission_config
);
```

**Usage Example**:
```cpp
// Setup commission config
config::CommissionConfig commission_config;
commission_config.use_ibkr_pro_rates = true;
commission_config.monthly_contract_volume = 25000;  // Track volume
commission_config.current_tier = commission_config.get_tier_from_volume(25000);

// Calculate commission for box spread
double commission = BoxSpreadCalculator::calculate_commission_ibkr_pro(spread, commission_config);
double effective_rate = BoxSpreadCalculator::calculate_effective_interest_rate(spread, commission_config);
```

### Commission Rates (IBKR Pro)

| Monthly Volume | Per Contract Fee |
|----------------|------------------|
| 0 - 10,000 | $0.65 |
| 10,001 - 50,000 | $0.60 |
| 50,001 - 100,000 | $0.55 |
| 100,001+ | $0.50 |

**Box Spread Commission** (4 legs):
- Entry: 4 contracts × tier rate
- Exit: 4 contracts × tier rate
- Total round trip: 8 contracts × tier rate

**Example**:
- Standard tier: 8 × $0.65 = $5.20 per box spread round trip
- Tier 3 (high volume): 8 × $0.50 = $4.00 per box spread round trip

---

## 2. Interest Rate Futures Hedging

### Hedge Manager

**New File**: `native/include/hedge_manager.h` and `native/src/hedge_manager.cpp`

**Supported Futures**:
- **SOFR 3M (SR3)**: $1M contract, 0.25 bp tick, $6.25 per tick
- **SOFR 1M (SR1)**: $1M contract, 0.25 bp tick, ~$2.08 per tick
- **Eurodollar (ED)**: $1M contract, 0.25 bp tick, $6.25 per tick
- **Fed Funds (ZQ)**: $5M contract, 0.25 bp tick, $31.25 per tick

**Key Structures**:
```cpp
struct InterestRateFuture {
    InterestRateFutureType type;
    std::string symbol;
    std::string expiry;
    double current_price;
    double contract_size;
    double tick_size;
    double tick_value;
    int days_to_expiry;

    // Calculate implied rate from futures price
    double calculate_implied_rate() const;

    // Calculate hedge ratio
    double calculate_hedge_ratio(
        const types::BoxSpreadLeg& box_spread,
        double box_spread_notional
    ) const;
};

struct RateHedgeCalculation {
    InterestRateFuture future;
    int contracts_needed;        // Number of futures contracts
    double hedge_ratio;          // Hedge ratio (1.0 = full hedge)
    double hedge_cost;           // Cost of hedge position
    double basis_risk_bps;       // Basis risk in basis points
    bool is_valid;
};
```

**Features**:
- ✅ Find suitable futures contract matching box spread DTE
- ✅ Calculate hedge ratio and number of contracts needed
- ✅ Calculate basis risk (rate difference between box spread and futures)
- ✅ Calculate hedge cost (commission + bid-ask spread)
- ✅ Validate hedge viability (basis risk < 50 bps, cost < 0.1% of notional)

**Usage Example**:
```cpp
HedgeManager hedge_mgr;

// Find suitable futures hedge
auto future_opt = hedge_mgr.find_rate_future_hedge(
    box_spread,
    InterestRateFutureType::SOFR_3M
);

if (future_opt.has_value()) {
    // Calculate hedge
    auto hedge = hedge_mgr.calculate_rate_hedge(
        box_spread,
        100000.0,  // $100K notional
        future_opt.value(),
        1.0  // Full hedge (100%)
    );

    if (hedge.is_valid) {
        spdlog::info("Hedge: {} contracts, basis risk: {:.1f} bps, cost: ${:.2f}",
                    hedge.contracts_needed, hedge.basis_risk_bps, hedge.hedge_cost);
    }
}
```

---

## 3. Currency Hedging

### Currency Hedge Manager

**Key Structures**:
```cpp
struct CurrencyHedge {
    std::string base_currency;      // Base currency (e.g., "USD")
    std::string hedge_currency;     // Hedge currency (e.g., "ILS")
    std::string pair_symbol;        // Currency pair (e.g., "USDILS")
    double current_rate;             // Current exchange rate
    double hedge_rate;               // Desired hedge rate
    double exposure_amount;          // Amount exposed in base currency
    double hedge_amount;             // Amount to hedge in hedge currency

    // Calculate hedge amount needed
    double calculate_hedge_amount(double exposure_usd) const;

    // Calculate hedge cost
    double calculate_hedge_cost() const;
};

struct CompleteHedge {
    RateHedgeCalculation rate_hedge;
    CurrencyHedge currency_hedge;
    double total_hedge_cost;
    double total_hedge_cost_bps;  // Cost as basis points
    bool is_viable;
};
```

**Features**:
- ✅ Calculate currency hedge for box spread positions
- ✅ Support for USD to ILS and other currency pairs
- ✅ Calculate hedge amount and cost
- ✅ Combined hedging (interest rate + currency)

**Usage Example**:
```cpp
HedgeManager hedge_mgr;

// Calculate currency hedge (USD -> ILS)
auto currency_hedge = hedge_mgr.calculate_currency_hedge(
    box_spread,
    "USD",      // Base currency
    "ILS",      // Hedge currency
    100000.0    // $100K exposure
);

spdlog::info("Currency hedge: {} {} -> {} {} (rate: {:.4f})",
            currency_hedge.exposure_amount, currency_hedge.base_currency,
            currency_hedge.hedge_amount, currency_hedge.hedge_currency,
            currency_hedge.current_rate);

// Calculate combined hedge (rate + currency)
HedgeStrategy strategy;
strategy.hedge_interest_rate = true;
strategy.hedge_currency = true;
strategy.hedge_currency_code = "ILS";

auto complete_hedge = hedge_mgr.calculate_complete_hedge(
    box_spread,
    100000.0,
    strategy
);

if (complete_hedge.is_viable) {
    spdlog::info("Complete hedge viable: cost ${:.2f} ({:.1f} bps)",
                complete_hedge.total_hedge_cost, complete_hedge.total_hedge_cost_bps);
}
```

---

## 4. Integration with Yield Curves

### Updated Yield Curve Calculations

Yield curve calculations now use IBKR Pro commission rates for accurate effective rate calculations:

```cpp
// Build yield curve with IBKR Pro commissions
types::YieldCurve curve = strategy.build_yield_curve(
    "SPX",
    100.0,  // $100 strike width
    5.0,    // 5% benchmark rate
    7,      // Min DTE
    180     // Max DTE
);

// Effective rates in curve points include IBKR Pro commissions
for (const auto& point : curve.points) {
    // point.effective_rate already includes IBKR Pro commissions
    spdlog::info("DTE {}: {:.2f}% effective rate (after commissions)",
                point.days_to_expiry, point.effective_rate);
}
```

---

## 5. Documentation

### Related Documents

1. **IBKR Pro Commissions**: `docs/IBKR_PRO_COMMISSIONS.md`
   - Detailed commission rate structure
   - Volume tier thresholds
   - Box spread commission examples

2. **API Documentation**: `docs/API_DOCUMENTATION_INDEX.md`
   - Reference to IBKR pricing page

### Code References

**Commission Configuration**:
- `native/include/config_manager.h` - `CommissionConfig` structure
- `native/include/box_spread_strategy.h` - Commission calculation methods
- `native/src/box_spread_strategy.cpp` - Commission calculation implementations

**Hedging**:
- `native/include/hedge_manager.h` - Hedge manager interface
- `native/src/hedge_manager.cpp` - Hedge manager implementation

---

## 6. Future Enhancements

### Phase 2: Market Data Integration

1. **Real-time Futures Prices**:
   - Integrate TWS API for live futures quotes
   - Update hedge calculations with current prices

2. **Real-time Exchange Rates**:
   - Fetch currency rates from TWS API
   - Support more currency pairs

3. **Hedge Monitoring**:
   - Monitor hedge effectiveness in real-time
   - Automatic rebalancing when drift exceeds threshold

4. **Hedge Optimization**:
   - Optimize hedge ratio based on correlation
   - Dynamic hedging based on market conditions

---

## 7. Testing Recommendations

### Unit Tests Needed

1. **Commission Calculations**:
   - Test tier calculation based on volume
   - Test minimum order fee application
   - Test effective rate with commissions

2. **Rate Hedging**:
   - Test hedge ratio calculation
   - Test basis risk calculation
   - Test contract sizing

3. **Currency Hedging**:
   - Test hedge amount calculation
   - Test exchange rate conversion
   - Test hedge cost calculation

### Integration Tests Needed

1. **End-to-End Hedging Flow**:
   - Find opportunity → Calculate hedge → Execute → Monitor

2. **Combined Hedging**:
   - Interest rate + currency hedging together

---

## Conclusion

✅ **IBKR Pro Commission System**: Complete with volume tiers
✅ **Interest Rate Futures Hedging**: Framework complete, requires market data integration
✅ **Currency Hedging**: Framework complete, requires market data integration

The implementation provides a solid foundation for accurate commission calculation and hedging functionality. The framework is ready for integration with real-time market data sources.

**Status**: Ready for testing and market data integration.
