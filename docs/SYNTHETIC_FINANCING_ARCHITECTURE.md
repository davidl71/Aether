# Synthetic Financing Architecture - Multi-Asset Relationship System

**Date**: 2025-01-27
**Purpose**: Design a comprehensive system for managing synthetic financing across multiple asset classes, currencies, and brokers, with complex interdependencies and collateral relationships.

---

## Overview

This system enables **synthetic financing optimization** by:

1. **Unifying financing instruments**: Box spreads, futures, T-bills, bonds, bank loans, pension loans
2. **Modeling asset relationships**: Collateral, margin, cross-product dependencies
3. **Cross-currency optimization**: Multi-currency financing with FX hedging
4. **Multi-broker aggregation**: Best execution across all available sources

---

## Core Concept: Asset Relationships

Each financial instrument can serve multiple roles and has relationships with other instruments:

### Relationship Types

1. **Collateral Relationships**
   - Bonds/T-bills/ETFs → Margin for options
   - Pension funds → Collateral for pension loans
   - Stocks → Margin for futures
   - Cash → Universal collateral

2. **Financing Relationships**
   - Box spreads → Synthetic borrowing/lending
   - Futures → Implied financing rates
   - Bank loans → Direct financing
   - Pension loans → Secured financing

3. **Cross-Currency Relationships**
   - FX swaps → Currency conversion
   - Cross-currency basis → Arbitrage opportunities
   - Currency hedging → Risk management

4. **Regulatory Relationships**
   - Portfolio margining → Margin offsets
   - Reg-T vs Portfolio margin → Capital efficiency
   - Haircuts → Collateral valuation

---

## Architecture Components

### 1. Asset Relationship Graph

```cpp
// native/include/asset_relationship.h

enum class RelationshipType {
    COLLATERAL,           // Asset can be used as collateral
    MARGIN,              // Asset can satisfy margin requirements
    FINANCING,           // Asset provides financing
    HEDGE,               // Asset hedges another position
    ARBITRAGE,           // Arbitrage relationship
    CROSS_CURRENCY,      // Cross-currency relationship
    REGULATORY           // Regulatory relationship (haircuts, offsets)
};

struct AssetRelationship {
    std::string source_asset_id;      // e.g., "T-BILL-3M-USD"
    std::string target_asset_id;      // e.g., "SPX-OPTIONS"
    RelationshipType type;

    // Relationship parameters
    double collateral_value_ratio;   // 0.0-1.0 (haircut applied)
    double margin_credit_ratio;       // Margin reduction percentage
    Currency base_currency;
    Currency target_currency;

    // Constraints
    double min_amount;
    double max_amount;
    int min_days_to_maturity;
    int max_days_to_maturity;

    // Broker-specific
    std::vector<std::string> applicable_brokers;  // IBKR, Alpaca, etc.
    std::string regulatory_regime;                // "REG-T", "PORTFOLIO", "SPAN"

    // Validity
    std::chrono::system_clock::time_point valid_from;
    std::chrono::system_clock::time_point valid_until;
    bool is_active;
};

class AssetRelationshipGraph {
public:
    // Add relationship
    void add_relationship(const AssetRelationship& rel);

    // Query relationships
    std::vector<AssetRelationship> get_collateral_for(
        const std::string& target_asset_id,
        Currency currency
    ) const;

    std::vector<AssetRelationship> get_financing_options(
        Currency currency,
        double amount,
        int days_needed
    ) const;

    // Find optimal collateral chain
    std::vector<AssetRelationship> find_collateral_chain(
        const std::string& target_asset_id,
        double required_margin,
        Currency currency
    ) const;

    // Cross-currency relationships
    std::vector<AssetRelationship> get_cross_currency_paths(
        Currency from,
        Currency to
    ) const;
};
```

### 2. Collateral Valuation System

