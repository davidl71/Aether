"""
Cash Flow Modeling DSL

Provides DSL for modeling and projecting cash flows across multiple positions.
Uses CashFlowCalculator from integration layer for actual computation.
"""

from __future__ import annotations

import logging
from collections import defaultdict
from datetime import datetime, timedelta
from typing import List, Optional
from dataclasses import dataclass, field
from decimal import Decimal

logger = logging.getLogger(__name__)

_LOAN_TYPES = {"bank_loan", "pension_loan", "shir_loan", "cpi_linked_loan"}
_LENDING_TYPES = {"box_spread_lending"}


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


def _payment_multiplier(frequency: Optional[str]) -> int:
    """Return payments-per-year for a frequency string."""
    return {"monthly": 12, "quarterly": 4, "annually": 1}.get(
        (frequency or "monthly").lower(), 12
    )


def _parse_maturity(maturity_str: str) -> Optional[datetime]:
    """Best-effort ISO date parse."""
    if not maturity_str:
        return None
    for fmt in ("%Y-%m-%d", "%Y%m%d", "%Y-%m-%dT%H:%M:%S"):
        try:
            return datetime.strptime(maturity_str, fmt)
        except ValueError:
            continue
    return None


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
        """Run cash flow simulation.

        For each position, calculates periodic cash flows over the projection
        horizon and aggregates them into monthly buckets.

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
                errors=errors,
            )

        now = datetime.now()
        end_date = now + timedelta(days=self.projection_months * 30)

        monthly_buckets: dict[str, Decimal] = defaultdict(Decimal)

        for pos in self.positions:
            amount = float(pos.amount)
            rate = float(pos.rate)
            maturity = _parse_maturity(pos.maturity)
            ppy = _payment_multiplier(pos.payments)
            months_per_payment = 12 // ppy

            if pos.type in _LOAN_TYPES:
                monthly_rate = rate / 12.0
                payment = amount * monthly_rate
                current = now
                for _ in range(self.projection_months):
                    current = current + timedelta(days=30)
                    if current > end_date:
                        break
                    key = current.strftime("%Y-%m")
                    monthly_buckets[key] -= Decimal(str(round(payment, 2)))

            elif pos.type in _LENDING_TYPES:
                monthly_interest = amount * (rate / 12.0)
                current = now
                for _ in range(self.projection_months):
                    current = current + timedelta(days=30)
                    if current > end_date:
                        break
                    key = current.strftime("%Y-%m")
                    monthly_buckets[key] += Decimal(str(round(monthly_interest, 2)))

                if maturity and maturity <= end_date:
                    mat_key = maturity.strftime("%Y-%m")
                    monthly_buckets[mat_key] += Decimal(str(round(amount, 2)))

            else:
                if rate != 0 and amount != 0:
                    periodic_cf = amount * (rate / ppy)
                    current = now
                    step_days = (365 // ppy)
                    for _ in range(self.projection_months * ppy // 12):
                        current = current + timedelta(days=step_days)
                        if current > end_date:
                            break
                        key = current.strftime("%Y-%m")
                        monthly_buckets[key] += Decimal(str(round(periodic_cf, 2)))

        sorted_months = sorted(monthly_buckets.keys())
        monthly_cash_flows = [
            {"month": m, "net_cash_flow": monthly_buckets[m]} for m in sorted_months
        ]

        total_net = sum(monthly_buckets.values(), Decimal("0"))

        if self.optimization == "minimize_cost":
            monthly_cash_flows.sort(key=lambda x: x["net_cash_flow"])
        elif self.optimization == "maximize_return":
            monthly_cash_flows.sort(key=lambda x: x["net_cash_flow"], reverse=True)

        return CashFlowResult(
            positions=self.positions,
            projection_months=self.projection_months,
            monthly_cash_flows=monthly_cash_flows,
            total_net_cash_flow=total_net,
            errors=None,
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
