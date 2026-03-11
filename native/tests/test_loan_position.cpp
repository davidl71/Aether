// test_loan_position.cpp - Loan position data model and manager tests
#include "loan_manager.h"
#include "loan_position.h"
#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>
#include <chrono>
#include <filesystem>
#include <fstream>

using namespace ib_box_spread;
using Catch::Matchers::WithinRel;

TEST_CASE("LoanPosition validation", "[loan]") {
  SECTION("Valid SHIR-based loan passes validation") {
    LoanPosition loan;
    loan.loan_id = "TEST-001";
    loan.bank_name = "Fibi";
    loan.account_number = "123456789";
    loan.loan_type = LoanType::SHIR_BASED;
    loan.principal = 500000.0;
    loan.original_principal = 500000.0;
    loan.interest_rate = 3.5;
    loan.spread = 1.2;
    loan.base_cpi = 0.0;
    loan.current_cpi = 0.0;
    loan.origination_date = std::chrono::system_clock::now();
    loan.maturity_date =
        loan.origination_date + std::chrono::hours(24 * 365 * 10); // 10 years
    loan.next_payment_date =
        loan.origination_date + std::chrono::hours(24 * 30); // 1 month
    loan.monthly_payment = 4500.0;
    loan.payment_frequency_months = 1;
    loan.status = LoanStatus::ACTIVE;
    loan.last_update = std::chrono::system_clock::now();

    REQUIRE(loan.is_valid());
  }

  SECTION("Valid CPI-linked loan passes validation") {
    LoanPosition loan;
    loan.loan_id = "TEST-002";
    loan.bank_name = "Discount";
    loan.account_number = "987654321";
    loan.loan_type = LoanType::CPI_LINKED;
    loan.principal = 750000.0;
    loan.original_principal = 700000.0;
    loan.interest_rate = 2.8;
    loan.spread = 0.0;
    loan.base_cpi = 105.2;
    loan.current_cpi = 112.5;
    loan.origination_date = std::chrono::system_clock::now();
    loan.maturity_date =
        loan.origination_date + std::chrono::hours(24 * 365 * 10);
    loan.next_payment_date =
        loan.origination_date + std::chrono::hours(24 * 30);
    loan.monthly_payment = 5200.0;
    loan.payment_frequency_months = 1;
    loan.status = LoanStatus::ACTIVE;
    loan.last_update = std::chrono::system_clock::now();

    REQUIRE(loan.is_valid());
  }

  SECTION("Empty loan_id fails validation") {
    LoanPosition loan;
    loan.loan_id = "";
    loan.bank_name = "Fibi";
    loan.account_number = "123456789";
    loan.principal = 500000.0;
    loan.original_principal = 500000.0;
    loan.interest_rate = 3.5;
    loan.monthly_payment = 4500.0;
    loan.payment_frequency_months = 1;
    loan.origination_date = std::chrono::system_clock::now();
    loan.maturity_date =
        loan.origination_date + std::chrono::hours(24 * 365 * 10);

    REQUIRE_FALSE(loan.is_valid());
  }

  SECTION("Negative principal fails validation") {
    LoanPosition loan;
    loan.loan_id = "TEST-003";
    loan.bank_name = "Fibi";
    loan.account_number = "123456789";
    loan.principal = -1000.0;
    loan.original_principal = 500000.0;
    loan.interest_rate = 3.5;
    loan.monthly_payment = 4500.0;
    loan.payment_frequency_months = 1;
    loan.origination_date = std::chrono::system_clock::now();
    loan.maturity_date =
        loan.origination_date + std::chrono::hours(24 * 365 * 10);

    REQUIRE_FALSE(loan.is_valid());
  }

  SECTION("Invalid date order fails validation") {
    LoanPosition loan;
    loan.loan_id = "TEST-004";
    loan.bank_name = "Fibi";
    loan.account_number = "123456789";
    loan.principal = 500000.0;
    loan.original_principal = 500000.0;
    loan.interest_rate = 3.5;
    loan.monthly_payment = 4500.0;
    loan.payment_frequency_months = 1;
    loan.origination_date = std::chrono::system_clock::now();
    loan.maturity_date = loan.origination_date -
                         std::chrono::hours(24 * 365); // Before origination

    REQUIRE_FALSE(loan.is_valid());
  }
}

