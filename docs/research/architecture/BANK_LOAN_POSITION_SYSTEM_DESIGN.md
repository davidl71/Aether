# Bank Loan Position System Design

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document
**Related Tasks:** T-74, T-76, T-77

## Overview

This document designs a comprehensive system for entering, storing, and managing bank loan positions (SHIR-based variable rate loans and CPI-linked fixed rate loans) in the investment strategy framework. Loans are factored into net portfolio value calculations and affect portfolio allocation decisions.

## Requirements

### User Specifications

1. **Loan Entry Methods:**
   - Manual entry via TUI form
   - CSV/JSON file import for bulk entry

2. **Storage:**
   - Separate `loans.json` file (current implementation)
   - Code stubs/TODO comments for future database migration

3. **Loan Types:**
   - **SHIR-based Variable Rate Loans:** Interest rate = SHIR + spread
   - **CPI-linked Fixed Rate Loans:** Principal adjusts with CPI

4. **Integration:**
   - Net Portfolio Value = IBKR Assets + Israeli Broker Assets - Loan Liabilities
   - Currency: Israeli Shekel (ILS) with USD conversion for unified view

## Data Model

### LoanPosition Structure

```cpp
// native/include/ib_box_spread/loan_position.h

namespace ib_box_spread {

enum class LoanType {
    SHIR_BASED,      // Variable rate: SHIR + spread
    CPI_LINKED       // Fixed rate, principal adjusts with CPI
};

enum class LoanStatus {
    ACTIVE,
    PAID_OFF,
    DEFAULTED
};

struct LoanPosition {
    // Identification
    std::string loan_id;              // Unique identifier
    std::string bank_name;             // Bank name (e.g., "Fibi", "Discount")
    std::string account_number;       // Bank account number

    // Loan Terms
    LoanType loan_type;                // SHIR_BASED or CPI_LINKED
    double principal;                  // Current principal (ILS)
    double original_principal;         // Original loan amount (ILS)
    double interest_rate;              // Base interest rate (%)
    double spread;                     // Spread above SHIR (for SHIR-based loans)

    // CPI-Linked Loan Fields
    double base_cpi;                   // CPI at loan origination
    double current_cpi;                 // Current CPI (updated monthly)

    // Dates
    std::chrono::system_clock::time_point origination_date;
    std::chrono::system_clock::time_point maturity_date;
    std::chrono::system_clock::time_point next_payment_date;

    // Payment Schedule
    double monthly_payment;            // Monthly payment amount (ILS)
    int payment_frequency_months;     // Payment frequency (typically 1)

    // Status
    LoanStatus status;
    std::chrono::system_clock::time_point last_update;

    // Helper Methods
    double get_adjusted_principal() const;
    double get_current_interest_rate() const;  // For SHIR-based loans
    double get_usd_value(double ils_usd_rate) const;
    bool is_overdue() const;
    int days_until_next_payment() const;
};

} // namespace ib_box_spread
```

### Loan Calculation Logic

```cpp
// native/src/loan_position.cpp

double LoanPosition::get_adjusted_principal() const {
    if (loan_type == LoanType::CPI_LINKED && base_cpi > 0 && current_cpi > 0) {
        // Principal adjusts with CPI: new_principal = original * (current_cpi / base_cpi)
        return original_principal * (current_cpi / base_cpi);
    }
    return principal;
}

double LoanPosition::get_current_interest_rate() const {
    if (loan_type == LoanType::SHIR_BASED) {
        // TODO: Fetch current SHIR from data source
        // For now, return base rate + spread
        // Future: return current_shir + spread
        return interest_rate + spread;
    }
    return interest_rate;  // Fixed rate for CPI-linked loans
}

double LoanPosition::get_usd_value(double ils_usd_rate) const {
    double adjusted_principal = get_adjusted_principal();
    return adjusted_principal * ils_usd_rate;
}
```

## Storage System

### JSON Storage Format

