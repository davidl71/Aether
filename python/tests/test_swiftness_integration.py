#!/usr/bin/env python3
"""
test_swiftness_integration.py - Test script for Swiftness integration with investment strategy framework
"""
import logging
import sys
from pathlib import Path
from datetime import datetime, timedelta

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

from python.integration.swiftness_storage import SwiftnessStorage
from python.integration.swiftness_integration import SwiftnessIntegration


def main():
    """Test Swiftness integration with investment strategy framework"""
    # Storage path
    storage_path = Path.home() / ".config" / "ib_box_spread" / "swiftness"

    try:
        # Initialize components
        storage = SwiftnessStorage(storage_path)
        integration = SwiftnessIntegration(storage, ils_to_usd_rate=0.27)

        # Load positions
        logger.info("Loading Swiftness positions...")
        positions = storage.load_positions()
        logger.info(f"Loaded {len(positions.products)} products, {len(positions.deposits)} deposits")

        # Test position conversion
        logger.info("=" * 80)
        logger.info("POSITION CONVERSION TEST")
        logger.info("=" * 80)
        position_snapshots = integration.get_positions(check_validity=True, max_age_days=90)
        logger.info(f"Converted {len(position_snapshots)} positions to PositionSnapshot format")
        for snapshot in position_snapshots[:5]:  # Show first 5
            logger.info(
                f"  {snapshot.symbol}: quantity={snapshot.quantity}, "
                f"cost_basis=${snapshot.cost_basis:.2f}, mark=${snapshot.mark:.2f}"
            )

        # Test Greeks calculation
        logger.info("=" * 80)
        logger.info("GREEKS CALCULATION TEST")
        logger.info("=" * 80)
        greeks = integration.get_greeks(check_validity=True, max_age_days=90)
        logger.info(f"Calculated Greeks for {len(greeks)} positions")
        for symbol, greek_values in list(greeks.items())[:3]:  # Show first 3
            logger.info(f"  {symbol}: delta={greek_values['delta']}, gamma={greek_values['gamma']}")

        # Test cash flow forecasting
        logger.info("=" * 80)
        logger.info("CASH FLOW FORECASTING TEST")
        logger.info("=" * 80)
        start_date = datetime.now()
        end_date = start_date + timedelta(days=365)
        cash_flows = integration.get_cash_flows(
            start_date=start_date,
            end_date=end_date,
            check_validity=True,
            max_age_days=90
        )
        logger.info(f"Generated {len(cash_flows)} cash flow events")
        for cf in cash_flows[:5]:  # Show first 5
            logger.info(
                f"  {cf.date.date()}: ${cf.amount:.2f} ({cf.currency}) - {cf.description[:50]}"
            )

        # Test portfolio value
        logger.info("=" * 80)
        logger.info("PORTFOLIO VALUE TEST")
        logger.info("=" * 80)
        total_value = integration.get_portfolio_value(check_validity=True, max_age_days=90)
        logger.info(f"Total Swiftness portfolio value: ${total_value:,.2f} USD")

        # Test validation
        logger.info("=" * 80)
        logger.info("VALIDATION TEST")
        logger.info("=" * 80)
        validation_report = integration.validate_positions()
        logger.info(f"Total products: {validation_report['total_products']}")
        logger.info(f"Valid products: {len(validation_report['valid_products'])}")
        logger.info(f"Stale products: {len(validation_report['stale_products'])}")

        logger.info("=" * 80)
        logger.info("SUCCESS: Integration tests completed!")
        logger.info("=" * 80)

        return 0

    except Exception as e:
        logger.error(f"Integration test failed: {e}", exc_info=True)
        return 1


if __name__ == "__main__":
    sys.exit(main())
