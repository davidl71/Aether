"""Tests for loan_manager.py - Python port of C++ loan manager."""

import json
import tempfile
from datetime import datetime
from pathlib import Path

import pytest

from python.integration.loan_manager import (
    LoanManager,
    LoanPosition,
    LoanStatus,
    LoanType,
)


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


def _make_loan(
    loan_id: str = "L-001",
    bank_name: str = "Test Bank",
    principal: float = 500_000.0,
    rate: float = 3.5,
    loan_type: LoanType = LoanType.FIXED,
    status: LoanStatus = LoanStatus.ACTIVE,
    monthly_payment: float = 2_500.0,
) -> LoanPosition:
    return LoanPosition(
        loan_id=loan_id,
        bank_name=bank_name,
        principal=principal,
        original_principal=principal,
        interest_rate=rate,
        loan_type=loan_type,
        status=status,
        monthly_payment=monthly_payment,
        origination_date=datetime(2023, 1, 1),
        maturity_date=datetime(2053, 1, 1),
        last_update=datetime.now(),
    )


# ---------------------------------------------------------------------------
# LoanPosition
# ---------------------------------------------------------------------------


class TestLoanPosition:
    def test_is_valid(self):
        loan = _make_loan()
        assert loan.is_valid()

    def test_invalid_no_id(self):
        loan = _make_loan(loan_id="")
        assert not loan.is_valid()

    def test_invalid_no_bank(self):
        loan = _make_loan(bank_name="")
        assert not loan.is_valid()

    def test_invalid_zero_principal(self):
        loan = _make_loan(principal=0)
        assert not loan.is_valid()

    def test_adjusted_principal_cpi(self):
        loan = _make_loan(loan_type=LoanType.CPI_LINKED)
        loan.base_cpi = 100.0
        loan.current_cpi = 110.0
        adjusted = loan.get_adjusted_principal()
        assert adjusted == pytest.approx(500_000.0 * 1.10)

    def test_adjusted_principal_non_cpi(self):
        loan = _make_loan(loan_type=LoanType.FIXED)
        assert loan.get_adjusted_principal() == pytest.approx(500_000.0)

    def test_effective_rate(self):
        loan = _make_loan(rate=3.5)
        loan.spread = 1.0
        assert loan.get_effective_rate() == pytest.approx(4.5)

    def test_to_dict_roundtrip(self):
        loan = _make_loan()
        d = loan.to_dict()
        loan2 = LoanPosition.from_dict(d)
        assert loan2.loan_id == loan.loan_id
        assert loan2.bank_name == loan.bank_name
        assert loan2.principal == pytest.approx(loan.principal)

    def test_from_dict_defaults(self):
        loan = LoanPosition.from_dict({"loan_id": "X", "bank_name": "B", "principal": 100})
        assert loan.loan_type == LoanType.FIXED
        assert loan.status == LoanStatus.ACTIVE


# ---------------------------------------------------------------------------
# LoanManager - CRUD
# ---------------------------------------------------------------------------


