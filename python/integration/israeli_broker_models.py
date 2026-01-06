"""
israeli_broker_models.py - Data models for Israeli broker position import
"""
from dataclasses import dataclass
from datetime import datetime
from typing import Optional
from enum import Enum


class PositionSource(Enum):
    """Source of position data."""
    IBKR = "ibkr"
    ISRAELI_BROKER_EXCEL = "israeli_excel"
    ISRAELI_BROKER_RTD = "israeli_rtd"
    ISRAELI_BROKER_DDE = "israeli_dde"
    ISRAELI_BROKER_WEB = "israeli_web"


@dataclass
class Position:
    """Standardized position data model for Israeli broker imports."""
    symbol: str
    quantity: float
    cost_basis: float
    current_price: float
    currency: str  # USD, ILS, etc.
    broker: str
    source: PositionSource
    account_id: Optional[str] = None
    last_updated: Optional[datetime] = None
    unrealized_pnl: Optional[float] = None

    # TASE-specific fields
    exchange: Optional[str] = None  # "TASE", "NYSE", etc.
    instrument_type: Optional[str] = None  # "stock", "option", "future", "bond", "etf"
    underlying: Optional[str] = None  # For derivatives: underlying asset (TA-35, USD/ILS, stock symbol)
    strike: Optional[float] = None  # For options/futures
    expiration_date: Optional[datetime] = None  # For options/futures
    option_type: Optional[str] = None  # "call", "put" for options

    def calculate_pnl(self) -> float:
        """Calculate unrealized P&L."""
        if self.unrealized_pnl is not None:
            return self.unrealized_pnl
        return (self.current_price - self.cost_basis) * self.quantity

    def get_market_value_usd(self, fx_rate: float) -> float:
        """Convert market value to USD."""
        market_value_local = self.current_price * self.quantity
        if self.currency == "USD":
            return market_value_local
        return market_value_local * fx_rate

    def is_tase_instrument(self) -> bool:
        """Check if position is TASE-listed."""
        return bool(self.exchange and "TASE" in self.exchange.upper())

    def is_tase_derivative(self) -> bool:
        """Check if position is a TASE derivative (option/future)."""
        return (self.is_tase_instrument() and
                self.instrument_type in ["option", "future"])

    def get_tase_index_type(self) -> Optional[str]:
        """Get TASE index type if this is an index derivative."""
        if not self.is_tase_derivative():
            return None

        underlying_upper = (self.underlying or "").upper()
        if "TA-35" in underlying_upper or "TA35" in underlying_upper:
            return "TA-35"
        elif "TA-125" in underlying_upper or "TA125" in underlying_upper:
            return "TA-125"
        elif "TA-90" in underlying_upper or "TA90" in underlying_upper:
            return "TA-90"
        elif "BANKS" in underlying_upper or "BANKS5" in underlying_upper:
            return "TA-Banks5"

        return None