```cpp
// native/include/collateral_valuator.h

struct CollateralPosition {
    std::string asset_id;
    double quantity;
    double market_value;
    Currency currency;
    int days_to_maturity;  // For bonds/T-bills
    double credit_rating;  // For bonds
};

struct CollateralValuation {
    double gross_value;              // Market value
    double haircut_percent;          // Applied haircut
    double net_collateral_value;     // After haircut
    double margin_credit;            // Credit toward margin
    Currency currency;
    std::string valuation_method;   // "MARKET", "STRESS", "REGULATORY"
};

class CollateralValuator {
public:
    // Calculate collateral value for margin
    CollateralValuation value_for_margin(
        const CollateralPosition& position,
        const std::string& margin_requirement_type,  // "OPTIONS", "FUTURES", etc.
        const std::string& broker
    ) const;

    // Portfolio-level collateral valuation
    CollateralValuation value_portfolio(
        const std::vector<CollateralPosition>& positions,
        const std::string& margin_requirement_type
    ) const;

    // Cross-currency collateral
    CollateralValuation value_cross_currency(
        const CollateralPosition& position,
        Currency target_currency,
        double fx_rate,
        double fx_haircut = 0.02  // 2% FX haircut
    ) const;

    // Get haircut schedule
    double get_haircut(
        const std::string& asset_type,  // "T-BILL", "BOND", "STOCK", "ETF"
        int days_to_maturity,
        double credit_rating,
        const std::string& broker
    ) const;
};
```

### 3. Financing Instrument Abstraction

```cpp
// native/include/financing_instrument.h

enum class InstrumentType {
    BOX_SPREAD,          // Options-based synthetic financing
    FUTURES,             // Futures-based financing
    TREASURY_BILL,       // T-bill
    TREASURY_BOND,       // T-bond
    CORPORATE_BOND,      // Corporate bond
    BANK_LOAN,           // Bank financing
    PENSION_LOAN,        // Pension fund loan
    REPO,                // Repurchase agreement
    FX_SWAP,             // FX swap
    MONEY_MARKET_FUND    // Money market fund
};

struct FinancingInstrument {
    InstrumentType type;
    std::string asset_id;
    std::string symbol;              // Market symbol
    Currency base_currency;

    // Financing terms
    double annual_rate;              // Annualized financing cost/return (%)
    double effective_rate;            // After fees, taxes, etc.
    double all_in_cost;              // Total cost including all fees

    // Terms
    int days_to_maturity;
    std::chrono::system_clock::time_point maturity_date;
    double min_size;
    double max_size;

    // Collateral requirements
    double required_collateral_ratio;  // 0.0-1.0
    std::vector<std::string> acceptable_collateral_types;

    // Liquidity and availability
    double liquidity_score;           // 0-100
    bool is_available;
    std::vector<std::string> available_brokers;

    // Relationship metadata
    std::vector<std::string> can_collateralize;  // What this can be collateral for
    std::vector<std::string> can_finance;        // What this can finance
};

class FinancingInstrumentRegistry {
public:
    // Register instrument
    void register_instrument(const FinancingInstrument& instrument);

    // Query instruments
    std::vector<FinancingInstrument> find_financing_options(
        Currency currency,
        double amount,
        int days_needed,
        double min_liquidity = 50.0
    ) const;

    // Get best financing option
    FinancingInstrument get_best_financing(
        Currency currency,
        double amount,
        int days_needed,
        const std::string& optimization_criteria = "COST"  // "COST", "LIQUIDITY", "FLEXIBILITY"
    ) const;

    // Get instruments that can serve as collateral
    std::vector<FinancingInstrument> get_collateral_instruments(
        const std::string& target_asset_id,
        Currency currency
    ) const;
};
```

### 4. Multi-Asset Financing Optimizer

```cpp
// native/include/financing_optimizer.h

struct FinancingRequirements {
    Currency target_currency;
    double amount;
    int days_needed;
    double risk_tolerance;           // 0.0-1.0
    std::vector<std::string> preferred_brokers;
    std::vector<InstrumentType> preferred_instruments;
    bool allow_cross_currency;
    double max_fx_exposure;          // Max FX risk
};

struct FinancingSolution {
    std::vector<FinancingInstrument> instruments;
    double total_cost;
    double effective_rate;
    std::vector<CollateralPosition> required_collateral;
    double total_collateral_value;
    std::map<Currency, double> currency_exposure;
    std::string optimization_method;
    double confidence_score;         // 0.0-1.0
};

class FinancingOptimizer {
public:
    FinancingOptimizer(
        const AssetRelationshipGraph& graph,
        const FinancingInstrumentRegistry& registry,
        const CollateralValuator& valuator
    );

    // Find optimal financing solution
    FinancingSolution optimize_financing(
        const FinancingRequirements& requirements
    ) const;

    // Multi-instrument financing (split across instruments)
    FinancingSolution optimize_multi_instrument(
        const FinancingRequirements& requirements,
        int max_instruments = 3
    ) const;

    // Cross-currency financing with hedging
    FinancingSolution optimize_cross_currency(
        Currency source_currency,
        Currency target_currency,
        double amount,
        int days_needed,
        bool include_hedging = true
    ) const;

    // Collateral optimization
    FinancingSolution optimize_with_collateral(
        const FinancingRequirements& requirements,
        const std::vector<CollateralPosition>& available_collateral
    ) const;

    // Portfolio-level optimization
    FinancingSolution optimize_portfolio_financing(
        const std::vector<FinancingRequirements>& requirements,
        const std::vector<CollateralPosition>& portfolio
    ) const;
};
```

