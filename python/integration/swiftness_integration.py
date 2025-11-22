"""
swiftness_integration.py - Integration layer for Swiftness positions with investment strategy framework
"""
import logging
from datetime import datetime, timedelta
from typing import List, Optional, Dict
from dataclasses import dataclass

# Month increment helper
def add_months(date: datetime, months: int) -> datetime:
    """Add months to a date, handling year rollover"""
    month = date.month - 1 + months
    year = date.year + month // 12
    month = month % 12 + 1
    day = min(date.day, [31, 29 if year % 4 == 0 and (year % 100 != 0 or year % 400 == 0) else 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month - 1])
    return date.replace(year=year, month=month, day=day)

from .swiftness_storage import SwiftnessStorage, SwiftnessPositions
from .swiftness_models import ProductDetails, DepositRecord, InsuranceCoverage

logger = logging.getLogger(__name__)


@dataclass
class PositionSnapshot:
    """Position snapshot compatible with SystemSnapshot.positions format"""
    id: str
    symbol: str
    quantity: int
    cost_basis: float
    mark: float
    unrealized_pnl: float


@dataclass
class CashFlowEvent:
    """Cash flow event for forecasting"""
    date: datetime
    amount: float  # USD
    currency: str  # Original currency (ILS converted to USD)
    description: str
    source: str  # "swiftness_product", "swiftness_deposit", etc.


