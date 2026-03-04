// loan_manager.cpp - Bank loan position management implementation
#include "loan_manager.h"
#include <fstream>
#include <nlohmann/json.hpp>
#include <spdlog/spdlog.h>

namespace ib_box_spread {

using json = nlohmann::json;

// ============================================================================
// LoanManager Implementation
// ============================================================================

LoanManager::LoanManager() : loans_file_path_() {}

bool LoanManager::initialize(const std::string &loans_file_path) {
  std::lock_guard<std::shared_mutex> lock(loans_mutex_);
  loans_file_path_ = loans_file_path;

  // Try to load existing loans
  if (!load_from_json()) {
    spdlog::warn("Could not load loans from {}, starting with empty loan list",
                 loans_file_path);
    // Not an error - file might not exist yet
  }

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

bool LoanManager::save() { return save_to_json(); }

bool LoanManager::load() { return load_from_json(); }

bool LoanManager::validate_loan(const LoanPosition &loan) const {
  return loan.is_valid();
}

// ============================================================================
// Private Helper Methods
// ============================================================================

bool LoanManager::save_to_json() const {
  std::shared_lock<std::shared_mutex> lock(loans_mutex_);

  if (loans_file_path_.empty()) {
    spdlog::error("Loans file path not set");
    return false;
  }

  try {
    json j;
    j["version"] = "1.0";
    j["last_updated"] = time_point_to_iso8601(std::chrono::system_clock::now());

    json loans_array = json::array();
    for (const auto &[loan_id, loan] : loans_) {
      json loan_json;
      loan_json["loan_id"] = loan.loan_id;
      loan_json["bank_name"] = loan.bank_name;
      loan_json["account_number"] = loan.account_number;
      loan_json["loan_type"] = loan_type_to_string(loan.loan_type);
      loan_json["principal"] = loan.principal;
      loan_json["original_principal"] = loan.original_principal;
      loan_json["interest_rate"] = loan.interest_rate;
      loan_json["spread"] = loan.spread;
      loan_json["base_cpi"] = loan.base_cpi;
      loan_json["current_cpi"] = loan.current_cpi;
      loan_json["origination_date"] =
          time_point_to_iso8601(loan.origination_date);
      loan_json["maturity_date"] = time_point_to_iso8601(loan.maturity_date);
      loan_json["next_payment_date"] =
          time_point_to_iso8601(loan.next_payment_date);
      loan_json["monthly_payment"] = loan.monthly_payment;
      loan_json["payment_frequency_months"] = loan.payment_frequency_months;
      loan_json["status"] = loan_status_to_string(loan.status);
      loan_json["last_update"] = time_point_to_iso8601(loan.last_update);

      loans_array.push_back(loan_json);
    }

    j["loans"] = loans_array;

    std::ofstream file(loans_file_path_);
    if (!file.is_open()) {
      spdlog::error("Failed to open loans file for writing: {}",
                    loans_file_path_);
      return false;
    }

    file << j.dump(2); // Pretty print with 2-space indent
    file.close();

    spdlog::info("Saved {} loans to {}", loans_.size(), loans_file_path_);
    return true;

  } catch (const std::exception &e) {
    spdlog::error("Error saving loans to JSON: {}", e.what());
    return false;
  }
}

bool LoanManager::load_from_json() {
  std::lock_guard<std::shared_mutex> lock(loans_mutex_);

  if (loans_file_path_.empty()) {
    spdlog::error("Loans file path not set");
    return false;
  }

  std::ifstream file(loans_file_path_);
  if (!file.is_open()) {
    spdlog::warn("Loans file not found: {}", loans_file_path_);
    return false;
  }

  try {
    json j;
    file >> j;
    file.close();

    // Validate version
    if (j.contains("version")) {
      std::string version = j["version"];
      if (version != "1.0") {
        spdlog::warn("Unexpected loans file version: {}", version);
      }
    }

    // Clear existing loans
    loans_.clear();

    // Load loans
    if (j.contains("loans") && j["loans"].is_array()) {
      for (const auto &loan_json : j["loans"]) {
        LoanPosition loan;
        loan.loan_id = loan_json["loan_id"];
        loan.bank_name = loan_json["bank_name"];
        loan.account_number = loan_json["account_number"];
        loan.loan_type = string_to_loan_type(loan_json["loan_type"]);
        loan.principal = loan_json["principal"];
        loan.original_principal = loan_json["original_principal"];
        loan.interest_rate = loan_json["interest_rate"];
        loan.spread = loan_json["spread"];
        loan.base_cpi = loan_json["base_cpi"];
        loan.current_cpi = loan_json["current_cpi"];
        loan.origination_date =
            iso8601_to_time_point(loan_json["origination_date"]);
        loan.maturity_date = iso8601_to_time_point(loan_json["maturity_date"]);
        loan.next_payment_date =
            iso8601_to_time_point(loan_json["next_payment_date"]);
        loan.monthly_payment = loan_json["monthly_payment"];
        loan.payment_frequency_months = loan_json["payment_frequency_months"];
        loan.status = string_to_loan_status(loan_json["status"]);
        loan.last_update = iso8601_to_time_point(loan_json["last_update"]);

        // Validate before adding
        if (loan.is_valid()) {
          loans_[loan.loan_id] = loan;
        } else {
          spdlog::warn("Skipping invalid loan: {}", loan.loan_id);
        }
      }
    }

    spdlog::info("Loaded {} loans from {}", loans_.size(), loans_file_path_);
    return true;

  } catch (const std::exception &e) {
    spdlog::error("Error loading loans from JSON: {}", e.what());
    return false;
  }
}

} // namespace ib_box_spread