---

## Ledger Integration

### Extended Ledger Schema

```rust
// agents/backend/crates/ledger/src/integration.rs

// Record financing transaction
pub async fn record_financing(
    ledger: Arc<LedgerEngine>,
    instrument_type: &str,  // "BOX_SPREAD", "T_BILL", "BANK_LOAN", etc.
    asset_id: &str,
    amount: Decimal,
    currency: Currency,
    rate: Decimal,
    days_to_maturity: i32,
    collateral_used: Option<Vec<CollateralPosting>>,
) -> Result<Uuid>;

// Record collateral pledge
pub async fn record_collateral_pledge(
    ledger: Arc<LedgerEngine>,
    collateral_asset_id: &str,
    collateral_amount: Decimal,
    collateral_currency: Currency,
    pledged_for: &str,  // Asset ID this collateral supports
    margin_credit: Decimal,
) -> Result<Uuid>;

// Record cross-currency financing
pub async fn record_cross_currency_financing(
    ledger: Arc<LedgerEngine>,
    source_currency: Currency,
    target_currency: Currency,
    source_amount: Decimal,
    target_amount: Decimal,
    fx_rate: Decimal,
    financing_instrument: &str,
    hedge_instrument: Option<&str>,
) -> Result<Uuid>;

// Record portfolio margin benefit
pub async fn record_portfolio_margin_benefit(
    ledger: Arc<LedgerEngine>,
    positions: Vec<&str>,  // Asset IDs
    margin_reduction: Decimal,
    currency: Currency,
) -> Result<Uuid>;
```

### Account Structure

```
Assets:
  Cash:
    USD:Bank:IBKR
    ILS:Bank:Discount
    EUR:Bank:...

  Investments:
    Options:SPX:BoxSpread:...
    Futures:ES:...
    Bonds:US-Treasury:3M:...
    T-Bills:US-Treasury:1M:...
    ETFs:SPY:...
    Pension:Fund:...

  Collateral:
    Pledged:SPY:For:Options:SPX
    Pledged:T-Bill:For:Futures:ES

Liabilities:
  Financing:
    BoxSpread:SPX:...
    BankLoan:Discount:...
    PensionLoan:...

  Margin:
    Options:SPX:...
    Futures:ES:...

Equity:
  NetWorth
```

---

## Example Use Cases

### Use Case 1: Options Margin with T-Bill Collateral

```cpp
// Scenario: Need margin for SPX box spread, have T-bills

FinancingRequirements req;
req.target_currency = Currency::USD;
req.amount = 100000.0;  // $100k margin needed
req.days_needed = 30;

// Available collateral
std::vector<CollateralPosition> collateral = {
    {"T-BILL-3M-USD", 150000.0, 150000.0, Currency::USD, 60, 0.0}
};

// Optimize
FinancingSolution solution = optimizer.optimize_with_collateral(req, collateral);

// Result:
// - Uses T-bills as collateral (95% haircut = $142.5k collateral value)
// - Sufficient for $100k margin requirement
// - No additional financing needed
// - Records: Collateral pledge in ledger
```

### Use Case 2: Cross-Currency Financing

```cpp
// Scenario: Need USD financing, have ILS pension fund

FinancingRequirements req;
req.target_currency = Currency::USD;
req.amount = 50000.0;
req.days_needed = 90;
req.allow_cross_currency = true;

// Available: ILS pension fund
std::vector<CollateralPosition> collateral = {
    {"PENSION-FUND-ILS", 200000.0, 200000.0, Currency::ILS, 0, 0.0}
};

// Optimize with cross-currency
FinancingSolution solution = optimizer.optimize_cross_currency(
    Currency::ILS,
    Currency::USD,
    50000.0,
    90,
    true  // Include FX hedging
);

// Result:
// - Pledge ILS pension fund as collateral
// - Convert to USD equivalent (with FX haircut)
// - Obtain USD financing (bank loan or box spread)
// - Hedge FX exposure with FX swap
// - Records: Cross-currency financing + hedge in ledger
```

