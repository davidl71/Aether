"""
Shared data models for TUI and PWA

These models match the TypeScript types in web/src/types/snapshot.ts
and the C++ types in native/include/tui_data.h. This ensures data
consistency across all frontends.

MIGRATION NOTES FOR FUTURE C++ MIGRATION (pybind11):
- These dataclasses can be exposed to C++ via pybind11
- Consider using pybind11's py::class_ with properties for getters/setters
- JSON serialization can be handled by nlohmann/json on C++ side
- Keep Python models as source of truth for API contracts
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field, asdict
from typing import List, Optional, Dict, Any
from enum import Enum


class Severity(str, Enum):
    """Alert/order severity levels matching TypeScript Severity type"""
    INFO = "info"
    SUCCESS = "success"
    WARN = "warn"
    WARNING = "warning"
    ERROR = "error"
    CRITICAL = "critical"


@dataclass
class Candle:
    """OHLC candle data matching web/src/types/snapshot.ts Candle interface"""
    open: float = 0.0
    high: float = 0.0
    low: float = 0.0
    close: float = 0.0
    volume: float = 0.0
    entry: float = 0.0
    updated: str = ""  # ISO 8601 timestamp string

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization"""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> Candle:
        """Create from dictionary (e.g., from JSON)"""
        return cls(**data)


@dataclass
class OptionStrike:
    """Option strike data matching web/src/types/snapshot.ts OptionStrike interface"""
    strike: float = 0.0
    call_bid: float = 0.0
    call_ask: float = 0.0
    put_bid: float = 0.0
    put_ask: float = 0.0

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> OptionStrike:
        return cls(**data)


@dataclass
class OptionSeries:
    """Option series data matching web/src/types/snapshot.ts OptionSeries interface"""
    expiration: str = ""  # ISO 8601 date string
    strikes: List[OptionStrike] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "expiration": self.expiration,
            "strikes": [s.to_dict() for s in self.strikes]
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> OptionSeries:
        return cls(
            expiration=data.get("expiration", ""),
            strikes=[OptionStrike.from_dict(s) for s in data.get("strikes", [])]
        )


@dataclass
class SymbolSnapshot:
    """Symbol snapshot matching web/src/types/snapshot.ts SymbolSnapshot interface"""
    symbol: str = ""
    last: float = 0.0
    bid: float = 0.0
    ask: float = 0.0
    spread: float = 0.0
    roi: float = 0.0
    maker_count: int = 0
    taker_count: int = 0
    volume: float = 0.0
    candle: Candle = field(default_factory=Candle)
    option_chains: List[OptionSeries] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "symbol": self.symbol,
            "last": self.last,
            "bid": self.bid,
            "ask": self.ask,
            "spread": self.spread,
            "roi": self.roi,
            "maker_count": self.maker_count,
            "taker_count": self.taker_count,
            "volume": self.volume,
            "candle": self.candle.to_dict(),
            "option_chains": [oc.to_dict() for oc in self.option_chains]
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> SymbolSnapshot:
        return cls(
            symbol=data.get("symbol", ""),
            last=data.get("last", 0.0),
            bid=data.get("bid", 0.0),
            ask=data.get("ask", 0.0),
            spread=data.get("spread", 0.0),
            roi=data.get("roi", 0.0),
            maker_count=data.get("maker_count", 0),
            taker_count=data.get("taker_count", 0),
            volume=data.get("volume", 0.0),
            candle=Candle.from_dict(data.get("candle", {})),
            option_chains=[OptionSeries.from_dict(oc) for oc in data.get("option_chains", [])]
        )


InstrumentType = str  # 'box_spread', 'bank_loan', 'pension_loan', 'bond', 't_bill', 'futures', 'other'


