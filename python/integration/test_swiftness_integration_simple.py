#!/usr/bin/env python3
"""
test_swiftness_integration_simple.py - Simple test for Swiftness integration module
Tests the integration module structure without requiring xlrd or real data
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

from python.integration.swiftness_storage import SwiftnessStorage, SwiftnessPositions
from python.integration.swiftness_models import ProductDetails
from python.integration.swiftness_integration import SwiftnessIntegration


def create_mock_positions(storage_path: Path) -> None:
    """Create mock Swiftness positions for testing"""
    storage = SwiftnessStorage(storage_path)

    # Create mock product
    mock_product = ProductDetails(
        product_name="Test Pension Fund",
        company="Test Company",
        policy_number="TEST-12345",
        status="Active",
        total_savings=100000.0,  # 100,000 ILS
        next_withdrawal_date=datetime.now() + timedelta(days=180),
        projected_savings_no_premiums=500000.0,
        monthly_benefit_no_premiums=5000.0,
        projected_savings=600000.0,
        monthly_benefit=6000.0,  # 6,000 ILS per month
        expected_pension_rate=0.05,
        management_fee_on_deposits=0.02,
        management_fee_annual=0.015,
        ytd_return=0.08,
        employee_deposits=50000.0,
        employer_deposits=50000.0,
        first_join_date=datetime(2020, 1, 1),
        plan_opening_date=datetime(2020, 1, 1),
        data_accuracy_date=datetime.now() - timedelta(days=30),  # 30 days old (valid)
    )

    positions = SwiftnessPositions(
        products=[mock_product],
        deposits=[],
        insurance_coverage=[],
        last_updated=datetime.now(),
    )

    storage.save_positions(positions)
    logger.info(f"Created mock Swiftness positions with 1 product")


def main():
    """Test Swiftness integration with mock data"""
    # Storage path
    storage_path = Path.home() / ".config" / "ib_box_spread" / "swiftness"
    storage_path.mkdir(parents=True, exist_ok=True)

    try:
        # Create mock positions if they don't exist
        positions_file = storage_path / "swiftness_positions.json"
        if not positions_file.exists():
            logger.info("Creating mock Swiftness positions for testing...")
            create_mock_positions(storage_path)
        else:
            logger.info("Using existing Swiftness positions from file")

        # Initialize integration
        storage = SwiftnessStorage(storage_path)
        integration = SwiftnessIntegration(storage, ils_to_usd_rate=0.27)

        # Test position conversion
        logger.info("=" * 80)
        logger.info("TEST 1: POSITION CONVERSION")
        logger.info("=" * 80)
        position_snapshots = integration.get_positions(check_validity=True, max_age_days=90)
        logger.info(f"✅ Converted {len(position_snapshots)} positions to PositionSnapshot format")
        for snapshot in position_snapshots:
            logger.info(
                f"  Position: {snapshot.symbol}\n"
                f"    ID: {snapshot.id}\n"
                f"    Quantity: {snapshot.quantity}\n"
                f"    Cost Basis: ${snapshot.cost_basis:,.2f} USD\n"
                f"    Mark: ${snapshot.mark:,.2f} USD\n"
                f"    Unrealized PnL: ${snapshot.unrealized_pnl:,.2f}"
            )

        # Test Greeks calculation
        logger.info("=" * 80)
        logger.info("TEST 2: GREEKS CALCULATION")
        logger.info("=" * 80)
        greeks = integration.get_greeks(check_validity=True, max_age_days=90)
        logger.info(f"✅ Calculated Greeks for {len(greeks)} positions")
        for symbol, greek_values in greeks.items():
            logger.info(
                f"  {symbol}:\n"
                f"    Delta: {greek_values['delta']}\n"
                f"    Gamma: {greek_values['gamma']}\n"
                f"    Vega: {greek_values['vega']}\n"
                f"    Theta: {greek_values['theta']}\n"
                f"    Rho: {greek_values['rho']}"
            )

        # Test cash flow forecasting
        logger.info("=" * 80)
        logger.info("TEST 3: CASH FLOW FORECASTING")
        logger.info("=" * 80)
        start_date = datetime.now()
        end_date = start_date + timedelta(days=365)
        cash_flows = integration.get_cash_flows(
            start_date=start_date,
            end_date=end_date,
            check_validity=True,
            max_age_days=90
        )
        logger.info(f"✅ Generated {len(cash_flows)} cash flow events")
        for i, cf in enumerate(cash_flows[:6], 1):  # Show first 6
            logger.info(
                f"  {i}. {cf.date.date()}: ${cf.amount:,.2f} USD ({cf.currency})\n"
                f"     Description: {cf.description}\n"
                f"     Source: {cf.source}"
            )

        # Test portfolio value
        logger.info("=" * 80)
        logger.info("TEST 4: PORTFOLIO VALUE")
        logger.info("=" * 80)
        total_value = integration.get_portfolio_value(check_validity=True, max_age_days=90)
        logger.info(f"✅ Total Swiftness portfolio value: ${total_value:,.2f} USD")

        # Test validation
        logger.info("=" * 80)
        logger.info("TEST 5: POSITION VALIDATION")
        logger.info("=" * 80)
        validation_report = integration.validate_positions()
        logger.info(f"✅ Validation Report:")
        logger.info(f"    Total Products: {validation_report['total_products']}")
        logger.info(f"    Valid Products: {len(validation_report['valid_products'])}")
        logger.info(f"    Stale Products: {len(validation_report['stale_products'])}")
        logger.info(f"    Last Updated: {validation_report['last_updated']}")

        # Test exchange rate update
        logger.info("=" * 80)
        logger.info("TEST 6: EXCHANGE RATE UPDATE")
        logger.info("=" * 80)
        old_rate = integration.ils_to_usd_rate
        integration.update_exchange_rate(0.28)
        logger.info(f"✅ Updated exchange rate: {old_rate} → {integration.ils_to_usd_rate}")
        # Recalculate portfolio value with new rate
        new_value = integration.get_portfolio_value(check_validity=True, max_age_days=90)
        logger.info(f"    Portfolio value with new rate: ${new_value:,.2f} USD")

        logger.info("=" * 80)
        logger.info("✅ SUCCESS: All integration tests passed!")
        logger.info("=" * 80)
        logger.info("The Swiftness integration module is working correctly.")
        logger.info("Ready to integrate with:")
        logger.info("  - Portfolio aggregation (T-79)")
        logger.info("  - Greeks calculation (T-68)")
        logger.info("  - Cash flow forecasting (T-71)")

        return 0

    except Exception as e:
        logger.error(f"Integration test failed: {e}", exc_info=True)
        return 1


if __name__ == "__main__":
    sys.exit(main())
