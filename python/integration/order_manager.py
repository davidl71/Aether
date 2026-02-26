"""
order_manager.py - Order management engine (Python port)

Ported from native/src/order_manager.cpp (739 LOC).

Coordinates interaction with broker APIs. Validates incoming requests,
translates them into single- or multi-leg orders, and tracks execution
statistics.  Live execution requires a broker client; the module provides
a dry-run mode for testing.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any, Callable, Dict, List, Optional

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Enumerations
# ---------------------------------------------------------------------------


class OrderAction(Enum):
    Buy = "BUY"
    Sell = "SELL"


class OrderStatus(Enum):
    Pending = "Pending"
    Submitted = "Submitted"
    PartiallyFilled = "PartiallyFilled"
    Filled = "Filled"
    Cancelled = "Cancelled"
    Rejected = "Rejected"
    Error = "Error"


class TimeInForce(Enum):
    DAY = "DAY"
    GTC = "GTC"
    IOC = "IOC"
    FOK = "FOK"


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class OrderContract:
    """Lightweight contract representation for order placement."""

    symbol: str = ""
    expiry: str = ""
    strike: float = 0.0
    option_type: str = ""  # "C" or "P"

    def is_valid(self) -> bool:
        return self.symbol != ""

    def to_string(self) -> str:
        parts = [self.symbol]
        if self.expiry:
            parts.append(self.expiry)
        if self.strike > 0:
            parts.append(f"{self.strike}")
        if self.option_type:
            parts.append(self.option_type)
        return " ".join(parts)


@dataclass
class Order:
    order_id: int = 0
    contract: OrderContract = field(default_factory=OrderContract)
    action: OrderAction = OrderAction.Buy
    quantity: int = 0
    limit_price: float = 0.0
    tif: TimeInForce = TimeInForce.DAY
    status: OrderStatus = OrderStatus.Pending
    status_message: str = ""
    filled_quantity: int = 0
    avg_fill_price: float = 0.0
    created_time: Optional[datetime] = None
    filled_time: Optional[datetime] = None


@dataclass
class ExecutionResult:
    success: bool = False
    order_ids: List[int] = field(default_factory=list)
    error_message: str = ""
    total_cost: float = 0.0
    execution_time: Optional[datetime] = None


@dataclass
class MultiLegOrder:
    strategy_id: str = ""
    legs: List[Order] = field(default_factory=list)
    status: OrderStatus = OrderStatus.Pending
    legs_filled: int = 0
    total_cost: float = 0.0
    created_time: Optional[datetime] = None

    def is_complete(self) -> bool:
        return self.legs_filled == len(self.legs)

    def is_partially_filled(self) -> bool:
        return 0 < self.legs_filled < len(self.legs)


@dataclass
class OrderStats:
    total_orders_placed: int = 0
    total_orders_filled: int = 0
    total_orders_cancelled: int = 0
    total_orders_rejected: int = 0
    executed_trades: int = 0
    total_volume_traded: float = 0.0
    average_fill_time_ms: float = 0.0
    fill_rate: float = 0.0
    efficiency_ratio: float = 0.0


@dataclass
class BoxSpreadLegSimple:
    """Minimal box-spread leg for order placement."""

    long_call: OrderContract = field(default_factory=OrderContract)
    short_call: OrderContract = field(default_factory=OrderContract)
    long_put: OrderContract = field(default_factory=OrderContract)
    short_put: OrderContract = field(default_factory=OrderContract)
    long_call_price: float = 0.0
    short_call_price: float = 0.0
    long_put_price: float = 0.0
    short_put_price: float = 0.0
    net_debit: float = 0.0


# ---------------------------------------------------------------------------
# OrderManager
# ---------------------------------------------------------------------------


class OrderManager:
    """Manages order lifecycle: validation, placement, tracking, cancellation.

    Set ``dry_run=True`` (default) for paper-mode operation where no orders
    are actually sent to the broker.  A ``client`` object can be injected
    for live execution.
    """

    def __init__(
        self,
        client: Any = None,
        dry_run: bool = True,
        max_order_size: int = 100,
    ):
        self._client = client
        self._dry_run = dry_run
        self._max_order_size = max_order_size
        self._stats = OrderStats()
        self._multi_leg_orders: Dict[str, MultiLegOrder] = {}
        self._next_order_id = 1000
        self._tracked_fills: set = set()
        self._order_update_callback: Optional[Callable] = None
        self._fill_callback: Optional[Callable] = None

    # -- order placement --

    def place_order(
        self,
        contract: OrderContract,
        action: OrderAction,
        quantity: int,
        limit_price: float = 0.0,
        tif: TimeInForce = TimeInForce.DAY,
    ) -> ExecutionResult:
        result = ExecutionResult(execution_time=datetime.now())

        ok, err = self.validate_order(contract, action, quantity, limit_price)
        if not ok:
            result.error_message = err
            return result

        if self._dry_run:
            logger.info(
                "[DRY RUN] Would place order: %s %d %s @ %s",
                action.value,
                quantity,
                contract.to_string(),
                f"{limit_price}" if limit_price > 0 else "MKT",
            )
            oid = self._next_order_id
            self._next_order_id += 1
            result.success = True
            result.order_ids = [oid]
            self._stats.total_orders_placed += 1
            return result

        if self._client is None:
            result.error_message = "No broker client configured"
            return result

        oid = self._client.place_order(contract, action, quantity, limit_price, tif)
        result.success = True
        result.order_ids = [oid]
        self._stats.total_orders_placed += 1
        return result

    def cancel_order(self, order_id: int) -> bool:
        logger.info("Cancelling order #%d", order_id)
        if self._dry_run:
            logger.info("[DRY RUN] Would cancel order #%d", order_id)
            return True
        if self._client:
            self._client.cancel_order(order_id)
        self._stats.total_orders_cancelled += 1
        return True

    def cancel_all_orders(self) -> None:
        logger.info("Cancelling all orders")
        if self._dry_run:
            logger.info("[DRY RUN] Would cancel all orders")
            return
        if self._client:
            self._client.cancel_all_orders()

    def get_order_status(self, order_id: int) -> Optional[Order]:
        if self._client:
            return self._client.get_order(order_id)
        return None

    # -- box spread orders --

    def place_box_spread(
        self,
        spread: BoxSpreadLegSimple,
        strategy_id: str = "",
    ) -> ExecutionResult:
        result = ExecutionResult(execution_time=datetime.now())

        if self._dry_run:
            logger.info("[DRY RUN] Would place box spread (4 legs)")
            ids = list(range(self._next_order_id, self._next_order_id + 4))
            self._next_order_id += 4
            result.success = True
            result.order_ids = ids
            self._stats.total_orders_placed += 4
            return result

        if self._client is None:
            result.error_message = "No broker client configured"
            return result

        order_ids: List[int] = []
        legs = [
            (spread.long_call, OrderAction.Buy, spread.long_call_price),
            (spread.short_call, OrderAction.Sell, spread.short_call_price),
            (spread.long_put, OrderAction.Buy, spread.long_put_price),
            (spread.short_put, OrderAction.Sell, spread.short_put_price),
        ]

        for contract, action, price in legs:
            oid = self._client.place_order(contract, action, 1, price)
            order_ids.append(oid)

        result.success = True
        result.order_ids = order_ids
        result.total_cost = spread.net_debit * 100.0
        self._stats.total_orders_placed += 4
        self._update_efficiency_ratio()

        if strategy_id:
            ml = MultiLegOrder(
                strategy_id=strategy_id,
                status=OrderStatus.Submitted,
                created_time=datetime.now(),
                total_cost=result.total_cost,
            )
            self._multi_leg_orders[strategy_id] = ml

        return result

    def close_box_spread(self, strategy_id: str) -> ExecutionResult:
        result = ExecutionResult(execution_time=datetime.now())
        if self._dry_run:
            logger.info("[DRY RUN] Would close box spread")
            result.success = True
            return result
        result.success = True
        return result

    def get_multi_leg_order(self, strategy_id: str) -> Optional[MultiLegOrder]:
        return self._multi_leg_orders.get(strategy_id)

    def are_all_legs_filled(self, strategy_id: str) -> bool:
        ml = self.get_multi_leg_order(strategy_id)
        return ml.is_complete() if ml else False

    # -- specialty order types --

    def execute_ioc(
        self,
        contract: OrderContract,
        action: OrderAction,
        quantity: int,
        limit_price: float,
    ) -> ExecutionResult:
        return self.place_order(contract, action, quantity, limit_price, TimeInForce.IOC)

    def execute_fok(
        self,
        contract: OrderContract,
        action: OrderAction,
        quantity: int,
        limit_price: float,
    ) -> ExecutionResult:
        return self.place_order(contract, action, quantity, limit_price, TimeInForce.FOK)

    # -- validation --

    def validate_order(
        self,
        contract: OrderContract,
        action: OrderAction,
        quantity: int,
        limit_price: float,
    ) -> tuple[bool, str]:
        if not contract.is_valid():
            return False, "Invalid contract"
        if quantity <= 0:
            return False, "Quantity must be positive"
        if self.exceeds_limits(quantity):
            return False, "Order size exceeds limits"
        if limit_price < 0:
            return False, "Limit price cannot be negative"
        return True, ""

    def exceeds_limits(self, quantity: int) -> bool:
        return quantity > self._max_order_size

    # -- config --

    def set_max_order_size(self, max_contracts: int) -> None:
        self._max_order_size = max_contracts

    def set_dry_run(self, enabled: bool) -> None:
        self._dry_run = enabled

    def is_dry_run(self) -> bool:
        return self._dry_run

    # -- stats --

    def get_stats(self) -> OrderStats:
        return self._stats

    def _update_efficiency_ratio(self) -> None:
        if self._stats.total_orders_placed > 0:
            self._stats.efficiency_ratio = (
                self._stats.executed_trades / self._stats.total_orders_placed
            )
        else:
            self._stats.efficiency_ratio = 0.0

    def track_order_fill(self, order_id: int) -> None:
        if order_id in self._tracked_fills:
            return
        self._stats.total_orders_filled += 1
        self._stats.executed_trades += 1
        self._tracked_fills.add(order_id)
        self._update_efficiency_ratio()
