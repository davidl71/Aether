"""
Domain-Specific Types for Box Spread DSL

Provides type-safe domain types for financial calculations:
- Rate: Interest rates with precision
- StrikeWidth: Option strike widths
- Expiration: Expiration dates
- Money: Monetary amounts with currency
- LiquidityConstraints: Trading constraints
"""

from dataclasses import dataclass
from enum import Enum
from typing import Optional
from decimal import Decimal
from datetime import datetime, date


class Direction(Enum):
    """Box spread direction"""
    LENDING = "lending"
    BORROWING = "borrowing"


class Benchmark(Enum):
    """Benchmark rate sources"""
    SOFR = "SOFR"
    TREASURY = "TREASURY"
    MARGIN_LOAN = "MARGIN_LOAN"
    CUSTOM = "CUSTOM"


@dataclass
class Rate:
    """Domain-specific rate type with precision"""
    value: Decimal
    unit: str = "percent"  # "percent" or "bps"

    def __post_init__(self):
        if isinstance(self.value, (int, float)):
            self.value = Decimal(str(self.value))
        if self.value < 0:
            raise ValueError("Rate cannot be negative")
        if self.unit not in ("percent", "bps"):
            raise ValueError(f"Invalid unit: {self.unit}. Must be 'percent' or 'bps'")

    def to_bps(self) -> int:
        """Convert to basis points"""
        if self.unit == "percent":
            return int(self.value * 100)
        return int(self.value)

    def to_percent(self) -> Decimal:
        """Convert to percentage"""
        if self.unit == "bps":
            return self.value / 100
        return self.value

    def __str__(self) -> str:
        if self.unit == "percent":
            return f"{self.value}%"
        return f"{self.value} bps"


@dataclass
class StrikeWidth:
    """Strike width with currency"""
    value: Decimal
    currency: str = "USD"

    def __post_init__(self):
        if isinstance(self.value, (int, float)):
            self.value = Decimal(str(self.value))
        if self.value <= 0:
            raise ValueError("Strike width must be positive")

    def __str__(self) -> str:
        return f"{self.value} {self.currency}"


@dataclass
class Expiration:
    """Expiration date with validation"""
    date: str  # ISO format: "YYYY-MM-DD"
    days_to_expiry: Optional[int] = None

    def __post_init__(self):
        try:
            dt = datetime.fromisoformat(self.date)
            if dt.date() < date.today():
                raise ValueError("Expiration date must be in the future")

            # Calculate days to expiry if not provided
            if self.days_to_expiry is None:
                delta = dt.date() - date.today()
                self.days_to_expiry = delta.days
        except ValueError as e:
            if "Invalid date format" not in str(e):
                raise ValueError(f"Invalid date format: {self.date}. Use YYYY-MM-DD") from e
            raise

    def __str__(self) -> str:
        return self.date


@dataclass
class Money:
    """Money type with currency"""
    amount: Decimal
    currency: str = "USD"

    def __post_init__(self):
        if isinstance(self.amount, (int, float)):
            self.amount = Decimal(str(self.amount))
        if self.amount < 0:
            raise ValueError("Money amount cannot be negative")

    def __str__(self) -> str:
        return f"{self.amount} {self.currency}"


@dataclass
class LiquidityConstraints:
    """Liquidity filtering constraints"""
    min_volume: int = 100
    min_open_interest: int = 500
    max_bid_ask_spread: Decimal = Decimal("0.1")
    min_fill_probability: Decimal = Decimal("0.5")

    def __post_init__(self):
        if isinstance(self.max_bid_ask_spread, (int, float)):
            self.max_bid_ask_spread = Decimal(str(self.max_bid_ask_spread))
        if isinstance(self.min_fill_probability, (int, float)):
            self.min_fill_probability = Decimal(str(self.min_fill_probability))

        if self.min_volume < 0:
            raise ValueError("Minimum volume must be non-negative")
        if self.min_open_interest < 0:
            raise ValueError("Minimum open interest must be non-negative")
        if self.max_bid_ask_spread < 0:
            raise ValueError("Maximum bid-ask spread must be non-negative")
        if not (0 <= self.min_fill_probability <= 1):
            raise ValueError("Minimum fill probability must be between 0 and 1")
