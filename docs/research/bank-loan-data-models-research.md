# Bank Loan Position Data Models Research

**Date**: 2025-12-30
**Task**: T-152
**Status**: Research Complete
**Related Tasks**: T-74, T-76, T-77

## Executive Summary

This research document analyzes data model patterns, storage strategies, and integration approaches for bank loan positions in the investment strategy framework. The research covers SHIR-based variable rate loans and CPI-linked fixed rate loans, with recommendations for implementation.

**Key Findings**:

1. **Data Model**: Well-defined structure exists in design document
2. **Storage**: JSON-first approach with database migration path recommended
3. **Integration**: Portfolio calculator integration pattern established
4. **Calculations**: CPI adjustment and SHIR rate calculation patterns identified

---

## 1. Local Codebase Analysis

### Existing Patterns

#### 1.1 Design Document Structure

**Location**: `docs/research/architecture/BANK_LOAN_POSITION_SYSTEM_DESIGN.md`

**Key Components Identified**:

- `LoanPosition` struct with comprehensive fields
- `LoanType` enum (SHIR_BASED, CPI_LINKED)
- `LoanStatus` enum (ACTIVE, PAID_OFF, DEFAULTED)
- Helper methods for calculations

**Code Snippet**:

```cpp
struct LoanPosition {
    std::string loan_id;
    std::string bank_name;
    std::string account_number;
    LoanType loan_type;
    double principal;
    double original_principal;
    double interest_rate;
    double spread;
    double base_cpi;
    double current_cpi;
    // ... dates, payments, status
};
```

#### 1.2 Storage Architecture Patterns

**Location**: `docs/research/architecture/BACKEND_DATA_STORAGE_ARCHITECTURE.md`

**Multi-Layer Storage Strategy**:

1. **Layer 1**: In-Memory Cache (fast access)
2. **Layer 2**: Operational Database (PostgreSQL/SQLite)
3. **Layer 3**: Time-Series Database (QuestDB)

**Loans Table Schema** (from design):

```sql
CREATE TABLE loans (
    id VARCHAR(255) PRIMARY KEY,
    loan_type VARCHAR(50) NOT NULL,  -- "SHIR" or "CPI_LINKED"
    currency VARCHAR(10) NOT NULL,  -- "ILS"
    principal_remaining DECIMAL(15, 4) NOT NULL,
    monthly_payment DECIMAL(15, 4) NOT NULL,
    spread DECIMAL(8, 4),  -- Spread over SHIR
    shir_rate DECIMAL(8, 4),  -- Current SHIR rate
    fixed_rate DECIMAL(8, 4),  -- Fixed interest rate
    cpi_index DECIMAL(10, 4),  -- Current CPI index
    remaining_months INTEGER NOT NULL,
    next_payment_date TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### 1.3 Python DSL Patterns

**Location**: `python/dsl/cash_flow_dsl.py`, `python/dsl/financing_strategy_dsl.py`

**Existing Loan Models**:

```python
# Simple loan model in cash flow DSL
def bank_loan(amount: float, rate: float, payments: str = "monthly") -> Position:
    return Position(
        type="bank_loan",
        amount=Decimal(str(amount)),
        rate=Decimal(str(rate)),
        maturity="",
        payments=payments
    )

# More detailed model in financing strategy DSL
def bank_loan(rate: float, amount: float, currency: str = "USD") -> FinancingSource:
    return FinancingSource(type="bank_loan", rate=rate, amount=amount, currency=currency)
```

**Key Insight**: Python models are simpler (amount + rate), while C++ design is more comprehensive (includes CPI, SHIR, dates, etc.)

#### 1.4 Cash Flow Integration

**Location**: `docs/research/architecture/CASH_FLOW_FORECASTING_SYSTEM.md`

**Loan Data Model in Cash Flow System**:

```python
@dataclass
class Loan:
    id: str
    loan_type: str  # "SHIR" or "CPI_LINKED"
    currency: str  # ILS
    principal_remaining: float
    monthly_payment: float
    fixed_rate: Optional[float] = None  # For CPI-linked loans
    spread: Optional[float] = None  # For SHIR-based loans
    remaining_months: int
    next_payment_date: datetime
    shir_rate: Optional[float] = None
    cpi_index: Optional[float] = None
