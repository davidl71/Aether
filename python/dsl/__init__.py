"""
Box Spread DSL - Domain-Specific Language for Synthetic Financing

This module provides a Python embedded DSL for expressing box spread
synthetic financing scenarios, multi-asset relationships, and cash flow models.

Example:
    from box_spread_dsl import BoxSpread, Direction, Benchmark

    scenario = BoxSpread("SPX") \\
        .strike_width(50) \\
        .expiration("2025-12-19") \\
        .direction(Direction.LENDING) \\
        .min_implied_rate(4.5) \\
        .benchmark(Benchmark.SOFR)
"""

from .box_spread_dsl import BoxSpread, BoxSpreadResult
from .types import (
    Rate, StrikeWidth, Expiration, Money,
    Direction, Benchmark, LiquidityConstraints
)
from .financing_strategy_dsl import FinancingStrategy, FinancingSource, FinancingUse
from .cash_flow_dsl import CashFlowModel, Position

__all__ = [
    "BoxSpread",
    "BoxSpreadResult",
    "Rate",
    "StrikeWidth",
    "Expiration",
    "Money",
    "Direction",
    "Benchmark",
    "LiquidityConstraints",
    "FinancingStrategy",
    "FinancingSource",
    "FinancingUse",
    "CashFlowModel",
    "Position",
]
