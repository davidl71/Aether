#!/usr/bin/env python3
"""
Manual script: Swiftness import from Excel (not a pytest test).

Run from repo root: python python/tests/test_swiftness_import.py
Update file_path in main() to point to your Swiftness Excel export.
"""
import logging
import sys
from pathlib import Path

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='[%(asctime)s] [%(levelname)s] [%(name)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)

# Add project root to path for imports
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.integration.swiftness_parser import SwiftnessParser
from python.integration.swiftness_storage import SwiftnessStorage
from python.integration.swiftness_updater import SwiftnessUpdater


def main():
    """Test Swiftness import system"""
    # Example file path (user should update)
    file_path = Path.home() / "Downloads" / "התמונה המלאה.xls"

    if not file_path.exists():
        logger.error(f"File not found: {file_path}")
        logger.info("Please update file_path in script to point to your Swiftness Excel file")
        return 1

    # Storage path
    storage_path = Path.home() / ".config" / "ib_box_spread" / "swiftness"

    try:
        # Initialize components
        storage = SwiftnessStorage(storage_path)
        updater = SwiftnessUpdater(storage)

        # Import file
        logger.info(f"Importing Swiftness file: {file_path}")
        result = updater.import_file(file_path)

        # Print results
        logger.info("=" * 80)
        logger.info("IMPORT RESULTS")
        logger.info("=" * 80)
        logger.info(f"Products - Added: {result.products_added}, "
                   f"Updated: {result.products_updated}, "
                   f"Skipped: {result.products_skipped}")
        logger.info(f"Deposits - Added: {result.deposits_added}, "
                   f"Skipped: {result.deposits_skipped}")
        logger.info(f"Coverage - Added: {result.coverage_added}, "
                   f"Updated: {result.coverage_updated}, "
                   f"Skipped: {result.coverage_skipped}")
        logger.info(f"Total Changes: {result.total_changes()}")

        if result.errors:
            logger.warning(f"Errors: {result.errors}")

        # Load and display positions
        positions = updater.get_positions()
        logger.info("=" * 80)
        logger.info(f"STORED POSITIONS")
        logger.info("=" * 80)
        logger.info(f"Products: {len(positions.products)}")
        logger.info(f"Deposits: {len(positions.deposits)}")
        logger.info(f"Insurance Coverage: {len(positions.insurance_coverage)}")
        logger.info(f"Last Updated: {positions.last_updated}")

        logger.info("=" * 80)
        logger.info("SUCCESS: Import completed successfully!")
        logger.info(f"Positions saved to: {storage_path / 'swiftness_positions.json'}")

        return 0

    except Exception as e:
        logger.error(f"Import failed: {e}", exc_info=True)
        return 1


if __name__ == "__main__":
    sys.exit(main())