```json
// loans.json

{
  "version": "1.0",
  "last_updated": "2025-11-18T12:00:00Z",
  "loans": [
    {
      "loan_id": "FIBI-001",
      "bank_name": "Fibi",
      "account_number": "123456789",
      "loan_type": "SHIR_BASED",
      "principal": 500000.0,
      "original_principal": 500000.0,
      "interest_rate": 3.5,
      "spread": 1.2,
      "base_cpi": 0.0,
      "current_cpi": 0.0,
      "origination_date": "2020-01-15T00:00:00Z",
      "maturity_date": "2030-01-15T00:00:00Z",
      "next_payment_date": "2025-12-01T00:00:00Z",
      "monthly_payment": 4500.0,
      "payment_frequency_months": 1,
      "status": "ACTIVE",
      "last_update": "2025-11-18T12:00:00Z"
    },
    {
      "loan_id": "DISCOUNT-001",
      "bank_name": "Discount",
      "account_number": "987654321",
      "loan_type": "CPI_LINKED",
      "principal": 750000.0,
      "original_principal": 700000.0,
      "interest_rate": 2.8,
      "spread": 0.0,
      "base_cpi": 105.2,
      "current_cpi": 112.5,
      "origination_date": "2019-06-01T00:00:00Z",
      "maturity_date": "2029-06-01T00:00:00Z",
      "next_payment_date": "2025-12-01T00:00:00Z",
      "monthly_payment": 5200.0,
      "payment_frequency_months": 1,
      "status": "ACTIVE",
      "last_update": "2025-11-18T12:00:00Z"
    }
  ]
}
```

### Database Migration Path (Future)

```cpp
// native/include/ib_box_spread/loan_storage.h

class LoanStorage {
public:
    // Current implementation: JSON file
    bool load_from_json(const std::string& file_path);
    bool save_to_json(const std::string& file_path);

    // TODO: Future database implementation
    // bool load_from_database(const std::string& connection_string);
    // bool save_to_database(const std::string& connection_string);
    //
    // Database Schema (PostgreSQL example):
    // CREATE TABLE loans (
    //     loan_id VARCHAR(50) PRIMARY KEY,
    //     bank_name VARCHAR(100),
    //     account_number VARCHAR(50),
    //     loan_type VARCHAR(20),
    //     principal DECIMAL(15,2),
    //     original_principal DECIMAL(15,2),
    //     interest_rate DECIMAL(5,2),
    //     spread DECIMAL(5,2),
    //     base_cpi DECIMAL(10,4),
    //     current_cpi DECIMAL(10,4),
    //     origination_date TIMESTAMP,
    //     maturity_date TIMESTAMP,
    //     next_payment_date TIMESTAMP,
    //     monthly_payment DECIMAL(10,2),
    //     payment_frequency_months INTEGER,
    //     status VARCHAR(20),
    //     last_update TIMESTAMP
    // );
};
```

## Loan Manager Interface

```cpp
// native/include/ib_box_spread/loan_manager.h

class LoanManager {
public:
    // Initialization
    bool initialize(const std::string& loans_file_path);

    // CRUD Operations
    bool add_loan(const LoanPosition& loan);
    bool update_loan(const std::string& loan_id, const LoanPosition& loan);
    bool delete_loan(const std::string& loan_id);
    std::optional<LoanPosition> get_loan(const std::string& loan_id) const;
    std::vector<LoanPosition> get_all_loans() const;
    std::vector<LoanPosition> get_active_loans() const;

    // Calculations
    double get_total_loan_liabilities_ils() const;
    double get_total_loan_liabilities_usd(double ils_usd_rate) const;
    double get_monthly_payment_total_ils() const;

    // Updates
    bool update_cpi_for_all_loans(double current_cpi);
    bool update_shir_for_all_loans(double current_shir);
    void refresh_loan_calculations();

    // Persistence
    bool save();
    bool load();

private:
    std::string loans_file_path_;
    std::unordered_map<std::string, LoanPosition> loans_;
    mutable std::shared_mutex loans_mutex_;
};
```

