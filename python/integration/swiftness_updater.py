"""
swiftness_updater.py - Update logic for Swiftness positions with validity date tracking
"""
import logging
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import List, Optional

from .swiftness_parser import SwiftnessParser
from .swiftness_storage import SwiftnessStorage, SwiftnessPositions
from .swiftness_models import (
    InsuranceCoverage,
    DepositRecord,
    ProductDetails,
)

logger = logging.getLogger(__name__)


@dataclass
class ImportResult:
    """Result of importing a Swiftness file"""

    products_added: int = 0
    products_updated: int = 0
    products_skipped: int = 0
    deposits_added: int = 0
    deposits_skipped: int = 0
    coverage_added: int = 0
    coverage_updated: int = 0
    coverage_skipped: int = 0
    errors: List[str] = None

    def __post_init__(self):
        if self.errors is None:
            self.errors = []

    def total_changes(self) -> int:
        """Total number of changes made"""
        return (
            self.products_added
            + self.products_updated
            + self.deposits_added
            + self.coverage_added
            + self.coverage_updated
        )


class SwiftnessUpdater:
    """Updates Swiftness positions respecting validity dates"""

    def __init__(self, storage: SwiftnessStorage):
        """
        Initialize updater.

        Args:
            storage: SwiftnessStorage instance for persistence
        """
        self.storage = storage

    def import_file(self, file_path: Path) -> ImportResult:
        """
        Import new Excel file and update positions.

        Args:
            file_path: Path to Swiftness Excel file

        Returns:
            ImportResult with statistics about the import
        """
        result = ImportResult()

        try:
            # Parse file
            logger.info(f"Parsing Swiftness file: {file_path}")
            parser = SwiftnessParser(file_path)
            new_data = parser.parse()

            # Load existing positions
            existing_positions = self.storage.load_positions()

            # Update positions
            self._update_products(
                existing_positions, new_data.products, result
            )
            self._update_deposits(
                existing_positions, new_data.deposits, result
            )
            self._update_coverage(
                existing_positions,
                new_data.insurance_coverage,
                new_data.file_modified_date,
                result,
            )

            # Save updated positions
            if result.total_changes() > 0:
                self.storage.save_positions(existing_positions)
                logger.info(
                    f"Import complete: {result.total_changes()} changes made"
                )
            else:
                logger.info("Import complete: No changes needed")

            return result

        except Exception as e:
            error_msg = f"Failed to import file {file_path}: {e}"
            logger.error(error_msg)
            result.errors.append(error_msg)
            return result

    def _update_products(
        self,
        positions: SwiftnessPositions,
        new_products: List[ProductDetails],
        result: ImportResult,
    ) -> None:
        """Update product details based on validity dates"""
        for new_product in new_products:
            existing = self.storage.get_product_by_policy(
                new_product.policy_number, positions
            )

            if existing is None:
                # New product - add it
                positions.products.append(new_product)
                result.products_added += 1
                logger.debug(
                    f"Added new product: {new_product.policy_number} "
                    f"({new_product.product_name})"
                )
            elif self._should_update_product(existing, new_product):
                # Update existing product (newer validity date)
                idx = positions.products.index(existing)
                positions.products[idx] = new_product
                result.products_updated += 1
                logger.debug(
                    f"Updated product: {new_product.policy_number} "
                    f"(validity: {existing.data_accuracy_date} -> "
                    f"{new_product.data_accuracy_date})"
                )
            else:
                # Skip (existing data is newer or same)
                result.products_skipped += 1
                logger.debug(
                    f"Skipped product: {new_product.policy_number} "
                    f"(existing validity: {existing.data_accuracy_date} >= "
                    f"new: {new_product.data_accuracy_date})"
                )

    def _update_deposits(
        self,
        positions: SwiftnessPositions,
        new_deposits: List[DepositRecord],
        result: ImportResult,
    ) -> None:
        """Add new deposit records (deposits are additive/historical)"""
        existing_deposits = {
            (d.policy_number, d.value_date): d for d in positions.deposits
        }

        for new_deposit in new_deposits:
            key = (new_deposit.policy_number, new_deposit.value_date)

            if key not in existing_deposits:
                # New deposit record - add it
                positions.deposits.append(new_deposit)
                result.deposits_added += 1
                logger.debug(
                    f"Added deposit: {new_deposit.policy_number} "
                    f"value_date: {new_deposit.value_date}"
                )
            else:
                # Duplicate deposit (same policy + value_date) - skip
                result.deposits_skipped += 1
                logger.debug(
                    f"Skipped duplicate deposit: {new_deposit.policy_number} "
                    f"value_date: {new_deposit.value_date}"
                )

    def _update_coverage(
        self,
        positions: SwiftnessPositions,
        new_coverage: List[InsuranceCoverage],
        file_modified_date: datetime,
        result: ImportResult,
    ) -> None:
        """Update insurance coverage (use file mod date for comparison)"""
        existing_coverage = {
            c.policy_number: c for c in positions.insurance_coverage
        }

        for new_cov in new_coverage:
            existing = existing_coverage.get(new_cov.policy_number)

            if existing is None:
                # New coverage - add it
                positions.insurance_coverage.append(new_cov)
                result.coverage_added += 1
                logger.debug(
                    f"Added coverage: {new_cov.policy_number} "
                    f"({new_cov.plan_name})"
                )
            elif file_modified_date > existing.file_modified_date:
                # Update (newer file)
                idx = positions.insurance_coverage.index(existing)
                positions.insurance_coverage[idx] = new_cov
                result.coverage_updated += 1
                logger.debug(
                    f"Updated coverage: {new_cov.policy_number} "
                    f"(file date: {existing.file_modified_date} -> "
                    f"{file_modified_date})"
                )
            else:
                # Skip (existing file is newer or same)
                result.coverage_skipped += 1
                logger.debug(
                    f"Skipped coverage: {new_cov.policy_number} "
                    f"(existing file date: {existing.file_modified_date} >= "
                    f"new: {file_modified_date})"
                )

    def _should_update_product(
        self, existing: ProductDetails, new: ProductDetails
    ) -> bool:
        """
        Check if product should be updated based on validity date.

        Args:
            existing: Existing product details
            new: New product details

        Returns:
            True if new data has later validity date
        """
        return new.data_accuracy_date > existing.data_accuracy_date

    def get_positions(self) -> SwiftnessPositions:
        """
        Get current positions.

        Returns:
            SwiftnessPositions from storage
        """
        return self.storage.load_positions()
