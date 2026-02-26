"""
api_models.py - Pydantic models for LEAN REST API

This module defines Pydantic models matching the API contract schema
for request/response validation and serialization.
"""

from pydantic import BaseModel, Field
from typing import List, Optional
from datetime import datetime


class Metrics(BaseModel):
    """Account and system metrics."""
    net_liq: float
    buying_power: float
    excess_liquidity: float
    margin_requirement: float
    commissions: float
    portal_ok: bool
    tws_ok: bool
    orats_ok: bool
    questdb_ok: bool


class CandleData(BaseModel):
    """OHLCV candle data."""
    open: float
    high: float
    low: float
    close: float
    volume: int
    entry: Optional[float] = None
    updated: datetime


class SymbolSnapshot(BaseModel):
    """Symbol market data snapshot."""
    symbol: str
    last: float
    bid: float
    ask: float
    spread: float
    roi: float
    maker_count: int
    taker_count: int
    volume: int
    candle: Optional[CandleData] = None


class PositionSnapshot(BaseModel):
    """Position snapshot."""
    id: str
    symbol: str
    quantity: int
    cost_basis: float
    mark: float
    unrealized_pnl: float


class HistoricPosition(BaseModel):
    """Historic/closed position."""
    id: str
    symbol: str
    quantity: int
    realized_pnl: float
    closed_at: datetime


class OrderSnapshot(BaseModel):
    """Order snapshot."""
    id: str
    symbol: str
    side: str
    quantity: int
    status: str
    submitted_at: datetime


class StrategyDecisionSnapshot(BaseModel):
    """Strategy decision snapshot."""
    symbol: str
    quantity: int
    side: str
    mark: float
    created_at: datetime


class Alert(BaseModel):
    """Alert/notification."""
    level: str  # "info", "warning", "error"
    message: str
    timestamp: datetime


class RiskStatus(BaseModel):
    """Risk management status."""
    allowed: bool
    reason: Optional[str] = None
    updated_at: datetime


class CashFlowEventModel(BaseModel):
    """Single cash flow event."""
    date: str
    amount: float
    description: str
    position_name: str
    type: str

class MonthlyCashFlowModel(BaseModel):
    """Aggregated monthly cash flow."""
    month: str
    inflows: float
    outflows: float
    net: float
    events: List[CashFlowEventModel] = []

class CashFlowTimelineModel(BaseModel):
    """Cash flow timeline for snapshot."""
    events: List[CashFlowEventModel] = []
    monthly_flows: dict = {}
    total_inflows: float = 0.0
    total_outflows: float = 0.0
    net_cash_flow: float = 0.0

class SnapshotResponse(BaseModel):
    """Complete system snapshot response matching API contract."""
    generated_at: datetime
    mode: str
    strategy: str
    account_id: str
    metrics: Metrics
    symbols: List[SymbolSnapshot]
    positions: List[PositionSnapshot]
    historic: List[HistoricPosition]
    orders: List[OrderSnapshot]
    decisions: List[StrategyDecisionSnapshot]
    alerts: List[Alert]
    risk: RiskStatus
    cash_flow_timeline: Optional[CashFlowTimelineModel] = None


class StrategyStartRequest(BaseModel):
    """Request to start strategy."""
    confirm: bool = Field(default=False, description="Confirmation required to start strategy")


class StrategyStopRequest(BaseModel):
    """Request to stop strategy."""
    confirm: bool = Field(default=False, description="Confirmation required to stop strategy")


class CancelOrderRequest(BaseModel):
    """Request to cancel an order."""
    order_id: str = Field(..., description="Order ID to cancel")
    confirm: bool = Field(default=False, description="Confirmation required")


class ComboOrderRequest(BaseModel):
    """Request to place a combo order."""
    symbols: List[str] = Field(..., description="List of option symbols")
    quantities: List[int] = Field(..., description="Quantities for each leg (positive=long, negative=short)")
    limit_price: Optional[float] = Field(None, description="Limit price (None for market order)")
    confirm: bool = Field(default=False, description="Confirmation required")


class ErrorResponse(BaseModel):
    """Error response model."""
    error: str
    detail: Optional[str] = None
    timestamp: datetime = Field(default_factory=lambda: datetime.utcnow())


class HealthResponse(BaseModel):
    """Health check response."""
    status: str
    lean_running: bool
    timestamp: datetime = Field(default_factory=lambda: datetime.utcnow())