### Use Case 3: Multi-Instrument Financing

```cpp
// Scenario: Large financing need, split across instruments

FinancingRequirements req;
req.target_currency = Currency::USD;
req.amount = 1000000.0;  // $1M
req.days_needed = 60;

// Optimize across multiple instruments
FinancingSolution solution = optimizer.optimize_multi_instrument(
    req,
    5  // Max 5 instruments
);

// Result:
// - $400k via box spreads (best rate, 4.5%)
// - $300k via T-bills (4.2%, but need to sell existing)
// - $200k via bank loan (4.8%, but flexible)
// - $100k via futures (4.0%, but requires margin)
// - Total weighted cost: 4.4%
// - Records: All financing instruments in ledger
```

### Use Case 4: Portfolio Margin Optimization

```cpp
// Scenario: Multiple positions, optimize margin across portfolio

std::vector<Position> portfolio = {
    {"SPX-BOX-SPREAD-1", ...},
    {"SPX-BOX-SPREAD-2", ...},
    {"ES-FUTURES", ...},
    {"T-BILL-3M", ...}
};

// Calculate portfolio margin benefit
double benefit = margin_calc.calculate_portfolio_margin_benefit(
    portfolio,
    underlying_price
);

// Record benefit in ledger
ledger.record_portfolio_margin_benefit(portfolio_ids, benefit, Currency::USD);

// Result:
// - Individual margin: $500k
// - Portfolio margin: $350k
// - Benefit: $150k capital freed
// - Can use freed capital for additional positions
```

---

## Implementation Phases

### Phase 1: Foundation (Current)

- ✅ Ledger system with multi-currency support
- ✅ Margin calculator for options
- ✅ Multi-broker infrastructure
- ✅ Box spread financing

### Phase 2: Asset Relationships

- [ ] Asset relationship graph
- [ ] Collateral valuation system
- [ ] Relationship database/persistence
- [ ] Broker-specific relationship rules

### Phase 3: Instrument Expansion

- [ ] Futures financing
- [ ] T-bill/T-bond integration
- [ ] Bank loan integration
- [ ] Pension loan integration
- [ ] Repo agreements

### Phase 4: Optimization Engine

- [ ] Financing optimizer
- [ ] Multi-instrument optimization
- [ ] Cross-currency optimization
- [ ] Portfolio-level optimization

### Phase 5: Advanced Features

- [ ] Real-time relationship updates
- [ ] Regulatory compliance engine
- [ ] Stress testing
- [ ] Scenario analysis

---

## Data Sources

### Relationship Data

- **Broker APIs**: IBKR, Alpaca margin requirements
- **Regulatory**: SEC, FINRA margin rules
- **Market Data**: Collateral haircuts, FX rates
- **Custom Rules**: User-defined relationships

### Instrument Data

- **Options**: Box spread rates (existing)
- **Futures**: Implied financing rates
- **Treasury**: TreasuryDirect, broker APIs
- **Bank Loans**: Bank APIs, Discount Bank parser
- **Pension Loans**: Pension fund APIs

---

## Integration Points

### With Existing Systems

1. **Ledger System**
   - Extend transaction types
   - Add collateral tracking
   - Cross-currency support

2. **Margin Calculator**
   - Extend to all asset types
   - Portfolio margin integration
   - Collateral valuation

3. **Multi-Broker System**
   - Broker-specific relationship rules
   - Best execution across instruments
   - Unified API for all financing

4. **Risk-Free Rate Service**
   - Compare all financing rates
   - Build unified yield curve
   - Arbitrage detection

---

## Next Steps

1. **Design Asset Relationship Database Schema**
   - SQLite/PostgreSQL for relationships
   - Versioning for regulatory changes
   - Broker-specific overrides

2. **Implement Collateral Valuator**
   - Haircut schedules
   - Cross-currency conversion
   - Portfolio-level valuation

3. **Build Financing Instrument Registry**
   - Instrument discovery
   - Rate aggregation
   - Availability tracking

4. **Create Financing Optimizer**
   - Constraint solver (NLopt/Eigen)
   - Multi-objective optimization
   - Real-time updates

This architecture transforms the system from "box spread generator" to "comprehensive synthetic financing platform" while leveraging all existing infrastructure.
