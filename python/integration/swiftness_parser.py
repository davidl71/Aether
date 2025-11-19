"""
swiftness_parser.py - Parser for Swiftness (Israeli Pension Clearing House) Excel files
"""
import logging
import xlrd
from datetime import datetime
from pathlib import Path
from typing import List, Optional

from .swiftness_models import (
    InsuranceCoverage,
    DepositRecord,
    ProductDetails,
    SwiftnessData,
)

logger = logging.getLogger(__name__)

# Cell type constants
XL_CELL_EMPTY = 0
XL_CELL_TEXT = 1
XL_CELL_NUMBER = 2
XL_CELL_DATE = 3


class SwiftnessParser:
    """Parses Swiftness Excel files and extracts position data."""

    def __init__(self, file_path: Path):
        """
        Initialize parser with Excel file path.

        Args:
            file_path: Path to Swiftness Excel file (.xls format)
        """
        self.file_path = Path(file_path)
        if not self.file_path.exists():
            raise FileNotFoundError(f"Swiftness file not found: {file_path}")
        self.workbook: Optional[xlrd.Book] = None
        self.file_modified_date = datetime.fromtimestamp(self.file_path.stat().st_mtime)

    def parse(self) -> SwiftnessData:
        """
        Parse all sheets and return structured data.

        Returns:
            SwiftnessData containing all parsed data from all sheets

        Raises:
            ValueError: If file format is invalid or sheets are missing
        """
        try:
            # Open workbook with Hebrew encoding
            self.workbook = xlrd.open_workbook(
                str(self.file_path),
                encoding_override='cp1255'
            )

            if self.workbook.nsheets < 3:
                raise ValueError(
                    f"Expected at least 3 sheets, found {self.workbook.nsheets}"
                )

            # Parse each sheet
            insurance_coverage = self._parse_insurance_coverage(
                self.workbook.sheet_by_index(0)
            )
            deposits = self._parse_deposit_tracking(
                self.workbook.sheet_by_index(1)
            )
            products = self._parse_product_details(
                self.workbook.sheet_by_index(2)
            )

            return SwiftnessData(
                insurance_coverage=insurance_coverage,
                deposits=deposits,
                products=products,
                file_path=str(self.file_path),
                file_modified_date=self.file_modified_date,
            )

        except Exception as e:
            logger.error(f"Failed to parse Swiftness file {self.file_path}: {e}")
            raise

    def _parse_insurance_coverage(self, sheet) -> List[InsuranceCoverage]:
        """Parse Sheet 1: Insurance Coverage (כיסויים ביטוחיים)"""
        coverage_list = []

        # Skip header row (row 0)
        for row_idx in range(1, sheet.nrows):
            try:
                row = sheet.row(row_idx)
                if not any(cell.value for cell in row[:7]):  # Skip empty rows
                    continue

                coverage = InsuranceCoverage(
                    coverage_type=self._get_cell_value(row[0], str),
                    plan_name=self._get_cell_value(row[1], str),
                    company=self._get_cell_value(row[2], str),
                    payment_recipient=self._get_cell_value(row[3], str),
                    one_time_amount=self._get_cell_value(row[4], float, 0.0),
                    monthly_benefit=self._get_cell_value(row[5], float, 0.0),
                    policy_number=str(self._get_cell_value(row[6], str)).strip(),
                    file_modified_date=self.file_modified_date,
                )

                # Only add if policy number exists
                if coverage.policy_number:
                    coverage_list.append(coverage)

            except Exception as e:
                logger.warning(
                    f"Failed to parse insurance coverage row {row_idx}: {e}"
                )
                continue

        return coverage_list

    def _parse_deposit_tracking(self, sheet) -> List[DepositRecord]:
        """Parse Sheet 2: Deposit Tracking (מעקב הפקדות)"""
        deposits = []

        # Skip header row (row 0)
        for row_idx in range(1, sheet.nrows):
            try:
                row = sheet.row(row_idx)
                if not any(cell.value for cell in row[:9]):  # Skip empty rows
                    continue

                # Parse value date (Column 4) - VALIDITY DATE
                value_date_str = self._get_cell_value(row[3], str)
                value_date = self._parse_date(value_date_str) if value_date_str else None

                # Parse salary month (Column 5)
                salary_month_str = self._get_cell_value(row[4], str)
                salary_month = (
                    self._parse_date(salary_month_str) if salary_month_str else None
                )

                if not value_date:  # Skip if no validity date
                    logger.warning(
                        f"Deposit record row {row_idx} missing value date, skipping"
                    )
                    continue

                deposit = DepositRecord(
                    product_type=self._get_cell_value(row[0], str),
                    company=self._get_cell_value(row[1], str),
                    policy_number=str(self._get_cell_value(row[2], str)).strip(),
                    value_date=value_date,
                    salary_month=salary_month or value_date,  # Fallback to value_date
                    employer_name=self._get_cell_value(row[5], str),
                    employee_deposit=self._get_cell_value(row[6], float, 0.0),
                    employer_deposit=self._get_cell_value(row[7], float, 0.0),
                    employer_severance=self._get_cell_value(row[8], float, 0.0),
                )

                # Only add if policy number exists
                if deposit.policy_number:
                    deposits.append(deposit)

            except Exception as e:
                logger.warning(f"Failed to parse deposit record row {row_idx}: {e}")
                continue

        return deposits

    def _parse_product_details(self, sheet) -> List[ProductDetails]:
        """Parse Sheet 3: Product Details (פרטי המוצרים שלי)"""
        products = []

        # Skip header row (row 0)
        for row_idx in range(1, sheet.nrows):
            try:
                row = sheet.row(row_idx)
                if not any(cell.value for cell in row[:30]):  # Skip empty rows
                    continue

                # Parse data accuracy date (Column 30) - VALIDITY DATE
                accuracy_date_str = self._get_cell_value(row[29], str)
                accuracy_date = (
                    self._parse_date(accuracy_date_str) if accuracy_date_str else None
                )

                if not accuracy_date:  # Skip if no validity date
                    logger.warning(
                        f"Product row {row_idx} missing data accuracy date, skipping"
                    )
                    continue

                # Parse optional dates
                next_withdrawal = self._parse_date(
                    self._get_cell_value(row[5], str)
                )
                first_join = self._parse_date(
                    self._get_cell_value(row[22], str)
                )
                plan_opening = self._parse_date(
                    self._get_cell_value(row[28], str)
                )

                product = ProductDetails(
                    product_name=self._get_cell_value(row[0], str),
                    company=self._get_cell_value(row[1], str),
                    policy_number=str(self._get_cell_value(row[2], str)).strip(),
                    status=self._get_cell_value(row[3], str),
                    total_savings=self._get_cell_value(row[4], float, 0.0),
                    next_withdrawal_date=next_withdrawal,
                    projected_savings_no_premiums=self._get_cell_value(
                        row[6], float, 0.0
                    ),
                    monthly_benefit_no_premiums=self._get_cell_value(
                        row[7], float, 0.0
                    ),
                    projected_savings=self._get_cell_value(row[8], float, 0.0),
                    monthly_benefit=self._get_cell_value(row[9], float, 0.0),
                    expected_pension_rate=self._get_cell_value(row[10], float, 0.0),
                    management_fee_on_deposits=self._get_cell_value(
                        row[11], float, 0.0
                    ),
                    management_fee_annual=self._get_cell_value(row[12], float, 0.0),
                    ytd_return=self._get_cell_value(row[13], float, 0.0),
                    employee_deposits=self._get_cell_value(row[14], float, 0.0),
                    employer_deposits=self._get_cell_value(row[15], float, 0.0),
                    first_join_date=first_join,
                    plan_opening_date=plan_opening,
                    data_accuracy_date=accuracy_date,
                )

                # Only add if policy number exists
                if product.policy_number:
                    products.append(product)

            except Exception as e:
                logger.warning(f"Failed to parse product row {row_idx}: {e}")
                continue

        return products

    def _parse_date(self, date_str: str) -> Optional[datetime]:
        """
        Parse M/D/YYYY date format from text.

        Args:
            date_str: Date string in M/D/YYYY format

        Returns:
            Parsed datetime or None if invalid
        """
        if not date_str or not isinstance(date_str, str):
            return None

        date_str = date_str.strip()
        if not date_str:
            return None

        try:
            # Try M/D/YYYY format
            return datetime.strptime(date_str, "%m/%d/%Y")
        except ValueError:
            try:
                # Try M/D/YY format (2-digit year)
                dt = datetime.strptime(date_str, "%m/%d/%y")
                # Adjust 2-digit years: assume 00-30 = 2000-2030, 31-99 = 1931-1999
                if dt.year > 2030:
                    dt = dt.replace(year=dt.year - 100)
                return dt
            except ValueError:
                logger.warning(f"Failed to parse date: {date_str}")
                return None

    def _get_cell_value(self, cell, target_type, default=None):
        """
        Extract cell value and convert to target type.

        Args:
            cell: xlrd cell object
            target_type: Target type (str, float, int)
            default: Default value if conversion fails

        Returns:
            Converted value or default
        """
        if cell.ctype == XL_CELL_EMPTY:
            return default

        value = cell.value

        if target_type == str:
            return str(value) if value is not None else ""
        elif target_type == float:
            try:
                return float(value) if value is not None else (default or 0.0)
            except (ValueError, TypeError):
                return default or 0.0
        elif target_type == int:
            try:
                return int(value) if value is not None else (default or 0)
            except (ValueError, TypeError):
                return default or 0
        else:
            return value if value is not None else default
