// loan_position.h - Bank loan position data structures
#pragma once

#include <string>
#include <chrono>
#include <optional>

namespace ib_box_spread {

// ============================================================================
// Loan Type and Status Enumerations
// ============================================================================

enum class LoanType {
    SHIR_BASED,      // Variable rate: SHIR + spread
    CPI_LINKED       // Fixed rate, principal adjusts with CPI
};

enum class LoanStatus {
    ACTIVE,
    PAID_OFF,
    DEFAULTED
};

// ============================================================================
// LoanPosition Structure
// ============================================================================

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
    double get_current_interest_rate(double current_shir = 0.0) const;  // For SHIR-based loans
    double get_usd_value(double ils_usd_rate) const;
    bool is_overdue() const;
    int days_until_next_payment() const;
    bool is_valid() const;  // Validate loan data
};

// ============================================================================
// Helper Functions
// ============================================================================

// Convert LoanType to string
std::string loan_type_to_string(LoanType type);

// Convert string to LoanType
LoanType string_to_loan_type(const std::string& str);

// Convert LoanStatus to string
std::string loan_status_to_string(LoanStatus status);

// Convert string to LoanStatus
LoanStatus string_to_loan_status(const std::string& str);

// Convert time_point to ISO 8601 string
std::string time_point_to_iso8601(const std::chrono::system_clock::time_point& tp);

// Convert ISO 8601 string to time_point
std::chrono::system_clock::time_point iso8601_to_time_point(const std::string& iso_str);

} // namespace ib_box_spread
