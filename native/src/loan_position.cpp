// loan_position.cpp - Bank loan position implementation
#include "loan_position.h"
#include <spdlog/spdlog.h>
#include <cmath>
#include <sstream>
#include <iomanip>
#include <ctime>

namespace ib_box_spread {

// ============================================================================
// LoanPosition Helper Methods
// ============================================================================

double LoanPosition::get_adjusted_principal() const {
    if (loan_type == LoanType::CPI_LINKED && base_cpi > 0 && current_cpi > 0) {
        // Principal adjusts with CPI: new_principal = original * (current_cpi / base_cpi)
        return original_principal * (current_cpi / base_cpi);
    }
    return principal;
}

double LoanPosition::get_current_interest_rate(double current_shir) const {
    if (loan_type == LoanType::SHIR_BASED) {
        // For SHIR-based loans: rate = current_shir + spread
        // If current_shir is provided, use it; otherwise use base interest_rate + spread
        if (current_shir > 0.0) {
            return current_shir + spread;
        }
        // Fallback: return base rate + spread
        return interest_rate + spread;
    }
    return interest_rate;  // Fixed rate for CPI-linked loans
}

double LoanPosition::get_usd_value(double ils_usd_rate) const {
    double adjusted_principal = get_adjusted_principal();
    return adjusted_principal * ils_usd_rate;
}

bool LoanPosition::is_overdue() const {
    if (status != LoanStatus::ACTIVE) {
        return false;
    }

    auto now = std::chrono::system_clock::now();
    return next_payment_date < now;
}

int LoanPosition::days_until_next_payment() const {
    auto now = std::chrono::system_clock::now();
    auto duration = next_payment_date - now;
    auto days = std::chrono::duration_cast<std::chrono::hours>(duration).count() / 24;
    return static_cast<int>(days);
}

bool LoanPosition::is_valid() const {
    // Validate required fields
    if (loan_id.empty() || bank_name.empty() || account_number.empty()) {
        return false;
    }

    // Validate principal > 0
    if (principal <= 0.0 || original_principal <= 0.0) {
        return false;
    }

    // Validate interest rate >= 0
    if (interest_rate < 0.0 || spread < 0.0) {
        return false;
    }

    // Validate dates: origination < maturity
    if (origination_date >= maturity_date) {
        return false;
    }

    // Validate payment amount > 0
    if (monthly_payment <= 0.0) {
        return false;
    }

    // Validate payment frequency > 0
    if (payment_frequency_months <= 0) {
        return false;
    }

    // Validate CPI fields for CPI-linked loans
    if (loan_type == LoanType::CPI_LINKED) {
        if (base_cpi <= 0.0 || current_cpi <= 0.0) {
            return false;
        }
    }

    return true;
}

// ============================================================================
// Conversion Functions
// ============================================================================

std::string loan_type_to_string(LoanType type) {
    switch (type) {
        case LoanType::SHIR_BASED:
            return "SHIR_BASED";
        case LoanType::CPI_LINKED:
            return "CPI_LINKED";
        default:
            return "UNKNOWN";
    }
}

LoanType string_to_loan_type(const std::string& str) {
    if (str == "SHIR_BASED") {
        return LoanType::SHIR_BASED;
    } else if (str == "CPI_LINKED") {
        return LoanType::CPI_LINKED;
    }
    // Default to SHIR_BASED if unknown
    spdlog::warn("Unknown loan type: {}, defaulting to SHIR_BASED", str);
    return LoanType::SHIR_BASED;
}

std::string loan_status_to_string(LoanStatus status) {
    switch (status) {
        case LoanStatus::ACTIVE:
            return "ACTIVE";
        case LoanStatus::PAID_OFF:
            return "PAID_OFF";
        case LoanStatus::DEFAULTED:
            return "DEFAULTED";
        default:
            return "UNKNOWN";
    }
}

LoanStatus string_to_loan_status(const std::string& str) {
    if (str == "ACTIVE") {
        return LoanStatus::ACTIVE;
    } else if (str == "PAID_OFF") {
        return LoanStatus::PAID_OFF;
    } else if (str == "DEFAULTED") {
        return LoanStatus::DEFAULTED;
    }
    // Default to ACTIVE if unknown
    spdlog::warn("Unknown loan status: {}, defaulting to ACTIVE", str);
    return LoanStatus::ACTIVE;
}

std::string time_point_to_iso8601(const std::chrono::system_clock::time_point& tp) {
    auto time_t = std::chrono::system_clock::to_time_t(tp);
    auto tm = *std::gmtime(&time_t);

    std::ostringstream oss;
    oss << std::put_time(&tm, "%Y-%m-%dT%H:%M:%SZ");
    return oss.str();
}

std::chrono::system_clock::time_point iso8601_to_time_point(const std::string& iso_str) {
    // Parse ISO 8601 format: "2025-11-18T12:00:00Z"
    std::tm tm = {};
    std::istringstream iss(iso_str);
    iss >> std::get_time(&tm, "%Y-%m-%dT%H:%M:%SZ");

    if (iss.fail()) {
        spdlog::error("Failed to parse ISO 8601 date: {}", iso_str);
        return std::chrono::system_clock::now();
    }

    auto time_t = std::mktime(&tm);
    return std::chrono::system_clock::from_time_t(time_t);
}

} // namespace ib_box_spread
