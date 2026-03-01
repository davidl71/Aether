#!/usr/bin/env python3
"""
swiftness_integration_manual.py - Manual script for Swiftness integration checks.

Run from repo root: python scripts/swiftness_integration_manual.py

Uses mock data if ~/.config/ib_box_spread/swiftness/swiftness_positions.json
does not exist. For real-data run, place Swiftness export there first.
"""
import logging
import sys
from pathlib import Path
from datetime import datetime, timedelta

logging.basicConfig(
    level=logging.INFO,
    format="[%(asctime)s] [%(levelname)s] [%(name)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)
logger = logging.getLogger(__name__)

project_root = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(project_root))

from python.integration.swiftness_storage import SwiftnessStorage, SwiftnessPositions
from python.integration.swiftness_models import ProductDetails
from python.integration.swiftness_integration import SwiftnessIntegration


def create_mock_positions(storage_path: Path) -> None:
    """Create mock Swiftness positions for manual testing."""
    storage = SwiftnessStorage(storage_path)
    mock_product = ProductDetails(
        product_name="Test Pension Fund",
        company="Test Company",
        policy_number="TEST-12345",
        status="Active",
        total_savings=100000.0,
        next_withdrawal_date=datetime.now() + timedelta(days=180),
        projected_savings_no_premiums=500000.0,
        monthly_benefit_no_premiums=5000.0,
        projected_savings=600000.0,
        monthly_benefit=6000.0,
        expected_pension_rate=0.05,
        management_fee_on_deposits=0.02,
        management_fee_annual=0.015,
        ytd_return=0.08,
        employee_deposits=50000.0,
        employer_deposits=50000.0,
        first_join_date=datetime(2020, 1, 1),
        plan_opening_date=datetime(2020, 1, 1),
        data_accuracy_date=datetime.now() - timedelta(days=30),
    )
    positions = SwiftnessPositions(
        products=[mock_product],
        deposits=[],
        insurance_coverage=[],
        last_updated=datetime.now(),
    )
    storage.save_positions(positions)
    logger.info("Created mock Swiftness positions with 1 product")


def main() -> int:
    storage_path = Path.home() / ".config" / "ib_box_spread" / "swiftness"
    storage_path.mkdir(parents=True, exist_ok=True)

    positions_file = storage_path / "swiftness_positions.json"
    if not positions_file.exists():
        logger.info("No Swiftness data found; creating mock positions...")
        create_mock_positions(storage_path)
    else:
        logger.info("Using existing Swiftness positions from file")

    try:
        storage = SwiftnessStorage(storage_path)
        integration = SwiftnessIntegration(storage, ils_to_usd_rate=0.27)

        logger.info("=" * 80)
        logger.info("POSITION CONVERSION")
        logger.info("=" * 80)
        snapshots = integration.get_positions(check_validity=True, max_age_days=90)
        logger.info("Converted %d positions to PositionSnapshot format", len(snapshots))
        for s in snapshots[:5]:
            logger.info("  %s: qty=%s, cost_basis=$%.2f, mark=$%.2f", s.symbol, s.quantity, s.cost_basis, s.mark)

        logger.info("=" * 80)
        logger.info("GREEKS")
        logger.info("=" * 80)
        greeks = integration.get_greeks(check_validity=True, max_age_days=90)
        logger.info("Greeks for %d positions", len(greeks))
        for sym, g in list(greeks.items())[:3]:
            logger.info("  %s: delta=%s, gamma=%s", sym, g.get("delta"), g.get("gamma"))

        logger.info("=" * 80)
        logger.info("CASH FLOWS (1Y)")
        logger.info("=" * 80)
        start = datetime.now()
        end = start + timedelta(days=365)
        cfs = integration.get_cash_flows(start_date=start, end_date=end, check_validity=True, max_age_days=90)
        logger.info("Generated %d cash flow events", len(cfs))
        for cf in cfs[:5]:
            logger.info("  %s: $%.2f %s - %s", cf.date.date(), cf.amount, cf.currency, cf.description[:50])

        logger.info("=" * 80)
        logger.info("PORTFOLIO VALUE")
        logger.info("=" * 80)
        total = integration.get_portfolio_value(check_validity=True, max_age_days=90)
        logger.info("Total Swiftness portfolio value: $%.2f USD", total)

        logger.info("=" * 80)
        logger.info("SUCCESS: Swiftness integration check completed")
        logger.info("=" * 80)
        return 0
    except Exception as e:
        logger.exception("Swiftness integration check failed: %s", e)
        return 1


if __name__ == "__main__":
    sys.exit(main())
