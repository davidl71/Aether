"""
box_spread_models.py - Data structures for box spread synthetic financing

Extracted from box_spread_strategy.py to separate data definitions from
strategy logic, calculators, and validators.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from typing import List, Optional

from .dte_utils import days_to_expiry_from_yyyymmdd


@dataclass
class OptionContract:
    symbol: str = ""
    expiry: str = ""  # YYYYMMDD
    strike: float = 0.0
    option_type: str = "C"  # "C" or "P"

    def to_string(self) -> str:
        return f"{self.symbol} {self.expiry} {self.strike} {self.option_type}"


@dataclass
class MarketData:
    bid: float = 0.0
    ask: float = 0.0
    last: float = 0.0
    mid: float = 0.0
    bid_size: int = 0
    ask_size: int = 0
    volume: int = 0
    open_interest: int = 0
    timestamp: Optional[datetime] = None

    def get_mid_price(self) -> float:
        if self.bid > 0 and self.ask > 0:
            return (self.bid + self.ask) / 2.0
        return self.last or self.mid

    def get_spread(self) -> float:
        if self.bid > 0 and self.ask > 0:
            return self.ask - self.bid
        return 0.0

    def get_spread_percent(self) -> float:
        mid = self.get_mid_price()
        if mid > 0:
            return (self.get_spread() / mid) * 100.0
        return 0.0

    def is_valid(self) -> bool:
        return self.bid > 0 and self.ask > 0 and self.ask >= self.bid


@dataclass
class OptionEntry:
    contract: OptionContract = field(default_factory=OptionContract)
    market_data: MarketData = field(default_factory=MarketData)
    liquidity_score: float = 0.0

    def is_valid(self) -> bool:
        return self.market_data.is_valid()

    def meets_liquidity_requirements(
        self, min_volume: int = 0, min_open_interest: int = 0
    ) -> bool:
        return (
            self.market_data.volume >= min_volume
            and self.market_data.open_interest >= min_open_interest
        )


@dataclass
class BoxSpreadLeg:
    long_call: OptionContract = field(default_factory=OptionContract)
    short_call: OptionContract = field(default_factory=OptionContract)
    long_put: OptionContract = field(default_factory=OptionContract)
    short_put: OptionContract = field(default_factory=OptionContract)

    long_call_price: float = 0.0
    short_call_price: float = 0.0
    long_put_price: float = 0.0
    short_put_price: float = 0.0

    long_call_bid_ask_spread: float = 0.0
    short_call_bid_ask_spread: float = 0.0
    long_put_bid_ask_spread: float = 0.0
    short_put_bid_ask_spread: float = 0.0

    net_debit: float = 0.0
    theoretical_value: float = 0.0
    arbitrage_profit: float = 0.0
    roi_percent: float = 0.0

    buy_net_debit: float = 0.0
    buy_profit: float = 0.0
    buy_implied_rate: float = 0.0
    sell_net_credit: float = 0.0
    sell_profit: float = 0.0
    sell_implied_rate: float = 0.0
    buy_sell_disparity: float = 0.0
    put_call_parity_violation: float = 0.0

    def get_strike_width(self) -> float:
        return self.short_call.strike - self.long_call.strike

    def get_days_to_expiry(self) -> int:
        if not self.long_call.expiry:
            return 0
        return days_to_expiry_from_yyyymmdd(self.long_call.expiry)

    def is_valid(self) -> bool:
        return (
            self.long_call.symbol != ""
            and self.short_call.symbol != ""
            and self.long_put.symbol != ""
            and self.short_put.symbol != ""
            and self.get_strike_width() > 0
        )


@dataclass
class CommissionConfig:
    per_contract_fee: float = 0.65
    minimum_order_fee: float = 1.00
    tier: str = "standard"  # standard, lite, pro

    def get_effective_rate(self) -> float:
        rates = {"standard": 0.65, "lite": 0.50, "pro": 0.15}
        return rates.get(self.tier, self.per_contract_fee)


@dataclass
class BoxSpreadOpportunity:
    spread: BoxSpreadLeg = field(default_factory=BoxSpreadLeg)
    expected_profit: float = 0.0
    confidence_score: float = 0.0
    risk_adjusted_return: float = 0.0
    liquidity_score: float = 0.0
    execution_probability: float = 0.0
    discovered_time: Optional[datetime] = None

    def is_actionable(self) -> bool:
        return (
            self.confidence_score >= 50.0
            and self.expected_profit > 0
            and self.execution_probability >= 0.7
        )


@dataclass
class StrategyParams:
    symbols: List[str] = field(default_factory=list)
    min_days_to_expiry: int = 7
    max_days_to_expiry: int = 180
    min_volume: int = 0
    min_open_interest: int = 0
    max_bid_ask_spread: float = 0.50
    min_arbitrage_profit: float = 0.10
    min_roi_percent: float = 0.01
    max_positions: int = 10
    max_total_exposure: float = 50000.0


@dataclass
class StrategyStats:
    total_opportunities_found: int = 0
    total_trades_executed: int = 0
    successful_trades: int = 0
    failed_trades: int = 0
    start_time: Optional[datetime] = None


@dataclass
class YieldCurvePoint:
    symbol: str = ""
    days_to_expiry: int = 0
    expiry_date: str = ""
    strike_width: float = 0.0
    implied_rate: float = 0.0
    effective_rate: float = 0.0
    net_debit: float = 0.0
    spread_bps: float = 0.0
    liquidity_score: float = 0.0
    timestamp: Optional[datetime] = None

    def is_valid(self) -> bool:
        return self.symbol != "" and self.days_to_expiry > 0 and self.strike_width > 0


@dataclass
class YieldCurve:
    symbol: str = ""
    strike_width: float = 0.0
    benchmark_rate: float = 0.0
    points: List[YieldCurvePoint] = field(default_factory=list)
    generated_time: Optional[datetime] = None

    def is_valid(self) -> bool:
        return self.symbol != "" and self.strike_width > 0 and len(self.points) > 0

    def sort_by_dte(self) -> None:
        self.points.sort(key=lambda p: p.days_to_expiry)


@dataclass
class BagPosition:
    quantity: int = 0
    entry_price: float = 0.0
    current_price: float = 0.0
    cost_basis: float = 0.0
    unrealized_pnl: float = 0.0


@dataclass
class BagCandle:
    open: float = 0.0
    high: float = 0.0
    low: float = 0.0
    close: float = 0.0
    entry: float = 0.0
    volume: float = 0.0
    period_start: Optional[datetime] = None
    period_end: Optional[datetime] = None
    updated: Optional[datetime] = None


@dataclass
class BagGreeks:
    delta: float = 0.0
    gamma: float = 0.0
    theta: float = 0.0
    vega: float = 0.0
    rho: float = 0.0
    calculated_at: Optional[datetime] = None


@dataclass
class BoxSpreadBag:
    spread: BoxSpreadLeg = field(default_factory=BoxSpreadLeg)
    complex_symbol: str = ""
    cboe_symbol: str = ""
    theoretical_value: float = 0.0
    net_debit: float = 0.0
    implied_rate: float = 0.0
    days_to_expiry: int = 0
    market_data: MarketData = field(default_factory=MarketData)
    position: BagPosition = field(default_factory=BagPosition)
    candle: BagCandle = field(default_factory=BagCandle)
    greeks: BagGreeks = field(default_factory=BagGreeks)
    candle_history: List[BagCandle] = field(default_factory=list)
    created_at: Optional[datetime] = None
    last_updated: Optional[datetime] = None

    def is_valid(self) -> bool:
        return (
            self.complex_symbol != ""
            and self.cboe_symbol != ""
            and self.spread.is_valid()
            and self.days_to_expiry > 0
        )

    def update_candle(self, price: float, volume: float = 1.0) -> None:
        now = datetime.now()
        if self.candle.period_start is None:
            self.candle = BagCandle(
                open=price, high=price, low=price, close=price,
                entry=self.position.entry_price if self.position.entry_price > 0 else price,
                volume=volume, period_start=now, period_end=now, updated=now,
            )
        else:
            self.candle.high = max(self.candle.high, price)
            self.candle.low = min(self.candle.low, price)
            self.candle.close = price
            self.candle.volume += volume
            self.candle.period_end = now
            self.candle.updated = now

    def reset_candle(self) -> None:
        if self.candle.period_start is not None:
            self.candle_history.append(self.candle)
            if len(self.candle_history) > 100:
                self.candle_history = self.candle_history[-100:]
        self.candle = BagCandle()

    def get_current_pnl(self) -> float:
        if self.position.quantity == 0:
            return 0.0
        current_value = self.market_data.get_mid_price() * self.position.quantity
        return current_value - self.position.cost_basis

    def get_pnl_per_contract(self) -> float:
        if self.position.quantity == 0:
            return 0.0
        return self.get_current_pnl() / abs(self.position.quantity)

    @staticmethod
    def generate_cboe_symbol(
        underlying: str, expiry: str, strike_low: float, strike_high: float
    ) -> str:
        return f"{underlying} {expiry} {int(strike_low)}/{int(strike_high)} BOX"
