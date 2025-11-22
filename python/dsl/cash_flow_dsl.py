"""
Cash Flow Modeling DSL

Provides DSL for modeling and projecting cash flows across multiple positions.
"""

from typing import List, Optional
from dataclasses import dataclass
from decimal import Decimal


@dataclass
class Position:
    """Position in cash flow model"""
    type: str  # "box_spread_lending", "bank_loan", "pension_loan", etc.
    amount: Decimal
    rate: Decimal
    maturity: str  # ISO date format
    payments: Optional[str] = None  # "monthly", "quarterly", "annually", etc.
    currency: str = "USD"


@dataclass
class CashFlowResult:
    """Result from cash flow simulation"""
    positions: List[Position]
    projection_months: int
    monthly_cash_flows: List[dict]
    total_net_cash_flow: Decimal
    errors: List[str] = None


class CashFlowModel:
    """Builder for cash flow models"""

    def __init__(self):
        self.positions: List[Position] = []
        self.projection_months: int = 12
        self.optimization: Optional[str] = None

    def add_position(self, position: Position) -> 'CashFlowModel':
        """Add position to cash flow model"""
        self.positions.append(position)
        return self

    def project(self, months: int) -> 'CashFlowModel':
        """Set projection period in months"""
        if months < 1:
            raise ValueError("Projection period must be at least 1 month")
        self.projection_months = months
        return self

    def optimize(self, objective: str) -> 'CashFlowModel':
        """Set optimization objective"""
        valid_objectives = ["net_cash_flow", "minimize_cost", "maximize_return"]
        if objective not in valid_objectives:
            raise ValueError(f"Invalid objective. Must be one of: {valid_objectives}")
        self.optimization = objective
        return self

    def validate(self) -> List[str]:
        """Validate cash flow model"""
        errors = []

        if not self.positions:
            errors.append("At least one position is required")

        if self.projection_months < 1:
            errors.append("Projection period must be at least 1 month")

        return errors

    def simulate(self) -> CashFlowResult:
        """Run cash flow simulation

        Note: This is a stub implementation. Full implementation would:
        1. Calculate monthly cash flows for each position
        2. Aggregate cash flows
        3. Apply optimization if specified
        4. Return detailed results

        Returns:
            CashFlowResult with monthly cash flows and totals
        """
        errors = self.validate()
        if errors:
            return CashFlowResult(
                positions=self.positions,
                projection_months=self.projection_months,
                monthly_cash_flows=[],
                total_net_cash_flow=Decimal("0"),
                errors=errors
            )

        # TODO: Implement actual cash flow calculation
        return CashFlowResult(
            positions=self.positions,
            projection_months=self.projection_months,
            monthly_cash_flows=[],
            total_net_cash_flow=Decimal("0"),
            errors=None
        )

    def __str__(self) -> str:
        """String representation"""
        return f"CashFlowModel(positions={len(self.positions)}, months={self.projection_months})"


# Helper functions for creating positions

def box_spread_lending(amount: float, rate: float, maturity: str) -> Position:
    """Create box spread lending position"""
    return Position(
        type="box_spread_lending",
        amount=Decimal(str(amount)),
        rate=Decimal(str(rate)),
        maturity=maturity
    )


def bank_loan(amount: float, rate: float, payments: str = "monthly") -> Position:
    """Create bank loan position"""
    return Position(
        type="bank_loan",
        amount=Decimal(str(amount)),
        rate=Decimal(str(rate)),
        maturity="",  # Ongoing
        payments=payments
    )


def pension_loan(amount: float, rate: float, collateral: str) -> Position:
    """Create pension loan position"""
    return Position(
        type="pension_loan",
        amount=Decimal(str(amount)),
        rate=Decimal(str(rate)),
        maturity="",  # Ongoing
        payments="monthly"
    )
