"""
Financing Strategy DSL - Multi-asset financing relationships

Provides DSL for expressing complex multi-asset financing strategies.
"""

from typing import List
from dataclasses import dataclass


@dataclass
class FinancingSource:
    """Source of financing (loan, box spread, etc.)"""
    type: str
    rate: float
    amount: float
    currency: str = "USD"


@dataclass
class FinancingUse:
    """How financing source is used"""
    type: str  # "margin_for", "invest_in", etc.
    target: str
    parameters: dict


@dataclass
class Optimization:
    """Optimization objective"""
    type: str  # "minimize_total_cost", "maximize_rate_advantage", etc.
    constraints: List[dict] = None


class FinancingStrategy:
    """Builder for multi-asset financing strategies"""

    def __init__(self, name: str):
        if not name:
            raise ValueError("Strategy name is required")
        self.name = name
        self.sources: List[FinancingSource] = []
        self.uses: List[FinancingUse] = []
        self.optimizations: List[Optimization] = []

    def source(self, source: FinancingSource) -> 'FinancingStrategy':
        """Add financing source (loan, box spread, etc.)"""
        self.sources.append(source)
        return self

    def use_as(self, use: FinancingUse) -> 'FinancingStrategy':
        """Specify how source is used"""
        self.uses.append(use)
        return self

    def then(self, action: FinancingUse) -> 'FinancingStrategy':
        """Chain financing actions"""
        self.uses.append(action)
        return self

    def optimize(self, objective: Optimization) -> 'FinancingStrategy':
        """Set optimization objective"""
        self.optimizations.append(objective)
        return self

    def validate(self) -> List[str]:
        """Validate strategy constraints"""
        errors = []

        if not self.sources:
            errors.append("At least one financing source is required")

        if not self.uses:
            errors.append("At least one financing use is required")

        return errors

    def __str__(self) -> str:
        """String representation"""
        return f"FinancingStrategy({self.name}, sources={len(self.sources)}, uses={len(self.uses)})"


# Helper functions for creating sources and uses

def bank_loan(rate: float, amount: float, currency: str = "USD") -> FinancingSource:
    """Create bank loan source"""
    return FinancingSource(type="bank_loan", rate=rate, amount=amount, currency=currency)


def box_spread(symbol: str, min_rate: float) -> FinancingSource:
    """Create box spread source"""
    return FinancingSource(type="box_spread", rate=min_rate, amount=0, currency="USD")


def margin_for(target: FinancingSource) -> FinancingUse:
    """Use source as margin for target"""
    return FinancingUse(
        type="margin_for",
        target=target.type,
        parameters={"target": target}
    )


def invest_in(fund: str, rate: float) -> FinancingUse:
    """Invest in fund"""
    return FinancingUse(
        type="invest_in",
        target=fund,
        parameters={"rate": rate}
    )


def minimize_total_cost() -> Optimization:
    """Optimize for minimum total cost"""
    return Optimization(type="minimize_total_cost")


def maximize_rate_advantage() -> Optimization:
    """Optimize for maximum rate advantage"""
    return Optimization(type="maximize_rate_advantage")