TEST_CASE("LoanPosition calculations", "[loan]") {
  SECTION("CPI-linked loan principal adjustment") {
    LoanPosition loan;
    loan.loan_type = LoanType::CPI_LINKED;
    loan.original_principal = 700000.0;
    loan.base_cpi = 105.2;
    loan.current_cpi = 112.5;

    double adjusted = loan.get_adjusted_principal();
    double expected = 700000.0 * (112.5 / 105.2);
    REQUIRE_THAT(adjusted, WithinRel(expected, 0.001));
  }

  SECTION("SHIR-based loan interest rate calculation") {
    LoanPosition loan;
    loan.loan_type = LoanType::SHIR_BASED;
    loan.interest_rate = 3.5;
    loan.spread = 1.2;

    double rate = loan.get_current_interest_rate(4.0); // current_shir = 4.0
    REQUIRE_THAT(rate, WithinRel(5.2, 0.001));         // 4.0 + 1.2
  }

  SECTION("USD value conversion") {
    LoanPosition loan;
    loan.principal = 500000.0; // ILS
    loan.loan_type = LoanType::SHIR_BASED;

    double usd_value = loan.get_usd_value(0.27);         // ILS/USD = 0.27
    REQUIRE_THAT(usd_value, WithinRel(135000.0, 0.001)); // 500000 * 0.27
  }

  SECTION("Days until next payment") {
    LoanPosition loan;
    auto now = std::chrono::system_clock::now();
    loan.next_payment_date =
        now + std::chrono::hours(24 * 7); // 7 days from now

    int days = loan.days_until_next_payment();
    REQUIRE(days >= 6); // Allow for some rounding
    REQUIRE(days <= 8);
  }

  SECTION("Overdue detection") {
    LoanPosition loan;
    loan.status = LoanStatus::ACTIVE;
    loan.next_payment_date = std::chrono::system_clock::now() -
                             std::chrono::hours(24 * 2); // 2 days ago

    REQUIRE(loan.is_overdue());
  }
}

