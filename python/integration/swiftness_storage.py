"""
swiftness_storage.py - Storage layer for Swiftness positions
"""
import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Optional, Dict, List

from .swiftness_models import (
    InsuranceCoverage,
    DepositRecord,
    ProductDetails,
    SwiftnessData,
)

logger = logging.getLogger(__name__)


class SwiftnessPositions:
    """Container for all Swiftness positions"""

    def __init__(
        self,
        products: List[ProductDetails] = None,
        deposits: List[DepositRecord] = None,
        insurance_coverage: List[InsuranceCoverage] = None,
        last_updated: Optional[datetime] = None,
    ):
        self.products = products or []
        self.deposits = deposits or []
        self.insurance_coverage = insurance_coverage or []
        self.last_updated = last_updated or datetime.now()

    def to_dict(self) -> Dict:
        """Convert to dictionary for JSON serialization"""
        return {
            "version": "1.0.0",
            "last_updated": self.last_updated.isoformat(),
            "products": [self._product_to_dict(p) for p in self.products],
            "deposits": [self._deposit_to_dict(d) for d in self.deposits],
            "insurance_coverage": [
                self._coverage_to_dict(c) for c in self.insurance_coverage
            ],
        }

    @classmethod
    def from_dict(cls, data: Dict) -> "SwiftnessPositions":
        """Create from dictionary (JSON deserialization)"""
        products = [
            cls._product_from_dict(p) for p in data.get("products", [])
        ]
        deposits = [
            cls._deposit_from_dict(d) for d in data.get("deposits", [])
        ]
        insurance_coverage = [
            cls._coverage_from_dict(c) for c in data.get("insurance_coverage", [])
        ]
        last_updated_str = data.get("last_updated")
        last_updated = (
            datetime.fromisoformat(last_updated_str) if last_updated_str else None
        )

        return cls(
            products=products,
            deposits=deposits,
            insurance_coverage=insurance_coverage,
            last_updated=last_updated,
        )

    @staticmethod
    def _product_to_dict(p: ProductDetails) -> Dict:
        """Convert ProductDetails to dict"""
        return {
            "policy_number": p.policy_number,
            "product_name": p.product_name,
            "company": p.company,
            "status": p.status,
            "total_savings": p.total_savings,
            "next_withdrawal_date": (
                p.next_withdrawal_date.isoformat()
                if p.next_withdrawal_date
                else None
            ),
            "projected_savings_no_premiums": p.projected_savings_no_premiums,
            "monthly_benefit_no_premiums": p.monthly_benefit_no_premiums,
            "projected_savings": p.projected_savings,
            "monthly_benefit": p.monthly_benefit,
            "expected_pension_rate": p.expected_pension_rate,
            "management_fee_on_deposits": p.management_fee_on_deposits,
            "management_fee_annual": p.management_fee_annual,
            "ytd_return": p.ytd_return,
            "employee_deposits": p.employee_deposits,
            "employer_deposits": p.employer_deposits,
            "first_join_date": (
                p.first_join_date.isoformat() if p.first_join_date else None
            ),
            "plan_opening_date": (
                p.plan_opening_date.isoformat() if p.plan_opening_date else None
            ),
            "data_accuracy_date": p.data_accuracy_date.isoformat(),
        }

    @staticmethod
    def _product_from_dict(d: Dict) -> ProductDetails:
        """Create ProductDetails from dict"""
        return ProductDetails(
            product_name=d["product_name"],
            company=d["company"],
            policy_number=d["policy_number"],
            status=d["status"],
            total_savings=d["total_savings"],
            next_withdrawal_date=(
                datetime.fromisoformat(d["next_withdrawal_date"])
                if d.get("next_withdrawal_date")
                else None
            ),
            projected_savings_no_premiums=d["projected_savings_no_premiums"],
            monthly_benefit_no_premiums=d["monthly_benefit_no_premiums"],
            projected_savings=d["projected_savings"],
            monthly_benefit=d["monthly_benefit"],
            expected_pension_rate=d["expected_pension_rate"],
            management_fee_on_deposits=d["management_fee_on_deposits"],
            management_fee_annual=d["management_fee_annual"],
            ytd_return=d["ytd_return"],
            employee_deposits=d["employee_deposits"],
            employer_deposits=d["employer_deposits"],
            first_join_date=(
                datetime.fromisoformat(d["first_join_date"])
                if d.get("first_join_date")
                else None
            ),
            plan_opening_date=(
                datetime.fromisoformat(d["plan_opening_date"])
                if d.get("plan_opening_date")
                else None
            ),
            data_accuracy_date=datetime.fromisoformat(d["data_accuracy_date"]),
        )

    @staticmethod
    def _deposit_to_dict(d: DepositRecord) -> Dict:
        """Convert DepositRecord to dict"""
        return {
            "product_type": d.product_type,
            "company": d.company,
            "policy_number": d.policy_number,
            "value_date": d.value_date.isoformat(),
            "salary_month": d.salary_month.isoformat(),
            "employer_name": d.employer_name,
            "employee_deposit": d.employee_deposit,
            "employer_deposit": d.employer_deposit,
            "employer_severance": d.employer_severance,
        }

    @staticmethod
    def _deposit_from_dict(d: Dict) -> DepositRecord:
        """Create DepositRecord from dict"""
        return DepositRecord(
            product_type=d["product_type"],
            company=d["company"],
            policy_number=d["policy_number"],
            value_date=datetime.fromisoformat(d["value_date"]),
            salary_month=datetime.fromisoformat(d["salary_month"]),
            employer_name=d["employer_name"],
            employee_deposit=d["employee_deposit"],
            employer_deposit=d["employer_deposit"],
            employer_severance=d["employer_severance"],
        )

    @staticmethod
    def _coverage_to_dict(c: InsuranceCoverage) -> Dict:
        """Convert InsuranceCoverage to dict"""
        return {
            "coverage_type": c.coverage_type,
            "plan_name": c.plan_name,
            "company": c.company,
            "payment_recipient": c.payment_recipient,
            "one_time_amount": c.one_time_amount,
            "monthly_benefit": c.monthly_benefit,
            "policy_number": c.policy_number,
            "file_modified_date": c.file_modified_date.isoformat(),
        }

    @staticmethod
    def _coverage_from_dict(d: Dict) -> InsuranceCoverage:
        """Create InsuranceCoverage from dict"""
        return InsuranceCoverage(
            coverage_type=d["coverage_type"],
            plan_name=d["plan_name"],
            company=d["company"],
            payment_recipient=d["payment_recipient"],
            one_time_amount=d["one_time_amount"],
            monthly_benefit=d["monthly_benefit"],
            policy_number=d["policy_number"],
            file_modified_date=datetime.fromisoformat(d["file_modified_date"]),
        )


