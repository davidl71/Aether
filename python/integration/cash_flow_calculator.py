"""
Cash Flow Calculator

Implements cash flow calculations for all asset types:
- Loan payments (SHIR-based variable, CPI-linked fixed)
- Option expiration cash flows
- Bond cash flows (coupons + maturity)
- Dividend cash flows

Based on design from docs/research/architecture/CASH_FLOW_FORECASTING_SYSTEM.md
"""

from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import Enum
from typing import List, Dict, Optional
import logging

logger = logging.getLogger(__name__)


class CashFlowType(Enum):
    """Types of cash flow events."""
    LOAN_PAYMENT = "loan_payment"
    OPTION_EXPIRATION = "option_expiration"
    BOND_COUPON = "bond_coupon"
    BOND_MATURITY = "bond_maturity"
    DIVIDEND = "dividend"


@dataclass
class CashFlowEvent:
    """Single cash flow event."""
    date: datetime
    amount: float  # Positive for inflows, negative for outflows
    currency: str  # USD, ILS, etc.
    cash_flow_type: CashFlowType
    description: str
    position_id: Optional[str] = None
    loan_id: Optional[str] = None
    # Type-specific fields
    underlying_price: Optional[float] = None  # For options
    strike: Optional[float] = None  # For options
    coupon_rate: Optional[float] = None  # For bonds
    dividend_per_share: Optional[float] = None  # For dividends
    is_box_spread: bool = False  # For box spreads
    box_strike_width: Optional[float] = None  # For box spreads