class TestLoanManagerCRUD:
    def test_add_loan(self):
        mgr = LoanManager()
        assert mgr.add_loan(_make_loan())
        assert mgr.loan_count() == 1

    def test_add_duplicate(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan())
        assert not mgr.add_loan(_make_loan())

    def test_add_invalid(self):
        mgr = LoanManager()
        assert not mgr.add_loan(_make_loan(loan_id=""))

    def test_get_loan(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1"))
        loan = mgr.get_loan("L-1")
        assert loan is not None
        assert loan.loan_id == "L-1"

    def test_get_missing(self):
        mgr = LoanManager()
        assert mgr.get_loan("nonexistent") is None

    def test_update_loan(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1"))
        updated = _make_loan(loan_id="L-1", principal=600_000)
        assert mgr.update_loan("L-1", updated)
        assert mgr.get_loan("L-1").principal == pytest.approx(600_000)

    def test_update_missing(self):
        mgr = LoanManager()
        assert not mgr.update_loan("missing", _make_loan())

    def test_update_invalid(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1"))
        assert not mgr.update_loan("L-1", _make_loan(loan_id=""))

    def test_delete_loan(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1"))
        assert mgr.delete_loan("L-1")
        assert mgr.loan_count() == 0

    def test_delete_missing(self):
        mgr = LoanManager()
        assert not mgr.delete_loan("missing")

    def test_get_all_loans(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1"))
        mgr.add_loan(_make_loan(loan_id="L-2"))
        assert len(mgr.get_all_loans()) == 2

    def test_get_active_loans(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1", status=LoanStatus.ACTIVE))
        mgr.add_loan(_make_loan(loan_id="L-2", status=LoanStatus.PAID_OFF))
        active = mgr.get_active_loans()
        assert len(active) == 1
        assert active[0].loan_id == "L-1"


# ---------------------------------------------------------------------------
# LoanManager - Aggregates
# ---------------------------------------------------------------------------


class TestLoanAggregates:
    def test_total_liabilities(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1", principal=100_000))
        mgr.add_loan(_make_loan(loan_id="L-2", principal=200_000))
        assert mgr.get_total_loan_liabilities() == pytest.approx(300_000)

    def test_total_liabilities_excludes_paid(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1", principal=100_000))
        mgr.add_loan(_make_loan(loan_id="L-2", principal=200_000, status=LoanStatus.PAID_OFF))
        assert mgr.get_total_loan_liabilities() == pytest.approx(100_000)

    def test_total_liabilities_usd(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1", principal=100_000))
        assert mgr.get_total_loan_liabilities_usd(0.28) == pytest.approx(28_000)

    def test_monthly_payment_total(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1", monthly_payment=1500))
        mgr.add_loan(_make_loan(loan_id="L-2", monthly_payment=2500))
        assert mgr.get_monthly_payment_total() == pytest.approx(4000)


# ---------------------------------------------------------------------------
# LoanManager - CPI / SHIR
# ---------------------------------------------------------------------------


class TestCPISHIR:
    def test_update_cpi(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1", loan_type=LoanType.CPI_LINKED))
        mgr.add_loan(_make_loan(loan_id="L-2", loan_type=LoanType.FIXED))
        updated = mgr.update_cpi_for_all_loans(110.0)
        assert updated == 1
        assert mgr.get_loan("L-1").current_cpi == pytest.approx(110.0)

    def test_update_shir(self):
        mgr = LoanManager()
        mgr.add_loan(_make_loan(loan_id="L-1", loan_type=LoanType.SHIR_BASED))
        updated = mgr.update_shir_for_all_loans(2.5)
        assert updated == 1

    def test_refresh_calculations(self):
        mgr = LoanManager()
        loan = _make_loan(loan_id="L-1", loan_type=LoanType.CPI_LINKED)
        loan.base_cpi = 100.0
        loan.current_cpi = 105.0
        mgr.add_loan(loan)
        mgr.refresh_loan_calculations()
        refreshed = mgr.get_loan("L-1")
        assert refreshed.principal == pytest.approx(500_000 * 1.05)


# ---------------------------------------------------------------------------
# LoanManager - Persistence
# ---------------------------------------------------------------------------


class TestPersistence:
    def test_save_and_load(self, tmp_path):
        fp = str(tmp_path / "loans.json")
        mgr = LoanManager(loans_file_path=fp)
        mgr.add_loan(_make_loan(loan_id="L-1"))
        mgr.add_loan(_make_loan(loan_id="L-2"))
        assert mgr.save()

        mgr2 = LoanManager(loans_file_path=fp)
        assert mgr2.loan_count() == 2
        assert mgr2.get_loan("L-1") is not None

    def test_save_no_path(self):
        mgr = LoanManager()
        assert not mgr.save()

    def test_load_no_path(self):
        mgr = LoanManager()
        assert not mgr.load()

    def test_load_missing_file(self, tmp_path):
        mgr = LoanManager(loans_file_path=str(tmp_path / "nope.json"))
        # load() is called in __init__ but returns False silently
        assert mgr.loan_count() == 0

    def test_load_invalid_json(self, tmp_path):
        fp = tmp_path / "bad.json"
        fp.write_text("not json!")
        mgr = LoanManager()
        mgr._file_path = str(fp)
        assert not mgr.load()

    def test_json_format(self, tmp_path):
        fp = str(tmp_path / "loans.json")
        mgr = LoanManager(loans_file_path=fp)
        mgr.add_loan(_make_loan(loan_id="L-1"))
        mgr.save()
        with open(fp) as f:
            data = json.load(f)
        assert data["version"] == "1.0"
        assert len(data["loans"]) == 1
        assert data["loans"][0]["loan_id"] == "L-1"