TEST_CASE("LoanManager CRUD operations", "[loan]") {
  // Create temporary test file
  std::string test_file = "/tmp/test_loans.json";
  std::filesystem::remove(test_file); // Clean up if exists

  LoanManager manager;
  REQUIRE(manager.initialize(test_file));

  SECTION("Add loan") {
    LoanPosition loan;
    loan.loan_id = "TEST-001";
    loan.bank_name = "Fibi";
    loan.account_number = "123456789";
    loan.loan_type = LoanType::SHIR_BASED;
    loan.principal = 500000.0;
    loan.original_principal = 500000.0;
    loan.interest_rate = 3.5;
    loan.spread = 1.2;
    loan.monthly_payment = 4500.0;
    loan.payment_frequency_months = 1;
    loan.origination_date = std::chrono::system_clock::now();
    loan.maturity_date =
        loan.origination_date + std::chrono::hours(24 * 365 * 10);
    loan.next_payment_date =
        loan.origination_date + std::chrono::hours(24 * 30);
    loan.status = LoanStatus::ACTIVE;
    loan.last_update = std::chrono::system_clock::now();

    REQUIRE(manager.add_loan(loan));

    auto retrieved = manager.get_loan("TEST-001");
    REQUIRE(retrieved.has_value());
    REQUIRE(retrieved->loan_id == "TEST-001");
    REQUIRE(retrieved->bank_name == "Fibi");
  }

  SECTION("Update loan") {
    LoanPosition loan;
    loan.loan_id = "TEST-002";
    loan.bank_name = "Discount";
    loan.account_number = "987654321";
    loan.loan_type = LoanType::CPI_LINKED;
    loan.principal = 750000.0;
    loan.original_principal = 700000.0;
    loan.interest_rate = 2.8;
    loan.spread = 0.0;
    loan.base_cpi = 105.2;
    loan.current_cpi = 112.5;
    loan.monthly_payment = 5200.0;
    loan.payment_frequency_months = 1;
    loan.origination_date = std::chrono::system_clock::now();
    loan.maturity_date =
        loan.origination_date + std::chrono::hours(24 * 365 * 10);
    loan.next_payment_date =
        loan.origination_date + std::chrono::hours(24 * 30);
    loan.status = LoanStatus::ACTIVE;
    loan.last_update = std::chrono::system_clock::now();

    REQUIRE(manager.add_loan(loan));

    // Update principal
    loan.principal = 700000.0;
    REQUIRE(manager.update_loan("TEST-002", loan));

    auto retrieved = manager.get_loan("TEST-002");
    REQUIRE(retrieved.has_value());
    REQUIRE_THAT(retrieved->principal, WithinRel(700000.0, 0.001));
  }

  SECTION("Delete loan") {
    LoanPosition loan;
    loan.loan_id = "TEST-003";
    loan.bank_name = "Fibi";
    loan.account_number = "111111111";
    loan.loan_type = LoanType::SHIR_BASED;
    loan.principal = 300000.0;
    loan.original_principal = 300000.0;
    loan.interest_rate = 3.0;
    loan.spread = 1.0;
    loan.monthly_payment = 3000.0;
    loan.payment_frequency_months = 1;
    loan.origination_date = std::chrono::system_clock::now();
    loan.maturity_date =
        loan.origination_date + std::chrono::hours(24 * 365 * 10);
    loan.next_payment_date =
        loan.origination_date + std::chrono::hours(24 * 30);
    loan.status = LoanStatus::ACTIVE;
    loan.last_update = std::chrono::system_clock::now();

    REQUIRE(manager.add_loan(loan));
    REQUIRE(manager.delete_loan("TEST-003"));

    auto retrieved = manager.get_loan("TEST-003");
    REQUIRE_FALSE(retrieved.has_value());
  }

  SECTION("Get all loans") {
    auto all_loans = manager.get_all_loans();
    REQUIRE(all_loans.size() >= 2); // At least TEST-001 and TEST-002
  }

  SECTION("Get active loans only") {
    auto active_loans = manager.get_active_loans();
    REQUIRE(active_loans.size() >= 2);

    for (const auto &loan : active_loans) {
      REQUIRE(loan.status == LoanStatus::ACTIVE);
    }
  }

  // Clean up
  std::filesystem::remove(test_file);
}