class SwiftnessIntegration:
    """Integrates Swiftness positions with investment strategy framework"""

    def __init__(
        self,
        storage: SwiftnessStorage,
        ils_to_usd_rate: float = 0.27,  # Default rate, should be updated from API
    ):
        """
        Initialize Swiftness integration.

        Args:
            storage: SwiftnessStorage instance for loading positions
            ils_to_usd_rate: ILS to USD exchange rate (default: 0.27, should be fetched from API)
        """
        self.storage = storage
        self.ils_to_usd_rate = ils_to_usd_rate

    def update_exchange_rate(self, rate: float) -> None:
        """
        Update ILS to USD exchange rate.

        Args:
            rate: Current ILS to USD exchange rate
        """
        if rate <= 0:
            logger.warning(f"Invalid exchange rate: {rate}, keeping current rate: {self.ils_to_usd_rate}")
            return
        self.ils_to_usd_rate = rate
        logger.info(f"Updated ILS/USD exchange rate: {rate}")

    def get_positions(
        self,
        check_validity: bool = True,
        max_age_days: int = 90,
    ) -> List[PositionSnapshot]:
        """
        Get Swiftness positions as PositionSnapshot format for portfolio aggregation.

        Args:
            check_validity: If True, only return positions with valid data_accuracy_date
            max_age_days: Maximum age of data_accuracy_date in days (default: 90)

        Returns:
            List of PositionSnapshot objects
        """
        positions = self.storage.load_positions()
        snapshots = []

        for product in positions.products:
            # Check validity date if requested
            if check_validity:
                age_days = (datetime.now() - product.data_accuracy_date).days
                if age_days > max_age_days:
                    logger.warning(
                        f"Skipping stale Swiftness position: {product.policy_number} "
                        f"(data_accuracy_date: {product.data_accuracy_date}, age: {age_days} days)"
                    )
                    continue

            # Convert ILS to USD
            total_savings_usd = product.total_savings * self.ils_to_usd_rate

            # Create PositionSnapshot
            # Symbol: Use policy number as identifier (format: SWIFTNESS-{policy_number})
            symbol = f"SWIFTNESS-{product.policy_number}"
            position_id = f"SWIFTNESS-POS-{product.policy_number}"

            snapshot = PositionSnapshot(
                id=position_id,
                symbol=symbol,
                quantity=1,  # Pension fund position is a single unit
                cost_basis=total_savings_usd,  # Current value in USD
                mark=total_savings_usd,  # Pension funds are cash-like, mark = cost basis
                unrealized_pnl=0.0,  # No unrealized PnL for cash-like positions
            )

            snapshots.append(snapshot)
            logger.debug(
                f"Converted Swiftness position: {product.policy_number} "
                f"({product.total_savings} ILS = {total_savings_usd:.2f} USD)"
            )

        logger.info(f"Converted {len(snapshots)} Swiftness positions to PositionSnapshot format")
        return snapshots

    def get_greeks(
        self,
        check_validity: bool = True,
        max_age_days: int = 90,
    ) -> Dict[str, Dict[str, float]]:
        """
        Get Greeks for Swiftness positions.

        Swiftness positions are cash-like (pension funds), so:
        - Delta: 1.0 (price sensitivity = 1.0 for cash)
        - Gamma: 0.0 (no convexity)
        - Vega: 0.0 (no volatility sensitivity)
        - Theta: 0.0 (no time decay)
        - Rho: 0.0 (no interest rate sensitivity)

        Args:
            check_validity: If True, only return positions with valid data_accuracy_date
            max_age_days: Maximum age of data_accuracy_date in days

        Returns:
            Dict mapping symbol to Greeks dict
        """
        positions = self.storage.load_positions()
        greeks = {}

        for product in positions.products:
            # Check validity date if requested
            if check_validity:
                age_days = (datetime.now() - product.data_accuracy_date).days
                if age_days > max_age_days:
                    continue

            symbol = f"SWIFTNESS-{product.policy_number}"
            greeks[symbol] = {
                "delta": 1.0,  # Cash-like position
                "gamma": 0.0,
                "vega": 0.0,
                "theta": 0.0,
                "rho": 0.0,
            }

        logger.debug(f"Calculated Greeks for {len(greeks)} Swiftness positions")
        return greeks

    def get_cash_flows(
        self,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        check_validity: bool = True,
        max_age_days: int = 90,
    ) -> List[CashFlowEvent]:
        """
        Get cash flow events from Swiftness positions.

        Includes:
        - Monthly benefits from products (monthly_benefit)
        - Next withdrawal dates (one-time withdrawals)
        - Historical deposits (for pattern analysis)

        Args:
            start_date: Start date for cash flow timeline (default: today)
            end_date: End date for cash flow timeline (default: 1 year from today)
            check_validity: If True, only use positions with valid data_accuracy_date
            max_age_days: Maximum age of data_accuracy_date in days

        Returns:
            List of CashFlowEvent objects
        """
        if start_date is None:
            start_date = datetime.now()
        if end_date is None:
            from datetime import timedelta
            end_date = start_date + timedelta(days=365)

        positions = self.storage.load_positions()
        cash_flows = []

        # Process products for monthly benefits and withdrawals
        for product in positions.products:
            # Check validity date if requested
            if check_validity:
                age_days = (datetime.now() - product.data_accuracy_date).days
                if age_days > max_age_days:
                    logger.warning(
                        f"Skipping stale Swiftness product for cash flows: {product.policy_number}"
                    )
                    continue

            # Monthly benefits (recurring cash flows)
            if product.monthly_benefit > 0:
                # Generate monthly cash flows from start_date to end_date
                current_date = start_date
                month_count = 0
                while current_date <= end_date and month_count < 12:  # Limit to 12 months
                    monthly_benefit_usd = product.monthly_benefit * self.ils_to_usd_rate
                    cash_flows.append(
                        CashFlowEvent(
                            date=current_date,
                            amount=monthly_benefit_usd,
                            currency="ILS",
                            description=f"Swiftness monthly benefit: {product.product_name} ({product.policy_number})",
                            source="swiftness_product",
                        )
                    )
                    # Move to next month
                    current_date = add_months(current_date, 1)
                    month_count += 1

            # Next withdrawal date (one-time cash flow)
            if product.next_withdrawal_date:
                if start_date <= product.next_withdrawal_date <= end_date:
                    # Estimate withdrawal amount (use projected_savings as proxy)
                    withdrawal_amount_usd = product.projected_savings * self.ils_to_usd_rate
                    cash_flows.append(
                        CashFlowEvent(
                            date=product.next_withdrawal_date,
                            amount=withdrawal_amount_usd,
                            currency="ILS",
                            description=f"Swiftness withdrawal: {product.product_name} ({product.policy_number})",
                            source="swiftness_product",
                        )
                    )

        # Process deposits for historical pattern analysis (optional)
        # Deposits are historical, but can be used to identify patterns
        for deposit in positions.deposits:
            if start_date <= deposit.value_date <= end_date:
                total_deposit_ils = (
                    deposit.employee_deposit
                    + deposit.employer_deposit
                    + deposit.employer_severance
                )
                if total_deposit_ils > 0:
                    total_deposit_usd = total_deposit_ils * self.ils_to_usd_rate
                    cash_flows.append(
                        CashFlowEvent(
                            date=deposit.value_date,
                            amount=total_deposit_usd,
                            currency="ILS",
                            description=f"Swiftness deposit: {deposit.policy_number}",
                            source="swiftness_deposit",
                        )
                    )

        # Sort by date
        cash_flows.sort(key=lambda x: x.date)

        logger.info(
            f"Generated {len(cash_flows)} cash flow events from Swiftness positions "
            f"({start_date.date()} to {end_date.date()})"
        )
        return cash_flows

    def get_portfolio_value(
        self,
        check_validity: bool = True,
        max_age_days: int = 90,
    ) -> float:
        """
        Get total portfolio value from Swiftness positions in USD.

        Args:
            check_validity: If True, only include positions with valid data_accuracy_date
            max_age_days: Maximum age of data_accuracy_date in days

        Returns:
            Total portfolio value in USD
        """
        positions = self.storage.load_positions()
        total_value_usd = 0.0

        for product in positions.products:
            # Check validity date if requested
            if check_validity:
                age_days = (datetime.now() - product.data_accuracy_date).days
                if age_days > max_age_days:
                    continue

            total_value_usd += product.total_savings * self.ils_to_usd_rate

        logger.debug(f"Swiftness portfolio value: {total_value_usd:.2f} USD")
        return total_value_usd

    def validate_positions(self) -> Dict[str, any]:
        """
        Validate Swiftness positions and return validation report.

        Returns:
            Dict with validation results
        """
        positions = self.storage.load_positions()
        now = datetime.now()

        report = {
            "total_products": len(positions.products),
            "total_deposits": len(positions.deposits),
            "total_coverage": len(positions.insurance_coverage),
            "stale_products": [],
            "valid_products": [],
            "missing_validity_dates": [],
            "last_updated": positions.last_updated.isoformat() if positions.last_updated else None,
        }

        for product in positions.products:
            age_days = (now - product.data_accuracy_date).days
            if age_days > 90:
                report["stale_products"].append({
                    "policy_number": product.policy_number,
                    "age_days": age_days,
                    "data_accuracy_date": product.data_accuracy_date.isoformat(),
                })
            else:
                report["valid_products"].append({
                    "policy_number": product.policy_number,
                    "age_days": age_days,
                    "total_savings_ils": product.total_savings,
                })

        logger.info(
            f"Validation complete: {len(report['valid_products'])} valid, "
            f"{len(report['stale_products'])} stale products"
        )
        return report