@dataclass
class PositionSnapshot:
    """Position snapshot matching web/src/types/snapshot.ts PositionSnapshot interface"""
    name: str = ""
    quantity: int = 0
    roi: float = 0.0
    maker_count: int = 0
    taker_count: int = 0
    rebate_estimate: float = 0.0
    vega: float = 0.0
    theta: float = 0.0
    fair_diff: float = 0.0
    candle: Candle = field(default_factory=Candle)
    # Extended fields for unified positions
    instrument_type: Optional[str] = None  # InstrumentType
    rate: Optional[float] = None  # Annual rate (APR) for loans/financing
    maturity_date: Optional[str] = None  # ISO 8601 date string
    cash_flow: Optional[float] = None  # Expected cash flow amount
    collateral_value: Optional[float] = None  # Collateral value if applicable
    currency: Optional[str] = None  # Currency code (defaults to USD)
    market_value: Optional[float] = None  # Market value (e.g. for cash positions)
    # Price metrics (when available from broker)
    bid: Optional[float] = None
    ask: Optional[float] = None
    last: Optional[float] = None
    spread: Optional[float] = None
    price: Optional[float] = None  # Last or mid price for display
    side: Optional[str] = None  # "long" or "short"
    expected_cash_at_expiry: Optional[float] = None  # Option/bond/bill cash if held to expiry
    dividend: Optional[float] = None  # Next/expected dividend for position (total amount)
    conid: Optional[int] = None  # IB contract ID (for IBCID symbol lookup)

    def to_dict(self) -> Dict[str, Any]:
        result = {
            "name": self.name,
            "quantity": self.quantity,
            "roi": self.roi,
            "maker_count": self.maker_count,
            "taker_count": self.taker_count,
            "rebate_estimate": self.rebate_estimate,
            "vega": self.vega,
            "theta": self.theta,
            "fair_diff": self.fair_diff,
            "candle": self.candle.to_dict()
        }
        # Add extended fields if present
        if self.instrument_type is not None:
            result["instrument_type"] = self.instrument_type
        if self.rate is not None:
            result["rate"] = self.rate
        if self.maturity_date is not None:
            result["maturity_date"] = self.maturity_date
        if self.cash_flow is not None:
            result["cash_flow"] = self.cash_flow
        if self.collateral_value is not None:
            result["collateral_value"] = self.collateral_value
        if self.currency is not None:
            result["currency"] = self.currency
        if self.market_value is not None:
            result["market_value"] = self.market_value
        if self.bid is not None:
            result["bid"] = self.bid
        if self.ask is not None:
            result["ask"] = self.ask
        if self.last is not None:
            result["last"] = self.last
        if self.spread is not None:
            result["spread"] = self.spread
        if self.price is not None:
            result["price"] = self.price
        if self.side is not None:
            result["side"] = self.side
        if self.expected_cash_at_expiry is not None:
            result["expected_cash_at_expiry"] = self.expected_cash_at_expiry
        if self.dividend is not None:
            result["dividend"] = self.dividend
        if self.conid is not None:
            result["conid"] = self.conid
        return result

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> PositionSnapshot:
        # Backends (e.g. IB) often send "symbol"; TUI displays "name"
        name = data.get("name") or data.get("symbol", "")
        if isinstance(name, str):
            name = name.strip()
        else:
            name = str(name) if name else ""
        return cls(
            name=name,
            quantity=data.get("quantity", 0),
            roi=data.get("roi", 0.0),
            maker_count=data.get("maker_count", 0),
            taker_count=data.get("taker_count", 0),
            rebate_estimate=data.get("rebate_estimate", 0.0),
            vega=data.get("vega", 0.0),
            theta=data.get("theta", 0.0),
            fair_diff=data.get("fair_diff", 0.0),
            candle=Candle.from_dict(data.get("candle", {})),
            instrument_type=data.get("instrument_type"),
            rate=data.get("rate"),
            maturity_date=data.get("maturity_date"),
            cash_flow=data.get("cash_flow"),
            collateral_value=data.get("collateral_value"),
            currency=data.get("currency"),
            market_value=data.get("market_value"),
            bid=data.get("bid"),
            ask=data.get("ask"),
            last=data.get("last"),
            spread=data.get("spread"),
            price=data.get("price"),
            side=data.get("side"),
            expected_cash_at_expiry=data.get("expected_cash_at_expiry"),
            dividend=data.get("dividend"),
            conid=data.get("conid"),
        )


