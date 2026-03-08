"""
box_spread_strategy.py - Box spread synthetic financing strategy (Python port)

Ported from native/src/strategies/box_spread/box_spread_strategy.cpp (1554 LOC)
and native/src/strategies/box_spread/box_spread_bag.cpp (315 LOC).

Implements the decision logic for identifying, validating, and evaluating
box spread synthetic financing opportunities. Extracts implied risk-free rates
for lending/borrowing by combining option chain data with benchmark rates.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any, List, Optional, Tuple

from .dte_utils import days_to_expiry_from_yyyymmdd

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


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
                open=price,
                high=price,
                low=price,
                close=price,
                entry=self.position.entry_price if self.position.entry_price > 0 else price,
                volume=volume,
                period_start=now,
                period_end=now,
                updated=now,
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


# ---------------------------------------------------------------------------
# BoxSpreadCalculator - static calculation helpers
# ---------------------------------------------------------------------------


class BoxSpreadCalculator:
    """Pure-function calculator for box spread metrics."""

    @staticmethod
    def calculate_theoretical_value(spread: BoxSpreadLeg) -> float:
        return spread.get_strike_width()

    @staticmethod
    def calculate_net_debit(spread: BoxSpreadLeg) -> float:
        return (
            spread.long_call_price
            - spread.short_call_price
            + spread.long_put_price
            - spread.short_put_price
        )

    @staticmethod
    def calculate_max_profit(spread: BoxSpreadLeg) -> float:
        return spread.theoretical_value - spread.net_debit

    @staticmethod
    def calculate_max_loss(spread: BoxSpreadLeg) -> float:
        return -spread.net_debit if spread.net_debit < 0 else 0.0

    @staticmethod
    def calculate_roi(spread: BoxSpreadLeg) -> float:
        if spread.net_debit > 0:
            return (BoxSpreadCalculator.calculate_max_profit(spread) / spread.net_debit) * 100.0
        return 0.0

    @staticmethod
    def calculate_breakeven(spread: BoxSpreadLeg) -> float:
        return spread.long_call.strike

    @staticmethod
    def calculate_commission(spread: BoxSpreadLeg, per_contract_fee: float = 0.65) -> float:
        return 4.0 * per_contract_fee

    @staticmethod
    def calculate_commission_ibkr_pro(
        spread: BoxSpreadLeg, config: CommissionConfig = None
    ) -> float:
        if config is None:
            config = CommissionConfig()
        rate = config.get_effective_rate()
        base = 4.0 * rate
        if base < config.minimum_order_fee:
            base = config.minimum_order_fee
        return base

    @staticmethod
    def calculate_total_cost(
        spread: BoxSpreadLeg, per_contract_fee: float = 0.65
    ) -> float:
        return spread.net_debit + BoxSpreadCalculator.calculate_commission(
            spread, per_contract_fee
        )

    @staticmethod
    def calculate_implied_interest_rate(spread: BoxSpreadLeg) -> float:
        """Annualised implied rate embedded in the box spread.

        Borrowing (net debit > 0):
            rate = ((net_debit - strike_width) / strike_width) * (365 / dte) * 100
        Lending (net credit, net_debit < 0):
            rate = ((strike_width - net_credit) / net_credit) * (365 / dte) * 100
        """
        strike_width = spread.get_strike_width()
        dte = spread.get_days_to_expiry()
        if dte <= 0:
            return 0.0

        net_cost = spread.net_debit
        if net_cost > 0:
            return ((net_cost - strike_width) / strike_width) * (365.0 / dte) * 100.0
        elif net_cost < 0:
            net_credit = -net_cost
            return ((strike_width - net_credit) / net_credit) * (365.0 / dte) * 100.0
        return 0.0

    @staticmethod
    def calculate_effective_interest_rate(
        spread: BoxSpreadLeg, per_contract_fee: float = 0.65
    ) -> float:
        strike_width = spread.get_strike_width()
        dte = spread.get_days_to_expiry()
        if dte <= 0:
            return 0.0

        commission = BoxSpreadCalculator.calculate_commission(spread, per_contract_fee)
        total_cost = spread.net_debit + commission
        if total_cost > 0:
            return ((total_cost - strike_width) / strike_width) * (365.0 / dte) * 100.0
        elif total_cost < 0:
            nc = -total_cost
            return ((strike_width - nc) / nc) * (365.0 / dte) * 100.0
        return 0.0

    @staticmethod
    def compare_to_benchmark(
        spread: BoxSpreadLeg,
        benchmark_rate_percent: float,
        per_contract_fee: float = 0.65,
    ) -> float:
        """Return advantage in basis points (positive = box spread beats benchmark)."""
        effective = BoxSpreadCalculator.calculate_effective_interest_rate(
            spread, per_contract_fee
        )
        spread_bps = (effective - benchmark_rate_percent) * 100.0
        if spread.net_debit > 0:
            return -spread_bps
        return spread_bps

    @staticmethod
    def calculate_buy_net_debit(
        spread: BoxSpreadLeg,
        long_call_ask: float,
        short_call_bid: float,
        long_put_ask: float,
        short_put_bid: float,
    ) -> float:
        return long_call_ask - short_call_bid + long_put_ask - short_put_bid

    @staticmethod
    def calculate_sell_net_credit(
        spread: BoxSpreadLeg,
        long_call_bid: float,
        short_call_ask: float,
        long_put_bid: float,
        short_put_ask: float,
    ) -> float:
        return long_call_bid - short_call_ask + long_put_bid - short_put_ask

    @staticmethod
    def calculate_buy_sell_disparity(buy_profit: float, sell_profit: float) -> float:
        return buy_profit - sell_profit

    @staticmethod
    def calculate_put_call_parity_violation(
        spread: BoxSpreadLeg,
        call_implied_rate: float,
        put_implied_rate: float,
    ) -> float:
        return (call_implied_rate - put_implied_rate) * 100.0  # bps


# ---------------------------------------------------------------------------
# BoxSpreadValidator
# ---------------------------------------------------------------------------


class BoxSpreadValidator:
    """Validates box spread structure and pricing."""

    @staticmethod
    def validate_structure(spread: BoxSpreadLeg) -> bool:
        return spread.is_valid()

    @staticmethod
    def validate_strikes(spread: BoxSpreadLeg) -> bool:
        return (
            spread.long_call.strike < spread.short_call.strike
            and spread.short_put.strike == spread.long_call.strike
            and spread.long_put.strike == spread.short_call.strike
        )

    @staticmethod
    def validate_expiries(spread: BoxSpreadLeg) -> bool:
        return (
            spread.long_call.expiry == spread.short_call.expiry
            and spread.long_call.expiry == spread.long_put.expiry
            and spread.long_call.expiry == spread.short_put.expiry
        )

    @staticmethod
    def validate_symbols(spread: BoxSpreadLeg) -> bool:
        return (
            spread.long_call.symbol == spread.short_call.symbol
            and spread.long_call.symbol == spread.long_put.symbol
            and spread.long_call.symbol == spread.short_put.symbol
        )

    @staticmethod
    def validate_pricing(spread: BoxSpreadLeg) -> bool:
        return (
            spread.net_debit > 0
            and spread.theoretical_value > 0
            and spread.net_debit < spread.theoretical_value
        )

    @staticmethod
    def validate(spread: BoxSpreadLeg) -> Tuple[bool, List[str]]:
        errors: List[str] = []
        if not BoxSpreadValidator.validate_structure(spread):
            errors.append("Invalid spread structure")
        if not BoxSpreadValidator.validate_strikes(spread):
            errors.append("Invalid strike configuration")
        if not BoxSpreadValidator.validate_expiries(spread):
            errors.append("Expiries do not match")
        if not BoxSpreadValidator.validate_symbols(spread):
            errors.append("Symbols do not match")
        if not BoxSpreadValidator.validate_pricing(spread):
            errors.append("Invalid pricing")

        strike_width = spread.get_strike_width()
        if abs(spread.theoretical_value - strike_width) > 0.01:
            errors.append(
                f"Theoretical value must equal strike width "
                f"(theoretical={spread.theoretical_value}, strike_width={strike_width})"
            )

        max_threshold = 0.50
        for label, val in [
            ("Long call", spread.long_call_bid_ask_spread),
            ("Short call", spread.short_call_bid_ask_spread),
            ("Long put", spread.long_put_bid_ask_spread),
            ("Short put", spread.short_put_bid_ask_spread),
        ]:
            if val > max_threshold:
                errors.append(f"{label} bid/ask spread too wide: {val}")

        if any(
            p <= 0
            for p in [
                spread.long_call_price,
                spread.short_call_price,
                spread.long_put_price,
                spread.short_put_price,
            ]
        ):
            errors.append("All option prices must be positive")

        return (len(errors) == 0, errors)


# ---------------------------------------------------------------------------
# BoxSpreadBagManager
# ---------------------------------------------------------------------------


class BoxSpreadBagManager:
    """Creates and manages BoxSpreadBag composite instruments."""

    @staticmethod
    def create_bag_from_spread(
        spread: BoxSpreadLeg, underlying_symbol: str
    ) -> BoxSpreadBag:
        now = datetime.now()
        bag = BoxSpreadBag(
            spread=spread,
            complex_symbol=f"{underlying_symbol} BOX",
            cboe_symbol=BoxSpreadBag.generate_cboe_symbol(
                underlying_symbol,
                spread.long_call.expiry,
                spread.long_call.strike,
                spread.short_call.strike,
            ),
            theoretical_value=spread.theoretical_value,
            net_debit=spread.net_debit,
            days_to_expiry=spread.get_days_to_expiry(),
            implied_rate=BoxSpreadCalculator.calculate_implied_interest_rate(spread),
            market_data=MarketData(last=spread.net_debit, mid=spread.net_debit),
            position=BagPosition(entry_price=spread.net_debit, current_price=spread.net_debit),
            candle=BagCandle(
                entry=spread.net_debit,
                open=spread.net_debit,
                high=spread.net_debit,
                low=spread.net_debit,
                close=spread.net_debit,
            ),
            created_at=now,
            last_updated=now,
        )
        bag.market_data.timestamp = now
        bag.greeks.calculated_at = now
        return bag

    @staticmethod
    def update_bag_market_data(
        bag: BoxSpreadBag,
        bid: float,
        ask: float,
        last: float,
        bid_size: int = 0,
        ask_size: int = 0,
    ) -> None:
        bag.market_data.bid = bid
        bag.market_data.ask = ask
        bag.market_data.last = last if last > 0 else bag.market_data.get_mid_price()
        bag.market_data.bid_size = bid_size
        bag.market_data.ask_size = ask_size
        bag.market_data.timestamp = datetime.now()

        mid = bag.market_data.get_mid_price()
        bag.update_candle(mid)

        if bag.position.quantity != 0:
            bag.position.current_price = mid
            bag.position.unrealized_pnl = bag.get_current_pnl()

        bag.last_updated = datetime.now()


# ---------------------------------------------------------------------------
# BoxSpreadStrategy
# ---------------------------------------------------------------------------


class BoxSpreadStrategy:
    """Identifies, validates, and (optionally) executes box-spread opportunities.

    This is a Python port of the C++ ``BoxSpreadStrategy`` class.  The live-market
    methods (``find_box_spreads``, ``execute_box_spread``) remain stubs because
    they require a TWS client and order manager.  The pure-math evaluation and
    scoring logic is fully functional and testable.
    """

    def __init__(self, params: Optional[StrategyParams] = None):
        self.params = params or StrategyParams()
        self.stats = StrategyStats(start_time=datetime.now())
        self._positions: List[Any] = []

    # -- evaluation helpers (fully ported) --

    def is_profitable(self, spread: BoxSpreadLeg) -> bool:
        profit = BoxSpreadCalculator.calculate_max_profit(spread)
        roi = BoxSpreadCalculator.calculate_roi(spread)
        return (
            profit >= self.params.min_arbitrage_profit
            and roi >= self.params.min_roi_percent
        )

    def calculate_confidence_score(
        self,
        spread: BoxSpreadLeg,
        *,
        extra_dte: Optional[int] = None,
    ) -> float:
        """4-component scoring algorithm (0-100).

        Components:
          1. Bid-ask spread tightness  (0-30)
          2. Pricing efficiency         (0-25)
          3. Buy/sell disparity         (0-20)
          4. Put-call parity adherence  (0-15)
          5. DTE appropriateness        (0-10)
        """
        score = 0.0

        avg_spread = (
            spread.long_call_bid_ask_spread
            + spread.short_call_bid_ask_spread
            + spread.long_put_bid_ask_spread
            + spread.short_put_bid_ask_spread
        ) / 4.0
        if avg_spread <= 0.02:
            score += 30.0
        elif avg_spread <= 0.05:
            score += 25.0
        elif avg_spread <= 0.10:
            score += 20.0
        elif avg_spread <= 0.20:
            score += 12.0
        elif avg_spread <= 0.50:
            score += 5.0

        strike_width = spread.get_strike_width()
        if strike_width > 0:
            ratio = spread.net_debit / strike_width
            if 0.95 <= ratio <= 1.005:
                score += 25.0
            elif 0.90 <= ratio <= 1.01:
                score += 20.0
            elif ratio >= 0.80:
                score += 10.0
            else:
                score += 2.0

        disparity = abs(spread.buy_sell_disparity)
        if disparity <= 0.02:
            score += 20.0
        elif disparity <= 0.05:
            score += 15.0
        elif disparity <= 0.10:
            score += 10.0
        elif disparity <= 0.25:
            score += 5.0

        parity_violation_bps = abs(spread.put_call_parity_violation)
        if parity_violation_bps <= 5.0:
            score += 15.0
        elif parity_violation_bps <= 15.0:
            score += 10.0
        elif parity_violation_bps <= 50.0:
            score += 5.0

        dte = extra_dte if extra_dte is not None else spread.get_days_to_expiry()
        if 14 <= dte <= 90:
            score += 10.0
        elif 7 <= dte <= 180:
            score += 7.0
        elif dte > 0:
            score += 3.0

        return min(score, 100.0)

    def evaluate_box_spread(
        self,
        long_call_entry: OptionEntry,
        short_call_entry: OptionEntry,
        long_put_entry: OptionEntry,
        short_put_entry: OptionEntry,
    ) -> Optional[BoxSpreadOpportunity]:
        """Build and evaluate a box spread from four option entries."""
        if not all(
            e.is_valid()
            for e in [long_call_entry, short_call_entry, long_put_entry, short_put_entry]
        ):
            return None

        max_spread = self.params.max_bid_ask_spread
        if any(
            e.market_data.get_spread() > max_spread
            for e in [long_call_entry, short_call_entry, long_put_entry, short_put_entry]
        ):
            return None

        spread = BoxSpreadLeg(
            long_call=long_call_entry.contract,
            short_call=short_call_entry.contract,
            long_put=long_put_entry.contract,
            short_put=short_put_entry.contract,
            long_call_price=long_call_entry.market_data.get_mid_price(),
            short_call_price=short_call_entry.market_data.get_mid_price(),
            long_put_price=long_put_entry.market_data.get_mid_price(),
            short_put_price=short_put_entry.market_data.get_mid_price(),
            long_call_bid_ask_spread=long_call_entry.market_data.get_spread(),
            short_call_bid_ask_spread=short_call_entry.market_data.get_spread(),
            long_put_bid_ask_spread=long_put_entry.market_data.get_spread(),
            short_put_bid_ask_spread=short_put_entry.market_data.get_spread(),
        )

        spread.net_debit = BoxSpreadCalculator.calculate_net_debit(spread)
        spread.theoretical_value = BoxSpreadCalculator.calculate_theoretical_value(spread)
        spread.arbitrage_profit = BoxSpreadCalculator.calculate_max_profit(spread)
        spread.roi_percent = BoxSpreadCalculator.calculate_roi(spread)

        spread.buy_net_debit = BoxSpreadCalculator.calculate_buy_net_debit(
            spread,
            long_call_entry.market_data.ask,
            short_call_entry.market_data.bid,
            long_put_entry.market_data.ask,
            short_put_entry.market_data.bid,
        )
        dte = spread.get_days_to_expiry()
        sw = spread.get_strike_width()
        spread.buy_profit = spread.theoretical_value - spread.buy_net_debit
        if dte > 0 and spread.buy_net_debit > 0:
            spread.buy_implied_rate = (
                (spread.buy_net_debit - sw) / sw
            ) * (365.0 / dte) * 100.0
        else:
            spread.buy_implied_rate = 0.0

        spread.sell_net_credit = BoxSpreadCalculator.calculate_sell_net_credit(
            spread,
            long_call_entry.market_data.bid,
            short_call_entry.market_data.ask,
            long_put_entry.market_data.bid,
            short_put_entry.market_data.ask,
        )
        spread.sell_profit = spread.sell_net_credit - spread.theoretical_value
        if dte > 0 and spread.sell_net_credit > 0:
            spread.sell_implied_rate = (
                (sw - spread.sell_net_credit) / spread.sell_net_credit
            ) * (365.0 / dte) * 100.0
        else:
            spread.sell_implied_rate = 0.0

        spread.buy_sell_disparity = BoxSpreadCalculator.calculate_buy_sell_disparity(
            spread.buy_profit, spread.sell_profit
        )
        spread.put_call_parity_violation = (
            BoxSpreadCalculator.calculate_put_call_parity_violation(
                spread, spread.buy_implied_rate, spread.sell_implied_rate
            )
        )

        valid, errors = BoxSpreadValidator.validate(spread)
        if not valid:
            logger.debug("Validation failed: %s", errors[0] if errors else "unknown")
            return None

        if not self.is_profitable(spread):
            return None

        avg_spread_pct = sum(
            e.market_data.get_spread_percent()
            for e in [long_call_entry, short_call_entry, long_put_entry, short_put_entry]
        ) / 4.0

        avg_liq = sum(
            e.liquidity_score
            for e in [long_call_entry, short_call_entry, long_put_entry, short_put_entry]
        ) / 4.0

        opp = BoxSpreadOpportunity(
            spread=spread,
            expected_profit=spread.arbitrage_profit,
            confidence_score=self.calculate_confidence_score(spread, extra_dte=dte),
            risk_adjusted_return=spread.roi_percent,
            liquidity_score=avg_liq,
            execution_probability=min(
                1.0,
                (avg_liq / 100.0) * (1.0 - min(avg_spread_pct / 10.0, 1.0)),
            ),
            discovered_time=datetime.now(),
        )
        return opp

    def beats_benchmark(
        self,
        spread: BoxSpreadLeg,
        benchmark_rate_percent: float,
        min_spread_bps: float = 0.0,
    ) -> bool:
        bps = BoxSpreadCalculator.compare_to_benchmark(spread, benchmark_rate_percent)
        return bps >= min_spread_bps

    def build_yield_curve(
        self,
        opportunities: List[BoxSpreadOpportunity],
        symbol: str,
        strike_width: float,
        benchmark_rate_percent: float = 0.0,
        min_dte: int = 0,
        max_dte: int = 365,
    ) -> YieldCurve:
        """Build a yield curve from pre-computed opportunities."""
        curve = YieldCurve(
            symbol=symbol,
            strike_width=strike_width,
            benchmark_rate=benchmark_rate_percent,
            generated_time=datetime.now(),
        )

        for opp in opportunities:
            opp_sw = opp.spread.get_strike_width()
            dte = opp.spread.get_days_to_expiry()
            if abs(opp_sw - strike_width) > 0.01:
                continue
            if dte < min_dte or dte > max_dte:
                continue

            implied = BoxSpreadCalculator.calculate_implied_interest_rate(opp.spread)
            effective = BoxSpreadCalculator.calculate_effective_interest_rate(opp.spread)
            spread_bps = BoxSpreadCalculator.compare_to_benchmark(
                opp.spread, benchmark_rate_percent
            )

            curve.points.append(
                YieldCurvePoint(
                    symbol=symbol,
                    days_to_expiry=dte,
                    expiry_date=opp.spread.long_call.expiry,
                    strike_width=opp_sw,
                    implied_rate=implied,
                    effective_rate=effective,
                    net_debit=opp.spread.net_debit,
                    spread_bps=spread_bps,
                    liquidity_score=opp.liquidity_score,
                    timestamp=datetime.now(),
                )
            )

        curve.sort_by_dte()
        return curve


# ---------------------------------------------------------------------------
# Free-standing helpers (match C++ free functions)
# ---------------------------------------------------------------------------


def sort_opportunities_by_profit(
    opportunities: List[BoxSpreadOpportunity],
) -> List[BoxSpreadOpportunity]:
    return sorted(opportunities, key=lambda o: o.expected_profit, reverse=True)


def sort_opportunities_by_confidence(
    opportunities: List[BoxSpreadOpportunity],
) -> List[BoxSpreadOpportunity]:
    return sorted(opportunities, key=lambda o: o.confidence_score, reverse=True)


def filter_by_min_profit(
    opportunities: List[BoxSpreadOpportunity], min_profit: float
) -> List[BoxSpreadOpportunity]:
    return [o for o in opportunities if o.expected_profit >= min_profit]


def filter_by_min_roi(
    opportunities: List[BoxSpreadOpportunity], min_roi_percent: float
) -> List[BoxSpreadOpportunity]:
    return [o for o in opportunities if o.spread.roi_percent >= min_roi_percent]
