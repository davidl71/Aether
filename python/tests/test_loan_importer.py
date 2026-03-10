import json

from python.tui.components.loan_entry import LoanImporter, LoanManager


def _write_csv(path, rows):
    headers = [
        "loan_id",
        "bank_name",
        "account_number",
        "loan_type",
        "principal",
        "original_principal",
        "interest_rate",
        "spread",
        "base_cpi",
        "current_cpi",
        "origination_date",
        "maturity_date",
        "next_payment_date",
        "monthly_payment",
        "payment_frequency_months",
        "status",
    ]
    with open(path, "w", encoding="utf-8") as f:
        f.write(",".join(headers) + "\n")
        for row in rows:
            f.write(",".join(row.get(header, "") for header in headers) + "\n")


def _make_importer(tmp_path):
    manager = LoanManager(str(tmp_path / "loans.json"))
    importer = LoanImporter(manager)
    return manager, importer


def test_csv_import_rejects_invalid_loan_type(tmp_path):
    manager, importer = _make_importer(tmp_path)
    csv_path = tmp_path / "loans.csv"
    _write_csv(
        csv_path,
        [{
            "loan_id": "loan-1",
            "bank_name": "Discount",
            "account_number": "123",
            "loan_type": "BAD_TYPE",
            "principal": "1000",
            "original_principal": "1000",
            "interest_rate": "4",
            "spread": "0",
            "base_cpi": "100",
            "current_cpi": "100",
            "origination_date": "2025-01-01",
            "maturity_date": "2030-01-01",
            "next_payment_date": "2025-02-01",
            "monthly_payment": "100",
            "payment_frequency_months": "1",
            "status": "ACTIVE",
        }],
    )

    imported, error_count, errors = importer.import_from_csv(str(csv_path))

    assert imported == 0
    assert error_count == 1
    assert "Loan type must be one of" in errors[0]
    assert manager.loans == {}


def test_csv_import_rejects_invalid_status(tmp_path):
    manager, importer = _make_importer(tmp_path)
    csv_path = tmp_path / "loans.csv"
    _write_csv(
        csv_path,
        [{
            "loan_id": "loan-1",
            "bank_name": "Discount",
            "account_number": "123",
            "loan_type": "SHIR_BASED",
            "principal": "1000",
            "original_principal": "1000",
            "interest_rate": "4",
            "spread": "0",
            "base_cpi": "100",
            "current_cpi": "100",
            "origination_date": "2025-01-01",
            "maturity_date": "2030-01-01",
            "next_payment_date": "2025-02-01",
            "monthly_payment": "100",
            "payment_frequency_months": "1",
            "status": "UNKNOWN",
        }],
    )

    imported, error_count, errors = importer.import_from_csv(str(csv_path))

    assert imported == 0
    assert error_count == 1
    assert "Status must be one of" in errors[0]
    assert manager.loans == {}


def test_csv_import_rejects_invalid_date(tmp_path):
    manager, importer = _make_importer(tmp_path)
    csv_path = tmp_path / "loans.csv"
    _write_csv(
        csv_path,
        [{
            "loan_id": "loan-1",
            "bank_name": "Discount",
            "account_number": "123",
            "loan_type": "SHIR_BASED",
            "principal": "1000",
            "original_principal": "1000",
            "interest_rate": "4",
            "spread": "0",
            "base_cpi": "100",
            "current_cpi": "100",
            "origination_date": "not-a-date",
            "maturity_date": "2030-01-01",
            "next_payment_date": "2025-02-01",
            "monthly_payment": "100",
            "payment_frequency_months": "1",
            "status": "ACTIVE",
        }],
    )

    imported, error_count, errors = importer.import_from_csv(str(csv_path))

    assert imported == 0
    assert error_count == 1
    assert "origination_date must be a valid date" in errors[0]
    assert manager.loans == {}


def test_csv_import_rejects_missing_required_date(tmp_path):
    manager, importer = _make_importer(tmp_path)
    csv_path = tmp_path / "loans.csv"
    _write_csv(
        csv_path,
        [{
            "loan_id": "loan-1",
            "bank_name": "Discount",
            "account_number": "123",
            "loan_type": "SHIR_BASED",
            "principal": "1000",
            "original_principal": "1000",
            "interest_rate": "4",
            "spread": "0",
            "base_cpi": "100",
            "current_cpi": "100",
            "origination_date": "",
            "maturity_date": "2030-01-01",
            "next_payment_date": "2025-02-01",
            "monthly_payment": "100",
            "payment_frequency_months": "1",
            "status": "ACTIVE",
        }],
    )

    imported, error_count, errors = importer.import_from_csv(str(csv_path))

    assert imported == 0
    assert error_count == 1
    assert "origination_date is required" in errors[0]
    assert manager.loans == {}


def test_json_import_rejects_invalid_enum_values(tmp_path):
    manager, importer = _make_importer(tmp_path)
    json_path = tmp_path / "loans.json"
    json_path.write_text(json.dumps([{
        "loan_id": "loan-1",
        "bank_name": "Discount",
        "account_number": "123",
        "loan_type": "BAD_TYPE",
        "principal": 1000,
        "original_principal": 1000,
        "interest_rate": 4,
        "spread": 0,
        "base_cpi": 100,
        "current_cpi": 100,
        "origination_date": "2025-01-01T00:00:00Z",
        "maturity_date": "2030-01-01T00:00:00Z",
        "next_payment_date": "2025-02-01T00:00:00Z",
        "monthly_payment": 100,
        "payment_frequency_months": 1,
        "status": "UNKNOWN",
        "last_update": "2025-01-01T00:00:00Z",
    }]))

    imported, error_count, errors = importer.import_from_json(str(json_path))

    assert imported == 0
    assert error_count == 1
    assert "Loan type must be one of" in errors[0]
    assert "Status must be one of" in errors[0]
    assert manager.loans == {}


def test_csv_import_accepts_valid_row(tmp_path):
    manager, importer = _make_importer(tmp_path)
    csv_path = tmp_path / "loans.csv"
    _write_csv(
        csv_path,
        [{
            "loan_id": "loan-1",
            "bank_name": "Discount",
            "account_number": "123",
            "loan_type": "SHIR_BASED",
            "principal": "1000",
            "original_principal": "1000",
            "interest_rate": "4",
            "spread": "0",
            "base_cpi": "100",
            "current_cpi": "100",
            "origination_date": "2025-01-01",
            "maturity_date": "2030-01-01",
            "next_payment_date": "2025-02-01",
            "monthly_payment": "100",
            "payment_frequency_months": "1",
            "status": "ACTIVE",
        }],
    )

    imported, error_count, errors = importer.import_from_csv(str(csv_path))

    assert imported == 1
    assert error_count == 0
    assert errors == []
    assert "loan-1" in manager.loans