@dataclass
class FutureEvent:
    """Reported future cash flow event (dividend, principal, interest coupon, expiry).
    Used for broker-reported and derived events; included in snapshot and cash flow timeline."""
    event_type: str  # 'dividend' | 'principal_repayment' | 'interest_coupon' | 'expiry'
    date: str = ""  # ISO date YYYY-MM-DD; empty if unknown (e.g. next dividend)
    amount: float = 0.0  # Positive = inflow, negative = outflow
    currency: str = "USD"
    position_name: str = ""
    description: str = ""

    def to_dict(self) -> Dict[str, Any]:
        return {
            "event_type": self.event_type,
            "date": self.date,
            "amount": self.amount,
            "currency": self.currency,
            "position_name": self.position_name,
            "description": self.description or self.event_type.replace("_", " ").title(),
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> FutureEvent:
        return cls(
            event_type=data.get("event_type", "other"),
            date=data.get("date", ""),
            amount=float(data.get("amount", 0.0)),
            currency=(data.get("currency") or "USD").strip() or "USD",
            position_name=data.get("position_name", ""),
            description=data.get("description", ""),
        )


@dataclass
class TimelineEvent:
    """Timeline event (order or alert) matching web/src/types/snapshot.ts TimelineEvent interface"""
    timestamp: str = ""  # ISO 8601 timestamp string
    text: str = ""
    severity: Severity = Severity.INFO

    def to_dict(self) -> Dict[str, Any]:
        return {
            "timestamp": self.timestamp,
            "text": self.text,
            "severity": self.severity.value
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> TimelineEvent:
        severity_str = data.get("severity", "info")
        try:
            severity = Severity(severity_str)
        except ValueError:
            severity = Severity.INFO
        return cls(
            timestamp=data.get("timestamp", ""),
            text=data.get("text", ""),
            severity=severity
        )


@dataclass
class AccountMetrics:
    """Account metrics matching web/src/types/snapshot.ts AccountMetrics interface"""
    net_liq: float = 0.0
    buying_power: float = 0.0
    excess_liquidity: float = 0.0
    margin_requirement: float = 0.0
    commissions: float = 0.0
    portal_ok: bool = False
    tws_ok: bool = False
    questdb_ok: bool = False

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> AccountMetrics:
        filtered = {
            key: value
            for key, value in data.items()
            if key in cls.__dataclass_fields__
        }
        return cls(**filtered)


@dataclass
class SnapshotPayload:
    """
    Complete snapshot payload matching web/src/types/snapshot.ts SnapshotPayload interface

    This is the main data structure shared between Python TUI, PWA, and future C++ TUI.
    All frontends consume this same structure for consistency.
    """
    generated_at: str = ""  # ISO 8601 timestamp string
    mode: str = "DRY-RUN"  # "DRY-RUN" or "LIVE"
    strategy: str = "STOPPED"  # "RUNNING" or "STOPPED"
    account_id: str = ""
    metrics: AccountMetrics = field(default_factory=AccountMetrics)
    symbols: List[SymbolSnapshot] = field(default_factory=list)
    positions: List[PositionSnapshot] = field(default_factory=list)
    historic: List[PositionSnapshot] = field(default_factory=list)
    orders: List[TimelineEvent] = field(default_factory=list)
    alerts: List[TimelineEvent] = field(default_factory=list)
    future_events: List[FutureEvent] = field(default_factory=list)  # Reported dividends, coupons, expiry, etc.
    cash_flow_timeline: Optional[Dict[str, Any]] = None
    decisions: List[Any] = field(default_factory=list)
    risk: Optional[Dict[str, Any]] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization (shared with PWA)"""
        result = {
            "generated_at": self.generated_at,
            "mode": self.mode,
            "strategy": self.strategy,
            "account_id": self.account_id,
            "metrics": self.metrics.to_dict(),
            "symbols": [s.to_dict() for s in self.symbols],
            "positions": [p.to_dict() for p in self.positions],
            "historic": [p.to_dict() for p in self.historic],
            "orders": [o.to_dict() for o in self.orders],
            "alerts": [a.to_dict() for a in self.alerts],
        }
        if self.future_events:
            result["future_events"] = [e.to_dict() for e in self.future_events]
        if self.cash_flow_timeline is not None:
            result["cash_flow_timeline"] = self.cash_flow_timeline
        if self.decisions:
            result["decisions"] = self.decisions
        if self.risk is not None:
            result["risk"] = self.risk
        return result

    def to_json(self) -> str:
        """Serialize to JSON string (shared format with PWA)"""
        return json.dumps(self.to_dict(), indent=2)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> SnapshotPayload:
        """Create from dictionary (e.g., from REST API or JSON file)"""
        return cls(
            generated_at=data.get("generated_at", ""),
            mode=data.get("mode", "DRY-RUN"),
            strategy=data.get("strategy", "STOPPED"),
            account_id=data.get("account_id", ""),
            metrics=AccountMetrics.from_dict(data.get("metrics", {})),
            symbols=[SymbolSnapshot.from_dict(s) for s in data.get("symbols", [])],
            positions=[PositionSnapshot.from_dict(p) for p in data.get("positions", [])],
            historic=[PositionSnapshot.from_dict(p) for p in data.get("historic", [])],
            orders=[TimelineEvent.from_dict(o) for o in data.get("orders", [])],
            alerts=[TimelineEvent.from_dict(a) for a in data.get("alerts", [])],
            future_events=[FutureEvent.from_dict(e) for e in data.get("future_events", [])],
            cash_flow_timeline=data.get("cash_flow_timeline"),
            decisions=data.get("decisions", []),
            risk=data.get("risk"),
        )

    @classmethod
    def from_json(cls, json_str: str) -> SnapshotPayload:
        """Create from JSON string (shared format with PWA)"""
        return cls.from_dict(json.loads(json_str))


@dataclass
class BoxSpreadScenario:
    """Box spread scenario matching web/src/types.ts BoxSpreadScenario interface"""
    width: float = 0.0
    put_bid: float = 0.0
    call_ask: float = 0.0
    synthetic_bid: float = 0.0
    synthetic_ask: float = 0.0
    mid_price: float = 0.0
    annualized_return: float = 0.0
    fill_probability: float = 0.0
    option_style: str = "European"  # "European" or "American"

    # Optional fields
    buy_profit: Optional[float] = None
    buy_implied_rate: Optional[float] = None
    sell_profit: Optional[float] = None
    sell_implied_rate: Optional[float] = None
    buy_sell_disparity: Optional[float] = None
    put_call_parity_violation: Optional[float] = None
    expiration_date: Optional[str] = None
    days_to_expiry: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> BoxSpreadScenario:
        return cls(**{k: v for k, v in data.items() if k in cls.__dataclass_fields__})


@dataclass
class BoxSpreadPayload:
    """Box spread payload matching web/src/types.ts BoxSpreadPayload interface"""
    as_of: str = ""  # ISO 8601 timestamp string
    underlying: str = ""
    scenarios: List[BoxSpreadScenario] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "as_of": self.as_of,
            "underlying": self.underlying,
            "scenarios": [s.to_dict() for s in self.scenarios]
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> BoxSpreadPayload:
        return cls(
            as_of=data.get("as_of", ""),
            underlying=data.get("underlying", ""),
            scenarios=[BoxSpreadScenario.from_dict(s) for s in data.get("scenarios", [])]
        )

    @classmethod
    def from_json(cls, json_str: str) -> BoxSpreadPayload:
        """Create from JSON string"""
        return cls.from_dict(json.loads(json_str))


@dataclass
class BoxSpreadSummary:
    """Box spread summary statistics matching web/src/types.ts BoxSpreadSummary interface"""
    total_scenarios: int = 0
    avg_apr: float = 0.0
    probable_count: int = 0
    max_apr_scenario: Optional[BoxSpreadScenario] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "total_scenarios": self.total_scenarios,
            "avg_apr": self.avg_apr,
            "probable_count": self.probable_count,
            "max_apr_scenario": self.max_apr_scenario.to_dict() if self.max_apr_scenario else None
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> BoxSpreadSummary:
        max_scenario = None
        if data.get("max_apr_scenario"):
            max_scenario = BoxSpreadScenario.from_dict(data["max_apr_scenario"])
        return cls(
            total_scenarios=data.get("total_scenarios", 0),
            avg_apr=data.get("avg_apr", 0.0),
            probable_count=data.get("probable_count", 0),
            max_apr_scenario=max_scenario
        )

    @classmethod
    def calculate(cls, payload: BoxSpreadPayload) -> BoxSpreadSummary:
        """Calculate summary statistics from payload"""
        if not payload.scenarios:
            return cls()

        # Filter to European-style scenarios only for summary (default behavior)
        european_scenarios = [s for s in payload.scenarios if s.option_style == "European"]
        scenarios_to_use = european_scenarios if european_scenarios else payload.scenarios

        total = len(scenarios_to_use)
        avg_apr = sum(s.annualized_return for s in scenarios_to_use) / total if total > 0 else 0.0
        probable_count = sum(1 for s in scenarios_to_use if s.fill_probability > 0)
        max_apr_scenario = max(scenarios_to_use, key=lambda s: s.annualized_return) if scenarios_to_use else None

        return cls(
            total_scenarios=total,
            avg_apr=avg_apr,
            probable_count=probable_count,
            max_apr_scenario=max_apr_scenario
        )
