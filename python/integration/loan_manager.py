"""
loan_manager.py - Bank loan position management (Python port)

Ported from native/src/loan_manager.cpp (357 LOC).

Manages bank loan positions with support for CPI-linked and SHIR-based
loans.  Provides CRUD operations, JSON persistence, and aggregate
calculations (total liabilities, monthly payments).
"""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Dict, List, Optional

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Enumerations
# ---------------------------------------------------------------------------


class LoanType(Enum):
    FIXED = "fixed"
    VARIABLE = "variable"
    CPI_LINKED = "cpi_linked"
    SHIR_BASED = "shir_based"


class LoanStatus(Enum):
    ACTIVE = "active"
    PAID_OFF = "paid_off"
    DEFAULTED = "defaulted"
    REFINANCED = "refinanced"


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class LoanPosition:
    loan_id: str = ""
    bank_name: str = ""
    account_number: str = ""
    loan_type: LoanType = LoanType.FIXED
    principal: float = 0.0
    original_principal: float = 0.0
    interest_rate: float = 0.0
    spread: float = 0.0
    base_cpi: float = 100.0
    current_cpi: float = 100.0
    origination_date: Optional[datetime] = None
    maturity_date: Optional[datetime] = None
    next_payment_date: Optional[datetime] = None
    monthly_payment: float = 0.0
    payment_frequency_months: int = 1
    status: LoanStatus = LoanStatus.ACTIVE
    last_update: Optional[datetime] = None

    def is_valid(self) -> bool:
        return (
            self.loan_id != ""
            and self.bank_name != ""
            and self.principal > 0
            and self.interest_rate >= 0
        )

    def get_adjusted_principal(self) -> float:
        """For CPI-linked loans, adjust principal by CPI ratio."""
        if self.loan_type == LoanType.CPI_LINKED and self.base_cpi > 0:
            return self.original_principal * (self.current_cpi / self.base_cpi)
        return self.principal

    def get_effective_rate(self) -> float:
        """Return interest rate plus any spread."""
        return self.interest_rate + self.spread

    def to_dict(self) -> Dict:
        def _dt(dt: Optional[datetime]) -> str:
            return dt.isoformat() if dt else ""

        return {
            "loan_id": self.loan_id,
            "bank_name": self.bank_name,
            "account_number": self.account_number,
            "loan_type": self.loan_type.value,
            "principal": self.principal,
            "original_principal": self.original_principal,
            "interest_rate": self.interest_rate,
            "spread": self.spread,
            "base_cpi": self.base_cpi,
            "current_cpi": self.current_cpi,
            "origination_date": _dt(self.origination_date),
            "maturity_date": _dt(self.maturity_date),
            "next_payment_date": _dt(self.next_payment_date),
            "monthly_payment": self.monthly_payment,
            "payment_frequency_months": self.payment_frequency_months,
            "status": self.status.value,
            "last_update": _dt(self.last_update),
        }

    @classmethod
    def from_dict(cls, data: Dict) -> "LoanPosition":
        def _parse_dt(val: str) -> Optional[datetime]:
            if not val:
                return None
            try:
                return datetime.fromisoformat(val)
            except ValueError:
                return None

        return cls(
            loan_id=data.get("loan_id", ""),
            bank_name=data.get("bank_name", ""),
            account_number=data.get("account_number", ""),
            loan_type=LoanType(data.get("loan_type", "fixed")),
            principal=data.get("principal", 0.0),
            original_principal=data.get("original_principal", 0.0),
            interest_rate=data.get("interest_rate", 0.0),
            spread=data.get("spread", 0.0),
            base_cpi=data.get("base_cpi", 100.0),
            current_cpi=data.get("current_cpi", 100.0),
            origination_date=_parse_dt(data.get("origination_date", "")),
            maturity_date=_parse_dt(data.get("maturity_date", "")),
            next_payment_date=_parse_dt(data.get("next_payment_date", "")),
            monthly_payment=data.get("monthly_payment", 0.0),
            payment_frequency_months=data.get("payment_frequency_months", 1),
            status=LoanStatus(data.get("status", "active")),
            last_update=_parse_dt(data.get("last_update", "")),
        )


# ---------------------------------------------------------------------------
# LoanManager
# ---------------------------------------------------------------------------