## Entry Interface

### Manual TUI Entry

```cpp
// native/src/loan_entry_ui.cpp

class LoanEntryUI {
public:
    LoanEntryUI(LoanManager& loan_manager);

    // TUI form for manual entry
    bool show_entry_form();

    // Form fields:
    // - Bank name (dropdown: Fibi, Discount, Other)
    // - Account number (text input)
    // - Loan type (dropdown: SHIR-based, CPI-linked)
    // - Principal (numeric input)
    // - Interest rate (numeric input)
    // - Spread (numeric input, for SHIR-based)
    // - Base CPI (numeric input, for CPI-linked)
    // - Origination date (date picker)
    // - Maturity date (date picker)
    // - Monthly payment (numeric input)
    // - Payment frequency (dropdown: Monthly, Quarterly, etc.)

private:
    LoanManager& loan_manager_;
    // FTXUI components for form
};
```

### File Import

```cpp
// native/src/loan_importer.cpp

class LoanImporter {
public:
    // CSV Import
    std::vector<LoanPosition> import_from_csv(const std::string& file_path);

    // JSON Import
    std::vector<LoanPosition> import_from_json(const std::string& file_path);

    // Validation
    bool validate_loan(const LoanPosition& loan, std::vector<std::string>& errors);

private:
    // CSV format:
    // loan_id,bank_name,account_number,loan_type,principal,original_principal,
    // interest_rate,spread,base_cpi,current_cpi,origination_date,maturity_date,
    // next_payment_date,monthly_payment,payment_frequency_months,status
};
```

## Integration with Portfolio Calculator

```cpp
// native/src/portfolio_calculator.cpp (updates)

class PortfolioCalculator {
public:
    // Existing methods...

    // New: Include loan liabilities in net portfolio value
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
};
```

## Configuration

```json
// config/config.json (additions)

{
  "loans": {
    "enabled": true,
    "loans_file": "config/loans.json",
    "auto_update_cpi": true,
    "auto_update_shir": true,
    "cpi_update_frequency_days": 30,
    "shir_update_frequency_days": 7,
    "default_currency": "ILS",
    "ils_usd_rate": 0.27,
    "ils_usd_rate_source": "manual"  // or "api" for future exchange rate API
  }
}
```

## Testing Requirements

### Unit Tests

```cpp
// native/tests/loan_position_test.cpp

TEST_CASE("LoanPosition - CPI Adjustment") {
    LoanPosition loan;
    loan.loan_type = LoanType::CPI_LINKED;
    loan.original_principal = 100000.0;
    loan.base_cpi = 100.0;
    loan.current_cpi = 105.0;

    REQUIRE(loan.get_adjusted_principal() == 105000.0);
}

TEST_CASE("LoanPosition - SHIR Rate Calculation") {
    LoanPosition loan;
    loan.loan_type = LoanType::SHIR_BASED;
    loan.interest_rate = 3.5;
    loan.spread = 1.2;

    // TODO: Mock SHIR data source
    REQUIRE(loan.get_current_interest_rate() == 4.7);  // 3.5 + 1.2
}
```

## Implementation Phases

### Phase 1: Core Data Model (T-76)

- Implement `LoanPosition` structure
- Implement `LoanManager` with JSON storage
- Basic CRUD operations
- Integration with portfolio calculator

### Phase 2: Entry Interface (T-77)

- TUI manual entry form
- CSV/JSON file import
- Validation and error handling
- Display loan positions in portfolio view

### Phase 3: Advanced Features (Future)

- Database migration
- Automated CPI/SHIR updates
- Payment scheduling and reminders
- Loan analytics and reporting

## References

- [Investment Strategy Framework](../docs/INVESTMENT_STRATEGY_FRAMEWORK.md)
- [Trading Economics API Research](../docs/RESEARCH_TRADING_ECONOMICS_API.md)
- [Israeli Broker Position Import](../docs/ISRAELI_BROKER_POSITION_IMPORT.md)
