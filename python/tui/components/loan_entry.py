"""
Loan entry and management component for TUI

Provides manual loan entry form, file import, and loan list management.
Uses JSON storage compatible with C++ LoanManager.
"""

from __future__ import annotations

import json
import csv
import logging
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any
from dataclasses import dataclass, asdict

from textual.app import ComposeResult
from textual.containers import Container, Horizontal, Vertical, ScrollableContainer
from textual.widgets import (
    Input, Select, Button, Label, DataTable, Static, Log
)
from textual.message import Message

logger = logging.getLogger(__name__)


@dataclass
class LoanPosition:
    """Loan position data model (matches C++ LoanPosition structure)"""
    loan_id: str
    bank_name: str
    account_number: str
    loan_type: str  # "SHIR_BASED" or "CPI_LINKED"
    principal: float
    original_principal: float
    interest_rate: float
    spread: float
    base_cpi: float
    current_cpi: float
    origination_date: str  # ISO 8601 format
    maturity_date: str  # ISO 8601 format
    next_payment_date: str  # ISO 8601 format
    monthly_payment: float
    payment_frequency_months: int
    status: str  # "ACTIVE", "PAID_OFF", "DEFAULTED"
    last_update: str  # ISO 8601 format

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization"""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'LoanPosition':
        """Create from dictionary (JSON deserialization)"""
        return cls(**data)

    def is_valid(self) -> tuple[bool, List[str]]:
        """Validate loan data"""
        errors = []

        if not self.loan_id:
            errors.append("Loan ID is required")
        if not self.bank_name:
            errors.append("Bank name is required")
        if not self.account_number:
            errors.append("Account number is required")
        if self.principal <= 0:
            errors.append("Principal must be > 0")
        if self.original_principal <= 0:
            errors.append("Original principal must be > 0")
        if self.interest_rate < 0:
            errors.append("Interest rate must be >= 0")
        if self.spread < 0:
            errors.append("Spread must be >= 0")
        if self.monthly_payment <= 0:
            errors.append("Monthly payment must be > 0")
        if self.payment_frequency_months <= 0:
            errors.append("Payment frequency must be > 0")

        # Validate dates
        try:
            orig = datetime.fromisoformat(self.origination_date.replace('Z', '+00:00'))
            mat = datetime.fromisoformat(self.maturity_date.replace('Z', '+00:00'))
            if orig >= mat:
                errors.append("Origination date must be before maturity date")
        except ValueError as e:
            errors.append(f"Invalid date format: {e}")

        # Validate CPI fields for CPI-linked loans
        if self.loan_type == "CPI_LINKED":
            if self.base_cpi <= 0:
                errors.append("Base CPI must be > 0 for CPI-linked loans")
            if self.current_cpi <= 0:
                errors.append("Current CPI must be > 0 for CPI-linked loans")

        return len(errors) == 0, errors


class LoanManager:
    """Python loan manager (reads/writes JSON compatible with C++ LoanManager)"""

    def __init__(self, loans_file_path: str = "config/loans.json"):
        self.loans_file_path = Path(loans_file_path)
        self.loans: Dict[str, LoanPosition] = {}
        self.load()

    def load(self) -> bool:
        """Load loans from JSON file"""
        if not self.loans_file_path.exists():
            logger.warning(f"Loans file not found: {self.loans_file_path}")
            return False

        try:
            with open(self.loans_file_path, 'r') as f:
                data = json.load(f)

            self.loans.clear()
            if 'loans' in data and isinstance(data['loans'], list):
                for loan_data in data['loans']:
                    loan = LoanPosition.from_dict(loan_data)
                    is_valid, errors = loan.is_valid()
                    if is_valid:
                        self.loans[loan.loan_id] = loan
                    else:
                        logger.warning(f"Skipping invalid loan {loan.loan_id}: {errors}")

            logger.info(f"Loaded {len(self.loans)} loans from {self.loans_file_path}")
            return True
        except Exception as e:
            logger.error(f"Error loading loans: {e}")
            return False

    def save(self) -> bool:
        """Save loans to JSON file"""
        try:
            # Ensure directory exists
            self.loans_file_path.parent.mkdir(parents=True, exist_ok=True)

            data = {
                "version": "1.0",
                "last_updated": datetime.utcnow().isoformat() + "Z",
                "loans": [loan.to_dict() for loan in self.loans.values()]
            }

            with open(self.loans_file_path, 'w') as f:
                json.dump(data, f, indent=2)

            logger.info(f"Saved {len(self.loans)} loans to {self.loans_file_path}")
            return True
        except Exception as e:
            logger.error(f"Error saving loans: {e}")
            return False

    def add_loan(self, loan: LoanPosition) -> tuple[bool, Optional[str]]:
        """Add a loan"""
        is_valid, errors = loan.is_valid()
        if not is_valid:
            return False, "; ".join(errors)

        if loan.loan_id in self.loans:
            return False, f"Loan with ID {loan.loan_id} already exists"

        self.loans[loan.loan_id] = loan
        return True, None

    def update_loan(self, loan_id: str, loan: LoanPosition) -> tuple[bool, Optional[str]]:
        """Update a loan"""
        is_valid, errors = loan.is_valid()
        if not is_valid:
            return False, "; ".join(errors)

        if loan_id not in self.loans:
            return False, f"Loan with ID {loan_id} not found"

        self.loans[loan_id] = loan
        return True, None

    def delete_loan(self, loan_id: str) -> bool:
        """Delete a loan"""
        if loan_id in self.loans:
            del self.loans[loan_id]
            return True
        return False

    def get_loan(self, loan_id: str) -> Optional[LoanPosition]:
        """Get a loan by ID"""
        return self.loans.get(loan_id)

    def get_all_loans(self) -> List[LoanPosition]:
        """Get all loans"""
        return list(self.loans.values())

    def get_active_loans(self) -> List[LoanPosition]:
        """Get active loans only"""
        return [loan for loan in self.loans.values() if loan.status == "ACTIVE"]


class LoanEntryForm(Container):
    """Form for entering/editing a loan"""

    def __init__(self, loan_manager: LoanManager, existing_loan: Optional[LoanPosition] = None):
        super().__init__()
        self.loan_manager = loan_manager
        self.existing_loan = existing_loan
        self.is_editing = existing_loan is not None

    class LoanSaved(Message):
        """Message sent when loan is saved"""
        def __init__(self, loan: LoanPosition):
            super().__init__()
            self.loan = loan

    class LoanFormCancelled(Message):
        """Message sent when form is cancelled"""
        pass

    def compose(self) -> ComposeResult:
        with ScrollableContainer():
            yield Label("Loan Entry Form", classes="form-title")

            # Loan ID
            yield Label("Loan ID *")
            yield Input(
                value=self.existing_loan.loan_id if self.existing_loan else "",
                placeholder="e.g., FIBI-001",
                id="loan-id",
                disabled=self.is_editing  # Can't change ID when editing
            )

            # Bank name
            yield Label("Bank Name *")
            yield Select(
                options=[("Fibi", "Fibi"), ("Discount", "Discount"), ("Other", "Other")],
                value=self.existing_loan.bank_name if self.existing_loan else "Fibi",
                id="bank-name"
            )

            # Account number
            yield Label("Account Number *")
            yield Input(
                value=self.existing_loan.account_number if self.existing_loan else "",
                placeholder="e.g., 123456789",
                id="account-number"
            )

            # Loan type
            yield Label("Loan Type *")
            yield Select(
                options=[("SHIR_BASED", "SHIR-based (Variable Rate)"), ("CPI_LINKED", "CPI-linked (Fixed Rate)")],
                value=self.existing_loan.loan_type if self.existing_loan else "SHIR_BASED",
                id="loan-type"
            )

            # Principal
            yield Label("Current Principal (ILS) *")
            yield Input(
                value=str(self.existing_loan.principal) if self.existing_loan else "",
                placeholder="e.g., 500000",
                id="principal"
            )

            # Original principal
            yield Label("Original Principal (ILS) *")
            yield Input(
                value=str(self.existing_loan.original_principal) if self.existing_loan else "",
                placeholder="e.g., 500000",
                id="original-principal"
            )

            # Interest rate
            yield Label("Interest Rate (%) *")
            yield Input(
                value=str(self.existing_loan.interest_rate) if self.existing_loan else "",
                placeholder="e.g., 3.5",
                id="interest-rate"
            )

            # Spread (for SHIR-based)
            yield Label("Spread (%) *")
            yield Input(
                value=str(self.existing_loan.spread) if self.existing_loan else "0.0",
                placeholder="e.g., 1.2",
                id="spread"
            )

            # Base CPI (for CPI-linked)
            yield Label("Base CPI")
            yield Input(
                value=str(self.existing_loan.base_cpi) if self.existing_loan else "0.0",
                placeholder="e.g., 105.2",
                id="base-cpi"
            )

            # Current CPI
            yield Label("Current CPI")
            yield Input(
                value=str(self.existing_loan.current_cpi) if self.existing_loan else "0.0",
                placeholder="e.g., 112.5",
                id="current-cpi"
            )

            # Dates
            yield Label("Origination Date (YYYY-MM-DD) *")
            yield Input(
                value=self.existing_loan.origination_date.split('T')[0] if self.existing_loan and self.existing_loan.origination_date else "",
                placeholder="e.g., 2020-01-15",
                id="origination-date"
            )

            yield Label("Maturity Date (YYYY-MM-DD) *")
            yield Input(
                value=self.existing_loan.maturity_date.split('T')[0] if self.existing_loan and self.existing_loan.maturity_date else "",
                placeholder="e.g., 2030-01-15",
                id="maturity-date"
            )

            yield Label("Next Payment Date (YYYY-MM-DD) *")
            yield Input(
                value=self.existing_loan.next_payment_date.split('T')[0] if self.existing_loan and self.existing_loan.next_payment_date else "",
                placeholder="e.g., 2025-12-01",
                id="next-payment-date"
            )

            # Monthly payment
            yield Label("Monthly Payment (ILS) *")
            yield Input(
                value=str(self.existing_loan.monthly_payment) if self.existing_loan else "",
                placeholder="e.g., 4500",
                id="monthly-payment"
            )

            # Payment frequency
            yield Label("Payment Frequency (months) *")
            yield Input(
                value=str(self.existing_loan.payment_frequency_months) if self.existing_loan else "1",
                placeholder="e.g., 1",
                id="payment-frequency"
            )

            # Status
            yield Label("Status *")
            yield Select(
                options=[("ACTIVE", "Active"), ("PAID_OFF", "Paid Off"), ("DEFAULTED", "Defaulted")],
                value=self.existing_loan.status if self.existing_loan else "ACTIVE",
                id="status"
            )

            # Error message
            yield Static("", id="error-message", classes="error")

            # Buttons
            with Horizontal():
                yield Button("Save", id="save-button", variant="primary")
                yield Button("Cancel", id="cancel-button")

    def _parse_form_data(self) -> tuple[Optional[LoanPosition], Optional[str]]:
        """Parse form data and create LoanPosition"""
        try:
            loan_id = self.query_one("#loan-id", Input).value.strip()
            bank_name = self.query_one("#bank-name", Select).value or ""
            account_number = self.query_one("#account-number", Input).value.strip()
            loan_type = self.query_one("#loan-type", Select).value or "SHIR_BASED"
            principal = float(self.query_one("#principal", Input).value or "0")
            original_principal = float(self.query_one("#original-principal", Input).value or "0")
            interest_rate = float(self.query_one("#interest-rate", Input).value or "0")
            spread = float(self.query_one("#spread", Input).value or "0")
            base_cpi = float(self.query_one("#base-cpi", Input).value or "0")
            current_cpi = float(self.query_one("#current-cpi", Input).value or "0")
            origination_date_str = self.query_one("#origination-date", Input).value.strip()
            maturity_date_str = self.query_one("#maturity-date", Input).value.strip()
            next_payment_date_str = self.query_one("#next-payment-date", Input).value.strip()
            monthly_payment = float(self.query_one("#monthly-payment", Input).value or "0")
            payment_frequency = int(self.query_one("#payment-frequency", Input).value or "1")
            status = self.query_one("#status", Select).value or "ACTIVE"

            # Convert dates to ISO 8601 format
            now = datetime.utcnow()
            origination_date = f"{origination_date_str}T00:00:00Z" if origination_date_str else now.isoformat() + "Z"
            maturity_date = f"{maturity_date_str}T00:00:00Z" if maturity_date_str else now.isoformat() + "Z"
            next_payment_date = f"{next_payment_date_str}T00:00:00Z" if next_payment_date_str else now.isoformat() + "Z"

            loan = LoanPosition(
                loan_id=loan_id,
                bank_name=bank_name,
                account_number=account_number,
                loan_type=loan_type,
                principal=principal,
                original_principal=original_principal,
                interest_rate=interest_rate,
                spread=spread,
                base_cpi=base_cpi,
                current_cpi=current_cpi,
                origination_date=origination_date,
                maturity_date=maturity_date,
                next_payment_date=next_payment_date,
                monthly_payment=monthly_payment,
                payment_frequency_months=payment_frequency,
                status=status,
                last_update=now.isoformat() + "Z"
            )

            return loan, None
        except ValueError as e:
            return None, f"Invalid numeric value: {e}"
        except Exception as e:
            return None, f"Error parsing form: {e}"

    def on_button_pressed(self, event: Button.Pressed) -> None:
        """Handle button presses"""
        if event.button.id == "save-button":
            loan, error = self._parse_form_data()
            if error:
                self.query_one("#error-message", Static).update(f"Error: {error}")
                return

            if loan:
                if self.is_editing:
                    success, error = self.loan_manager.update_loan(self.existing_loan.loan_id, loan)
                else:
                    success, error = self.loan_manager.add_loan(loan)

                if success:
                    self.loan_manager.save()
                    self.post_message(self.LoanSaved(loan))
                else:
                    self.query_one("#error-message", Static).update(f"Error: {error}")

        elif event.button.id == "cancel-button":
            self.post_message(self.LoanFormCancelled())


class LoanListTab(Container):
    """Tab showing loan list and management"""

    def __init__(self, loan_manager: LoanManager):
        super().__init__()
        self.loan_manager = loan_manager

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Loan Positions", classes="tab-title")

            with Horizontal():
                yield Button("Add Loan", id="add-loan-button", variant="primary")
                yield Button("Import CSV", id="import-csv-button")
                yield Button("Import JSON", id="import-json-button")
                yield Button("Refresh", id="refresh-button")

            yield DataTable(id="loans-table")
            yield Static("", id="loan-status")
            yield Log(id="loan-log", max_lines=10)

    def on_mount(self) -> None:
        """Initialize on mount"""
        table = self.query_one("#loans-table", DataTable)
        table.add_columns(
            "Loan ID", "Bank", "Type", "Principal (ILS)", "Rate (%)",
            "Monthly Payment", "Status", "Next Payment"
        )
        self._refresh_table()

    def _refresh_table(self) -> None:
        """Refresh the loans table"""
        table = self.query_one("#loans-table", DataTable)
        table.clear()

        loans = self.loan_manager.get_all_loans()
        for loan in loans:
            table.add_row(
                loan.loan_id,
                loan.bank_name,
                loan.loan_type,
                f"{loan.principal:,.0f}",
                f"{loan.interest_rate:.2f}",
                f"{loan.monthly_payment:,.0f}",
                loan.status,
                loan.next_payment_date.split('T')[0] if loan.next_payment_date else ""
            )

        status = self.query_one("#loan-status", Static)
        status.update(f"Total Loans: {len(loans)} | Active: {len([ln for ln in loans if ln.status == 'ACTIVE'])}")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        """Handle button presses"""
        if event.button.id == "add-loan-button":
            # Show loan entry form (would need modal/screen push)
            log = self.query_one("#loan-log", Log)
            log.write("Add loan button pressed - form would open here")

        elif event.button.id == "refresh-button":
            self.loan_manager.load()
            self._refresh_table()
            log = self.query_one("#loan-log", Log)
            log.write("Loans refreshed")

    def on_data_table_row_selected(self, event: DataTable.RowSelected) -> None:
        """Handle row selection"""
        row_key = event.cursor_row
        # Could show loan details or edit form here
        log = self.query_one("#loan-log", Log)
        log.write(f"Selected loan row: {row_key}")


class LoanImporter:
    """Import loans from CSV or JSON files"""

    def __init__(self, loan_manager: LoanManager):
        self.loan_manager = loan_manager

    def import_from_csv(self, file_path: str) -> tuple[int, int, List[str]]:
        """Import loans from CSV file"""
        imported = 0
        errors = []

        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                for row_num, row in enumerate(reader, start=2):  # Start at 2 (header is row 1)
                    try:
                        loan = self._parse_csv_row(row)
                        is_valid, validation_errors = loan.is_valid()
                        if not is_valid:
                            errors.append(f"Row {row_num}: {', '.join(validation_errors)}")
                            continue

                        success, error = self.loan_manager.add_loan(loan)
                        if success:
                            imported += 1
                        else:
                            errors.append(f"Row {row_num}: {error}")
                    except Exception as e:
                        errors.append(f"Row {row_num}: {str(e)}")

            if imported > 0:
                self.loan_manager.save()

            return imported, len(errors), errors
        except Exception as e:
            errors.append(f"Error reading CSV file: {e}")
            return 0, len(errors), errors

    def import_from_json(self, file_path: str) -> tuple[int, int, List[str]]:
        """Import loans from JSON file"""
        imported = 0
        errors = []

        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                data = json.load(f)

            loans_data = data.get('loans', []) if isinstance(data, dict) else data

            for idx, loan_data in enumerate(loans_data, start=1):
                try:
                    loan = LoanPosition.from_dict(loan_data)
                    is_valid, validation_errors = loan.is_valid()
                    if not is_valid:
                        errors.append(f"Loan {idx}: {', '.join(validation_errors)}")
                        continue

                    success, error = self.loan_manager.add_loan(loan)
                    if success:
                        imported += 1
                    else:
                        errors.append(f"Loan {idx}: {error}")
                except Exception as e:
                    errors.append(f"Loan {idx}: {str(e)}")

            if imported > 0:
                self.loan_manager.save()

            return imported, len(errors), errors
        except Exception as e:
            errors.append(f"Error reading JSON file: {e}")
            return 0, len(errors), errors

    def _parse_csv_row(self, row: Dict[str, str]) -> LoanPosition:
        """Parse a CSV row into LoanPosition"""
        now = datetime.utcnow().isoformat() + "Z"

        # Helper to parse dates
        def parse_date(date_str: str) -> str:
            if not date_str:
                return now
            try:
                # Try various date formats
                for fmt in ["%Y-%m-%d", "%m/%d/%Y", "%d/%m/%Y"]:
                    try:
                        dt = datetime.strptime(date_str.strip(), fmt)
                        return dt.isoformat() + "Z"
                    except ValueError:
                        continue
                return now
            except Exception:
                return now

        return LoanPosition(
            loan_id=row.get('loan_id', '').strip(),
            bank_name=row.get('bank_name', '').strip(),
            account_number=row.get('account_number', '').strip(),
            loan_type=row.get('loan_type', 'SHIR_BASED').strip(),
            principal=float(row.get('principal', '0') or '0'),
            original_principal=float(row.get('original_principal', '0') or '0'),
            interest_rate=float(row.get('interest_rate', '0') or '0'),
            spread=float(row.get('spread', '0') or '0'),
            base_cpi=float(row.get('base_cpi', '0') or '0'),
            current_cpi=float(row.get('current_cpi', '0') or '0'),
            origination_date=parse_date(row.get('origination_date', '')),
            maturity_date=parse_date(row.get('maturity_date', '')),
            next_payment_date=parse_date(row.get('next_payment_date', '')),
            monthly_payment=float(row.get('monthly_payment', '0') or '0'),
            payment_frequency_months=int(row.get('payment_frequency_months', '1') or '1'),
            status=row.get('status', 'ACTIVE').strip(),
            last_update=now
        )