TEST_CASE("LoanManager calculations", "[loan]") {
  std::string test_file = "/tmp/test_loans_calc.json";
  std::filesystem::remove(test_file);

  LoanManager manager;
  REQUIRE(manager.initialize(test_file));

  // Add test loans
  LoanPosition loan1;
  loan1.loan_id = "CALC-001";
  loan1.bank_name = "Fibi";
  loan1.account_number = "111";
  loan1.loan_type = LoanType::SHIR_BASED;
  loan1.principal = 500000.0;
  loan1.original_principal = 500000.0;
  loan1.interest_rate = 3.5;
  loan1.spread = 1.2;
  loan1.monthly_payment = 4500.0;
  loan1.payment_frequency_months = 1;
  loan1.origination_date = std::chrono::system_clock::now();
  loan1.maturity_date =
      loan1.origination_date + std::chrono::hours(24 * 365 * 10);
  loan1.next_payment_date =
      loan1.origination_date + std::chrono::hours(24 * 30);
  loan1.status = LoanStatus::ACTIVE;
  loan1.last_update = std::chrono::system_clock::now();

  LoanPosition loan2;
  loan2.loan_id = "CALC-002";
  loan2.bank_name = "Discount";
  loan2.account_number = "222";
  loan2.loan_type = LoanType::CPI_LINKED;
  loan2.principal = 750000.0;
  loan2.original_principal = 700000.0;
  loan2.interest_rate = 2.8;
  loan2.spread = 0.0;
  loan2.base_cpi = 105.2;
  loan2.current_cpi = 112.5;
  loan2.monthly_payment = 5200.0;
  loan2.payment_frequency_months = 1;
  loan2.origination_date = std::chrono::system_clock::now();
  loan2.maturity_date =
      loan2.origination_date + std::chrono::hours(24 * 365 * 10);
  loan2.next_payment_date =
      loan2.origination_date + std::chrono::hours(24 * 30);
  loan2.status = LoanStatus::ACTIVE;
  loan2.last_update = std::chrono::system_clock::now();

  REQUIRE(manager.add_loan(loan1));
  REQUIRE(manager.add_loan(loan2));

  SECTION("Total loan liabilities in ILS") {
    double total_ils = manager.get_total_loan_liabilities_ils();
    // loan1: 500000, loan2: adjusted = 700000 * (112.5/105.2) ≈ 748,669
    double expected = 500000.0 + (700000.0 * (112.5 / 105.2));
    REQUIRE_THAT(total_ils, WithinRel(expected, 0.01));
  }

  SECTION("Total loan liabilities in USD") {
    double ils_usd_rate = 0.27;
    double total_usd = manager.get_total_loan_liabilities_usd(ils_usd_rate);
    double total_ils = manager.get_total_loan_liabilities_ils();
    double expected = total_ils * ils_usd_rate;
    REQUIRE_THAT(total_usd, WithinRel(expected, 0.01));
  }

  SECTION("Monthly payment total") {
    double monthly_total = manager.get_monthly_payment_total_ils();
    REQUIRE_THAT(monthly_total, WithinRel(9700.0, 0.01)); // 4500 + 5200
  }

  // Clean up
  std::filesystem::remove(test_file);
}

TEST_CASE("LoanManager persistence is retired", "[loan]") {
  LoanManager manager;
  REQUIRE(manager.initialize("/tmp/test_loans_persistence.json"));
  REQUIRE_FALSE(manager.save());
  REQUIRE_FALSE(manager.load());
}

TEST_CASE("LoanManager CPI and SHIR updates", "[loan]") {
  std::string test_file = "/tmp/test_loans_updates.json";
  std::filesystem::remove(test_file);

  LoanManager manager;
  REQUIRE(manager.initialize(test_file));

  LoanPosition cpi_loan;
  cpi_loan.loan_id = "CPI-001";
  cpi_loan.bank_name = "Discount";
  cpi_loan.account_number = "111";
  cpi_loan.loan_type = LoanType::CPI_LINKED;
  cpi_loan.principal = 750000.0;
  cpi_loan.original_principal = 700000.0;
  cpi_loan.interest_rate = 2.8;
  cpi_loan.spread = 0.0;
  cpi_loan.base_cpi = 105.2;
  cpi_loan.current_cpi = 112.5;
  cpi_loan.monthly_payment = 5200.0;
  cpi_loan.payment_frequency_months = 1;
  cpi_loan.origination_date = std::chrono::system_clock::now();
  cpi_loan.maturity_date =
      cpi_loan.origination_date + std::chrono::hours(24 * 365 * 10);
  cpi_loan.next_payment_date =
      cpi_loan.origination_date + std::chrono::hours(24 * 30);
  cpi_loan.status = LoanStatus::ACTIVE;
  cpi_loan.last_update = std::chrono::system_clock::now();

  REQUIRE(manager.add_loan(cpi_loan));

  SECTION("Update CPI for all loans") {
    double new_cpi = 115.0;
    REQUIRE(manager.update_cpi_for_all_loans(new_cpi));

    auto updated = manager.get_loan("CPI-001");
    REQUIRE(updated.has_value());
    REQUIRE_THAT(updated->current_cpi, WithinRel(115.0, 0.001));
  }

  SECTION("Update SHIR for all loans") {
    double new_shir = 4.5;
    REQUIRE(manager.update_shir_for_all_loans(new_shir));

    // SHIR update triggers recalculation
    auto updated = manager.get_loan("CPI-001");
    REQUIRE(updated.has_value());
    // last_update should be refreshed
  }

  // Clean up
  std::filesystem::remove(test_file);
}
