// loan_manager.h - Bank loan position management
#pragma once

#include "loan_position.h"
#include <string>
#include <vector>
#include <unordered_map>
#include <optional>
#include <mutex>
#include <shared_mutex>

namespace ib_box_spread {

// ============================================================================
// LoanManager Class
// ============================================================================

class LoanManager {
public:
    LoanManager();
    ~LoanManager() = default;

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

    // Persistence (retired: Rust backend owns durable loan storage)
    bool save();
    bool load();

    // Validation
    bool validate_loan(const LoanPosition& loan) const;

private:
    std::string loans_file_path_;
    std::unordered_map<std::string, LoanPosition> loans_;
    mutable std::shared_mutex loans_mutex_;

};

} // namespace ib_box_spread