@dataclass
class CashFlowTimeline:
    """Aggregated cash flow timeline for portfolio."""
    cash_flows: List[CashFlowEvent]  # Sorted by date
    total_inflows: float  # Sum of positive cash flows
    total_outflows: float  # Sum of negative cash flows
    net_cash_flow: float  # Total inflows - total outflows
    cumulative_balance: Dict[datetime, float]  # Projected cash balance over time
    currency: str  # Base currency (USD)

    def get_cash_flow_by_date_range(
        self,
        start_date: datetime,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Get cash flows within date range."""
        return [
            cf for cf in self.cash_flows
            if start_date <= cf.date <= end_date
        ]


@dataclass
class Loan:
    """Loan position for cash flow calculation."""
    id: str
    loan_type: str  # "SHIR" or "CPI_LINKED"
    principal: float
    currency: str  # ILS or USD
    # SHIR-based loan fields
    shir_rate: Optional[float] = None  # Current SHIR rate
    spread: Optional[float] = None  # Spread over SHIR
    # CPI-linked loan fields
    fixed_rate: Optional[float] = None  # Fixed interest rate
    cpi_linked: bool = False  # Whether principal is CPI-linked


@dataclass
class Position:
    """Position for cash flow calculation (simplified interface)."""
    id: str
    symbol: str
    instrument_type: str  # "option", "bond", "bond_etf", "stock", "etf"
    quantity: float
    currency: str
    current_price: Optional[float] = None
    expiration_date: Optional[datetime] = None
    strike: Optional[float] = None
    option_type: Optional[str] = None  # "call" or "put"
    is_box_spread: bool = False
    box_strike_width: Optional[float] = None
    # Bond fields
    coupon_rate: Optional[float] = None
    maturity_date: Optional[datetime] = None
    payment_frequency: Optional[int] = None  # Payments per year (2 = semi-annual, 4 = quarterly, 12 = monthly)


class CashFlowCalculator:
    """Calculate future cash flows from all portfolio positions and loans."""

    def __init__(
        self,
        orats_client=None,  # For dividend schedules
        ibkr_client=None,  # For option/bond details
        fx_rates: Optional[Dict[str, float]] = None  # ILS/USD, etc.
    ):
        self.orats_client = orats_client
        self.ibkr_client = ibkr_client
        self.fx_rates = fx_rates or {"ILS_USD": 0.27}  # Default ILS/USD rate

    def calculate_portfolio_cash_flows(
        self,
        positions: List[Position],
        loans: List[Loan],
        forecast_horizon_days: int = 365
    ) -> CashFlowTimeline:
        """
        Calculate all future cash flows for portfolio.

        Args:
            positions: List of all positions (IBKR + Israeli brokers)
            loans: List of loans (SHIR-based + CPI-linked)
            forecast_horizon_days: Number of days to forecast ahead

        Returns:
            CashFlowTimeline with all cash flows sorted by date
        """
        all_cash_flows: List[CashFlowEvent] = []
        end_date = datetime.now() + timedelta(days=forecast_horizon_days)

        # Calculate loan payment cash flows
        for loan in loans:
            loan_cash_flows = self.calculate_loan_cash_flows(loan, end_date)
            all_cash_flows.extend(loan_cash_flows)

        # Calculate option expiration cash flows
        for position in positions:
            if position.instrument_type == "option":
                option_cash_flows = self.calculate_option_cash_flows(position, end_date)
                all_cash_flows.extend(option_cash_flows)

        # Calculate bond cash flows (coupons + maturity)
        for position in positions:
            if position.instrument_type in ["bond", "bond_etf"]:
                bond_cash_flows = self.calculate_bond_cash_flows(position, end_date)
                all_cash_flows.extend(bond_cash_flows)

        # Calculate dividend cash flows
        for position in positions:
            if position.instrument_type in ["stock", "etf"]:
                dividend_cash_flows = self.calculate_dividend_cash_flows(position, end_date)
                all_cash_flows.extend(dividend_cash_flows)

        # Sort by date
        all_cash_flows.sort(key=lambda cf: cf.date)

        # Convert all to base currency (USD)
        for cf in all_cash_flows:
            if cf.currency != "USD":
                fx_key = f"{cf.currency}_USD"
                fx_rate = self.fx_rates.get(fx_key, 1.0)
                cf.amount *= fx_rate
                cf.currency = "USD"

        # Calculate totals and cumulative balance
        total_inflows = sum(cf.amount for cf in all_cash_flows if cf.amount > 0)
        total_outflows = sum(abs(cf.amount) for cf in all_cash_flows if cf.amount < 0)
        net_cash_flow = total_inflows - total_outflows

        # Generate cumulative balance projection
        cumulative_balance = self._calculate_cumulative_balance(all_cash_flows)

        return CashFlowTimeline(
            cash_flows=all_cash_flows,
            total_inflows=total_inflows,
            total_outflows=total_outflows,
            net_cash_flow=net_cash_flow,
            cumulative_balance=cumulative_balance,
            currency="USD"
        )

    def calculate_loan_cash_flows(
        self,
        loan: Loan,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Calculate loan payment cash flows."""
        cash_flows: List[CashFlowEvent] = []
        current_date = datetime.now()

        # Calculate monthly payments
        while current_date <= end_date:
            # Calculate next payment date (monthly)
            next_payment_date = self._add_months(current_date, 1)
            if next_payment_date > end_date:
                break

            # Calculate payment amount
            if loan.loan_type == "SHIR":
                # Variable SHIR-based loan
                payment_amount = self._calculate_shir_loan_payment(loan)
            else:  # CPI_LINKED
                # Fixed rate CPI-linked loan
                payment_amount = self._calculate_cpi_linked_loan_payment(loan)

            cash_flows.append(CashFlowEvent(
                date=next_payment_date,
                amount=-payment_amount,  # Negative for outflows
                currency=loan.currency,
                cash_flow_type=CashFlowType.LOAN_PAYMENT,
                description=f"{loan.loan_type} loan payment",
                loan_id=loan.id
            ))

            current_date = next_payment_date

        return cash_flows

    def calculate_option_cash_flows(
        self,
        position: Position,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Calculate option expiration cash flows."""
        if not position.expiration_date:
            return []

        if position.expiration_date > end_date:
            return []

        cash_flows: List[CashFlowEvent] = []

        # For box spreads, calculate guaranteed cash flow
        if position.is_box_spread and position.box_strike_width:
            contract_multiplier = 100
            guaranteed_cash_flow = position.box_strike_width * contract_multiplier * position.quantity

            cash_flows.append(CashFlowEvent(
                date=position.expiration_date,
                amount=guaranteed_cash_flow,  # Positive for long box spread
                currency=position.currency,
                cash_flow_type=CashFlowType.OPTION_EXPIRATION,
                description=f"Box spread expiration: {position.symbol}",
                position_id=position.id,
                underlying_price=position.current_price,
                strike=position.strike,
                is_box_spread=True,
                box_strike_width=position.box_strike_width
            ))
        else:
            # Regular option - calculate intrinsic value
            if position.current_price and position.strike and position.option_type:
                intrinsic_value = self._calculate_option_intrinsic_value(
                    position, position.current_price
                )
                contract_multiplier = 100
                cash_flow_amount = intrinsic_value * contract_multiplier * position.quantity

                cash_flows.append(CashFlowEvent(
                    date=position.expiration_date,
                    amount=cash_flow_amount,  # Positive for long, negative for short
                    currency=position.currency,
                    cash_flow_type=CashFlowType.OPTION_EXPIRATION,
                    description=f"Option expiration: {position.symbol}",
                    position_id=position.id,
                    underlying_price=position.current_price,
                    strike=position.strike
                ))

        return cash_flows

    def calculate_bond_cash_flows(
        self,
        position: Position,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Calculate bond cash flows (coupons + maturity)."""
        cash_flows: List[CashFlowEvent] = []

        if not position.coupon_rate:
            # No coupon rate, skip coupon payments
            pass
        else:
            # Calculate coupon payments
            payment_frequency = position.payment_frequency or 2  # Default: semi-annual
            months_per_payment = 12 / payment_frequency

            current_date = datetime.now()
            face_value = abs(position.quantity) * (position.current_price or 100.0)  # Approximate face value
            coupon_payment = face_value * (position.coupon_rate / payment_frequency)

            # Estimate next payment date
            next_payment_date = self._add_months(current_date, months_per_payment)

            while next_payment_date <= end_date:
                cash_flows.append(CashFlowEvent(
                    date=next_payment_date,
                    amount=coupon_payment,
                    currency=position.currency,
                    cash_flow_type=CashFlowType.BOND_COUPON,
                    description=f"Bond coupon: {position.symbol}",
                    position_id=position.id,
                    coupon_rate=position.coupon_rate
                ))

                next_payment_date = self._add_months(next_payment_date, months_per_payment)

        # Bond maturity (principal returned)
        if position.maturity_date and position.maturity_date <= end_date:
            face_value = abs(position.quantity) * (position.current_price or 100.0)
            cash_flows.append(CashFlowEvent(
                date=position.maturity_date,
                amount=face_value,
                currency=position.currency,
                cash_flow_type=CashFlowType.BOND_MATURITY,
                description=f"Bond maturity: {position.symbol}",
                position_id=position.id
            ))

        return cash_flows

    def calculate_dividend_cash_flows(
        self,
        position: Position,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Calculate dividend cash flows."""
        cash_flows: List[CashFlowEvent] = []

        # TODO: Get dividend schedule from ORATS or IBKR
        # For now, return empty list (dividend schedules require external data)
        if self.orats_client:
            # Would fetch dividend schedule here
            pass

        return cash_flows

    def _calculate_shir_loan_payment(self, loan: Loan) -> float:
        """Calculate SHIR-based loan payment."""
        if not loan.shir_rate:
            logger.warning(f"SHIR rate not provided for loan {loan.id}, using default 4.5%")
            loan.shir_rate = 0.045

        spread = loan.spread or 0.0
        annual_rate = loan.shir_rate + spread
        monthly_rate = annual_rate / 12.0
        payment = loan.principal * monthly_rate

        return payment

    def _calculate_cpi_linked_loan_payment(self, loan: Loan) -> float:
        """Calculate CPI-linked loan payment."""
        if not loan.fixed_rate:
            logger.warning(f"Fixed rate not provided for loan {loan.id}, using default 3.5%")
            loan.fixed_rate = 0.035

        # Fixed interest payment (principal adjustment for CPI happens separately)
        monthly_rate = loan.fixed_rate / 12.0
        interest_payment = loan.principal * monthly_rate

        # TODO: Add CPI adjustment to principal
        # For now, return interest payment only

        return interest_payment

    def _calculate_option_intrinsic_value(
        self,
        position: Position,
        underlying_price: float
    ) -> float:
        """Calculate option intrinsic value."""
        if not position.strike or not position.option_type:
            return 0.0

        if position.option_type == "call":
            return max(0.0, underlying_price - position.strike)
        else:  # put
            return max(0.0, position.strike - underlying_price)

    def _calculate_cumulative_balance(
        self,
        cash_flows: List[CashFlowEvent],
        initial_cash: float = 0.0
    ) -> Dict[datetime, float]:
        """Calculate cumulative cash balance over time."""
        balance = initial_cash
        cumulative: Dict[datetime, float] = {}

        for cf in cash_flows:
            balance += cf.amount
            cumulative[cf.date] = balance

        return cumulative

    def _add_months(self, date: datetime, months: float) -> datetime:
        """Add months to a date (handles month-end edge cases)."""
        # Simplified implementation - add approximately 30 days per month
        days = int(months * 30)
        return date + timedelta(days=days)