class LoanManager:
    """CRUD manager for bank loan positions with JSON persistence."""

    def __init__(self, loans_file_path: Optional[str] = None):
        self._loans: Dict[str, LoanPosition] = {}
        self._file_path = loans_file_path or ""

        if self._file_path:
            self.load()

    # -- CRUD --

    def add_loan(self, loan: LoanPosition) -> bool:
        if not loan.is_valid():
            logger.error("Invalid loan data for loan_id: %s", loan.loan_id)
            return False
        if loan.loan_id in self._loans:
            logger.error("Loan with ID %s already exists", loan.loan_id)
            return False
        self._loans[loan.loan_id] = loan
        logger.info("Added loan: %s (%s)", loan.loan_id, loan.bank_name)
        return True

    def update_loan(self, loan_id: str, loan: LoanPosition) -> bool:
        if not loan.is_valid():
            logger.error("Invalid loan data for loan_id: %s", loan_id)
            return False
        if loan_id not in self._loans:
            logger.error("Loan with ID %s not found", loan_id)
            return False
        self._loans[loan_id] = loan
        return True

    def delete_loan(self, loan_id: str) -> bool:
        if loan_id not in self._loans:
            logger.error("Loan with ID %s not found", loan_id)
            return False
        del self._loans[loan_id]
        return True

    def get_loan(self, loan_id: str) -> Optional[LoanPosition]:
        return self._loans.get(loan_id)

    def get_all_loans(self) -> List[LoanPosition]:
        return list(self._loans.values())

    def get_active_loans(self) -> List[LoanPosition]:
        return [loan for loan in self._loans.values() if loan.status == LoanStatus.ACTIVE]

    # -- aggregates --

    def get_total_loan_liabilities(self) -> float:
        return sum(
            loan.get_adjusted_principal()
            for loan in self._loans.values()
            if loan.status == LoanStatus.ACTIVE
        )

    def get_total_loan_liabilities_usd(self, exchange_rate: float = 1.0) -> float:
        return self.get_total_loan_liabilities() * exchange_rate

    def get_monthly_payment_total(self) -> float:
        return sum(
            loan.monthly_payment
            for loan in self._loans.values()
            if loan.status == LoanStatus.ACTIVE
        )

    # -- CPI / SHIR updates --

    def update_cpi_for_all_loans(self, current_cpi: float) -> int:
        updated = 0
        for loan in self._loans.values():
            if loan.loan_type == LoanType.CPI_LINKED and loan.status == LoanStatus.ACTIVE:
                loan.current_cpi = current_cpi
                loan.last_update = datetime.now()
                updated += 1
        return updated

    def update_shir_for_all_loans(self, current_shir: float) -> int:
        self.refresh_loan_calculations()
        updated = 0
        for loan in self._loans.values():
            if loan.loan_type == LoanType.SHIR_BASED and loan.status == LoanStatus.ACTIVE:
                loan.last_update = datetime.now()
                updated += 1
        return updated

    def refresh_loan_calculations(self) -> None:
        for loan in self._loans.values():
            if loan.status == LoanStatus.ACTIVE:
                if loan.loan_type == LoanType.CPI_LINKED:
                    loan.principal = loan.get_adjusted_principal()
                loan.last_update = datetime.now()

    # -- persistence --

    def save(self) -> bool:
        if not self._file_path:
            logger.error("Loans file path not set")
            return False
        try:
            data = {
                "version": "1.0",
                "last_updated": datetime.now().isoformat(),
                "loans": [loan.to_dict() for loan in self._loans.values()],
            }
            Path(self._file_path).parent.mkdir(parents=True, exist_ok=True)
            with open(self._file_path, "w") as f:
                json.dump(data, f, indent=2)
            logger.info("Saved %d loans to %s", len(self._loans), self._file_path)
            return True
        except Exception as e:
            logger.error("Error saving loans: %s", e)
            return False

    def load(self) -> bool:
        if not self._file_path:
            logger.error("Loans file path not set")
            return False
        path = Path(self._file_path)
        if not path.exists():
            logger.warning("Loans file not found: %s", self._file_path)
            return False
        try:
            with open(path) as f:
                data = json.load(f)
            self._loans.clear()
            for ld in data.get("loans", []):
                loan = LoanPosition.from_dict(ld)
                if loan.is_valid():
                    self._loans[loan.loan_id] = loan
            logger.info("Loaded %d loans from %s", len(self._loans), self._file_path)
            return True
        except Exception as e:
            logger.error("Error loading loans: %s", e)
            return False

    def loan_count(self) -> int:
        return len(self._loans)