class SwiftnessStorage:
    """Manages storage and retrieval of Swiftness positions"""

    def __init__(self, storage_path: Path):
        """
        Initialize storage.

        Args:
            storage_path: Directory path for storing positions JSON file
        """
        self.storage_path = Path(storage_path)
        self.storage_path.mkdir(parents=True, exist_ok=True)
        self.positions_file = self.storage_path / "swiftness_positions.json"

    def load_positions(self) -> SwiftnessPositions:
        """
        Load existing positions from JSON file.

        Returns:
            SwiftnessPositions object, or empty positions if file doesn't exist
        """
        if not self.positions_file.exists():
            logger.info(
                f"Positions file not found at {self.positions_file}, returning empty positions"
            )
            return SwiftnessPositions()

        try:
            with open(self.positions_file, "r", encoding="utf-8") as f:
                data = json.load(f)
            return SwiftnessPositions.from_dict(data)
        except Exception as e:
            logger.error(f"Failed to load positions from {self.positions_file}: {e}")
            return SwiftnessPositions()

    def save_positions(self, positions: SwiftnessPositions) -> None:
        """
        Save positions to JSON file.

        Args:
            positions: SwiftnessPositions to save
        """
        try:
            positions.last_updated = datetime.now()
            data = positions.to_dict()

            # Write to temporary file first, then rename (atomic write)
            temp_file = self.positions_file.with_suffix(".json.tmp")
            with open(temp_file, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)

            temp_file.replace(self.positions_file)
            logger.info(f"Saved positions to {self.positions_file}")

        except Exception as e:
            logger.error(f"Failed to save positions to {self.positions_file}: {e}")
            raise

    def get_product_by_policy(
        self, policy_number: str, positions: SwiftnessPositions
    ) -> Optional[ProductDetails]:
        """
        Get product details by policy number.

        Args:
            policy_number: Policy number to search for
            positions: SwiftnessPositions to search in

        Returns:
            ProductDetails if found, None otherwise
        """
        for product in positions.products:
            if product.policy_number == policy_number:
                return product
        return None

    def get_deposits_by_policy(
        self, policy_number: str, positions: SwiftnessPositions
    ) -> List[DepositRecord]:
        """
        Get all deposit records for a policy number.

        Args:
            policy_number: Policy number to search for
            positions: SwiftnessPositions to search in

        Returns:
            List of DepositRecord objects
        """
        return [
            d for d in positions.deposits if d.policy_number == policy_number
        ]