```

**Integration Pattern**: Loans generate cash flow events for payment scheduling

#### 1.5 Portfolio Calculator Integration

**Location**: `docs/research/architecture/BANK_LOAN_POSITION_SYSTEM_DESIGN.md`, `docs/platform/INVESTMENT_STRATEGY_FRAMEWORK.md`

**Net Portfolio Value Calculation**:

```cpp
double calculate_net_portfolio_value(
    const std::vector<Position>& ibkr_positions,
    const std::vector<Position>& israeli_broker_positions,
    const LoanManager& loan_manager,
    double ils_usd_rate
) {
    double total_assets = calculate_total_assets(ibkr_positions, israeli_broker_positions);
    double total_loan_liabilities_usd = loan_manager.get_total_loan_liabilities_usd(ils_usd_rate);
    return total_assets - total_loan_liabilities_usd;
}
```

**Formula**: `Net Portfolio Value = Assets - Loan Liabilities (converted to USD)`

---

## 2. Data Model Patterns Analysis

### 2.1 Loan Type Differentiation

#### SHIR-Based Loans (Variable Rate)

**Characteristics**:

- Interest rate = SHIR + spread
- Rate changes with SHIR fluctuations
- Monthly payments adjust with rate changes
- Requires SHIR data source integration

**Data Model Requirements**:

- `spread` field (fixed)
- `shir_rate` field (variable, updated periodically)
- `get_current_interest_rate()` = `shir_rate + spread`

**Calculation Pattern**:

```cpp
double get_current_interest_rate() const {
    if (loan_type == LoanType::SHIR_BASED) {
        return current_shir_rate + spread;  // Requires SHIR data source
    }
    return interest_rate;  // Fixed rate
}
```

#### CPI-Linked Loans (Fixed Rate, Adjusting Principal)

**Characteristics**:

- Fixed interest rate
- Principal adjusts with CPI changes
- Monthly payment may adjust with principal
- Requires CPI data source integration

**Data Model Requirements**:

- `base_cpi` field (CPI at origination)
- `current_cpi` field (updated monthly)
- `original_principal` field (for calculation)
- `get_adjusted_principal()` = `original_principal * (current_cpi / base_cpi)`

**Calculation Pattern**:

```cpp
double get_adjusted_principal() const {
    if (loan_type == LoanType::CPI_LINKED && base_cpi > 0 && current_cpi > 0) {
        return original_principal * (current_cpi / base_cpi);
    }
    return principal;
}
```

### 2.2 Principal Adjustment Patterns

**CPI Adjustment Formula**:

```
adjusted_principal = original_principal × (current_cpi / base_cpi)
```

**Example**:

- Original principal: 700,000 ILS
- Base CPI: 105.2
- Current CPI: 112.5
- Adjusted principal: 700,000 × (112.5 / 105.2) = 748,574 ILS

**Key Considerations**:

- CPI updates monthly (typically 15th of month)
- Principal adjustment affects loan value
- Monthly payment may need recalculation

### 2.3 Interest Rate Calculation Patterns

**SHIR-Based Rate**:

```
current_rate = current_shir_rate + spread
```

**CPI-Linked Rate**:

```
current_rate = fixed_rate  // Constant
```

**Key Considerations**:

- SHIR rate changes frequently (weekly/daily)
- Need to track rate history for payment calculations
- Rate changes affect monthly payment amount

---

## 3. Storage Pattern Comparison

### 3.1 JSON Storage (Recommended for Phase 1)

**Advantages**:

- ✅ Simple implementation
- ✅ Human-readable
- ✅ Easy to edit manually
- ✅ Version control friendly
- ✅ No database setup required
- ✅ Fast for small datasets (< 100 loans)

**Disadvantages**:

- ❌ No concurrent write support
- ❌ Limited query capabilities
- ❌ No transaction support
- ❌ Performance degrades with large datasets
- ❌ No built-in validation

**Implementation Pattern** (from design):

```json
{
  "version": "1.0",
  "last_updated": "2025-11-18T12:00:00Z",
  "loans": [
    {
      "loan_id": "FIBI-001",
      "bank_name": "Fibi",
      "loan_type": "SHIR_BASED",
      "principal": 500000.0,
      "spread": 1.2,
      // ... other fields
    }
  ]
}
```

**File Location**: `config/loans.json`

### 3.2 Database Storage (Recommended for Phase 2+)

**Advantages**:

- ✅ ACID guarantees
- ✅ Concurrent access support
- ✅ Query capabilities
- ✅ Transaction support
- ✅ Scalable to large datasets
- ✅ Built-in validation
- ✅ Historical tracking

**Disadvantages**:

- ❌ Requires database setup
- ❌ More complex implementation
- ❌ Migration needed from JSON

**Database Options**:

#### SQLite (Development)

- Zero configuration
- File-based
- Good for single-instance
- Easy backups

#### PostgreSQL (Production)

- Robust ACID guarantees
- Excellent JSON support (JSONB)
- Concurrent access
- Time-series extensions (TimescaleDB)

**Migration Strategy**:

1. Start with JSON (Phase 1)
2. Design database schema (Phase 2)
3. Implement dual-write (JSON + DB)
4. Migrate existing data
5. Switch to DB-only

### 3.3 Hybrid Approach (Recommended)

**Strategy**:

- **Phase 1**: JSON storage only
- **Phase 2**: Add database, dual-write
- **Phase 3**: Database primary, JSON backup
- **Phase 4**: Database only (optional)

**Benefits**:

- Gradual migration
- No data loss risk
- Easy rollback
- Flexible deployment

---

## 4. Integration Patterns

### 4.1 Portfolio Calculator Integration

**Pattern**: Dependency Injection

```cpp
class PortfolioCalculator {
    double calculate_net_portfolio_value(
        const LoanManager& loan_manager,  // Injected dependency
        double ils_usd_rate
    );
};
```

**Benefits**:

- Loose coupling
- Testable (mock LoanManager)
- Flexible (can swap implementations)

### 4.2 Currency Conversion Pattern

**Formula**:

```cpp
double get_usd_value(double ils_usd_rate) const {
    double adjusted_principal = get_adjusted_principal();
    return adjusted_principal * ils_usd_rate;
}
```

**Key Considerations**:

- ILS/USD rate updates needed
- Rate source: manual or API
- Update frequency: daily or on-demand

### 4.3 Cash Flow Integration Pattern

**Pattern**: Event Generation

```cpp
class LoanManager {
    std::vector<CashFlowEvent> generate_cash_flow_events(
        const std::chrono::system_clock::time_point& start_date,
        int months_ahead
    ) const;
};
```

**Cash Flow Events**:

- Monthly payment dates
- Payment amounts (negative for outflows)
- Currency (ILS)
- Loan ID reference

---

## 5. Calculation Patterns

### 5.1 CPI Adjustment Calculation

**Formula**:

```
adjusted_principal = original_principal × (current_cpi / base_cpi)
```

**Implementation Considerations**:

- Handle division by zero (base_cpi = 0)
- Handle negative CPI (shouldn't happen, but validate)
- Precision: Use `double` for financial calculations
- Rounding: Round to 2 decimal places for display

**Edge Cases**:

- Base CPI not set (use original principal)
- Current CPI not updated (use last known value)
- CPI decreases (principal decreases)

### 5.2 SHIR Rate Calculation

**Formula**:

```
current_rate = current_shir_rate + spread
monthly_payment = calculate_payment(principal, current_rate, remaining_months)
```

**Implementation Considerations**:

- SHIR rate source (API, manual entry, file)
- Rate update frequency (daily, weekly)
- Rate history tracking (for payment calculations)
- Payment recalculation when rate changes

**Edge Cases**:

- SHIR rate not available (use last known rate)
- Rate source fails (fallback to cached rate)
- Rate changes mid-month (pro-rate payment?)

### 5.3 Payment Calculation

**Amortization Formula**:

```
monthly_payment = principal × (rate/12) × (1 + rate/12)^n / ((1 + rate/12)^n - 1)
```

Where:

- `rate` = annual interest rate (as decimal)
- `n` = number of months remaining

**Implementation Considerations**:

- Handle zero-rate loans
- Handle interest-only periods
- Handle balloon payments
- Currency-specific rounding rules

---

## 6. Recommendations

### 6.1 Data Model Recommendations

**✅ Recommended Structure** (from design document):

- Use `LoanPosition` struct as defined
- Include all fields from design
- Add helper methods for calculations
- Use enums for type safety

**Rationale**:

- Comprehensive coverage
- Type-safe design
- Extensible for future loan types

### 6.2 Storage Recommendations

**Phase 1 (Immediate)**: JSON Storage

- ✅ Start with `config/loans.json`
- ✅ Simple, fast to implement
- ✅ No database dependency
- ✅ Easy to edit manually

**Phase 2 (Future)**: Database Migration

- ✅ Design database schema
- ✅ Implement dual-write
- ✅ Migrate existing data
- ✅ Switch to database primary

**Rationale**:

- JSON is sufficient for initial implementation
- Database provides scalability and features
- Migration path is clear

### 6.3 Integration Recommendations

**Portfolio Calculator**:

- ✅ Inject `LoanManager` as dependency
- ✅ Calculate loan liabilities in USD
- ✅ Subtract from total assets
- ✅ Update net portfolio value calculation

**Cash Flow System**:

- ✅ Generate payment events from loans
- ✅ Include loan payments in cash flow timeline
- ✅ Support currency conversion (ILS → USD)

**Rationale**:

- Loose coupling enables testing
- Clear integration points
- Follows existing patterns

### 6.4 Calculation Recommendations

**CPI Adjustment**:

- ✅ Use formula: `original × (current_cpi / base_cpi)`
- ✅ Validate CPI values (non-zero, positive)
- ✅ Handle edge cases (missing CPI, zero base)

**SHIR Rate**:

- ✅ Use formula: `shir_rate + spread`
- ✅ Cache SHIR rate with timestamp
- ✅ Handle rate source failures gracefully

**Payment Calculation**:

- ✅ Use standard amortization formula
- ✅ Handle edge cases (zero rate, interest-only)
- ✅ Round appropriately for currency

---

## 7. Comparison Table

| Aspect | JSON Storage | Database Storage |
|--------|--------------|------------------|
| **Setup Complexity** | Low | Medium-High |
| **Concurrent Access** | No | Yes |
| **Query Capabilities** | Limited | Full SQL |
| **Transaction Support** | No | Yes |
| **Scalability** | < 100 loans | Unlimited |
| **Performance** | Fast (small) | Fast (all sizes) |
| **Backup** | File copy | Database dump |
| **Version Control** | Yes | No (data) |
| **Human Readable** | Yes | No |
| **ACID Guarantees** | No | Yes |
| **Migration Path** | Easy | N/A |

**Recommendation**: Start with JSON, migrate to database when needed.

---

## 8. Implementation Considerations

### 8.1 Data Source Integration

**SHIR Rate Source**:

- Option 1: Manual entry (config file)
- Option 2: API integration (Bank of Israel API)
- Option 3: File import (CSV/JSON)

**CPI Data Source**:

- Option 1: Manual entry (config file)
- Option 2: API integration (Central Bureau of Statistics)
- Option 3: File import (CSV/JSON)

**Recommendation**: Start with manual entry, add API integration later.

### 8.2 Update Frequency

**SHIR Rate**:

- Update frequency: Weekly or on-demand
- Cache with timestamp
- Validate rate changes (reasonable bounds)

**CPI Index**:

- Update frequency: Monthly (typically 15th)
- Cache with timestamp
- Validate CPI changes (reasonable bounds)

### 8.3 Validation Rules

**Loan Validation**:

- Loan ID must be unique
- Principal > 0
- Interest rate >= 0
- Dates must be valid (origination < maturity)
- SHIR loans: spread >= 0
- CPI loans: base_cpi > 0

**Data Integrity**:

- Prevent duplicate loan IDs
- Validate currency (ILS for Israeli loans)
- Validate loan type matches fields (SHIR vs CPI)

---

## 9. Migration Strategy

### 9.1 JSON to Database Migration

**Phase 1: Dual-Write**

```cpp
class LoanManager {
    bool save() {
        save_to_json();  // Keep existing
        save_to_database();  // New
    }
};
```

**Phase 2: Database Primary**

```cpp
class LoanManager {
    bool save() {
        save_to_database();  // Primary
        export_to_json();  // Backup/export
    }
};
```

**Phase 3: Database Only** (Optional)

- Remove JSON storage
- Use database exclusively
- Export to JSON for backup

### 9.2 Schema Evolution

**Versioning Strategy**:

- Add `version` field to JSON
- Add migration scripts for database
- Support multiple versions during transition

**Backward Compatibility**:

- Read old format, convert to new
- Write in new format
- Maintain conversion functions

---

## 10. Testing Patterns

### 10.1 Unit Test Patterns

**LoanPosition Tests**:

- CPI adjustment calculation
- SHIR rate calculation
- USD conversion
- Date calculations (overdue, days until payment)

**LoanManager Tests**:

- CRUD operations
- JSON load/save
- Calculation methods
- Validation

### 10.2 Integration Test Patterns

**Portfolio Calculator Integration**:

- Net portfolio value with loans
- Currency conversion
- Multiple loan types

**Cash Flow Integration**:

- Payment event generation
- Timeline construction
- Currency aggregation

---

## 11. References

### Internal Documentation

- `docs/research/architecture/BANK_LOAN_POSITION_SYSTEM_DESIGN.md` - Design document
- `docs/research/architecture/BACKEND_DATA_STORAGE_ARCHITECTURE.md` - Storage architecture
- `docs/research/architecture/CASH_FLOW_FORECASTING_SYSTEM.md` - Cash flow integration
- `docs/platform/INVESTMENT_STRATEGY_FRAMEWORK.md` - Portfolio integration

### Code References

- `python/dsl/cash_flow_dsl.py` - Python loan models
- `python/dsl/financing_strategy_dsl.py` - Financing strategy DSL
- `docs/research/architecture/DATABASE_ABSTRACTION_LAYER.md` - Database patterns

---

## 12. Next Steps

### Immediate (T-76 Implementation)

1. ✅ Use `LoanPosition` struct from design document
2. ✅ Implement JSON storage first
3. ✅ Add calculation methods
4. ✅ Integrate with portfolio calculator

### Future Enhancements

1. Database migration (when needed)
2. SHIR rate API integration
3. CPI data API integration
4. Automated payment scheduling
5. Loan analytics and reporting

---

**Last Updated**: 2025-12-30
**Status**: Research Complete ✅
**Ready for**: T-76 Implementation
