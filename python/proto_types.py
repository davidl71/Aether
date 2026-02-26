"""Canonical Python types mirroring proto/messages.proto.

These dataclasses are the single source of truth for cross-language
message shapes until betterproto codegen is wired into the Python
build.  When codegen is enabled, replace imports of this module with
imports from ``python/generated/``.

All field names, types, and ordering match ``ib.platform.v1`` in
``proto/messages.proto``.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import List, Optional


# ---------------------------------------------------------------------------
# Market Data
# ---------------------------------------------------------------------------


@dataclass
class MarketDataEvent:
    symbol: str = ""
    bid: float = 0.0
    ask: float = 0.0
    last: float = 0.0
    volume: int = 0
    timestamp: Optional[datetime] = None


@dataclass
class CandleSnapshot:
    open: float = 0.0
    high: float = 0.0
    low: float = 0.0
    close: float = 0.0
    volume: int = 0
    entry: float = 0.0
    updated: Optional[datetime] = None


@dataclass
class SymbolSnapshot:
    symbol: str = ""
    last: float = 0.0
    bid: float = 0.0
    ask: float = 0.0
    spread: float = 0.0
    roi: float = 0.0
    maker_count: int = 0
    taker_count: int = 0
    volume: int = 0
    candle: Optional[CandleSnapshot] = None


# ---------------------------------------------------------------------------
# Positions & Orders
# ---------------------------------------------------------------------------


@dataclass
class Position:
    """Core position — maps 1-to-1 with ``ib.platform.v1.Position``."""

    id: str = ""
    symbol: str = ""
    quantity: int = 0
    cost_basis: float = 0.0
    mark: float = 0.0
    unrealized_pnl: float = 0.0

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "symbol": self.symbol,
            "quantity": self.quantity,
            "cost_basis": self.cost_basis,
            "mark": self.mark,
            "unrealized_pnl": self.unrealized_pnl,
        }

    @classmethod
    def from_dict(cls, d: dict) -> "Position":
        return cls(
            id=d.get("id", ""),
            symbol=d.get("symbol", ""),
            quantity=int(d.get("quantity", 0)),
            cost_basis=float(d.get("cost_basis", 0.0)),
            mark=float(d.get("mark", 0.0)),
            unrealized_pnl=float(d.get("unrealized_pnl", 0.0)),
        )


@dataclass
class HistoricPosition:
    id: str = ""
    symbol: str = ""
    quantity: int = 0
    realized_pnl: float = 0.0
    closed_at: Optional[datetime] = None

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "symbol": self.symbol,
            "quantity": self.quantity,
            "realized_pnl": self.realized_pnl,
            "closed_at": self.closed_at.isoformat() if self.closed_at else None,
        }


@dataclass
class Order:
    id: str = ""
    symbol: str = ""
    side: str = ""
    quantity: int = 0
    status: str = ""
    submitted_at: Optional[datetime] = None


# ---------------------------------------------------------------------------
# Strategy
# ---------------------------------------------------------------------------


@dataclass
class StrategyDecision:
    symbol: str = ""
    quantity: int = 0
    side: str = ""
    mark: float = 0.0
    created_at: Optional[datetime] = None


@dataclass
class StrategySignal:
    symbol: str = ""
    price: float = 0.0
    timestamp: Optional[datetime] = None


# ---------------------------------------------------------------------------
# Risk
# ---------------------------------------------------------------------------


@dataclass
class RiskStatus:
    allowed: bool = True
    reason: str = ""
    updated_at: Optional[datetime] = None


@dataclass
class RiskLimit:
    symbol: str = ""
    max_position: int = 0
    max_notional: float = 0.0


# ---------------------------------------------------------------------------
# System
# ---------------------------------------------------------------------------


@dataclass
class Alert:
    level: str = "INFO"  # INFO, WARNING, ERROR
    message: str = ""
    timestamp: Optional[datetime] = None

    @classmethod
    def info(cls, message: str) -> "Alert":
        return cls(level="INFO", message=message, timestamp=datetime.now(timezone.utc))


@dataclass
class Metrics:
    net_liq: float = 0.0
    buying_power: float = 0.0
    excess_liquidity: float = 0.0
    margin_requirement: float = 0.0
    commissions: float = 0.0
    portal_ok: bool = False
    tws_ok: bool = False
    orats_ok: bool = False
    questdb_ok: bool = False
    nats_ok: bool = False


@dataclass
class SystemSnapshot:
    generated_at: Optional[datetime] = None
    started_at: Optional[datetime] = None
    mode: str = "DRY-RUN"
    strategy: str = "IDLE"
    account_id: str = ""
    metrics: Optional[Metrics] = None
    symbols: List[SymbolSnapshot] = field(default_factory=list)
    positions: List[Position] = field(default_factory=list)
    historic: List[HistoricPosition] = field(default_factory=list)
    orders: List[Order] = field(default_factory=list)
    decisions: List[StrategyDecision] = field(default_factory=list)
    alerts: List[Alert] = field(default_factory=list)
    risk: Optional[RiskStatus] = None


# ---------------------------------------------------------------------------
# Box Spread
# ---------------------------------------------------------------------------


@dataclass
class BoxSpreadScenario:
    symbol: str = ""
    strike_width: float = 0.0
    theoretical_value: float = 0.0
    estimated_net_debit: float = 0.0
    implied_apr: float = 0.0
    scenario_type: str = ""


@dataclass
class BoxSpreadExecution:
    symbol: str = ""
    lower_strike: int = 0
    upper_strike: int = 0
    expiry: str = ""
    net_debit: float = 0.0
    trade_id: str = ""
    executed_at: Optional[datetime] = None
