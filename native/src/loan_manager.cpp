// loan_manager.cpp - Bank loan position management implementation
//
// This JSON persistence path is compatibility legacy for config/loans.json.
// It should not expand; long term durable loan ownership is expected to move
// behind a Rust-owned API/store.
#include "loan_manager.h"
#include <spdlog/spdlog.h>

namespace ib_box_spread {

// ============================================================================
// LoanManager Implementation
// ============================================================================

LoanManager::LoanManager() : loans_file_path_() {}

bool LoanManager::initialize(const std::string &loans_file_path) {
  std::lock_guard<std::shared_mutex> lock(loans_mutex_);
  loans_file_path_ = loans_file_path;
  spdlog::warn(
      "Native LoanManager persistence is retired; using in-memory compatibility "
      "mode only. Requested path: {}",
      loans_file_path_);
  return true;
}

bool LoanManager::add_loan(const LoanPosition &loan) {
  if (!validate_loan(loan)) {
    spdlog::error("Invalid loan data for loan_id: {}", loan.loan_id);
    return false;
  }

  std::lock_guard<std::shared_mutex> lock(loans_mutex_);

  // Check if loan_id already exists
  if (loans_.find(loan.loan_id) != loans_.end()) {
    spdlog::error("Loan with ID {} already exists", loan.loan_id);
    return false;
  }

  loans_[loan.loan_id] = loan;
  spdlog::info("Added loan: {} ({})", loan.loan_id, loan.bank_name);

  return true;
}

bool LoanManager::update_loan(const std::string &loan_id,
                              const LoanPosition &loan) {
  if (!validate_loan(loan)) {
    spdlog::error("Invalid loan data for loan_id: {}", loan_id);
    return false;
  }

  std::lock_guard<std::shared_mutex> lock(loans_mutex_);

  if (loans_.find(loan_id) == loans_.end()) {
    spdlog::error("Loan with ID {} not found", loan_id);
    return false;
  }

  loans_[loan_id] = loan;
  spdlog::info("Updated loan: {}", loan_id);

  return true;
}

bool LoanManager::delete_loan(const std::string &loan_id) {
  std::lock_guard<std::shared_mutex> lock(loans_mutex_);

  auto it = loans_.find(loan_id);
  if (it == loans_.end()) {
    spdlog::error("Loan with ID {} not found", loan_id);
    return false;
  }

  loans_.erase(it);
  spdlog::info("Deleted loan: {}", loan_id);

  return true;
}

std::optional<LoanPosition>
LoanManager::get_loan(const std::string &loan_id) const {
  std::shared_lock<std::shared_mutex> lock(loans_mutex_);

  auto it = loans_.find(loan_id);
  if (it == loans_.end()) {
    return std::nullopt;
  }

  return it->second;
}

std::vector<LoanPosition> LoanManager::get_all_loans() const {
  std::shared_lock<std::shared_mutex> lock(loans_mutex_);

  std::vector<LoanPosition> result;
  result.reserve(loans_.size());

  for (const auto &[loan_id, loan] : loans_) {
    result.push_back(loan);
  }

  return result;
}

std::vector<LoanPosition> LoanManager::get_active_loans() const {
  std::shared_lock<std::shared_mutex> lock(loans_mutex_);

  std::vector<LoanPosition> result;

  for (const auto &[loan_id, loan] : loans_) {
    if (loan.status == LoanStatus::ACTIVE) {
      result.push_back(loan);
    }
  }

  return result;
}

double LoanManager::get_total_loan_liabilities_ils() const {
  std::shared_lock<std::shared_mutex> lock(loans_mutex_);

  double total = 0.0;

  for (const auto &[loan_id, loan] : loans_) {
    if (loan.status == LoanStatus::ACTIVE) {
      total += loan.get_adjusted_principal();
    }
  }

  return total;
}

double LoanManager::get_total_loan_liabilities_usd(double ils_usd_rate) const {
  double total_ils = get_total_loan_liabilities_ils();
  return total_ils * ils_usd_rate;
}

double LoanManager::get_monthly_payment_total_ils() const {
  std::shared_lock<std::shared_mutex> lock(loans_mutex_);

  double total = 0.0;

  for (const auto &[loan_id, loan] : loans_) {
    if (loan.status == LoanStatus::ACTIVE) {
      total += loan.monthly_payment;
    }
  }

  return total;
}

bool LoanManager::update_cpi_for_all_loans(double current_cpi) {
  std::lock_guard<std::shared_mutex> lock(loans_mutex_);

  int updated = 0;
  for (auto &[loan_id, loan] : loans_) {
    if (loan.loan_type == LoanType::CPI_LINKED &&
        loan.status == LoanStatus::ACTIVE) {
      loan.current_cpi = current_cpi;
      loan.last_update = std::chrono::system_clock::now();
      updated++;
    }
  }

  if (updated > 0) {
    spdlog::info("Updated CPI for {} loans", updated);
  }

  return true;
}

bool LoanManager::update_shir_for_all_loans(double current_shir) {
  // Note: SHIR is used in calculations, not stored directly
  // This method triggers recalculation of interest rates
  refresh_loan_calculations();

  std::lock_guard<std::shared_mutex> lock(loans_mutex_);

  int updated = 0;
  for (auto &[loan_id, loan] : loans_) {
    if (loan.loan_type == LoanType::SHIR_BASED &&
        loan.status == LoanStatus::ACTIVE) {
      // Update last_update timestamp to reflect SHIR update
      loan.last_update = std::chrono::system_clock::now();
      updated++;
    }
  }

  if (updated > 0) {
    spdlog::info("Updated SHIR calculations for {} loans", updated);
  }

  return true;
}

void LoanManager::refresh_loan_calculations() {
  // Refresh loan calculations (principal adjustments, etc.)
  // This is called after CPI or SHIR updates
  std::lock_guard<std::shared_mutex> lock(loans_mutex_);

  for (auto &[loan_id, loan] : loans_) {
    if (loan.status == LoanStatus::ACTIVE) {
      // Update principal for CPI-linked loans
      if (loan.loan_type == LoanType::CPI_LINKED) {
        loan.principal = loan.get_adjusted_principal();
      }
      loan.last_update = std::chrono::system_clock::now();
    }
  }
}

bool LoanManager::save() {
  spdlog::warn(
      "Native LoanManager::save() is retired; durable loan storage belongs to "
      "the Rust backend");
  return false;
}

bool LoanManager::load() {
  spdlog::warn(
      "Native LoanManager::load() is retired; durable loan storage belongs to "
      "the Rust backend");
  return false;
}

bool LoanManager::validate_loan(const LoanPosition &loan) const {
  return loan.is_valid();
}

} // namespace ib_box_spread
